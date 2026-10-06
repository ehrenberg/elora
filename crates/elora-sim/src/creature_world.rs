//! Kreaturen in der Welt (A1.2): Verhalten, Bewegung, Treffer, Berührung, Geschosse, Beute.

use crate::character::PHYS_SIZE;
use crate::collision::{Collision, Tile};
use crate::creature::{
    Behavior, Creature, CreatureShot, DiverDef, HookTarget, Loot, SerpentDef, WardenDef, rng,
    rng_f32,
};
use crate::event::CreatureAct;
use crate::event::{DeathCause, Event};
use crate::math::Vec2;
use crate::tuning::ms_to_ticks;
use crate::world::World;

/// Kantenlänge der Beute-Box.
const LOOT_SIZE: f32 = 12.0;
/// Beute ist erst nach dieser Zeit einsammelbar (Ticks), damit sie sichtbar herausspringt.
const LOOT_DELAY: u32 = 20;
/// Flugtempo der Beute zu Elora (Einheiten/Tick).
const LOOT_MAGNET_SPEED: f32 = 12.0;
/// Höchstens so viele einzelne Stücke je Eintrag (der Rest wird zusammengelegt).
const LOOT_PIECES: u32 = 8;
/// Radius eines Gegner-Geschosses.
const SHOT_RADIUS: f32 = 8.0;
/// So lange bleibt eine Hälfte der Hook-Blüten welk (Ticks, 2,5 s).
const WILT_TICKS: u64 = 125;
/// Lebensdauer eines Gegner-Geschosses (Ticks).
const SHOT_LIFETIME: u32 = 150;

fn sign(v: f32) -> i8 {
    if v < 0.0 { -1 } else { 1 }
}

/// Steht etwas Festes (oder eine Plattform) unter dem Punkt?
fn floor_at(col: &Collision, p: Vec2) -> bool {
    let t = col.tile_at(p);
    t.is_solid() || t == Tile::Platform
}

fn grounded(col: &Collision, pos: Vec2, size: Vec2) -> bool {
    let y = pos.y + size.y / 2.0 + 2.0;
    floor_at(col, Vec2::new(pos.x - size.x / 2.0 + 1.0, y))
        || floor_at(col, Vec2::new(pos.x + size.x / 2.0 - 1.0, y))
}

impl World {
    /// Hook-Blüten welken abwechselnd, solange ein Hüter aus der Luft wütend ist (E-298).
    pub(crate) fn update_hook_wilt(&mut self) {
        let angry = self.creatures.iter().any(|c| {
            let k = &self.creature_kinds[c.kind];
            match &k.behavior {
                Behavior::Diver(d) => {
                    #[allow(clippy::cast_precision_loss)]
                    let life = c.health as f32 / k.health.max(1) as f32;
                    c.mode != crate::creature::diver::SLEEP
                        && d.enrage_at > 0.0
                        && life <= d.enrage_at
                }
                _ => false,
            }
        });
        self.collision.hook_wilt = angry.then(|| (self.tick / WILT_TICKS).is_multiple_of(2));
    }

    /// Index einer Gegnerart nach Name.
    pub fn creature_kind(&self, name: &str) -> Option<usize> {
        self.creature_kinds.iter().position(|k| k.name == name)
    }

    /// Setzt einen Gegner der Art `kind` an `pos`; liefert seine Id.
    pub fn add_creature(&mut self, kind: usize, pos: Vec2) -> Option<u32> {
        let k = self.creature_kinds.get(kind)?;
        let id = self.next_id;
        self.next_id += 1;
        self.creatures.push(Creature {
            id,
            kind,
            pos,
            vel: Vec2::ZERO,
            home: pos,
            health: k.health,
            facing: -1,
            timer: 0,
            stun: 0,
            hit_tick: None,
            grounded: false,
            mode: 0,
            goal: pos,
            count: match &k.behavior {
                Behavior::Warden(d) => d.cores,
                _ => 0,
            },
            tug: 0,
            wall_timer: 0,
            hits: 0,
        });
        Some(id)
    }

    pub(crate) fn hook_targets(&self) -> Vec<HookTarget> {
        self.creatures
            .iter()
            .filter(|c| c.hookable(&self.creature_kinds[c.kind]))
            .map(|c| {
                let k = &self.creature_kinds[c.kind];
                HookTarget {
                    id: c.id,
                    pos: c.pos,
                    radius: k.radius(),
                    small: k.small,
                    anchor: matches!(k.behavior, Behavior::Warden(_)),
                }
            })
            .collect()
    }

    fn creature_index(&self, id: u32) -> Option<usize> {
        self.creatures.iter().position(|c| c.id == id)
    }

    /// Erster Gegner auf der Strecke `from`–`to`; liefert Index und Schnittpunkt.
    pub(crate) fn intersect_creature(
        &self,
        from: Vec2,
        to: Vec2,
        radius: f32,
    ) -> Option<(usize, Vec2)> {
        let mut best: Option<(usize, Vec2, f32)> = None;
        for (i, c) in self.creatures.iter().enumerate() {
            let r = self.creature_kinds[c.kind].radius();
            let at = Vec2::closest_point_on_segment(from, to, c.pos);
            if c.pos.distance(at) < r + radius {
                let d = from.distance(at);
                if best.is_none_or(|b| d < b.2) {
                    best = Some((i, at, d));
                }
            }
        }
        best.map(|(i, at, _)| (i, at))
    }

    /// Gegner auf der Strecke `from`–`to`, nach Entfernung sortiert (Id, Schnittpunkt).
    pub(crate) fn creatures_on_line(&self, from: Vec2, to: Vec2) -> Vec<(u32, Vec2)> {
        let mut hits: Vec<(u32, Vec2, f32)> = self
            .creatures
            .iter()
            .filter_map(|c| {
                let r = self.creature_kinds[c.kind].radius();
                let at = Vec2::closest_point_on_segment(from, to, c.pos);
                (c.pos.distance(at) < r).then(|| (c.id, at, from.distance(at)))
            })
            .collect();
        hits.sort_by(|a, b| a.2.total_cmp(&b.2));
        hits.into_iter().map(|(id, at, _)| (id, at)).collect()
    }

    /// Splitter (A-20): kleine Nach-Explosionen um den Einschlag, nur gegen Gegner.
    pub(crate) fn grenade_shards(&mut self, pos: Vec2, owner: usize, max_damage: i32) {
        let n = self.tuning.grenade_shards;
        for k in 0..n {
            #[allow(clippy::cast_precision_loss)]
            let a = std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::TAU / n as f32;
            let p = pos + Vec2::new(a.cos(), a.sin()) * 48.0;
            self.events.push(Event::Explosion { owner, pos: p });
            self.explode_creatures(p, pos, owner, (max_damage / 3).max(1));
        }
    }

    /// Hammerschlag auf Gegner; liefert die Anzahl der Treffer. Mit Schockwelle (A-19)
    /// trifft er alle Gegner um Elora.
    pub(crate) fn hammer_creatures(&mut self, owner: usize, pos: Vec2, start: Vec2) -> usize {
        let (reach, shockwave) = (self.tuning.hammer_reach, self.tuning.hammer_shockwave);
        let hits: Vec<(u32, Vec2)> = self
            .creatures
            .iter()
            .filter(|c| {
                let r = self.creature_kinds[c.kind].radius();
                let near = if shockwave {
                    c.pos.distance(pos) < reach + PHYS_SIZE * 2.0 + r
                } else {
                    c.pos.distance(start) < reach + r
                };
                near && self.collision.intersect_line(pos, c.pos).is_none()
            })
            .map(|c| (c.id, c.pos))
            .collect();
        let (knockback, damage) = (self.tuning.hammer_knockback, self.tuning.hammer_damage);
        for &(id, p) in &hits {
            self.events.push(Event::HammerHit { owner, pos: p });
            let dir = if p.distance(pos) > 0.0 {
                (p - pos).normalize()
            } else {
                Vec2::new(0.0, -1.0)
            };
            let force = (dir + Vec2::new(0.0, -1.1)).normalize() * knockback;
            if let Some(i) = self.creature_index(id) {
                let stun = ms_to_ticks(self.tuning.hammer_stun);
                self.creatures[i].stun = self.creatures[i].stun.max(stun);
                self.damage_creature(i, force, damage, Some(owner), Some(pos));
            }
        }
        hits.len()
    }

    /// Explosion trifft Gegner (wie Figuren, ohne Rüstung); `src` ist der Einschlag (Panzer).
    pub(crate) fn explode_creatures(
        &mut self,
        pos: Vec2,
        src: Vec2,
        owner: usize,
        max_damage: i32,
    ) {
        let t = &self.tuning;
        let (radius, inner, max_force) = (
            t.explosion_radius,
            t.explosion_inner_radius,
            t.explosion_max_force,
        );
        let hits: Vec<(u32, Vec2, i32)> = self
            .creatures
            .iter()
            .filter_map(|c| {
                let r = self.creature_kinds[c.kind].radius();
                let diff = c.pos - pos;
                let l = diff.length();
                if l >= radius + r {
                    return None;
                }
                let force = if l > 0.0 {
                    diff.normalize() * max_force
                } else {
                    Vec2::new(0.0, -max_force)
                };
                let factor = 1.0 - ((l - inner) / (radius - inner)).clamp(0.0, 1.0);
                #[allow(clippy::cast_possible_truncation)]
                let damage = (factor * max_damage as f32) as i32;
                (damage != 0).then_some((c.id, force * factor, damage))
            })
            .collect();
        for (id, force, damage) in hits {
            if let Some(i) = self.creature_index(id) {
                self.damage_creature(i, force, damage, Some(owner), Some(src));
            }
        }
    }

    /// Stoßwelle des Stampfens: Schaden und Betäubung (A-13, A-14).
    pub(crate) fn stomp_creatures(&mut self, player: usize, pos: Vec2) {
        let r = self.tuning.stomp_radius;
        let (damage, stun) = (
            self.tuning.stomp_damage,
            ms_to_ticks(self.tuning.stomp_stun),
        );
        let hits: Vec<(u32, Vec2)> = self
            .creatures
            .iter()
            .filter(|c| c.pos.distance(pos) < r + self.creature_kinds[c.kind].radius())
            .map(|c| (c.id, c.pos))
            .collect();
        for (id, p) in hits {
            let force = Vec2::new(f32::from(sign(p.x - pos.x)) * 4.0, -6.0);
            if let Some(i) = self.creature_index(id) {
                self.creatures[i].stun = stun;
                // die Sandschlange trifft Stampfen doppelt (E-316)
                let serpent = matches!(
                    self.creature_kinds[self.creatures[i].kind].behavior,
                    Behavior::Serpent(_)
                );
                let damage = if serpent { damage * 2 } else { damage };
                self.damage_creature(i, force, damage, Some(player), None);
            }
        }
    }

    /// Schaden an Gegner `id` ohne Rückstoß (Tests, Werkzeuge); prallt an Hütern in der Luft ab.
    pub fn hurt_creature(&mut self, id: u32, damage: i32) {
        if let Some(i) = self.creature_index(id) {
            self.damage_creature(i, Vec2::ZERO, damage, None, None);
        }
    }

    /// Schaden an Gegner `i` aus Richtung `src`; besiegt ihn bei 0 Leben.
    pub(crate) fn damage_creature(
        &mut self,
        i: usize,
        force: Vec2,
        damage: i32,
        from: Option<usize>,
        src: Option<Vec2>,
    ) {
        if self.prediction {
            return;
        }
        let tick = self.tick;
        let c = &mut self.creatures[i];
        let kind = &self.creature_kinds[c.kind];
        // Hüter in der Luft und Panzer von der Seite: Treffer prallen ab (E-299, E-317)
        if !c.vulnerable(kind) || c.armor_blocks(kind, src) {
            self.events.push(Event::CreatureHit {
                id: c.id,
                pos: c.pos,
                damage: 0,
                from,
            });
            return;
        }
        if !matches!(
            kind.behavior,
            Behavior::Turret { .. } | Behavior::Diver(_) | Behavior::Serpent(_)
        ) {
            c.vel += force;
        }
        c.health -= damage;
        c.hits = c.hits.saturating_add(1);
        c.hit_tick = Some(tick);
        self.events.push(Event::CreatureHit {
            id: c.id,
            pos: c.pos,
            damage,
            from,
        });
        if c.health <= 0 {
            self.kill_creature(i, from, true);
        }
    }

    fn kill_creature(&mut self, i: usize, killer: Option<usize>, drop_loot: bool) {
        let c = self.creatures.remove(i);
        self.events.push(Event::CreatureDeath {
            id: c.id,
            kind: c.kind,
            pos: c.pos,
            killer,
        });
        if drop_loot {
            self.drop_loot(&c);
        }
    }

    fn drop_loot(&mut self, c: &Creature) {
        let entries = self.creature_kinds[c.kind].loot.clone();
        let seed = u64::from(c.id) << 32 ^ self.tick;
        for (e, entry) in entries.iter().enumerate() {
            let s = rng(seed ^ (e as u64) << 16);
            if rng_f32(s) >= entry.chance {
                continue;
            }
            let span = entry.max.saturating_sub(entry.min) + 1;
            #[allow(clippy::cast_possible_truncation)]
            let count = entry.min + (rng(s ^ 1) % u64::from(span)) as u32;
            if count == 0 {
                continue;
            }
            let pieces = count.min(LOOT_PIECES);
            for p in 0..pieces {
                let share = count / pieces + u32::from(p < count % pieces);
                let r = rng(s ^ u64::from(p + 2) << 8);
                let id = self.next_id;
                self.next_id += 1;
                self.loot.push(Loot {
                    id,
                    item: entry.item.clone(),
                    count: share,
                    pos: c.pos,
                    vel: Vec2::new((rng_f32(r) - 0.5) * 8.0, -6.0 - rng_f32(r ^ 7) * 4.0),
                    age: 0,
                });
            }
        }
    }

    /// Lebende Figuren (Slot, Position).
    fn living(&self) -> Vec<(usize, Vec2)> {
        self.players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| Some((i, p.as_ref()?.character.as_ref()?.core.pos)))
            .collect()
    }

    pub(crate) fn tick_creatures(&mut self) {
        if self.creatures.is_empty() && self.creature_shots.is_empty() && self.loot.is_empty() {
            return;
        }
        let chars = self.living();
        self.tick_pull();
        self.tick_temp_tiles();
        self.tick_behavior(&chars);
        self.tick_shots(&chars);
        self.tick_contact(&chars);
        self.tick_loot(&chars);
    }

    /// Heranhooken (E-233): kleine Gegner fliegen zu Elora.
    fn tick_pull(&mut self) {
        let accel = self.tuning.pull_accel;
        let pulls: Vec<(u32, Vec2)> = self
            .players
            .iter()
            .filter_map(|p| p.as_ref()?.character.as_ref())
            .filter(|ch| ch.core.pulling)
            .filter_map(|ch| Some((ch.core.hooked_creature?, ch.core.pos)))
            .collect();
        for (id, to) in pulls {
            if let Some(c) = self.creatures.iter_mut().find(|c| c.id == id)
                && c.pos.distance(to) > PHYS_SIZE * 1.5
            {
                c.vel += (to - c.pos).normalize() * accel;
            }
        }
    }

    /// Wurzelwände verschwinden wieder.
    fn tick_temp_tiles(&mut self) {
        let tick = self.tick;
        let mut i = 0;
        while i < self.temp_tiles.len() {
            let (tx, ty, old, until) = self.temp_tiles[i];
            if tick >= until {
                self.collision.set_tile(tx, ty, old);
                self.events.push(Event::TileSet { tx, ty, tile: old });
                self.temp_tiles.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Spieler, die gerade am Wurzelwächter ziehen: am Hook und Laufrichtung weg von ihm.
    fn tuggers(&self) -> Vec<u32> {
        self.players
            .iter()
            .filter_map(|p| p.as_ref()?.character.as_ref())
            .filter_map(|ch| {
                let id = ch.core.hooked_creature?;
                let c = self.creatures.iter().find(|c| c.id == id)?;
                let away = ch.core.direction != 0
                    && sign(ch.core.pos.x - c.pos.x) == ch.core.direction.signum();
                away.then_some(id)
            })
            .collect()
    }

    #[allow(clippy::too_many_lines)]
    fn tick_behavior(&mut self, chars: &[(usize, Vec2)]) {
        let tuggers = self.tuggers();
        let Self {
            creatures,
            creature_kinds,
            collision,
            tuning,
            creature_shots,
            events,
            ..
        } = self;
        let gravity = tuning.gravity;
        let mut died = Vec::new();
        let mut summons: Vec<(u32, String, u32, Vec2)> = Vec::new();
        // Wurzelstöße (Mitte am Boden, Breite, Höhe, Schaden) und Wurzelwände (Fuß, Höhe, Dauer)
        let mut spikes: Vec<(Vec2, f32, f32, i32)> = Vec::new();
        let mut walls: Vec<(Vec2, u32, u32)> = Vec::new();
        // Treibsand der Sandschlange (Mitte am Boden, Breite in Tiles, Dauer)
        let mut sands: Vec<(Vec2, u32, u32)> = Vec::new();
        for c in creatures.iter_mut() {
            let kind = &creature_kinds[c.kind];
            let size = kind.size();
            let target = chars
                .iter()
                .map(|&(_, p)| (p, p.distance(c.pos)))
                .min_by(|a, b| a.1.total_cmp(&b.1));
            c.stun = c.stun.saturating_sub(1);
            let active = c.stun == 0;
            let mut fixed = false;
            match kind.behavior {
                Behavior::Walker {
                    speed,
                    turn_at_edges,
                } => {
                    c.vel.y += gravity;
                    if active {
                        let ahead = Vec2::new(
                            c.pos.x + f32::from(c.facing) * (size.x / 2.0 + 2.0),
                            c.pos.y + size.y / 2.0 + 4.0,
                        );
                        if turn_at_edges && c.grounded && !floor_at(collision, ahead) {
                            c.facing = -c.facing;
                        }
                        c.vel.x = f32::from(c.facing) * speed;
                    } else {
                        c.vel.x *= 0.8;
                    }
                }
                Behavior::Hopper {
                    wait_ms,
                    jump_x,
                    jump_y,
                    sight,
                } => {
                    c.vel.y += gravity;
                    if c.grounded {
                        c.vel.x *= 0.5;
                        if active {
                            c.timer += 1;
                            if let Some((p, d)) = target
                                && d <= sight
                                && c.timer >= ms_to_ticks(wait_ms)
                            {
                                c.facing = sign(p.x - c.pos.x);
                                c.vel = Vec2::new(f32::from(c.facing) * jump_x, -jump_y);
                                c.timer = 0;
                            }
                        }
                    }
                }
                Behavior::Turret {
                    interval_ms,
                    range,
                    shot_speed,
                    shot_damage,
                    lob,
                } => {
                    fixed = true;
                    c.vel = Vec2::ZERO;
                    c.timer = c.timer.saturating_add(1);
                    if active
                        && let Some((p, d)) = target
                        && d <= range
                    {
                        c.facing = sign(p.x - c.pos.x);
                        if c.timer >= ms_to_ticks(interval_ms)
                            && (lob || collision.intersect_line(c.pos, p).is_none())
                        {
                            let dir = (p - c.pos).normalize();
                            let (pos, vel, g) = if lob {
                                // Bogen: waagerecht mit `shot_speed`, senkrecht so, dass die
                                // Nuss bei Elora ankommt (flache Würfe mindestens etwas hoch)
                                let pos = c.pos + Vec2::new(0.0, -(kind.radius() + SHOT_RADIUS));
                                let d = p - pos;
                                let t = (d.x.abs() / shot_speed).max(12.0);
                                let vy = (d.y - 0.5 * gravity * t * t) / t;
                                (pos, Vec2::new(d.x / t, vy.min(-3.0)), gravity)
                            } else {
                                (
                                    c.pos + dir * (kind.radius() + SHOT_RADIUS),
                                    dir * shot_speed,
                                    0.0,
                                )
                            };
                            creature_shots.push(CreatureShot {
                                owner: c.id,
                                pos,
                                vel,
                                damage: shot_damage,
                                ticks: 0,
                                gravity: g,
                                glow: 0,
                                landed: false,
                            });
                            events.push(Event::CreatureFire { id: c.id, pos });
                            c.timer = 0;
                        }
                    }
                }
                Behavior::Diver(ref d) => {
                    if let Some(s) =
                        tick_diver(c, d, kind.health, target, collision, creature_shots, events)
                    {
                        summons.push((c.id, d.summon.clone(), d.summon_max, s));
                    }
                    if c.mode == crate::creature::diver::SLEEP {
                        continue;
                    }
                }
                Behavior::Warden(ref d) => {
                    fixed = true;
                    c.vel = Vec2::ZERO;
                    let feet = c.pos.y + size.y / 2.0;
                    let out = tick_warden(
                        c,
                        d,
                        kind.health,
                        target,
                        feet,
                        tuggers.contains(&c.id),
                        events,
                    );
                    spikes.extend(
                        out.0
                            .into_iter()
                            .map(|p| (p, d.spike_width, d.spike_height, d.spike_damage)),
                    );
                    if let Some(p) = out.1 {
                        walls.push((p, d.wall_height, d.wall_ms));
                    }
                }
                Behavior::Burrower {
                    sight,
                    out_ms,
                    hide_ms,
                    rise_ms,
                } => {
                    use crate::creature::burrow::{HIDDEN, OUT, RISING};
                    fixed = true;
                    c.vel = Vec2::ZERO;
                    c.timer = c.timer.saturating_add(1);
                    if let Some((p, _)) = target {
                        c.facing = sign(p.x - c.pos.x);
                    }
                    match c.mode {
                        HIDDEN => {
                            if active
                                && c.timer >= ms_to_ticks(hide_ms)
                                && target.is_some_and(|(_, d)| d <= sight)
                            {
                                c.mode = RISING;
                                c.timer = 0;
                                events.push(Event::CreatureAct {
                                    id: c.id,
                                    pos: c.pos,
                                    act: CreatureAct::Emerge,
                                });
                            }
                        }
                        RISING => {
                            if c.timer >= ms_to_ticks(rise_ms) {
                                c.mode = OUT;
                                c.timer = 0;
                            }
                        }
                        _ => {
                            if c.timer >= ms_to_ticks(out_ms) {
                                c.mode = HIDDEN;
                                c.timer = 0;
                                events.push(Event::CreatureAct {
                                    id: c.id,
                                    pos: c.pos,
                                    act: CreatureAct::Burrow,
                                });
                            }
                        }
                    }
                }
                Behavior::Follower { speed, jump } => {
                    c.vel.y += gravity;
                    let Some((p, d)) = target else {
                        c.vel.x *= 0.7;
                        continue;
                    };
                    if d > 64.0 && (p.x - c.pos.x).abs() > 24.0 {
                        let dir = sign(p.x - c.pos.x);
                        c.facing = dir;
                        let front = c.pos.x + f32::from(dir) * (size.x / 2.0 + 4.0);
                        let feet = c.pos.y + size.y / 2.0;
                        // Lücke oder Gefahr vor den Füßen: warten (E-308). Unter der
                        // Fußspitze muss das erste Tile (bis 2 Tiles tief) tragen.
                        let below = (0..5).find_map(|k| {
                            #[allow(clippy::cast_precision_loss)]
                            let y = feet + 4.0 + k as f32 * 16.0;
                            let t = collision.tile_at(Vec2::new(front, y));
                            (t != crate::Tile::Air).then_some(t)
                        });
                        let danger = below == Some(crate::Tile::Death);
                        let ground_ahead = below.is_some() && !danger;
                        let wall = collision.is_solid(Vec2::new(front, feet - 8.0));
                        if c.grounded && (danger || (!ground_ahead && p.y <= c.pos.y + 32.0)) {
                            c.vel.x = 0.0;
                        } else {
                            c.vel.x = f32::from(dir) * speed;
                            if wall && c.grounded {
                                c.vel.y = -jump;
                            }
                        }
                    } else {
                        c.vel.x *= 0.7;
                    }
                }
                Behavior::Serpent(ref d) => {
                    let feet = c.pos.y + size.y / 2.0;
                    let (sand, slam) =
                        tick_serpent(c, d, kind.health, gravity, feet, target, events);
                    if let Some(p) = sand {
                        sands.push((p, d.sand_width, d.sand_ms));
                    }
                    if let Some(p) = slam {
                        spikes.push((p, d.land_radius * 2.0, 64.0, d.land_damage));
                    }
                    if c.mode == crate::creature::serpent::SLEEP {
                        continue;
                    }
                }
                Behavior::Leaper {
                    sight,
                    speed,
                    warn_ms,
                    jump_x,
                    jump_y,
                    rest_ms,
                } => {
                    tick_leaper(
                        c,
                        &LeaperDef {
                            sight,
                            speed,
                            warn_ms,
                            jump_x,
                            jump_y,
                            rest_ms,
                        },
                        size,
                        gravity,
                        if active { target } else { None },
                        collision,
                        events,
                    );
                }
                Behavior::Flyer {
                    speed,
                    sight,
                    hover,
                    drop_ms,
                    drop_damage,
                    glow_ms,
                } => {
                    let chase = target.filter(|&(_, d)| d <= sight && active);
                    let goal = chase.map_or(c.home, |(p, _)| p - Vec2::new(0.0, hover));
                    c.timer = c.timer.saturating_add(1);
                    // Funken fallen lassen, wenn er über Elora ist
                    if drop_ms > 0
                        && let Some((p, _)) = chase
                        && (p.x - c.pos.x).abs() < 48.0
                        && p.y > c.pos.y
                        && c.timer >= ms_to_ticks(drop_ms)
                    {
                        let pos = c.pos + Vec2::new(0.0, size.y / 2.0 + SHOT_RADIUS);
                        creature_shots.push(CreatureShot {
                            owner: c.id,
                            pos,
                            vel: Vec2::new(c.vel.x * 0.5, 1.0),
                            damage: drop_damage,
                            ticks: 0,
                            gravity: gravity * 0.4,
                            glow: ms_to_ticks(glow_ms),
                            landed: false,
                        });
                        events.push(Event::CreatureFire { id: c.id, pos });
                        c.timer = 0;
                    }
                    let to = goal - c.pos;
                    let want = if to.length() > 4.0 {
                        to.normalize() * speed
                    } else {
                        Vec2::ZERO
                    };
                    c.vel = if active {
                        c.vel * 0.85 + want * 0.15
                    } else {
                        c.vel * 0.9
                    };
                    if to.x.abs() > 1.0 {
                        c.facing = sign(to.x);
                    }
                }
            }
            if fixed {
                continue;
            }
            let wanted_x = c.vel.x;
            let (mut pos, mut vel) = (c.pos, c.vel);
            let diving = matches!(kind.behavior, Behavior::Diver(_));
            let follower = matches!(kind.behavior, Behavior::Follower { .. });
            let death = collision.move_box_platforms(&mut pos, &mut vel, size, 0.0, !diving)
                && !diving
                && !follower;
            if matches!(kind.behavior, Behavior::Walker { .. }) && wanted_x != 0.0 && vel.x == 0.0 {
                c.facing = -c.facing;
            }
            c.pos = pos;
            c.vel = vel;
            c.grounded = grounded(collision, pos, size);
            if death {
                died.push(c.id);
            }
        }
        for id in died {
            if let Some(i) = self.creature_index(id) {
                self.kill_creature(i, None, false);
            }
        }
        self.root_spikes(&spikes, chars);
        self.root_walls(&walls, chars);
        self.quicksand_patches(&sands);
        // Helfer der Hüter, höchstens `max` zugleich
        for (owner, name, max, pos) in summons {
            let Some(kind) = self.creature_kind(&name) else {
                continue;
            };
            let alive = self.creatures.iter().filter(|c| c.kind == kind).count();
            if alive < max as usize
                && let Some(id) = self.add_creature(kind, pos)
            {
                self.events.push(Event::CreatureFire { id: owner, pos });
                let _ = id;
            }
        }
    }

    /// Wurzelstöße treffen Figuren in ihrer Zone (Rückstoß nach oben).
    fn root_spikes(&mut self, spikes: &[(Vec2, f32, f32, i32)], chars: &[(usize, Vec2)]) {
        let kb = self.tuning.hit_knockback;
        for &(at, w, h, damage) in spikes {
            for &(j, p) in chars {
                if (p.x - at.x).abs() < w / 2.0 + PHYS_SIZE / 2.0
                    && p.y > at.y - h
                    && p.y < at.y + 4.0
                {
                    self.take_damage(
                        j,
                        Vec2::new(0.0, -kb * 1.2),
                        damage,
                        None,
                        DeathCause::Creature,
                    );
                }
            }
        }
    }

    /// Wurzelwände: eine Tile-Spalte vom Boden aufwärts, nur in Luft und nicht auf Figuren.
    fn root_walls(&mut self, walls: &[(Vec2, u32, u32)], chars: &[(usize, Vec2)]) {
        let ts = crate::TILE_SIZE;
        for &(foot, height, ms) in walls {
            #[allow(clippy::cast_possible_truncation)]
            let tx = (foot.x as i32).div_euclid(ts);
            #[allow(clippy::cast_possible_truncation)]
            let ground = (foot.y as i32 - 1).div_euclid(ts);
            let until = self.tick + u64::from(ms_to_ticks(ms));
            #[allow(clippy::cast_possible_wrap)]
            for k in 0..height as i32 {
                let ty = ground - k;
                if self.collision.tile(tx, ty) != Tile::Air {
                    break;
                }
                #[allow(clippy::cast_precision_loss)]
                let center =
                    Vec2::new((tx as f32 + 0.5) * ts as f32, (ty as f32 + 0.5) * ts as f32);
                if chars.iter().any(|&(_, p)| p.distance(center) < PHYS_SIZE) {
                    break;
                }
                self.collision.set_tile(tx, ty, Tile::Unhookable);
                self.temp_tiles.push((tx, ty, Tile::Air, until));
                self.events.push(Event::TileSet {
                    tx,
                    ty,
                    tile: Tile::Unhookable,
                });
            }
        }
    }

    /// Treibsand im Kessel (Sandschlange, wütend): die Oberfläche des Bodens um `at` wird für
    /// eine Weile zu Treibsand (nur feste Tiles mit Luft darüber und festem Grund darunter).
    fn quicksand_patches(&mut self, patches: &[(Vec2, u32, u32)]) {
        let ts = crate::TILE_SIZE;
        for &(at, width, ms) in patches {
            #[allow(clippy::cast_possible_truncation)]
            let (cx, ty) = (
                (at.x as i32).div_euclid(ts),
                (at.y as i32 + 4).div_euclid(ts),
            );
            let until = self.tick + u64::from(ms_to_ticks(ms));
            #[allow(clippy::cast_possible_wrap)]
            let half = width as i32 / 2;
            for tx in cx - half..=cx + half {
                let t = self.collision.tile(tx, ty);
                if !matches!(t, Tile::Solid | Tile::Unhookable)
                    || self.collision.tile(tx, ty - 1) != Tile::Air
                    || !self.collision.tile(tx, ty + 1).is_solid()
                {
                    continue;
                }
                self.collision.set_tile(tx, ty, Tile::Quicksand);
                self.temp_tiles.push((tx, ty, t, until));
                self.events.push(Event::TileSet {
                    tx,
                    ty,
                    tile: Tile::Quicksand,
                });
            }
        }
    }

    fn tick_shots(&mut self, chars: &[(usize, Vec2)]) {
        let kb = self.tuning.hit_knockback;
        let mut i = 0;
        while i < self.creature_shots.len() {
            let s = &mut self.creature_shots[i];
            s.ticks += 1;
            if s.landed {
                s.glow = s.glow.saturating_sub(1);
            } else {
                s.vel.y += s.gravity;
                s.pos += s.vel;
            }
            let (pos, vel, damage) = (s.pos, s.vel, s.damage);
            let hit = chars
                .iter()
                .find(|&&(_, p)| p.distance(pos) < PHYS_SIZE / 2.0 + SHOT_RADIUS)
                .map(|&(j, _)| j);
            if let Some(j) = hit {
                self.creature_shots.remove(i);
                let force = vel.normalize() * (kb * 0.5);
                self.take_damage(j, force, damage, None, DeathCause::Creature);
            } else if self.creature_shots[i].landed {
                // Funke glüht am Boden aus
                if self.creature_shots[i].glow == 0 {
                    self.creature_shots.remove(i);
                } else {
                    i += 1;
                }
            } else if self.collision.is_solid(pos) && self.creature_shots[i].glow > 0 {
                let s = &mut self.creature_shots[i];
                s.pos -= s.vel;
                s.vel = Vec2::ZERO;
                s.landed = true;
                i += 1;
            } else if self.collision.is_solid(pos) || self.creature_shots[i].ticks > SHOT_LIFETIME {
                self.creature_shots.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Berührung schadet Elora (E-232) mit Rückstoß (A-12); der Schutz danach steckt in
    /// `take_damage` (E-234).
    fn tick_contact(&mut self, chars: &[(usize, Vec2)]) {
        let kb = self.tuning.hit_knockback;
        let mut hits = Vec::new();
        let mut dazes = Vec::new();
        for c in &self.creatures {
            let k = &self.creature_kinds[c.kind];
            // benommene Hüter, versteckte Schlangen und Begleiter schaden nicht
            if (k.touch_damage <= 0 && k.daze_ms == 0) || !c.harmful(k) {
                continue;
            }
            for &(j, p) in chars {
                let d = p - c.pos;
                if d.x.abs() < f32::midpoint(PHYS_SIZE, k.size[0])
                    && d.y.abs() < f32::midpoint(PHYS_SIZE, k.size[1])
                {
                    let force = Vec2::new(f32::from(sign(d.x)) * kb, -kb * 0.6);
                    hits.push((j, force, k.touch_damage));
                    if k.daze_ms > 0 {
                        dazes.push((j, ms_to_ticks(k.daze_ms)));
                    }
                }
            }
        }
        for (j, force, damage) in hits {
            if damage > 0 {
                self.take_damage(j, force, damage, None, DeathCause::Creature);
            }
        }
        // bunter Rausch (E-311): nicht nachladen, solange er noch wirkt
        for (j, ticks) in dazes {
            if let Some(ch) = self.character_mut(j)
                && ch.core.dazed == 0
            {
                ch.core.dazed = ticks;
            }
        }
    }

    fn tick_loot(&mut self, chars: &[(usize, Vec2)]) {
        let (gravity, magnet) = (self.tuning.gravity, self.tuning.loot_magnet);
        let size = Vec2::new(LOOT_SIZE, LOOT_SIZE);
        let mut collected = Vec::new();
        for l in &mut self.loot {
            l.age = l.age.saturating_add(1);
            let near = chars
                .iter()
                .map(|&(j, p)| (j, p, p.distance(l.pos)))
                .filter(|x| x.2 < magnet)
                .min_by(|a, b| a.2.total_cmp(&b.2));
            if l.age >= LOOT_DELAY
                && let Some((j, p, d)) = near
            {
                l.vel = Vec2::ZERO;
                if d < PHYS_SIZE / 2.0 + 4.0 {
                    collected.push((l.id, j));
                } else {
                    l.pos += (p - l.pos).normalize() * LOOT_MAGNET_SPEED.min(d);
                }
                continue;
            }
            l.vel.y += gravity;
            let (mut pos, mut vel) = (l.pos, l.vel);
            self.collision
                .move_box_platforms(&mut pos, &mut vel, size, 0.5, true);
            if grounded(&self.collision, pos, size) {
                vel.x *= 0.9;
            }
            l.pos = pos;
            l.vel = vel;
        }
        for (id, player) in collected {
            if let Some(i) = self.loot.iter().position(|l| l.id == id) {
                let l = self.loot.remove(i);
                self.events.push(Event::LootCollect {
                    player,
                    item: l.item,
                    count: l.count,
                    pos: l.pos,
                });
            }
        }
    }
}

/// Ein Tick des Hüters im Sand; liefert eine Stelle für Treibsand (wütend) und die Stelle
/// einer Landung, die Sand schleudert.
#[allow(clippy::too_many_lines)]
fn tick_serpent(
    c: &mut Creature,
    d: &SerpentDef,
    max_health: i32,
    gravity: f32,
    feet: f32,
    target: Option<(Vec2, f32)>,
    events: &mut Vec<Event>,
) -> (Option<Vec2>, Option<Vec2>) {
    use crate::creature::serpent::{LEAP, SLEEP, STUNNED, TRAIL, WARN};
    #[allow(clippy::cast_precision_loss)]
    let life = c.health as f32 / max_health.max(1) as f32;
    let angry = d.enrage_at > 0.0 && life <= d.enrage_at;
    let double = d.double_at > 0.0 && life <= d.double_at;
    let pace = if angry { 1.35 } else { 1.0 };
    let ticks = |ms: u32| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let t = (ms_to_ticks(ms) as f32 / pace) as u32;
        t.max(1)
    };
    let id = c.id;
    let act = |events: &mut Vec<Event>, pos: Vec2, act: CreatureAct| {
        events.push(Event::CreatureAct { id, pos, act });
    };
    c.stun = 0;
    c.timer = c.timer.saturating_add(1);
    c.vel.y += gravity;
    let mut sand = None;
    let mut slam = None;
    // ab halbem Leben springt sie gezielt auf Elora zu
    let aim = target.map(|(p, _)| p).filter(|_| angry);
    match c.mode {
        SLEEP => {
            c.vel = Vec2::ZERO;
            if target.is_some_and(|(_, dist)| dist <= d.sight) {
                c.mode = TRAIL;
                c.timer = 0;
                act(events, c.pos, CreatureAct::Wake);
            }
            return (None, None);
        }
        TRAIL => {
            let Some((p, _)) = target else {
                c.vel.x = 0.0;
                return (None, None);
            };
            let dx = p.x - c.pos.x;
            c.facing = sign(dx);
            c.vel.x = if dx.abs() < 6.0 {
                0.0
            } else {
                f32::from(c.facing) * (d.speed * pace).min(dx.abs())
            };
            if c.timer >= ticks(d.trail_ms) && dx.abs() < 20.0 && c.grounded {
                c.mode = WARN;
                c.timer = 0;
                c.goal = Vec2::new(c.pos.x, feet);
                c.vel.x = 0.0;
                act(events, c.goal, CreatureAct::Warn);
            }
        }
        WARN => {
            c.vel.x = 0.0;
            if c.timer >= ticks(d.warn_ms) {
                c.hits = 0;
                leap(c, d, gravity, pace, aim);
                act(events, c.pos, CreatureAct::Emerge);
            }
        }
        LEAP => {
            if c.vel.x.abs() > 0.1 {
                c.facing = sign(c.vel.x);
            }
            if c.grounded && c.timer > 4 && c.vel.y >= 0.0 {
                c.vel.x = 0.0;
                act(events, c.pos, CreatureAct::Land);
                if d.land_damage > 0 {
                    slam = Some(Vec2::new(c.pos.x, feet));
                }
                if double && c.count == 0 {
                    // gleich noch ein Bogen, jetzt auf Elora zu
                    c.count = 1;
                    leap(c, d, gravity, pace, target.map(|(p, _)| p));
                    act(events, c.pos, CreatureAct::Emerge);
                } else {
                    c.count = 0;
                    c.mode = STUNNED;
                    c.timer = 0;
                }
            }
        }
        _ => {
            // STUNNED: benommen, dann wieder unter den Sand – nach zu vielen Treffern sofort
            c.vel.x = 0.0;
            let too_many = d.open_hits > 0 && c.hits >= d.open_hits;
            if c.timer >= ms_to_ticks(d.stun_ms) || too_many {
                c.hits = 0;
                c.mode = TRAIL;
                c.timer = 0;
                act(events, c.pos, CreatureAct::Burrow);
            }
        }
    }
    // wütend: Teile des Kessels werden zu Treibsand, abwechselnd links und rechts von Elora
    if angry && d.sand_every_ms > 0 && matches!(c.mode, TRAIL | WARN) {
        c.wall_timer += 1;
        if c.wall_timer >= ms_to_ticks(d.sand_every_ms)
            && let Some((p, _)) = target
        {
            c.wall_timer = 0;
            c.tug = 1 - c.tug.min(1);
            let side = if c.tug == 0 { -1.0 } else { 1.0 };
            #[allow(clippy::cast_precision_loss)]
            let at = Vec2::new(p.x + side * 5.0 * crate::TILE_SIZE as f32, feet);
            sand = Some(at);
        }
    }
    (sand, slam)
}

/// Sprung der Sandschlange: hoch und zur Mitte des Kessels (Startpunkt) – oder mit `aim`
/// so, dass sie auf dieser Stelle landet.
fn leap(c: &mut Creature, d: &SerpentDef, gravity: f32, pace: f32, aim: Option<Vec2>) {
    use crate::creature::serpent::LEAP;
    let vx = if let Some(p) = aim {
        // Flugzeit bis zurück auf die Ausgangshöhe
        let air = 2.0 * d.jump_y / gravity.max(0.01);
        let max = d.jump_x * 1.6;
        ((p.x - c.pos.x) / air).clamp(-max, max)
    } else {
        let toward = if (c.home.x - c.pos.x).abs() > 1.0 {
            sign(c.home.x - c.pos.x)
        } else {
            -c.facing
        };
        f32::from(toward) * d.jump_x * pace.sqrt()
    };
    if vx.abs() > 0.1 {
        c.facing = sign(vx);
    }
    c.vel = Vec2::new(vx, -d.jump_y);
    c.mode = LEAP;
    c.timer = 0;
}

/// Werte des Dünenwurms (aus [`Behavior::Leaper`]).
struct LeaperDef {
    sight: f32,
    speed: f32,
    warn_ms: u32,
    jump_x: f32,
    jump_y: f32,
    rest_ms: u32,
}

/// Ein Tick des Dünenwurms: unter dem Sand heran, Warnung, Sprung im Bogen auf die Stelle,
/// an der Elora bei der Warnung stand, Landung und Eintauchen.
fn tick_leaper(
    c: &mut Creature,
    d: &LeaperDef,
    size: Vec2,
    gravity: f32,
    target: Option<(Vec2, f32)>,
    collision: &Collision,
    events: &mut Vec<Event>,
) {
    use crate::creature::leaper::{LEAP, UNDER, WARN};
    c.vel.y += gravity;
    c.timer = c.timer.saturating_add(1);
    // Flugzeit eines Sprungs bis zurück auf die Ausgangshöhe
    let air = 2.0 * d.jump_y / gravity.max(0.01);
    let act = |events: &mut Vec<Event>, pos: Vec2, act: CreatureAct| {
        events.push(Event::CreatureAct { id: c.id, pos, act });
    };
    match c.mode {
        UNDER => {
            let Some((p, _)) = target.filter(|&(_, dist)| dist <= d.sight) else {
                c.vel.x *= 0.8;
                return;
            };
            let dx = p.x - c.pos.x;
            c.facing = sign(dx);
            if c.grounded
                && c.timer >= ms_to_ticks(d.rest_ms)
                && dx.abs() <= d.jump_x * air
                && (p.y - c.pos.y).abs() < 160.0
            {
                c.mode = WARN;
                c.timer = 0;
                c.goal = p;
                c.vel.x = 0.0;
                act(events, c.pos, CreatureAct::Warn);
                return;
            }
            let ahead = Vec2::new(
                c.pos.x + f32::from(c.facing) * (size.x / 2.0 + 2.0),
                c.pos.y + size.y / 2.0 + 4.0,
            );
            let edge = c.grounded && !floor_at(collision, ahead);
            c.vel.x = if edge || dx.abs() < 8.0 {
                0.0
            } else {
                f32::from(c.facing) * d.speed
            };
        }
        WARN => {
            c.vel.x = 0.0;
            if c.timer >= ms_to_ticks(d.warn_ms) {
                let dx = c.goal.x - c.pos.x;
                c.vel = Vec2::new((dx / air).clamp(-d.jump_x, d.jump_x), -d.jump_y);
                c.mode = LEAP;
                c.timer = 0;
                act(events, c.pos, CreatureAct::Emerge);
            }
        }
        _ => {
            if c.grounded && c.timer > 4 && c.vel.y >= 0.0 {
                c.vel.x = 0.0;
                c.mode = UNDER;
                c.timer = 0;
                act(events, c.pos, CreatureAct::Burrow);
            }
        }
    }
}

/// Ein Tick des Hüters aus der Luft; liefert eine Position, wenn er einen Helfer ruft.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn tick_diver(
    c: &mut Creature,
    d: &DiverDef,
    max_health: i32,
    target: Option<(Vec2, f32)>,
    collision: &Collision,
    shots: &mut Vec<CreatureShot>,
    events: &mut Vec<Event>,
) -> Option<Vec2> {
    use crate::creature::diver::{AIM, CIRCLE, DIVE, RISE, SLEEP, STUNNED};
    #[allow(clippy::cast_precision_loss)]
    let life = c.health as f32 / max_health.max(1) as f32;
    let angry = d.enrage_at > 0.0 && life <= d.enrage_at;
    let pace = if angry { 1.35 } else { 1.0 };
    let ticks = |ms: u32| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let t = (ms_to_ticks(ms) as f32 / pace) as u32;
        t.max(1)
    };
    c.stun = 0;
    c.timer = c.timer.saturating_add(1);
    let mut summon = None;
    // Helfer rufen (in jedem wachen Zustand)
    if d.summon_at > 0.0
        && life <= d.summon_at
        && c.mode != SLEEP
        && d.summon_ms > 0
        && c.timer.is_multiple_of(ms_to_ticks(d.summon_ms).max(1))
    {
        summon = Some(c.pos + Vec2::new(f32::from(c.facing) * 40.0, -30.0));
    }
    match c.mode {
        SLEEP => {
            c.vel = Vec2::ZERO;
            if target.is_some_and(|(_, dist)| dist <= d.sight) {
                c.mode = CIRCLE;
                c.timer = 0;
                events.push(Event::CreatureAct {
                    id: c.id,
                    pos: c.pos,
                    act: CreatureAct::Wake,
                });
            }
        }
        CIRCLE => {
            // Ellipse über dem Startpunkt
            c.count = (c.count + 1) % 100_000;
            #[allow(clippy::cast_precision_loss)]
            let a = c.count as f32 * d.speed * pace / d.circle[0].max(1.0);
            let want = c.home + Vec2::new(a.cos() * d.circle[0], a.sin() * d.circle[1]);
            c.vel = (want - c.pos) * 0.2;
            if c.vel.x.abs() > 0.3 {
                c.facing = sign(c.vel.x);
            }
            if d.drop_ms > 0 && c.timer.is_multiple_of(ticks(d.drop_ms)) {
                let pos = c.pos + Vec2::new(0.0, SHOT_RADIUS + 20.0);
                shots.push(CreatureShot {
                    owner: c.id,
                    pos,
                    vel: Vec2::new(0.0, d.drop_speed),
                    damage: d.drop_damage,
                    ticks: 0,
                    gravity: 0.0,
                    glow: 0,
                    landed: false,
                });
                events.push(Event::CreatureFire { id: c.id, pos });
            }
            if c.timer >= ticks(d.circle_ms) && target.is_some() {
                c.mode = AIM;
                c.timer = 0;
                // ab hier zählt `count` die Sturzflüge
                c.count = 0;
            }
        }
        AIM => {
            c.vel = Vec2::ZERO;
            if let Some((p, _)) = target {
                c.goal = p;
                c.facing = sign(p.x - c.pos.x);
            }
            if c.timer >= ticks(d.aim_ms) {
                c.mode = DIVE;
                c.timer = 0;
                events.push(Event::CreatureAct {
                    id: c.id,
                    pos: c.pos,
                    act: CreatureAct::Dive,
                });
            }
        }
        DIVE => {
            let to = c.goal - c.pos;
            let dir = if to.length() > 1.0 {
                to.normalize()
            } else {
                Vec2::new(0.0, 1.0)
            };
            c.vel = dir * d.dive_speed * pace;
            // Boden erreicht oder zu lange unterwegs
            let ahead = c.pos + c.vel;
            if collision.is_solid(ahead) || c.grounded || c.timer > 150 {
                c.vel = Vec2::ZERO;
                events.push(Event::CreatureAct {
                    id: c.id,
                    pos: c.pos,
                    act: CreatureAct::Land,
                });
                if angry && c.count == 0 {
                    // gleich noch einmal
                    c.count = 1;
                    c.mode = AIM;
                } else {
                    c.count = 0;
                    c.mode = STUNNED;
                }
                c.timer = 0;
            }
        }
        STUNNED => {
            c.vel = Vec2::new(0.0, 4.0);
            if c.timer >= ms_to_ticks(d.stun_ms) {
                c.mode = RISE;
                c.timer = 0;
            }
        }
        _ => {
            // RISE: zurück zum Kreis
            let to = c.home - c.pos;
            c.vel = if to.length() > 8.0 {
                to.normalize() * d.speed * 1.5 * pace
            } else {
                Vec2::ZERO
            };
            if to.length() <= 8.0 || c.timer > 250 {
                c.mode = CIRCLE;
                c.timer = 0;
            }
        }
    }
    summon
}

/// Ein Tick des Hüters am Boden; liefert Wurzelstöße (Mitte am Boden) und eine Wurzelwand.
#[allow(clippy::too_many_lines)]
fn tick_warden(
    c: &mut Creature,
    d: &WardenDef,
    max_health: i32,
    target: Option<(Vec2, f32)>,
    feet: f32,
    tugged: bool,
    events: &mut Vec<Event>,
) -> (Vec<Vec2>, Option<Vec2>) {
    use crate::creature::warden::{IDLE, OPEN, SLEEP, WARN};
    #[allow(clippy::cast_precision_loss)]
    let life = c.health as f32 / max_health.max(1) as f32;
    let angry = d.enrage_at > 0.0 && life <= d.enrage_at;
    let pace = if angry { 1.4 } else { 1.0 };
    let ticks = |ms: u32| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let t = (ms_to_ticks(ms) as f32 / pace) as u32;
        t.max(1)
    };
    let mut spikes = Vec::new();
    let mut wall = None;
    c.timer = c.timer.saturating_add(1);
    if let Some((p, _)) = target {
        c.facing = sign(p.x - c.pos.x);
    }
    let act = |events: &mut Vec<Event>, pos: Vec2, act: CreatureAct| {
        events.push(Event::CreatureAct { id: c.id, pos, act });
    };
    match c.mode {
        SLEEP => {
            if target.is_some_and(|(_, dist)| dist <= d.sight) {
                c.mode = IDLE;
                c.timer = 0;
                act(events, c.pos, CreatureAct::Wake);
            }
            return (spikes, wall);
        }
        OPEN => {
            c.tug = 0;
            if c.timer >= ms_to_ticks(d.open_ms) {
                c.mode = IDLE;
                c.timer = 0;
                if c.count == 0 {
                    c.count = d.cores;
                }
            }
            return (spikes, wall);
        }
        _ => {}
    }
    // Tauziehen: Kern löst sich
    c.tug = if tugged {
        c.tug + 1
    } else {
        c.tug.saturating_sub(2)
    };
    if c.tug >= ms_to_ticks(d.pull_ms) && c.count > 0 {
        c.count -= 1;
        c.tug = 0;
        c.mode = OPEN;
        c.timer = 0;
        act(events, c.pos, CreatureAct::Core);
        return (spikes, wall);
    }
    match c.mode {
        WARN => {
            if c.timer >= ticks(d.warn_ms) {
                spikes.push(c.goal);
                act(events, c.goal, CreatureAct::Strike);
                // letzter Kern: gleich noch eine Stelle näher am Wächter
                if c.count == 1 {
                    let toward = sign(c.pos.x - c.goal.x);
                    let second =
                        c.goal + Vec2::new(f32::from(toward) * 3.0 * crate::TILE_SIZE as f32, 0.0);
                    spikes.push(second);
                    act(events, second, CreatureAct::Strike);
                }
                c.mode = IDLE;
                c.timer = 0;
            }
        }
        _ => {
            if let Some((p, _)) = target
                && c.timer >= ticks(d.attack_ms)
            {
                c.goal = Vec2::new(p.x, feet);
                c.mode = WARN;
                c.timer = 0;
                act(events, c.goal, CreatureAct::Warn);
            }
        }
    }
    // Wurzelwände, wenn wütend: zwischen Elora und dem Wächter
    if angry && d.wall_every_ms > 0 {
        c.wall_timer += 1;
        if c.wall_timer >= ms_to_ticks(d.wall_every_ms)
            && let Some((p, _)) = target
        {
            c.wall_timer = 0;
            let mid = Vec2::new(f32::midpoint(p.x, c.pos.x), feet);
            if (mid.x - c.pos.x).abs() > 64.0 && (mid.x - p.x).abs() > 48.0 {
                wall = Some(mid);
            }
        }
    }
    (spikes, wall)
}

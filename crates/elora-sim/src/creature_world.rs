//! Kreaturen in der Welt (A1.2): Verhalten, Bewegung, Treffer, Berührung, Geschosse, Beute.

use crate::character::PHYS_SIZE;
use crate::collision::{Collision, Tile};
use crate::creature::{Behavior, Creature, CreatureShot, HookTarget, Loot, rng, rng_f32};
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
        });
        Some(id)
    }

    pub(crate) fn hook_targets(&self) -> Vec<HookTarget> {
        self.creatures
            .iter()
            .map(|c| {
                let k = &self.creature_kinds[c.kind];
                HookTarget {
                    id: c.id,
                    pos: c.pos,
                    radius: k.radius(),
                    small: k.small,
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
            self.explode_creatures(p, owner, (max_damage / 3).max(1));
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
                self.damage_creature(i, force, damage, Some(owner));
            }
        }
        hits.len()
    }

    /// Explosion trifft Gegner (wie Figuren, ohne Rüstung).
    pub(crate) fn explode_creatures(&mut self, pos: Vec2, owner: usize, max_damage: i32) {
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
                self.damage_creature(i, force, damage, Some(owner));
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
                self.damage_creature(i, force, damage, Some(player));
            }
        }
    }

    /// Schaden an Gegner `i`; besiegt ihn bei 0 Leben.
    pub(crate) fn damage_creature(
        &mut self,
        i: usize,
        force: Vec2,
        damage: i32,
        from: Option<usize>,
    ) {
        if self.prediction {
            return;
        }
        let tick = self.tick;
        let c = &mut self.creatures[i];
        if !matches!(
            self.creature_kinds[c.kind].behavior,
            Behavior::Turret { .. }
        ) {
            c.vel += force;
        }
        c.health -= damage;
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

    #[allow(clippy::too_many_lines)]
    fn tick_behavior(&mut self, chars: &[(usize, Vec2)]) {
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
                            && collision.intersect_line(c.pos, p).is_none()
                        {
                            let dir = (p - c.pos).normalize();
                            let pos = c.pos + dir * (kind.radius() + SHOT_RADIUS);
                            creature_shots.push(CreatureShot {
                                owner: c.id,
                                pos,
                                vel: dir * shot_speed,
                                damage: shot_damage,
                                ticks: 0,
                            });
                            events.push(Event::CreatureFire { id: c.id, pos });
                            c.timer = 0;
                        }
                    }
                }
                Behavior::Flyer { speed, sight } => {
                    let goal = match target {
                        Some((p, d)) if d <= sight && active => p,
                        _ => c.home,
                    };
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
            let death = collision.move_box_platforms(&mut pos, &mut vel, size, 0.0, true);
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
    }

    fn tick_shots(&mut self, chars: &[(usize, Vec2)]) {
        let kb = self.tuning.hit_knockback;
        let mut i = 0;
        while i < self.creature_shots.len() {
            let s = &mut self.creature_shots[i];
            s.pos += s.vel;
            s.ticks += 1;
            let (pos, vel, damage) = (s.pos, s.vel, s.damage);
            let hit = chars
                .iter()
                .find(|&&(_, p)| p.distance(pos) < PHYS_SIZE / 2.0 + SHOT_RADIUS)
                .map(|&(j, _)| j);
            if let Some(j) = hit {
                self.creature_shots.remove(i);
                let force = vel.normalize() * (kb * 0.5);
                self.take_damage(j, force, damage, None, DeathCause::Creature);
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
        for c in &self.creatures {
            let k = &self.creature_kinds[c.kind];
            if k.touch_damage <= 0 {
                continue;
            }
            for &(j, p) in chars {
                let d = p - c.pos;
                if d.x.abs() < f32::midpoint(PHYS_SIZE, k.size[0])
                    && d.y.abs() < f32::midpoint(PHYS_SIZE, k.size[1])
                {
                    let force = Vec2::new(f32::from(sign(d.x)) * kb, -kb * 0.6);
                    hits.push((j, force, k.touch_damage));
                }
            }
        }
        for (j, force, damage) in hits {
            self.take_damage(j, force, damage, None, DeathCause::Creature);
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

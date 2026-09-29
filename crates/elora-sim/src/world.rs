//! Spielwelt: fester Tick über Spieler, Projektile, Laser und Pickups.
//!
//! Ablauf eines Ticks wie im Original (`CGameContext::OnTick`):
//! 1. Eingaben übernehmen, Waffenwechsel und Klick-Schüsse (`OnDirectInput`)
//! 2. Projektile, Laser, Pickups
//! 3. Figuren: Kräfte und Hook, danach Waffen (Reload bzw. Dauerfeuer)
//! 4. Figuren: Hook-Zug anwenden, bewegen, quantisieren, Todes-Tiles
//! 5. Spieler: Respawn

use crate::TICKS_PER_SECOND;
use crate::character::{CharacterCore, PHYS_SIZE};
use crate::collision::Collision;
use crate::dummy::{DummyBrain, DummyPattern};
use crate::entities::{Laser, Pickup, Projectile};
use crate::event::{DeathCause, Event, PickupKind};
use crate::input::{PlayerInput, count_presses};
use crate::math::Vec2;
use crate::player::{Character, Controller, Player};
use crate::tuning::{Tuning, ms_to_ticks, secs_to_ticks};
use crate::weapon::Weapon;

/// Sperre nach Feuern ohne Munition (Original: 125 ms).
const NO_AMMO_DELAY_MS: u32 = 125;
/// Sperre nach einem Hammer-Treffer (Original: 1/3 s).
const HAMMER_HIT_DELAY: u32 = TICKS_PER_SECOND / 3;
/// Radius des Hammer-Treffers um den Schlagpunkt (Original: halbe Körpergröße).
const HAMMER_RADIUS: f32 = PHYS_SIZE * 0.5;
/// Radius eines Projektils bei der Treffer-Prüfung.
const PROJECTILE_RADIUS: f32 = 6.0;
/// Aufnahme-Radius eines Pickups (Original: 20, effektiv < 40 durch `ClosestEntity`).
const PICKUP_RADIUS: f32 = 20.0;

#[derive(Debug, Clone)]
pub struct World {
    pub tuning: Tuning,
    pub collision: Collision,
    pub tick: u64,
    /// Ein Slot pro Spieler; `None` = unbesetzt.
    pub players: Vec<Option<Player>>,
    pub projectiles: Vec<Projectile>,
    pub lasers: Vec<Laser>,
    pub pickups: Vec<Pickup>,
    /// Spawnpunkte für menschliche Spieler.
    pub spawn_points: Vec<Vec2>,
    /// Ereignisse des letzten Ticks.
    pub events: Vec<Event>,
    /// Client-Vorhersage (E-057): Kräfte wirken, aber kein Schaden, kein Tod, keine
    /// Pickups und kein Respawn – das entscheidet allein der Server.
    pub prediction: bool,
}

impl World {
    pub fn new(tuning: Tuning, collision: Collision) -> Self {
        Self {
            tuning,
            collision,
            tick: 0,
            players: Vec::new(),
            projectiles: Vec::new(),
            lasers: Vec::new(),
            pickups: Vec::new(),
            spawn_points: Vec::new(),
            events: Vec::new(),
            prediction: false,
        }
    }

    fn add_player(&mut self, player: Player) -> usize {
        if let Some(i) = self.players.iter().position(Option::is_none) {
            self.players[i] = Some(player);
            i
        } else {
            self.players.push(Some(player));
            self.players.len() - 1
        }
    }

    /// Fügt einen menschlichen Spieler hinzu, der beim nächsten Tick an einem
    /// Spawnpunkt erscheint.
    pub fn join(&mut self) -> usize {
        self.add_player(Player::new(Controller::Human))
    }

    /// Fügt einen menschlichen Spieler hinzu, der sofort an `pos` steht.
    pub fn spawn(&mut self, pos: Vec2) -> usize {
        let i = self.join();
        self.spawn_character(i, pos);
        i
    }

    /// Fügt einen Trainings-Dummy hinzu (E-053).
    pub fn add_dummy(&mut self, pos: Vec2, pattern: DummyPattern) -> usize {
        let i = self.add_player(Player::new(Controller::Dummy {
            pattern,
            home: pos,
            brain: DummyBrain::default(),
        }));
        self.spawn_character(i, pos);
        i
    }

    pub fn add_pickup(&mut self, kind: PickupKind, pos: Vec2) {
        self.pickups.push(Pickup {
            kind,
            pos,
            respawn_tick: None,
        });
    }

    pub fn remove(&mut self, index: usize) {
        if let Some(slot) = self.players.get_mut(index) {
            *slot = None;
        }
    }

    pub fn player(&self, i: usize) -> Option<&Player> {
        self.players.get(i)?.as_ref()
    }

    pub fn character(&self, i: usize) -> Option<&Character> {
        self.player(i)?.character.as_ref()
    }

    pub fn character_mut(&mut self, i: usize) -> Option<&mut Character> {
        self.players.get_mut(i)?.as_mut()?.character.as_mut()
    }

    /// Setzt eine Figur sofort (neu) an `pos`, z. B. für den Sandbox-Respawn.
    pub fn spawn_character(&mut self, i: usize, pos: Vec2) {
        let max_health = self.tuning.max_health;
        if let Some(p) = self.players.get_mut(i).and_then(Option::as_mut) {
            p.character = Some(Character::spawn(pos, max_health));
            p.spawning = false;
            if let Controller::Dummy { brain, .. } = &mut p.controller {
                *brain = DummyBrain::default();
            }
            self.events.push(Event::Spawn { player: i, pos });
        }
    }

    /// Positionen aller lebenden Figuren (Index = Slot).
    fn positions(&self) -> Vec<Option<Vec2>> {
        self.players
            .iter()
            .map(|p| {
                p.as_ref()
                    .and_then(|p| p.character.as_ref())
                    .map(|c| c.core.pos)
            })
            .collect()
    }

    /// Simuliert einen Tick. `inputs[i]` gehört zum Slot `i`; fehlende Einträge
    /// gelten als leere Eingabe. Dummies erzeugen ihre Eingaben selbst.
    pub fn step(&mut self, inputs: &[PlayerInput]) {
        self.events.clear();
        self.apply_inputs(inputs);

        self.tick += 1;
        self.tick_projectiles();
        self.tick_lasers();
        self.tick_pickups();
        self.tick_characters();
        self.tick_characters_deferred();
        self.tick_respawns();
    }

    // ---------------------------------------------------------------- Eingaben

    fn apply_inputs(&mut self, inputs: &[PlayerInput]) {
        for i in 0..self.players.len() {
            let Some(p) = self.players[i].as_mut() else {
                continue;
            };
            let mut input = match &mut p.controller {
                Controller::Remote => continue,
                Controller::Human => inputs.get(i).copied().unwrap_or_default(),
                Controller::Dummy { pattern, brain, .. } => match &p.character {
                    Some(c) => brain.input(*pattern, &c.core, &self.collision, self.tick),
                    None => PlayerInput::default(),
                },
            };
            // nicht ins Zentrum zielen
            if input.target_x == 0 && input.target_y == 0 {
                input.target_y = -1;
            }
            p.prev_input = p.input;
            p.input = input;
            if p.character.is_none() {
                if input.fire_held() && !p.is_dummy() {
                    p.spawning = true;
                }
                continue;
            }
            let prev = p.prev_input;
            self.handle_weapon_switch(i, &prev, &input);
            self.fire_weapon(i, count_presses(prev.fire, input.fire));
        }
    }

    fn handle_weapon_switch(&mut self, i: usize, prev: &PlayerInput, input: &PlayerInput) {
        let Some(ch) = self.character_mut(i) else {
            return;
        };
        let a = &ch.arsenal;
        let mut wanted = a.queued.unwrap_or(a.active);
        wanted = a.cycle(
            wanted,
            count_presses(prev.next_weapon, input.next_weapon),
            true,
        );
        wanted = a.cycle(
            wanted,
            count_presses(prev.prev_weapon, input.prev_weapon),
            false,
        );
        if let Some(w) = Weapon::from_number(input.wanted_weapon) {
            wanted = w;
        }
        if wanted != a.active && a.has(wanted) {
            ch.arsenal.queued = Some(wanted);
        }
        self.do_weapon_switch(i);
    }

    fn do_weapon_switch(&mut self, i: usize) {
        let Some(ch) = self.character_mut(i) else {
            return;
        };
        let a = &mut ch.arsenal;
        if a.reload_timer != 0 {
            return;
        }
        let Some(w) = a.queued.take() else { return };
        if w != a.active {
            a.active = w;
            self.events.push(Event::WeaponSwitch {
                player: i,
                weapon: w,
            });
        }
    }

    // ---------------------------------------------------------------- Waffen

    /// `presses`: Klicks seit der letzten Eingabe (0 = nur Dauerfeuer).
    fn fire_weapon(&mut self, i: usize, presses: u32) {
        if self
            .character(i)
            .is_none_or(|c| c.arsenal.reload_timer != 0)
        {
            return;
        }
        self.do_weapon_switch(i);
        let input = self.players[i].as_ref().expect("Slot existiert").input;
        let ch = self.character(i).expect("lebt");
        let weapon = ch.arsenal.active;
        let ammo = ch.arsenal.slot(weapon).ammo;
        let pos = ch.core.pos;
        let dir = Vec2::new(input.target_x as f32, input.target_y as f32).normalize();

        let will_fire = presses > 0 || (weapon.full_auto() && input.fire_held() && ammo != Some(0));
        if !will_fire {
            return;
        }
        if ammo == Some(0) {
            self.character_mut(i).expect("lebt").arsenal.reload_timer =
                ms_to_ticks(NO_AMMO_DELAY_MS);
            self.events.push(Event::NoAmmo { player: i });
            return;
        }

        let start = pos + dir * (PHYS_SIZE * 0.75);
        let mut reload = 0;
        match weapon {
            Weapon::Hammer => {
                if self.hammer(i, pos, start) > 0 {
                    reload = HAMMER_HIT_DELAY;
                }
            }
            Weapon::Grenade => {
                let t = &self.tuning;
                self.projectiles.push(Projectile::new(
                    i,
                    start,
                    dir,
                    self.tick,
                    (TICKS_PER_SECOND as f32 * t.grenade_lifetime) as i32,
                    t.grenade_damage,
                ));
            }
            Weapon::Laser => {
                let laser = Laser::new(i, pos, dir, self.tuning.laser_reach, self.tick);
                self.lasers.push(laser);
                let idx = self.lasers.len() - 1;
                if !self.laser_bounce(idx) {
                    self.lasers.remove(idx);
                }
            }
        }
        self.events.push(Event::Fire {
            player: i,
            weapon,
            pos,
        });

        let delay = weapon.fire_delay(&self.tuning);
        if let Some(ch) = self.character_mut(i) {
            let slot = &mut ch.arsenal.slots[weapon.index()];
            if let Some(a) = &mut slot.ammo {
                *a -= 1;
            }
            ch.arsenal.reload_timer = if reload == 0 { delay } else { reload };
        }
    }

    /// Hammerschlag; liefert die Anzahl der Treffer.
    fn hammer(&mut self, i: usize, pos: Vec2, start: Vec2) -> usize {
        let targets: Vec<(usize, Vec2)> = self
            .positions()
            .into_iter()
            .enumerate()
            .filter_map(|(j, p)| Some((j, p?)))
            .filter(|&(j, p)| {
                j != i
                    && p.distance(start) < HAMMER_RADIUS + PHYS_SIZE
                    && self.collision.intersect_line(start, p).is_none()
            })
            .collect();
        let knockback = self.tuning.hammer_knockback;
        let damage = self.tuning.hammer_damage;
        for &(j, p) in &targets {
            let hit = if p.distance(start) > 0.0 {
                p - (p - start).normalize() * (PHYS_SIZE * 0.5)
            } else {
                start
            };
            self.events.push(Event::HammerHit { owner: i, pos: hit });
            let dir = if p.distance(pos) > 0.0 {
                (p - pos).normalize()
            } else {
                Vec2::new(0.0, -1.0)
            };
            let force = Vec2::new(0.0, -1.0) + (dir + Vec2::new(0.0, -1.1)).normalize() * knockback;
            self.take_damage(j, force, damage, Some(i), Weapon::Hammer);
        }
        targets.len()
    }

    /// Erster Spieler auf der Strecke `from`–`to` (Abstand < Körper + `radius`),
    /// außer `exclude`. Liefert Slot und Schnittpunkt.
    fn intersect_character(
        &self,
        from: Vec2,
        to: Vec2,
        radius: f32,
        exclude: Option<usize>,
    ) -> Option<(usize, Vec2)> {
        let mut closest = from.distance(to) * 100.0;
        let mut hit = None;
        for (j, p) in self.positions().into_iter().enumerate() {
            let Some(p) = p else { continue };
            if Some(j) == exclude {
                continue;
            }
            let at = Vec2::closest_point_on_segment(from, to, p);
            if p.distance(at) < PHYS_SIZE + radius {
                let len = from.distance(at);
                if len < closest {
                    closest = len;
                    hit = Some((j, at));
                }
            }
        }
        hit
    }

    /// Nächster Abschnitt eines Laserstrahls. `false` = Laser ist erloschen.
    fn laser_bounce(&mut self, idx: usize) -> bool {
        let mut l = self.lasers[idx].clone();
        l.eval_tick = self.tick;
        if l.energy < 0.0 {
            return false;
        }
        let owner = self.character(l.owner).is_some().then_some(l.owner);
        let to = l.pos + l.dir * l.energy;
        let hit_wall = self.collision.intersect_line_detail(l.pos, to);
        let to = hit_wall.map_or(to, |h| h.before);

        if let Some((target, at)) = self.intersect_character(l.pos, to, 0.0, owner) {
            l.from = l.pos;
            l.pos = at;
            l.energy = -1.0;
            let dir = (to - l.from).normalize();
            let (force, damage) = (dir * self.tuning.laser_knockback, self.tuning.laser_damage);
            self.lasers[idx] = l.clone();
            self.take_damage(target, force, damage, Some(l.owner), Weapon::Laser);
            return true;
        }
        if hit_wall.is_some() {
            l.from = l.pos;
            l.pos = to;
            let mut p = l.pos;
            let mut d = l.dir * 4.0;
            self.collision.move_point(&mut p, &mut d, 1.0);
            l.pos = p;
            l.dir = d.normalize();
            l.energy -= l.from.distance(l.pos) + self.tuning.laser_bounce_cost;
            l.bounces += 1;
            if l.bounces > self.tuning.laser_bounce_num {
                l.energy = -1.0;
            }
            self.events.push(Event::LaserBounce {
                owner: l.owner,
                pos: l.pos,
            });
        } else {
            l.from = l.pos;
            l.pos = to;
            l.energy = -1.0;
        }
        self.lasers[idx] = l;
        true
    }

    fn tick_lasers(&mut self) {
        let delay = (TICKS_PER_SECOND * self.tuning.laser_bounce_delay) as f32 / 1000.0;
        let mut i = 0;
        while i < self.lasers.len() {
            let due = (self.tick - self.lasers[i].eval_tick) as f32 > delay;
            if due && !self.laser_bounce(i) {
                self.lasers.remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn tick_projectiles(&mut self) {
        let mut i = 0;
        while i < self.projectiles.len() {
            let pr = self.projectiles[i].clone();
            let t = &self.tuning;
            let age = (self.tick - pr.start_tick) as f32;
            let prev = pr.pos_at((age - 1.0) / TICKS_PER_SECOND as f32, t);
            let mut cur = pr.pos_at(age / TICKS_PER_SECOND as f32, t);
            let wall = self.collision.intersect_line(prev, cur);
            if let Some((p, _)) = wall {
                cur = p;
            }
            let owner = self.character(pr.owner).is_some().then_some(pr.owner);
            let target = self.intersect_character(prev, cur, PROJECTILE_RADIUS, owner);
            if let Some((_, at)) = target {
                cur = at;
            }
            self.projectiles[i].lifespan -= 1;
            if target.is_some() || wall.is_some() || self.projectiles[i].lifespan < 0 {
                self.projectiles.remove(i);
                self.explosion(cur, pr.owner, Weapon::Grenade, pr.damage);
            } else {
                i += 1;
            }
        }
    }

    fn explosion(&mut self, pos: Vec2, owner: usize, weapon: Weapon, max_damage: i32) {
        self.events.push(Event::Explosion { owner, pos });
        let t = &self.tuning;
        let (radius, inner, max_force) = (
            t.explosion_radius,
            t.explosion_inner_radius,
            t.explosion_max_force,
        );
        let hits: Vec<(usize, Vec2, i32)> = self
            .positions()
            .into_iter()
            .enumerate()
            .filter_map(|(j, p)| {
                let p = p?;
                let diff = p - pos;
                let l = diff.length();
                if l >= radius + PHYS_SIZE {
                    return None;
                }
                let force = if l > 0.0 {
                    diff.normalize() * max_force
                } else {
                    Vec2::new(0.0, max_force)
                };
                let factor = 1.0 - ((l - inner) / (radius - inner)).clamp(0.0, 1.0);
                let damage = (factor * max_damage as f32) as i32;
                (damage != 0).then_some((j, force * factor, damage))
            })
            .collect();
        for (j, force, damage) in hits {
            self.take_damage(j, force, damage, Some(owner), weapon);
        }
    }

    /// Schaden und Kraft auf Slot `i`. Liefert `true`, wenn die Figur gestorben ist.
    fn take_damage(
        &mut self,
        i: usize,
        force: Vec2,
        damage: i32,
        from: Option<usize>,
        weapon: Weapon,
    ) -> bool {
        let prediction = self.prediction;
        let Some(ch) = self.character_mut(i) else {
            return false;
        };
        ch.core.vel += force;
        if prediction {
            return false;
        }
        // Eigenschaden halbiert (T-26)
        let mut dmg = if from == Some(i) {
            (damage / 2).max(1)
        } else {
            damage
        };
        let (old_health, old_armor) = (ch.health, ch.armor);
        if dmg > 0 {
            if ch.armor > 0 {
                if dmg > 1 {
                    ch.health -= 1;
                    dmg -= 1;
                }
                if dmg > ch.armor {
                    dmg -= ch.armor;
                    ch.armor = 0;
                } else {
                    ch.armor -= dmg;
                    dmg = 0;
                }
            }
            ch.health -= dmg;
        }
        let (health, armor, dead) = (old_health - ch.health, old_armor - ch.armor, ch.health <= 0);
        self.events.push(Event::Damage {
            player: i,
            from,
            health,
            armor,
        });
        if dead {
            self.die(i, from, DeathCause::Weapon(weapon));
        }
        dead
    }

    /// Tötet die Figur in Slot `i`.
    pub fn die(&mut self, i: usize, killer: Option<usize>, cause: DeathCause) {
        let tick = self.tick;
        let delay = secs_to_ticks(self.tuning.respawn_delay);
        let Some(p) = self.players.get_mut(i).and_then(Option::as_mut) else {
            return;
        };
        let Some(ch) = p.character.take() else { return };
        p.die_tick = tick;
        p.respawn_tick = tick + delay;
        p.spawning = false;
        self.events.push(Event::Death {
            player: i,
            killer,
            cause,
            pos: ch.core.pos,
        });
    }

    // ---------------------------------------------------------------- Pickups

    fn tick_pickups(&mut self) {
        if self.prediction {
            return;
        }
        for k in 0..self.pickups.len() {
            let pk = self.pickups[k].clone();
            if let Some(t) = pk.respawn_tick {
                if self.tick > t {
                    self.pickups[k].respawn_tick = None;
                    self.events.push(Event::PickupRespawn {
                        kind: pk.kind,
                        pos: pk.pos,
                    });
                } else {
                    continue;
                }
            }
            // nächste Figur im Radius (wie `ClosestEntity`: zusätzlich < 2 · Radius)
            let mut best: Option<(usize, f32)> = None;
            for (j, p) in self.positions().into_iter().enumerate() {
                let Some(p) = p else { continue };
                let len = pk.pos.distance(p);
                if len < PHYS_SIZE + PICKUP_RADIUS
                    && len < best.map_or(PICKUP_RADIUS * 2.0, |b| b.1)
                {
                    best = Some((j, len));
                }
            }
            let Some((j, _)) = best else { continue };
            let t = self.tuning.clone();
            let ch = self.character_mut(j).expect("lebt");
            let picked = match pk.kind {
                PickupKind::Health if ch.health < t.max_health => {
                    ch.health = (ch.health + 1).min(t.max_health);
                    true
                }
                PickupKind::Armor if ch.armor < t.max_armor => {
                    ch.armor = (ch.armor + 1).min(t.max_armor);
                    true
                }
                PickupKind::Weapon(w) => ch.arsenal.give(w, t.max_ammo, t.max_ammo),
                _ => false,
            };
            if picked {
                self.pickups[k].respawn_tick = Some(self.tick + secs_to_ticks(t.pickup_respawn));
                self.events.push(Event::Pickup {
                    player: j,
                    kind: pk.kind,
                    pos: pk.pos,
                });
            }
        }
    }

    // ---------------------------------------------------------------- Figuren

    fn tick_characters(&mut self) {
        let positions = self.positions();
        let mut drag = vec![Vec2::ZERO; self.players.len()];
        for i in 0..self.players.len() {
            let Some(p) = self.players[i].as_mut() else {
                continue;
            };
            let remote = matches!(p.controller, Controller::Remote);
            let input = p.input;
            let Some(ch) = p.character.as_mut() else {
                continue;
            };
            ch.core.tick(
                (!remote).then_some(&input),
                &self.tuning,
                &self.collision,
                i,
                &positions,
                &mut drag,
            );
            // Waffen: Reload herunterzählen, sonst Dauerfeuer
            let ready = ch.arsenal.reload_timer == 0;
            ch.arsenal.reload_timer = ch.arsenal.reload_timer.saturating_sub(1);
            if ready && !remote {
                self.fire_weapon(i, 0);
            }
        }
        for (slot, d) in self.players.iter_mut().zip(drag) {
            if let Some(ch) = slot.as_mut().and_then(|p| p.character.as_mut()) {
                ch.core.hook_drag_vel += d;
            }
        }
    }

    fn tick_characters_deferred(&mut self) {
        for i in 0..self.players.len() {
            let positions = self.positions();
            let Some(ch) = self.players[i].as_mut().and_then(|p| p.character.as_mut()) else {
                continue;
            };
            ch.core
                .apply_drag_and_move(&self.tuning, &self.collision, i, &positions);
            if ch.core.death && !self.prediction {
                self.die(i, None, DeathCause::World);
            }
        }
    }

    fn tick_respawns(&mut self) {
        if self.prediction {
            return;
        }
        let auto = secs_to_ticks(self.tuning.auto_respawn);
        for i in 0..self.players.len() {
            let Some(p) = self.players[i].as_mut() else {
                continue;
            };
            if p.character.is_some() {
                continue;
            }
            if p.die_tick + auto <= self.tick {
                p.spawning = true;
            }
            if !p.spawning || p.respawn_tick > self.tick {
                continue;
            }
            let pos = match &p.controller {
                Controller::Dummy { home, .. } => Some(*home),
                Controller::Human | Controller::Remote => self.best_spawn(),
            };
            if let Some(pos) = pos {
                self.spawn_character(i, pos);
            }
        }
    }

    /// Spawnpunkt mit dem geringsten „Gefahrenwert“ (Summe 1/Abstand zu allen
    /// Figuren), freie Nachbarposition wie im Original (`EvaluateSpawnType`).
    pub fn best_spawn(&self) -> Option<Vec2> {
        const OFFSETS: [Vec2; 5] = [
            Vec2::new(0.0, 0.0),
            Vec2::new(-32.0, 0.0),
            Vec2::new(0.0, -32.0),
            Vec2::new(32.0, 0.0),
            Vec2::new(0.0, 32.0),
        ];
        let alive: Vec<Vec2> = self.positions().into_iter().flatten().collect();
        let mut best: Option<(f32, Vec2)> = None;
        for &sp in &self.spawn_points {
            let near: Vec<Vec2> = alive
                .iter()
                .copied()
                .filter(|p| p.distance(sp) < 64.0 + PHYS_SIZE)
                .collect();
            let free = OFFSETS.iter().map(|&o| sp + o).find(|&pos| {
                near.iter()
                    .all(|&c| !self.collision.is_solid(pos) && c.distance(pos) > PHYS_SIZE)
            });
            let Some(pos) = free else { continue };
            let score: f32 = alive
                .iter()
                .map(|&c| {
                    let d = c.distance(pos);
                    if d == 0.0 { 1_000_000_000.0 } else { 1.0 / d }
                })
                .sum();
            if best.is_none_or(|(s, _)| s > score) {
                best = Some((score, pos));
            }
        }
        best.map(|(_, p)| p)
    }

    /// Nur für Tests und Werkzeuge: Kern einer lebenden Figur.
    pub fn core(&self, i: usize) -> Option<&CharacterCore> {
        self.character(i).map(|c| &c.core)
    }
}

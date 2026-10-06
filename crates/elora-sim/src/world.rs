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
use crate::creature::{Creature, CreatureKind, CreatureShot, HookTarget, Loot};
use crate::dummy::{DummyBrain, DummyPattern};
use crate::entities::{Flag, Laser, Pickup, Projectile};
use crate::event::{DeathCause, Event, PickupKind};
use crate::input::{PlayerInput, count_presses};
use crate::math::Vec2;
use crate::player::{Character, Controller, Player, Team};
use crate::tuning::{Tuning, ms_to_ticks, secs_to_ticks};
use crate::weapon::Weapon;

/// Sperre nach Feuern ohne Munition (Original: 125 ms).
const NO_AMMO_DELAY_MS: u32 = 125;
/// Sperre nach einem Hammer-Treffer (Original: 1/3 s).
const HAMMER_HIT_DELAY: u32 = TICKS_PER_SECOND / 3;
/// Radius eines Projektils bei der Treffer-Prüfung.
const PROJECTILE_RADIUS: f32 = 6.0;
/// Aufnahme-Radius eines Pickups (Original: 20, effektiv < 40 durch `ClosestEntity`).
const PICKUP_RADIUS: f32 = 20.0;

/// Die Schalter (`friendly_fire`, `pickups_enabled`, `paused`, `prediction`) sind
/// unabhängige Welt-Einstellungen, keine Zustandsmaschine.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct World {
    pub tuning: Tuning,
    pub collision: Collision,
    pub tick: u64,
    /// Ein Slot pro Spieler; `None` = unbesetzt.
    pub players: Vec<Option<Player>>,
    pub projectiles: Vec<Projectile>,
    pub lasers: Vec<Laser>,
    pub pickups: Vec<Pickup>,
    /// Neutrale Spawnpunkte für menschliche Spieler.
    pub spawn_points: Vec<Vec2>,
    /// Team-Spawnpunkte (Rot, Blau).
    pub team_spawns: [Vec<Vec2>; 2],
    /// Flaggen (CTF); leer = kein CTF.
    pub flags: Vec<Flag>,
    /// Flaggenstände aus der Karte (Rot, Blau) – die Regeln legen daraus Flaggen an.
    pub flag_stands: [Option<Vec2>; 2],
    /// Schaden an Teammitgliedern (E-069). Aus: nur Rückstoß wie im Original.
    pub friendly_fire: bool,
    /// Pickups aktiv (Instagib schaltet sie ab, E-026).
    pub pickups_enabled: bool,
    /// Eingefroren (Countdown): Figuren und Objekte stehen, Zeiten laufen mit.
    pub paused: bool,
    /// Ereignisse des letzten Ticks.
    pub events: Vec<Event>,
    /// Client-Vorhersage (E-057): Kräfte wirken, aber kein Schaden, kein Tod, keine
    /// Pickups und kein Respawn – das entscheidet allein der Server.
    pub prediction: bool,
    /// Abenteuer-Regeln: Schutz nach Treffern (E-234), kein Eigenschaden (E-237).
    pub adventure: bool,
    /// Gegnerarten (A1.2); Kreaturen verweisen per Index darauf.
    pub creature_kinds: Vec<CreatureKind>,
    pub creatures: Vec<Creature>,
    pub creature_shots: Vec<CreatureShot>,
    /// Zeitweise gesetzte Tiles (Wurzelwände): Tile-Position, ursprüngliches Tile, Ende (Tick).
    pub temp_tiles: Vec<(i32, i32, crate::Tile, u64)>,
    pub loot: Vec<Loot>,
    /// Nächste Id für Kreaturen und Beute.
    pub next_id: u32,
    /// Wetter (R2-W1, nur Abenteuer; die Sitzung setzt es) und Zustand der Blitze.
    pub weather: Option<crate::weather::WeatherEnv>,
    pub lightning: crate::weather::Lightning,
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
            team_spawns: [Vec::new(), Vec::new()],
            flags: Vec::new(),
            flag_stands: [None, None],
            friendly_fire: true,
            pickups_enabled: true,
            paused: false,
            events: Vec::new(),
            prediction: false,
            adventure: false,
            creature_kinds: Vec::new(),
            creatures: Vec::new(),
            creature_shots: Vec::new(),
            temp_tiles: Vec::new(),
            loot: Vec::new(),
            next_id: 1,
            weather: None,
            lightning: crate::weather::Lightning::default(),
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
            let mut ch = Character::spawn(pos, max_health);
            ch.core.abilities = p.abilities;
            p.character = Some(ch);
            p.spawning = false;
            if let Controller::Dummy { brain, .. } = &mut p.controller {
                *brain = DummyBrain::default();
            }
            self.events.push(Event::Spawn { player: i, pos });
        }
    }

    /// Fähigkeiten eines Spielers setzen – gilt sofort und nach jedem Respawn.
    pub fn set_abilities(&mut self, i: usize, abilities: crate::Abilities) {
        if let Some(p) = self.players.get_mut(i).and_then(Option::as_mut) {
            p.abilities = abilities;
            if let Some(ch) = &mut p.character {
                ch.core.abilities = abilities;
            }
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
        if self.paused {
            self.step_paused(inputs);
            return;
        }
        self.apply_inputs(inputs);

        self.tick += 1;
        self.tick_projectiles();
        self.tick_lasers();
        self.tick_pickups();
        self.update_hook_wilt();
        self.tick_weather();
        self.tick_characters();
        self.tick_flags_physics();
        self.tick_characters_deferred();
        if self.adventure {
            self.remember_safe_ground();
        }
        self.tick_creatures();
        self.tick_respawns();
        self.tick_flags_rules();
    }

    /// Eingefrorener Tick (wie `TickPaused` im Original): Eingaben merken, alle
    /// Zeitstempel mitschieben, nichts bewegen.
    fn step_paused(&mut self, inputs: &[PlayerInput]) {
        for (i, p) in self.players.iter_mut().enumerate() {
            let Some(p) = p else { continue };
            if matches!(p.controller, Controller::Human) {
                p.prev_input = p.input;
                p.input = inputs.get(i).copied().unwrap_or_default();
            }
            p.die_tick += 1;
            p.respawn_tick += 1;
        }
        for pr in &mut self.projectiles {
            pr.start_tick += 1;
        }
        for l in &mut self.lasers {
            l.start_tick += 1;
            l.eval_tick += 1;
        }
        for pk in &mut self.pickups {
            if let Some(t) = &mut pk.respawn_tick {
                *t += 1;
            }
        }
        for f in &mut self.flags {
            f.drop_tick += 1;
            if f.grab_tick != 0 {
                f.grab_tick += 1;
            }
        }
        self.tick += 1;
    }

    /// Team eines Slots.
    pub fn team(&self, i: usize) -> Team {
        self.player(i).map_or(Team::None, |p| p.team)
    }

    /// Neue Runde/neues Match: Schüsse weg, Pickups und Flaggen zurück, alle Figuren
    /// (außer Zuschauern und gesperrten) sofort neu am Spawnpunkt.
    pub fn reset_round(&mut self) {
        self.projectiles.clear();
        self.lasers.clear();
        for pk in &mut self.pickups {
            pk.respawn_tick = None;
        }
        for f in &mut self.flags {
            f.reset();
        }
        for p in self.players.iter_mut().flatten() {
            p.character = None;
            p.spawning = false;
        }
        for i in 0..self.players.len() {
            let Some(p) = &self.players[i] else { continue };
            if p.team == Team::Spectator || p.respawn_disabled {
                continue;
            }
            let pos = match &p.controller {
                Controller::Dummy { home, .. } => Some(*home),
                _ => self.best_spawn_for(p.team),
            };
            if let Some(pos) = pos {
                self.spawn_character(i, pos);
            }
        }
    }

    /// Selbstmord (`kill`, E-055): wie im Original 3 s bis zum Respawn.
    pub fn kill(&mut self, i: usize) {
        if self.character(i).is_none() {
            return;
        }
        self.die(i, Some(i), DeathCause::Suicide);
        let tick = self.tick;
        if let Some(p) = self.players.get_mut(i).and_then(Option::as_mut) {
            p.respawn_tick = tick + 3 * u64::from(TICKS_PER_SECOND);
        }
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
                let mut p = Projectile::new(
                    i,
                    start,
                    dir,
                    self.tick,
                    (TICKS_PER_SECOND as f32 * t.grenade_lifetime) as i32,
                    t.grenade_damage,
                );
                // Wind lenkt die Granate ab (R2-W1, nur Abenteuer)
                p.wind = self.collision.wind;
                self.projectiles.push(p);
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
                    && p.distance(start) < self.tuning.hammer_reach + PHYS_SIZE
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
            self.take_damage(
                j,
                force,
                damage,
                Some(i),
                DeathCause::Weapon(Weapon::Hammer),
            );
        }
        targets.len() + self.hammer_creatures(i, pos, start)
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

        let player_hit = self.intersect_character(l.pos, to, 0.0, owner);
        let limit = player_hit.map_or(to, |(_, p)| p);
        // Gegner bis zum ersten Spieler; mit Durchschlag (A-21) mehrere hintereinander
        let creatures = self.creatures_on_line(l.pos, limit);
        if !creatures.is_empty() {
            let dir = (to - l.pos).normalize();
            let (force, damage) = (dir * self.tuning.laser_knockback, self.tuning.laser_damage);
            let n = 1 + self.tuning.laser_pierce as usize;
            for &(id, _) in creatures.iter().take(n) {
                if let Some(c) = self.creatures.iter().position(|c| c.id == id) {
                    self.damage_creature(c, force, damage, Some(l.owner), Some(l.pos));
                }
            }
            if creatures.len() >= n || self.tuning.laser_pierce == 0 {
                l.from = l.pos;
                l.pos = creatures[(n - 1).min(creatures.len() - 1)].1;
                l.energy = -1.0;
                self.lasers[idx] = l;
                return true;
            }
        }
        if let Some((target, at)) = player_hit {
            l.from = l.pos;
            l.pos = at;
            l.energy = -1.0;
            let dir = (to - l.from).normalize();
            let (force, damage) = (dir * self.tuning.laser_knockback, self.tuning.laser_damage);
            self.lasers[idx] = l.clone();
            self.take_damage(
                target,
                force,
                damage,
                Some(l.owner),
                DeathCause::Weapon(Weapon::Laser),
            );
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
            let creature = self.intersect_creature(prev, cur, PROJECTILE_RADIUS);
            if let Some((_, at)) = creature {
                cur = at;
            }
            self.projectiles[i].lifespan -= 1;
            if target.is_some()
                || creature.is_some()
                || wall.is_some()
                || self.projectiles[i].lifespan < 0
            {
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
            self.take_damage(j, force, damage, Some(owner), DeathCause::Weapon(weapon));
        }
        self.explode_creatures(pos, pos, owner, max_damage);
        if weapon == Weapon::Grenade {
            self.grenade_shards(pos, owner, max_damage);
        }
    }

    /// Schaden und Kraft auf Slot `i`. Liefert `true`, wenn die Figur gestorben ist.
    pub(crate) fn take_damage(
        &mut self,
        i: usize,
        force: Vec2,
        damage: i32,
        from: Option<usize>,
        cause: DeathCause,
    ) -> bool {
        let prediction = self.prediction;
        let (tick, adventure) = (self.tick, self.adventure);
        let invulnerable = u64::from(ms_to_ticks(self.tuning.hit_invulnerable));
        let Some(ch) = self.character_mut(i) else {
            return false;
        };
        // Abenteuer: eigene Granaten nur mit Rückstoß (E-237), kurz unverwundbar nach Treffer (E-234)
        if (adventure || cause == DeathCause::Creature)
            && from != Some(i)
            && tick < ch.invulnerable_until
        {
            return false;
        }
        ch.core.vel += force;
        if prediction || (adventure && from == Some(i)) {
            return false;
        }
        // Friendly Fire (E-069): ausgeschaltet → nur Rückstoß wie im Original
        if !self.friendly_fire
            && let Some(f) = from
            && f != i
            && self.team(f).is_mate(self.team(i))
        {
            return false;
        }
        let Some(ch) = self.character_mut(i) else {
            return false;
        };
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
        if (adventure || cause == DeathCause::Creature) && ch.health < old_health {
            ch.invulnerable_until = tick + invulnerable;
        }
        let (health, armor, dead) = (old_health - ch.health, old_armor - ch.armor, ch.health <= 0);
        self.events.push(Event::Damage {
            player: i,
            from,
            health,
            armor,
        });
        if dead {
            self.die(i, from, cause);
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
        // getragene Flagge fällt
        for f in &mut self.flags {
            if f.carrier == Some(i) {
                f.carrier = None;
                f.vel = Vec2::ZERO;
                f.drop_tick = tick;
                self.events.push(Event::FlagDrop {
                    team: f.team,
                    player: i,
                    pos: f.pos,
                });
            }
        }
    }

    // ---------------------------------------------------------------- Pickups

    fn tick_pickups(&mut self) {
        if self.prediction || !self.pickups_enabled {
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
            let adventure = self.adventure;
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
                // Abenteuer: Waffen gibt es nur über den Fortschritt, Pickups füllen Munition (E-243)
                PickupKind::Weapon(w) if adventure && !ch.arsenal.has(w) => false,
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
        let targets: Vec<HookTarget> = self.hook_targets();
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
                &targets,
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
            let landed = ch.core.triggered_events & crate::character::events::STOMP_LAND != 0;
            let feet = ch.core.pos + Vec2::new(0.0, PHYS_SIZE / 2.0);
            let buried = ch.core.buried(&self.collision);
            if (ch.core.death || buried) && !self.prediction {
                let damage = if ch.core.death {
                    self.tuning.thorn_damage
                } else {
                    self.tuning.quicksand_damage
                };
                if self.adventure {
                    self.back_to_safe_ground(i, damage);
                } else {
                    self.die(i, None, DeathCause::World);
                }
            } else if landed {
                self.stomp_wave(i, feet);
            }
        }
    }

    /// Dornen und Treibsand im Abenteuer (E-283, E-318): Schaden, dann zurück auf den letzten
    /// sicheren Boden.
    fn back_to_safe_ground(&mut self, i: usize, damage: i32) {
        if self.take_damage(i, Vec2::ZERO, damage, None, DeathCause::World) {
            return;
        }
        let Some(ch) = self.character_mut(i) else {
            return;
        };
        let core = CharacterCore {
            direction: ch.core.direction,
            angle: ch.core.angle,
            abilities: ch.core.abilities,
            ..CharacterCore::new(ch.safe_pos)
        };
        ch.core = core;
    }

    /// Sicheren Boden merken: fester Boden ohne Dornen und Treibsand in der Nähe.
    fn remember_safe_ground(&mut self) {
        let collision = &self.collision;
        for p in self.players.iter_mut().flatten() {
            let Some(ch) = p.character.as_mut() else {
                continue;
            };
            let core = &ch.core;
            if !matches!(
                core.ground_tile(collision),
                Some(crate::Tile::Solid | crate::Tile::Unhookable | crate::Tile::Ice)
            ) {
                continue;
            }
            let ts = crate::TILE_SIZE as f32;
            let near_thorns = (-2..=2).any(|dx: i8| {
                (0..=3).any(|dy: i8| {
                    let at = core.pos + Vec2::new(f32::from(dx) * ts, f32::from(dy) * ts);
                    matches!(
                        collision.tile_at(at),
                        crate::Tile::Death | crate::Tile::Quicksand
                    )
                })
            });
            if !near_thorns {
                ch.safe_pos = core.pos;
            }
        }
    }

    /// Stoßwelle beim Aufprall des Stampfens (A-05): bricht Bröckelboden im Radius (E-230).
    fn stomp_wave(&mut self, player: usize, pos: Vec2) {
        self.events.push(Event::Stomp { player, pos });
        self.stomp_creatures(player, pos);
        if self.prediction {
            return;
        }
        let r = self.tuning.stomp_radius;
        let tile = crate::TILE_SIZE as f32;
        #[allow(clippy::cast_possible_truncation)]
        let (x0, x1, y0, y1) = (
            ((pos.x - r) / tile).floor() as i32,
            ((pos.x + r) / tile).floor() as i32,
            ((pos.y - r) / tile).floor() as i32,
            ((pos.y + r) / tile).floor() as i32,
        );
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                #[allow(clippy::cast_precision_loss)]
                let center = Vec2::new((tx as f32 + 0.5) * tile, (ty as f32 + 0.5) * tile);
                if self.collision.tile(tx, ty) == crate::Tile::Crumble && center.distance(pos) <= r
                {
                    self.collision.set_tile(tx, ty, crate::Tile::Air);
                    self.events.push(Event::TileBroken { tx, ty });
                }
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
            if p.character.is_some() || p.team == Team::Spectator || p.respawn_disabled {
                continue;
            }
            if p.die_tick + auto <= self.tick {
                p.spawning = true;
            }
            if !p.spawning || p.respawn_tick > self.tick {
                continue;
            }
            let (home, team) = match &p.controller {
                Controller::Dummy { home, .. } => (Some(*home), p.team),
                Controller::Human | Controller::Remote => (None, p.team),
            };
            let pos = home.or_else(|| self.best_spawn_for(team));
            if let Some(pos) = pos {
                self.spawn_character(i, pos);
            }
        }
    }

    /// Spawnpunkt mit dem geringsten „Gefahrenwert“ (Summe 1/Abstand zu allen
    /// Figuren), freie Nachbarposition wie im Original (`EvaluateSpawnType`).
    pub fn best_spawn(&self) -> Option<Vec2> {
        self.best_spawn_for(Team::None)
    }

    /// Wie im Original (`CanSpawn`): Teams zuerst eigene Spawnpunkte, dann neutrale,
    /// dann gegnerische; ohne Team alle. Teammitglieder zählen halb als Gefahr.
    pub fn best_spawn_for(&self, team: Team) -> Option<Vec2> {
        match team.index() {
            Some(t) => self
                .eval_spawns(&self.team_spawns[t], team)
                .or_else(|| self.eval_spawns(&self.spawn_points, team))
                .or_else(|| self.eval_spawns(&self.team_spawns[1 - t], team))
                .map(|(_, p)| p),
            None => [
                &self.spawn_points,
                &self.team_spawns[0],
                &self.team_spawns[1],
            ]
            .into_iter()
            .filter_map(|pts| self.eval_spawns(pts, team))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, p)| p),
        }
    }

    fn eval_spawns(&self, points: &[Vec2], team: Team) -> Option<(f32, Vec2)> {
        const OFFSETS: [Vec2; 5] = [
            Vec2::new(0.0, 0.0),
            Vec2::new(-32.0, 0.0),
            Vec2::new(0.0, -32.0),
            Vec2::new(32.0, 0.0),
            Vec2::new(0.0, 32.0),
        ];
        let alive: Vec<(Vec2, Team)> = self
            .players
            .iter()
            .flatten()
            .filter_map(|p| Some((p.character.as_ref()?.core.pos, p.team)))
            .collect();
        let mut best: Option<(f32, Vec2)> = None;
        for &sp in points {
            let near: Vec<Vec2> = alive
                .iter()
                .map(|a| a.0)
                .filter(|p| p.distance(sp) < 64.0 + PHYS_SIZE)
                .collect();
            let free = OFFSETS.iter().map(|&o| sp + o).find(|&pos| {
                near.iter()
                    .all(|&c| !self.collision.is_solid(pos) && c.distance(pos) > PHYS_SIZE)
            });
            let Some(pos) = free else { continue };
            let score: f32 = alive
                .iter()
                .map(|&(c, t)| {
                    let modifier = if team.is_mate(t) { 0.5 } else { 1.0 };
                    let d = c.distance(pos);
                    modifier * if d == 0.0 { 1_000_000_000.0 } else { 1.0 / d }
                })
                .sum();
            if best.is_none_or(|(s, _)| s > score) {
                best = Some((score, pos));
            }
        }
        best
    }

    // ---------------------------------------------------------------- Flaggen (CTF)

    /// Physik (wie `CFlag::TickDefered`): folgt dem Träger, fällt sonst, kehrt nach
    /// 30 s oder auf Todes-Tiles zurück.
    fn tick_flags_physics(&mut self) {
        let tick = self.tick;
        let gravity = self.tuning.gravity;
        let return_after = Flag::RETURN_SECS * u64::from(TICKS_PER_SECOND);
        for k in 0..self.flags.len() {
            let carrier_pos = self.flags[k]
                .carrier
                .and_then(|c| self.core(c))
                .map(|c| c.pos);
            let f = &mut self.flags[k];
            if let Some(p) = carrier_pos {
                f.pos = p;
                continue;
            }
            if self.prediction {
                continue;
            }
            let on_death = self.collision.tile_at(f.pos) == crate::Tile::Death;
            if on_death || (!f.at_stand && tick > f.drop_tick + return_after) {
                f.reset();
                self.events.push(Event::FlagReturn {
                    team: f.team,
                    player: None,
                });
                continue;
            }
            if !f.at_stand {
                f.vel.y += gravity;
                let size = Vec2::new(Flag::PHYS_SIZE, Flag::PHYS_SIZE);
                self.collision.move_box(&mut f.pos, &mut f.vel, size, 0.5);
            }
        }
    }

    /// Aufnehmen, Zurückbringen, Erobern (wie `CGameControllerCTF::Tick`).
    fn tick_flags_rules(&mut self) {
        if self.prediction || self.flags.len() != 2 {
            return;
        }
        let reach = Flag::PHYS_SIZE + PHYS_SIZE;
        for k in 0..2 {
            let other = 1 - k;
            if let Some(carrier) = self.flags[k].carrier {
                // Eroberung: Träger an der eigenen Flagge, die am Stand ist
                if self.flags[other].at_stand
                    && self.flags[k].pos.distance(self.flags[other].pos) < reach
                {
                    let ticks = self.tick.saturating_sub(self.flags[k].grab_tick);
                    let team = self.flags[k].team;
                    self.events.push(Event::FlagCapture {
                        team,
                        player: carrier,
                        ticks,
                    });
                    self.flags[0].reset();
                    self.flags[1].reset();
                }
                continue;
            }
            let f = self.flags[k].clone();
            let touching: Vec<(usize, Team)> = self
                .players
                .iter()
                .enumerate()
                .filter_map(|(i, p)| {
                    let p = p.as_ref()?;
                    let c = p.character.as_ref()?;
                    let near = c.core.pos.distance(f.pos) < reach;
                    let visible = self.collision.intersect_line(f.pos, c.core.pos).is_none();
                    (near && visible && p.team.index().is_some()).then_some((i, p.team))
                })
                .collect();
            for (i, team) in touching {
                if team == f.team {
                    if !f.at_stand {
                        self.flags[k].reset();
                        self.events.push(Event::FlagReturn {
                            team: f.team,
                            player: Some(i),
                        });
                    }
                } else {
                    let from_stand = f.at_stand;
                    let flag = &mut self.flags[k];
                    flag.carrier = Some(i);
                    if from_stand {
                        flag.grab_tick = self.tick;
                    }
                    flag.at_stand = false;
                    self.events.push(Event::FlagGrab {
                        team: f.team,
                        player: i,
                        from_stand,
                    });
                    break;
                }
            }
        }
    }

    /// Nur für Tests und Werkzeuge: Kern einer lebenden Figur.
    pub fn core(&self, i: usize) -> Option<&CharacterCore> {
        self.character(i).map(|c| &c.core)
    }
}

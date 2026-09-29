//! Welt-Snapshots mit feldweisem Delta (E-063).
//!
//! Jedes Objekt ist ein festes Array ganzer Zahlen. Das Delta gegen einen Basis-
//! Snapshot schreibt pro Objekt eine Änderungsmaske (1 Bit je Feld) und nur die
//! geänderten Felder als ZigZag-Differenz. Positionen und Geschwindigkeiten sind in
//! der Simulation quantisiert (E-021) – der Rundlauf ist daher bit-genau, und die
//! Client-Vorhersage startet exakt vom Server-Zustand.

// Wire-Format: Umwandlungen zwischen den Feld-Ganzzahlen und den Simulationstypen.
// Wertebereiche werden beim Dekodieren geprüft (`validate`, `Reader::int`).
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]

use std::collections::BTreeMap;

use elora_sim::character::HookState;
use elora_sim::entities::{Laser, Projectile};
use elora_sim::weapon::{Arsenal, WeaponSlot};
use elora_sim::{Character, CharacterCore, Controller, Player, Vec2, Weapon, World};

use crate::codec::{DecodeError, DecodeResult, Reader, Writer};

/// Objektarten in fester Reihenfolge.
const KINDS: usize = 4;
const PLAYER: usize = 0;
const PROJECTILE: usize = 1;
const LASER: usize = 2;
const PICKUP: usize = 3;
/// Felder pro Objektart.
const FIELDS: [usize; KINDS] = [28, 8, 12, 1];
/// Obergrenze für Objekte pro Art (Schutz gegen manipulierte Pakete).
const MAX_OBJECTS: usize = 4096;

/// Ein Snapshot: pro Objektart eine Tabelle Schlüssel → Felder.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Snapshot {
    pub tick: u64,
    objects: [BTreeMap<u64, Vec<i64>>; KINDS],
}

fn shot_key(owner: usize, tick: u64) -> u64 {
    ((owner as u64) << 48) | (tick & 0xffff_ffff_ffff)
}

fn key_owner(key: u64) -> usize {
    (key >> 48) as usize
}

fn f(v: f32) -> i64 {
    // quantisierte Werte sind ganzzahlig und exakt darstellbar
    v as i64
}

fn bits(v: f32) -> i64 {
    i64::from(v.to_bits())
}

fn unbits(v: i64) -> f32 {
    f32::from_bits(v as u32)
}

fn hook_code(h: HookState) -> i64 {
    match h {
        HookState::Retracted => 0,
        HookState::Idle => 1,
        HookState::Retracting(n) => 1 + i64::from(n),
        HookState::Flying => 5,
        HookState::Grabbed => 6,
    }
}

fn hook_state(c: i64) -> HookState {
    match c {
        0 => HookState::Retracted,
        2..=4 => HookState::Retracting((c - 1) as u8),
        5 => HookState::Flying,
        6 => HookState::Grabbed,
        _ => HookState::Idle,
    }
}

fn weapon_code(w: Option<Weapon>) -> i64 {
    w.map_or(-1, |w| w.index() as i64)
}

fn weapon(c: i64) -> Option<Weapon> {
    usize::try_from(c)
        .ok()
        .and_then(|i| Weapon::ALL.get(i).copied())
}

fn ammo_code(s: &WeaponSlot) -> i64 {
    match (s.got, s.ammo) {
        (false, _) => -2,
        (true, None) => -1,
        (true, Some(a)) => i64::from(a),
    }
}

fn ammo_slot(c: i64) -> WeaponSlot {
    match c {
        -2 => WeaponSlot {
            got: false,
            ammo: None,
        },
        -1 => WeaponSlot {
            got: true,
            ammo: None,
        },
        a => WeaponSlot {
            got: true,
            ammo: Some(a as i32),
        },
    }
}

impl Snapshot {
    /// Momentaufnahme des Server-Zustands.
    pub fn from_world(world: &World) -> Self {
        let mut s = Self {
            tick: world.tick,
            ..Self::default()
        };
        for (i, p) in world.players.iter().enumerate() {
            let Some(p) = p else { continue };
            s.objects[PLAYER].insert(i as u64, player_fields(p));
        }
        for pr in &world.projectiles {
            let fields = vec![
                pr.owner as i64,
                f(pr.pos.x),
                f(pr.pos.y),
                (pr.dir.x * 100.0).round() as i64,
                (pr.dir.y * 100.0).round() as i64,
                pr.start_tick as i64,
                i64::from(pr.lifespan),
                i64::from(pr.damage),
            ];
            s.objects[PROJECTILE].insert(shot_key(pr.owner, pr.start_tick), fields);
        }
        for l in &world.lasers {
            let fields = vec![
                l.owner as i64,
                l.start_tick as i64,
                l.eval_tick as i64,
                i64::from(l.bounces),
                bits(l.energy),
                bits(l.pos.x),
                bits(l.pos.y),
                bits(l.from.x),
                bits(l.from.y),
                bits(l.dir.x),
                bits(l.dir.y),
                0,
            ];
            s.objects[LASER].insert(shot_key(l.owner, l.start_tick), fields);
        }
        for (k, pk) in world.pickups.iter().enumerate() {
            let v = pk.respawn_tick.map_or(0, |t| t as i64 + 1);
            s.objects[PICKUP].insert(k as u64, vec![v]);
        }
        s
    }

    /// Überträgt den Snapshot in eine Client-Welt (Karte, Tuning und Pickup-Positionen
    /// stammen aus der Karte). Alle Figuren werden `Remote`, außer `local` (Human).
    pub fn apply_to(&self, world: &mut World, local: Option<usize>) {
        world.tick = self.tick;
        let len = self.objects[PLAYER]
            .keys()
            .next_back()
            .map_or(0, |k| *k as usize + 1);
        world.players = (0..len).map(|_| None).collect();
        for (&i, v) in &self.objects[PLAYER] {
            let i = i as usize;
            let controller = if Some(i) == local {
                Controller::Human
            } else {
                Controller::Remote
            };
            world.players[i] = Some(player_from(v, controller));
        }
        world.projectiles = self.objects[PROJECTILE]
            .values()
            .map(|v| Projectile {
                owner: v[0] as usize,
                pos: Vec2::new(v[1] as f32, v[2] as f32),
                dir: Vec2::new(v[3] as f32 / 100.0, v[4] as f32 / 100.0),
                start_tick: v[5] as u64,
                lifespan: v[6] as i32,
                damage: v[7] as i32,
            })
            .collect();
        world.lasers = self.objects[LASER]
            .values()
            .map(|v| Laser {
                owner: v[0] as usize,
                start_tick: v[1] as u64,
                eval_tick: v[2] as u64,
                bounces: v[3] as u32,
                energy: unbits(v[4]),
                pos: Vec2::new(unbits(v[5]), unbits(v[6])),
                from: Vec2::new(unbits(v[7]), unbits(v[8])),
                dir: Vec2::new(unbits(v[9]), unbits(v[10])),
            })
            .collect();
        for (&k, v) in &self.objects[PICKUP] {
            if let Some(pk) = world.pickups.get_mut(k as usize) {
                pk.respawn_tick = u64::try_from(v[0] - 1).ok();
            }
        }
        world.events.clear();
    }

    /// Prüfsumme über alle Felder (FNV-1a), um fehlerhafte Deltas zu erkennen.
    pub fn checksum(&self) -> u32 {
        let mut h: u32 = 0x811c_9dc5;
        let mut eat = |v: i64| {
            for b in v.to_le_bytes() {
                h ^= u32::from(b);
                h = h.wrapping_mul(0x0100_0193);
            }
        };
        eat(self.tick as i64);
        for table in &self.objects {
            for (&k, v) in table {
                eat(k as i64);
                v.iter().copied().for_each(&mut eat);
            }
        }
        h
    }

    /// Anzahl der Objekte (für Messungen).
    pub fn object_count(&self) -> usize {
        self.objects.iter().map(BTreeMap::len).sum()
    }

    /// Kodiert den Snapshot als Delta gegen `base` (oder vollständig, wenn `None`).
    pub fn encode_delta(&self, base: Option<&Self>, w: &mut Writer) {
        let empty = Self::default();
        let base = base.unwrap_or(&empty);
        for (kind, fields) in FIELDS.iter().copied().enumerate() {
            let (cur, old) = (&self.objects[kind], &base.objects[kind]);
            // entfernte Objekte (Schlüssel aufsteigend, als Differenzen)
            let removed: Vec<u64> = old
                .keys()
                .filter(|k| !cur.contains_key(k))
                .copied()
                .collect();
            w.uvar(removed.len() as u64);
            let mut last = 0;
            for k in removed {
                w.uvar(k - last);
                last = k;
            }
            // neue oder geänderte Objekte
            let changed: Vec<(&u64, &Vec<i64>)> =
                cur.iter().filter(|(k, v)| old.get(k) != Some(v)).collect();
            w.uvar(changed.len() as u64);
            let mut last = 0;
            for (&k, v) in changed {
                w.uvar(k - last);
                last = k;
                let zero = vec![0; fields];
                let prev = old.get(&k).unwrap_or(&zero);
                let mask = v
                    .iter()
                    .zip(prev)
                    .enumerate()
                    .fold(0u64, |m, (i, (a, b))| if a == b { m } else { m | (1 << i) });
                w.uvar(mask);
                for (i, (a, b)) in v.iter().zip(prev).enumerate() {
                    if mask & (1 << i) != 0 {
                        w.ivar(a.wrapping_sub(*b));
                    }
                }
            }
        }
    }

    /// Nur für Vergleichsmessungen (`cargo xtask net-stats`): Delta wie im Original –
    /// bei geänderten Objekten jedes Feld als Differenz, ohne Änderungsmaske.
    #[doc(hidden)]
    pub fn encode_delta_like_original(&self, base: Option<&Self>, w: &mut Writer) {
        let empty = Self::default();
        let base = base.unwrap_or(&empty);
        for (kind, fields) in FIELDS.iter().copied().enumerate() {
            let (cur, old) = (&self.objects[kind], &base.objects[kind]);
            let removed: Vec<u64> = old
                .keys()
                .filter(|k| !cur.contains_key(k))
                .copied()
                .collect();
            w.uvar(removed.len() as u64);
            for k in removed {
                w.uvar(k);
            }
            let changed: Vec<(&u64, &Vec<i64>)> =
                cur.iter().filter(|(k, v)| old.get(k) != Some(v)).collect();
            w.uvar(changed.len() as u64);
            for (&k, v) in changed {
                w.uvar(k);
                let zero = vec![0; fields];
                let prev = old.get(&k).unwrap_or(&zero);
                for (a, b) in v.iter().zip(prev) {
                    w.ivar(a.wrapping_sub(*b));
                }
            }
        }
    }

    /// Gegenstück zu [`Self::encode_delta`].
    ///
    /// # Errors
    /// Bei fehlerhaften oder manipulierten Daten.
    pub fn decode_delta(tick: u64, base: Option<&Self>, r: &mut Reader<'_>) -> DecodeResult<Self> {
        let mut s = base.cloned().unwrap_or_default();
        s.tick = tick;
        for (kind, fields) in FIELDS.iter().copied().enumerate() {
            let table = &mut s.objects[kind];
            let removed: usize = r.uint("Anzahl")?;
            if removed > MAX_OBJECTS {
                return Err(DecodeError::Invalid("Anzahl"));
            }
            let mut key = 0u64;
            for _ in 0..removed {
                key = key.checked_add(r.uvar()?).ok_or(DecodeError::Overflow)?;
                table
                    .remove(&key)
                    .ok_or(DecodeError::Invalid("unbekanntes Objekt"))?;
            }
            let changed: usize = r.uint("Anzahl")?;
            if changed > MAX_OBJECTS {
                return Err(DecodeError::Invalid("Anzahl"));
            }
            let mut key = 0u64;
            for _ in 0..changed {
                key = key.checked_add(r.uvar()?).ok_or(DecodeError::Overflow)?;
                let mask = r.uvar()?;
                if mask >> fields != 0 {
                    return Err(DecodeError::Invalid("Maske"));
                }
                let v = table.entry(key).or_insert_with(|| vec![0; fields]);
                for (i, field) in v.iter_mut().enumerate() {
                    if mask & (1 << i) != 0 {
                        *field = field.wrapping_add(r.ivar()?);
                    }
                }
            }
            if table.len() > MAX_OBJECTS {
                return Err(DecodeError::Invalid("Anzahl"));
            }
        }
        s.validate()?;
        Ok(s)
    }

    /// Plausibilitätsprüfung, damit `apply_to` nie auf unsinnigen Werten arbeitet.
    fn validate(&self) -> DecodeResult<()> {
        let players = self.objects[PLAYER]
            .keys()
            .next_back()
            .copied()
            .unwrap_or(0);
        if players > 255 {
            return Err(DecodeError::Invalid("Spieler-Slot"));
        }
        for (&k, v) in self.objects[PROJECTILE].iter().chain(&self.objects[LASER]) {
            if key_owner(k) as i64 != v[0] || v[0] > 255 || v[0] < 0 {
                return Err(DecodeError::Invalid("Schütze"));
            }
        }
        Ok(())
    }
}

fn player_fields(p: &Player) -> Vec<i64> {
    let kind = match p.controller {
        Controller::Human | Controller::Remote => 0,
        Controller::Dummy { .. } => 1,
    };
    let mut v = vec![0; FIELDS[PLAYER]];
    v[0] = kind;
    v[1] = p.die_tick as i64;
    let Some(ch) = &p.character else { return v };
    let c = &ch.core;
    let a = &ch.arsenal;
    v[2] = 1;
    v[3] = f(c.pos.x);
    v[4] = f(c.pos.y);
    v[5] = f(c.vel.x * 256.0);
    v[6] = f(c.vel.y * 256.0);
    v[7] = hook_code(c.hook_state);
    v[8] = f(c.hook_pos.x);
    v[9] = f(c.hook_pos.y);
    v[10] = f(c.hook_dir.x * 256.0);
    v[11] = f(c.hook_dir.y * 256.0);
    v[12] = i64::from(c.hook_tick);
    v[13] = c.hooked_player.map_or(-1, |h| h as i64);
    v[14] = i64::from(c.jumped);
    v[15] = i64::from(c.direction);
    v[16] = i64::from(c.angle);
    v[17] = i64::from(ch.health);
    v[18] = i64::from(ch.armor);
    v[19] = weapon_code(Some(a.active));
    v[20] = weapon_code(a.queued);
    v[21] = i64::from(a.reload_timer);
    for (i, slot) in a.slots.iter().enumerate() {
        v[22 + i] = ammo_code(slot);
    }
    v
}

fn player_from(v: &[i64], controller: Controller) -> Player {
    let mut p = Player::new(controller);
    p.die_tick = v[1] as u64;
    p.spawning = false;
    if v[2] == 1 {
        let core = CharacterCore {
            pos: Vec2::new(v[3] as f32, v[4] as f32),
            vel: Vec2::new(v[5] as f32 / 256.0, v[6] as f32 / 256.0),
            hook_pos: Vec2::new(v[8] as f32, v[9] as f32),
            hook_dir: Vec2::new(v[10] as f32 / 256.0, v[11] as f32 / 256.0),
            hook_state: hook_state(v[7]),
            hook_tick: v[12] as u32,
            hooked_player: usize::try_from(v[13]).ok(),
            jumped: v[14] as u8,
            direction: v[15] as i8,
            angle: v[16] as i32,
            ..CharacterCore::default()
        };
        let mut arsenal = Arsenal {
            active: weapon(v[19]).unwrap_or(Weapon::Hammer),
            queued: weapon(v[20]),
            reload_timer: v[21] as u32,
            ..Arsenal::default()
        };
        for i in 0..3 {
            arsenal.slots[i] = ammo_slot(v[22 + i]);
        }
        p.character = Some(Character {
            core,
            health: v[17] as i32,
            armor: v[18] as i32,
            arsenal,
        });
    }
    p
}

/// Ist Slot `i` im Snapshot ein Dummy?
pub fn is_dummy(snap: &Snapshot, i: usize) -> bool {
    snap.objects[PLAYER]
        .get(&(i as u64))
        .is_some_and(|v| v[0] == 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_sim::{Collision, DummyPattern, PickupKind, PlayerInput, Tile, Tuning};

    fn world() -> World {
        let (w, h) = (40, 20);
        let mut tiles = vec![Tile::Air; w * h];
        for x in 0..w {
            tiles[18 * w + x] = Tile::Solid;
        }
        let mut world = World::new(Tuning::default(), Collision::new(w, h, tiles));
        world.spawn_points.push(Vec2::new(100.0, 500.0));
        world.add_pickup(PickupKind::Weapon(Weapon::Grenade), Vec2::new(150.0, 561.0));
        world.spawn(Vec2::new(100.0, 561.0));
        world.add_dummy(Vec2::new(600.0, 561.0), DummyPattern::WalkJump);
        world
    }

    fn roundtrip(s: &Snapshot, base: Option<&Snapshot>) -> (Snapshot, usize) {
        let mut w = Writer::new();
        s.encode_delta(base, &mut w);
        let bytes = w.into_bytes();
        let mut r = Reader::new(&bytes);
        let out = Snapshot::decode_delta(s.tick, base, &mut r).unwrap();
        r.finish().unwrap();
        (out, bytes.len())
    }

    #[test]
    fn delta_roundtrip_is_exact_and_small() {
        let mut w = world();
        let mut fire = 0;
        let mut prev = Snapshot::from_world(&w);
        let (full, full_len) = roundtrip(&prev, None);
        assert_eq!(full, prev);
        for t in 0..300 {
            if t % 10 == 0 {
                fire += 1;
            }
            w.step(&[PlayerInput {
                direction: 1,
                fire,
                target_x: 100,
                target_y: -30,
                ..PlayerInput::default()
            }]);
            let cur = Snapshot::from_world(&w);
            let (out, len) = roundtrip(&cur, Some(&prev));
            assert_eq!(out, cur, "Tick {t}");
            assert_eq!(out.checksum(), cur.checksum());
            assert!(len < full_len.max(60), "Delta {len} B, voll {full_len} B");
            prev = cur;
        }
    }

    #[test]
    fn applied_snapshot_reproduces_simulation() {
        // Server-Welt und aus Snapshots rekonstruierte Welt laufen identisch weiter
        let mut server = world();
        let input = PlayerInput {
            direction: 1,
            jump: true,
            target_x: 50,
            target_y: -80,
            ..PlayerInput::default()
        };
        for _ in 0..40 {
            server.step(&[input]);
        }
        let snap = Snapshot::from_world(&server);
        let mut client = world();
        snap.apply_to(&mut client, Some(0));
        // Dummy ist auf dem Client `Remote`, auf dem Server `Dummy` → nur Elora vergleichen
        client.players[0].as_mut().unwrap().input = server.players[0].as_ref().unwrap().input;
        for _ in 0..20 {
            server.step(&[input]);
            client.step(&[input]);
            assert_eq!(server.core(0), client.core(0));
        }
    }

    #[test]
    fn rejects_garbage() {
        let base = Snapshot::from_world(&world());
        let bad = [0u8, 1, 0, 0xff, 0xff, 0xff, 0x7f];
        assert!(Snapshot::decode_delta(1, Some(&base), &mut Reader::new(&bad)).is_err());
        // Entfernen eines unbekannten Objekts
        let bad = [1u8, 99];
        assert!(Snapshot::decode_delta(1, Some(&base), &mut Reader::new(&bad)).is_err());
    }
}

//! Projektile, Laserstrahlen und Pickups.

use crate::event::PickupKind;
use crate::math::{Vec2, round_to_int};
use crate::tuning::Tuning;

/// Eindeutiger Schlüssel eines Schusses: Schütze und Tick (höchstens ein Schuss pro
/// Spieler und Tick). Auf Server und Client-Vorhersage identisch.
pub type ShotKey = (usize, u64);

/// Granate (Referenz: `CProjectile`). Die Flugbahn wird analytisch aus Start,
/// Richtung und Zeit berechnet – kein Aufsummieren von Rundungsfehlern.
#[derive(Debug, Clone, PartialEq)]
pub struct Projectile {
    pub owner: usize,
    /// Startposition (auf ganze Einheiten gerundet).
    pub pos: Vec2,
    /// Richtung (auf 0,01 gerundet, wie im Netzwerkformat des Originals).
    pub dir: Vec2,
    pub start_tick: u64,
    pub lifespan: i32,
    pub damage: i32,
}

impl Projectile {
    pub fn key(&self) -> ShotKey {
        (self.owner, self.start_tick)
    }

    pub fn new(
        owner: usize,
        pos: Vec2,
        dir: Vec2,
        start_tick: u64,
        lifespan: i32,
        damage: i32,
    ) -> Self {
        let r = |v: f32| round_to_int(v) as f32;
        let r100 = |v: f32| round_to_int(v * 100.0) as f32 / 100.0;
        Self {
            owner,
            pos: Vec2::new(r(pos.x), r(pos.y)),
            dir: Vec2::new(r100(dir.x), r100(dir.y)),
            start_tick,
            lifespan,
            damage,
        }
    }

    /// Position nach `time` Sekunden (`CalcPos` im Original).
    pub fn pos_at(&self, time: f32, t: &Tuning) -> Vec2 {
        let time = time * t.grenade_speed;
        Vec2::new(
            self.pos.x + self.dir.x * time,
            self.pos.y + self.dir.y * time + t.grenade_curvature / 10000.0 * (time * time),
        )
    }
}

/// Laserstrahl (Referenz: `CLaser`). Sichtbar ist das Stück `from`–`pos`.
#[derive(Debug, Clone, PartialEq)]
pub struct Laser {
    pub owner: usize,
    pub pos: Vec2,
    pub from: Vec2,
    pub dir: Vec2,
    /// Verbleibende Reichweite; < 0 = erloschen.
    pub energy: f32,
    pub bounces: u32,
    /// Tick des letzten Abschnitts.
    pub eval_tick: u64,
    /// Tick des Schusses; zusammen mit `owner` eindeutig.
    pub start_tick: u64,
}

impl Laser {
    pub fn key(&self) -> ShotKey {
        (self.owner, self.start_tick)
    }

    pub fn new(owner: usize, pos: Vec2, dir: Vec2, energy: f32, tick: u64) -> Self {
        Self {
            owner,
            pos,
            from: pos,
            dir,
            energy,
            bounces: 0,
            eval_tick: tick,
            start_tick: tick,
        }
    }
}

/// Pickup auf der Karte. `respawn_tick = Some(t)`: aufgenommen, wieder da nach Tick `t`.
#[derive(Debug, Clone, PartialEq)]
pub struct Pickup {
    pub kind: PickupKind,
    pub pos: Vec2,
    pub respawn_tick: Option<u64>,
}

impl Pickup {
    pub fn available(&self) -> bool {
        self.respawn_tick.is_none()
    }
}

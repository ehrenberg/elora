//! Projectiles, laser beams and pickups.

use crate::event::PickupKind;
use crate::math::{Vec2, round_to_int};
use crate::tuning::Tuning;

/// Unique key of a shot: shooter and tick (at most one shot per
/// player and tick). Identical on the server and in client prediction.
pub type ShotKey = (usize, u64);

/// Grenade (reference: `CProjectile`). The trajectory is computed analytically from start,
/// direction and time – no accumulation of rounding errors.
#[derive(Debug, Clone, PartialEq)]
pub struct Projectile {
    pub owner: usize,
    /// Start position (rounded to whole units).
    pub pos: Vec2,
    /// Direction (rounded to 0.01, as in the network format of the original).
    pub dir: Vec2,
    pub start_tick: u64,
    pub lifespan: i32,
    pub damage: i32,
    /// Wind at launch (R2-W1, adventure only): deflects the trajectory sideways.
    pub wind: f32,
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
            wind: 0.0,
        }
    }

    /// Position after `time` seconds (`CalcPos` in the original).
    pub fn pos_at(&self, time: f32, t: &Tuning) -> Vec2 {
        let time = time * t.grenade_speed;
        Vec2::new(
            self.pos.x + self.dir.x * time + self.wind * t.grenade_wind / 10000.0 * (time * time),
            self.pos.y + self.dir.y * time + t.grenade_curvature / 10000.0 * (time * time),
        )
    }
}

/// Laser beam (reference: `CLaser`). The visible part is `from`–`pos`.
#[derive(Debug, Clone, PartialEq)]
pub struct Laser {
    pub owner: usize,
    pub pos: Vec2,
    pub from: Vec2,
    pub dir: Vec2,
    /// Remaining range; < 0 = extinguished.
    pub energy: f32,
    pub bounces: u32,
    /// Tick of the last segment.
    pub eval_tick: u64,
    /// Tick of the shot; unique together with `owner`.
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

/// Pickup on the map. `respawn_tick = Some(t)`: picked up, back after tick `t`.
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

/// Flag of a team (CTF, reference: `CFlag`).
#[derive(Debug, Clone, PartialEq)]
pub struct Flag {
    pub team: crate::Team,
    pub stand: Vec2,
    pub pos: Vec2,
    pub vel: Vec2,
    pub carrier: Option<usize>,
    pub at_stand: bool,
    pub drop_tick: u64,
    pub grab_tick: u64,
}

impl Flag {
    /// Collision size (original: 14).
    pub const PHYS_SIZE: f32 = 14.0;
    /// Return after this many seconds on the ground (original: 30).
    pub const RETURN_SECS: u64 = 30;

    pub fn new(team: crate::Team, stand: Vec2) -> Self {
        Self {
            team,
            stand,
            pos: stand,
            vel: Vec2::ZERO,
            carrier: None,
            at_stand: true,
            drop_tick: 0,
            grab_tick: 0,
        }
    }

    pub fn reset(&mut self) {
        self.carrier = None;
        self.at_stand = true;
        self.pos = self.stand;
        self.vel = Vec2::ZERO;
        self.grab_tick = 0;
    }
}

//! Events of a tick – basis for effects, sounds (M5) and network events (M3).

use crate::Vec2;
use crate::weapon::Weapon;

/// Cause of a death.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    Weapon(Weapon),
    /// Death tile
    World,
    /// Suicide command (`kill`, E-055)
    Suicide,
    /// Removed by the game (team change, restart) – not counted.
    Game,
    /// By an enemy in the adventure (A1.2).
    Creature,
}

/// Kind of a pickup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PickupKind {
    Health,
    Armor,
    Weapon(Weapon),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Thunderstorm (R2-W1, E-336): lightning is about to strike here (the ground glows).
    LightningWarn {
        pos: Vec2,
    },
    /// Lightning strike on the ground.
    Lightning {
        pos: Vec2,
    },
    /// Shot fired.
    Fire {
        player: usize,
        weapon: Weapon,
        pos: Vec2,
    },
    /// Fire key without ammo.
    NoAmmo {
        player: usize,
    },
    /// Weapon switched.
    WeaponSwitch {
        player: usize,
        weapon: Weapon,
    },
    /// Hammer of `owner` has hit.
    HammerHit {
        owner: usize,
        pos: Vec2,
    },
    /// Laser of `owner` bounced off a wall.
    LaserBounce {
        owner: usize,
        pos: Vec2,
    },
    /// Explosion of a grenade of `owner`.
    Explosion {
        owner: usize,
        pos: Vec2,
    },
    /// Damage taken (after armor).
    Damage {
        player: usize,
        from: Option<usize>,
        health: i32,
        armor: i32,
    },
    Death {
        player: usize,
        killer: Option<usize>,
        cause: DeathCause,
        pos: Vec2,
    },
    Spawn {
        player: usize,
        pos: Vec2,
    },
    Pickup {
        player: usize,
        kind: PickupKind,
        pos: Vec2,
    },
    /// Flag of `team` picked up (`from_stand`: from the stand, not from the ground).
    FlagGrab {
        team: crate::Team,
        player: usize,
        from_stand: bool,
    },
    /// Flag of `team` dropped (carrier died).
    FlagDrop {
        team: crate::Team,
        player: usize,
        pos: Vec2,
    },
    /// Flag of `team` back at the stand (`player`: touched by its own team, otherwise
    /// time/death tile).
    FlagReturn {
        team: crate::Team,
        player: Option<usize>,
    },
    /// Flag of `team` captured by `player` after `ticks` ticks.
    FlagCapture {
        team: crate::Team,
        player: usize,
        ticks: u64,
    },
    /// Pickup is available again.
    PickupRespawn {
        kind: PickupKind,
        pos: Vec2,
    },
    /// Stomp has landed (shockwave, A-05).
    Stomp {
        player: usize,
        pos: Vec2,
    },
    /// A tile has broken (crumbling floor, E-230) and is now air.
    TileBroken {
        tx: i32,
        ty: i32,
    },
    /// Thin ice (R2-M2.4): gets cracks under Elora or breaks (`broken`).
    IceCrack {
        tx: i32,
        ty: i32,
        broken: bool,
    },
    /// A tile was temporarily placed or reset (root wall, R2-M2.2).
    TileSet {
        tx: i32,
        ty: i32,
        tile: crate::Tile,
    },
    /// Enemy hit (`from`: player slot).
    CreatureHit {
        id: u32,
        pos: Vec2,
        damage: i32,
        from: Option<usize>,
    },
    /// Enemy defeated (`killer`: player slot); `kind` is the index of the kind.
    CreatureDeath {
        id: u32,
        kind: usize,
        pos: Vec2,
        killer: Option<usize>,
    },
    /// Enemy has fired.
    CreatureFire {
        id: u32,
        pos: Vec2,
    },
    /// A guardian changes phase (sound and effects, R2-M2.1).
    CreatureAct {
        id: u32,
        pos: Vec2,
        act: CreatureAct,
    },
    /// Loot collected.
    LootCollect {
        player: usize,
        item: String,
        count: u32,
        pos: Vec2,
    },
}

/// Phase change of a guardian.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CreatureAct {
    /// Woken up, the fight begins.
    Wake,
    /// Dive begins.
    Dive,
    /// Landed (stunned or right back up).
    Land,
    /// Shot out of the ground (root snake, R2-M2.2).
    Emerge,
    /// Back into the ground.
    Burrow,
    /// Ground quakes: a root thrust is about to hit this spot (Root Warden).
    Warn,
    /// Root thrust.
    Strike,
    /// A core has come loose.
    Core,
    /// Blizzard in the hall begins (Kristella, R2-M2.4): the session sets the weather.
    Storm,
}

impl Event {
    /// Originator of a shot event (for client prediction, E-057).
    pub fn shooter(&self) -> Option<usize> {
        match *self {
            Self::Fire { player, .. }
            | Self::NoAmmo { player }
            | Self::WeaponSwitch { player, .. } => Some(player),
            Self::HammerHit { owner, .. }
            | Self::LaserBounce { owner, .. }
            | Self::Explosion { owner, .. } => Some(owner),
            _ => None,
        }
    }
}

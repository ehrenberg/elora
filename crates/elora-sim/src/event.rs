//! Events of a tick – basis for effects, sounds (M5) and network events (M3).

use crate::Vec2;
use crate::weapon::Weapon;

/// Cause of a death.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    /// Hit by a weapon.
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
    /// Restores health.
    Health,
    /// Grants armor.
    Armor,
    /// Grants the weapon and ammo.
    Weapon(Weapon),
}

/// Something that happened in a tick.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Thunderstorm (R2-W1, E-336): lightning is about to strike here (the ground glows).
    LightningWarn {
        /// Target point (world units).
        pos: Vec2,
    },
    /// Lightning strike on the ground.
    Lightning {
        /// Impact point (world units).
        pos: Vec2,
    },
    /// Shot fired.
    Fire {
        /// Shooter slot.
        player: usize,
        /// Weapon fired.
        weapon: Weapon,
        /// Muzzle position.
        pos: Vec2,
    },
    /// Fire key without ammo.
    NoAmmo {
        /// Player slot.
        player: usize,
    },
    /// Weapon switched.
    WeaponSwitch {
        /// Player slot.
        player: usize,
        /// New active weapon.
        weapon: Weapon,
    },
    /// Hammer of `owner` has hit.
    HammerHit {
        /// Player slot of the attacker.
        owner: usize,
        /// Hit point.
        pos: Vec2,
    },
    /// Laser of `owner` bounced off a wall.
    LaserBounce {
        /// Player slot of the shooter.
        owner: usize,
        /// Bounce point.
        pos: Vec2,
    },
    /// Explosion of a grenade of `owner`.
    Explosion {
        /// Player slot of the thrower.
        owner: usize,
        /// Center of the explosion.
        pos: Vec2,
    },
    /// Damage taken (after armor).
    Damage {
        /// Player slot of the victim.
        player: usize,
        /// Player slot of the attacker (`None`: world or creature).
        from: Option<usize>,
        /// Health lost.
        health: i32,
        /// Armor lost.
        armor: i32,
    },
    /// A character died.
    Death {
        /// Player slot of the victim.
        player: usize,
        /// Player slot of the killer (`None`: no player).
        killer: Option<usize>,
        /// Cause of death.
        cause: DeathCause,
        /// Position of death.
        pos: Vec2,
    },
    /// A character spawned.
    Spawn {
        /// Player slot.
        player: usize,
        /// Spawn position.
        pos: Vec2,
    },
    /// A pickup was collected.
    Pickup {
        /// Player slot of the collector.
        player: usize,
        /// Kind of the pickup.
        kind: PickupKind,
        /// Position of the pickup.
        pos: Vec2,
    },
    /// Flag of `team` picked up (`from_stand`: from the stand, not from the ground).
    FlagGrab {
        /// Team of the flag.
        team: crate::Team,
        /// Player slot of the new carrier.
        player: usize,
        /// Taken from the stand.
        from_stand: bool,
    },
    /// Flag of `team` dropped (carrier died).
    FlagDrop {
        /// Team of the flag.
        team: crate::Team,
        /// Player slot of the former carrier.
        player: usize,
        /// Drop position.
        pos: Vec2,
    },
    /// Flag of `team` back at the stand (`player`: touched by its own team, otherwise
    /// time/death tile).
    FlagReturn {
        /// Team of the flag.
        team: crate::Team,
        /// Player slot that returned it (`None`: time or death tile).
        player: Option<usize>,
    },
    /// Flag of `team` captured by `player` after `ticks` ticks.
    FlagCapture {
        /// Team of the captured flag.
        team: crate::Team,
        /// Player slot of the capturer.
        player: usize,
        /// Ticks from the grab to the capture.
        ticks: u64,
    },
    /// Pickup is available again.
    PickupRespawn {
        /// Kind of the pickup.
        kind: PickupKind,
        /// Position of the pickup.
        pos: Vec2,
    },
    /// Stomp has landed (shockwave, A-05).
    Stomp {
        /// Player slot of the stomper.
        player: usize,
        /// Landing point.
        pos: Vec2,
    },
    /// A tile has broken (crumbling floor, E-230) and is now air.
    TileBroken {
        /// Tile column.
        tx: i32,
        /// Tile row.
        ty: i32,
    },
    /// Thin ice (R2-M2.4): gets cracks under Elora or breaks (`broken`).
    IceCrack {
        /// Tile column.
        tx: i32,
        /// Tile row.
        ty: i32,
        /// Broken through (now ice water).
        broken: bool,
    },
    /// A tile was temporarily placed or reset (root wall, R2-M2.2).
    TileSet {
        /// Tile column.
        tx: i32,
        /// Tile row.
        ty: i32,
        /// New tile.
        tile: crate::Tile,
    },
    /// Enemy hit (`from`: player slot).
    CreatureHit {
        /// Creature id.
        id: u32,
        /// Hit point.
        pos: Vec2,
        /// Damage dealt.
        damage: i32,
        /// Player slot of the attacker.
        from: Option<usize>,
    },
    /// Enemy defeated (`killer`: player slot); `kind` is the index of the kind.
    CreatureDeath {
        /// Creature id.
        id: u32,
        /// Index into [`crate::World::creature_kinds`].
        kind: usize,
        /// Position of death.
        pos: Vec2,
        /// Player slot of the killer.
        killer: Option<usize>,
    },
    /// Enemy has fired.
    CreatureFire {
        /// Creature id.
        id: u32,
        /// Muzzle position.
        pos: Vec2,
    },
    /// A guardian changes phase (sound and effects, R2-M2.1).
    CreatureAct {
        /// Creature id.
        id: u32,
        /// Position of the guardian.
        pos: Vec2,
        /// New phase.
        act: CreatureAct,
    },
    /// Loot collected.
    LootCollect {
        /// Player slot of the collector.
        player: usize,
        /// Item key.
        item: String,
        /// Number of pieces.
        count: u32,
        /// Position of the loot.
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

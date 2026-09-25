//! Ereignisse eines Ticks – Grundlage für Effekte, Sounds (M5) und Netzwerk-Events (M3).

use crate::Vec2;
use crate::weapon::Weapon;

/// Ursache eines Todes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    Weapon(Weapon),
    /// Todes-Tile
    World,
}

/// Art eines Pickups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PickupKind {
    Health,
    Armor,
    Weapon(Weapon),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Schuss abgegeben.
    Fire {
        player: usize,
        weapon: Weapon,
        pos: Vec2,
    },
    /// Feuertaste ohne Munition.
    NoAmmo {
        player: usize,
    },
    /// Waffe gewechselt.
    WeaponSwitch {
        player: usize,
        weapon: Weapon,
    },
    /// Hammer hat getroffen.
    HammerHit {
        pos: Vec2,
    },
    /// Laser ist an einer Wand abgeprallt.
    LaserBounce {
        pos: Vec2,
    },
    Explosion {
        pos: Vec2,
    },
    /// Schaden erhalten (nach Rüstung).
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
    /// Pickup ist wieder verfügbar.
    PickupRespawn {
        kind: PickupKind,
        pos: Vec2,
    },
}

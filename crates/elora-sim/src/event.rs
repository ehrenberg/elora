//! Ereignisse eines Ticks – Grundlage für Effekte, Sounds (M5) und Netzwerk-Events (M3).

use crate::Vec2;
use crate::weapon::Weapon;

/// Ursache eines Todes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    Weapon(Weapon),
    /// Todes-Tile
    World,
    /// Selbstmord-Befehl (`kill`, E-055)
    Suicide,
    /// Durch das Spiel entfernt (Team-Wechsel, Neustart) – wird nicht gewertet.
    Game,
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
    /// Hammer von `owner` hat getroffen.
    HammerHit {
        owner: usize,
        pos: Vec2,
    },
    /// Laser von `owner` ist an einer Wand abgeprallt.
    LaserBounce {
        owner: usize,
        pos: Vec2,
    },
    /// Explosion einer Granate von `owner`.
    Explosion {
        owner: usize,
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
    /// Flagge von `team` aufgenommen (`from_stand`: vom Stand, nicht vom Boden).
    FlagGrab {
        team: crate::Team,
        player: usize,
        from_stand: bool,
    },
    /// Flagge von `team` fallen gelassen (Träger gestorben).
    FlagDrop {
        team: crate::Team,
        player: usize,
        pos: Vec2,
    },
    /// Flagge von `team` zurück am Stand (`player`: vom eigenen Team berührt, sonst Zeit/Todes-Tile).
    FlagReturn {
        team: crate::Team,
        player: Option<usize>,
    },
    /// Flagge von `team` erobert durch `player` nach `ticks` Ticks.
    FlagCapture {
        team: crate::Team,
        player: usize,
        ticks: u64,
    },
    /// Pickup ist wieder verfügbar.
    PickupRespawn {
        kind: PickupKind,
        pos: Vec2,
    },
    /// Stampfen ist aufgeprallt (Stoßwelle, A-05).
    Stomp {
        player: usize,
        pos: Vec2,
    },
    /// Ein Tile ist zerbrochen (Bröckelboden, E-230) und jetzt Luft.
    TileBroken {
        tx: i32,
        ty: i32,
    },
}

impl Event {
    /// Verursacher eines Schuss-Ereignisses (für die Client-Vorhersage, E-057).
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

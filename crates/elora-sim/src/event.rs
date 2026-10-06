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
    /// Von einem Gegner im Abenteuer (A1.2).
    Creature,
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
    /// Gewitter (R2-W1, E-336): hier schlägt gleich ein Blitz ein (der Boden glimmt).
    LightningWarn {
        pos: Vec2,
    },
    /// Blitzeinschlag am Boden.
    Lightning {
        pos: Vec2,
    },
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
    /// Dünnes Eis (R2-M2.4): bekommt Risse unter Elora bzw. bricht (`broken`).
    IceCrack {
        tx: i32,
        ty: i32,
        broken: bool,
    },
    /// Ein Tile wurde zeitweise gesetzt oder zurückgesetzt (Wurzelwand, R2-M2.2).
    TileSet {
        tx: i32,
        ty: i32,
        tile: crate::Tile,
    },
    /// Gegner getroffen (`from`: Spieler-Slot).
    CreatureHit {
        id: u32,
        pos: Vec2,
        damage: i32,
        from: Option<usize>,
    },
    /// Gegner besiegt (`killer`: Spieler-Slot); `kind` ist der Index der Art.
    CreatureDeath {
        id: u32,
        kind: usize,
        pos: Vec2,
        killer: Option<usize>,
    },
    /// Gegner hat geschossen.
    CreatureFire {
        id: u32,
        pos: Vec2,
    },
    /// Ein Hüter wechselt die Phase (Sound und Effekte, R2-M2.1).
    CreatureAct {
        id: u32,
        pos: Vec2,
        act: CreatureAct,
    },
    /// Beute eingesammelt.
    LootCollect {
        player: usize,
        item: String,
        count: u32,
        pos: Vec2,
    },
}

/// Phasenwechsel eines Hüters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CreatureAct {
    /// Aufgewacht, der Kampf beginnt.
    Wake,
    /// Sturzflug beginnt.
    Dive,
    /// Aufgeprallt (benommen oder gleich wieder hoch).
    Land,
    /// Aus dem Boden geschossen (Wurzelschlange, R2-M2.2).
    Emerge,
    /// Zurück in den Boden.
    Burrow,
    /// Boden bebt: gleich ein Wurzelstoß an dieser Stelle (Wurzelwächter).
    Warn,
    /// Wurzelstoß.
    Strike,
    /// Ein Kern hat sich gelöst.
    Core,
    /// Schneesturm in der Halle beginnt (Kristella, R2-M2.4): die Sitzung setzt das Wetter.
    Storm,
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

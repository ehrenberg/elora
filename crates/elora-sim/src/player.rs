//! Spieler-Slots und lebende Figuren.

use crate::Vec2;
use crate::ability::Abilities;
use crate::character::CharacterCore;
use crate::dummy::{DummyBrain, DummyPattern};
use crate::input::PlayerInput;
use crate::weapon::Arsenal;

/// Eine lebende Figur: Physik, Leben und Waffen.
#[derive(Debug, Clone, PartialEq)]
pub struct Character {
    pub core: CharacterCore,
    pub health: i32,
    pub armor: i32,
    pub arsenal: Arsenal,
    /// Im Abenteuer nach einem Treffer bis zu diesem Tick unverwundbar (E-234).
    pub invulnerable_until: u64,
    /// Zuletzt sicher betretener Boden: hierher setzen Dornen im Abenteuer zurück (E-283).
    pub safe_pos: Vec2,
}

impl Character {
    /// Frisch gespawnte Figur mit vollen Lebenspunkten, ohne Rüstung, nur Hammer.
    pub fn spawn(pos: Vec2, max_health: i32) -> Self {
        Self {
            core: CharacterCore::new(pos),
            health: max_health,
            armor: 0,
            arsenal: Arsenal::default(),
            invulnerable_until: 0,
            safe_pos: pos,
        }
    }
}

/// Team eines Spielers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Team {
    /// Kein Team (DM, LMS, Sandbox).
    #[default]
    None,
    Red,
    Blue,
    /// Zuschauer: spawnt nie.
    Spectator,
}

impl Team {
    /// Index 0 (Rot) bzw. 1 (Blau) für Team-Tabellen.
    pub fn index(self) -> Option<usize> {
        match self {
            Self::Red => Some(0),
            Self::Blue => Some(1),
            Self::None | Self::Spectator => None,
        }
    }

    #[must_use]
    pub fn other(self) -> Self {
        match self {
            Self::Red => Self::Blue,
            Self::Blue => Self::Red,
            t => t,
        }
    }

    /// Gleiches (echtes) Team?
    pub fn is_mate(self, other: Self) -> bool {
        self.index().is_some() && self == other
    }
}

/// Wer steuert einen Slot?
#[derive(Debug, Clone, PartialEq)]
pub enum Controller {
    /// Eingaben kommen von außen (`World::step`).
    Human,
    /// Fremde Figur in der Client-Vorhersage: wird ohne Eingaben weitergerechnet
    /// (wie `Tick(false)` im Original), feuert nicht.
    Remote,
    /// Trainings-Dummy mit festem Bewegungsmuster (E-053); respawnt an `home`.
    Dummy {
        pattern: DummyPattern,
        home: Vec2,
        brain: DummyBrain,
    },
}

/// Ein Spieler-Slot. Existiert auch, während die Figur tot ist.
#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub controller: Controller,
    pub character: Option<Character>,
    /// Aktuelle und vorige Eingabe (für Klick-Erkennung).
    pub input: PlayerInput,
    pub prev_input: PlayerInput,
    /// Tick des letzten Todes.
    pub die_tick: u64,
    /// Frühester Respawn-Tick.
    pub respawn_tick: u64,
    /// Respawn angefordert (Feuertaste oder Auto-Respawn).
    pub spawning: bool,
    pub team: Team,
    /// Kein Respawn (Survival-Modi während einer Runde).
    pub respawn_disabled: bool,
    /// Fähigkeiten, mit denen die Figur spawnt (Abenteuer, Quellenkampf).
    pub abilities: Abilities,
}

impl Player {
    pub fn new(controller: Controller) -> Self {
        Self {
            controller,
            character: None,
            input: PlayerInput::default(),
            prev_input: PlayerInput::default(),
            die_tick: 0,
            respawn_tick: 0,
            spawning: true,
            team: Team::None,
            respawn_disabled: false,
            abilities: Abilities::NONE,
        }
    }

    pub fn is_dummy(&self) -> bool {
        matches!(self.controller, Controller::Dummy { .. })
    }
}

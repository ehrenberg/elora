//! Spieler-Slots und lebende Figuren.

use crate::Vec2;
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
}

impl Character {
    /// Frisch gespawnte Figur mit vollen Lebenspunkten, ohne Rüstung, nur Hammer.
    pub fn spawn(pos: Vec2, max_health: i32) -> Self {
        Self {
            core: CharacterCore::new(pos),
            health: max_health,
            armor: 0,
            arsenal: Arsenal::default(),
        }
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
        }
    }

    pub fn is_dummy(&self) -> bool {
        matches!(self.controller, Controller::Dummy { .. })
    }
}

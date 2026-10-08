//! Player slots and living characters.

use crate::Vec2;
use crate::ability::Abilities;
use crate::character::CharacterCore;
use crate::dummy::{DummyBrain, DummyPattern};
use crate::input::PlayerInput;
use crate::weapon::Arsenal;

/// A living character: physics, health and weapons.
#[derive(Debug, Clone, PartialEq)]
pub struct Character {
    pub core: CharacterCore,
    pub health: i32,
    pub armor: i32,
    pub arsenal: Arsenal,
    /// In the adventure, invulnerable until this tick after a hit (E-234).
    pub invulnerable_until: u64,
    /// Last ground stepped on safely: thorns in the adventure reset to here (E-283).
    pub safe_pos: Vec2,
}

impl Character {
    /// Freshly spawned character with full health, no armor, only the hammer.
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

/// Team of a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Team {
    /// No team (DM, LMS, sandbox).
    #[default]
    None,
    Red,
    Blue,
    /// Spectator: never spawns.
    Spectator,
}

impl Team {
    /// Index 0 (red) or 1 (blue) for team tables.
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

    /// Same (real) team?
    pub fn is_mate(self, other: Self) -> bool {
        self.index().is_some() && self == other
    }
}

/// Who controls a slot?
#[derive(Debug, Clone, PartialEq)]
pub enum Controller {
    /// Inputs come from outside (`World::step`).
    Human,
    /// Foreign character in client prediction: simulated further without inputs
    /// (like `Tick(false)` in the original), does not fire.
    Remote,
    /// Training dummy with a fixed movement pattern (E-053); respawns at `home`.
    Dummy {
        pattern: DummyPattern,
        home: Vec2,
        brain: DummyBrain,
    },
}

/// A player slot. Exists even while the character is dead.
#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub controller: Controller,
    pub character: Option<Character>,
    /// Current and previous input (for click detection).
    pub input: PlayerInput,
    pub prev_input: PlayerInput,
    /// Tick of the last death.
    pub die_tick: u64,
    /// Earliest respawn tick.
    pub respawn_tick: u64,
    /// Respawn requested (fire key or auto-respawn).
    pub spawning: bool,
    pub team: Team,
    /// No respawn (survival modes during a round).
    pub respawn_disabled: bool,
    /// Abilities the character spawns with (adventure, spring battle).
    pub abilities: Abilities,
    /// Just joined: the first input is only the baseline for click detection
    /// (otherwise an already incremented fire counter counts as a click, e.g. on a map change
    /// in the adventure – playtest).
    pub fresh: bool,
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
            fresh: false,
        }
    }

    pub fn is_dummy(&self) -> bool {
        matches!(self.controller, Controller::Dummy { .. })
    }
}

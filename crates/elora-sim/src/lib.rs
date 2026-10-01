//! Deterministische Spielsimulation von Elora.
//!
//! Enthält reine Spiellogik ohne Abhängigkeiten zu Fenster, Grafik oder Netzwerk
//! (siehe `docs/handbuch/architektur.md`). Server, Client-Vorhersage und Tests nutzen
//! denselben Code.

pub mod character;
pub mod collision;
pub mod dummy;
pub mod entities;
pub mod event;
pub mod input;
pub mod math;
pub mod player;
#[cfg(feature = "serde")]
pub mod replay;
pub mod tuning;
pub mod weapon;
pub mod world;

pub use character::{CharacterCore, HookState, PHYS_SIZE};
pub use collision::{BeltDir, Collision, JumpDir, TILE_SIZE, Tile};
pub use dummy::DummyPattern;
pub use event::{DeathCause, Event, PickupKind};
pub use input::PlayerInput;
pub use math::Vec2;
pub use player::{Character, Controller, Player, Team};
pub use tuning::Tuning;
pub use weapon::Weapon;
pub use world::World;

/// Feste Simulationsrate in Ticks pro Sekunde (T-01).
pub const TICKS_PER_SECOND: u32 = 50;

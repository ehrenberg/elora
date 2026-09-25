//! Deterministische Spielsimulation von Elora.
//!
//! Enthält reine Spiellogik ohne Abhängigkeiten zu Fenster, Grafik oder Netzwerk
//! (siehe `docs/03-architektur.md`). Server, Client-Vorhersage und Tests nutzen
//! denselben Code.

pub mod character;
pub mod collision;
pub mod input;
pub mod math;
#[cfg(feature = "serde")]
pub mod replay;
pub mod tuning;
pub mod world;

pub use character::{CharacterCore, HookState, PHYS_SIZE};
pub use collision::{Collision, TILE_SIZE, Tile};
pub use input::PlayerInput;
pub use math::Vec2;
pub use tuning::Tuning;
pub use world::World;

/// Feste Simulationsrate in Ticks pro Sekunde (T-01).
pub const TICKS_PER_SECOND: u32 = 50;

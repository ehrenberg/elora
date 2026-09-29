//! Karten-Datenmodell und Kartenformate von Elora.
//!
//! - Textformat für Test- und Entwicklungskarten (`docs/05-kartenformat.md`, E-024)
//! - Release-Kartenformat (E-028), folgt in M6

mod text;

use elora_sim::{
    Collision, DummyPattern, PickupKind, TILE_SIZE, Tile, Tuning, Vec2, Weapon, World,
};

pub use text::{MapError, parse_text_map};

/// Unterstützte Version des Textformats.
pub const TEXT_FORMAT_VERSION: u32 = 1;
/// Maximale Kantenlänge einer Karte in Tiles.
pub const MAX_SIZE: usize = 1000;

/// Art eines Entities auf der Karte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityKind {
    Spawn,
    SpawnRed,
    SpawnBlue,
    FlagRed,
    FlagBlue,
    Health,
    Armor,
    Laser,
    Grenade,
    /// Trainings-Dummy (E-053, E-054), nur für die Sandbox.
    Dummy(DummyPattern),
}

/// Ein Entity, sitzt in der Mitte seines Tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    pub kind: EntityKind,
    pub tx: usize,
    pub ty: usize,
}

impl Entity {
    /// Weltposition (Tile-Mitte).
    pub fn pos(&self) -> Vec2 {
        let half = TILE_SIZE as f32 / 2.0;
        Vec2::new(
            self.tx as f32 * TILE_SIZE as f32 + half,
            self.ty as f32 * TILE_SIZE as f32 + half,
        )
    }
}

/// Spielmodi, die eine Karte anhand ihrer Entities unterstützt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct SupportedModes {
    /// DM, LMS, Instagib (neutrale Spawns)
    pub free_for_all: bool,
    /// TDM, LTS (rote und blaue Spawns)
    pub team: bool,
    /// CTF (Team-Spawns + genau eine Flagge je Team)
    pub ctf: bool,
}

/// Eine geladene Karte.
#[derive(Debug, Clone, PartialEq)]
pub struct Map {
    pub name: String,
    pub author: Option<String>,
    pub width: usize,
    pub height: usize,
    /// Zeilenweise, oben links beginnend.
    pub tiles: Vec<Tile>,
    pub entities: Vec<Entity>,
}

impl Map {
    /// Kollisionsraster für die Simulation.
    pub fn collision(&self) -> Collision {
        Collision::new(self.width, self.height, self.tiles.clone())
    }

    /// Welt mit Spawnpunkten, Pickups und Dummies dieser Karte (ohne Spieler).
    pub fn world(&self, tuning: Tuning) -> World {
        let mut world = World::new(tuning, self.collision());
        for e in &self.entities {
            let pos = e.pos();
            match e.kind {
                EntityKind::Spawn => world.spawn_points.push(pos),
                EntityKind::SpawnRed => world.team_spawns[0].push(pos),
                EntityKind::SpawnBlue => world.team_spawns[1].push(pos),
                EntityKind::Health => world.add_pickup(PickupKind::Health, pos),
                EntityKind::Armor => world.add_pickup(PickupKind::Armor, pos),
                EntityKind::Laser => world.add_pickup(PickupKind::Weapon(Weapon::Laser), pos),
                EntityKind::Grenade => world.add_pickup(PickupKind::Weapon(Weapon::Grenade), pos),
                EntityKind::Dummy(pattern) => {
                    world.add_dummy(pos, pattern);
                }
                // Flaggen legt das Regelwerk im CTF-Modus an (elora-game)
                EntityKind::FlagRed => world.flag_stands[0] = Some(pos),
                EntityKind::FlagBlue => world.flag_stands[1] = Some(pos),
            }
        }
        world
    }

    pub fn entities_of(&self, kind: EntityKind) -> impl Iterator<Item = &Entity> {
        self.entities.iter().filter(move |e| e.kind == kind)
    }

    pub fn supported_modes(&self) -> SupportedModes {
        let count = |k| self.entities_of(k).count();
        let team = count(EntityKind::SpawnRed) > 0 && count(EntityKind::SpawnBlue) > 0;
        SupportedModes {
            free_for_all: count(EntityKind::Spawn) > 0,
            team,
            ctf: team && count(EntityKind::FlagRed) == 1 && count(EntityKind::FlagBlue) == 1,
        }
    }
}

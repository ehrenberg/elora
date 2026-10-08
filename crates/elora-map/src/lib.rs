//! Map data model and map format of Elora.
//!
//! Release format `.emap` (binary, E-129, E-143 to E-146), described in
//! `docs/handbook/map-format.md`.

pub mod adventure;
mod ascii;
mod binary;
pub mod look;

use elora_sim::{
    Collision, DummyPattern, PickupKind, TILE_SIZE, Tile, Tuning, Vec2, Weapon, World,
};

pub use adventure::{Adventure, Object, ObjectKind};
pub use ascii::ENTITY_CHARS;
pub use binary::{FORMAT_VERSION, MapError, checksum, decode, decode_draft, encode, validate};
pub use look::{Art, Background, Decor, Envelope, Image, Rgba, Sky, Weather, WeatherKind};

/// File extension of maps (without dot).
pub const EXTENSION: &str = "emap";
/// Maximum edge length of a map in tiles.
pub const MAX_SIZE: usize = 1000;

/// Kind of an entity on the map.
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
    /// Training dummy (E-053, E-054), only for the sandbox.
    Dummy(DummyPattern),
}

/// An entity, sits in the center of its tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    pub kind: EntityKind,
    pub tx: usize,
    pub ty: usize,
}

impl Entity {
    /// World position (tile center).
    pub fn pos(&self) -> Vec2 {
        let half = TILE_SIZE as f32 / 2.0;
        Vec2::new(
            self.tx as f32 * TILE_SIZE as f32 + half,
            self.ty as f32 * TILE_SIZE as f32 + half,
        )
    }
}

/// Game modes a map supports based on its entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct SupportedModes {
    /// DM, LMS, Instagib (neutral spawns)
    pub free_for_all: bool,
    /// TDM, LTS (red and blue spawns)
    pub team: bool,
    /// CTF (team spawns + exactly one flag per team)
    pub ctf: bool,
}

/// A loaded map.
#[derive(Debug, Clone, PartialEq)]
pub struct Map {
    pub name: String,
    pub author: Option<String>,
    pub width: usize,
    pub height: usize,
    /// Row by row, starting at the top left.
    pub tiles: Vec<Tile>,
    pub entities: Vec<Entity>,
    /// Names of the materials used (built-in sets, M6.3).
    pub materials: Vec<String>,
    /// Material per tile: 0 = default of the tile kind, otherwise index + 1 into `materials`.
    /// Empty if the map does not define materials.
    pub material_map: Vec<u8>,
    pub sky: Sky,
    /// Weather of the map (R2-W1); in the adventure the region can replace it.
    pub weather: Weather,
    /// From back to front.
    pub backgrounds: Vec<Background>,
    /// Decoration behind the playing field.
    pub decor_back: Vec<Decor>,
    /// Decoration in front of the playing field.
    pub decor_front: Vec<Decor>,
    pub envelopes: Vec<Envelope>,
    pub images: Vec<Image>,
    /// Adventure objects (A1.5); empty for multiplayer maps.
    pub adventure: adventure::Adventure,
}

impl Map {
    /// Reads a map file.
    ///
    /// # Errors
    /// If the file is missing or is not a valid map.
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let data = std::fs::read(path)?;
        Ok(decode(&data)?)
    }

    /// Writes the map as a file.
    ///
    /// # Errors
    /// If the file cannot be written.
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        std::fs::write(path, encode(self))
    }

    /// Collision grid for the simulation.
    pub fn collision(&self) -> Collision {
        Collision::new(self.width, self.height, self.tiles.clone())
    }

    /// World with spawn points, pickups and dummies of this map (without players).
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
                // Flags are placed by the rules in CTF mode (elora-game)
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

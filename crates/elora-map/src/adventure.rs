//! Adventure objects of a map (R2-M1, A1.5, E-252 to E-259): enemies, NPCs, chests,
//! switches, doors, collectibles, save points, healing plants, entrances, transitions, zones
//! and camera zones. Section `ADVN` in the map format; multiplayer maps have none.
//!
//! The map only checks the structure (unique ids, position, sizes); whether enemy kinds,
//! dialogues or target maps exist is checked by the adventure (`elora-adventure`).

use elora_sim::{TILE_SIZE, Vec2};

/// Maximum number of objects on a map.
pub const MAX_OBJECTS: usize = 4096;
/// Maximum number of entries in lists (chest contents).
pub const MAX_LIST: usize = 64;

/// How a switch is triggered (E-256).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SwitchTrigger {
    /// Action key.
    #[default]
    Interact,
    /// Hammer hit.
    Hammer,
    /// Hook with the pull-hook ability (pull switch, R2-M2.2).
    Hook,
}

/// Camera zone (E-259).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraMode {
    /// Fixed view: the camera shows the zone.
    #[default]
    Fixed,
    /// Clamp: the camera stays inside the zone.
    Bounds,
}

/// Kind of an object with its settings.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectKind {
    /// Enemy (`creatures.toml`); `persistent` = boss/special, stays defeated (E-235).
    Creature { kind: String, persistent: bool },
    /// Character with dialogue (E-257); `walk` = half the length of its walking path (0 = stands).
    Npc {
        character: String,
        dialog: String,
        facing: i8,
        walk: f32,
    },
    /// Chest with fixed contents (E-255); `lock` = condition to open it (empty = open).
    Chest {
        contents: Vec<(String, u32)>,
        lock: String,
    },
    /// Lever or hit switch (E-256): sets `flag` to 1/0.
    Switch {
        flag: String,
        once: bool,
        trigger: SwitchTrigger,
    },
    /// Door/gate (E-254): block of `size` (tiles, from `pos` as top-left corner), closed like
    /// stone, open as soon as `open_if` holds.
    Door { size: (u8, u8), open_if: String },
    /// One-time find (e.g. glitter stone).
    Collectible { item: String },
    /// Source stone (P-31).
    SavePoint,
    /// Healing plant (E-258), grows back when the area is re-entered.
    HealPlant { heal: i32 },
    /// Entrance: Elora appears here when a transition with this id arrives.
    Spawn,
    /// Transition to `map`/`spawn` (E-252); area `size` from `pos`. `on_touch` = on
    /// walking in, otherwise with the action key.
    Exit {
        size: Vec2,
        map: String,
        spawn: String,
        on_touch: bool,
    },
    /// Trigger zone for quests ("reach a place"); area `size` from `pos`.
    Zone { size: Vec2 },
    /// Camera zone; area `size` from `pos`.
    Camera { size: Vec2, mode: CameraMode },
}

impl ObjectKind {
    /// Identifier for the editor and error messages.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Creature { .. } => "gegner",
            Self::Npc { .. } => "npc",
            Self::Chest { .. } => "truhe",
            Self::Switch { .. } => "schalter",
            Self::Door { .. } => "tuer",
            Self::Collectible { .. } => "sammelstueck",
            Self::SavePoint => "speicherpunkt",
            Self::HealPlant { .. } => "heilpflanze",
            Self::Spawn => "eingang",
            Self::Exit { .. } => "uebergang",
            Self::Zone { .. } => "zone",
            Self::Camera { .. } => "kamera",
        }
    }

    /// Size of an area (transition, zone, camera, door) in units.
    pub fn area(&self) -> Option<Vec2> {
        match self {
            Self::Exit { size, .. } | Self::Zone { size } | Self::Camera { size, .. } => {
                Some(*size)
            }
            Self::Door { size, .. } => {
                #[allow(clippy::cast_precision_loss)]
                let ts = TILE_SIZE as f32;
                Some(Vec2::new(f32::from(size.0) * ts, f32::from(size.1) * ts))
            }
            _ => None,
        }
    }
}

/// An object. `id` is unique on the map and a key in the save game
/// (e.g. opened chest `wiese-1:truhe-3`).
#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub id: String,
    /// Center (characters, items) or top-left corner (areas, doors) in units.
    pub pos: Vec2,
    pub kind: ObjectKind,
}

/// Adventure part of a map.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Adventure {
    pub objects: Vec<Object>,
}

impl Adventure {
    pub fn object(&self, id: &str) -> Option<&Object> {
        self.objects.iter().find(|o| o.id == id)
    }

    /// Check the structure (map `w`×`h` tiles).
    ///
    /// # Errors
    /// With the first error and the id of the object.
    pub fn validate(&self, w: usize, h: usize) -> Result<(), String> {
        #[allow(clippy::cast_precision_loss)]
        let (ts, mw, mh) = (
            TILE_SIZE as f32,
            w as f32 * TILE_SIZE as f32,
            h as f32 * TILE_SIZE as f32,
        );
        if self.objects.len() > MAX_OBJECTS {
            return Err(format!("more than {MAX_OBJECTS} adventure objects"));
        }
        let mut ids = std::collections::BTreeSet::new();
        for o in &self.objects {
            let at = |m: &str| Err(format!("object `{}` ({}): {m}", o.id, o.kind.name()));
            if o.id.is_empty() || o.id.len() > crate::binary::MAX_NAME || o.id.contains(':') {
                return at("id empty, too long or containing `:`");
            }
            if !ids.insert(o.id.as_str()) {
                return at("duplicate id");
            }
            let inside = |p: Vec2| p.x >= 0.0 && p.y >= 0.0 && p.x <= mw && p.y <= mh;
            if !o.pos.x.is_finite() || !o.pos.y.is_finite() || !inside(o.pos) {
                return at("lies outside the map");
            }
            if let Some(size) = o.kind.area()
                && (size.x <= 0.0 || size.y <= 0.0 || !inside(o.pos + size))
            {
                return at("area empty or outside the map");
            }
            match &o.kind {
                ObjectKind::Door { .. } if o.pos.x % ts != 0.0 || o.pos.y % ts != 0.0 => {
                    return at("door is not on the tile grid");
                }
                ObjectKind::Chest { contents, .. } if contents.len() > MAX_LIST => {
                    return at("too many items");
                }
                ObjectKind::Npc { facing, walk, .. }
                    if !matches!(facing, -1 | 1) || *walk < 0.0 =>
                {
                    return at("facing or walk path invalid");
                }
                ObjectKind::Exit { map, spawn, .. } if map.is_empty() || spawn.is_empty() => {
                    return at("target missing");
                }
                _ => {}
            }
        }
        Ok(())
    }
}

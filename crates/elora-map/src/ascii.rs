//! Karten aus Zeichenrastern bauen – für Tests und Werkzeuge, kein Dateiformat (E-146).
//!
//! Tiles nutzen dieselben Zeichen wie die Aufzeichnungen ([`Tile::from_char`]), Entities Buchstaben.

use elora_sim::{DummyPattern, Tile};

use crate::{Entity, EntityKind, Map, MapError};

/// Zeichen der Entities.
pub const ENTITY_CHARS: [(char, EntityKind); 13] = [
    ('S', EntityKind::Spawn),
    ('R', EntityKind::SpawnRed),
    ('B', EntityKind::SpawnBlue),
    ('r', EntityKind::FlagRed),
    ('b', EntityKind::FlagBlue),
    ('h', EntityKind::Health),
    ('a', EntityKind::Armor),
    ('L', EntityKind::Laser),
    ('G', EntityKind::Grenade),
    ('D', EntityKind::Dummy(DummyPattern::Stand)),
    ('W', EntityKind::Dummy(DummyPattern::Walk)),
    ('J', EntityKind::Dummy(DummyPattern::Jump)),
    ('X', EntityKind::Dummy(DummyPattern::WalkJump)),
];

impl Map {
    /// Leere Karte (nur Luft) ohne Aussehen.
    pub fn new(name: &str, width: usize, height: usize) -> Self {
        Self {
            name: name.to_owned(),
            author: None,
            width,
            height,
            tiles: vec![Tile::Air; width * height],
            entities: Vec::new(),
            materials: Vec::new(),
            material_map: Vec::new(),
            sky: crate::Sky::default(),
            backgrounds: Vec::new(),
            decor_back: Vec::new(),
            decor_front: Vec::new(),
            envelopes: Vec::new(),
            images: Vec::new(),
        }
    }

    /// Karte aus Zeilen gleicher Länge; ein Entity macht sein Feld zu Luft.
    ///
    /// # Errors
    /// Bei unterschiedlich langen Zeilen, unbekannten Zeichen, zu großem Raster oder unspielbarer Karte.
    pub fn from_rows(name: &str, rows: &[&str]) -> Result<Self, MapError> {
        let width = rows.first().map_or(0, |r| r.chars().count());
        let height = rows.len();
        if width == 0 {
            return Err(MapError::Invalid("leeres Raster"));
        }
        if width > crate::MAX_SIZE || height > crate::MAX_SIZE {
            return Err(MapError::TooLarge { width, height });
        }
        let mut map = Self::new(name, width, height);
        for (ty, row) in rows.iter().enumerate() {
            let found = row.chars().count();
            if found != width {
                return Err(MapError::RaggedRow {
                    line: ty + 1,
                    found,
                    expected: width,
                });
            }
            for (tx, c) in row.chars().enumerate() {
                if let Some(t) = Tile::from_char(c) {
                    map.tiles[ty * width + tx] = t;
                } else if let Some(&(_, kind)) = ENTITY_CHARS.iter().find(|(e, _)| *e == c) {
                    map.entities.push(Entity { kind, tx, ty });
                } else {
                    return Err(MapError::UnknownSymbol {
                        line: ty + 1,
                        column: tx + 1,
                        symbol: c,
                    });
                }
            }
        }
        crate::binary::validate(&map)?;
        Ok(map)
    }

    /// Kollision und Entities als Zeichenraster (Gegenstück zu [`Map::from_rows`]).
    pub fn to_rows(&self) -> Vec<String> {
        let mut rows: Vec<Vec<char>> = self
            .tiles
            .chunks(self.width)
            .map(|r| r.iter().map(|t| t.to_char()).collect())
            .collect();
        for e in &self.entities {
            if let Some(&(c, _)) = ENTITY_CHARS.iter().find(|(_, k)| *k == e.kind) {
                rows[e.ty][e.tx] = c;
            }
        }
        rows.into_iter().map(String::from_iter).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_roundtrip_and_errors() {
        let rows = ["#####", "#S.h#", "#%^=#", "#!<>#", "#####"];
        let m = Map::from_rows("T", &rows).unwrap();
        assert_eq!((m.width, m.height), (5, 5));
        assert_eq!(m.tiles[2 * 5 + 1], Tile::Unhookable);
        assert_eq!(m.tiles[5 + 1], Tile::Air, "Entity-Feld ist Luft");
        assert_eq!(m.to_rows(), rows);
        assert_eq!(
            Map::from_rows("T", &["###", "#S##"]).unwrap_err(),
            MapError::RaggedRow {
                line: 2,
                found: 4,
                expected: 3
            }
        );
        assert_eq!(
            Map::from_rows("T", &["#S#", "#x#"]).unwrap_err(),
            MapError::UnknownSymbol {
                line: 2,
                column: 2,
                symbol: 'x'
            }
        );
        assert_eq!(
            Map::from_rows("T", &["#.#"]).unwrap_err(),
            MapError::NoSpawn
        );
        assert!(
            Map::from_rows("T", &["#RrbB#"])
                .unwrap()
                .supported_modes()
                .ctf
        );
    }

    #[test]
    fn bundled_maps_load() {
        for (data, size) in [
            (&include_bytes!("../../../maps/sandbox.emap")[..], (48, 20)),
            (&include_bytes!("../../../maps/ctf-test.emap")[..], (0, 0)),
            (
                &include_bytes!("../../../maps/tiles-test.emap")[..],
                (60, 27),
            ),
        ] {
            let m = crate::decode(data).unwrap();
            if size != (0, 0) {
                assert_eq!((m.width, m.height), size);
            }
            assert!(m.supported_modes().free_for_all || m.supported_modes().ctf);
        }
    }
}

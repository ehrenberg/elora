//! Parser für das Textformat `.emap.toml` (E-024).

use std::collections::HashMap;

use elora_sim::{DummyPattern, Tile};
use serde::Deserialize;

use crate::{Entity, EntityKind, MAX_SIZE, Map, TEXT_FORMAT_VERSION};

/// Fehler beim Laden einer Textkarte. Zeilen und Spalten beginnen bei 1.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MapError {
    #[error("ungültiges TOML: {0}")]
    Toml(String),
    #[error("nicht unterstützte Formatversion {found} (unterstützt: {TEXT_FORMAT_VERSION})")]
    UnsupportedFormat { found: u32 },
    #[error("Legende: unbekannter Eintrag `{value}` für Zeichen `{symbol}`")]
    UnknownLegendValue { symbol: String, value: String },
    #[error("Legende: Schlüssel `{0}` muss genau ein Zeichen sein")]
    InvalidLegendKey(String),
    #[error("Raster ist leer")]
    EmptyGrid,
    #[error(
        "Zeile {line}: Länge {found}, erwartet {expected} (alle Zeilen müssen gleich lang sein)"
    )]
    RaggedRow {
        line: usize,
        found: usize,
        expected: usize,
    },
    #[error("Raster {width}×{height} ist größer als erlaubt ({MAX_SIZE}×{MAX_SIZE})")]
    TooLarge { width: usize, height: usize },
    #[error("Zeile {line}, Spalte {column}: unbekanntes Zeichen `{symbol}`")]
    UnknownSymbol {
        line: usize,
        column: usize,
        symbol: char,
    },
    #[error("Karte hat keinen Spawnpunkt")]
    NoSpawn,
    #[error("Flaggen: {red}× rot, {blue}× blau – für CTF genau je eine, sonst keine")]
    InvalidFlags { red: usize, blue: usize },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMap {
    format: u32,
    name: String,
    author: Option<String>,
    #[serde(default)]
    legend: HashMap<String, String>,
    grid: RawGrid,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGrid {
    tiles: toml::Spanned<String>,
}

/// Bedeutung eines Zeichens im Raster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Symbol {
    Tile(Tile),
    Entity(EntityKind),
}

fn symbol_by_name(name: &str) -> Option<Symbol> {
    Some(match name {
        "air" => Symbol::Tile(Tile::Air),
        "solid" => Symbol::Tile(Tile::Solid),
        "unhookable" => Symbol::Tile(Tile::Unhookable),
        "death" => Symbol::Tile(Tile::Death),
        "spawn" => Symbol::Entity(EntityKind::Spawn),
        "spawn_red" => Symbol::Entity(EntityKind::SpawnRed),
        "spawn_blue" => Symbol::Entity(EntityKind::SpawnBlue),
        "flag_red" => Symbol::Entity(EntityKind::FlagRed),
        "flag_blue" => Symbol::Entity(EntityKind::FlagBlue),
        "health" => Symbol::Entity(EntityKind::Health),
        "armor" => Symbol::Entity(EntityKind::Armor),
        "laser" => Symbol::Entity(EntityKind::Laser),
        "grenade" => Symbol::Entity(EntityKind::Grenade),
        "dummy" => Symbol::Entity(EntityKind::Dummy(DummyPattern::Stand)),
        "dummy_walk" => Symbol::Entity(EntityKind::Dummy(DummyPattern::Walk)),
        "dummy_jump" => Symbol::Entity(EntityKind::Dummy(DummyPattern::Jump)),
        "dummy_walk_jump" => Symbol::Entity(EntityKind::Dummy(DummyPattern::WalkJump)),
        _ => return None,
    })
}

/// Standard-Legende: Satzzeichen sind Tiles, Buchstaben sind Entities.
fn default_legend() -> HashMap<char, Symbol> {
    [
        ('.', "air"),
        ('#', "solid"),
        ('%', "unhookable"),
        ('^', "death"),
        ('S', "spawn"),
        ('R', "spawn_red"),
        ('B', "spawn_blue"),
        ('r', "flag_red"),
        ('b', "flag_blue"),
        ('h', "health"),
        ('a', "armor"),
        ('L', "laser"),
        ('G', "grenade"),
        ('D', "dummy"),
        ('W', "dummy_walk"),
        ('J', "dummy_jump"),
        ('X', "dummy_walk_jump"),
    ]
    .into_iter()
    .map(|(c, n)| (c, symbol_by_name(n).expect("Standard-Legende ist gültig")))
    .collect()
}

/// Liest eine Karte im Textformat.
///
/// # Errors
/// Bei ungültigem TOML, falscher Formatversion oder ungültigem Raster
/// (mit Zeile und Spalte in der Datei).
pub fn parse_text_map(source: &str) -> Result<Map, MapError> {
    let raw: RawMap = toml::from_str(source).map_err(|e| MapError::Toml(e.to_string()))?;
    if raw.format != TEXT_FORMAT_VERSION {
        return Err(MapError::UnsupportedFormat { found: raw.format });
    }

    let mut legend = default_legend();
    for (key, value) in &raw.legend {
        let mut chars = key.chars();
        let (Some(symbol), None) = (chars.next(), chars.next()) else {
            return Err(MapError::InvalidLegendKey(key.clone()));
        };
        let meaning = symbol_by_name(value).ok_or_else(|| MapError::UnknownLegendValue {
            symbol: key.clone(),
            value: value.clone(),
        })?;
        legend.insert(symbol, meaning);
    }

    // Zeile in der Datei, in der das Raster beginnt: `'''` + direkt folgender Umbruch
    let grid_start = raw.grid.tiles.span().start;
    let first_line = source[..grid_start].matches('\n').count() + 2;

    let lines: Vec<(usize, &str)> = raw
        .grid
        .tiles
        .get_ref()
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .enumerate()
        .map(|(i, l)| (first_line + i, l))
        .collect();
    let first = lines.iter().position(|(_, l)| !l.trim().is_empty());
    let last = lines.iter().rposition(|(_, l)| !l.trim().is_empty());
    let (Some(first), Some(last)) = (first, last) else {
        return Err(MapError::EmptyGrid);
    };
    let rows = &lines[first..=last];

    let width = rows[0].1.chars().count();
    let height = rows.len();
    if width > MAX_SIZE || height > MAX_SIZE {
        return Err(MapError::TooLarge { width, height });
    }

    let mut tiles = Vec::with_capacity(width * height);
    let mut entities = Vec::new();
    for (ty, &(line, row)) in rows.iter().enumerate() {
        let found = row.chars().count();
        if found != width {
            return Err(MapError::RaggedRow {
                line,
                found,
                expected: width,
            });
        }
        for (tx, symbol) in row.chars().enumerate() {
            match legend.get(&symbol) {
                Some(Symbol::Tile(tile)) => tiles.push(*tile),
                Some(Symbol::Entity(kind)) => {
                    tiles.push(Tile::Air);
                    entities.push(Entity {
                        kind: *kind,
                        tx,
                        ty,
                    });
                }
                None => {
                    return Err(MapError::UnknownSymbol {
                        line,
                        column: tx + 1,
                        symbol,
                    });
                }
            }
        }
    }

    let map = Map {
        name: raw.name,
        author: raw.author,
        width,
        height,
        tiles,
        entities,
    };
    validate(&map)?;
    Ok(map)
}

fn validate(map: &Map) -> Result<(), MapError> {
    let count = |k| map.entities_of(k).count();
    let spawns =
        count(EntityKind::Spawn) + count(EntityKind::SpawnRed) + count(EntityKind::SpawnBlue);
    if spawns == 0 {
        return Err(MapError::NoSpawn);
    }
    let (red, blue) = (count(EntityKind::FlagRed), count(EntityKind::FlagBlue));
    if (red, blue) != (0, 0) && (red, blue) != (1, 1) {
        return Err(MapError::InvalidFlags { red, blue });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(grid: &str) -> String {
        format!("format = 1\nname = \"Test\"\n\n[grid]\ntiles = '''\n{grid}'''\n")
    }

    #[test]
    fn parses_tiles_and_entities() {
        let m = parse_text_map(&map("#####\n#S.h#\n#%^.#\n#####\n")).unwrap();
        assert_eq!((m.width, m.height), (5, 4));
        assert_eq!(m.tiles[5 + 1], Tile::Air); // Spawn-Feld wird Luft
        assert_eq!(m.tiles[10 + 1], Tile::Unhookable);
        assert_eq!(m.tiles[10 + 2], Tile::Death);
        assert_eq!(m.entities.len(), 2);
        assert_eq!(
            m.entities[0],
            Entity {
                kind: EntityKind::Spawn,
                tx: 1,
                ty: 1
            }
        );
        assert!(m.supported_modes().free_for_all);
        assert!(!m.supported_modes().team);
    }

    #[test]
    fn reports_line_and_column_of_unknown_symbol() {
        // Raster beginnt in Zeile 6 der Datei
        let err = parse_text_map(&map("###\n#S#\n#x#\n###\n")).unwrap_err();
        assert_eq!(
            err,
            MapError::UnknownSymbol {
                line: 8,
                column: 2,
                symbol: 'x'
            }
        );
    }

    #[test]
    fn rejects_ragged_rows() {
        let err = parse_text_map(&map("###\n#S##\n###\n")).unwrap_err();
        assert_eq!(
            err,
            MapError::RaggedRow {
                line: 7,
                found: 4,
                expected: 3
            }
        );
    }

    #[test]
    fn ignores_surrounding_blank_lines_and_crlf() {
        let m = parse_text_map(&map("\n\n###\r\n#S#\r\n###\r\n\n")).unwrap();
        assert_eq!((m.width, m.height), (3, 3));
    }

    #[test]
    fn requires_spawn() {
        assert_eq!(
            parse_text_map(&map("###\n#.#\n###\n")),
            Err(MapError::NoSpawn)
        );
    }

    #[test]
    fn validates_flags() {
        let err = parse_text_map(&map("#####\n#Rrr#\n#####\n")).unwrap_err();
        assert_eq!(err, MapError::InvalidFlags { red: 2, blue: 0 });
        let m = parse_text_map(&map("######\n#RrbB#\n######\n")).unwrap();
        assert!(m.supported_modes().ctf);
    }

    #[test]
    fn custom_legend() {
        let src = "format = 1\nname = \"T\"\n[legend]\n\"~\" = \"death\"\n[grid]\ntiles = '''\n#~S#\n'''\n";
        let m = parse_text_map(src).unwrap();
        assert_eq!(m.tiles[1], Tile::Death);
        let bad = src.replace("\"death\"", "\"lava\"");
        assert!(matches!(
            parse_text_map(&bad),
            Err(MapError::UnknownLegendValue { .. })
        ));
    }

    #[test]
    fn dummy_symbols() {
        let m = parse_text_map(&map("#######\n#SDWJX#\n#######\n")).unwrap();
        let kinds: Vec<_> = m.entities.iter().map(|e| e.kind).collect();
        assert_eq!(
            kinds[1..],
            [
                EntityKind::Dummy(DummyPattern::Stand),
                EntityKind::Dummy(DummyPattern::Walk),
                EntityKind::Dummy(DummyPattern::Jump),
                EntityKind::Dummy(DummyPattern::WalkJump),
            ]
        );
    }

    #[test]
    fn rejects_unknown_format() {
        let src = map("#S#\n").replace("format = 1", "format = 2");
        assert_eq!(
            parse_text_map(&src),
            Err(MapError::UnsupportedFormat { found: 2 })
        );
    }

    #[test]
    fn sandbox_map_is_valid() {
        let src = include_str!("../../../maps/sandbox.emap.toml");
        let m = parse_text_map(src).unwrap();
        assert_eq!((m.width, m.height), (48, 20));
        assert!(m.supported_modes().free_for_all);
    }
}

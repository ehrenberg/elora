//! Release-Karten bauen (M6.10): Layout aus [`super::release_layouts`], Thema (Material,
//! Himmel, Hintergrund-Vorlage, Deko) und Deko auf den Oberflächen – mit den Werkzeugen des Editors.
//!
//! `cargo test -p elora-client --bin elora write_release_maps -- --ignored` schreibt `maps/*.emap`.

use std::time::Instant;

use elora_map::look::{Curve, EnvKind, EnvPoint, EnvRef, Envelope};
use elora_map::{Art, Decor, Map, Rgba, Sky};
use elora_sim::{TILE_SIZE, Tile, Vec2};

use super::Editor;
use super::look::Preset;
use super::release_layouts as layouts;

/// Thema einer Karte (E-155).
pub struct Theme {
    pub file: &'static str,
    pub name: &'static str,
    pub rows: &'static [&'static str],
    /// Material fester Tiles (`None` = Standard Erde).
    pub material: Option<&'static str>,
    pub preset: Preset,
    pub sky: Option<Sky>,
    /// Färbung der Hintergrund-Ebenen (nach Vorlage), `None` = unverändert.
    pub background_tint: Option<Rgba>,
    /// Wald-Ebene weglassen (Wüste).
    pub no_forest: bool,
    /// Große Deko hinter der Spielfläche (braucht Platz nach oben): Name, nötige Höhe in Tiles.
    pub back: &'static [(&'static str, usize)],
    /// Kleine Deko vor der Spielfläche.
    pub front: &'static [&'static str],
    /// Färbung aller Deko (Nacht).
    pub decor_tint: Option<Rgba>,
    /// Anteil der Oberflächen-Tiles mit Deko (vorn) und Abstand großer Deko (Tiles).
    pub front_density: u32,
    pub back_spacing: usize,
}

pub const THEMES: [Theme; 5] = [
    Theme {
        file: "dm-wiese",
        name: "Wiese",
        rows: layouts::WIESE,
        material: None,
        preset: Preset::Day,
        sky: None,
        background_tint: None,
        no_forest: false,
        back: &[
            ("tree-round", 6),
            ("bush-2", 2),
            ("bush-1", 2),
            ("fence", 2),
            ("rock-1", 1),
        ],
        front: &[
            "grass-1",
            "flower-pink",
            "grass-2",
            "flower-yellow",
            "flower-blue",
        ],
        decor_tint: None,
        front_density: 30,
        back_spacing: 7,
    },
    Theme {
        file: "dm-wueste",
        name: "Wüste",
        rows: layouts::WUESTE,
        material: Some("sand"),
        preset: Preset::Day,
        sky: Some(Sky {
            top: Rgba::hex(0x8fbcdf),
            bottom: Rgba::hex(0xf6e2bf),
        }),
        background_tint: Some(Rgba::hex(0xf0d2a0)),
        no_forest: true,
        back: &[("rock-2", 1), ("sign-board", 2), ("rock-1", 1)],
        front: &["rock-1", "grass-2"],
        decor_tint: None,
        front_density: 12,
        back_spacing: 11,
    },
    Theme {
        file: "dm-winter",
        name: "Winter",
        rows: layouts::WINTER,
        material: Some("snow"),
        preset: Preset::Day,
        sky: Some(Sky {
            top: Rgba::hex(0xc9dcec),
            bottom: Rgba::hex(0xf2f6fa),
        }),
        background_tint: Some(Rgba::hex(0xe4edf5)),
        no_forest: false,
        back: &[("tree-pine", 6), ("rock-2", 1), ("fence", 2)],
        front: &["rock-1"],
        decor_tint: None,
        front_density: 8,
        back_spacing: 8,
    },
    Theme {
        file: "ctf-wald",
        name: "Wald",
        rows: layouts::WALD,
        material: None,
        preset: Preset::Day,
        sky: None,
        background_tint: None,
        no_forest: false,
        back: &[
            ("tree-round", 6),
            ("tree-pine", 6),
            ("bush-2", 2),
            ("bush-1", 2),
            ("mushroom-brown", 1),
        ],
        front: &[
            "grass-1",
            "grass-2",
            "mushroom-red",
            "flower-blue",
            "grass-1",
        ],
        decor_tint: None,
        front_density: 35,
        back_spacing: 5,
    },
    Theme {
        file: "ctf-nacht",
        name: "Nacht",
        rows: layouts::NACHT,
        material: None,
        preset: Preset::Night,
        sky: None,
        background_tint: None,
        no_forest: false,
        back: &[
            ("tree-pine", 6),
            ("rock-2", 1),
            ("fence", 2),
            ("sign-board", 2),
        ],
        front: &["grass-2", "mushroom-red", "grass-1"],
        decor_tint: Some(Rgba::hex(0x8a94c0)),
        front_density: 18,
        back_spacing: 9,
    },
];

/// Fester Pseudo-Zufall je Feld (gleiche Karte bei jedem Bauen).
fn hash(x: usize, y: usize, salt: u32) -> u32 {
    #[allow(clippy::cast_possible_truncation)]
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ salt;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^ (h >> 13)
}

/// Freie Oberfläche: festes Tile (Erde/Sand/Schnee/Stein) mit Luft darüber, ohne Entity.
fn surface(map: &Map, x: usize, y: usize) -> bool {
    let w = map.width;
    y > 0
        && matches!(map.tiles[y * w + x], Tile::Solid | Tile::Unhookable)
        && map.tiles[(y - 1) * w + x] == Tile::Air
        && !map.entities.iter().any(|e| e.tx == x && e.ty == y - 1)
}

/// So viele Tiles Luft über (x, y) (bis `max`).
fn clearance(map: &Map, x: usize, y: usize, max: usize) -> usize {
    (1..=max)
        .take_while(|d| y >= *d && map.tiles[(y - d) * map.width + x] == Tile::Air)
        .count()
}

pub(super) fn place(theme: &Theme, map: &mut Map) {
    let ts = TILE_SIZE as f32;
    let tint = theme.decor_tint.unwrap_or(Rgba::WHITE);
    let mut next_back = 0;
    for y in 1..map.height {
        for x in 1..map.width - 1 {
            if !surface(map, x, y) {
                continue;
            }
            #[allow(clippy::cast_precision_loss)]
            let ground = Vec2::new(x as f32 * ts + ts / 2.0, y as f32 * ts);
            let h = hash(x, y, 7);
            // große Deko hinten: Abstand halten, genug Platz nach oben, nicht direkt an Kanten
            let edge = !surface(map, x - 1, y) || !surface(map, x + 1, y);
            if !edge && (y * 1000 + x) >= next_back && h.is_multiple_of(3) {
                let (name, need) = theme.back[(h as usize / 3) % theme.back.len()];
                if clearance(map, x, y, need + 1) > need {
                    let mut d = Decor::new(Art::Builtin(name.into()), ground);
                    d.flip_x = h & 8 != 0;
                    d.tint = tint;
                    map.decor_back.push(d);
                    next_back = y * 1000 + x + theme.back_spacing;
                    continue;
                }
            }
            if h % 100 < theme.front_density {
                let name = theme.front[(h as usize / 100) % theme.front.len()];
                let mut d = Decor::new(Art::Builtin(name.into()), ground);
                d.flip_x = h & 16 != 0;
                d.tint = tint;
                map.decor_front.push(d);
            }
        }
    }
}

/// Leuchtende Pilze in der Nacht: Farb-Animation, je Pilz versetzt.
fn glow(map: &mut Map) {
    if !map
        .decor_front
        .iter()
        .any(|d| d.art == Art::Builtin("mushroom-red".into()))
    {
        return;
    }
    let index = u16::try_from(map.envelopes.len()).unwrap_or(0);
    map.envelopes.push(Envelope {
        name: "Leuchten".into(),
        kind: EnvKind::Color,
        synced: false,
        points: vec![
            EnvPoint {
                time_ms: 0,
                value: [1.0; 4],
                curve: Curve::Smooth,
            },
            EnvPoint {
                time_ms: 1400,
                value: [1.0, 0.8, 0.85, 0.7],
                curve: Curve::Smooth,
            },
            EnvPoint {
                time_ms: 2800,
                value: [1.0; 4],
                curve: Curve::Smooth,
            },
        ],
    });
    for (k, d) in map.decor_front.iter_mut().enumerate() {
        if d.art == Art::Builtin("mushroom-red".into()) {
            d.tint = Rgba::WHITE;
            d.color_env = Some(EnvRef {
                index,
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                offset_ms: (k as i32 * 373) % 2800,
            });
        }
    }
}

/// Eine Release-Karte bauen.
///
/// # Panics
/// Wenn das Layout ungültig ist (von Tests abgedeckt).
pub fn build(theme: &Theme) -> Map {
    let mut editor = Editor::new(None, std::path::PathBuf::from("maps"));
    editor.map = Map::from_rows(theme.name, theme.rows).expect("Layout gültig");
    editor.map.author = Some("Elora-Team".into());
    let now = Instant::now();
    editor.apply_preset(theme.preset, now);
    let map = &mut editor.map;
    if let Some(m) = theme.material {
        map.materials = vec![m.into()];
        map.material_map = map
            .tiles
            .iter()
            .map(|t| u8::from(*t == Tile::Solid))
            .collect();
    }
    if let Some(sky) = theme.sky {
        map.sky = sky;
    }
    if theme.no_forest {
        map.backgrounds.retain(|b| b.name != "Wald");
    }
    if let Some(t) = theme.background_tint {
        for b in &mut map.backgrounds {
            for d in &mut b.items {
                if !matches!(&d.art, Art::Builtin(n) if n.starts_with("cloud")) {
                    d.tint = t;
                }
            }
        }
    }
    place(theme, map);
    glow(map);
    editor.map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_maps_are_valid_and_support_their_modes() {
        for theme in &THEMES {
            let map = build(theme);
            let data = elora_map::encode(&map);
            assert!(
                data.len() < elora_protocol::MAX_MAP,
                "{} zu groß",
                theme.file
            );
            let back = elora_map::decode(&data).unwrap_or_else(|e| panic!("{}: {e}", theme.file));
            let modes = back.supported_modes();
            if theme.file.starts_with("ctf") {
                assert!(modes.ctf && modes.team, "{}", theme.file);
            } else {
                assert!(modes.free_for_all, "{}", theme.file);
            }
            assert!(
                !back.decor_back.is_empty() && !back.decor_front.is_empty(),
                "{} ohne Deko",
                theme.file
            );
            assert!(!back.backgrounds.is_empty());
        }
    }

    /// Schreibt die Release-Karten nach `maps/` (nach Änderungen an Layout oder Thema).
    #[test]
    #[ignore = "schreibt maps/*.emap"]
    fn write_release_maps() {
        for theme in &THEMES {
            let path = format!(
                "{}/../../maps/{}.{}",
                env!("CARGO_MANIFEST_DIR"),
                theme.file,
                elora_map::EXTENSION
            );
            build(theme).save(std::path::Path::new(&path)).unwrap();
        }
    }

    /// Übersicht jeder Release-Karte: `… release_sheets -- --ignored` → `target/release-<datei>.svg`.
    #[test]
    #[ignore = "erzeugt nur Dateien zur Sichtprüfung"]
    fn release_sheets() {
        use crate::editor::panel::Preview;
        use crate::editor::view;
        for theme in &THEMES {
            let mut editor = Editor::new(None, std::path::PathBuf::from("maps"));
            editor.map = build(theme);
            editor.center_view();
            editor.visible.grid = false;
            #[allow(clippy::cast_precision_loss)]
            let size =
                Vec2::new(editor.map.width as f32, editor.map.height as f32) * TILE_SIZE as f32;
            let window = Vec2::new(1600.0, 1600.0 * size.y / size.x);
            editor.zoom = size.x / window.x;
            // Kamera wie im Spiel etwa in Bodenhöhe, damit die Parallax-Ebenen stimmen
            let cam = view::camera(&editor, window, window * 0.5);
            let mut batch = elora_render::ShapeBatch::default();
            view::draw(
                &mut batch,
                &editor,
                &mut crate::map_view::MapView::default(),
                &crate::items::ItemArt::load(),
                &cam,
                0.0,
                Preview::None,
            );
            let tl = cam.top_left();
            let svg = batch.debug_svg(tl, tl + cam.size, view::OUTSIDE);
            let path = format!(
                "{}/../../target/release-{}.svg",
                env!("CARGO_MANIFEST_DIR"),
                theme.file
            );
            std::fs::write(path, svg).unwrap();
        }
    }
}

//! Karten des Prologs (A1.9, `docs/release-2/prolog.md`): Tauwinkel und Blütenwiesen 1 –
//! Gelände, Objekte des Abenteuers und Deko (Gebäude E-278, verblasste Blumen E-277).
//!
//! `cargo test -p elora-client --bin elora write_prolog_maps -- --ignored` schreibt
//! `maps/abenteuer/*.emap`.

#![allow(clippy::cast_precision_loss, clippy::many_single_char_names)]

use std::time::Instant;

use elora_map::{Art, Decor, Map, Object, ObjectKind};
use elora_sim::{TILE_SIZE, Vec2};

use super::Editor;
use super::look::Preset;
use super::release::{self, Theme};

const T: f32 = TILE_SIZE as f32;

/// Zeichenraster der Karte; außen links und rechts Wand, unten ab `floor` Boden.
struct Grid(Vec<Vec<char>>);

impl Grid {
    fn new(w: usize, h: usize, floor: usize) -> Self {
        let mut g = vec![vec!['.'; w]; h];
        for (y, row) in g.iter_mut().enumerate() {
            for (x, c) in row.iter_mut().enumerate() {
                if y >= floor || x == 0 || x == w - 1 {
                    *c = '#';
                }
            }
        }
        Self(g)
    }

    /// Rechteck (Spalten `x`, Zeilen `y`, jeweils einschließlich) füllen.
    fn fill(&mut self, x: (usize, usize), y: (usize, usize), c: char) {
        for row in &mut self.0[y.0..=y.1] {
            for cell in &mut row[x.0..=x.1] {
                *cell = c;
            }
        }
    }

    /// Karte aus dem Raster; das `S` braucht nur das Textformat, die Karte nutzt Eingänge.
    fn map(&self, name: &str) -> Map {
        let mut g = self.0.clone();
        g[0][1] = 'S';
        let rows: Vec<String> = g.iter().map(|r| r.iter().collect()).collect();
        let r: Vec<&str> = rows.iter().map(String::as_str).collect();
        let mut m = Map::from_rows(name, &r).expect("Layout gültig");
        m.entities.clear();
        m
    }
}

/// Mitte über dem Boden (Oberkante von Zeile `ty`) in Spalte `tx` für ein Objekt der Höhe `h`.
fn at(tx: usize, ty: usize, h: f32) -> Vec2 {
    Vec2::new(tx as f32 * T + T / 2.0, ty as f32 * T - h / 2.0 - 1.0)
}

fn corner(tx: usize, ty: usize) -> Vec2 {
    Vec2::new(tx as f32 * T, ty as f32 * T)
}

fn o(id: &str, pos: Vec2, kind: ObjectKind) -> Object {
    Object {
        id: id.into(),
        pos,
        kind,
    }
}

fn npc(id: &str, tx: usize, ty: usize, facing: i8, walk: f32) -> Object {
    o(
        id,
        at(tx, ty, 28.0),
        ObjectKind::Npc {
            character: id.into(),
            dialog: id.into(),
            facing,
            walk,
        },
    )
}

/// Wegweiser-Schild mit Hinweis (E-273).
fn sign(dialog: &str, tx: usize, ty: usize) -> Object {
    o(
        dialog,
        at(tx, ty, 28.0),
        ObjectKind::Npc {
            character: "wegweiser".into(),
            dialog: dialog.into(),
            facing: 1,
            walk: 0.0,
        },
    )
}

fn creature(id: &str, kind: &str, tx: usize, ty: usize, h: f32) -> Object {
    o(
        id,
        at(tx, ty, h),
        ObjectKind::Creature {
            kind: kind.into(),
            persistent: false,
        },
    )
}

fn chest(id: &str, tx: usize, ty: usize, contents: &[(&str, u32)]) -> Object {
    o(
        id,
        at(tx, ty, 26.0),
        ObjectKind::Chest {
            contents: contents.iter().map(|(i, n)| ((*i).into(), *n)).collect(),
            lock: String::new(),
        },
    )
}

fn plant(id: &str, tx: usize, ty: usize) -> Object {
    o(id, at(tx, ty, 16.0), ObjectKind::HealPlant { heal: 2 })
}

/// Deko auf dem Boden (Oberkante von Zeile `ty`), Mitte in Spalte `tx` (halbe Tiles erlaubt).
fn decor(name: &str, tx: f32, ty: usize) -> Decor {
    Decor::new(
        Art::Builtin(name.into()),
        Vec2::new(tx * T + T / 2.0, ty as f32 * T),
    )
}

/// Gras vor der Spielfläche (Tauwinkel: keine bunten Blumen, E-210).
const GRASS: Theme = Theme {
    file: "",
    name: "",
    rows: &[],
    material: None,
    preset: Preset::Day,
    sky: None,
    background_tint: None,
    no_forest: false,
    back: &[("bush-1", 2)],
    front: &["grass-1", "grass-2"],
    decor_tint: None,
    front_density: 25,
    back_spacing: 1000,
};

fn finish(map: Map, theme: &Theme) -> Map {
    let mut editor = Editor::new(None, std::path::PathBuf::from("maps"));
    let (back, front) = (map.decor_back.clone(), map.decor_front.clone());
    editor.map = map;
    editor.map.author = Some("Elora-Team".into());
    editor.map.decor_back.clear();
    editor.map.decor_front.clear();
    editor.apply_preset(Preset::Day, Instant::now());
    release::place(theme, &mut editor.map);
    // eigene Deko zuerst (hinter der automatischen)
    editor.map.decor_back.splice(0..0, back);
    editor.map.decor_front.splice(0..0, front);
    editor.map
}

/// Tauwinkel, etwa 140 × 34: West (Eloras Haus, Baumhaus), Hecke, Steg, Brunnenplatz,
/// Werkstatt mit Hook-Übung, Schmiede mit Strohpuppen, Laden, Ostpfad.
pub fn tauwinkel() -> Map {
    let (w, h, f) = (140, 34, 28);
    let mut g = Grid::new(w, h, f);
    // Hecke: nur mit Doppelsprung
    g.fill((30, 32), (21, f - 1), '#');
    // Steg über einer Mulde (Runter zum Durchfallen, unten eine Truhe)
    g.fill((35, 43), (f, f + 2), '.');
    g.fill((35, 43), (f, f), '=');
    // Hook-Übung: Überhang und hoher Sims hinter der Werkstatt
    g.fill((78, 93), (8, 9), '#');
    g.fill((84, 89), (15, 15), '#');
    let mut m = g.map("Tauwinkel");
    m.adventure.objects = vec![
        o("start", at(12, f, 28.0), ObjectKind::Spawn),
        sign("schild-start", 15, f),
        npc("pip", 19, f, -1, 0.0),
        sign("schild-hecke", 28, f),
        sign("schild-plattform", 37, f),
        chest("truhe-1", 41, f + 3, &[("glanztropfen", 20)]),
        sign("schild-brunnen", 47, f),
        o("brunnen", at(50, f, 40.0), ObjectKind::SavePoint),
        npc("oma", 57, f, -1, 0.0),
        npc("tueftel", 69, f, 1, 24.0),
        sign("schild-hook", 77, f),
        chest(
            "truhe-hook",
            87,
            15,
            &[("glanztropfen", 15), ("heiltrank", 1)],
        ),
        npc("klonk", 95, f, 1, 0.0),
        sign("schild-hammer", 101, f),
        creature("puppe-1", "strohpuppe", 104, f, 40.0),
        creature("puppe-2", "strohpuppe", 107, f, 40.0),
        creature("puppe-3", "strohpuppe", 110, f, 40.0),
        npc("lotte", 119, f, -1, 0.0),
        sign("schild-ostpfad", 128, f),
        o("ost", at(132, f, 28.0), ObjectKind::Spawn),
        o(
            "weg-wiese",
            corner(137, f - 8),
            ObjectKind::Exit {
                size: Vec2::new(2.0 * T, 8.0 * T),
                map: "wiese-1".into(),
                spawn: "west".into(),
                on_touch: true,
            },
        ),
    ];
    m.decor_back = vec![
        decor("tree-round", 3.0, f),
        decor("haus-elora", 9.0, f),
        decor("baumhaus", 23.0, f),
        decor("fence", 27.0, f),
        decor("fahne-blass", 45.0, f),
        decor("brunnen", 53.5, f),
        decor("haus-oma", 61.0, f),
        decor("fahne-blass", 65.5, f),
        decor("werkstatt", 72.0, f),
        decor("schmiede", 97.0, f),
        decor("laden", 117.0, f),
        decor("anschlagbrett", 123.5, f),
        decor("fahne-blass", 126.0, f),
        decor("tree-round", 134.0, f),
    ];
    m.decor_front = vec![
        decor("bush-2", 30.0, 21),
        decor("bush-2", 31.5, 21),
        decor("bush-2", 32.5, 21),
        decor("beet-blass", 6.0, f),
        decor("blumenkasten-blass", 13.5, f),
        decor("beet-blass", 52.0, f),
        decor("kraeuterbeet-blass", 59.5, f),
        decor("kraeuterbeet-blass", 63.0, f),
        decor("blumenkasten-blass", 114.0, f),
        decor("beet-blass", 121.5, f),
    ];
    finish(m, &GRASS)
}

/// Blütenwiesen 1, etwa 180 × 40: Hügel, Plattformen mit Truhe, Bach mit Brücke (drei
/// Stachelkäfer), Pollenbläser, hoher Glitzerstein (Hook), Bröckelboden, Quellstein am Wiesenrand.
pub fn wiese() -> Map {
    let (w, h, f) = (180, 40, 32);
    let mut g = Grid::new(w, h, f);
    // Hügel
    g.fill((14, 15), (f - 1, f - 1), '#');
    g.fill((16, 24), (f - 2, f - 1), '#');
    g.fill((25, 26), (f - 1, f - 1), '#');
    // Plattformen hinauf zur Truhe
    g.fill((44, 50), (27, 27), '=');
    g.fill((51, 57), (22, 22), '=');
    // Bach mit Brücke
    g.fill((62, 73), (f, f + 3), '.');
    g.fill((62, 73), (f, f), '=');
    // großer Hügel
    g.fill((110, 111), (f - 1, f - 1), '#');
    g.fill((112, 128), (f - 3, f - 1), '#');
    g.fill((129, 130), (f - 1, f - 1), '#');
    // hoher Glitzerstein: Überhang zum Hooken, Sims darunter
    g.fill((134, 150), (11, 12), '#');
    g.fill((140, 145), (18, 18), '#');
    // Bröckelboden über einer Grube
    g.fill((150, 153), (f, f + 3), '.');
    g.fill((150, 153), (f, f), ':');
    let mut m = g.map("Blütenwiesen 1");
    m.adventure.objects = vec![
        o("west", at(5, f, 28.0), ObjectKind::Spawn),
        o(
            "weg-dorf",
            corner(1, f - 8),
            ObjectKind::Exit {
                size: Vec2::new(T, 8.0 * T),
                map: "tauwinkel".into(),
                spawn: "ost".into(),
                on_touch: true,
            },
        ),
        sign("schild-wiese", 9, f),
        plant("blume-1", 20, f - 2),
        creature("kaefer-1", "stachelkaefer", 32, f, 26.0),
        creature("huepfer-1", "grashuepfer", 40, f, 28.0),
        chest(
            "truhe-wiese",
            54,
            22,
            &[("glanztropfen", 25), ("heiltrank", 1)],
        ),
        o(
            "bruecke",
            corner(58, f - 8),
            ObjectKind::Zone {
                size: Vec2::new(28.0 * T, 12.0 * T),
            },
        ),
        creature("kaefer-2", "stachelkaefer", 70, f, 26.0),
        creature("kaefer-3", "stachelkaefer", 78, f, 26.0),
        creature("kaefer-4", "stachelkaefer", 83, f, 26.0),
        plant("blume-2", 67, f + 4),
        creature("blaeser", "pollenblaeser", 95, f, 60.0),
        creature("huepfer-2", "grashuepfer", 104, f, 28.0),
        plant("blume-3", 120, f - 3),
        creature("huepfer-3", "grashuepfer", 136, f, 28.0),
        o(
            "glitzerstein",
            at(142, 18, 28.0),
            ObjectKind::Collectible {
                item: "glitzerstein".into(),
            },
        ),
        plant("blume-4", 157, f),
        o(
            "wiesenrand",
            corner(162, f - 10),
            ObjectKind::Zone {
                size: Vec2::new(16.0 * T, 10.0 * T),
            },
        ),
        o("quellstein", at(170, f, 40.0), ObjectKind::SavePoint),
    ];
    m.decor_back = vec![decor("tree-round", 166.0, f)];
    m.decor_front = vec![decor("bush-2", 141.0, 18), decor("bush-1", 144.5, 18)];
    finish(m, &release::THEMES[0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_adventure::Content;
    use elora_adventure::check::{map_links, map_objects};

    fn shipped(name: &str) -> String {
        format!(
            "{}/../../maps/abenteuer/{name}.{}",
            env!("CARGO_MANIFEST_DIR"),
            elora_map::EXTENSION
        )
    }

    #[test]
    fn prolog_maps_match_content_and_link_up() {
        let c = Content::builtin();
        let (a, b) = (tauwinkel(), wiese());
        for m in [&a, &b] {
            let back = elora_map::decode(&elora_map::encode(m)).expect("Karte gültig");
            let errors = map_objects(&c, &back);
            assert!(errors.is_empty(), "{}: {errors:?}", m.name);
            assert!(!back.decor_back.is_empty() && !back.backgrounds.is_empty());
        }
        let errors = map_links(&[("tauwinkel", &a), ("wiese-1", &b)]);
        assert!(errors.is_empty(), "{errors:?}");
        assert!(a.adventure.object(&c.progression.start_spawn).is_some());
        // Zonen und Gegner der Hauptaufgabe
        assert!(b.adventure.object("wiesenrand").is_some());
        let kaefer = b
            .adventure
            .objects
            .iter()
            .filter(
                |o| matches!(&o.kind, ObjectKind::Creature { kind, .. } if kind == "stachelkaefer"),
            )
            .count();
        assert!(kaefer >= 3);
    }

    #[test]
    fn shipped_prolog_maps_are_current() {
        for (name, map) in [("tauwinkel", tauwinkel()), ("wiese-1", wiese())] {
            let file = std::fs::read(shipped(name)).expect("Karte vorhanden");
            assert_eq!(
                elora_map::decode(&file).expect("gültig"),
                map,
                "{name} veraltet – write_prolog_maps -- --ignored"
            );
        }
    }

    #[test]
    #[ignore = "schreibt maps/abenteuer/*.emap"]
    fn write_prolog_maps() {
        for (name, map) in [("tauwinkel", tauwinkel()), ("wiese-1", wiese())] {
            map.save(std::path::Path::new(&shipped(name))).unwrap();
        }
    }

    /// Übersicht: `… prolog_sheets -- --ignored` → `target/prolog-<karte>.svg`.
    #[test]
    #[ignore = "erzeugt nur Dateien zur Sichtprüfung"]
    fn prolog_sheets() {
        use crate::editor::panel::Preview;
        use crate::editor::view;
        for (name, map) in [("tauwinkel", tauwinkel()), ("wiese-1", wiese())] {
            let mut editor = Editor::new(None, std::path::PathBuf::from("maps"));
            editor.map = map;
            editor.center_view();
            editor.visible.grid = false;
            let size =
                Vec2::new(editor.map.width as f32, editor.map.height as f32) * TILE_SIZE as f32;
            let window = Vec2::new(2400.0, 2400.0 * size.y / size.x);
            editor.zoom = size.x / window.x;
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
                "{}/../../target/prolog-{name}.svg",
                env!("CARGO_MANIFEST_DIR")
            );
            std::fs::write(path, svg).unwrap();
        }
    }
}

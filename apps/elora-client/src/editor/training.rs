//! Trainingskarte (E-293): etwa 120 × 40, ein Bereich je Neuerung – Bewegung, Hook,
//! Fähigkeiten (Kletterwand, Bröckelboden, Gleiten), Sondertiles (Eis, Beschleuniger,
//! Sprungfelder), Waffen an Dummies und ein Gegner-Übungsplatz.
//!
//! `cargo test -p elora-client --bin elora write_training_map -- --ignored` schreibt
//! `maps/training.emap`.

#![allow(clippy::cast_precision_loss, clippy::many_single_char_names)]

use elora_map::{Map, ObjectKind};

use super::prolog::{Grid, animate, creature, decor, finish};
use super::release;

/// Zeichen setzen (Tiles wie im Textformat, Entities als Buchstaben).
fn put(g: &mut Grid, x: usize, y: usize, c: char) {
    g.0[y][x] = c;
}

pub fn training() -> Map {
    let (w, h, f) = (120, 40, 34);
    let mut g = Grid::new(w, h, f);
    // Rahmen: Arena mit Decke und Wänden
    g.fill((0, w - 1), (0, 0), '#');
    g.fill((0, 0), (0, h - 1), '#');
    g.fill((w - 1, w - 1), (0, h - 1), '#');

    // Start: Spawns, Leben, Rüstung, Waffen auf zwei Stegen
    put(&mut g, 3, f - 1, 'S');
    put(&mut g, 8, f - 1, 'S');
    put(&mut g, 12, f - 1, 'h');
    put(&mut g, 15, f - 1, 'a');
    g.fill((3, 9), (30, 30), '=');
    put(&mut g, 6, 29, 'G');
    g.fill((12, 16), (30, 30), '=');
    put(&mut g, 14, 29, 'L');

    // Bewegung: Treppe, Sprungfelder, Stege nach oben
    g.ground(19, 21, 32);
    g.ground(22, 24, 30);
    g.ground(25, 27, 28);
    put(&mut g, 30, f, '!');
    put(&mut g, 34, f, '/');
    g.fill((28, 34), (24, 24), '=');
    g.fill((30, 37), (18, 18), '=');
    put(&mut g, 33, 17, 'S');

    // Hook: Grube mit Dornen unter einer Decke, in der Mitte Stein (Hook rutscht ab)
    g.fill((41, 56), (f, 36), '.');
    g.fill((41, 56), (37, 37), '^');
    g.fill((40, 57), (14, 15), '#');
    g.fill((46, 50), (14, 15), '%');
    // Hook-Blüten (R2-M2.1): Hookpunkte mitten in der Luft
    for (x, y) in [(44, 22), (49, 20), (54, 22)] {
        put(&mut g, x, y, '*');
    }
    put(&mut g, 58, f - 1, 'D');

    // Fähigkeiten: Kletterschacht (Halten), Bröckelboden (Stampfen), Gleiten zum Sims
    g.fill((60, 60), (10, f - 1), '|');
    g.fill((64, 64), (18, f - 1), '|');
    g.fill((65, 72), (18, 18), '#');
    g.fill((67, 72), (f, f), ':');
    g.fill((67, 72), (f + 1, 37), '.');
    put(&mut g, 70, 37, 'a');
    g.fill((86, 91), (22, 22), '#');
    put(&mut g, 88, 21, 'S');

    // Sondertiles: Eis, Beschleuniger am Boden und auf einem Steg
    g.fill((76, 83), (f, f), '~');
    g.fill((86, 90), (f, f), '>');
    g.fill((77, 82), (28, 28), '<');

    // Waffen und Gegner: Dummies, Waffen auf einem Steg, Sims für den Pollenbläser
    put(&mut g, 96, f - 1, 'D');
    put(&mut g, 100, f - 1, 'W');
    put(&mut g, 104, f - 1, 'J');
    put(&mut g, 108, f - 1, 'X');
    g.fill((95, 105), (28, 28), '=');
    put(&mut g, 98, 27, 'L');
    put(&mut g, 103, 27, 'G');
    g.fill((110, 116), (26, 26), '#');
    put(&mut g, 117, f - 1, 'h');
    put(&mut g, 116, f - 1, 'S');

    // Rahmen zuletzt (Gelände überschreibt sonst die Decke)
    g.fill((0, w - 1), (0, 0), '#');
    g.fill((0, 0), (0, h - 1), '#');
    g.fill((w - 1, w - 1), (0, h - 1), '#');

    let rows: Vec<String> = g.0.iter().map(|r| r.iter().collect()).collect();
    let r: Vec<&str> = rows.iter().map(String::as_str).collect();
    let mut m = Map::from_rows("Training", &r).expect("Layout gültig");
    // Gegner-Übungsplatz (im Training kehren sie nach einer Weile zurück)
    m.adventure.objects = vec![
        creature("kaefer", "stachelkaefer", 111, f, 26.0),
        creature("huepfer", "grashuepfer", 114, f, 28.0),
        creature("blaeser", "pollenblaeser", 113, 26, 60.0),
        creature("puppe-1", "strohpuppe", 92, f, 40.0),
        creature("puppe-2", "strohpuppe", 94, f, 40.0),
    ];
    m.decor_back = vec![
        decor("werkstatt", 10.0, f),
        decor("fahne-bunt", 18.0, f),
        decor("anschlagbrett", 37.5, f),
        decor("fahne-bunt", 39.0, f),
        decor("fahne-bunt", 59.0, f),
        decor("fahne-bunt", 75.0, f),
        decor("fahne-bunt", 92.5, 34),
        decor("faesser", 118.0, f),
        decor("holzstapel", 2.0, f),
    ];
    m.decor_front = vec![
        decor("heuballen", 48.0, 38),
        decor("bank", 16.5, f),
        decor("beet-bunt", 5.5, f),
        decor("blumentopf-bunt", 13.5, f),
        decor("kuerbisse", 72.0, 38),
        decor("korb", 106.0, 28),
        decor("schmetterling", 8.0, 26),
        decor("schmetterling", 31.0, 15),
        decor("schmetterling", 68.0, 14),
        decor("schmetterling", 102.0, 22),
    ];
    for x in (41..56).step_by(2) {
        m.decor_front.push(decor("dornen", x as f32 + 0.5, 38));
    }
    m.decor_back.extend([
        decor("tree-round", 23.0, 30),
        decor("tree-pine", 73.0, f),
        decor("tree-round", 119.0 - 6.0, f),
    ]);
    let mut map = finish(m, &release::THEMES[0]);
    map.author = Some("Elora-Team".into());
    animate(&mut map, &[]);
    map
}

/// Gegner-Objekte der Karte (für das Training).
pub fn creature_count(map: &Map) -> usize {
    map.adventure
        .objects
        .iter()
        .filter(|o| matches!(o.kind, ObjectKind::Creature { .. }))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_sim::{BeltDir, JumpDir, Tile};

    fn shipped() -> String {
        format!(
            "{}/../../maps/training.{}",
            env!("CARGO_MANIFEST_DIR"),
            elora_map::EXTENSION
        )
    }

    #[test]
    fn training_map_has_every_area() {
        let m = training();
        assert_eq!((m.width, m.height), (120, 40));
        let has = |t: Tile| m.tiles.contains(&t);
        for t in [
            Tile::Climb,
            Tile::Crumble,
            Tile::HookPoint,
            Tile::Death,
            Tile::Ice,
            Tile::Platform,
            Tile::Unhookable,
            Tile::JumpPad(JumpDir::Up),
            Tile::JumpPad(JumpDir::UpRight),
            Tile::Conveyor(BeltDir::Left),
            Tile::Conveyor(BeltDir::Right),
        ] {
            assert!(has(t), "{t:?} fehlt");
        }
        assert!(m.supported_modes().free_for_all);
        assert_eq!(creature_count(&m), 5);
        let back = elora_map::decode(&elora_map::encode(&m)).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn shipped_training_map_is_current() {
        let file = std::fs::read(shipped()).expect("Karte vorhanden");
        assert_eq!(
            elora_map::decode(&file).unwrap(),
            training(),
            "veraltet – write_training_map -- --ignored"
        );
    }

    #[test]
    #[ignore = "schreibt maps/training.emap"]
    fn write_training_map() {
        training().save(std::path::Path::new(&shipped())).unwrap();
    }

    /// Übersicht: `… training_sheet -- --ignored` → `target/training.svg`.
    #[test]
    #[ignore = "erzeugt nur eine Datei zur Sichtprüfung"]
    fn training_sheet() {
        use crate::editor::panel::Preview;
        use crate::editor::view;
        use elora_sim::{TILE_SIZE, Vec2};
        let mut editor = super::super::Editor::new(None, std::path::PathBuf::from("maps"));
        editor.map = training();
        editor.center_view();
        editor.visible.grid = false;
        let size = Vec2::new(editor.map.width as f32, editor.map.height as f32) * TILE_SIZE as f32;
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
        let path = format!("{}/../../target/training.svg", env!("CARGO_MANIFEST_DIR"));
        std::fs::write(path, svg).unwrap();
    }
}

//! Karten von Kapitel 1 (R2-M2.1, E-297): `wiese-2` (Imkerei, Riesenblumen, Bach, erste
//! schwere Hook-Strecke), `wiese-3` (Höhlen unter den Wurzeln) und `wiese-arena`
//! (Blütenquelle mit der Brummbär-Hummel).
//!
//! `cargo test -p elora-client --bin elora write_kapitel1_maps -- --ignored` schreibt
//! `maps/abenteuer/*.emap`.

#![allow(
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::too_many_lines
)]

use elora_map::adventure::CameraMode;
use elora_map::{Map, ObjectKind};
use elora_sim::Vec2;

use super::prolog::{
    Grid, T, animate, at, chest, corner, creature, decor, finish, npc, o, plant, pull_vault,
    ruck_gate, sign,
};
use super::release;

/// Übergang über die ganze Kartenhöhe am Rand (E-279).
fn edge_exit(id: &str, x: usize, h: usize, map: &str, spawn: &str) -> elora_map::Object {
    o(
        id,
        corner(x, 0),
        ObjectKind::Exit {
            size: Vec2::new(2.0 * T, h as f32 * T),
            map: map.into(),
            spawn: spawn.into(),
            on_touch: true,
        },
    )
}

/// Deko in anderer Größe (Riesenblumen, Quelle).
fn big(name: &str, tx: f32, ty: usize, scale: f32) -> elora_map::Decor {
    let mut d = decor(name, tx, ty);
    d.scale = scale;
    d
}

fn bee(id: &str, tx: usize, ty: usize) -> elora_map::Object {
    o(
        id,
        at(tx, ty, 24.0),
        ObjectKind::Collectible {
            item: "biene".into(),
        },
    )
}

/// Dornenranken über einer Dornenreihe (Zeile `y` = Oberkante der Dornen).
fn thorn_decor(m: &mut Map, y: usize, x0: usize, x1: usize) {
    let mut x = x0;
    while x < x1 {
        m.decor_front.push(decor("dornen", x as f32 + 0.5, y + 1));
        x += 2;
    }
}

/// Blütenwiesen 2, 300 × 60: Imkerei mit Wabe, Riesenblumen-Wald mit Blatt-Stegen,
/// Bach mit Trittsteinen, erste schwere Hook-Strecke über einer Dornensenke.
pub fn wiese_2() -> Map {
    let (w, h) = (300, 60);
    let mut g = Grid::new(w, h, 44);
    // Imkerei auf einer kleinen Anhöhe
    g.ground(18, 46, 42);
    // Riesenblumen-Wald: Blätter als Stege, oben eine Blütenkrone (Biene 1)
    for (x, y) in [
        (52, 38),
        (58, 33),
        (64, 28),
        (72, 34),
        (78, 29),
        (84, 24),
        (90, 31),
        (97, 36),
    ] {
        g.fill((x, x + 4), (y, y), '=');
    }
    g.fill((82, 88), (16, 17), '#');
    // Bach in einer Senke mit Trittsteinen (Plattformen)
    g.ground(110, 148, 49);
    g.fill((118, 140), (49, 50), '.');
    g.fill((118, 140), (51, 51), '#');
    for x in [120, 126, 132, 138] {
        g.fill((x, x + 1), (49, 49), '=');
    }
    // erste schwere Hook-Strecke: Dornensenke unter Erdbrocken und Hook-Blüten
    g.ground(158, 214, 56);
    g.fill((158, 214), (55, 55), '^');
    for (x, y) in [(162, 35), (174, 33), (186, 36), (198, 33), (209, 35)] {
        g.fill((x, x + 2), (y, y + 1), '#');
    }
    for x in [168, 180, 192, 204] {
        g.fill((x, x), (39, 39), '*');
    }
    g.ground(215, 230, 44);
    // alter Steinturm am Ende: Ruck-Stelle mit Biene 4 (M2.1.5)
    let ruck_top = ruck_gate(&mut g, 276, 42);
    // Zugtruhe für die Rückkehr mit Heranhooken (M2.2.6)
    let zug = pull_vault(
        &mut g,
        "wiese2",
        244,
        40,
        "wiese2.zug",
        &[("glanztropfen", 50), ("heiltrank", 2)],
    );
    // Hügel, Quellstein und Weg nach wiese-3
    g.ground(231, 250, 40);
    g.ground(251, w - 1, 42);
    let mut m = g.map("Blütenwiesen 2");
    m.adventure.objects = vec![
        edge_exit("weg-wiese-1", 0, h, "wiese-1", "ost"),
        o("west", at(5, 44, 28.0), ObjectKind::Spawn),
        o("rast", at(12, 44, 40.0), ObjectKind::SavePoint),
        npc("wabe", 30, 42, -1, 0.0),
        bee("biene-1", 85, 16),
        plant("blume-1", 49, 44),
        creature("kaefer-1", "stachelkaefer", 60, 44, 26.0),
        creature("blaeser-1", "pollenblaeser", 75, 44, 60.0),
        creature("huepfer-1", "grashuepfer", 94, 44, 28.0),
        chest(
            "truhe-krone",
            87,
            16,
            &[("glanztropfen", 25), ("bernstein", 1)],
        ),
        creature("kaefer-2", "stachelkaefer", 112, 49, 26.0),
        creature("kaefer-3", "stachelkaefer", 145, 49, 26.0),
        plant("blume-2", 152, 44),
        creature("blaeser-2", "pollenblaeser", 220, 44, 60.0),
        bee("biene-4", 285, ruck_top),
        plant("blume-3", 228, 44),
        creature("huepfer-2", "grashuepfer", 236, 40, 28.0),
        creature("kaefer-4", "stachelkaefer", 262, 42, 26.0),
        o("quellstein", at(266, 42, 40.0), ObjectKind::SavePoint),
        o("ost", at(292, 42, 28.0), ObjectKind::Spawn),
        edge_exit("weg-wiese-3", w - 2, h, "wiese-3", "west"),
    ];
    m.adventure.objects.extend(zug);
    m.decor_back = vec![
        decor("honigstand", 24.0, 42),
        decor("bienenstock", 34.0, 42),
        decor("bienenstock", 37.5, 42),
        decor("beutenstapel", 41.5, 42),
        decor("fence", 45.0, 42),
        big("riesenblume-rosa", 54.0, 44, 2.0),
        big("riesenblume-gelb", 66.0, 44, 2.0),
        big("riesenblume-lila", 80.0, 44, 2.0),
        big("riesenblume-rosa", 92.0, 44, 2.0),
        big("riesenblume-gelb", 101.0, 44, 2.0),
        decor("tree-round", 108.0, 44),
        decor("tree-round", 152.0, 44),
        big("riesenblume-lila", 222.0, 44, 1.6),
        decor("tree-round", 246.0, 40),
        big("riesenblume-rosa", 272.0, 42, 1.4),
    ];
    m.decor_front = vec![
        decor("blumentopf-bunt", 20.0, 42),
        decor("korb", 27.0, 42),
        decor("trittsteine", 129.0, 51),
        decor("farn", 116.0, 49),
        decor("farn", 144.0, 49),
        decor("loewenzahn", 236.0, 40),
        decor("beerenbusch", 256.0, 42),
        decor("farn", 292.0, 42),
    ];
    thorn_decor(&mut m, 55, 158, 214);
    for (tx, ty) in [
        (28, 36),
        (62, 24),
        (88, 12),
        (130, 44),
        (190, 26),
        (240, 34),
        (276, 36),
    ] {
        m.decor_front.push(decor("schmetterling", tx as f32, ty));
    }
    let mut map = finish(m, &release::THEMES[0]);
    animate(&mut map, &[]);
    map
}

/// Blütenwiesen 3, 220 × 70: hinunter in die Höhlen unter den Wurzeln, Dornen und
/// Bröckelboden, versteckte Nische (Biene 2), Aufstieg an Wurzeldecken zur Blütenquelle.
pub fn wiese_3() -> Map {
    let (w, h) = (220, 70);
    let mut g = Grid::new(w, h, 30);
    // Höhle unter den Wurzeln: 10 Tiles Erde darüber, Gang von Zeile 41 bis 60
    g.fill((31, 175), (41, 60), '.');
    // Einstieg: Schacht hinunter
    g.fill((27, 30), (30, 60), '.');
    // Höhlenboden mit Stufen, Dornengruben und Bröckelboden
    g.fill((31, 60), (57, 60), '#');
    g.fill((61, 72), (60, 60), '^');
    g.fill((73, 100), (55, 60), '#');
    g.fill((101, 108), (55, 55), ':');
    g.fill((101, 108), (56, 59), '.');
    g.fill((101, 108), (60, 60), '^');
    g.fill((109, 140), (57, 60), '#');
    g.fill((141, 156), (60, 60), '^');
    g.fill((157, 175), (52, 60), '#');
    // hängende Wurzeln (Hook-Punkte an der Decke über den Gruben)
    for x in [64, 69, 144, 149, 154] {
        g.fill((x, x), (41, 43), '#');
    }
    // versteckte Nische in der Decke (Biene 2): nur über eine Wurzel zu erreichen
    g.fill((118, 126), (37, 40), '.');
    g.fill((121, 121), (41, 42), '#');
    // Aufstieg: Kammer nach oben mit Stegen und einer Wurzel
    g.fill((168, 175), (24, 51), '.');
    for (x, y) in [(168, 46), (172, 40), (168, 34), (172, 29)] {
        g.fill((x, x + 3), (y, y), '=');
    }
    // oben hinaus zur Arena
    g.ground(176, 199, 26);
    g.ground(200, w - 1, 24);
    let mut m = g.map("Blütenwiesen 3");
    m.adventure.objects = vec![
        edge_exit("weg-wiese-2", 0, h, "wiese-2", "ost"),
        o("west", at(5, 30, 28.0), ObjectKind::Spawn),
        sign("schild-wurzeln", 22, 30),
        creature("kaefer-1", "stachelkaefer", 45, 57, 26.0),
        plant("blume-1", 36, 57),
        creature("huepfer-1", "grashuepfer", 85, 55, 28.0),
        creature("blaeser-1", "pollenblaeser", 95, 55, 60.0),
        o("hoehle", at(112, 57, 40.0), ObjectKind::SavePoint),
        bee("biene-2", 120, 41),
        chest(
            "truhe-nische",
            124,
            41,
            &[("glanztropfen", 30), ("heiltrank", 1)],
        ),
        creature("kaefer-2", "stachelkaefer", 132, 57, 26.0),
        bee("biene-3", 160, 52),
        creature("huepfer-2", "grashuepfer", 168, 52, 28.0),
        plant("blume-2", 162, 52),
        creature("kaefer-3", "stachelkaefer", 205, 24, 26.0),
        o("ost", at(214, 24, 28.0), ObjectKind::Spawn),
        edge_exit("weg-arena", w - 2, h, "wiese-arena", "west"),
    ];
    m.decor_back = vec![
        decor("tree-round", 8.0, 30),
        big("riesenblume-lila", 16.0, 30, 1.6),
        decor("tree-round", 60.0, 30),
        decor("tree-round", 110.0, 30),
        decor("tree-pine", 150.0, 30),
        decor("tree-pine", 190.0, 26),
        big("riesenblume-gelb", 208.0, 24, 1.6),
    ];
    m.decor_front = vec![
        decor("farn", 33.0, 57),
        decor("baumstumpf", 79.0, 55),
        decor("farn", 112.0, 57),
        decor("beerenbusch", 128.0, 57),
        decor("farn", 165.0, 52),
        decor("loewenzahn", 203.0, 24),
        decor("farn", 40.0, 30),
        decor("beerenbusch", 90.0, 30),
        decor("loewenzahn", 130.0, 30),
    ];
    thorn_decor(&mut m, 60, 61, 72);
    thorn_decor(&mut m, 60, 101, 108);
    thorn_decor(&mut m, 60, 141, 156);
    let mut map = finish(m, &release::THEMES[0]);
    animate(&mut map, &[]);
    map
}

/// Die Blütenquelle, 90 × 50: vom Sims mit dem Quellstein hinunter in die Arena,
/// Hook-Blüten über dem Boden, Ausgang öffnet sich nach dem Sieg (E-254).
pub fn wiese_arena() -> Map {
    let (w, h) = (90, 50);
    let mut g = Grid::new(w, h, 44);
    // Einstiegs-Sims links (zu hoch, um zurückzuspringen)
    g.ground(0, 17, 26);
    g.fill((18, 19), (26, 43), '%');
    // rechte Wand aus Stein (nicht hookbar), unten das Tor
    g.fill((70, 71), (12, 35), '%');
    // Hook-Blüten in zwei Reihen (11 und 17 Tiles über dem Boden)
    for x in (24..68).step_by(5) {
        g.fill((x, x), (33, 33), '*');
    }
    for x in (27..66).step_by(10) {
        g.fill((x, x), (27, 27), '*');
    }
    // dahinter der Wurzelpfad zurück
    g.ground(72, w - 1, 44);
    let mut m = g.map("Blütenquelle");
    m.adventure.objects = vec![
        o("west", at(5, 26, 28.0), ObjectKind::Spawn),
        o("vor-der-quelle", at(11, 26, 40.0), ObjectKind::SavePoint),
        o(
            "brummbaer",
            at(45, 30, 120.0),
            ObjectKind::Creature {
                kind: "brummbaer".into(),
                persistent: true,
            },
        ),
        npc("hummel", 52, 44, -1, 0.0),
        o(
            "tor",
            corner(70, 36),
            ObjectKind::Door {
                size: (2, 8),
                open_if: "merker besiegt.brummbaer".into(),
            },
        ),
        o(
            "arena",
            corner(20, 14),
            ObjectKind::Camera {
                size: Vec2::new(50.0 * T, 30.0 * T),
                mode: CameraMode::Bounds,
            },
        ),
        o("ost", at(84, 44, 28.0), ObjectKind::Spawn),
        edge_exit("weg-heim", w - 2, h, "wiese-1", "ost"),
    ];
    m.decor_back = vec![
        big("bluetenquelle-verdorrt", 45.0, 44, 1.5),
        big("riesenblume-rosa", 4.0, 26, 1.4),
        big("riesenblume-lila", 13.5, 26, 1.6),
        big("riesenblume-gelb", 21.5, 44, 1.5),
        big("riesenblume-rosa", 67.0, 44, 1.5),
        decor("tree-round", 80.0, 44),
    ];
    m.decor_front = vec![decor("farn", 30.0, 44), decor("farn", 60.0, 44)];
    let mut map = finish(m, &release::THEMES[0]);
    animate(&mut map, &[]);
    map
}

/// Alle Abenteuer-Karten (Prolog und Kapitel 1) mit ihren Namen.
pub fn all_maps() -> Vec<(&'static str, Map)> {
    vec![
        ("tauwinkel", super::prolog::tauwinkel()),
        ("wiese-1", super::prolog::wiese()),
        ("wiese-2", wiese_2()),
        ("wiese-3", wiese_3()),
        ("wiese-arena", wiese_arena()),
    ]
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
    fn adventure_maps_match_content_and_link_up() {
        let c = Content::builtin();
        let maps = all_maps();
        for (name, m) in &maps {
            let back = elora_map::decode(&elora_map::encode(m)).expect("Karte gültig");
            let errors = map_objects(&c, &back);
            assert!(errors.is_empty(), "{name}: {errors:?}");
        }
        // Übergänge über alle Abenteuer-Karten (auch Kapitel 2)
        let mut linked = all_maps();
        linked.extend(super::super::kapitel2::maps());
        linked.extend(super::super::kapitel3::maps());
        let refs: Vec<(&str, &Map)> = linked.iter().map(|(n, m)| (*n, m)).collect();
        let errors = map_links(&refs);
        assert!(errors.is_empty(), "{errors:?}");
        // fünf Bienen, davon vier in Kapitel 1 und eine für die Rückkehr (M2.1.5)
        let bees: usize = maps
            .iter()
            .map(|(_, m)| {
                m.adventure
                    .objects
                    .iter()
                    .filter(
                        |o| matches!(&o.kind, ObjectKind::Collectible { item } if item == "biene"),
                    )
                    .count()
            })
            .sum();
        assert!(bees >= 4, "{bees} Bienen");
    }

    #[test]
    fn shipped_chapter_maps_are_current() {
        for (name, map) in [
            ("wiese-2", wiese_2()),
            ("wiese-3", wiese_3()),
            ("wiese-arena", wiese_arena()),
        ] {
            let file = std::fs::read(shipped(name)).expect("Karte vorhanden");
            assert_eq!(
                elora_map::decode(&file).expect("gültig"),
                map,
                "{name} veraltet – write_kapitel1_maps -- --ignored"
            );
        }
    }

    #[test]
    #[ignore = "schreibt maps/abenteuer/*.emap"]
    fn write_kapitel1_maps() {
        for (name, map) in all_maps() {
            map.save(std::path::Path::new(&shipped(name))).unwrap();
        }
    }

    /// Übersicht: `… kapitel1_sheets -- --ignored` → `target/kapitel1-<karte>.svg`.
    #[test]
    #[ignore = "erzeugt nur Dateien zur Sichtprüfung"]
    fn kapitel1_sheets() {
        use crate::editor::panel::Preview;
        use crate::editor::view;
        use elora_sim::TILE_SIZE;
        for (name, map) in all_maps().into_iter().skip(2) {
            let mut editor = super::super::Editor::new(None, std::path::PathBuf::from("maps"));
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
                "{}/../../target/kapitel1-{name}.svg",
                env!("CARGO_MANIFEST_DIR")
            );
            std::fs::write(path, svg).unwrap();
        }
    }
}

//! Karten von Kapitel 2 (R2-M2.2): `wald-1` (Waldrand mit Pilzring), `wald-2` (Baumhaus-
//! Siedlung in den Kronen), `wald-3` (Wurzelhöhlen) und `wald-arena` (Waldquelle mit dem
//! Wurzelwächter). Der Wald liegt westlich von Tauwinkel: die Karten werden von links (Dorf)
//! nach rechts (tiefer im Wald) gebaut und dann gespiegelt.
//!
//! `cargo test -p elora-client --bin elora write_kapitel2_maps -- --ignored` schreibt
//! `maps/abenteuer/*.emap`.

#![allow(
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::too_many_lines
)]

use elora_map::adventure::CameraMode;
use elora_map::{Decor, Map, Object, ObjectKind};
use elora_sim::{TILE_SIZE, Vec2};

use super::prolog::{
    Grid, T, animate, at, chest, corner, creature, decor, finish, npc, o, plant, ruck_gate, sign,
    stomp_vault,
};
use super::release;

/// Übergang über die ganze Kartenhöhe am Rand (E-279).
fn edge_exit(id: &str, x: usize, h: usize, map: &str, spawn: &str) -> Object {
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

fn rune(id: &str, tx: usize, ty: usize) -> Object {
    o(
        id,
        at(tx, ty, 24.0),
        ObjectKind::Collectible {
            item: "rune".into(),
        },
    )
}

fn big(name: &str, tx: f32, ty: usize, scale: f32) -> Decor {
    let mut d = decor(name, tx, ty);
    d.scale = scale;
    d
}

fn thorn_decor(m: &mut Map, y: usize, x0: usize, x1: usize) {
    let mut x = x0;
    while x < x1 {
        m.decor_front.push(decor("dornen", x as f32 + 0.5, y + 1));
        x += 2;
    }
}

/// Waagerecht spiegeln: Tiles, Objekte (Bereiche an ihrer linken Kante) und Deko.
fn mirror(m: &mut Map) {
    let (w, h) = (m.width, m.height);
    for y in 0..h {
        m.tiles[y * w..(y + 1) * w].reverse();
    }
    let full = w as f32 * T;
    for ob in &mut m.adventure.objects {
        let width = ob.kind.area().map_or(0.0, |a| a.x);
        ob.pos.x = full - ob.pos.x - width;
        if let ObjectKind::Npc { facing, .. } = &mut ob.kind {
            *facing = -*facing;
        }
    }
    for d in m.decor_back.iter_mut().chain(m.decor_front.iter_mut()) {
        d.pos.x = full - d.pos.x;
        d.flip_x = !d.flip_x;
    }
}

/// Wald-Look: Thema „Wald“ der Release-Karten, etwas schattigerer Himmel.
fn forest(mut m: Map) -> Map {
    mirror(&mut m);
    let mut map = finish(m, &release::THEMES[3]);
    map.sky = elora_map::Sky {
        top: elora_map::Rgba::hex(0x8fb7a8),
        bottom: elora_map::Rgba::hex(0xd8e8d4),
    };
    animate(&mut map, &[]);
    map
}

/// Murmelwald 1, 220 × 60: Waldrand mit dem Pilzring nahe Tauwinkel, sanfte Hügel (der Weg
/// am Boden hat keine Lücken – das Pilzkind kommt mit), Kronen-Stege als oberer Weg,
/// Steinsäule mit einer Rune (erst mit Heranhooken).
pub fn wald_1() -> Map {
    let (w, h, f) = (220, 60, 44);
    let mut g = Grid::new(w, h, f);
    g.ground(34, 45, 42);
    g.ground(46, 62, 40);
    g.ground(63, 90, 42);
    g.ground(91, 94, 44);
    g.ground(95, 120, 46);
    g.ground(121, 124, 44);
    g.ground(125, 150, 42);
    g.ground(151, 175, 40);
    g.ground(176, 200, 38);
    g.ground(201, w - 1, 40);
    // oberer Weg durch die Kronen
    g.fill((66, 74), (32, 32), '=');
    g.fill((78, 86), (27, 27), '=');
    g.fill((88, 92), (20, 21), '#');
    g.fill((100, 108), (34, 34), '=');
    g.fill((110, 116), (36, 36), '=');
    // schwebende Steinsäule: Rune oben, nur mit Heranhooken (M2.2.2); unten frei für den Weg
    g.fill((186, 188), (20, 30), '%');
    // Stampfkammer für die Rückkehr mit Stampfen (M2.3.6)
    let stampf = stomp_vault(
        &mut g,
        "wald1-stampf",
        140,
        42,
        &[("glanztropfen", 60), ("harz", 2)],
    );
    let mut m = g.map("Murmelwald 1");
    m.adventure.objects = vec![
        edge_exit("weg-dorf", 0, h, "tauwinkel", "west"),
        o("ost", at(6, 44, 28.0), ObjectKind::Spawn),
        sign("schild-wald", 11, 44),
        o(
            "pilzring",
            corner(17, 34),
            ObjectKind::Zone {
                size: Vec2::new(14.0 * T, 10.0 * T),
            },
        ),
        npc("pilzmama", 22, 44, 1, 0.0),
        npc("pilzkind_froh", 26, 44, -1, 0.0),
        sign("schild-pilzring", 32, 44),
        creature("schlange-1", "wurzelschlange", 52, 40, 110.0),
        creature("wicht-1", "pilzwicht", 72, 42, 40.0),
        rune("rune-1", 90, 20),
        creature("wicht-2", "pilzwicht", 86, 42, 40.0),
        creature("schlange-2", "wurzelschlange", 104, 46, 110.0),
        chest(
            "truhe-munition",
            100,
            46,
            &[("munition_granate", 1), ("glanztropfen", 10)],
        ),
        creature("pirat-1", "eichhornpirat", 113, 36, 52.0),
        plant("blume-1", 118, 46),
        o("quellstein", at(130, 42, 40.0), ObjectKind::SavePoint),
        creature("wicht-3", "pilzwicht", 160, 40, 40.0),
        creature("schlange-3", "wurzelschlange", 172, 40, 110.0),
        rune("rune-2", 187, 20),
        plant("blume-2", 196, 38),
        o("west", at(214, 40, 28.0), ObjectKind::Spawn),
        edge_exit("weg-wald-2", w - 2, h, "wald-2", "ost"),
        stampf,
    ];
    m.decor_back = vec![
        big("waldbaum", 4.0, 44, 0.8),
        big("pilzring", 24.0, 44, 1.0),
        big("waldbaum", 40.0, 42, 0.9),
        big("waldbaum", 70.0, 42, 1.4),
        big("waldbaum", 84.0, 42, 1.5),
        big("waldbaum", 112.0, 46, 1.3),
        big("wurzelbogen", 140.0, 42, 1.0),
        big("waldbaum", 165.0, 40, 1.1),
        big("wurzelbogen", 196.0, 38, 0.9),
        big("waldbaum", 208.0, 40, 1.0),
    ];
    m.decor_front = vec![
        decor("leuchtpilze", 15.0, 44),
        decor("farn", 36.0, 42),
        decor("leuchtpilze", 60.0, 40),
        decor("farn", 98.0, 46),
        decor("leuchtpilze", 122.0, 44),
        decor("farn", 152.0, 40),
        decor("baumstumpf", 180.0, 38),
        decor("leuchtpilze", 204.0, 40),
    ];
    forest(m)
}

/// Murmelwald 2, 200 × 90: hohe Bäume, Stege und Hängebrücken in den Kronen, Baumhäuser,
/// Plumm ganz oben; das Pilzkind am Boden bei den Leuchtpilzen.
pub fn wald_2() -> Map {
    let (w, h, f) = (200, 90, 80);
    let mut g = Grid::new(w, h, f);
    g.ground(30, 59, 78);
    g.ground(60, 140, 76);
    g.ground(141, 170, 78);
    // Aufstieg: Stege im Zickzack (je 4 Tiles höher)
    for (k, y) in (40..=72).rev().step_by(4).enumerate() {
        let x = if k % 2 == 0 { 62 } else { 69 };
        g.fill((x, x + 5), (y, y), '=');
    }
    // Hängebrücke und Plattformen in den Kronen
    g.fill((76, 118), (44, 44), '=');
    g.fill((120, 132), (40, 40), '=');
    g.fill((134, 146), (36, 36), '=');
    g.fill((150, 156), (28, 29), '#');
    // zweiter Abstieg auf der anderen Seite
    for (k, y) in (40..=72).step_by(4).enumerate() {
        let x = if k % 2 == 0 { 160 } else { 167 };
        g.fill((x, x + 5), (y, y), '=');
    }
    let mut m = g.map("Murmelwald 2");
    m.adventure.objects = vec![
        edge_exit("weg-wald-1", 0, h, "wald-1", "west"),
        o("ost", at(6, 80, 28.0), ObjectKind::Spawn),
        npc("pilzkind", 40, 78, 1, 0.0),
        chest(
            "truhe-munition",
            50,
            78,
            &[("munition_granate", 1), ("glanztropfen", 10)],
        ),
        creature("wicht-1", "pilzwicht", 52, 78, 40.0),
        creature("schlange-1", "wurzelschlange", 90, 76, 110.0),
        o("baum-rast", at(100, 76, 40.0), ObjectKind::SavePoint),
        creature("pirat-1", "eichhornpirat", 95, 44, 52.0),
        npc("plumm", 126, 40, -1, 0.0),
        creature("pirat-2", "eichhornpirat", 140, 36, 52.0),
        rune("rune-3", 153, 28),
        chest("truhe-krone", 155, 28, &[("glanztropfen", 35), ("harz", 2)]),
        creature("schlange-2", "wurzelschlange", 150, 78, 110.0),
        plant("blume-1", 120, 76),
        creature("wicht-2", "pilzwicht", 180, 80, 40.0),
        o("west", at(194, 80, 28.0), ObjectKind::Spawn),
        edge_exit("weg-wald-3", w - 2, h, "wald-3", "ost"),
    ];
    m.decor_back = vec![
        big("waldbaum", 20.0, 80, 1.3),
        big("waldbaum", 60.0, 78, 2.1),
        big("waldbaum", 75.0, 76, 2.2),
        big("waldhaus", 96.0, 44, 1.0),
        big("haengebruecke", 106.0, 44, 1.0),
        big("waldbaum", 112.0, 76, 2.2),
        big("waldhaus", 126.0, 40, 1.1),
        big("waldbaum", 140.0, 76, 2.2),
        big("waldbaum", 158.0, 78, 2.1),
        big("waldbaum", 185.0, 80, 1.3),
    ];
    m.decor_front = vec![
        decor("leuchtpilze", 36.0, 78),
        decor("leuchtpilze", 44.0, 78),
        decor("farn", 64.0, 76),
        decor("farn", 104.0, 76),
        decor("leuchtpilze", 130.0, 76),
        decor("baumstumpf", 175.0, 80),
    ];
    forest(m)
}

/// Murmelwald 3, 200 × 70: hinunter in die Wurzelhöhlen, Dornen, Bröckelboden, hängende
/// Wurzeln; Zugschalter-Kammer mit einer Rune (erst mit Heranhooken), Ruck-Stelle mit Rune,
/// am Ende hinauf zur Waldquelle.
pub fn wald_3() -> Map {
    let (w, h) = (200, 70);
    let mut g = Grid::new(w, h, 30);
    // Höhle unter den Wurzeln: Gang von Zeile 40 bis 60
    g.fill((24, 160), (40, 60), '.');
    g.fill((20, 23), (30, 60), '.');
    g.fill((24, 50), (57, 60), '#');
    g.fill((51, 62), (60, 60), '^');
    g.fill((63, 90), (55, 60), '#');
    g.fill((91, 98), (55, 55), ':');
    g.fill((91, 98), (56, 59), '.');
    g.fill((91, 98), (60, 60), '^');
    g.fill((99, 130), (57, 60), '#');
    g.fill((131, 144), (60, 60), '^');
    g.fill((145, 160), (52, 60), '#');
    for x in [54, 59, 134, 139, 143] {
        g.fill((x, x), (40, 42), '#');
    }
    // Wurzel mit dem Zugschalter (vom Höhlenboden mit dem Hook erreichbar)
    g.fill((122, 123), (40, 46), '#');
    // Zugschalter-Kammer in der Decke: Steg als Boden, Tor darunter (`merker zug.wald3`)
    g.fill((106, 116), (34, 37), '.');
    g.fill((106, 116), (38, 38), '=');
    g.fill((106, 116), (39, 39), '.');
    // Aufstieg am Ende
    g.fill((154, 160), (24, 51), '.');
    for (x, y) in [(154, 46), (157, 40), (154, 34), (157, 28)] {
        g.fill((x, x + 3), (y, y), '=');
    }
    g.ground(161, w - 1, 26);
    // Ruck-Stelle oben mit Rune 4
    let ruck_top = ruck_gate(&mut g, 170, 26);
    // Stampfkammer im Höhlenboden (M2.3.6)
    let stampf = stomp_vault(
        &mut g,
        "wald3-stampf",
        26,
        57,
        &[("glanztropfen", 70), ("tautrank", 1)],
    );
    let mut m = g.map("Murmelwald 3");
    m.adventure.objects = vec![
        edge_exit("weg-wald-2", 0, h, "wald-2", "west"),
        o("ost", at(5, 30, 28.0), ObjectKind::Spawn),
        creature("schlange-1", "wurzelschlange", 36, 57, 110.0),
        creature("wicht-1", "pilzwicht", 44, 57, 40.0),
        creature("pirat-1", "eichhornpirat", 75, 55, 52.0),
        o("hoehle", at(102, 57, 40.0), ObjectKind::SavePoint),
        chest(
            "truhe-munition",
            108,
            57,
            &[("munition_granate", 1), ("glanztropfen", 10)],
        ),
        o(
            "zug",
            Vec2::new(123.0 * T, 47.0 * T + 14.0),
            ObjectKind::Switch {
                flag: "zug.wald3".into(),
                once: true,
                trigger: elora_map::adventure::SwitchTrigger::Hook,
            },
        ),
        o(
            "tor-kammer",
            corner(106, 39),
            ObjectKind::Door {
                size: (11, 1),
                open_if: "merker zug.wald3".into(),
            },
        ),
        rune("rune-5", 109, 38),
        chest(
            "truhe-kammer",
            114,
            38,
            &[("glanztropfen", 40), ("tautrank", 1)],
        ),
        creature("schlange-2", "wurzelschlange", 120, 57, 110.0),
        creature("wicht-2", "pilzwicht", 150, 52, 40.0),
        plant("blume-1", 148, 52),
        rune("rune-4", 179, ruck_top),
        o("west", at(194, 26, 28.0), ObjectKind::Spawn),
        edge_exit("weg-arena", w - 2, h, "wald-arena", "ost"),
        stampf,
    ];
    m.decor_back = vec![
        big("waldbaum", 8.0, 30, 1.0),
        big("waldbaum", 40.0, 30, 1.2),
        big("waldbaum", 90.0, 30, 1.1),
        big("waldbaum", 140.0, 30, 1.2),
        big("waldbaum", 190.0, 26, 1.1),
    ];
    m.decor_front = vec![
        decor("leuchtpilze", 30.0, 57),
        decor("leuchtpilze", 70.0, 55),
        decor("leuchtpilze", 104.0, 57),
        decor("leuchtpilze", 126.0, 57),
        decor("leuchtpilze", 150.0, 52),
        decor("farn", 166.0, 26),
    ];
    thorn_decor(&mut m, 60, 51, 62);
    thorn_decor(&mut m, 60, 91, 98);
    thorn_decor(&mut m, 60, 131, 144);
    forest(m)
}

/// Waldquelle, 100 × 50: vom Sims hinunter in die Arena, Hook-Blüten zum Ausweichen vor
/// den Wurzelwänden, Tor nach dem Sieg, Wurzelpfad zurück nach Tauwinkel.
pub fn wald_arena() -> Map {
    let (w, h) = (100, 50);
    let mut g = Grid::new(w, h, 44);
    g.ground(0, 17, 26);
    g.fill((18, 19), (26, 43), '%');
    g.fill((80, 81), (12, 35), '%');
    for x in (24..78).step_by(6) {
        g.fill((x, x), (34, 34), '*');
    }
    g.ground(82, w - 1, 44);
    let mut m = g.map("Waldquelle");
    m.adventure.objects = vec![
        o("ost", at(5, 26, 28.0), ObjectKind::Spawn),
        o("vor-der-quelle", at(11, 26, 40.0), ObjectKind::SavePoint),
        o(
            "wurzelwaechter",
            at(56, 44, 280.0),
            ObjectKind::Creature {
                kind: "wurzelwaechter".into(),
                persistent: true,
            },
        ),
        npc("waechter", 56, 44, -1, 0.0),
        o(
            "tor",
            corner(80, 36),
            ObjectKind::Door {
                size: (2, 8),
                open_if: "merker besiegt.wurzelwaechter".into(),
            },
        ),
        o(
            "arena",
            corner(20, 14),
            ObjectKind::Camera {
                size: Vec2::new(60.0 * T, 30.0 * T),
                mode: CameraMode::Bounds,
            },
        ),
        o("west", at(94, 44, 28.0), ObjectKind::Spawn),
        edge_exit("weg-heim", w - 2, h, "tauwinkel", "west"),
    ];
    m.decor_back = vec![
        big("waldquelle-verdorrt", 56.0, 44, 1.5),
        big("waldbaum", 4.0, 26, 1.0),
        big("waldbaum", 14.0, 26, 1.1),
        big("waldbaum", 22.0, 44, 1.5),
        big("waldbaum", 77.0, 44, 1.5),
        big("waldbaum", 90.0, 44, 1.2),
    ];
    m.decor_front = vec![
        decor("leuchtpilze", 30.0, 44),
        decor("leuchtpilze", 70.0, 44),
        decor("farn", 40.0, 44),
    ];
    forest(m)
}

/// Karten von Kapitel 2 mit ihren Namen.
pub fn maps() -> Vec<(&'static str, Map)> {
    vec![
        ("wald-1", wald_1()),
        ("wald-2", wald_2()),
        ("wald-3", wald_3()),
        ("wald-arena", wald_arena()),
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
    fn forest_maps_match_content_and_link_up() {
        let c = Content::builtin();
        let mut all = super::super::kapitel1::all_maps();
        all.extend(maps());
        all.extend(super::super::kapitel3::maps());
        all.extend(super::super::kapitel4::maps());
        for (name, m) in &all {
            let back = elora_map::decode(&elora_map::encode(m)).expect("Karte gültig");
            let errors = map_objects(&c, &back);
            assert!(errors.is_empty(), "{name}: {errors:?}");
        }
        let refs: Vec<(&str, &Map)> = all.iter().map(|(n, m)| (*n, m)).collect();
        let errors = map_links(&refs);
        assert!(errors.is_empty(), "{errors:?}");
        let runes: usize = maps()
            .iter()
            .map(|(_, m)| {
                m.adventure
                    .objects
                    .iter()
                    .filter(
                        |o| matches!(&o.kind, ObjectKind::Collectible { item } if item == "rune"),
                    )
                    .count()
            })
            .sum();
        assert_eq!(runes, 5);
    }

    #[test]
    fn shipped_forest_maps_are_current() {
        for (name, map) in maps() {
            let file = std::fs::read(shipped(name)).expect("Karte vorhanden");
            assert_eq!(
                elora_map::decode(&file).expect("gültig"),
                map,
                "{name} veraltet – write_kapitel2_maps -- --ignored"
            );
        }
    }

    #[test]
    #[ignore = "schreibt maps/abenteuer/*.emap"]
    fn write_kapitel2_maps() {
        for (name, map) in maps() {
            map.save(std::path::Path::new(&shipped(name))).unwrap();
        }
    }

    /// Übersicht: `… kapitel2_sheets -- --ignored` → `target/kapitel2-<karte>.svg`.
    #[test]
    #[ignore = "erzeugt nur Dateien zur Sichtprüfung"]
    fn kapitel2_sheets() {
        use crate::editor::panel::Preview;
        use crate::editor::view;
        for (name, map) in maps() {
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
                "{}/../../target/kapitel2-{name}.svg",
                env!("CARGO_MANIFEST_DIR")
            );
            std::fs::write(path, svg).unwrap();
        }
    }
}

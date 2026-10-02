//! Test-Karten des Abenteuers (A1.6): `maps/abenteuer/tauwinkel.emap` und `wiese-1.emap`.
//! Schlichte Testfassungen, bis die richtigen Karten mit A1.9 entstehen.
//!
//! Neu schreiben: `cargo test -p elora-adventure --test maps -- --ignored`

#![allow(clippy::many_single_char_names)]

use elora_adventure::Content;
use elora_adventure::check::{map_links, map_objects};
use elora_map::adventure::{CameraMode, SwitchTrigger};
use elora_map::{Map, Object, ObjectKind};
use elora_sim::Vec2;

const T: f32 = 32.0;

fn grid(w: usize, h: usize, floor: usize) -> Vec<Vec<char>> {
    let mut g = vec![vec!['.'; w]; h];
    for (y, row) in g.iter_mut().enumerate() {
        for (x, c) in row.iter_mut().enumerate() {
            if y == 0 || y >= floor || x == 0 || x == w - 1 {
                *c = '#';
            }
        }
    }
    g
}

/// Zeilen für `Map::from_rows`; das `S` dort braucht nur das Textformat, die Karte
/// nutzt Eingänge (Entities werden danach entfernt).
fn rows(g: &[Vec<char>]) -> Vec<String> {
    let mut g = g.to_vec();
    g[1][1] = 'S';
    g.iter().map(|r| r.iter().collect()).collect()
}

/// Mitte über dem Boden in Spalte `tx` für ein Objekt der Höhe `h`.
#[allow(clippy::cast_precision_loss)]
fn at(tx: usize, floor: usize, h: f32) -> Vec2 {
    Vec2::new(tx as f32 * T + T / 2.0, floor as f32 * T - h / 2.0 - 1.0)
}

#[allow(clippy::cast_precision_loss)]
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

fn tauwinkel() -> Map {
    let (w, h, floor) = (70, 24, 20);
    let mut g = grid(w, h, floor);
    // Tor: Durchgang in Spalte 46, darüber eine Decke, damit man nicht drüberspringt
    for row in g.iter_mut().take(17).skip(1) {
        row[46] = '%';
    }
    let refs = rows(&g);
    let r: Vec<&str> = refs.iter().map(String::as_str).collect();
    let mut m = Map::from_rows("Tauwinkel (Test)", &r).expect("gültig");
    m.entities.clear();
    m.adventure.objects = vec![
        o("start", at(5, floor, 28.0), ObjectKind::Spawn),
        o("ost", at(62, floor, 28.0), ObjectKind::Spawn),
        o(
            "oma",
            at(12, floor, 28.0),
            ObjectKind::Npc {
                character: "oma".into(),
                dialog: "oma".into(),
                facing: 1,
                walk: 0.0,
            },
        ),
        o("brunnen", at(18, floor, 40.0), ObjectKind::SavePoint),
        o(
            "tueftel",
            at(24, floor, 28.0),
            ObjectKind::Npc {
                character: "tueftel".into(),
                dialog: "tueftel".into(),
                facing: -1,
                walk: 24.0,
            },
        ),
        o(
            "truhe-1",
            at(34, floor, 26.0),
            ObjectKind::Chest {
                contents: vec![("glanztropfen".into(), 20), ("heiltrank".into(), 1)],
                lock: String::new(),
            },
        ),
        o(
            "hebel",
            at(42, floor, 30.0),
            ObjectKind::Switch {
                flag: "tor.dorf".into(),
                once: false,
                trigger: SwitchTrigger::Interact,
            },
        ),
        o(
            "tor",
            corner(46, 17),
            ObjectKind::Door {
                size: (1, 3),
                open_if: "merker tor.dorf".into(),
            },
        ),
        o(
            "blume",
            at(52, floor, 16.0),
            ObjectKind::HealPlant { heal: 2 },
        ),
        o(
            "weg-wiese",
            corner(66, 14),
            ObjectKind::Exit {
                size: Vec2::new(3.0 * T, 6.0 * T),
                map: "wiese-1".into(),
                spawn: "west".into(),
                on_touch: true,
            },
        ),
    ];
    m
}

fn wiese() -> Map {
    let (w, h, floor) = (90, 26, 22);
    let mut g = grid(w, h, floor);
    // Brücke über eine Senke
    for row in g.iter_mut().take(floor + 2).skip(floor) {
        for c in &mut row[26..34] {
            *c = '.';
        }
    }
    for c in &mut g[floor][26..34] {
        *c = '=';
    }
    // Sims mit Glitzerstein und Bröckelboden darunter
    for c in &mut g[16][60..68] {
        *c = '#';
    }
    for c in &mut g[floor][70..74] {
        *c = ':';
    }
    let refs = rows(&g);
    let r: Vec<&str> = refs.iter().map(String::as_str).collect();
    let mut m = Map::from_rows("Blütenwiesen 1 (Test)", &r).expect("gültig");
    m.entities.clear();
    let creature = |id: &str, kind: &str, tx: usize, hh: f32| {
        o(
            id,
            at(tx, floor, hh),
            ObjectKind::Creature {
                kind: kind.into(),
                persistent: false,
            },
        )
    };
    m.adventure.objects = vec![
        o("west", at(5, floor, 28.0), ObjectKind::Spawn),
        o(
            "weg-dorf",
            corner(1, 14),
            ObjectKind::Exit {
                size: Vec2::new(T, 8.0 * T),
                map: "tauwinkel".into(),
                spawn: "ost".into(),
                on_touch: true,
            },
        ),
        o(
            "bruecke",
            corner(24, 12),
            ObjectKind::Zone {
                size: Vec2::new(12.0 * T, 10.0 * T),
            },
        ),
        creature("kaefer-1", "stachelkaefer", 38, 26.0),
        creature("kaefer-2", "stachelkaefer", 44, 26.0),
        creature("kaefer-3", "stachelkaefer", 50, 26.0),
        creature("blaeser", "pollenblaeser", 56, 60.0),
        creature("huepfer", "grashuepfer", 62, 28.0),
        o(
            "blume",
            at(48, floor, 16.0),
            ObjectKind::HealPlant { heal: 2 },
        ),
        o(
            "stein",
            Vec2::new(64.0 * T, 16.0 * T - 14.0),
            ObjectKind::Collectible {
                item: "glitzerstein".into(),
            },
        ),
        o("quellstein", at(78, floor, 40.0), ObjectKind::SavePoint),
        o(
            "truhe-quelle",
            at(84, floor, 26.0),
            ObjectKind::Chest {
                contents: vec![("glanztropfen".into(), 40), ("bernstein".into(), 3)],
                lock: "hat glitzerstein".into(),
            },
        ),
        o(
            "quelle",
            corner(80, 12),
            ObjectKind::Zone {
                size: Vec2::new(9.0 * T, 10.0 * T),
            },
        ),
        o(
            "kamera-quelle",
            corner(74, 4),
            ObjectKind::Camera {
                size: Vec2::new(15.0 * T, 18.0 * T),
                mode: CameraMode::Bounds,
            },
        ),
    ];
    m
}

#[test]
fn test_maps_match_content_and_link_up() {
    let c = Content::builtin();
    let (a, b) = (tauwinkel(), wiese());
    for m in [&a, &b] {
        let enc = elora_map::encode(m);
        let back = elora_map::decode(&enc).expect("Karte gültig");
        let errors = map_objects(&c, &back);
        assert!(errors.is_empty(), "{errors:?}");
    }
    let errors = map_links(&[("tauwinkel", &a), ("wiese-1", &b)]);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(c.progression.start_map, "tauwinkel");
    assert!(a.adventure.object(&c.progression.start_spawn).is_some());
}

#[test]
fn shipped_test_maps_are_current() {
    for (name, map) in [("tauwinkel", tauwinkel()), ("wiese-1", wiese())] {
        let path = format!(
            "{}/../../maps/abenteuer/{name}.emap",
            env!("CARGO_MANIFEST_DIR")
        );
        let file = std::fs::read(&path).expect("Karte vorhanden – mit --ignored erzeugen");
        assert_eq!(
            elora_map::decode(&file).expect("gültig"),
            map,
            "{name} veraltet – mit --ignored neu schreiben"
        );
    }
}

#[test]
#[ignore = "schreibt maps/abenteuer/*.emap"]
fn write_test_maps() {
    let dir = format!("{}/../../maps/abenteuer", env!("CARGO_MANIFEST_DIR"));
    std::fs::create_dir_all(&dir).unwrap();
    for (name, map) in [("tauwinkel", tauwinkel()), ("wiese-1", wiese())] {
        std::fs::write(format!("{dir}/{name}.emap"), elora_map::encode(&map)).unwrap();
    }
}

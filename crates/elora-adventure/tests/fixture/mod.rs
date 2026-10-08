//! Simple test maps for the session (A1.6), independent of the prologue maps (A1.9):
//! `tauwinkel` and `wiese-1` with chest, lever and gate, transition, zones and enemies.

#![allow(clippy::many_single_char_names, dead_code)]

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

/// Rows for `Map::from_rows`; the `S` there is only needed by the text format, the map
/// uses entrances (entities are removed afterwards).
fn rows(g: &[Vec<char>]) -> Vec<String> {
    let mut g = g.to_vec();
    g[1][1] = 'S';
    g.iter().map(|r| r.iter().collect()).collect()
}

/// Centre above the ground in column `tx` for an object of height `h`.
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

pub fn tauwinkel() -> Map {
    let (w, h, floor) = (70, 24, 20);
    let mut g = grid(w, h, floor);
    // gate: passage in column 46, a ceiling above it so you can't jump over it
    for row in g.iter_mut().take(17).skip(1) {
        row[46] = '%';
    }
    let refs = rows(&g);
    let r: Vec<&str> = refs.iter().map(String::as_str).collect();
    let mut m = Map::from_rows("Tauwinkel (Test)", &r).expect("valid");
    m.entities.clear();
    m.adventure.objects = vec![
        o("start", at(5, floor, 28.0), ObjectKind::Spawn),
        o("east", at(62, floor, 28.0), ObjectKind::Spawn),
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
        o("well", at(18, floor, 40.0), ObjectKind::SavePoint),
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
            "lotte",
            at(29, floor, 28.0),
            ObjectKind::Npc {
                character: "lotte".into(),
                dialog: "lotte".into(),
                facing: -1,
                walk: 0.0,
            },
        ),
        o(
            "klonk",
            at(38, floor, 28.0),
            ObjectKind::Npc {
                character: "klonk".into(),
                dialog: "klonk".into(),
                facing: -1,
                walk: 0.0,
            },
        ),
        o(
            "chest-1",
            at(34, floor, 26.0),
            ObjectKind::Chest {
                contents: vec![("gleam_drops".into(), 20), ("healing_potion".into(), 1)],
                lock: String::new(),
            },
        ),
        o(
            "lever",
            at(42, floor, 30.0),
            ObjectKind::Switch {
                flag: "gate.village".into(),
                once: false,
                trigger: SwitchTrigger::Interact,
            },
        ),
        o(
            "gate",
            corner(46, 17),
            ObjectKind::Door {
                size: (1, 3),
                open_if: "flag gate.village".into(),
            },
        ),
        o(
            "flower",
            at(52, floor, 16.0),
            ObjectKind::HealPlant { heal: 2 },
        ),
        o(
            "path-meadow",
            corner(66, 14),
            ObjectKind::Exit {
                size: Vec2::new(3.0 * T, 6.0 * T),
                map: "meadow-1".into(),
                spawn: "west".into(),
                on_touch: true,
            },
        ),
    ];
    m
}

pub fn meadow() -> Map {
    let (w, h, floor) = (90, 26, 22);
    let mut g = grid(w, h, floor);
    // bridge over a dip
    for row in g.iter_mut().take(floor + 2).skip(floor) {
        for c in &mut row[26..34] {
            *c = '.';
        }
    }
    for c in &mut g[floor][26..34] {
        *c = '=';
    }
    // ledge with glitter stone and crumble floor below
    for c in &mut g[16][60..68] {
        *c = '#';
    }
    for c in &mut g[floor][70..74] {
        *c = ':';
    }
    let refs = rows(&g);
    let r: Vec<&str> = refs.iter().map(String::as_str).collect();
    let mut m = Map::from_rows("Blütenwiesen 1 (Test)", &r).expect("valid");
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
            "path-village",
            corner(1, 14),
            ObjectKind::Exit {
                size: Vec2::new(T, 8.0 * T),
                map: "tauwinkel".into(),
                spawn: "east".into(),
                on_touch: true,
            },
        ),
        o(
            "meadow_edge",
            corner(24, 12),
            ObjectKind::Zone {
                size: Vec2::new(12.0 * T, 10.0 * T),
            },
        ),
        creature("beetle-1", "spike_beetle", 38, 26.0),
        creature("beetle-2", "spike_beetle", 44, 26.0),
        creature("beetle-3", "spike_beetle", 50, 26.0),
        creature("blower", "pollen_blower", 56, 60.0),
        creature("hopper", "grasshopper", 62, 28.0),
        o(
            "flower",
            at(48, floor, 16.0),
            ObjectKind::HealPlant { heal: 2 },
        ),
        o(
            "stone",
            Vec2::new(64.0 * T, 16.0 * T - 14.0),
            ObjectKind::Collectible {
                item: "glitter_stone".into(),
            },
        ),
        o("spring_stone", at(78, floor, 40.0), ObjectKind::SavePoint),
        o(
            "chest-spring",
            at(84, floor, 26.0),
            ObjectKind::Chest {
                contents: vec![("gleam_drops".into(), 40), ("amber".into(), 3)],
                lock: "has glitter_stone".into(),
            },
        ),
        o(
            "spring",
            corner(80, 12),
            ObjectKind::Zone {
                size: Vec2::new(9.0 * T, 10.0 * T),
            },
        ),
        o(
            "camera-spring",
            corner(74, 4),
            ObjectKind::Camera {
                size: Vec2::new(15.0 * T, 18.0 * T),
                mode: CameraMode::Bounds,
            },
        ),
    ];
    m
}

/// Test map by name.
pub fn load(name: &str) -> Map {
    match name {
        "tauwinkel" => tauwinkel(),
        "meadow-1" => meadow(),
        _ => panic!("no test map {name}"),
    }
}

#[test]
fn test_maps_match_content_and_link_up() {
    let c = elora_adventure::Content::builtin();
    let (a, b) = (tauwinkel(), meadow());
    for m in [&a, &b] {
        let back = elora_map::decode(&elora_map::encode(m)).expect("map valid");
        let errors = elora_adventure::check::map_objects(&c, &back);
        assert!(errors.is_empty(), "{errors:?}");
    }
    let errors = elora_adventure::check::map_links(&[("tauwinkel", &a), ("meadow-1", &b)]);
    assert!(errors.is_empty(), "{errors:?}");
}

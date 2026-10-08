//! Maps of chapter 2 (R2-M2.2): `wald-1` (forest edge with mushroom ring), `wald-2` (tree house
//! settlement in the crowns), `wald-3` (root caves) and `wald-arena` (forest spring with the
//! root guardian). The forest lies west of Tauwinkel: the maps are built from left (village)
//! to right (deeper in the forest) and then mirrored.
//!
//! `cargo test -p elora-client --bin elora write_chapter2_maps -- --ignored` writes
//! `maps/adventure/*.emap`.

#![allow(
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::too_many_lines
)]

use elora_map::adventure::CameraMode;
use elora_map::{Decor, Map, Object, ObjectKind};
use elora_sim::{TILE_SIZE, Vec2};

use super::prologue::{
    Grid, T, animate, at, chest, climb_vault, corner, creature, decor, finish, jerk_gate, npc, o,
    plant, sign, stomp_vault,
};
use super::release;

/// Transition over the full map height at the edge (E-279).
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
        m.decor_front.push(decor("thorns", x as f32 + 0.5, y + 1));
        x += 2;
    }
}

/// Mirror horizontally: tiles, objects (areas at their left edge) and decoration.
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

/// Forest look: theme „Wald“ of the release maps, a somewhat shadier sky.
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

/// Murmelwald 1, 220 × 60: forest edge with the mushroom ring near Tauwinkel, gentle hills (the
/// path on the ground has no gaps – the mushroom child comes along), crown ledges as the upper
/// path, stone pillar with a rune (only with the pull hook).
pub fn forest_1() -> Map {
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
    // upper path through the crowns
    g.fill((66, 74), (32, 32), '=');
    g.fill((78, 86), (27, 27), '=');
    g.fill((88, 92), (20, 21), '#');
    g.fill((100, 108), (34, 34), '=');
    g.fill((110, 116), (36, 36), '=');
    // floating stone pillar: rune at the top, only with the pull hook (M2.2.2); free below for
    // the path
    g.fill((186, 188), (20, 30), '%');
    // climbing spot for the return with the ice grip (M2.4.7)
    let climb = climb_vault(
        &mut g,
        "forest1-climb",
        152,
        40,
        &[("gleam_drops", 60), ("resin", 2)],
    );
    // stomp chamber for the return with stomp (M2.3.6)
    let stomp = stomp_vault(
        &mut g,
        "forest1-stomp",
        140,
        42,
        &[("gleam_drops", 60), ("resin", 2)],
    );
    let mut m = g.map("Murmelwald 1");
    m.adventure.objects = vec![
        edge_exit("path-village", 0, h, "tauwinkel", "west"),
        o("east", at(6, 44, 28.0), ObjectKind::Spawn),
        sign("sign-forest", 11, 44),
        o(
            "mushroom_ring",
            corner(17, 34),
            ObjectKind::Zone {
                size: Vec2::new(14.0 * T, 10.0 * T),
            },
        ),
        npc("mushroom_mama", 22, 44, 1, 0.0),
        npc("mushroom_child_happy", 26, 44, -1, 0.0),
        sign("sign-mushroom-ring", 32, 44),
        creature("serpent-1", "root_snake", 52, 40, 110.0),
        creature("imp-1", "mushroom_imp", 72, 42, 40.0),
        rune("rune-1", 90, 20),
        creature("imp-2", "mushroom_imp", 86, 42, 40.0),
        creature("serpent-2", "root_snake", 104, 46, 110.0),
        chest(
            "chest-ammo",
            100,
            46,
            &[("ammo_grenade", 1), ("gleam_drops", 10)],
        ),
        creature("pirate-1", "squirrel_pirate", 113, 36, 52.0),
        plant("flower-1", 118, 46),
        o("spring_stone", at(130, 42, 40.0), ObjectKind::SavePoint),
        creature("imp-3", "mushroom_imp", 160, 40, 40.0),
        creature("serpent-3", "root_snake", 172, 40, 110.0),
        rune("rune-2", 187, 20),
        plant("flower-2", 196, 38),
        o("west", at(214, 40, 28.0), ObjectKind::Spawn),
        edge_exit("path-forest-2", w - 2, h, "forest-2", "east"),
        stomp,
        climb,
    ];
    m.decor_back = vec![
        big("forest_tree", 4.0, 44, 0.8),
        big("mushroom_ring", 24.0, 44, 1.0),
        big("forest_tree", 40.0, 42, 0.9),
        big("forest_tree", 70.0, 42, 1.4),
        big("forest_tree", 84.0, 42, 1.5),
        big("forest_tree", 112.0, 46, 1.3),
        big("root_arch", 140.0, 42, 1.0),
        big("forest_tree", 165.0, 40, 1.1),
        big("root_arch", 196.0, 38, 0.9),
        big("forest_tree", 208.0, 40, 1.0),
    ];
    m.decor_front = vec![
        decor("glow_mushrooms", 15.0, 44),
        decor("fern", 36.0, 42),
        decor("glow_mushrooms", 60.0, 40),
        decor("fern", 98.0, 46),
        decor("glow_mushrooms", 122.0, 44),
        decor("fern", 152.0, 40),
        decor("tree_stump", 180.0, 38),
        decor("glow_mushrooms", 204.0, 40),
    ];
    forest(m)
}

/// Murmelwald 2, 200 × 90: tall trees, ledges and suspension bridges in the crowns, tree houses,
/// Plumm at the very top; the mushroom child on the ground by the glowing mushrooms.
pub fn forest_2() -> Map {
    let (w, h, f) = (200, 90, 80);
    let mut g = Grid::new(w, h, f);
    g.ground(30, 59, 78);
    g.ground(60, 140, 76);
    g.ground(141, 170, 78);
    // ascent: ledges in a zigzag (each 4 tiles higher)
    for (k, y) in (40..=72).rev().step_by(4).enumerate() {
        let x = if k % 2 == 0 { 62 } else { 69 };
        g.fill((x, x + 5), (y, y), '=');
    }
    // suspension bridge and platforms in the crowns
    g.fill((76, 118), (44, 44), '=');
    g.fill((120, 132), (40, 40), '=');
    g.fill((134, 146), (36, 36), '=');
    g.fill((150, 156), (28, 29), '#');
    // second descent on the other side
    for (k, y) in (40..=72).step_by(4).enumerate() {
        let x = if k % 2 == 0 { 160 } else { 167 };
        g.fill((x, x + 5), (y, y), '=');
    }
    let mut m = g.map("Murmelwald 2");
    m.adventure.objects = vec![
        edge_exit("path-forest-1", 0, h, "forest-1", "west"),
        o("east", at(6, 80, 28.0), ObjectKind::Spawn),
        npc("mushroom_child", 40, 78, 1, 0.0),
        chest(
            "chest-ammo",
            50,
            78,
            &[("ammo_grenade", 1), ("gleam_drops", 10)],
        ),
        creature("imp-1", "mushroom_imp", 52, 78, 40.0),
        creature("serpent-1", "root_snake", 90, 76, 110.0),
        o("tree-rest", at(100, 76, 40.0), ObjectKind::SavePoint),
        creature("pirate-1", "squirrel_pirate", 95, 44, 52.0),
        npc("plumm", 126, 40, -1, 0.0),
        creature("pirate-2", "squirrel_pirate", 140, 36, 52.0),
        rune("rune-3", 153, 28),
        chest("chest-crown", 155, 28, &[("gleam_drops", 35), ("resin", 2)]),
        creature("serpent-2", "root_snake", 150, 78, 110.0),
        plant("flower-1", 120, 76),
        creature("imp-2", "mushroom_imp", 180, 80, 40.0),
        o("west", at(194, 80, 28.0), ObjectKind::Spawn),
        edge_exit("path-forest-3", w - 2, h, "forest-3", "east"),
    ];
    m.decor_back = vec![
        big("forest_tree", 20.0, 80, 1.3),
        big("forest_tree", 60.0, 78, 2.1),
        big("forest_tree", 75.0, 76, 2.2),
        big("forest_house", 96.0, 44, 1.0),
        big("hanging_bridge", 106.0, 44, 1.0),
        big("forest_tree", 112.0, 76, 2.2),
        big("forest_house", 126.0, 40, 1.1),
        big("forest_tree", 140.0, 76, 2.2),
        big("forest_tree", 158.0, 78, 2.1),
        big("forest_tree", 185.0, 80, 1.3),
    ];
    m.decor_front = vec![
        decor("glow_mushrooms", 36.0, 78),
        decor("glow_mushrooms", 44.0, 78),
        decor("fern", 64.0, 76),
        decor("fern", 104.0, 76),
        decor("glow_mushrooms", 130.0, 76),
        decor("tree_stump", 175.0, 80),
    ];
    forest(m)
}

/// Murmelwald 3, 200 × 70: down into the root caves, thorns, crumbling floor, hanging
/// roots; pull switch chamber with a rune (only with the pull hook), hook jerk spot with a rune,
/// at the end up to the forest spring.
pub fn forest_3() -> Map {
    let (w, h) = (200, 70);
    let mut g = Grid::new(w, h, 30);
    // cave under the roots: passage from row 40 to 60
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
    // root with the pull switch (reachable with the hook from the cave floor)
    g.fill((122, 123), (40, 46), '#');
    // pull switch chamber in the ceiling: ledge as the floor, gate below (`merker zug.wald3`)
    g.fill((106, 116), (34, 37), '.');
    g.fill((106, 116), (38, 38), '=');
    g.fill((106, 116), (39, 39), '.');
    // ascent at the end
    g.fill((154, 160), (24, 51), '.');
    for (x, y) in [(154, 46), (157, 40), (154, 34), (157, 28)] {
        g.fill((x, x + 3), (y, y), '=');
    }
    g.ground(161, w - 1, 26);
    // hook jerk spot at the top with rune 4
    let jerk_top = jerk_gate(&mut g, 170, 26);
    // stomp chamber in the cave floor (M2.3.6)
    let stomp = stomp_vault(
        &mut g,
        "forest3-stomp",
        26,
        57,
        &[("gleam_drops", 70), ("dew_potion", 1)],
    );
    let mut m = g.map("Murmelwald 3");
    m.adventure.objects = vec![
        edge_exit("path-forest-2", 0, h, "forest-2", "west"),
        o("east", at(5, 30, 28.0), ObjectKind::Spawn),
        creature("serpent-1", "root_snake", 36, 57, 110.0),
        creature("imp-1", "mushroom_imp", 44, 57, 40.0),
        creature("pirate-1", "squirrel_pirate", 75, 55, 52.0),
        o("cave", at(102, 57, 40.0), ObjectKind::SavePoint),
        chest(
            "chest-ammo",
            108,
            57,
            &[("ammo_grenade", 1), ("gleam_drops", 10)],
        ),
        o(
            "pull",
            Vec2::new(123.0 * T, 47.0 * T + 14.0),
            ObjectKind::Switch {
                flag: "pull.forest3".into(),
                once: true,
                trigger: elora_map::adventure::SwitchTrigger::Hook,
            },
        ),
        o(
            "gate-chamber",
            corner(106, 39),
            ObjectKind::Door {
                size: (11, 1),
                open_if: "flag pull.forest3".into(),
            },
        ),
        rune("rune-5", 109, 38),
        chest(
            "chest-chamber",
            114,
            38,
            &[("gleam_drops", 40), ("dew_potion", 1)],
        ),
        creature("serpent-2", "root_snake", 120, 57, 110.0),
        creature("imp-2", "mushroom_imp", 150, 52, 40.0),
        plant("flower-1", 148, 52),
        rune("rune-4", 179, jerk_top),
        o("west", at(194, 26, 28.0), ObjectKind::Spawn),
        edge_exit("path-arena", w - 2, h, "forest-arena", "east"),
        stomp,
    ];
    m.decor_back = vec![
        big("forest_tree", 8.0, 30, 1.0),
        big("forest_tree", 40.0, 30, 1.2),
        big("forest_tree", 90.0, 30, 1.1),
        big("forest_tree", 140.0, 30, 1.2),
        big("forest_tree", 190.0, 26, 1.1),
    ];
    m.decor_front = vec![
        decor("glow_mushrooms", 30.0, 57),
        decor("glow_mushrooms", 70.0, 55),
        decor("glow_mushrooms", 104.0, 57),
        decor("glow_mushrooms", 126.0, 57),
        decor("glow_mushrooms", 150.0, 52),
        decor("fern", 166.0, 26),
    ];
    thorn_decor(&mut m, 60, 51, 62);
    thorn_decor(&mut m, 60, 91, 98);
    thorn_decor(&mut m, 60, 131, 144);
    forest(m)
}

/// Forest spring, 100 × 50: from the ledge down into the arena, hook flowers to dodge
/// the root walls, gate after the victory, root path back to Tauwinkel.
pub fn forest_arena() -> Map {
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
        o("east", at(5, 26, 28.0), ObjectKind::Spawn),
        o("before-the-spring", at(11, 26, 40.0), ObjectKind::SavePoint),
        o(
            "root_warden",
            at(56, 44, 280.0),
            ObjectKind::Creature {
                kind: "root_warden".into(),
                persistent: true,
            },
        ),
        npc("warden", 56, 44, -1, 0.0),
        o(
            "gate",
            corner(80, 36),
            ObjectKind::Door {
                size: (2, 8),
                open_if: "flag defeated.root_warden".into(),
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
        edge_exit("path-home", w - 2, h, "tauwinkel", "west"),
    ];
    m.decor_back = vec![
        big("moss-spring-withered", 56.0, 44, 1.5),
        big("forest_tree", 4.0, 26, 1.0),
        big("forest_tree", 14.0, 26, 1.1),
        big("forest_tree", 22.0, 44, 1.5),
        big("forest_tree", 77.0, 44, 1.5),
        big("forest_tree", 90.0, 44, 1.2),
    ];
    m.decor_front = vec![
        decor("glow_mushrooms", 30.0, 44),
        decor("glow_mushrooms", 70.0, 44),
        decor("fern", 40.0, 44),
    ];
    forest(m)
}

/// Maps of chapter 2 with their names.
pub fn maps() -> Vec<(&'static str, Map)> {
    vec![
        ("forest-1", forest_1()),
        ("forest-2", forest_2()),
        ("forest-3", forest_3()),
        ("forest-arena", forest_arena()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_adventure::Content;
    use elora_adventure::check::{map_links, map_objects};

    fn shipped(name: &str) -> String {
        format!(
            "{}/../../maps/adventure/{name}.{}",
            env!("CARGO_MANIFEST_DIR"),
            elora_map::EXTENSION
        )
    }

    #[test]
    fn forest_maps_match_content_and_link_up() {
        let c = Content::builtin();
        let mut all = super::super::chapter1::all_maps();
        all.extend(maps());
        all.extend(super::super::chapter3::maps());
        all.extend(super::super::chapter4::maps());
        for (name, m) in &all {
            let back = elora_map::decode(&elora_map::encode(m)).expect("map valid");
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
            let file = std::fs::read(shipped(name)).expect("map present");
            assert_eq!(
                elora_map::decode(&file).expect("valid"),
                map,
                "{name} outdated – write_chapter2_maps -- --ignored"
            );
        }
    }

    #[test]
    #[ignore = "writes maps/adventure/*.emap"]
    fn write_chapter2_maps() {
        for (name, map) in maps() {
            map.save(std::path::Path::new(&shipped(name))).unwrap();
        }
    }

    /// Overview: `… chapter2_sheets -- --ignored` → `target/chapter2-<karte>.svg`.
    #[test]
    #[ignore = "only writes files for visual inspection"]
    fn chapter2_sheets() {
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
                "{}/../../target/chapter2-{name}.svg",
                env!("CARGO_MANIFEST_DIR")
            );
            std::fs::write(path, svg).unwrap();
        }
    }
}

//! Maps of chapter 3 (R2-M2.3): `wueste-1` (dune edge), `wueste-2` (caravan camp and
//! oasis), `wueste-3` (ruins of the ember folk with the buried chamber) and `wueste-arena`
//! (ember spring in the sand basin with the sand snake). The desert lies south of Tauwinkel:
//! the sunken path at the east path leads down, the maps are built from left to right.
//!
//! `cargo test -p elora-client --bin elora write_chapter3_maps -- --ignored` writes
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
    plant, pull_vault, sign,
};
use super::release::{self, Theme};

/// Desert: theme „Wüste“ of the release maps, large decoration matching the region.
const DESERT: Theme = Theme {
    back: &[("cactus", 5), ("rock-2", 1), ("rock-1", 1)],
    front: &["rock-1", "grass-2"],
    front_density: 10,
    back_spacing: 13,
    ..release::THEMES[1]
};

/// Transition over the full map height at the edge (E-279).
pub(super) fn edge_exit(id: &str, x: usize, h: usize, map: &str, spawn: &str) -> Object {
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

pub(super) fn big(name: &str, tx: f32, ty: usize, scale: f32) -> Decor {
    let mut d = decor(name, tx, ty);
    d.scale = scale;
    d
}

pub(super) fn zone(id: &str, x: (usize, usize), y: (usize, usize)) -> Object {
    o(
        id,
        corner(x.0, y.0),
        ObjectKind::Zone {
            size: Vec2::new((x.1 - x.0 + 1) as f32 * T, (y.1 - y.0 + 1) as f32 * T),
        },
    )
}

fn crab(id: &str, tx: usize, ty: usize) -> Object {
    creature(id, "sand_crab", tx, ty, 30.0)
}

fn worm(id: &str, tx: usize, ty: usize) -> Object {
    creature(id, "dune_worm", tx, ty, 36.0)
}

/// Spark moth `rows` tiles above the ground.
fn moth(id: &str, tx: usize, ty: usize, rows: usize) -> Object {
    creature(id, "spark_moth", tx, ty - rows, 32.0)
}

/// Quicksand instead of ground in the columns `x0..=x1` (one row, solid ground below, E-318).
fn quicksand(g: &mut Grid, x0: usize, x1: usize, top: usize) {
    g.fill((x0, x1), (top, top), '&');
}

/// Gray footprints of the wanderer (E-325).
fn tracks(m: &mut Map, spots: &[(f32, usize)]) {
    for &(tx, ty) in spots {
        m.decor_front.push(big("grey_trail", tx, ty, 2.2));
    }
}

/// Desert look: theme „Wüste“, large custom decoration stays in the back.
fn desert(m: Map) -> Map {
    let mut map = finish(m, &DESERT);
    release::apply_look(&DESERT, &mut map);
    animate(&mut map, &[]);
    map
}

/// Glutsandwüste 1, 240 × 60: from the sunken path down onto the dune edge, a rock roof as
/// shade, first quicksand pits, sand crabs and dune worms.
pub fn desert_1() -> Map {
    let (w, h) = (240, 60);
    let mut g = Grid::new(w, h, 50);
    // down from the sunken path
    g.ground(0, 14, 24);
    g.ground(15, 20, 28);
    g.ground(21, 26, 32);
    g.ground(27, 34, 36);
    g.ground(35, 60, 40);
    // rock roof as shade (E-320)
    g.fill((41, 53), (30, 31), '#');
    g.ground(61, 64, 42);
    g.ground(65, 90, 44);
    quicksand(&mut g, 72, 79, 44);
    g.ground(91, 110, 41);
    g.ground(111, 130, 38);
    g.ground(131, 140, 41);
    g.ground(141, 170, 44);
    quicksand(&mut g, 148, 153, 44);
    quicksand(&mut g, 160, 165, 44);
    // second rock roof
    g.fill((175, 186), (34, 35), '#');
    g.ground(171, 200, 42);
    g.ground(201, 215, 40);
    g.ground(216, w - 1, 42);
    let mut m = g.map("Glutsandwüste 1");
    m.adventure.objects = vec![
        edge_exit("path-village", 0, h, "tauwinkel", "sunken_path"),
        o("north", at(6, 24, 28.0), ObjectKind::Spawn),
        sign("sign-heat", 11, 24),
        chest(
            "chest-ammo",
            13,
            24,
            &[("ammo_grenade", 1), ("gleam_drops", 10)],
        ),
        crab("crab-1", 56, 40),
        sign("sign-quicksand", 68, 44),
        crab("crab-2", 85, 44),
        worm("worm-1", 100, 41),
        o("dunes-rest", at(120, 38, 40.0), ObjectKind::SavePoint),
        plant("flower-1", 126, 38),
        worm("worm-2", 144, 44),
        crab("crab-3", 157, 44),
        worm("worm-3", 168, 44),
        crab("crab-4", 192, 42),
        worm("worm-4", 208, 40),
        o("east", at(234, 42, 28.0), ObjectKind::Spawn),
        edge_exit("path-camp", w - 2, h, "desert-2", "west"),
    ];
    m.decor_back = vec![
        big("dune", 8.0, 24, 1.2),
        big("rock_arch", 47.0, 40, 1.5),
        big("dune", 70.0, 44, 1.6),
        big("palm", 115.0, 38, 1.0),
        big("dune", 135.0, 41, 1.3),
        big("rock_arch", 180.5, 42, 1.4),
        big("dune", 205.0, 40, 1.5),
        big("palm", 228.0, 42, 0.9),
    ];
    m.decor_front = vec![
        big("cactus", 30.0, 36, 0.6),
        big("cactus", 95.0, 41, 0.5),
        big("cactus", 133.0, 41, 0.6),
        big("cactus", 196.0, 42, 0.5),
    ];
    tracks(
        &mut m,
        &[
            (20.0, 28),
            (38.0, 40),
            (62.0, 42),
            (105.0, 41),
            (138.0, 41),
            (175.0, 42),
            (222.0, 42),
        ],
    );
    desert(m)
}

/// Glutsandwüste 2, 220 × 50: caravan camp with Sirup (tent roofs give shade) and
/// Palma's oasis in a hollow with three withered spots (E-319).
pub fn desert_2() -> Map {
    let (w, h) = (220, 50);
    let mut g = Grid::new(w, h, 44);
    g.ground(0, 80, 40);
    // tent roofs
    g.fill((36, 44), (34, 34), '=');
    g.fill((58, 66), (34, 34), '=');
    g.ground(81, 95, 42);
    // oasis in the hollow
    g.ground(96, 140, 45);
    g.ground(141, 150, 42);
    g.ground(151, w - 1, 40);
    // climbing spot for the return with the ice grip (M2.4.7)
    let climb = climb_vault(
        &mut g,
        "desert2-climb",
        16,
        40,
        &[("gleam_drops", 60), ("ember_stone", 2)],
    );
    let mut m = g.map("Karawanenlager");
    let blossom = |n: usize, tx: usize| {
        o(
            &format!("blossom-{n}"),
            at(tx, 45, 28.0),
            ObjectKind::Npc {
                character: format!("blossom-{n}"),
                dialog: "oasis-blossom".into(),
                facing: 1,
                walk: 0.0,
            },
        )
    };
    m.adventure.objects = vec![
        edge_exit("path-dunes", 0, h, "desert-1", "east"),
        o("west", at(5, 40, 28.0), ObjectKind::Spawn),
        o("camp", at(47, 40, 40.0), ObjectKind::SavePoint),
        npc("sirup", 52, 40, -1, 24.0),
        chest(
            "chest-ammo",
            68,
            40,
            &[("ammo_grenade", 1), ("gleam_drops", 10)],
        ),
        crab("crab-1", 90, 42),
        npc("palma", 104, 45, 1, 0.0),
        npc("watering-spot-1", 100, 45, 1, 0.0),
        blossom(1, 100),
        npc("watering-spot-2", 109, 45, 1, 0.0),
        blossom(2, 109),
        npc("watering-spot-3", 133, 45, 1, 0.0),
        blossom(3, 133),
        zone("oasis-1", (96, 140), (33, 44)),
        plant("flower-1", 138, 45),
        moth("moth-1", 160, 40, 9),
        crab("crab-2", 175, 40),
        plant("flower-2", 170, 40),
        moth("moth-2", 188, 40, 8),
        crab("crab-3", 202, 40),
        o("east", at(214, 40, 28.0), ObjectKind::Spawn),
        edge_exit("path-ruins", w - 2, h, "desert-3", "west"),
        climb,
    ];
    m.decor_back = vec![
        big("dune", 12.0, 40, 1.4),
        big("tent", 40.0, 40, 0.75),
        big("tent", 62.0, 40, 0.75),
        big("camel", 72.0, 40, 0.55),
        big("palm", 98.0, 45, 1.1),
        big("oasis", 120.0, 45, 1.4),
        big("palm", 128.0, 45, 0.9),
        big("palm", 139.0, 45, 1.2),
        big("dune", 170.0, 40, 1.5),
        big("rock_arch", 205.0, 40, 1.0),
    ];
    m.decor_front = vec![
        decor("barrels", 46.0, 40),
        decor("basket", 57.0, 40),
        big("cactus", 84.0, 42, 0.5),
        big("cactus", 158.0, 40, 0.6),
    ];
    tracks(
        &mut m,
        &[
            (116.0, 45),
            (146.0, 42),
            (165.0, 40),
            (190.0, 40),
            (212.0, 40),
        ],
    );
    desert(m)
}

/// Glutsandwüste 3, 240 × 70: ruins of the ember folk – tablets, spike pit, quicksand, the
/// courtyard with the ruin spring under a stone roof, pull chest, hook jerk spot and the
/// buried chamber under crumbling floor (only with stomp, E-323).
pub fn desert_3() -> Map {
    let (w, h) = (240, 70);
    let mut g = Grid::new(w, h, 50);
    g.ground(0, 100, 46);
    // raised ruin floors
    g.fill((30, 35), (43, 45), '#');
    g.fill((44, 50), (40, 45), '#');
    // spike pit with a hook flower above
    g.fill((61, 66), (46, 46), '^');
    g.fill((63, 63), (36, 36), '*');
    quicksand(&mut g, 92, 97, 46);
    // courtyard with the ruin spring, stone roof as shade
    g.ground(101, 130, 49);
    g.fill((101, 103), (46, 48), '#');
    g.fill((128, 130), (46, 48), '#');
    g.fill((101, 130), (38, 39), '%');
    g.ground(131, 239, 46);
    // pull chest (pull hook) and hook jerk spot (hook jerk)
    let vault = pull_vault(
        &mut g,
        "ruin",
        134,
        46,
        "ruin.pull_gate",
        &[("gleam_drops", 45), ("ember_stone", 2)],
    );
    let jerk_top = jerk_gate(&mut g, 148, 46);
    // buried chamber: crumbling floor (two rows), below it a room with a ledge
    g.fill((171, 185), (46, 56), '#');
    g.fill((172, 184), (48, 55), '.');
    g.fill((175, 179), (46, 47), ':');
    g.fill((180, 184), (52, 52), '=');
    // spike pit before the exit
    g.fill((205, 209), (46, 46), '^');
    let mut m = g.map("Ruinen des Glutvolks");
    m.adventure.objects = vec![
        edge_exit("path-camp", 0, h, "desert-2", "east"),
        o("west", at(4, 46, 28.0), ObjectKind::Spawn),
        npc("tablet-1", 14, 46, 1, 0.0),
        crab("crab-1", 26, 46),
        moth("moth-1", 40, 40, 8),
        worm("worm-1", 56, 46),
        npc("tablet-2", 76, 46, 1, 0.0),
        chest(
            "chest-ammo",
            58,
            46,
            &[("ammo_grenade", 1), ("gleam_drops", 10)],
        ),
        crab("crab-2", 86, 46),
        o("yard", at(108, 49, 40.0), ObjectKind::SavePoint),
        npc("ruin_spring", 116, 49, 1, 0.0),
        moth("moth-2", 122, 49, 7),
        chest(
            "chest-jerk",
            155,
            jerk_top,
            &[("gleam_drops", 50), ("dew_potion", 1)],
        ),
        worm("worm-2", 165, 46),
        sign("sign-chamber", 173, 46),
        zone("ruin_chamber", (172, 184), (48, 55)),
        npc("tablet-chamber", 176, 56, 1, 0.0),
        chest("chest-chamber", 182, 56, &[("sun_veil", 1)]),
        crab("crab-3", 194, 46),
        moth("moth-3", 200, 46, 8),
        worm("worm-3", 216, 46),
        plant("flower-1", 222, 46),
        o("east", at(234, 46, 28.0), ObjectKind::Spawn),
        edge_exit("path-basin", w - 2, h, "desert-arena", "west"),
    ];
    m.adventure.objects.extend(vault);
    m.decor_back = vec![
        big("ruin_gate", 22.0, 46, 0.9),
        big("pillar", 38.0, 46, 0.9),
        big("pillar-broken", 54.0, 46, 1.0),
        big("pillar", 72.0, 46, 1.0),
        big("pillar-broken", 84.0, 46, 0.9),
        big("pillar", 105.0, 49, 0.9),
        big("pillar", 126.0, 49, 0.9),
        big("ruin_gate", 116.0, 49, 1.0),
        big("pillar-broken", 145.0, 46, 1.0),
        big("pillar", 168.0, 46, 1.0),
        big("ruin_gate", 190.0, 46, 0.9),
        big("pillar-broken", 200.0, 46, 0.8),
        big("dune", 225.0, 46, 1.3),
    ];
    m.decor_front = vec![
        big("cactus", 8.0, 46, 0.5),
        big("cactus", 90.0, 46, 0.5),
        big("cactus", 228.0, 46, 0.6),
    ];
    tracks(
        &mut m,
        &[
            (10.0, 46),
            (42.0, 40),
            (80.0, 46),
            (112.0, 49),
            (140.0, 46),
            (198.0, 46),
            (230.0, 46),
        ],
    );
    desert(m)
}

/// Ember spring, 110 × 50: from the ledge down into the sand basin; the basin lies in the shade
/// of the rock walls (no heat in the fight). Ground three tiles thick (quicksand of the angry
/// sand snake), two ledges to dodge; gate after the victory, path back to Tauwinkel.
pub fn desert_arena() -> Map {
    let (w, h) = (110, 50);
    let mut g = Grid::new(w, h, 44);
    g.ground(0, 17, 28);
    g.fill((18, 19), (28, 43), '%');
    g.fill((30, 35), (39, 39), '=');
    g.fill((64, 69), (39, 39), '=');
    g.fill((80, 81), (12, 35), '%');
    g.ground(82, w - 1, 44);
    let mut m = g.map("Glutquelle");
    m.adventure.objects = vec![
        o("west", at(5, 28, 28.0), ObjectKind::Spawn),
        o("before-the-spring", at(11, 28, 40.0), ObjectKind::SavePoint),
        chest(
            "chest-ammo",
            14,
            28,
            &[("ammo_grenade", 1), ("ammo_laser", 1)],
        ),
        o(
            "sand_serpent",
            at(50, 44, 70.0),
            ObjectKind::Creature {
                kind: "sand_serpent".into(),
                persistent: true,
            },
        ),
        npc("serpent", 50, 44, -1, 0.0),
        zone("shade-basin", (20, 79), (14, 43)),
        o(
            "gate",
            corner(80, 36),
            ObjectKind::Door {
                size: (2, 8),
                open_if: "flag defeated.sand_serpent".into(),
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
        o("east", at(104, 44, 28.0), ObjectKind::Spawn),
        edge_exit("path-home", w - 2, h, "tauwinkel", "sunken_path"),
    ];
    m.decor_back = vec![
        big("ember-spring-withered", 50.0, 44, 1.6),
        big("palm", 4.0, 28, 1.0),
        big("rock_arch", 11.0, 28, 0.9),
        big("pillar", 24.0, 44, 1.2),
        big("pillar-broken", 76.0, 44, 1.2),
        big("dune", 95.0, 44, 1.3),
    ];
    m.decor_front = vec![big("cactus", 15.0, 28, 0.5), big("cactus", 100.0, 44, 0.5)];
    tracks(&mut m, &[(40.0, 44), (60.0, 44), (90.0, 44), (102.0, 44)]);
    desert(m)
}

/// Maps of chapter 3 with their names.
pub fn maps() -> Vec<(&'static str, Map)> {
    vec![
        ("desert-1", desert_1()),
        ("desert-2", desert_2()),
        ("desert-3", desert_3()),
        ("desert-arena", desert_arena()),
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
    fn desert_maps_match_content_and_link_up() {
        let c = Content::builtin();
        let mut all = super::super::chapter1::all_maps();
        all.extend(super::super::chapter2::maps());
        all.extend(maps());
        all.extend(super::super::chapter4::maps());
        for (name, m) in &all {
            let back = elora_map::decode(&elora_map::encode(m)).expect("map valid");
            let errors = map_objects(&c, &back);
            assert!(errors.is_empty(), "{name}: {errors:?}");
        }
        let refs: Vec<(&str, &Map)> = all.iter().map(|(n, m)| (*n, m)).collect();
        let errors = map_links(&refs);
        assert!(errors.is_empty(), "{errors:?}");
        // the quests' contents are on the maps
        let desert = maps();
        let objects: Vec<&Object> = desert
            .iter()
            .flat_map(|(_, m)| &m.adventure.objects)
            .collect();
        for id in [
            "sirup",
            "palma",
            "watering-spot-1",
            "watering-spot-2",
            "watering-spot-3",
            "ruin_spring",
            "tablet-chamber",
            "ruin_chamber",
            "oasis-1",
            "sand_serpent",
        ] {
            assert!(objects.iter().any(|o| o.id == id), "{id} missing");
        }
    }

    /// Quicksand always lies on solid ground, and the basin has three rows of ground.
    #[test]
    fn quicksand_rests_on_solid_ground() {
        for (name, m) in maps() {
            for y in 0..m.height - 1 {
                for x in 0..m.width {
                    if m.tiles[y * m.width + x] == elora_sim::Tile::Quicksand {
                        assert!(
                            m.tiles[(y + 1) * m.width + x].is_solid(),
                            "{name}: quicksand at {x},{y} without a floor"
                        );
                    }
                }
            }
        }
        let arena = desert_arena();
        for y in 44..47 {
            assert!(arena.tiles[y * arena.width + 50].is_solid());
        }
    }

    #[test]
    fn shipped_desert_maps_are_current() {
        for (name, map) in maps() {
            let file = std::fs::read(shipped(name)).expect("map present");
            assert_eq!(
                elora_map::decode(&file).expect("valid"),
                map,
                "{name} outdated – write_chapter3_maps -- --ignored"
            );
        }
    }

    #[test]
    #[ignore = "writes maps/adventure/*.emap"]
    fn write_chapter3_maps() {
        for (name, map) in maps() {
            map.save(std::path::Path::new(&shipped(name))).unwrap();
        }
    }

    /// Overview: `… chapter3_sheets -- --ignored` → `target/chapter3-<karte>.svg`.
    #[test]
    #[ignore = "only writes files for visual inspection"]
    fn chapter3_sheets() {
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
                "{}/../../target/chapter3-{name}.svg",
                env!("CARGO_MANIFEST_DIR")
            );
            std::fs::write(path, svg).unwrap();
        }
    }
}

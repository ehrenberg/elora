//! Maps of chapter 4 (R2-M2.4): `frost-1` (glacier foot), `frost-2` (abandoned mountain village
//! with Flocke's hut, the frozen cellar and the climbing chimney), `frost-3` (summit ridge in
//! the blizzard) and `frost-arena` (ice hall of the frost spring with Kristella). The mountain
//! trail starts at the top of Tauwinkel's upper village under an ice lid (D-M24-01); the maps
//! are built from left to right.
//!
//! `cargo test -p elora-client --bin elora write_chapter4_maps -- --ignored` writes
//! `maps/adventure/*.emap`.

#![allow(
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::too_many_lines
)]

use elora_map::adventure::CameraMode;
use elora_map::{Map, Object, ObjectKind, Weather, WeatherKind};
use elora_sim::{TILE_SIZE, Vec2};

use super::chapter3::{big, edge_exit, zone};
use super::prologue::{
    Grid, T, animate, at, chest, corner, creature, decor, finish, npc, o, plant, sign,
};
use super::release::{self, Theme};

/// Mountains: theme „Winter“ of the release maps, snowy firs in the back.
const WINTER: Theme = Theme {
    back: &[("fir-snow", 6), ("rock-2", 1)],
    front: &["rock-1"],
    front_density: 9,
    back_spacing: 11,
    ..release::THEMES[2]
};

fn winter(m: Map) -> Map {
    let mut map = finish(m, &WINTER);
    release::apply_look(&WINTER, &mut map);
    // in the mountains only clouds and distant peaks – no green forest, no meadow hills
    map.backgrounds
        .retain(|b| b.name == "Clouds" || b.name == "Mountains");
    animate(&mut map, &[]);
    map
}

/// Clear crystal (collectible for Klonk, D-M24-09) on the ground of row `ty`.
fn crystal(id: &str, tx: usize, ty: usize) -> Object {
    o(
        id,
        at(tx, ty, 24.0),
        ObjectKind::Collectible {
            item: "clear_crystal".into(),
        },
    )
}

/// Fireplace (decoration) with warmth zone `feuer…` (E-342) on the ground of row `ty`.
fn fire(m: &mut Map, id: &str, tx: usize, ty: usize) {
    m.adventure
        .objects
        .push(zone(id, (tx - 3, tx + 3), (ty - 5, ty - 1)));
    m.decor_back.push(big("fireplace", tx as f32, ty, 0.8));
}

/// Something hanging under the ceiling (bottom edge of the ceiling = top edge of row `ty`).
fn hanging(id: &str, kind: &str, tx: usize, ty: usize, h: f32) -> Object {
    o(
        id,
        Vec2::new(tx as f32 * T + T / 2.0, ty as f32 * T + h / 2.0 + 1.0),
        ObjectKind::Creature {
            kind: kind.into(),
            persistent: false,
        },
    )
}

fn icicle(id: &str, tx: usize, ty: usize) -> Object {
    hanging(id, "icicle", tx, ty, 48.0)
}

fn bat(id: &str, tx: usize, ty: usize) -> Object {
    hanging(id, "bat", tx, ty, 36.0)
}

fn seal(id: &str, tx: usize, ty: usize) -> Object {
    creature(id, "snowball_seal", tx, ty, 28.0)
}

/// Frost spirit `rows` tiles above the ground.
fn ghost(id: &str, tx: usize, ty: usize, rows: usize) -> Object {
    creature(id, "frost_ghost", tx, ty - rows, 52.0)
}

/// Gray spots in the ice: traces of the Withering (chapter 4).
fn traces(m: &mut Map, spots: &[(f32, usize)]) {
    for &(tx, ty) in spots {
        m.decor_front.push(big("grey-trail-ice", tx, ty, 0.7));
    }
}

/// Climbing chimney as on the abilities test map: two climbing walls in the columns `x0` and
/// `x0 + 4` (three tiles of air between them) from row `top` to `bottom`.
fn chimney(g: &mut Grid, x0: usize, top: usize, bottom: usize) {
    g.fill((x0, x0), (top, bottom), '|');
    g.fill((x0 + 1, x0 + 3), (top, bottom), '.');
    g.fill((x0 + 4, x0 + 4), (top, bottom), '|');
}

/// Glacier foot, 240 × 60: from the passage under the mountain trail onto the first slopes,
/// fireplace at the entrance, ice surfaces, rock passage with icicles, thin ice bridge over ice
/// water (below it a crystal on dry rock), Bolle's niche (only with the ice grip) and an
/// avalanche slope up to the mountain village.
pub fn frost_1() -> Map {
    let (w, h) = (240, 60);
    let mut g = Grid::new(w, h, 50);
    g.ground(0, 14, 40);
    g.ground(15, 24, 38);
    g.ground(25, 34, 36);
    g.ground(35, 50, 34);
    g.fill((38, 48), (34, 34), '~');
    g.ground(51, 60, 36);
    // rock passage with icicles
    g.ground(61, 90, 38);
    g.fill((61, 90), (30, 33), '#');
    g.ground(91, 100, 38);
    // thin ice bridge: ice water on the left, dry rock with a crystal on the right
    g.fill((101, 116), (38, 44), '.');
    g.fill((101, 116), (38, 38), '-');
    g.fill((101, 110), (41, 44), '+');
    g.fill((101, 116), (45, 49), '#');
    g.fill((111, 116), (41, 44), '#');
    g.ground(117, 125, 41);
    g.ground(126, 140, 39);
    g.ground(141, 170, 39);
    // Bolle's niche: climbing shaft on an unhookable rock wall
    g.fill((161, 167), (22, 38), '%');
    chimney(&mut g, 156, 22, 38);
    g.fill((157, 159), (39, 39), '#');
    g.fill((141, 145), (29, 29), '#');
    g.ground(171, 200, 37);
    // avalanche slope up (the boulders roll down to the west)
    g.ground(201, 205, 36);
    g.ground(206, 210, 34);
    g.ground(211, 215, 32);
    g.ground(216, 220, 30);
    g.ground(221, w - 1, 30);
    let mut m = g.map("Gletscherfuß");
    m.adventure.objects = vec![
        edge_exit("path-village", 0, h, "tauwinkel", "mountain_path"),
        o("west", at(5, 40, 28.0), ObjectKind::Spawn),
        sign("sign-cold", 8, 40),
        chest(
            "chest-ammo",
            20,
            38,
            &[("ammo_grenade", 1), ("ammo_laser", 1)],
        ),
        seal("seal-1", 46, 34),
        icicle("icicle-1", 67, 34),
        icicle("icicle-2", 75, 34),
        icicle("icicle-3", 83, 34),
        crystal("crystal-1", 88, 38),
        sign("sign-ice", 98, 38),
        crystal("crystal-2", 114, 41),
        o("glacier-rest", at(130, 39, 40.0), ObjectKind::SavePoint),
        plant("flower-1", 134, 39),
        crystal("crystal-3", 143, 29),
        seal("seal-2", 150, 39),
        npc("bolle", 164, 22, -1, 0.0),
        ghost("ghost-1", 185, 37, 4),
        seal("seal-3", 195, 37),
        zone("avalanche-glacier", (200, 225), (16, 35)),
        zone("avalanche-glacier-step", (203, 205), (32, 35)),
        o("east", at(233, 30, 28.0), ObjectKind::Spawn),
        edge_exit("path-mountain-village", w - 2, h, "frost-2", "west"),
    ];
    fire(&mut m, "fire-entrance", 12, 40);
    fire(&mut m, "fire-rest", 137, 39);
    m.decor_back.extend([
        big("peak", 30.0, 36, 1.2),
        big("glacier", 56.0, 36, 0.9),
        big("snowdrift", 95.0, 38, 0.8),
        big("peak", 150.0, 39, 1.4),
        big("glacier", 190.0, 37, 1.0),
        big("fir-snow", 232.0, 30, 0.9),
    ]);
    m.decor_front.extend([
        big("snowdrift", 3.0, 40, 0.5),
        big("snowdrift", 128.0, 39, 0.5),
        big("snowdrift", 226.0, 30, 0.6),
    ]);
    traces(&mut m, &[(53.0, 36), (122.0, 41), (178.0, 37)]);
    winter(m)
}

/// Abandoned mountain village, 260 × 60: cheese dairy with a frozen cellar (thin ice over ice
/// water, bats, Flocke's rope in the chest), Flocke's hut with a fire, behind it the climbing
/// chimney onto the plateau; there Kiesel's glacier crevasse and the path to the summit ridge.
pub fn frost_2() -> Map {
    let (w, h) = (260, 60);
    let mut g = Grid::new(w, h, 50);
    g.ground(0, 116, 38);
    // cellar under the cheese dairy: entry columns 47–49, floor of thin ice over ice water
    g.fill((46, 74), (39, 43), '.');
    g.fill((47, 49), (38, 38), '.');
    g.fill((50, 69), (44, 44), '-');
    g.fill((50, 69), (45, 47), '+');
    // roof of the cheese dairy and of Flocke's hut
    g.fill((42, 66), (31, 31), '=');
    g.fill((90, 104), (32, 32), '=');
    // climbing chimney behind the hut, on the right the rock wall up to the plateau
    chimney(&mut g, 112, 12, 37);
    g.fill((117, w - 1), (11, h - 1), '%');
    g.fill((117, w - 1), (11, 11), '#');
    // Kiesel's glacier crevasse on the plateau
    chimney(&mut g, 162, 12, 29);
    g.fill((163, 165), (11, 11), '.');
    // tower with a crystal
    g.fill((238, 240), (4, 10), '#');
    let mut m = g.map("Verlassenes Bergdorf");
    m.adventure.objects = vec![
        edge_exit("path-glacier", 0, h, "frost-1", "east"),
        o("west", at(4, 38, 28.0), ObjectKind::Spawn),
        seal("seal-1", 30, 38),
        bat("bat-1", 56, 39),
        bat("bat-2", 64, 39),
        chest("chest-rope", 72, 44, &[("rope", 1)]),
        o("village_well", at(82, 38, 40.0), ObjectKind::SavePoint),
        npc("bolle-hut", 92, 38, 1, 0.0),
        npc("kiesel-hut", 94, 38, 1, 0.0),
        npc("flocke", 97, 38, -1, 0.0),
        npc("wicke-hut", 101, 38, -1, 0.0),
        sign("sign-chimney", 109, 38),
        o("chimney-top", at(125, 11, 40.0), ObjectKind::SavePoint),
        plant("flower-1", 130, 11),
        ghost("ghost-1", 145, 11, 3),
        npc("kiesel", 164, 30, 1, 0.0),
        crystal("crystal-4", 165, 30),
        seal("seal-2", 185, 11),
        ghost("ghost-2", 205, 11, 4),
        crystal("crystal-5", 239, 4),
        o("east", at(254, 11, 28.0), ObjectKind::Spawn),
        edge_exit("path-ridge", w - 2, h, "frost-3", "west"),
    ];
    fire(&mut m, "fire-hut", 97, 38);
    fire(&mut m, "fire-plateau", 220, 11);
    m.decor_back.extend([
        big("mountain_hut", 20.0, 38, 0.9),
        big("mountain_hut", 55.0, 38, 1.3),
        big("mountain_hut", 97.0, 38, 1.0),
        big("fir-snow", 80.0, 38, 1.0),
        big("peak", 150.0, 11, 1.2),
        big("glacier", 195.0, 11, 1.0),
        big("fir-snow", 228.0, 11, 0.9),
        big("peak", 250.0, 11, 1.0),
    ]);
    m.decor_front.extend([
        decor("barrels", 34.0, 38),
        decor("woodpile", 86.0, 38),
        big("snowdrift", 106.0, 38, 0.5),
        big("snowdrift", 140.0, 11, 0.6),
    ]);
    traces(&mut m, &[(40.0, 38), (175.0, 11), (230.0, 11)]);
    winter(m)
}

/// Summit ridge, 280 × 60, always in a blizzard: two avalanche slopes, a valley with Wicke's
/// ledge above a climbing chimney, bats under a rock roof, the gray spot, fireplaces
/// against the cold and the spring stone before the ice hall.
pub fn frost_3() -> Map {
    let (w, h) = (280, 60);
    let mut g = Grid::new(w, h, 50);
    g.ground(0, 20, 20);
    // avalanche slope 1 up
    g.ground(21, 30, 18);
    g.ground(31, 40, 16);
    g.ground(41, 50, 14);
    g.ground(51, 70, 12);
    // valley with Wicke's ledge and rock roof
    g.ground(71, 110, 26);
    chimney(&mut g, 78, 11, 25);
    g.fill((83, 88), (10, 10), '#');
    g.fill((100, 108), (15, 16), '#');
    g.ground(111, 115, 23);
    g.ground(116, 120, 20);
    g.ground(121, 125, 17);
    g.ground(126, 160, 14);
    // avalanche slope 2 up
    g.ground(161, 170, 12);
    g.ground(171, 180, 10);
    g.ground(181, 190, 8);
    g.ground(191, 200, 6);
    g.ground(201, 220, 10);
    g.ground(221, w - 1, 10);
    g.fill((228, 228), (2, 9), '#');
    let mut m = g.map("Gipfelgrat");
    m.weather = Weather {
        kind: WeatherKind::Blizzard,
        intensity: 0.55,
        wind: 0.5,
    };
    m.adventure.objects = vec![
        edge_exit("path-mountain-village", 0, h, "frost-2", "east"),
        o("west", at(4, 20, 28.0), ObjectKind::Spawn),
        sign("sign-avalanche", 18, 20),
        zone("avalanche-ridge-1", (21, 60), (0, 17)),
        zone("avalanche-ridge-1-step", (24, 26), (14, 17)),
        seal("seal-1", 62, 12),
        npc("wicke", 86, 10, -1, 0.0),
        crystal("crystal-6", 84, 10),
        bat("bat-1", 102, 17),
        bat("bat-2", 106, 17),
        ghost("ghost-1", 95, 26, 4),
        npc("grey-spot", 135, 14, 1, 0.0),
        seal("seal-2", 150, 14),
        zone("avalanche-ridge-2", (161, 200), (0, 11)),
        zone("avalanche-ridge-2-step", (163, 165), (8, 11)),
        crystal("crystal-7", 198, 6),
        ghost("ghost-2", 210, 10, 4),
        o("before-the-hall", at(215, 10, 40.0), ObjectKind::SavePoint),
        plant("flower-1", 218, 10),
        crystal("crystal-8", 228, 2),
        o("east", at(274, 10, 28.0), ObjectKind::Spawn),
        edge_exit("path-hall", w - 2, h, "frost-arena", "west"),
    ];
    fire(&mut m, "fire-start", 10, 20);
    fire(&mut m, "fire-valley", 92, 26);
    fire(&mut m, "fire-hall", 206, 10);
    m.decor_back.extend([
        big("peak", 45.0, 14, 1.3),
        big("glacier", 90.0, 26, 1.0),
        big("peak", 140.0, 14, 1.4),
        big("peak", 240.0, 10, 1.2),
        big("fir-snow", 260.0, 10, 0.9),
    ]);
    m.decor_back.push(big("grey-trail-ice", 135.0, 14, 1.0));
    m.decor_front.extend([
        big("snowdrift", 66.0, 12, 0.6),
        big("snowdrift", 155.0, 14, 0.6),
        big("snowdrift", 250.0, 10, 0.6),
    ]);
    traces(&mut m, &[(140.0, 14), (225.0, 10), (268.0, 10)]);
    winter(m)
}

/// Ice hall of the frost spring, 100 × 50: from the ledge down into the hall; ice floor exactly as
/// wide as Kristella's frost waves (35 tiles), climbing walls on both sides and two
/// climbing pillars, above them the ceiling for the icicles. Gate after the victory, path back to
/// Tauwinkel.
pub fn frost_arena() -> Map {
    let (w, h) = (100, 50);
    let mut g = Grid::new(w, h, 44);
    g.ground(0, 24, 28);
    g.fill((25, 25), (29, 43), '|');
    g.fill((26, 60), (44, 44), '~');
    // hall under the rock: ceiling for the icicles, closed above
    g.fill((25, 61), (0, 18), '%');
    g.fill((25, 61), (17, 18), '#');
    g.fill((35, 35), (34, 43), '|');
    g.fill((51, 51), (34, 43), '|');
    g.fill((61, 61), (19, 43), '|');
    g.ground(62, w - 1, 44);
    let mut m = g.map("Eishalle");
    m.adventure.objects = vec![
        o("west", at(5, 28, 28.0), ObjectKind::Spawn),
        o("before-the-spring", at(11, 28, 40.0), ObjectKind::SavePoint),
        chest(
            "chest-ammo",
            15,
            28,
            &[("ammo_grenade", 1), ("ammo_laser", 1)],
        ),
        o(
            "ice_queen",
            Vec2::new(43.5 * T, 44.0 * T - 300.0),
            ObjectKind::Creature {
                kind: "kristella".into(),
                persistent: true,
            },
        ),
        npc("kristella", 43, 44, -1, 0.0),
        o(
            "gate",
            corner(61, 40),
            ObjectKind::Door {
                size: (1, 4),
                open_if: "flag defeated.kristella".into(),
            },
        ),
        o(
            "hall",
            corner(25, 16),
            ObjectKind::Camera {
                size: Vec2::new(37.0 * T, 30.0 * T),
                mode: CameraMode::Bounds,
            },
        ),
        o("east", at(94, 44, 28.0), ObjectKind::Spawn),
        edge_exit("path-home", w - 2, h, "tauwinkel", "mountain_path"),
    ];
    m.decor_back = vec![
        big("frost-spring-withered", 43.0, 44, 1.6),
        big("peak", 8.0, 28, 0.9),
        big("fir-snow", 20.0, 28, 0.8),
        big("glacier", 80.0, 44, 1.0),
    ];
    m.decor_front = vec![
        big("snowdrift", 3.0, 28, 0.5),
        big("snowdrift", 90.0, 44, 0.5),
    ];
    traces(&mut m, &[(70.0, 44), (85.0, 44)]);
    winter(m)
}

/// Maps of chapter 4 with their names.
pub fn maps() -> Vec<(&'static str, Map)> {
    vec![
        ("frost-1", frost_1()),
        ("frost-2", frost_2()),
        ("frost-3", frost_3()),
        ("frost-arena", frost_arena()),
    ]
}

/// Center of the hall in world units (Kristella's starting point), for tests.
#[cfg(test)]
const HALL_MID: f32 = 43.5 * TILE_SIZE as f32;

#[cfg(test)]
mod tests {
    use super::*;
    use elora_adventure::Content;
    use elora_adventure::check::{map_links, map_objects};
    use elora_sim::Tile;

    fn shipped(name: &str) -> String {
        format!(
            "{}/../../maps/adventure/{name}.{}",
            env!("CARGO_MANIFEST_DIR"),
            elora_map::EXTENSION
        )
    }

    #[test]
    fn mountain_maps_match_content_and_link_up() {
        let c = Content::builtin();
        let mut all = super::super::chapter1::all_maps();
        all.extend(super::super::chapter2::maps());
        all.extend(super::super::chapter3::maps());
        all.extend(maps());
        for (name, m) in &all {
            let back = elora_map::decode(&elora_map::encode(m)).expect("map valid");
            let errors = map_objects(&c, &back);
            assert!(errors.is_empty(), "{name}: {errors:?}");
        }
        let refs: Vec<(&str, &Map)> = all.iter().map(|(n, m)| (*n, m)).collect();
        let errors = map_links(&refs);
        assert!(errors.is_empty(), "{errors:?}");
        // the quests' contents are on the maps
        let mountains = maps();
        let objects: Vec<&Object> = mountains
            .iter()
            .flat_map(|(_, m)| &m.adventure.objects)
            .collect();
        for id in [
            "flocke",
            "bolle",
            "kiesel",
            "wicke",
            "bolle-hut",
            "kiesel-hut",
            "wicke-hut",
            "grey-spot",
            "chest-rope",
            "kristella",
            "ice_queen",
            "gate",
        ] {
            assert!(objects.iter().any(|o| o.id == id), "{id} missing");
        }
        let crystals = objects
            .iter()
            .filter(
                |o| matches!(&o.kind, ObjectKind::Collectible { item } if item == "clear_crystal"),
            )
            .count();
        assert_eq!(crystals, 8, "eight clear crystals (D-M24-09)");
        assert!(
            objects.iter().filter(|o| o.id.starts_with("fire")).count() >= 6,
            "enough fireplaces against the cold"
        );
    }

    /// Thin ice lies over ice water or air, ice water has a floor.
    #[test]
    fn thin_ice_and_ice_water_are_built_sensibly() {
        for (name, m) in maps() {
            for y in 0..m.height - 1 {
                for x in 0..m.width {
                    let t = m.tiles[y * m.width + x];
                    let below = m.tiles[(y + 1) * m.width + x];
                    if t == Tile::ThinIce {
                        assert!(!below.is_solid(), "{name}: thin ice at {x},{y} on rock");
                    }
                    if t == Tile::IceWater {
                        assert!(
                            below == Tile::IceWater || below.is_solid(),
                            "{name}: ice water at {x},{y} without a floor"
                        );
                    }
                }
            }
        }
    }

    /// The hall is as wide as Kristella's frost waves, the starting point lies in the middle.
    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn ice_hall_fits_the_frost_waves() {
        let c = Content::builtin();
        let k = c.creatures.iter().find(|k| k.name == "kristella").unwrap();
        let elora_sim::Behavior::Queen(d) = &k.behavior else {
            panic!("Kristella is a queen");
        };
        let m = frost_arena();
        let (lo, hi) = (HALL_MID - d.width / 2.0, HALL_MID + d.width / 2.0);
        let floor = 44;
        let x0 = (lo / T).floor() as usize;
        let x1 = (hi / T).floor() as usize;
        for x in x0..=x1 {
            assert_eq!(m.tiles[floor * m.width + x], Tile::Ice, "ice floor at {x}");
        }
        assert_eq!(
            m.tiles[(floor - 1) * m.width + x0 - 1],
            Tile::Climb,
            "wall on the left"
        );
        assert_eq!(
            m.tiles[(floor - 1) * m.width + x1 + 1],
            Tile::Climb,
            "wall with gate on the right"
        );
        let o = m.adventure.object("ice_queen").unwrap();
        assert!((o.pos.x - HALL_MID).abs() < 1.0);
    }

    #[test]
    fn shipped_mountain_maps_are_current() {
        for (name, map) in maps() {
            let file = std::fs::read(shipped(name)).expect("map present");
            assert_eq!(
                elora_map::decode(&file).expect("valid"),
                map,
                "{name} outdated – write_chapter4_maps -- --ignored"
            );
        }
    }

    #[test]
    #[ignore = "writes maps/adventure/*.emap"]
    fn write_chapter4_maps() {
        for (name, map) in maps() {
            map.save(std::path::Path::new(&shipped(name))).unwrap();
        }
    }

    /// Overview: `… chapter4_sheets -- --ignored` → `target/chapter4-<karte>.svg`.
    #[test]
    #[ignore = "only writes files for visual inspection"]
    fn chapter4_sheets() {
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
                "{}/../../target/chapter4-{name}.svg",
                env!("CARGO_MANIFEST_DIR")
            );
            std::fs::write(path, svg).unwrap();
        }
    }
}

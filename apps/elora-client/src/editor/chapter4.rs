//! Maps of chapter 4 (R2-M2.4): `frost-1` (glacier foot), `frost-2` (abandoned mountain village
//! with Flocke's hut, the frozen cellar and the climbing chimney), `frost-3` (summit ridge in
//! the blizzard) and `frost-arena` (ice hall of the frost spring with Kristella). The mountain
//! trail starts at the top of Tauwinkel's upper village under an ice lid (D-M24-01); the maps
//! are built from left to right.
//!
//! `cargo test -p elora-client --bin elora write_chapter4_maps -- --ignored` writes
//! `maps/abenteuer/*.emap`.

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
    back: &[("tanne-schnee", 6), ("rock-2", 1)],
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
        .retain(|b| b.name == "Wolken" || b.name == "Berge");
    animate(&mut map, &[]);
    map
}

/// Clear crystal (collectible for Klonk, D-M24-09) on the ground of row `ty`.
fn crystal(id: &str, tx: usize, ty: usize) -> Object {
    o(
        id,
        at(tx, ty, 24.0),
        ObjectKind::Collectible {
            item: "klarkristall".into(),
        },
    )
}

/// Fireplace (decoration) with warmth zone `feuer…` (E-342) on the ground of row `ty`.
fn fire(m: &mut Map, id: &str, tx: usize, ty: usize) {
    m.adventure
        .objects
        .push(zone(id, (tx - 3, tx + 3), (ty - 5, ty - 1)));
    m.decor_back.push(big("feuerstelle", tx as f32, ty, 0.8));
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
    hanging(id, "eiszapfen", tx, ty, 48.0)
}

fn bat(id: &str, tx: usize, ty: usize) -> Object {
    hanging(id, "fledermaus", tx, ty, 36.0)
}

fn seal(id: &str, tx: usize, ty: usize) -> Object {
    creature(id, "schneeballrobbe", tx, ty, 28.0)
}

/// Frost spirit `rows` tiles above the ground.
fn ghost(id: &str, tx: usize, ty: usize, rows: usize) -> Object {
    creature(id, "frostgeist", tx, ty - rows, 52.0)
}

/// Gray spots in the ice: traces of the Withering (chapter 4).
fn traces(m: &mut Map, spots: &[(f32, usize)]) {
    for &(tx, ty) in spots {
        m.decor_front.push(big("grauspur-eis", tx, ty, 0.7));
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
        edge_exit("weg-dorf", 0, h, "tauwinkel", "bergsteig"),
        o("west", at(5, 40, 28.0), ObjectKind::Spawn),
        sign("schild-kaelte", 8, 40),
        chest(
            "truhe-munition",
            20,
            38,
            &[("munition_granate", 1), ("munition_laser", 1)],
        ),
        seal("robbe-1", 46, 34),
        icicle("zapfen-1", 67, 34),
        icicle("zapfen-2", 75, 34),
        icicle("zapfen-3", 83, 34),
        crystal("kristall-1", 88, 38),
        sign("schild-eis", 98, 38),
        crystal("kristall-2", 114, 41),
        o("gletscher-rast", at(130, 39, 40.0), ObjectKind::SavePoint),
        plant("blume-1", 134, 39),
        crystal("kristall-3", 143, 29),
        seal("robbe-2", 150, 39),
        npc("bolle", 164, 22, -1, 0.0),
        ghost("geist-1", 185, 37, 4),
        seal("robbe-3", 195, 37),
        zone("lawine-gletscher", (200, 225), (16, 35)),
        zone("lawine-gletscher-tritt", (203, 205), (32, 35)),
        o("ost", at(233, 30, 28.0), ObjectKind::Spawn),
        edge_exit("weg-bergdorf", w - 2, h, "frost-2", "west"),
    ];
    fire(&mut m, "feuer-eingang", 12, 40);
    fire(&mut m, "feuer-rast", 137, 39);
    m.decor_back.extend([
        big("gipfel", 30.0, 36, 1.2),
        big("gletscher", 56.0, 36, 0.9),
        big("schneewehe", 95.0, 38, 0.8),
        big("gipfel", 150.0, 39, 1.4),
        big("gletscher", 190.0, 37, 1.0),
        big("tanne-schnee", 232.0, 30, 0.9),
    ]);
    m.decor_front.extend([
        big("schneewehe", 3.0, 40, 0.5),
        big("schneewehe", 128.0, 39, 0.5),
        big("schneewehe", 226.0, 30, 0.6),
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
        edge_exit("weg-gletscher", 0, h, "frost-1", "ost"),
        o("west", at(4, 38, 28.0), ObjectKind::Spawn),
        seal("robbe-1", 30, 38),
        bat("fledermaus-1", 56, 39),
        bat("fledermaus-2", 64, 39),
        chest("truhe-seil", 72, 44, &[("seil", 1)]),
        o("dorfbrunnen", at(82, 38, 40.0), ObjectKind::SavePoint),
        npc("bolle-huette", 92, 38, 1, 0.0),
        npc("kiesel-huette", 94, 38, 1, 0.0),
        npc("flocke", 97, 38, -1, 0.0),
        npc("wicke-huette", 101, 38, -1, 0.0),
        sign("schild-kamin", 109, 38),
        o("kamin-oben", at(125, 11, 40.0), ObjectKind::SavePoint),
        plant("blume-1", 130, 11),
        ghost("geist-1", 145, 11, 3),
        npc("kiesel", 164, 30, 1, 0.0),
        crystal("kristall-4", 165, 30),
        seal("robbe-2", 185, 11),
        ghost("geist-2", 205, 11, 4),
        crystal("kristall-5", 239, 4),
        o("ost", at(254, 11, 28.0), ObjectKind::Spawn),
        edge_exit("weg-grat", w - 2, h, "frost-3", "west"),
    ];
    fire(&mut m, "feuer-huette", 97, 38);
    fire(&mut m, "feuer-hochebene", 220, 11);
    m.decor_back.extend([
        big("berghuette", 20.0, 38, 0.9),
        big("berghuette", 55.0, 38, 1.3),
        big("berghuette", 97.0, 38, 1.0),
        big("tanne-schnee", 80.0, 38, 1.0),
        big("gipfel", 150.0, 11, 1.2),
        big("gletscher", 195.0, 11, 1.0),
        big("tanne-schnee", 228.0, 11, 0.9),
        big("gipfel", 250.0, 11, 1.0),
    ]);
    m.decor_front.extend([
        decor("faesser", 34.0, 38),
        decor("holzstapel", 86.0, 38),
        big("schneewehe", 106.0, 38, 0.5),
        big("schneewehe", 140.0, 11, 0.6),
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
        edge_exit("weg-bergdorf", 0, h, "frost-2", "ost"),
        o("west", at(4, 20, 28.0), ObjectKind::Spawn),
        sign("schild-lawine", 18, 20),
        zone("lawine-grat-1", (21, 60), (0, 17)),
        zone("lawine-grat-1-tritt", (24, 26), (14, 17)),
        seal("robbe-1", 62, 12),
        npc("wicke", 86, 10, -1, 0.0),
        crystal("kristall-6", 84, 10),
        bat("fledermaus-1", 102, 17),
        bat("fledermaus-2", 106, 17),
        ghost("geist-1", 95, 26, 4),
        npc("graue-stelle", 135, 14, 1, 0.0),
        seal("robbe-2", 150, 14),
        zone("lawine-grat-2", (161, 200), (0, 11)),
        zone("lawine-grat-2-tritt", (163, 165), (8, 11)),
        crystal("kristall-7", 198, 6),
        ghost("geist-2", 210, 10, 4),
        o("vor-der-halle", at(215, 10, 40.0), ObjectKind::SavePoint),
        plant("blume-1", 218, 10),
        crystal("kristall-8", 228, 2),
        o("ost", at(274, 10, 28.0), ObjectKind::Spawn),
        edge_exit("weg-halle", w - 2, h, "frost-arena", "west"),
    ];
    fire(&mut m, "feuer-start", 10, 20);
    fire(&mut m, "feuer-tal", 92, 26);
    fire(&mut m, "feuer-halle", 206, 10);
    m.decor_back.extend([
        big("gipfel", 45.0, 14, 1.3),
        big("gletscher", 90.0, 26, 1.0),
        big("gipfel", 140.0, 14, 1.4),
        big("gipfel", 240.0, 10, 1.2),
        big("tanne-schnee", 260.0, 10, 0.9),
    ]);
    m.decor_back.push(big("grauspur-eis", 135.0, 14, 1.0));
    m.decor_front.extend([
        big("schneewehe", 66.0, 12, 0.6),
        big("schneewehe", 155.0, 14, 0.6),
        big("schneewehe", 250.0, 10, 0.6),
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
        o("vor-der-quelle", at(11, 28, 40.0), ObjectKind::SavePoint),
        chest(
            "truhe-munition",
            15,
            28,
            &[("munition_granate", 1), ("munition_laser", 1)],
        ),
        o(
            "eiskoenigin",
            Vec2::new(43.5 * T, 44.0 * T - 300.0),
            ObjectKind::Creature {
                kind: "kristella".into(),
                persistent: true,
            },
        ),
        npc("kristella", 43, 44, -1, 0.0),
        o(
            "tor",
            corner(61, 40),
            ObjectKind::Door {
                size: (1, 4),
                open_if: "merker besiegt.kristella".into(),
            },
        ),
        o(
            "halle",
            corner(25, 16),
            ObjectKind::Camera {
                size: Vec2::new(37.0 * T, 30.0 * T),
                mode: CameraMode::Bounds,
            },
        ),
        o("ost", at(94, 44, 28.0), ObjectKind::Spawn),
        edge_exit("weg-heim", w - 2, h, "tauwinkel", "bergsteig"),
    ];
    m.decor_back = vec![
        big("frostquelle-verdorrt", 43.0, 44, 1.6),
        big("gipfel", 8.0, 28, 0.9),
        big("tanne-schnee", 20.0, 28, 0.8),
        big("gletscher", 80.0, 44, 1.0),
    ];
    m.decor_front = vec![
        big("schneewehe", 3.0, 28, 0.5),
        big("schneewehe", 90.0, 44, 0.5),
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
            "{}/../../maps/abenteuer/{name}.{}",
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
            let back = elora_map::decode(&elora_map::encode(m)).expect("Karte gültig");
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
            "bolle-huette",
            "kiesel-huette",
            "wicke-huette",
            "graue-stelle",
            "truhe-seil",
            "kristella",
            "eiskoenigin",
            "tor",
        ] {
            assert!(objects.iter().any(|o| o.id == id), "{id} fehlt");
        }
        let crystals = objects
            .iter()
            .filter(
                |o| matches!(&o.kind, ObjectKind::Collectible { item } if item == "klarkristall"),
            )
            .count();
        assert_eq!(crystals, 8, "acht Klarkristalle (D-M24-09)");
        assert!(
            objects.iter().filter(|o| o.id.starts_with("feuer")).count() >= 6,
            "genug Feuerstellen gegen die Kälte"
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
                        assert!(!below.is_solid(), "{name}: dünnes Eis bei {x},{y} auf Fels");
                    }
                    if t == Tile::IceWater {
                        assert!(
                            below == Tile::IceWater || below.is_solid(),
                            "{name}: Eiswasser bei {x},{y} ohne Boden"
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
            panic!("Kristella ist eine Königin");
        };
        let m = frost_arena();
        let (lo, hi) = (HALL_MID - d.width / 2.0, HALL_MID + d.width / 2.0);
        let floor = 44;
        let x0 = (lo / T).floor() as usize;
        let x1 = (hi / T).floor() as usize;
        for x in x0..=x1 {
            assert_eq!(m.tiles[floor * m.width + x], Tile::Ice, "Eisboden bei {x}");
        }
        assert_eq!(
            m.tiles[(floor - 1) * m.width + x0 - 1],
            Tile::Climb,
            "Wand links"
        );
        assert_eq!(
            m.tiles[(floor - 1) * m.width + x1 + 1],
            Tile::Climb,
            "Wand mit Tor rechts"
        );
        let o = m.adventure.object("eiskoenigin").unwrap();
        assert!((o.pos.x - HALL_MID).abs() < 1.0);
    }

    #[test]
    fn shipped_mountain_maps_are_current() {
        for (name, map) in maps() {
            let file = std::fs::read(shipped(name)).expect("Karte vorhanden");
            assert_eq!(
                elora_map::decode(&file).expect("gültig"),
                map,
                "{name} veraltet – write_chapter4_maps -- --ignored"
            );
        }
    }

    #[test]
    #[ignore = "schreibt maps/abenteuer/*.emap"]
    fn write_chapter4_maps() {
        for (name, map) in maps() {
            map.save(std::path::Path::new(&shipped(name))).unwrap();
        }
    }

    /// Overview: `… chapter4_sheets -- --ignored` → `target/chapter4-<karte>.svg`.
    #[test]
    #[ignore = "erzeugt nur Dateien zur Sichtprüfung"]
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

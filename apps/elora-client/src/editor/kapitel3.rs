//! Karten von Kapitel 3 (R2-M2.3): `wueste-1` (Dünenrand), `wueste-2` (Karawanenlager und
//! Oase), `wueste-3` (Ruinen des Glutvolks mit der verschütteten Kammer) und `wueste-arena`
//! (Glutquelle im Sandkessel mit der Sandschlange). Die Wüste liegt südlich von Tauwinkel:
//! Der Hohlweg am Ostpfad führt hinab, die Karten werden von links nach rechts gebaut.
//!
//! `cargo test -p elora-client --bin elora write_kapitel3_maps -- --ignored` schreibt
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
    Grid, T, animate, at, chest, corner, creature, decor, finish, npc, o, plant, pull_vault,
    ruck_gate, sign,
};
use super::release::{self, Theme};

/// Wüste: Thema „Wüste“ der Release-Karten, große Deko passend zum Gebiet.
const DESERT: Theme = Theme {
    back: &[("kaktus", 5), ("rock-2", 1), ("rock-1", 1)],
    front: &["rock-1", "grass-2"],
    front_density: 10,
    back_spacing: 13,
    ..release::THEMES[1]
};

/// Übergang über die ganze Kartenhöhe am Rand (E-279).
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
    creature(id, "sandkrabbe", tx, ty, 30.0)
}

fn worm(id: &str, tx: usize, ty: usize) -> Object {
    creature(id, "duenenwurm", tx, ty, 36.0)
}

/// Funkenmotte `rows` Tiles über dem Boden.
fn moth(id: &str, tx: usize, ty: usize, rows: usize) -> Object {
    creature(id, "funkenmotte", tx, ty - rows, 32.0)
}

/// Treibsand statt Boden in den Spalten `x0..=x1` (eine Reihe, fester Grund darunter, E-318).
fn quicksand(g: &mut Grid, x0: usize, x1: usize, top: usize) {
    g.fill((x0, x1), (top, top), '&');
}

/// Graue Fußspuren des Wanderers (E-325).
fn tracks(m: &mut Map, spots: &[(f32, usize)]) {
    for &(tx, ty) in spots {
        m.decor_front.push(big("grauspur", tx, ty, 2.2));
    }
}

/// Wüsten-Look: Thema „Wüste“, große eigene Deko bleibt hinten.
fn desert(m: Map) -> Map {
    let mut map = finish(m, &DESERT);
    release::apply_look(&DESERT, &mut map);
    animate(&mut map, &[]);
    map
}

/// Glutsandwüste 1, 240 × 60: vom Hohlweg hinab auf den Dünenrand, ein Felsdach als
/// Schatten, erste Treibsandgruben, Sandkrabben und Dünenwürmer.
pub fn wueste_1() -> Map {
    let (w, h) = (240, 60);
    let mut g = Grid::new(w, h, 50);
    // hinab vom Hohlweg
    g.ground(0, 14, 24);
    g.ground(15, 20, 28);
    g.ground(21, 26, 32);
    g.ground(27, 34, 36);
    g.ground(35, 60, 40);
    // Felsdach als Schatten (E-320)
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
    // zweites Felsdach
    g.fill((175, 186), (34, 35), '#');
    g.ground(171, 200, 42);
    g.ground(201, 215, 40);
    g.ground(216, w - 1, 42);
    let mut m = g.map("Glutsandwüste 1");
    m.adventure.objects = vec![
        edge_exit("weg-dorf", 0, h, "tauwinkel", "hohlweg"),
        o("nord", at(6, 24, 28.0), ObjectKind::Spawn),
        sign("schild-hitze", 11, 24),
        chest(
            "truhe-munition",
            13,
            24,
            &[("munition_granate", 1), ("glanztropfen", 10)],
        ),
        crab("krabbe-1", 56, 40),
        sign("schild-treibsand", 68, 44),
        crab("krabbe-2", 85, 44),
        worm("wurm-1", 100, 41),
        o("duenen-rast", at(120, 38, 40.0), ObjectKind::SavePoint),
        plant("blume-1", 126, 38),
        worm("wurm-2", 144, 44),
        crab("krabbe-3", 157, 44),
        worm("wurm-3", 168, 44),
        crab("krabbe-4", 192, 42),
        worm("wurm-4", 208, 40),
        o("ost", at(234, 42, 28.0), ObjectKind::Spawn),
        edge_exit("weg-lager", w - 2, h, "wueste-2", "west"),
    ];
    m.decor_back = vec![
        big("duene", 8.0, 24, 1.2),
        big("felsbogen", 47.0, 40, 1.5),
        big("duene", 70.0, 44, 1.6),
        big("palme", 115.0, 38, 1.0),
        big("duene", 135.0, 41, 1.3),
        big("felsbogen", 180.5, 42, 1.4),
        big("duene", 205.0, 40, 1.5),
        big("palme", 228.0, 42, 0.9),
    ];
    m.decor_front = vec![
        big("kaktus", 30.0, 36, 0.6),
        big("kaktus", 95.0, 41, 0.5),
        big("kaktus", 133.0, 41, 0.6),
        big("kaktus", 196.0, 42, 0.5),
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

/// Glutsandwüste 2, 220 × 50: Karawanenlager mit Sirup (Zeltdächer spenden Schatten) und
/// Palmas Oase in einer Senke mit drei verdorrten Stellen (E-319).
pub fn wueste_2() -> Map {
    let (w, h) = (220, 50);
    let mut g = Grid::new(w, h, 44);
    g.ground(0, 80, 40);
    // Zeltdächer
    g.fill((36, 44), (34, 34), '=');
    g.fill((58, 66), (34, 34), '=');
    g.ground(81, 95, 42);
    // Oase in der Senke
    g.ground(96, 140, 45);
    g.ground(141, 150, 42);
    g.ground(151, w - 1, 40);
    let mut m = g.map("Karawanenlager");
    let bluete = |n: usize, tx: usize| {
        o(
            &format!("bluete-{n}"),
            at(tx, 45, 28.0),
            ObjectKind::Npc {
                character: format!("bluete-{n}"),
                dialog: "oase-bluete".into(),
                facing: 1,
                walk: 0.0,
            },
        )
    };
    m.adventure.objects = vec![
        edge_exit("weg-duenen", 0, h, "wueste-1", "ost"),
        o("west", at(5, 40, 28.0), ObjectKind::Spawn),
        o("lager", at(47, 40, 40.0), ObjectKind::SavePoint),
        npc("sirup", 52, 40, -1, 24.0),
        chest(
            "truhe-munition",
            68,
            40,
            &[("munition_granate", 1), ("glanztropfen", 10)],
        ),
        crab("krabbe-1", 90, 42),
        npc("palma", 104, 45, 1, 0.0),
        npc("giessstelle-1", 100, 45, 1, 0.0),
        bluete(1, 100),
        npc("giessstelle-2", 109, 45, 1, 0.0),
        bluete(2, 109),
        npc("giessstelle-3", 133, 45, 1, 0.0),
        bluete(3, 133),
        zone("oase-1", (96, 140), (33, 44)),
        plant("blume-1", 138, 45),
        moth("motte-1", 160, 40, 9),
        crab("krabbe-2", 175, 40),
        plant("blume-2", 170, 40),
        moth("motte-2", 188, 40, 8),
        crab("krabbe-3", 202, 40),
        o("ost", at(214, 40, 28.0), ObjectKind::Spawn),
        edge_exit("weg-ruinen", w - 2, h, "wueste-3", "west"),
    ];
    m.decor_back = vec![
        big("duene", 12.0, 40, 1.4),
        big("zelt", 40.0, 40, 0.75),
        big("zelt", 62.0, 40, 0.75),
        big("kamel", 72.0, 40, 0.55),
        big("palme", 98.0, 45, 1.1),
        big("oase", 120.0, 45, 1.4),
        big("palme", 128.0, 45, 0.9),
        big("palme", 139.0, 45, 1.2),
        big("duene", 170.0, 40, 1.5),
        big("felsbogen", 205.0, 40, 1.0),
    ];
    m.decor_front = vec![
        decor("faesser", 46.0, 40),
        decor("korb", 57.0, 40),
        big("kaktus", 84.0, 42, 0.5),
        big("kaktus", 158.0, 40, 0.6),
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

/// Glutsandwüste 3, 240 × 70: Ruinen des Glutvolks – Tafeln, Stachelgrube, Treibsand, der
/// Hof mit der Ruinenquelle unter einem Steindach, Zugtruhe, Ruck-Stelle und die
/// verschüttete Kammer unter Bröckelboden (nur mit Stampfen, E-323).
pub fn wueste_3() -> Map {
    let (w, h) = (240, 70);
    let mut g = Grid::new(w, h, 50);
    g.ground(0, 100, 46);
    // erhöhte Ruinenböden
    g.fill((30, 35), (43, 45), '#');
    g.fill((44, 50), (40, 45), '#');
    // Stachelgrube mit Hook-Blüte darüber
    g.fill((61, 66), (46, 46), '^');
    g.fill((63, 63), (36, 36), '*');
    quicksand(&mut g, 92, 97, 46);
    // Hof mit der Ruinenquelle, Steindach als Schatten
    g.ground(101, 130, 49);
    g.fill((101, 103), (46, 48), '#');
    g.fill((128, 130), (46, 48), '#');
    g.fill((101, 130), (38, 39), '%');
    g.ground(131, 239, 46);
    // Zugtruhe (Heranhooken) und Ruck-Stelle (Hook-Ruck)
    let vault = pull_vault(
        &mut g,
        "ruine",
        134,
        46,
        "ruine.zugtor",
        &[("glanztropfen", 45), ("glutstein", 2)],
    );
    let ruck_top = ruck_gate(&mut g, 148, 46);
    // verschüttete Kammer: Bröckelboden (zwei Reihen), darunter ein Raum mit Steg
    g.fill((171, 185), (46, 56), '#');
    g.fill((172, 184), (48, 55), '.');
    g.fill((175, 179), (46, 47), ':');
    g.fill((180, 184), (52, 52), '=');
    // Stachelgrube vor dem Ausgang
    g.fill((205, 209), (46, 46), '^');
    let mut m = g.map("Ruinen des Glutvolks");
    m.adventure.objects = vec![
        edge_exit("weg-lager", 0, h, "wueste-2", "ost"),
        o("west", at(4, 46, 28.0), ObjectKind::Spawn),
        npc("tafel-1", 14, 46, 1, 0.0),
        crab("krabbe-1", 26, 46),
        moth("motte-1", 40, 40, 8),
        worm("wurm-1", 56, 46),
        npc("tafel-2", 76, 46, 1, 0.0),
        chest(
            "truhe-munition",
            58,
            46,
            &[("munition_granate", 1), ("glanztropfen", 10)],
        ),
        crab("krabbe-2", 86, 46),
        o("hof", at(108, 49, 40.0), ObjectKind::SavePoint),
        npc("ruinenquelle", 116, 49, 1, 0.0),
        moth("motte-2", 122, 49, 7),
        chest(
            "truhe-ruck",
            155,
            ruck_top,
            &[("glanztropfen", 50), ("tautrank", 1)],
        ),
        worm("wurm-2", 165, 46),
        sign("schild-kammer", 173, 46),
        zone("ruinenkammer", (172, 184), (48, 55)),
        npc("tafel-kammer", 176, 56, 1, 0.0),
        chest("truhe-kammer", 182, 56, &[("sonnenschleier", 1)]),
        crab("krabbe-3", 194, 46),
        moth("motte-3", 200, 46, 8),
        worm("wurm-3", 216, 46),
        plant("blume-1", 222, 46),
        o("ost", at(234, 46, 28.0), ObjectKind::Spawn),
        edge_exit("weg-kessel", w - 2, h, "wueste-arena", "west"),
    ];
    m.adventure.objects.extend(vault);
    m.decor_back = vec![
        big("ruinentor", 22.0, 46, 0.9),
        big("saeule", 38.0, 46, 0.9),
        big("saeule-bruch", 54.0, 46, 1.0),
        big("saeule", 72.0, 46, 1.0),
        big("saeule-bruch", 84.0, 46, 0.9),
        big("saeule", 105.0, 49, 0.9),
        big("saeule", 126.0, 49, 0.9),
        big("ruinentor", 116.0, 49, 1.0),
        big("saeule-bruch", 145.0, 46, 1.0),
        big("saeule", 168.0, 46, 1.0),
        big("ruinentor", 190.0, 46, 0.9),
        big("saeule-bruch", 200.0, 46, 0.8),
        big("duene", 225.0, 46, 1.3),
    ];
    m.decor_front = vec![
        big("kaktus", 8.0, 46, 0.5),
        big("kaktus", 90.0, 46, 0.5),
        big("kaktus", 228.0, 46, 0.6),
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

/// Glutquelle, 110 × 50: vom Sims hinab in den Sandkessel; der Kessel liegt im Schatten der
/// Felswände (keine Hitze im Kampf). Boden drei Tiles dick (Treibsand der wütenden
/// Sandschlange), zwei Stege zum Ausweichen; Tor nach dem Sieg, Weg zurück nach Tauwinkel.
pub fn wueste_arena() -> Map {
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
        o("vor-der-quelle", at(11, 28, 40.0), ObjectKind::SavePoint),
        chest(
            "truhe-munition",
            14,
            28,
            &[("munition_granate", 1), ("munition_laser", 1)],
        ),
        o(
            "sandschlange",
            at(50, 44, 70.0),
            ObjectKind::Creature {
                kind: "sandschlange".into(),
                persistent: true,
            },
        ),
        npc("schlange", 50, 44, -1, 0.0),
        zone("schatten-kessel", (20, 79), (14, 43)),
        o(
            "tor",
            corner(80, 36),
            ObjectKind::Door {
                size: (2, 8),
                open_if: "merker besiegt.sandschlange".into(),
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
        o("ost", at(104, 44, 28.0), ObjectKind::Spawn),
        edge_exit("weg-heim", w - 2, h, "tauwinkel", "hohlweg"),
    ];
    m.decor_back = vec![
        big("glutquelle-verdorrt", 50.0, 44, 1.6),
        big("palme", 4.0, 28, 1.0),
        big("felsbogen", 11.0, 28, 0.9),
        big("saeule", 24.0, 44, 1.2),
        big("saeule-bruch", 76.0, 44, 1.2),
        big("duene", 95.0, 44, 1.3),
    ];
    m.decor_front = vec![big("kaktus", 15.0, 28, 0.5), big("kaktus", 100.0, 44, 0.5)];
    tracks(&mut m, &[(40.0, 44), (60.0, 44), (90.0, 44), (102.0, 44)]);
    desert(m)
}

/// Karten von Kapitel 3 mit ihren Namen.
pub fn maps() -> Vec<(&'static str, Map)> {
    vec![
        ("wueste-1", wueste_1()),
        ("wueste-2", wueste_2()),
        ("wueste-3", wueste_3()),
        ("wueste-arena", wueste_arena()),
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
    fn desert_maps_match_content_and_link_up() {
        let c = Content::builtin();
        let mut all = super::super::kapitel1::all_maps();
        all.extend(super::super::kapitel2::maps());
        all.extend(maps());
        all.extend(super::super::kapitel4::maps());
        for (name, m) in &all {
            let back = elora_map::decode(&elora_map::encode(m)).expect("Karte gültig");
            let errors = map_objects(&c, &back);
            assert!(errors.is_empty(), "{name}: {errors:?}");
        }
        let refs: Vec<(&str, &Map)> = all.iter().map(|(n, m)| (*n, m)).collect();
        let errors = map_links(&refs);
        assert!(errors.is_empty(), "{errors:?}");
        // Inhalte der Aufgaben stehen auf den Karten
        let desert = maps();
        let objects: Vec<&Object> = desert
            .iter()
            .flat_map(|(_, m)| &m.adventure.objects)
            .collect();
        for id in [
            "sirup",
            "palma",
            "giessstelle-1",
            "giessstelle-2",
            "giessstelle-3",
            "ruinenquelle",
            "tafel-kammer",
            "ruinenkammer",
            "oase-1",
            "sandschlange",
        ] {
            assert!(objects.iter().any(|o| o.id == id), "{id} fehlt");
        }
    }

    /// Treibsand liegt immer auf festem Grund, und der Kessel hat drei Reihen Boden.
    #[test]
    fn quicksand_rests_on_solid_ground() {
        for (name, m) in maps() {
            for y in 0..m.height - 1 {
                for x in 0..m.width {
                    if m.tiles[y * m.width + x] == elora_sim::Tile::Quicksand {
                        assert!(
                            m.tiles[(y + 1) * m.width + x].is_solid(),
                            "{name}: Treibsand bei {x},{y} ohne Grund"
                        );
                    }
                }
            }
        }
        let arena = wueste_arena();
        for y in 44..47 {
            assert!(arena.tiles[y * arena.width + 50].is_solid());
        }
    }

    #[test]
    fn shipped_desert_maps_are_current() {
        for (name, map) in maps() {
            let file = std::fs::read(shipped(name)).expect("Karte vorhanden");
            assert_eq!(
                elora_map::decode(&file).expect("gültig"),
                map,
                "{name} veraltet – write_kapitel3_maps -- --ignored"
            );
        }
    }

    #[test]
    #[ignore = "schreibt maps/abenteuer/*.emap"]
    fn write_kapitel3_maps() {
        for (name, map) in maps() {
            map.save(std::path::Path::new(&shipped(name))).unwrap();
        }
    }

    /// Übersicht: `… kapitel3_sheets -- --ignored` → `target/kapitel3-<karte>.svg`.
    #[test]
    #[ignore = "erzeugt nur Dateien zur Sichtprüfung"]
    fn kapitel3_sheets() {
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
                "{}/../../target/kapitel3-{name}.svg",
                env!("CARGO_MANIFEST_DIR")
            );
            std::fs::write(path, svg).unwrap();
        }
    }
}

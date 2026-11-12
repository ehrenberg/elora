//! Maps of chapter 5 (R2-M2.5, M2.5.7): `gorge-1` (descent at dusk with the first star islands
//! and note stones), `gorge-2` (mirror cave, Luma's hollow and the first gliding chasm),
//! `gorge-3` (the dark heights: long gliding routes between islands and updrafts) and
//! `gorge-spring` (ruins of the Star Spring, climax “Protect Luma”). The way in is a crack in
//! the rocks at the east end of `meadow-3` that opens after chapter 4 (E-367). Night maps:
//! light comes from crystal stars, lanterns and glowing mushrooms (`assets/map/lights.toml`).
//!
//! Characters, collectibles and signs of chapter 5 come from the content (M2.5.6,
//! `docs/release-2/m2-5-content.md`): Glimmer, Luma in her hollow and at the ruins, the three
//! light threads at the ends of star bridges (D-M25-01), eight star fragments only reachable by
//! gliding, a note tablet per map with its verse of Luma's song (D-M25-02) and the grey stones
//! of the Withered One.
//!
//! `cargo test -p elora-client --bin elora write_chapter5_maps -- --ignored` writes
//! `maps/adventure/*.emap`.

// Map generator: `x`/`y`/`w`/`h` (tiles) as in mapgen.rs
#![allow(clippy::many_single_char_names)]

use elora_map::{Map, Object, ObjectKind};
use elora_sim::Vec2;

use super::look::Preset;
use super::mapgen::{
    Grid, T, animate, at, big, chest, collectible, corner, creature, decor, decor_px, edge_exit,
    finish_with, npc, o, plant, sign, zone,
};
use super::release::{self, Theme};

/// Gorge: rocky night theme on the stone material, crystal stars and night rocks in the back.
const GORGE: Theme = Theme {
    material: Some("stone"),
    back: &[("night_rock", 5), ("crystal_star", 4), ("rock-2", 2)],
    front: &["grass-2", "rock-1"],
    front_density: 8,
    back_spacing: 14,
    ..release::THEMES[4]
};

fn night(m: Map) -> Map {
    let mut map = finish_with(m, &GORGE, Preset::Night);
    release::apply_look(&GORGE, &mut map);
    map.backgrounds.retain(|b| b.name != "Forest");
    animate(&mut map, &[]);
    map
}

/// Thorn pit over the columns `x0..=x1`: open from the top, thorns in row `bottom`.
fn pit(g: &mut Grid, x0: usize, x1: usize, bottom: usize) {
    g.fill((x0, x1), (0, bottom - 1), '.');
    g.fill((x0, x1), (bottom, bottom), '^');
}

/// Star island (zone `island…`, M2.5.2) of `w` tiles whose walkable top edge is row `ty`.
fn island(id: &str, x0: usize, ty: usize, w: usize) -> Object {
    zone(id, (x0, x0 + w - 1), (ty, ty))
}

/// Updraft (zone `updraft…`) over the columns `x` from row `top` to `bottom`.
fn updraft(id: &str, x: (usize, usize), top: usize, bottom: usize) -> Object {
    zone(id, x, (top, bottom))
}

/// Note stone (zone `note-<pitch>…`, M2.5.3), 2 × 2 tiles standing on row `ty`.
fn note(id: &str, tx: usize, ty: usize) -> Object {
    zone(id, (tx, tx + 1), (ty - 2, ty - 1))
}

/// Star bridge of a note stone: its walkable top edge is the top of row `ty`, columns
/// `x0..=x1`.
fn bridge(stone: &str, x0: usize, x1: usize, ty: usize) -> Object {
    o(
        &format!("{stone}-bridge"),
        corner(x0, ty),
        ObjectKind::Zone {
            size: Vec2::new((x1 - x0 + 1) as f32 * T, 0.5 * T),
        },
    )
}

/// Star lantern (zone `lantern…`, M2.5.5), 1 × 2 tiles standing on row `ty`.
fn lantern(id: &str, tx: usize, ty: usize) -> Object {
    zone(id, (tx, tx), (ty - 2, ty - 1))
}

/// Character in a second place (`luma-ruins`, …) that speaks the dialog of its first one.
fn npc_with(id: &str, dialog: &str, tx: usize, ty: usize, facing: i8) -> Object {
    o(
        id,
        at(tx, ty, 28.0),
        ObjectKind::Npc {
            character: id.into(),
            dialog: dialog.into(),
            facing,
            walk: 0.0,
        },
    )
}

/// Light thread for Luma (D-M25-01) on the ground of row `ty`.
fn thread(id: &str, tx: usize, ty: usize) -> Object {
    collectible(id, "light_thread", tx, ty)
}

/// Star fragment (side quest “Star fragments”) on the ground of row `ty` (or in the air).
fn fragment(id: &str, tx: usize, ty: usize) -> Object {
    collectible(id, "star_fragment", tx, ty)
}

fn goblin(id: &str, tx: usize, ty: usize) -> Object {
    creature(id, "stone_goblin", tx, ty, 30.0)
}

/// Shadow moths as a small swarm around (`tx`, `rows` tiles above the ground of row `ty`).
fn moths(m: &mut Map, id: &str, tx: usize, ty: usize, rows: usize) {
    for (k, dx) in [0usize, 2, 4].into_iter().enumerate() {
        m.adventure.objects.push(creature(
            &format!("{id}-{k}"),
            "shadow_moth",
            tx + dx,
            ty - rows - k % 2,
            20.0,
        ));
    }
}

fn mirror(id: &str, tx: usize, ty: usize) -> Object {
    creature(id, "echo_mirror", tx, ty, 28.0)
}

/// Spring stone (save point) on row `ty`.
fn rest(id: &str, tx: usize, ty: usize) -> Object {
    o(id, at(tx, ty, 40.0), ObjectKind::SavePoint)
}

/// Grey footprints of the Withered One.
fn tracks(m: &mut Map, spots: &[(f32, usize)]) {
    for &(tx, ty) in spots {
        m.decor_front.push(big("grey_trail", tx, ty, 2.2));
    }
}

/// Light in the dark: crystal stars on the ground of row `ty`.
fn stars(m: &mut Map, spots: &[(f32, usize)]) {
    for &(tx, ty) in spots {
        m.decor_back.push(big("crystal_star", tx, ty, 0.8));
    }
}

/// Descent at dusk, 260 × 64: from the crack down a few steps into the gorge; the first chasm
/// is crossed on star islands (fixed, then drifting), the second on the star bridge of a note
/// stone – a second stone on the far side lights it for the way back. Above the second chasm
/// a high ledge that only a bridge on the top reaches (light thread 1, D-M25-01).
pub fn gorge_1() -> Map {
    let (w, h) = (260, 64);
    let mut g = Grid::new(w, h, 60);
    g.ground(0, 22, 20);
    g.ground(23, 30, 24);
    g.ground(31, 38, 28);
    g.ground(39, 46, 32);
    g.ground(47, 90, 36);
    pit(&mut g, 91, 130, 62);
    g.ground(131, 165, 36);
    // steps up to the high note stone
    g.fill((150, 154), (31, 31), '=');
    g.fill((156, 160), (26, 26), '=');
    g.fill((150, 154), (21, 21), '=');
    g.fill((156, 160), (16, 16), '#');
    pit(&mut g, 166, 212, 62);
    // high ledge above the chasm (only the bridge of the high stone reaches it)
    g.fill((195, 201), (16, 18), '%');
    g.ground(213, w - 1, 36);
    let mut m = g.map("Sternschlucht 1");
    m.adventure.objects = vec![
        edge_exit("path-meadow", 0, h, "meadow-3", "gorge"),
        o("west", at(4, 20, 28.0), ObjectKind::Spawn),
        npc("glimmer", 14, 20, -1, 0.0),
        sign("sign-dark", 20, 20),
        npc("grey-stone-1", 60, 36, 1, 0.0),
        goblin("goblin-1", 56, 36),
        rest("dusk-rest", 72, 36),
        plant("flower-1", 76, 36),
        island("island-fixed-1", 98, 36, 5),
        island("island-drift-1", 108, 35, 5),
        island("island-drift-1-to", 118, 35, 5),
        // in the air above the chasm, 11+ tiles above everything near it (a jump with the air
        // jump reaches about 10): only gliding down from the high steps gets there
        fragment("fragment-1", 128, 24),
        goblin("goblin-2", 140, 36),
        npc("tablet-song-1", 145, 36, 1, 0.0),
        note("note-e-1", 160, 36),
        bridge("note-e-1", 166, 212, 36),
        note("note-c2-1", 158, 16),
        bridge("note-c2-1", 161, 194, 16),
        chest("chest-ledge", 198, 16, &[("gleam_drops", 40)]),
        thread("thread-1", 196, 16),
        note("note-g-1", 215, 36),
        thread("thread-2", 219, 36),
        bridge("note-g-1", 166, 212, 36),
        goblin("goblin-3", 236, 36),
        o("east", at(255, 36, 28.0), ObjectKind::Spawn),
        edge_exit("path-hollow", w - 2, h, "gorge-2", "west"),
    ];
    moths(&mut m, "moths-1", 142, 36, 4);
    stars(
        &mut m,
        &[
            (12.0, 20),
            (64.0, 36),
            (134.0, 36),
            (163.0, 36),
            (222.0, 36),
        ],
    );
    m.decor_back.extend([
        decor("lantern", 82.0, 36),
        big("night_rock", 30.0, 24, 1.2),
        big("night_rock", 248.0, 36, 1.3),
    ]);
    m.decor_front.extend([
        decor("glow_mushrooms", 27.0, 24),
        decor("glow_mushrooms", 43.0, 32),
        decor("glow_mushrooms", 246.0, 36),
    ]);
    tracks(&mut m, &[(8.0, 20), (60.0, 36), (146.0, 36), (230.0, 36)]);
    night(m)
}

/// Luma's hollow, 300 × 70: a mirror cave under a low rock ceiling (two dark mirrors, a
/// lantern whose light makes the echoes fade), Luma's hollow with hanging light threads and a
/// note stone whose bridge leads to an alcove (light thread 2, D-M25-01), then the first chasm
/// that only gliding crosses (60 tiles, an updraft and a sinking island help) and a second one
/// with a drifting island.
pub fn gorge_2() -> Map {
    let (w, h) = (300, 70);
    let mut g = Grid::new(w, h, 66);
    g.ground(0, 40, 40);
    // mirror cave
    g.ground(41, 110, 44);
    g.fill((41, 110), (0, 30), '%');
    // Luma's hollow, closed above; alcove on the west wall
    g.ground(111, 150, 48);
    g.fill((111, 150), (0, 28), '%');
    g.fill((111, 116), (34, 36), '%');
    g.fill((146, 149), (41, 41), '=');
    // cliff top and the gliding chasm
    g.ground(151, 165, 40);
    pit(&mut g, 166, 225, 68);
    g.ground(226, 245, 40);
    pit(&mut g, 246, 285, 68);
    g.ground(286, w - 1, 36);
    let mut m = g.map("Sternschlucht 2");
    m.adventure.objects = vec![
        edge_exit("path-dusk", 0, h, "gorge-1", "east"),
        o("west", at(4, 40, 28.0), ObjectKind::Spawn),
        goblin("goblin-1", 30, 40),
        mirror("mirror-1", 60, 44),
        lantern("lantern-cave", 78, 44),
        mirror("mirror-2", 98, 44),
        rest("hollow", 116, 48),
        plant("flower-1", 120, 48),
        // verse 2 of Luma's song on three stones in her hollow
        note("note-c-2", 122, 48),
        note("note-e-2", 125, 48),
        note("note-g-2", 128, 48),
        npc("luma", 133, 48, -1, 0.0),
        npc("tablet-song-2", 138, 48, -1, 0.0),
        note("note-a-1", 143, 48),
        bridge("note-a-1", 117, 149, 34),
        chest(
            "chest-alcove",
            113,
            34,
            &[("gleam_drops", 30), ("healing_potion", 1)],
        ),
        thread("thread-3", 115, 34),
        sign("sign-glide", 160, 40),
        updraft("updraft-1", (185, 189), 18, 67),
        fragment("fragment-2", 187, 20),
        island("island-sink-1", 205, 41, 5),
        fragment("fragment-3", 207, 41),
        rest("cliff", 232, 40),
        goblin("goblin-2", 240, 40),
        island("island-drift-2", 252, 38, 5),
        island("island-drift-2-to", 272, 38, 5),
        fragment("fragment-4", 265, 30),
        o("east", at(295, 36, 28.0), ObjectKind::Spawn),
        edge_exit("path-heights", w - 2, h, "gorge-3", "west"),
    ];
    moths(&mut m, "moths-1", 262, 38, 9);
    stars(
        &mut m,
        &[
            (18.0, 40),
            (152.0, 40),
            (164.0, 40),
            (229.0, 40),
            (290.0, 36),
        ],
    );
    // light threads hanging from the ceiling of the hollow
    for x in [122.0, 131.0, 140.0] {
        m.decor_back
            .push(decor_px("light_threads", x * T, 29.0 * T));
    }
    m.decor_back.extend([
        decor("lantern", 50.0, 44),
        big("night_rock", 8.0, 40, 1.1),
        big("crystal_star", 136.0, 48, 1.0),
    ]);
    m.decor_front.extend([
        decor("glow_mushrooms", 126.0, 48),
        decor("glow_mushrooms", 158.0, 40),
    ]);
    tracks(&mut m, &[(26.0, 40), (128.0, 48), (238.0, 40)]);
    night(m)
}

/// The dark heights, 320 × 80: rising from west to east. A chasm with islands and an updraft up
/// onto a rock pillar with three note stones (verse 3 of Luma's song); from there a sinking
/// island route or the stones' bridge; a spring stone and a dark mirror; ledges and an updraft
/// up to the last, widest gliding chasm (50 tiles, nothing to hook).
pub fn gorge_3() -> Map {
    let (w, h) = (320, 80);
    let mut g = Grid::new(w, h, 76);
    g.ground(0, 25, 56);
    pit(&mut g, 26, 90, 78);
    g.ground(91, 130, 44);
    pit(&mut g, 131, 175, 78);
    g.ground(176, 230, 40);
    g.fill((207, 212), (35, 35), '=');
    g.fill((214, 219), (30, 30), '=');
    g.ground(231, 250, 24);
    pit(&mut g, 251, 300, 78);
    g.ground(301, w - 1, 24);
    let mut m = g.map("Sternschlucht 3");
    m.adventure.objects = vec![
        edge_exit("path-hollow", 0, h, "gorge-2", "east"),
        o("west", at(4, 56, 28.0), ObjectKind::Spawn),
        island("island-fixed-1", 40, 54, 5),
        island("island-drift-1", 56, 50, 5),
        island("island-drift-1-to", 70, 46, 5),
        updraft("updraft-1", (80, 84), 16, 77),
        fragment("fragment-5", 82, 18),
        goblin("goblin-1", 100, 44),
        npc("tablet-song-3", 107, 44, 1, 0.0),
        note("note-c-3", 112, 44),
        note("note-e-3", 118, 44),
        note("note-g-3", 124, 44),
        bridge("note-g-3", 131, 175, 44),
        island("island-sink-1", 145, 45, 5),
        fragment("fragment-6", 147, 45),
        island("island-sink-2", 160, 43, 5),
        rest("heights-rest", 184, 40),
        npc("grey-stone-2", 191, 40, -1, 0.0),
        plant("flower-1", 188, 40),
        goblin("goblin-2", 194, 40),
        mirror("mirror-1", 202, 40),
        updraft("updraft-2", (222, 226), 8, 39),
        fragment("fragment-7", 224, 9),
        goblin("goblin-3", 244, 24),
        fragment("fragment-8", 275, 14),
        o("east", at(315, 24, 28.0), ObjectKind::Spawn),
        edge_exit("path-ruins", w - 2, h, "gorge-spring", "west"),
    ];
    moths(&mut m, "moths-1", 236, 24, 5);
    stars(
        &mut m,
        &[
            (10.0, 56),
            (96.0, 44),
            (128.0, 44),
            (180.0, 40),
            (233.0, 24),
            (310.0, 24),
        ],
    );
    m.decor_back.extend([
        decor("lantern", 115.0, 44),
        big("night_rock", 20.0, 56, 1.3),
        big("ruin_arch", 305.0, 24, 1.0),
    ]);
    m.decor_front.push(decor("glow_mushrooms", 198.0, 40));
    tracks(&mut m, &[(16.0, 56), (190.0, 40), (308.0, 24)]);
    night(m)
}

/// Ruins of the Star Spring, 120 × 60: from the ledge down into the basin with the dark spring
/// in the middle where Luma sings; four star lanterns (two on the ground, one on a pillar, one on
/// a star island the updraft reaches); grey shadows come from the rock on both sides and from
/// the dark above; the Withered One stands on the east ledge, then the west ledge, then high
/// above the spring. The gate on the east ledge opens once Luma is protected; the way leads home
/// to Tauwinkel.
pub fn gorge_spring() -> Map {
    let (w, h) = (120, 60);
    let mut g = Grid::new(w, h, 56);
    g.ground(0, 17, 30);
    g.ground(18, 22, 36);
    g.ground(23, 97, 46);
    // pillar with a lantern on the west side of the basin
    g.fill((38, 40), (38, 45), '#');
    g.ground(98, w - 1, 30);
    let mut m = g.map("Ruinen der Sternquelle");
    m.adventure.objects = vec![
        edge_exit("path-heights", 0, h, "gorge-3", "east"),
        o("west", at(4, 30, 28.0), ObjectKind::Spawn),
        rest("before-the-ruins", 9, 30),
        plant("flower-1", 12, 30),
        chest(
            "chest-ammo",
            15,
            30,
            &[("ammo_grenade", 1), ("ammo_laser", 1)],
        ),
        // climax “Protect Luma” (M2.5.5)
        // starts once Luma has asked for the lanterns (flag `luma.sings`), anywhere in the basin
        zone("climax-start", (23, 97), (20, 45)),
        zone("climax-luma", (59, 60), (44, 45)),
        npc_with("luma-ruins", "luma", 60, 46, -1),
        zone("shade-spawn-west", (2, 12), (34, 44)),
        zone("shade-spawn-east", (106, 116), (34, 44)),
        zone("shade-spawn-sky", (45, 75), (2, 6)),
        zone("withered-1", (100, 101), (26, 28)),
        zone("withered-2", (17, 18), (26, 28)),
        zone("withered-3", (59, 60), (8, 10)),
        lantern("lantern-1", 30, 46),
        lantern("lantern-2", 39, 38),
        lantern("lantern-3", 89, 46),
        lantern("lantern-4", 77, 32),
        island("island-ruin", 74, 32, 6),
        updraft("updraft-ruin", (81, 84), 22, 45),
        o(
            "gate",
            corner(106, 26),
            ObjectKind::Door {
                size: (1, 4),
                open_if: format!("flag {}", elora_adventure::session::LUMA_PROTECTED),
            },
        ),
        o("east", at(114, 30, 28.0), ObjectKind::Spawn),
        edge_exit("path-home", w - 2, h, "tauwinkel", "east"),
    ];
    m.decor_back = vec![
        big("star-spring-withered", 60.0, 46, 1.3),
        big("ruin_arch", 26.0, 46, 1.1),
        big("ruin_arch", 94.0, 46, 1.1),
        big("crystal_star", 5.0, 30, 0.9),
        big("crystal_star", 116.0, 30, 0.9),
        big("ruin_gate", 106.0, 30, 0.9),
    ];
    m.decor_front = vec![decor("pillar-broken", 50.0, 46), decor("pillar", 70.0, 46)];
    tracks(&mut m, &[(20.0, 36), (104.0, 30)]);
    night(m)
}

/// Maps of chapter 5 with their names.
pub fn maps() -> Vec<(&'static str, Map)> {
    vec![
        ("gorge-1", gorge_1()),
        ("gorge-2", gorge_2()),
        ("gorge-3", gorge_3()),
        ("gorge-spring", gorge_spring()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::mapgen::{ADVENTURE, assert_current, write_maps, write_sheet};
    use elora_adventure::Content;
    use elora_adventure::check::{map_links, map_objects};
    use elora_sim::Tile;

    fn all_linked() -> Vec<(&'static str, Map)> {
        let mut all = super::super::chapter1::all_maps();
        all.extend(super::super::chapter2::maps());
        all.extend(super::super::chapter3::maps());
        all.extend(super::super::chapter4::maps());
        all.extend(maps());
        all
    }

    #[test]
    fn gorge_maps_match_content_and_link_up() {
        let c = Content::builtin();
        let all = all_linked();
        for (name, m) in &all {
            let back = elora_map::decode(&elora_map::encode(m)).expect("map valid");
            let errors = map_objects(&c, &back);
            assert!(errors.is_empty(), "{name}: {errors:?}");
        }
        let refs: Vec<(&str, &Map)> = all.iter().map(|(n, m)| (*n, m)).collect();
        let errors = map_links(&refs);
        assert!(errors.is_empty(), "{errors:?}");
        // the world builds the mechanics from the zones
        let worlds: Vec<_> = maps()
            .into_iter()
            .map(|(n, m)| (n, m.world(elora_sim::Tuning::default())))
            .collect();
        let count = |f: &dyn Fn(&elora_sim::World) -> usize| {
            worlds.iter().map(|(_, w)| f(w)).sum::<usize>()
        };
        assert!(count(&|w| w.islands.len()) >= 8, "star islands");
        assert!(count(&|w| w.note_stones.len()) >= 6, "note stones");
        assert_eq!(
            count(&|w| w.lanterns.len()),
            5,
            "lanterns: one in the cave, four at the ruins"
        );
    }

    /// Every bridge belongs to a stone on the same map, and every chasm of the main way can be
    /// crossed: thorns at the bottom, so nobody gets stuck below.
    #[test]
    fn note_stones_have_their_bridges() {
        for (name, m) in maps() {
            let (stones, bridges) = elora_map::islands::map_notes(&m);
            for (i, _) in bridges.iter().enumerate() {
                assert!(
                    stones.iter().any(|s| s.bridges.contains(&i)),
                    "{name}: bridge {i} without a stone"
                );
            }
        }
    }

    /// The gliding chasms are wider than a jump with the air jump (about 29 tiles) and narrower
    /// than a glide from the same height (about 74 tiles), with nothing to hook in between.
    #[test]
    fn gliding_chasms_need_the_star_cape() {
        for (name, x0, x1, row) in [("gorge-2", 166, 225, 40), ("gorge-3", 251, 300, 24)] {
            let m = maps().into_iter().find(|(n, _)| *n == name).unwrap().1;
            let width = x1 - x0 + 1;
            assert!((35..70).contains(&width), "{name}: {width} tiles");
            for x in x0..=x1 {
                for y in 0..row + 20 {
                    assert_eq!(
                        m.tiles[y * m.width + x],
                        Tile::Air,
                        "{name}: something to hook at {x},{y}"
                    );
                }
            }
        }
    }

    /// The climax at the ruins is complete and its gate opens with the flag.
    #[test]
    fn ruins_hold_the_climax() {
        let m = gorge_spring();
        for id in [
            "climax-start",
            "climax-luma",
            "withered-1",
            "withered-2",
            "withered-3",
            "gate",
        ] {
            assert!(m.adventure.object(id).is_some(), "{id} missing");
        }
        let world = m.world(elora_sim::Tuning::default());
        assert_eq!(world.lanterns.len(), 4);
    }

    /// Gliding spots in chapters 1–4 (M2.5.8): one flap arch per chapter, each in an open,
    /// flat place, so only the strengthened star cape reaches its chest (the arch itself is
    /// checked in `crates/elora-sim/tests/abilities.rs`).
    #[test]
    fn every_chapter_has_a_gliding_spot() {
        use crate::editor::mapgen::flap_spot_clear;
        let mut all = super::super::chapter1::all_maps();
        all.extend(super::super::chapter2::maps());
        all.extend(super::super::chapter3::maps());
        all.extend(super::super::chapter4::maps());
        for (map, id, x0, floor) in [
            ("meadow-3", "meadow3-flap", 64, 30),
            ("forest-3", "forest3-flap", 140, 30),
            ("desert-2", "desert2-flap", 180, 40),
            ("frost-3", "frost3-flap", 127, 14),
        ] {
            let m = &all.iter().find(|(n, _)| *n == map).unwrap().1;
            assert!(
                m.adventure.object(&format!("{id}-chest")).is_some(),
                "{map}: chest"
            );
            if let Err(e) = flap_spot_clear(m, x0, floor) {
                panic!("{map}: {e}");
            }
        }
    }

    /// Finds open, flat places for flap pillars in chapters 1–4 (M2.5.8):
    /// `… flap_spots -- --ignored --nocapture`.
    #[test]
    #[ignore = "only prints candidates"]
    fn flap_spots() {
        use crate::editor::mapgen::flap_spot_clear;
        let mut all = super::super::chapter1::all_maps();
        all.extend(super::super::chapter2::maps());
        all.extend(super::super::chapter3::maps());
        all.extend(super::super::chapter4::maps());
        for (name, m) in &all {
            let mut runs: Vec<(usize, usize, usize)> = Vec::new();
            for x in 0..m.width {
                // ground: first solid tile from the top
                let Some(floor) = (1..m.height).find(|&y| m.tiles[y * m.width + x].is_solid())
                else {
                    continue;
                };
                if floor > 11 && flap_spot_clear(m, x, floor).is_ok() {
                    match runs.last_mut() {
                        Some((_, x1, f)) if *x1 + 1 == x && *f == floor => *x1 = x,
                        _ => runs.push((x, x, floor)),
                    }
                }
            }
            println!("{name}: {runs:?}");
        }
    }

    #[test]
    fn shipped_gorge_maps_are_current() {
        assert_current(ADVENTURE, maps(), "write_chapter5_maps");
    }

    #[test]
    #[ignore = "writes maps/adventure/*.emap"]
    fn write_chapter5_maps() {
        write_maps(ADVENTURE, maps());
    }

    /// Overview: `… chapter5_sheets -- --ignored` → `target/chapter5-<map>.svg`.
    #[test]
    #[ignore = "only writes files for visual inspection"]
    fn chapter5_sheets() {
        for (name, map) in maps() {
            write_sheet(map, 2400.0, &format!("chapter5-{name}"));
        }
    }
}

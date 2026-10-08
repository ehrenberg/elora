//! Maps of the prologue (A1.9, `docs/release-2/prolog.md`): Tauwinkel and Blütenwiesen 1 –
//! terrain, adventure objects and decoration (buildings E-278, faded flowers E-277).
//!
//! `cargo test -p elora-client --bin elora write_prologue_maps -- --ignored` writes
//! `maps/adventure/*.emap`.

#![allow(
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::too_many_lines
)]

use std::time::Instant;

use elora_map::look::{Curve, EnvKind, EnvPoint, EnvRef, Envelope};
use elora_map::{Art, Decor, Map, Object, ObjectKind};
use elora_sim::{TILE_SIZE, Vec2};

use super::Editor;
use super::look::Preset;
use super::release::{self, Theme};

pub(super) const T: f32 = TILE_SIZE as f32;

/// Character grid of the map; ground from `floor` downwards, open at the sides (E-279).
pub(super) struct Grid(pub(super) Vec<Vec<char>>);

impl Grid {
    pub(super) fn new(w: usize, h: usize, floor: usize) -> Self {
        let mut g = vec![vec!['.'; w]; h];
        for (y, row) in g.iter_mut().enumerate() {
            for c in row.iter_mut() {
                if y >= floor {
                    *c = '#';
                }
            }
        }
        Self(g)
    }

    /// Fill a rectangle (columns `x`, rows `y`, each inclusive).
    pub(super) fn fill(&mut self, x: (usize, usize), y: (usize, usize), c: char) {
        for row in &mut self.0[y.0..=y.1] {
            for cell in &mut row[x.0..=x.1] {
                *cell = c;
            }
        }
    }

    /// Set the ground of columns `x0..=x1` at row `top` (air above).
    pub(super) fn ground(&mut self, x0: usize, x1: usize, top: usize) {
        for (y, row) in self.0.iter_mut().enumerate() {
            for cell in &mut row[x0..=x1] {
                *cell = if y >= top { '#' } else { '.' };
            }
        }
    }

    /// Map from the grid; the `S` is only needed by the text format, the map uses entrances.
    pub(super) fn map(&self, name: &str) -> Map {
        let mut g = self.0.clone();
        g[0][1] = 'S';
        let rows: Vec<String> = g.iter().map(|r| r.iter().collect()).collect();
        let r: Vec<&str> = rows.iter().map(String::as_str).collect();
        let mut m = Map::from_rows(name, &r).expect("layout valid");
        m.entities.clear();
        m
    }
}

/// Hook jerk spot (M2.1.5): stone wall on the left, shaft 5 tiles wide,
/// hook flower 12 tiles above the ground on the right side, on the right a stone tower 7 tiles wide
/// whose top edge lies 9 tiles above the flower – only reachable with the hook jerk. Checked in
/// `crates/elora-sim/tests/abilities.rs` (`jerk_gate_needs_the_hook_jerk`). At the bottom a
/// passage (3 tiles high) leads through wall and tower so the path stays free. Returns the row
/// of the tower's top edge.
pub(super) fn jerk_gate(g: &mut Grid, x0: usize, floor: usize) -> usize {
    let top = floor - 21;
    g.fill((x0, x0), (top, floor - 4), '%');
    g.fill((x0 + 1, x0 + 5), (top, floor - 1), '.');
    g.fill((x0 + 5, x0 + 5), (floor - 12, floor - 12), '*');
    g.fill((x0 + 6, x0 + 12), (top, floor - 4), '%');
    top
}

/// Pull chest (R2-M2.2, M2.2.6): small stone hut (columns `x0..x0+4`) with a gate on the left and
/// a chest inside; 10 tiles above the ground hangs a root with a pull switch (`merker <flag>`).
/// Can only be opened with the pull hook. Returns switch, gate and chest.
pub(super) fn pull_vault(
    g: &mut Grid,
    id: &str,
    x0: usize,
    floor: usize,
    flag: &str,
    contents: &[(&str, u32)],
) -> Vec<Object> {
    g.fill((x0, x0 + 4), (floor - 4, floor - 4), '%');
    g.fill((x0 + 4, x0 + 4), (floor - 4, floor - 1), '%');
    g.fill((x0 + 1, x0 + 3), (floor - 3, floor - 1), '.');
    g.fill((x0 + 2, x0 + 2), (floor - 12, floor - 11), '#');
    vec![
        o(
            &format!("{id}-pull"),
            Vec2::new((x0 as f32 + 2.5) * T, (floor - 10) as f32 * T + 14.0),
            ObjectKind::Switch {
                flag: flag.into(),
                once: true,
                trigger: elora_map::adventure::SwitchTrigger::Hook,
            },
        ),
        o(
            &format!("{id}-gate"),
            corner(x0, floor - 3),
            ObjectKind::Door {
                size: (1, 3),
                open_if: format!("flag {flag}"),
            },
        ),
        chest(&format!("{id}-chest"), x0 + 2, floor, contents),
    ]
}

/// Stomp chamber (R2-M2.3, M2.3.6): crumbling floor (three tiles wide, two rows) above a
/// small chamber (five tiles wide, three high) with a chest – can only be opened with stomp. From
/// the chamber floor to the surface there are five rows, a jump leads out again.
/// `x0` is the left wall, `floor` the top edge of the ground. Returns the chest `<id>-truhe`.
pub(super) fn stomp_vault(
    g: &mut Grid,
    id: &str,
    x0: usize,
    floor: usize,
    contents: &[(&str, u32)],
) -> Object {
    g.fill((x0, x0 + 6), (floor, floor + 5), '#');
    g.fill((x0 + 1, x0 + 5), (floor + 2, floor + 4), '.');
    g.fill((x0 + 2, x0 + 4), (floor, floor + 1), ':');
    chest(&format!("{id}-chest"), x0 + 5, floor + 5, contents)
}

/// Climbing spot (R2-M2.4, M2.4.7): a hanging chimney of two climbing walls (columns `x0`
/// and `x0 + 4`, three tiles of air between them) that end four rows above the ground – you walk
/// through underneath, a jump reaches the walls. At the top right an unhookable
/// ledge 19 rows above the ground with a chest: only reachable with the ice grip.
/// `floor` is the top edge of the ground. Returns the chest `<id>-truhe`.
pub(super) fn climb_vault(
    g: &mut Grid,
    id: &str,
    x0: usize,
    floor: usize,
    contents: &[(&str, u32)],
) -> Object {
    g.fill((x0, x0), (floor - 18, floor - 5), '|');
    g.fill((x0 + 4, x0 + 4), (floor - 18, floor - 5), '|');
    g.fill((x0 + 5, x0 + 8), (floor - 19, floor - 19), '%');
    chest(&format!("{id}-chest"), x0 + 7, floor - 19, contents)
}

/// Center above the ground (top edge of row `ty`) in column `tx` for an object of height `h`.
pub(super) fn at(tx: usize, ty: usize, h: f32) -> Vec2 {
    Vec2::new(tx as f32 * T + T / 2.0, ty as f32 * T - h / 2.0 - 1.0)
}

pub(super) fn corner(tx: usize, ty: usize) -> Vec2 {
    Vec2::new(tx as f32 * T, ty as f32 * T)
}

pub(super) fn o(id: &str, pos: Vec2, kind: ObjectKind) -> Object {
    Object {
        id: id.into(),
        pos,
        kind,
    }
}

pub(super) fn npc(id: &str, tx: usize, ty: usize, facing: i8, walk: f32) -> Object {
    o(
        id,
        at(tx, ty, 28.0),
        ObjectKind::Npc {
            character: id.into(),
            dialog: id.into(),
            facing,
            walk,
        },
    )
}

/// Signpost with a hint (E-273).
pub(super) fn sign(dialog: &str, tx: usize, ty: usize) -> Object {
    o(
        dialog,
        at(tx, ty, 28.0),
        ObjectKind::Npc {
            character: "signpost".into(),
            dialog: dialog.into(),
            facing: 1,
            walk: 0.0,
        },
    )
}

pub(super) fn creature(id: &str, kind: &str, tx: usize, ty: usize, h: f32) -> Object {
    o(
        id,
        at(tx, ty, h),
        ObjectKind::Creature {
            kind: kind.into(),
            persistent: false,
        },
    )
}

pub(super) fn chest(id: &str, tx: usize, ty: usize, contents: &[(&str, u32)]) -> Object {
    o(
        id,
        at(tx, ty, 26.0),
        ObjectKind::Chest {
            contents: contents.iter().map(|(i, n)| ((*i).into(), *n)).collect(),
            lock: String::new(),
        },
    )
}

pub(super) fn plant(id: &str, tx: usize, ty: usize) -> Object {
    o(id, at(tx, ty, 16.0), ObjectKind::HealPlant { heal: 2 })
}

/// Decoration on the ground (top edge of row `ty`), centered in column `tx` (half tiles allowed).
pub(super) fn decor(name: &str, tx: f32, ty: usize) -> Decor {
    Decor::new(
        Art::Builtin(name.into()),
        Vec2::new(tx * T + T / 2.0, ty as f32 * T),
    )
}

/// Decoration at a world position (pixels), e.g. on roofs and benches.
pub(super) fn decor_px(name: &str, x: f32, y: f32) -> Decor {
    Decor::new(Art::Builtin(name.into()), Vec2::new(x, y))
}

/// Append an animation; returns its index.
fn envelope(map: &mut Map, name: &str, kind: EnvKind, points: &[(u32, [f32; 4], Curve)]) -> u16 {
    map.envelopes.push(Envelope {
        name: name.into(),
        kind,
        synced: false,
        points: points
            .iter()
            .map(|&(time_ms, value, curve)| EnvPoint {
                time_ms,
                value,
                curve,
            })
            .collect(),
    });
    u16::try_from(map.envelopes.len() - 1).expect("few animations")
}

/// Small movements (detail): butterflies flutter, smoke rises, flags wave.
pub(super) fn animate(map: &mut Map, smoke: &[Vec2]) {
    use Curve::{Linear, Smooth};
    let flutter = envelope(
        map,
        "Flattern",
        EnvKind::Position,
        &[
            (0, [0.0, 0.0, 0.0, 0.0], Smooth),
            (1500, [30.0, -24.0, 8.0, 0.0], Smooth),
            (3000, [64.0, -4.0, -6.0, 0.0], Smooth),
            (4500, [30.0, 18.0, 6.0, 0.0], Smooth),
            (6000, [0.0, 0.0, 0.0, 0.0], Smooth),
        ],
    );
    let wave = envelope(
        map,
        "Wehen",
        EnvKind::Position,
        &[
            (0, [0.0, 0.0, -1.5, 0.0], Smooth),
            (1800, [0.0, 0.0, 1.5, 0.0], Smooth),
            (3600, [0.0, 0.0, -1.5, 0.0], Smooth),
        ],
    );
    let rise = envelope(
        map,
        "Rauch",
        EnvKind::Position,
        &[
            (0, [0.0, 0.0, 0.0, 0.0], Linear),
            (3000, [14.0, -90.0, 0.0, 0.0], Linear),
        ],
    );
    let fade = envelope(
        map,
        "Rauch verblasst",
        EnvKind::Color,
        &[
            (0, [1.0, 1.0, 1.0, 0.0], Linear),
            (400, [1.0, 1.0, 1.0, 0.9], Linear),
            (3000, [1.0, 1.0, 1.0, 0.0], Linear),
        ],
    );
    // glowing mushrooms pulse (R2-M2.2)
    let pulse = envelope(
        map,
        "Leuchten der Pilze",
        EnvKind::Color,
        &[
            (0, [1.0, 1.0, 1.0, 1.0], Smooth),
            (1300, [0.82, 0.92, 1.0, 0.85], Smooth),
            (2600, [1.0, 1.0, 1.0, 1.0], Smooth),
        ],
    );
    let mut k = 0;
    for d in map.decor_front.iter_mut().chain(map.decor_back.iter_mut()) {
        let Art::Builtin(name) = &d.art else {
            continue;
        };
        k += 1;
        let offset_ms = (k * 977) % 6000;
        match name.as_str() {
            "butterfly" => {
                d.pos_env = Some(EnvRef {
                    index: flutter,
                    offset_ms,
                });
            }
            "glow_mushrooms" => {
                d.color_env = Some(EnvRef {
                    index: pulse,
                    offset_ms: offset_ms % 2600,
                });
            }
            n if n.starts_with("banner-") => {
                d.pos_env = Some(EnvRef {
                    index: wave,
                    offset_ms,
                });
            }
            _ => {}
        }
    }
    for (i, &at) in smoke.iter().enumerate() {
        for puff in 0..2 {
            let offset_ms = i32::try_from(i * 700 + puff * 1500).unwrap_or(0);
            let mut d = decor_px("smoke", at.x, at.y);
            d.pos_env = Some(EnvRef {
                index: rise,
                offset_ms,
            });
            d.color_env = Some(EnvRef {
                index: fade,
                offset_ms,
            });
            map.decor_back.push(d);
        }
    }
}

/// Grass in front of the playfield (Tauwinkel: no colorful flowers, E-210).
const GRASS: Theme = Theme {
    file: "",
    name: "",
    rows: &[],
    material: None,
    preset: Preset::Day,
    sky: None,
    background_tint: None,
    no_forest: false,
    back: &[("bush-1", 2)],
    front: &["grass-1", "grass-2"],
    decor_tint: None,
    front_density: 25,
    back_spacing: 1000,
};

pub(super) fn finish(map: Map, theme: &Theme) -> Map {
    let mut editor = Editor::new(None, std::path::PathBuf::from("maps"));
    let (back, front) = (map.decor_back.clone(), map.decor_front.clone());
    editor.map = map;
    editor.map.author = Some("Elora-Team".into());
    editor.map.decor_back.clear();
    editor.map.decor_front.clear();
    editor.apply_preset(Preset::Day, Instant::now());
    release::place(theme, &mut editor.map);
    // large decoration only on continuous ground (not on hook rocks and ledges)
    let auto = std::mem::take(&mut editor.map.decor_back);
    let map = &editor.map;
    let grounded = |d: &Decor| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let (tx, ty) = ((d.pos.x / T) as usize, (d.pos.y / T) as usize);
        (ty..map.height).all(|y| map.tiles[y * map.width + tx] != elora_sim::Tile::Air)
    };
    let back: Vec<Decor> = back
        .into_iter()
        .chain(auto.into_iter().filter(|d| grounded(d)))
        .collect();
    editor.map.decor_back = back;
    // custom decoration first (behind the automatic one)
    editor.map.decor_front.splice(0..0, front);
    editor.map
}

/// Tauwinkel, 450 × 50 (E-280): steep slope in the west, Elora's garden with tree house, hedge,
/// ledge, well square, workshop and Tüftel's yard (E-281), climb to the upper village with smithy,
/// straw dummies and shop, east path down to the open transition (E-279).
pub fn tauwinkel() -> Map {
    let (w, h) = (450, 50);
    let mut g = Grid::new(w, h, 40);
    // west: slope of earth and roots up into the Murmelwald (E-306), the transition at the top
    g.fill((0, 3), (13, h - 1), '#');
    g.fill((4, 6), (21, h - 1), '#');
    g.fill((7, 8), (29, h - 1), '#');
    g.fill((9, 10), (35, h - 1), '#');
    // roots to hold on to
    g.fill((12, 12), (24, 24), '*');
    g.fill((8, 8), (16, 16), '*');
    // tree house: platform 5 tiles above the ground
    g.fill((40, 44), (35, 35), '=');
    // hedge: only with a double jump
    g.fill((54, 56), (33, 39), '#');
    // meadow path two tiles higher, hollow with a ledge (down to drop through)
    g.ground(67, 180, 38);
    g.fill((75, 85), (38, 40), '.');
    g.fill((75, 85), (38, 38), '=');
    // Tüftel's yard: pit under a ceiling, rock arch with stone and earth overhang,
    // crumbling bridge, block and high seat under an overhang
    g.ground(181, 252, 38);
    // hook jerk route behind the workshop: the gate opens once Tüftel has built the hook jerk
    let jerk_top = jerk_gate(&mut g, 170, 38);
    // Tüftel's pull chest: practice for the pull hook (M2.2.6)
    let pull = pull_vault(
        &mut g,
        "yard",
        238,
        38,
        "yard.pull_gate",
        &[("gleam_drops", 50), ("dew_potion", 1)],
    );
    // stomp plate (M2.3.6): practice for stomp, Tüftel builds it after chapter 3
    let stomp = stomp_vault(
        &mut g,
        "stomp",
        228,
        38,
        &[("gleam_drops", 30), ("healing_potion", 1)],
    );
    g.fill((186, 203), (38, 41), '.');
    g.fill((185, 204), (29, 30), '#');
    g.fill((211, 221), (26, 34), '#');
    g.fill((205, 212), (15, 16), '%');
    g.fill((213, 225), (15, 16), '#');
    g.fill((222, 229), (26, 26), ':');
    g.fill((230, 236), (26, 27), '#');
    g.fill((241, 247), (15, 15), '#');
    g.fill((236, 251), (5, 6), '#');
    // climb to the upper village and east path down
    g.ground(253, 256, 36);
    g.ground(257, 260, 34);
    g.ground(261, 350, 32);
    g.ground(351, 356, 34);
    g.ground(357, 362, 36);
    g.ground(363, w - 1, 38);
    g.ground(404, 417, 36);
    // mountain trail into the Frostspitzen (D-M24-01): an ice lid of crumbling floor over a
    // passage, can only be opened with stomp; the passage leads under the upper village to the
    // transition
    g.fill((342, 350), (34, 37), '.');
    g.fill((343, 345), (32, 33), ':');
    // sunken path at the east path down into the Glutsandwüste (E-315): steps, the transition at
    // the bottom; at the top a rock lid until chapter 3 begins
    g.fill((388, 394), (38, 47), '.');
    g.fill((388, 388), (41, 47), '#');
    g.fill((389, 389), (44, 47), '#');
    let mut m = g.map("Tauwinkel");
    m.adventure.objects = vec![
        o(
            "path-forest",
            corner(0, 0),
            ObjectKind::Exit {
                size: Vec2::new(2.0 * T, 13.0 * T),
                map: "forest-1".into(),
                spawn: "east".into(),
                on_touch: true,
            },
        ),
        o("west", at(3, 13, 28.0), ObjectKind::Spawn),
        sign("sign-west-slope", 14, 40),
        o("start", at(23, 40, 28.0), ObjectKind::Spawn),
        sign("sign-start", 26, 40),
        npc("pip", 37, 40, -1, 0.0),
        chest("chest-treehouse", 41, 35, &[("gleam_drops", 10)]),
        sign("sign-hedge", 51, 40),
        sign("sign-platform", 72, 38),
        chest("chest-1", 83, 41, &[("gleam_drops", 20)]),
        sign("sign-well", 102, 38),
        o("well", at(110, 38, 40.0), ObjectKind::SavePoint),
        npc("oma", 123, 38, -1, 0.0),
        npc("tueftel", 158, 38, 1, 32.0),
        sign("sign-hook", 184, 38),
        sign("sign-jerk", 172, 38),
        sign("sign-pull", 236, 38),
        sign("sign-stomp", 226, 38),
        stomp,
        chest(
            "chest-jerk",
            179,
            jerk_top,
            &[("gleam_drops", 40), ("dew_potion", 1)],
        ),
        chest(
            "chest-hook",
            244,
            15,
            &[("gleam_drops", 25), ("healing_potion", 1)],
        ),
        npc("klonk", 281, 32, 1, 0.0),
        sign("sign-hammer", 285, 32),
        creature("dummy-1", "straw_dummy", 290, 32, 40.0),
        creature("dummy-2", "straw_dummy", 295, 32, 40.0),
        creature("dummy-3", "straw_dummy", 300, 32, 40.0),
        npc("lotte", 321, 32, -1, 0.0),
        o("mountain_path", at(339, 32, 28.0), ObjectKind::Spawn),
        sign("sign-mountain-path", 341, 32),
        o(
            "path-mountains",
            corner(347, 34),
            ObjectKind::Exit {
                size: Vec2::new(4.0 * T, 4.0 * T),
                map: "frost-1".into(),
                spawn: "west".into(),
                on_touch: true,
            },
        ),
        o("sunken_path", at(384, 38, 28.0), ObjectKind::Spawn),
        sign("sign-desert", 386, 38),
        o(
            "sunken-path-lid",
            corner(388, 38),
            ObjectKind::Door {
                size: (7, 1),
                open_if: "not quest glutsand new".into(),
            },
        ),
        o(
            "path-desert",
            corner(390, 44),
            ObjectKind::Exit {
                size: Vec2::new(5.0 * T, 4.0 * T),
                map: "desert-1".into(),
                spawn: "north".into(),
                on_touch: true,
            },
        ),
        sign("sign-east-path", 398, 38),
        o("east", at(436, 38, 28.0), ObjectKind::Spawn),
        o(
            "path-meadow",
            corner(w - 3, 0),
            ObjectKind::Exit {
                size: Vec2::new(3.0 * T, h as f32 * T),
                map: "meadow-1".into(),
                spawn: "west".into(),
                on_touch: true,
            },
        ),
    ];
    m.adventure.objects.extend(pull);
    let mut treehouse = decor("treehouse", 42.0, 40);
    treehouse.pos.y += 8.0;
    m.decor_back = vec![
        decor("tree-pine", 5.0, 21),
        decor("tree-round", 8.0, 29),
        decor("tree-round", 11.0, 40),
        decor("house-elora", 18.0, 40),
        decor("clothesline", 31.0, 40),
        treehouse,
        decor("fence", 48.0, 40),
        decor("tree-round", 62.0, 40),
        decor("fence", 69.0, 38),
        decor("bench", 90.0, 38),
        decor("lantern", 93.0, 38),
        decor("notice_board", 99.0, 38),
        decor("banner-pale", 106.0, 38),
        decor("well", 115.0, 38),
        decor("house-oma", 132.0, 38),
        decor("bench", 140.0, 38),
        decor("banner-pale", 144.0, 38),
        decor("lantern", 148.0, 38),
        decor("workshop", 166.0, 38),
        decor("woodpile", 148.0, 38),
        decor("barrels", 151.5, 38),
        decor("tree-round", 208.0, 38),
        decor("tree-pine", 255.0, 36),
        decor("smithy", 272.0, 32),
        decor("woodpile", 309.0, 32),
        decor("shop", 328.0, 32),
        decor("barrels", 334.0, 32),
        decor("bench", 348.0, 32),
        decor("lantern", 350.0, 32),
        decor("tree-round", 368.0, 38),
        decor("tree-pine", 384.0, 38),
        decor("banner-pale", 401.0, 38),
        decor("tree-round", 411.0, 36),
        decor("tree-pine", 422.0, 38),
        decor("tree-round", 444.0, 38),
    ];
    m.decor_front = vec![
        decor("bush-2", 9.0, 40),
        decor("flower-bed-pale", 14.0, 40),
        decor("window-box-pale", 21.5, 40),
        decor("herb-bed-pale", 28.0, 40),
        decor("bush-1", 55.0, 33),
        decor("hay_bale", 79.0, 41),
        decor("flower-bed-pale", 113.0, 38),
        decor("flower-bed-pale", 119.0, 38),
        decor("herb-bed-pale", 128.5, 38),
        decor("herb-bed-pale", 135.5, 38),
        decor("wall", 151.0, 38),
        decor("hay_bale", 191.0, 42),
        decor("hay_bale", 198.0, 42),
        decor("hay_bale", 306.0, 32),
        decor("window-box-pale", 325.5, 32),
        decor("flower-bed-pale", 332.0, 32),
        decor("wall", 374.0, 38),
        decor("rock-2", 380.0, 38),
        decor("bush-2", 395.0, 38),
        decor("hay_bale", 407.0, 36),
        decor("rock-1", 415.0, 36),
        decor("bush-1", 419.0, 38),
        decor("fence", 427.0, 38),
        decor("fence", 429.0, 38),
        decor("bush-1", 430.0, 38),
        decor("rock-2", 440.0, 38),
    ];
    // hedge of bushes in front of the block
    for k in 0..6 {
        let y = 40.0 * T - k as f32 * 38.0;
        let x = 55.0 * T + T / 2.0 + if k % 2 == 0 { -12.0 } else { 12.0 };
        let mut d = Decor::new(Art::Builtin("bush-2".into()), Vec2::new(x, y));
        d.flip_x = k % 2 == 1;
        m.decor_front.push(d);
    }
    let ground = |tx: f32, ty: usize| Vec2::new(tx * T + T / 2.0, ty as f32 * T);
    m.decor_front.extend([
        decor("flower-pot-pale", 13.0, 40),
        decor("watering_can", 15.0, 40),
        decor("mailbox", 24.5, 40),
        decor("pumpkins", 35.0, 40),
        decor("birdhouse", 47.0, 40),
        decor("cat", 43.0, 35),
        decor("berry_bush", 64.5, 40),
        decor("stepping_stones", 96.0, 38),
        decor("basket", 126.0, 38),
        decor("flower-pot-pale", 130.5, 38),
        decor("flower-pot-pale", 133.5, 38),
        decor_px("cat", ground(140.0, 38).x, ground(140.0, 38).y - 28.0),
        decor("stepping_stones", 160.0, 38),
        decor("watering_can", 170.0, 38),
        decor("tree_stump", 207.0, 38),
        decor("fern", 224.0, 38),
        decor_px("bird", ground(244.0, 15).x + 40.0, ground(244.0, 15).y),
        decor("pumpkins", 264.0, 32),
        decor("basket", 324.0, 32),
        decor("flower-pot-pale", 331.0, 32),
        decor("mailbox", 318.0, 32),
        decor("tree_stump", 376.0, 38),
        decor("fern", 386.0, 38),
        decor("dandelion", 397.0, 38),
        decor("berry_bush", 414.0, 36),
        decor_px("bird", ground(428.0, 38).x, ground(428.0, 38).y - 28.0),
        decor("butterfly", 395.0, 36),
        decor("butterfly", 420.0, 35),
        decor("dandelion", 433.0, 38),
    ]);
    // festive decorations after chapter 1 (E-301): only visible with flag `fest`
    for tx in [104.0, 119.0, 141.0, 286.0, 312.0] {
        m.decor_back
            .push(decor("garland-party", tx, if tx > 250.0 { 32 } else { 38 }));
    }
    // lanterns hang on the garlands (string at −120, line there at about −133)
    for (k, tx) in [104.0, 119.0, 141.0, 286.0, 312.0].into_iter().enumerate() {
        let g = ground(tx, if tx > 250.0 { 32 } else { 38 });
        for (side, name) in [
            (-40.0, "party-lantern-party"),
            (40.0, "party-lantern-yellow-party"),
        ] {
            let flip = if k % 2 == 0 { side } else { -side };
            m.decor_back.push(decor_px(name, g.x + flip, g.y - 13.0));
        }
    }
    // birds on the roofs
    for (tx, ty, dy) in [(18.0, 40, 246.0), (115.0, 38, 178.0)] {
        let p = ground(tx, ty);
        m.decor_back.push(decor_px("bird", p.x - 10.0, p.y - dy));
    }
    let chimneys = [
        ground(18.0, 40) + Vec2::new(56.0, -240.0),
        ground(166.0, 38) + Vec2::new(-42.0, -258.0),
        ground(328.0, 32) + Vec2::new(-40.0, -246.0),
    ];
    // ice block above the mountain trail (does not disappear by itself: the lid is tiles)
    m.decor_front.push(decor("ice_block", 344.0, 34));
    let mut map = finish(m, &GRASS);
    animate(&mut map, &chimneys);
    map
}

/// Blütenwiesen 1, 300 × 60 (E-282): hills, valley with a thorn pit and a hook ceiling above,
/// bridge over a thorn gorge (spike beetles), hill ridge with pollen blower, crumbling floor,
/// hook rocks to the high plateau with the glitter stone, spring stone at the meadow edge.
pub fn meadow() -> Map {
    let (w, h) = (300, 60);
    let mut g = Grid::new(w, h, 44);
    // hills
    g.ground(22, 29, 42);
    g.ground(30, 38, 39);
    g.ground(39, 47, 41);
    // valley with a thorn pit; above a ceiling for hooking and a ledge with a chest
    g.ground(60, 95, 50);
    g.fill((72, 78), (50, 52), '.');
    g.fill((72, 78), (53, 53), '^');
    g.ground(96, 99, 47);
    g.fill((62, 69), (32, 33), '#');
    g.fill((72, 80), (32, 33), '#');
    g.fill((84, 92), (38, 39), '#');
    // thorn gorge with a bridge
    g.ground(112, 134, 55);
    g.fill((113, 133), (54, 54), '^');
    g.fill((112, 134), (44, 44), '=');
    // hill ridge, behind it a valley with crumbling floor over thorns
    g.ground(150, 160, 40);
    g.ground(161, 170, 36);
    g.ground(171, 186, 32);
    g.ground(187, 230, 46);
    g.fill((196, 203), (46, 46), ':');
    g.fill((196, 203), (47, 49), '.');
    g.fill((196, 203), (50, 50), '^');
    // hook rocks over to the high plateau (with a passage underneath)
    g.fill((189, 192), (22, 23), '#');
    g.fill((197, 200), (22, 23), '#');
    g.fill((205, 208), (22, 23), '#');
    g.fill((215, 230), (30, 40), '#');
    // ascent to the meadow edge, dense forest on the slope in the east
    g.ground(241, 250, 43);
    g.ground(251, w - 1, 40);
    // hook jerk spot with bee 5 (return after chapter 1)
    let jerk_top = jerk_gate(&mut g, 252, 40);
    // pull chest for the return with the pull hook (M2.2.6)
    let pull = pull_vault(
        &mut g,
        "meadow1",
        241,
        43,
        "meadow1.pull",
        &[("gleam_drops", 45), ("amber", 2)],
    );
    let mut m = g.map("Blütenwiesen 1");
    m.adventure.objects = vec![
        o(
            "path-village",
            corner(0, 0),
            ObjectKind::Exit {
                size: Vec2::new(2.0 * T, h as f32 * T),
                map: "tauwinkel".into(),
                spawn: "east".into(),
                on_touch: true,
            },
        ),
        o("west", at(6, 44, 28.0), ObjectKind::Spawn),
        sign("sign-meadow", 10, 44),
        plant("flower-1", 34, 39),
        creature("hopper-1", "grasshopper", 43, 41, 28.0),
        creature("beetle-1", "spike_beetle", 53, 44, 26.0),
        creature("beetle-2", "spike_beetle", 66, 50, 26.0),
        plant("flower-2", 83, 50),
        creature("hopper-2", "grasshopper", 89, 50, 28.0),
        chest(
            "chest-top-1",
            88,
            38,
            &[("gleam_drops", 30), ("healing_potion", 1)],
        ),
        o(
            "bridge",
            corner(108, 34),
            ObjectKind::Zone {
                size: Vec2::new(30.0 * T, 10.0 * T),
            },
        ),
        creature("beetle-3", "spike_beetle", 117, 44, 26.0),
        creature("beetle-4", "spike_beetle", 128, 44, 26.0),
        creature("beetle-5", "spike_beetle", 140, 44, 26.0),
        plant("flower-3", 146, 44),
        creature("blower", "pollen_blower", 180, 32, 60.0),
        creature("hopper-3", "grasshopper", 209, 46, 28.0),
        o(
            "glitter_stone",
            at(224, 30, 28.0),
            ObjectKind::Collectible {
                item: "glitter_stone".into(),
            },
        ),
        chest("chest-top-2", 228, 30, &[("gleam_drops", 40), ("amber", 2)]),
        plant("flower-4", 236, 46),
        creature("beetle-6", "spike_beetle", 237, 44, 26.0),
        creature("hopper-4", "grasshopper", 270, 40, 28.0),
        o(
            "meadow_edge",
            corner(266, 30),
            ObjectKind::Zone {
                size: Vec2::new(28.0 * T, 10.0 * T),
            },
        ),
        o("spring_stone", at(280, 40, 40.0), ObjectKind::SavePoint),
        // on into the Blütenwiesen (chapter 1)
        o("east", at(292, 40, 28.0), ObjectKind::Spawn),
        o(
            "bee-5",
            at(261, jerk_top, 24.0),
            ObjectKind::Collectible { item: "bee".into() },
        ),
        o(
            "path-meadow-2",
            corner(w - 2, 0),
            ObjectKind::Exit {
                size: Vec2::new(2.0 * T, h as f32 * T),
                map: "meadow-2".into(),
                spawn: "west".into(),
                on_touch: true,
            },
        ),
    ];
    m.adventure.objects.extend(pull);
    m.decor_back = vec![
        decor("tree-round", 15.0, 44),
        decor("tree-round", 64.0, 50),
        decor("tree-round", 104.0, 44),
        decor("tree-round", 165.0, 36),
        decor("tree-round", 239.0, 46),
        decor("tree-pine", 286.0, 40),
        decor("giant-flower-pink", 296.0, 40),
    ];
    m.decor_front = vec![
        decor("bush-2", 222.5, 30),
        decor("bush-1", 225.5, 30),
        decor("bush-2", 289.0, 40),
    ];
    for (y, x0, x1) in [(54, 72, 78), (55, 113, 133), (51, 196, 203)] {
        let mut x = x0;
        while x < x1 {
            m.decor_front.push(decor("thorns", x as f32 + 0.5, y));
            x += 2;
        }
    }
    m.decor_front.extend([
        decor("fern", 18.0, 44),
        decor("dandelion", 26.0, 42),
        decor("tree_stump", 45.0, 41),
        decor("berry_bush", 57.0, 44),
        decor("fern", 62.0, 50),
        decor("dandelion", 86.0, 50),
        decor("fern", 92.0, 50),
        decor("stepping_stones", 102.0, 44),
        decor("berry_bush", 138.0, 44),
        decor("dandelion", 155.0, 40),
        decor("tree_stump", 168.0, 36),
        decor("fern", 176.0, 32),
        decor("berry_bush", 190.0, 46),
        decor("dandelion", 212.0, 46),
        decor("fern", 219.0, 30),
        decor("tree_stump", 233.0, 46),
        decor("berry_bush", 248.0, 43),
        decor("dandelion", 268.0, 40),
        decor("fern", 274.0, 40),
        decor("berry_bush", 287.0, 40),
    ]);
    for (tx, ty) in [
        (14, 41),
        (33, 37),
        (70, 47),
        (90, 47),
        (122, 41),
        (146, 41),
        (176, 29),
        (206, 43),
        (226, 27),
        (252, 37),
        (270, 37),
        (284, 36),
    ] {
        m.decor_front.push(decor("butterfly", tx as f32, ty));
    }
    let mut map = finish(m, &release::THEMES[0]);
    animate(&mut map, &[]);
    map
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
    fn prologue_maps_match_content_and_link_up() {
        let c = Content::builtin();
        let (a, b) = (tauwinkel(), meadow());
        for m in [&a, &b] {
            let back = elora_map::decode(&elora_map::encode(m)).expect("map valid");
            let errors = map_objects(&c, &back);
            assert!(errors.is_empty(), "{}: {errors:?}", m.name);
            assert!(!back.decor_back.is_empty() && !back.backgrounds.is_empty());
        }
        // transitions are checked by chapter1::tests across all adventure maps
        let _ = map_links;
        assert!(a.adventure.object(&c.progression.start_spawn).is_some());
        // zones and enemies of the main quest
        assert!(b.adventure.object("meadow_edge").is_some());
        let beetle = b
            .adventure
            .objects
            .iter()
            .filter(
                |o| matches!(&o.kind, ObjectKind::Creature { kind, .. } if kind == "spike_beetle"),
            )
            .count();
        assert!(beetle >= 3);
    }

    #[test]
    fn shipped_prologue_maps_are_current() {
        for (name, map) in [("tauwinkel", tauwinkel()), ("meadow-1", meadow())] {
            let file = std::fs::read(shipped(name)).expect("map present");
            assert_eq!(
                elora_map::decode(&file).expect("valid"),
                map,
                "{name} outdated – write_prologue_maps -- --ignored"
            );
        }
    }

    #[test]
    #[ignore = "writes maps/adventure/*.emap"]
    fn write_prologue_maps() {
        for (name, map) in [("tauwinkel", tauwinkel()), ("meadow-1", meadow())] {
            map.save(std::path::Path::new(&shipped(name))).unwrap();
        }
    }

    /// Overview: `… prologue_sheets -- --ignored` → `target/prolog-<karte>.svg`.
    #[test]
    #[ignore = "only writes files for visual inspection"]
    fn prologue_sheets() {
        use crate::editor::panel::Preview;
        use crate::editor::view;
        for (name, map) in [("tauwinkel", tauwinkel()), ("meadow-1", meadow())] {
            let mut editor = Editor::new(None, std::path::PathBuf::from("maps"));
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
                "{}/../../target/prolog-{name}.svg",
                env!("CARGO_MANIFEST_DIR")
            );
            std::fs::write(path, svg).unwrap();
        }
    }
}

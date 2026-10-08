//! Shared helpers of the map generators (R2-RF, RF-27): prologue, chapters 1–4, training map,
//! weather test map and release maps all build their maps with these.
//!
//! - [`Grid`]: character grid in the text format of [`Map::from_rows`] (`#` ground, `.` air,
//!   `%` unhookable, `|` climbing wall, `:` crumbling floor, `*` hook flower, `^` thorns, …).
//! - Ability spots carved into the grid: [`jerk_gate`], [`pull_vault`], [`stomp_vault`],
//!   [`climb_vault`], [`chimney`].
//! - Positions in tiles: [`at`] (on the ground), [`corner`] (top left corner of a tile).
//! - Adventure objects: [`o`], [`npc`], [`sign`], [`creature`], [`chest`], [`plant`],
//!   [`collectible`], [`edge_exit`], [`zone`].
//! - Decoration: [`decor`], [`decor_px`], [`big`], [`thorn_decor`], [`animate`].
//! - Finishing: [`finish`] (author, preset, automatic decoration of a theme) and [`mirror`].
//! - Test scaffolding for the shipped maps: [`shipped`], [`assert_current`], [`write_maps`],
//!   [`write_sheet`].
//!
//! # Mirrored forest maps
//!
//! The forest of chapter 2 lies *west* of Tauwinkel, but its maps are written like all other
//! maps: from left (the side towards the village) to right (deeper into the forest), with
//! coordinates counted from the left edge. Before finishing, `chapter2::forest` calls [`mirror`]
//! on the raw map, so the shipped map runs from right (village) to left (forest):
//!
//! - every tile row is reversed (column `x` becomes `w - 1 - x`);
//! - objects: `pos.x` becomes `w·T - pos.x - width`, where `width` is the area width of exits,
//!   zones and doors (positioned at their left edge) and `0` for point objects (positioned at
//!   their center); NPCs and signposts turn around (`facing` negated);
//! - custom decoration (back and front) is mirrored at the map center and `flip_x` toggled.
//!
//! Mirroring happens before [`finish`], so the automatic theme decoration and the animations
//! are placed on the already mirrored terrain and are not flipped.

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

/// Collectible (bee, rune, crystal, …) on the ground of row `ty`.
pub(super) fn collectible(id: &str, item: &str, tx: usize, ty: usize) -> Object {
    o(
        id,
        at(tx, ty, 24.0),
        ObjectKind::Collectible { item: item.into() },
    )
}

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

/// Zone over the tiles `x.0..=x.1` × `y.0..=y.1`.
pub(super) fn zone(id: &str, x: (usize, usize), y: (usize, usize)) -> Object {
    o(
        id,
        corner(x.0, y.0),
        ObjectKind::Zone {
            size: Vec2::new((x.1 - x.0 + 1) as f32 * T, (y.1 - y.0 + 1) as f32 * T),
        },
    )
}

/// Decoration in a different size (giant flowers, spring, fireplace, …).
pub(super) fn big(name: &str, tx: f32, ty: usize, scale: f32) -> Decor {
    let mut d = decor(name, tx, ty);
    d.scale = scale;
    d
}

/// Thorn vines above a row of thorns (row `y` = top edge of the thorns).
pub(super) fn thorn_decor(m: &mut Map, y: usize, x0: usize, x1: usize) {
    let mut x = x0;
    while x < x1 {
        m.decor_front.push(decor("thorns", x as f32 + 0.5, y + 1));
        x += 2;
    }
}

/// Climbing chimney as on the abilities test map: two climbing walls in the columns `x0` and
/// `x0 + 4` (three tiles of air between them) from row `top` to `bottom`.
pub(super) fn chimney(g: &mut Grid, x0: usize, top: usize, bottom: usize) {
    g.fill((x0, x0), (top, bottom), '|');
    g.fill((x0 + 1, x0 + 3), (top, bottom), '.');
    g.fill((x0 + 4, x0 + 4), (top, bottom), '|');
}

/// Mirror horizontally: tiles, objects (areas at their left edge) and decoration
/// (see the module docs).
pub(super) fn mirror(m: &mut Map) {
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

/// Finish a generated map: author, preset „Day“ and the automatic decoration of `theme` (large
/// decoration only on continuous ground); the map's own decoration stays behind the automatic one.
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
        #[allow(clippy::cast_sign_loss)] // decoration lies inside the map
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

/// Directory of the adventure maps below `maps/`.
pub(super) const ADVENTURE: &str = "adventure/";

/// Path of a shipped map, relative to `maps/` and without extension (e.g. `adventure/meadow-2`).
pub(super) fn shipped(name: &str) -> String {
    format!(
        "{}/../../maps/{name}.{}",
        env!("CARGO_MANIFEST_DIR"),
        elora_map::EXTENSION
    )
}

/// The generated maps equal the shipped files `maps/<dir><name>.emap` (`dir` is empty or ends
/// with `/`); `writer` names the ignored test that rewrites them.
///
/// # Panics
/// If a map file is missing, invalid or outdated.
pub(super) fn assert_current<'a>(
    dir: &str,
    maps: impl IntoIterator<Item = (&'a str, Map)>,
    writer: &str,
) {
    for (name, map) in maps {
        let file = std::fs::read(shipped(&format!("{dir}{name}"))).expect("map present");
        assert!(
            elora_map::decode(&file).expect("valid") == map,
            "{dir}{name} outdated – {writer} -- --ignored"
        );
    }
}

/// Write the maps to `maps/<dir><name>.emap` (after changes to a generator).
///
/// # Panics
/// If a file cannot be written.
pub(super) fn write_maps<'a>(dir: &str, maps: impl IntoIterator<Item = (&'a str, Map)>) {
    for (name, map) in maps {
        map.save(std::path::Path::new(&shipped(&format!("{dir}{name}"))))
            .unwrap();
    }
}

/// Overview for visual inspection: the whole map `width` pixels wide as `target/<file>.svg`.
///
/// # Panics
/// If the file cannot be written.
pub(super) fn write_sheet(map: Map, width: f32, file: &str) {
    use super::panel::Preview;
    use super::view;
    let mut editor = Editor::new(None, std::path::PathBuf::from("maps"));
    editor.map = map;
    editor.center_view();
    editor.visible.grid = false;
    let size = Vec2::new(editor.map.width as f32, editor.map.height as f32) * TILE_SIZE as f32;
    let window = Vec2::new(width, width * size.y / size.x);
    editor.zoom = size.x / window.x;
    // camera roughly at ground level as in the game, so that the parallax layers are right
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
    let path = format!("{}/../../target/{file}.svg", env!("CARGO_MANIFEST_DIR"));
    std::fs::write(path, svg).unwrap();
}

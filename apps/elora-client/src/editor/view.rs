//! Editor-Ansicht: Karte (wie im Spiel), Raster, Kartenrand, Entities und Pinsel-Vorschau.

// Zeichen-Code: kurze Namen für Ecken und Koordinaten
#![allow(clippy::many_single_char_names)]

use elora_map::EntityKind;
use elora_render::{Camera, Color, ShapeBatch};
use elora_sim::{PickupKind, TILE_SIZE, Team, Vec2, Weapon};

use super::Editor;
use super::panel::{BRUSHES, Preview};
use super::tools::Cells;
use crate::items::ItemArt;
use crate::map_view::{LookTime, MapView};

const GRID: Color = Color::rgba(1.0, 1.0, 1.0, 0.12);
const GRID_MAJOR: Color = Color::rgba(1.0, 1.0, 1.0, 0.25);
const BORDER: Color = Color::hex(0xf2c14e);
const HOVER: Color = Color::rgba(1.0, 1.0, 1.0, 0.6);
const SELECTION: Color = Color::hex(0x5aaee8);
const SELECTION_FILL: Color = Color::rgba(0.35, 0.68, 0.91, 0.15);
const SPAWN: Color = Color::rgb(1.0, 1.0, 1.0);
const DUMMY: Color = Color::hex(0x9aa6b2);
/// Hintergrund außerhalb der Karte (dunkel, passend zu egui dunkel).
pub const OUTSIDE: Color = Color::hex(0x1b1d22);

/// Kamera so, dass `editor.center` in der Mitte der Kartenfläche `area` (Pixel) liegt.
pub fn camera(editor: &Editor, window: Vec2, area_center: Vec2) -> Camera {
    Camera {
        center: editor.center + (window * 0.5 - area_center) * editor.zoom,
        size: window * editor.zoom,
    }
}

/// Weltpunkt unter einem Bildschirmpunkt (Pixel).
pub fn to_world(camera: &Camera, window: Vec2, pixel: Vec2) -> Vec2 {
    camera.screen_to_world(pixel, window)
}

fn team_color(team: Team) -> Color {
    crate::draw::team_color(team)
}

/// Alles zeichnen; `preview` = was das Werkzeug unter der Maus zeigt.
pub fn draw(
    batch: &mut ShapeBatch,
    editor: &Editor,
    map_view: &mut MapView,
    items: &ItemArt,
    camera: &Camera,
    time: f32,
    preview: Preview,
) {
    let map = &editor.map;
    let ts = TILE_SIZE as f32;
    #[allow(clippy::cast_precision_loss)]
    let size = Vec2::new(map.width as f32 * ts, map.height as f32 * ts);
    let tl = camera.top_left();
    batch.fill_rect(tl, tl + camera.size, OUTSIDE);
    #[allow(clippy::cast_possible_truncation)]
    let look_time = LookTime {
        local_ms: (f64::from(time) * 1000.0) as i64,
        server_ms: (f64::from(time) * 1000.0) as i64,
        hook_wilt: None,
        wind: 0.0,
    };
    // Himmel nur innerhalb der Karte; draußen bleibt es dunkel
    batch.fill_rect_vgradient(
        Vec2::ZERO,
        size,
        crate::map_art::rgba(map.sky.top),
        crate::map_art::rgba(map.sky.bottom),
    );
    let shown = editor.visible.layers;
    let behind = crate::map_view::Layers {
        sky: false,
        terrain: false,
        ..shown
    };
    map_view.draw_back_layers(batch, map, camera, look_time, behind);
    // Hintergründe reichen über die Karte hinaus: außen wieder abdecken
    let (a, b) = (
        tl - Vec2::new(1.0, 1.0),
        tl + camera.size + Vec2::new(1.0, 1.0),
    );
    batch.fill_rect(a, Vec2::new(b.x, 0.0), OUTSIDE);
    batch.fill_rect(Vec2::new(a.x, size.y), b, OUTSIDE);
    batch.fill_rect(Vec2::new(a.x, 0.0), Vec2::new(0.0, size.y), OUTSIDE);
    batch.fill_rect(Vec2::new(size.x, 0.0), Vec2::new(b.x, size.y), OUTSIDE);
    if shown.terrain {
        let terrain = crate::map_view::Layers {
            sky: false,
            backgrounds: false,
            decor_back: false,
            terrain: true,
            decor_front: false,
        };
        map_view.draw_back_layers(batch, map, camera, look_time, terrain);
        map_view.draw_quicksand(batch, map, camera, look_time);
    }
    if editor.visible.entities {
        entities(batch, editor, items, time);
    }
    if editor.visible.layers.decor_front {
        map_view.draw_decor_front(batch, map, camera, look_time);
    }
    if editor.tool == super::tools::Tool::Decor
        && let Some(r) = editor.selected_decor
        && let Some(d) = editor.decor(r)
        && let Some(bounds) = map_view.decor_bounds(map, d)
        && let Some(at) = editor.decor_world_pos(r, camera.center, camera.center)
    {
        let c = super::look::corners(d, at, bounds);
        batch.stroke_polyline(
            &[c[0], c[1], c[2], c[3], c[0]],
            2.0 * editor.zoom,
            SELECTION,
        );
    }
    if editor.visible.grid {
        grid(batch, camera, size, editor.zoom);
    }
    // Kartenrand
    let w = 2.0 * editor.zoom;
    batch.stroke_polyline(
        &[
            Vec2::ZERO,
            Vec2::new(size.x, 0.0),
            size,
            Vec2::new(0.0, size.y),
            Vec2::ZERO,
        ],
        w,
        BORDER,
    );
    if editor.tool == super::tools::Tool::Select
        && let Some(sel) = editor.selection
    {
        let (min, max) = cell_rect(sel);
        batch.fill_rect(min, max, SELECTION_FILL);
        outline(batch, min, max, 2.0 * editor.zoom, SELECTION);
    }
    match preview {
        Preview::None => {}
        Preview::Cells(cells) => {
            let (min, max) = cell_rect(cells);
            outline(batch, min, max, 1.5 * editor.zoom, HOVER);
        }
        Preview::Stamp(x, y) => {
            if let Some(clip) = &editor.clipboard {
                stamp(batch, clip, x, y, editor.zoom);
            }
        }
    }
}

/// Weltrechteck eines Tile-Bereichs.
fn cell_rect(c: Cells) -> (Vec2, Vec2) {
    let ts = TILE_SIZE as f32;
    #[allow(clippy::cast_precision_loss)]
    (
        Vec2::new(c.x0 as f32 * ts, c.y0 as f32 * ts),
        Vec2::new((c.x1 + 1) as f32 * ts, (c.y1 + 1) as f32 * ts),
    )
}

fn outline(batch: &mut ShapeBatch, min: Vec2, max: Vec2, width: f32, color: Color) {
    batch.stroke_polyline(
        &[
            min,
            Vec2::new(max.x, min.y),
            max,
            Vec2::new(min.x, max.y),
            min,
        ],
        width,
        color,
    );
}

/// Vorschau beim Einfügen: Tiles halbdurchsichtig in ihrer Pinselfarbe.
fn stamp(batch: &mut ShapeBatch, clip: &super::tools::Clip, x: usize, y: usize, zoom: f32) {
    let ts = TILE_SIZE as f32;
    for cy in 0..clip.height {
        for cx in 0..clip.width {
            let tile = clip.tiles[cy * clip.width + cx];
            let Some(&(_, _, hex)) = BRUSHES.iter().find(|(t, _, _)| *t == tile) else {
                continue;
            };
            if tile == elora_sim::Tile::Air {
                continue;
            }
            let mut c = Color::hex(hex);
            c.0[3] = 0.55;
            #[allow(clippy::cast_precision_loss)]
            let min = Vec2::new((x + cx) as f32 * ts, (y + cy) as f32 * ts);
            batch.fill_rect(min, min + Vec2::new(ts, ts), c);
        }
    }
    for &(_, ex, ey) in &clip.entities {
        #[allow(clippy::cast_precision_loss)]
        let p = Vec2::new(
            (x + ex) as f32 * ts + ts / 2.0,
            (y + ey) as f32 * ts + ts / 2.0,
        );
        batch.stroke_circle(p, 10.0, 2.0, SPAWN);
    }
    let (min, max) = cell_rect(Cells {
        x0: x,
        y0: y,
        x1: x + clip.width - 1,
        y1: y + clip.height - 1,
    });
    outline(batch, min, max, 2.0 * zoom, SELECTION);
}

fn entities(batch: &mut ShapeBatch, editor: &Editor, items: &ItemArt, time: f32) {
    for e in &editor.map.entities {
        let p = e.pos();
        match e.kind {
            EntityKind::Spawn => spawn(batch, p, SPAWN),
            EntityKind::SpawnRed => spawn(batch, p, team_color(Team::Red)),
            EntityKind::SpawnBlue => spawn(batch, p, team_color(Team::Blue)),
            EntityKind::FlagRed => {
                items.draw_flag(batch, p + Vec2::new(0.0, 16.0), team_color(Team::Red), time);
            }
            EntityKind::FlagBlue => items.draw_flag(
                batch,
                p + Vec2::new(0.0, 16.0),
                team_color(Team::Blue),
                time,
            ),
            EntityKind::Health => items.draw_pickup(batch, p, PickupKind::Health, time),
            EntityKind::Armor => items.draw_pickup(batch, p, PickupKind::Armor, time),
            EntityKind::Laser => {
                items.draw_pickup(batch, p, PickupKind::Weapon(Weapon::Laser), time);
            }
            EntityKind::Grenade => {
                items.draw_pickup(batch, p, PickupKind::Weapon(Weapon::Grenade), time);
            }
            EntityKind::Dummy(_) => {
                batch.fill_circle(p, 13.0, DUMMY);
                batch.stroke_circle(p, 13.0, 2.0, Color::hex(0x2b2b2b));
            }
        }
    }
}

/// Spawnpunkt: Ring mit Pfeil nach unten (dort steht die Figur).
fn spawn(batch: &mut ShapeBatch, p: Vec2, color: Color) {
    batch.stroke_circle(p, 12.0, 3.0, color);
    batch.fill_polygon(
        &[
            p + Vec2::new(-5.0, -2.0),
            p + Vec2::new(5.0, -2.0),
            p + Vec2::new(0.0, 5.0),
        ],
        color,
    );
}

/// Raster je Tile, alle 8 Tiles kräftiger; bei starkem Herauszoomen nur das grobe Raster.
fn grid(batch: &mut ShapeBatch, camera: &Camera, size: Vec2, zoom: f32) {
    let ts = TILE_SIZE as f32;
    let a = camera.top_left();
    let b = a + camera.size;
    let tl = Vec2::new(a.x.max(0.0), a.y.max(0.0));
    let br = Vec2::new(b.x.min(size.x), b.y.min(size.y));
    if br.x <= tl.x || br.y <= tl.y {
        return;
    }
    let fine = zoom < 2.5;
    let w = zoom;
    #[allow(clippy::cast_possible_truncation)]
    let (x0, x1, y0, y1) = (
        (tl.x / ts).floor() as i32,
        (br.x / ts).ceil() as i32,
        (tl.y / ts).floor() as i32,
        (br.y / ts).ceil() as i32,
    );
    for x in x0..=x1 {
        let major = x % 8 == 0;
        if fine || major {
            let fx = x as f32 * ts;
            batch.stroke_line(
                Vec2::new(fx, tl.y),
                Vec2::new(fx, br.y),
                w,
                if major { GRID_MAJOR } else { GRID },
            );
        }
    }
    for y in y0..=y1 {
        let major = y % 8 == 0;
        if fine || major {
            let fy = y as f32 * ts;
            batch.stroke_line(
                Vec2::new(tl.x, fy),
                Vec2::new(br.x, fy),
                w,
                if major { GRID_MAJOR } else { GRID },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vorlage „Tag“ auf einer neuen Karte mit Boden: `… preset_sheet -- --ignored` → `target/preset.svg`.
    #[test]
    #[ignore = "erzeugt nur eine Datei zur Sichtprüfung"]
    fn preset_sheet() {
        let mut editor = Editor::new(None, std::path::PathBuf::from("maps"));
        let t = std::time::Instant::now();
        let (w, h) = (editor.map.width, editor.map.height);
        editor.fill_cells(
            super::super::tools::Cells::span((0, h - 2), (w - 1, h - 1)),
            elora_sim::Tile::Solid,
            "boden",
            t,
        );
        editor.apply_preset(super::super::look::Preset::Day, t);
        editor.zoom = 0.75;
        // tiefste Kamera (Karte 30 Tiles hoch): früher klaffte hier eine Lücke über dem Boden
        editor.center.y = 960.0 - 337.0;
        let window = Vec2::new(1600.0, 900.0);
        let cam = camera(&editor, window, window * 0.5);
        let mut batch = ShapeBatch::default();
        draw(
            &mut batch,
            &editor,
            &mut MapView::default(),
            &ItemArt::load(),
            &cam,
            0.0,
            Preview::None,
        );
        let tl = cam.top_left();
        let svg = batch.debug_svg(tl, tl + cam.size, OUTSIDE);
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/preset.svg"),
            svg,
        )
        .unwrap();
    }

    /// Editor-Ansicht ohne Oberfläche: `cargo test -p elora-client --bin elora editor_sheet -- --ignored`,
    /// danach `cargo xtask svg-preview target/editor.svg target/editor.png 1400`.
    #[test]
    #[ignore = "erzeugt nur eine Datei zur Sichtprüfung"]
    fn editor_sheet() {
        let mut editor = Editor::new(None, std::path::PathBuf::from("maps"));
        editor.map = elora_map::decode(include_bytes!("../../../../maps/look-test.emap")).unwrap();
        editor.center_view();
        editor.zoom = 1.6;
        let window = Vec2::new(1600.0, 900.0);
        let cam = camera(&editor, window, Vec2::new(650.0, 450.0));
        let mut batch = ShapeBatch::default();
        draw(
            &mut batch,
            &editor,
            &mut MapView::default(),
            &ItemArt::load(),
            &cam,
            0.5,
            Preview::Cells(Cells::span((10, 17), (12, 18))),
        );
        let tl = cam.top_left();
        let svg = batch.debug_svg(tl, tl + cam.size, OUTSIDE);
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/editor.svg"),
            svg,
        )
        .unwrap();
    }
}

//! Editor-Ansicht: Karte (wie im Spiel), Raster, Kartenrand, Entities und Pinsel-Vorschau.

use elora_map::EntityKind;
use elora_render::{Camera, Color, ShapeBatch};
use elora_sim::{PickupKind, TILE_SIZE, Team, Vec2, Weapon};

use super::Editor;
use crate::items::ItemArt;
use crate::map_view::{LookTime, MapView};

const GRID: Color = Color::rgba(1.0, 1.0, 1.0, 0.12);
const GRID_MAJOR: Color = Color::rgba(1.0, 1.0, 1.0, 0.25);
const BORDER: Color = Color::hex(0xf2c14e);
const HOVER: Color = Color::rgba(1.0, 1.0, 1.0, 0.35);
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

/// Alles zeichnen; `hover` = Tile unter der Maus (Pinsel-Vorschau).
pub fn draw(
    batch: &mut ShapeBatch,
    editor: &Editor,
    map_view: &mut MapView,
    items: &ItemArt,
    camera: &Camera,
    time: f32,
    hover: Option<(usize, usize)>,
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
    }
    if editor.visible.entities {
        entities(batch, editor, items, time);
    }
    if editor.visible.layers.decor_front {
        map_view.draw_front(batch, map, camera, look_time);
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
    if let Some((x, y)) = hover {
        #[allow(clippy::cast_precision_loss)]
        let min = Vec2::new(x as f32 * ts, y as f32 * ts);
        let max = min + Vec2::new(ts, ts);
        batch.stroke_polyline(
            &[
                min,
                Vec2::new(max.x, min.y),
                max,
                Vec2::new(min.x, max.y),
                min,
            ],
            1.5 * editor.zoom,
            HOVER,
        );
    }
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
            Some((10, 17)),
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

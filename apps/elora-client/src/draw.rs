//! Platzhalter-Darstellung für die Sandbox (E-043): einfache Vektorformen.

use elora_map::EntityKind;
use elora_render::{Camera, Color, ShapeBatch};
use elora_sim::{HookState, PHYS_SIZE, TILE_SIZE, Tile, Vec2};

use crate::sandbox::Sandbox;

pub const BACKGROUND: Color = Color::hex(0x8fb8d9);
const SOLID: Color = Color::hex(0x5b6b7c);
const UNHOOKABLE: Color = Color::hex(0x3a4450);
const DEATH: Color = Color::hex(0xc94f4f);
const ELORA: Color = Color::hex(0xf2c14e);
const ELORA_OUTLINE: Color = Color::hex(0x2b2b2b);
const HOOK: Color = Color::hex(0xe8e8e8);
const CURSOR: Color = Color::hex(0xffffff);

/// Zeichnet Karte, Figur, Hook und Fadenkreuz.
pub fn sandbox(batch: &mut ShapeBatch, s: &Sandbox, camera: &Camera, mouse_pos: Vec2) {
    tiles(batch, s, camera);
    entities(batch, s);

    let alpha = s.alpha();
    let core = s.character();
    let pos = s.render_pos();

    if matches!(
        core.hook_state,
        HookState::Flying | HookState::Grabbed | HookState::Retracting(_)
    ) {
        let hook = s.prev.hook_pos.lerp(core.hook_pos, alpha);
        batch.stroke_line(pos, hook, 3.0, HOOK);
        batch.fill_circle(hook, 5.0, HOOK);
    }

    // Figur: Kreis in Hitbox-Größe, Blickrichtung als Auge
    let r = PHYS_SIZE / 2.0;
    batch.fill_circle(pos, r + 1.5, ELORA_OUTLINE);
    batch.fill_circle(pos, r, ELORA);
    let aim = mouse_pos.normalize();
    batch.fill_circle(pos + aim * (r * 0.5), 3.5, ELORA_OUTLINE);

    // Fadenkreuz
    let c = pos + mouse_pos;
    batch.stroke_circle(c, 8.0, 2.0, CURSOR);
    batch.fill_circle(c, 1.5, CURSOR);
}

fn tiles(batch: &mut ShapeBatch, s: &Sandbox, camera: &Camera) {
    let ts = TILE_SIZE as f32;
    let tl = camera.top_left();
    let br = tl + camera.size;
    let x0 = (tl.x / ts).floor() as i32 - 1;
    let y0 = (tl.y / ts).floor() as i32 - 1;
    let x1 = (br.x / ts).ceil() as i32 + 1;
    let y1 = (br.y / ts).ceil() as i32 + 1;
    let col = &s.world.collision;
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            let color = match col.tile(tx, ty) {
                Tile::Air => continue,
                Tile::Solid => SOLID,
                Tile::Unhookable => UNHOOKABLE,
                Tile::Death => DEATH,
            };
            let min = Vec2::new(tx as f32 * ts, ty as f32 * ts);
            batch.fill_rect(min, min + Vec2::new(ts, ts), color);
        }
    }
}

fn entities(batch: &mut ShapeBatch, s: &Sandbox) {
    for e in &s.map.entities {
        let (color, radius) = match e.kind {
            EntityKind::Spawn => (Color::rgba(1.0, 1.0, 1.0, 0.35), 10.0),
            EntityKind::SpawnRed | EntityKind::FlagRed => (Color::hex(0xd9534f), 10.0),
            EntityKind::SpawnBlue | EntityKind::FlagBlue => (Color::hex(0x4f7fd9), 10.0),
            EntityKind::Health => (Color::hex(0xe05a7a), 8.0),
            EntityKind::Armor => (Color::hex(0xe0b85a), 8.0),
            EntityKind::Laser => (Color::hex(0x5ad0e0), 9.0),
            EntityKind::Grenade => (Color::hex(0x6fbf4f), 9.0),
        };
        batch.fill_circle(e.pos(), radius, color);
    }
}

//! Darstellung der Szene. Die Figur ist final (M5.3, [`crate::figure`]); Welt,
//! Waffen und Effekte sind noch Platzhalter (E-043) und folgen in M5.5/M5.6.

use elora_client::scene::Scene;

use std::collections::BTreeMap;

use elora_protocol::Skin;

use crate::effects::Effects;
use crate::figure::{FigureArt, Figures};
use crate::items::ItemArt;
use crate::skins;

/// Alles, was neben der Szene zum Zeichnen der Figuren und Effekte gebraucht wird.
#[derive(Debug, Clone, Copy)]
pub struct Looks<'a> {
    pub effects: &'a Effects,
    pub figures: &'a Figures,
    pub art: &'a FigureArt,
    pub items: &'a ItemArt,
    /// Skins der anderen Slots (online).
    pub skins: &'a BTreeMap<usize, Skin>,
    /// Eigener Skin (sofort sichtbar, ohne Umweg über den Server).
    pub own_skin: Skin,
}
use elora_render::{Camera, Color, ShapeBatch};
use elora_sim::Team;
use elora_sim::{Collision, HookState, PHYS_SIZE, TILE_SIZE, Tile, Tuning, Vec2};

pub const BACKGROUND: Color = Color::hex(0x8fb8d9);
const SKY_TOP: Color = Color::hex(0xa9cde8);
const SKY_BOTTOM: Color = Color::hex(0x7ea8cb);
const SOLID: Color = Color::hex(0x5b6b7c);
const SOLID_EDGE: Color = Color::hex(0x2f3944);
const UNHOOKABLE: Color = Color::hex(0x3a4450);
const UNHOOKABLE_EDGE: Color = Color::hex(0x1c2229);
const DEATH: Color = Color::hex(0xc94f4f);
const DEATH_EDGE: Color = Color::hex(0x7c2a2a);
/// Breite der Tile-Kontur in Welteinheiten.
const EDGE_WIDTH: f32 = 3.0;
const ELORA: Color = Color::hex(0xf2c14e);
/// Andere menschliche Spieler (online, ohne Team).
const OTHER: Color = Color::hex(0x7ccf8a);
pub const RED: Color = Color::hex(0xe0574f);
pub const BLUE: Color = Color::hex(0x4f86e0);
const OUTLINE: Color = Color::hex(0x2b2b2b);
const HOOK: Color = Color::hex(0xe8e8e8);
const GRENADE: Color = Color::hex(0x6fbf4f);
const HEALTH: Color = Color::hex(0xe05a7a);
const ARMOR: Color = Color::hex(0xe0b85a);
const SPAWN: Color = Color::rgba(1.0, 1.0, 1.0, 0.35);

/// Zeichnet Karte, Pickups, Figuren, Projektile, Laser, Effekte und Fadenkreuz.
pub fn scene(
    batch: &mut ShapeBatch,
    scene: &Scene,
    collision: &Collision,
    tuning: &Tuning,
    camera: &Camera,
    mouse_pos: Vec2,
    looks: &Looks<'_>,
) {
    let Looks {
        effects,
        figures,
        art,
        items,
        skins,
        own_skin,
    } = *looks;
    let time = figures.time();
    sky(batch, camera);
    tiles(batch, collision, camera);
    spawns_and_pickups(batch, scene, items, time);

    for c in &scene.chars {
        let pos = c.pos();
        let core = &c.ch.core;
        if matches!(
            core.hook_state,
            HookState::Flying | HookState::Grabbed | HookState::Retracting(_)
        ) {
            let hook = c.hook_pos();
            batch.stroke_line(pos, hook, 3.0, HOOK);
            batch.fill_circle(hook, 5.0, HOOK);
        }
        // Blickrichtung: Elora folgt der Maus direkt, andere dem Winkel aus der Simulation
        let aim = if c.local {
            mouse_pos.normalize()
        } else {
            let a = core.angle as f32 / 256.0;
            Vec2::new(a.cos(), a.sin())
        };
        let skin = if c.local {
            own_skin
        } else {
            skins.get(&c.slot).copied().unwrap_or_default()
        };
        let r = PHYS_SIZE / 2.0;
        figures.draw(
            batch,
            art,
            c,
            aim,
            &skins::tint(skin, c.team, c.dummy, team_color),
        );
        let facing = if aim.x < 0.0 { -1.0 } else { 1.0 };
        let swing = figures.weapon_swing(c.slot, facing);
        items.draw_weapon(batch, pos, aim, swing, c.ch.arsenal.active);
        if c.local && c.team.index().is_some() {
            // eigene Figur im Team: gelber Ring am Boden zur Unterscheidung
            batch.stroke_line(
                pos + Vec2::new(-r, r + 3.0),
                pos + Vec2::new(r, r + 3.0),
                2.0,
                ELORA,
            );
        }
        if !c.local {
            health_bar(batch, pos, c.ch.health, c.ch.armor, tuning.max_health);
        }
    }

    for f in &scene.flags {
        if !f.at_stand {
            batch.stroke_circle(f.stand, 16.0, 2.0, team_color(f.team));
        }
        items.draw_flag(batch, f.pos, team_color(f.team), time);
    }

    for &p in &scene.projectiles {
        batch.fill_circle(p, 7.0, OUTLINE);
        batch.fill_circle(p, 5.5, GRENADE);
    }
    for l in &scene.lasers {
        batch.stroke_line(
            l.from,
            l.to,
            7.0 * l.fade,
            Color::rgba(0.35, 0.8, 0.9, 0.6 * l.fade),
        );
        batch.stroke_line(
            l.from,
            l.to,
            3.0 * l.fade,
            Color::rgba(0.9, 1.0, 1.0, l.fade),
        );
    }
    effects.draw(batch);

    // Fadenkreuz
    if let Some(local) = scene.local() {
        let c = local.pos() + mouse_pos;
        let color = crate::hud::crosshair_color(local.ch.health, tuning.max_health);
        batch.stroke_circle(c, 8.0, 2.0, color);
        batch.fill_circle(c, 1.5, color);
        effects.draw_hit_marker(batch, c);
    }
}

pub fn team_color(t: Team) -> Color {
    match t {
        Team::Red => RED,
        Team::Blue => BLUE,
        _ => OTHER,
    }
}

fn health_bar(batch: &mut ShapeBatch, pos: Vec2, health: i32, armor: i32, max: i32) {
    let w = 36.0;
    let top = pos + Vec2::new(-w / 2.0, -PHYS_SIZE);
    let frac = |v: i32| (v.max(0) as f32 / max as f32).min(1.0);
    batch.fill_rect(
        top,
        top + Vec2::new(w, 4.0),
        Color::rgba(0.0, 0.0, 0.0, 0.4),
    );
    batch.fill_rect(top, top + Vec2::new(w * frac(health), 4.0), HEALTH);
    if armor > 0 {
        let t = top + Vec2::new(0.0, -5.0);
        batch.fill_rect(t, t + Vec2::new(w * frac(armor), 3.0), ARMOR);
    }
}

/// Himmel: senkrechter Verlauf über den sichtbaren Bereich (E-089).
fn sky(batch: &mut ShapeBatch, camera: &Camera) {
    let tl = camera.top_left();
    batch.fill_rect_vgradient(tl, tl + camera.size, SKY_TOP, SKY_BOTTOM);
}

/// Füllung und Kontur je Tile-Art.
fn tile_colors(t: Tile) -> Option<(Color, Color)> {
    match t {
        Tile::Air => None,
        Tile::Solid => Some((SOLID, SOLID_EDGE)),
        Tile::Unhookable => Some((UNHOOKABLE, UNHOOKABLE_EDGE)),
        Tile::Death => Some((DEATH, DEATH_EDGE)),
    }
}

/// Tiles einfarbig mit Kontur an allen Kanten zu anderen Tile-Arten (E-089).
fn tiles(batch: &mut ShapeBatch, col: &Collision, camera: &Camera) {
    let ts = TILE_SIZE as f32;
    let tl = camera.top_left();
    let br = tl + camera.size;
    let x0 = (tl.x / ts).floor() as i32 - 1;
    let y0 = (tl.y / ts).floor() as i32 - 1;
    let x1 = (br.x / ts).ceil() as i32 + 1;
    let y1 = (br.y / ts).ceil() as i32 + 1;
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            let tile = col.tile(tx, ty);
            let Some((fill, edge)) = tile_colors(tile) else {
                continue;
            };
            let min = Vec2::new(tx as f32 * ts, ty as f32 * ts);
            let max = min + Vec2::new(ts, ts);
            batch.fill_rect(min, max, fill);
            let w = EDGE_WIDTH;
            // (Nachbar, Streifen innerhalb des Tiles)
            let edges = [
                ((tx, ty - 1), min, Vec2::new(max.x, min.y + w)),
                ((tx, ty + 1), Vec2::new(min.x, max.y - w), max),
                ((tx - 1, ty), min, Vec2::new(min.x + w, max.y)),
                ((tx + 1, ty), Vec2::new(max.x - w, min.y), max),
            ];
            for ((nx, ny), a, b) in edges {
                if col.tile(nx, ny) != tile {
                    batch.fill_rect(a, b, edge);
                }
            }
        }
    }
}

fn spawns_and_pickups(batch: &mut ShapeBatch, scene: &Scene, items: &ItemArt, time: f32) {
    for &sp in &scene.spawns {
        batch.stroke_circle(sp, 10.0, 2.0, SPAWN);
    }
    for &(kind, p) in &scene.pickups {
        items.draw_pickup(batch, p, kind, time);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_client::scene::{SceneChar, SceneFlag};
    use elora_sim::{Character, Weapon};

    /// Kartenausschnitt zur Sichtprüfung: `cargo test -p elora-client --bin elora world_sheet -- --ignored`,
    /// danach `cargo xtask svg-preview target/world.svg target/world.png 1200`.
    #[test]
    #[ignore = "erzeugt nur eine Datei zur Sichtprüfung"]
    fn world_sheet() {
        let map =
            elora_map::parse_text_map(include_str!("../../../maps/sandbox.emap.toml")).unwrap();
        let world = map.world(Tuning::default());
        let col = map.collision();
        let camera = Camera {
            center: Vec2::new(600.0, 420.0),
            size: Vec2::new(1200.0, 440.0),
        };
        // (Position, Waffe, Zielwinkel in 1/256 rad, Team)
        let chars = [
            (Vec2::new(250.0, 530.0), Weapon::Hammer, 0, Team::None),
            (Vec2::new(420.0, 530.0), Weapon::Grenade, -100, Team::Red),
            (Vec2::new(900.0, 434.0), Weapon::Laser, 804, Team::Blue),
        ];
        let chars = chars
            .iter()
            .enumerate()
            .map(|(slot, &(pos, w, angle, team))| {
                let mut ch = Character::spawn(pos, 10);
                ch.arsenal.give(w, 10, 10);
                ch.arsenal.active = w;
                ch.core.angle = angle;
                SceneChar {
                    slot,
                    prev: ch.core.clone(),
                    ch,
                    alpha: 1.0,
                    dummy: false,
                    local: false,
                    team,
                }
            })
            .collect();
        let scene = Scene {
            pickups: world.pickups.iter().map(|p| (p.kind, p.pos)).collect(),
            spawns: world.spawn_points.clone(),
            chars,
            flags: vec![
                SceneFlag {
                    team: Team::Red,
                    pos: Vec2::new(560.0, 530.0),
                    at_stand: true,
                    stand: Vec2::new(560.0, 530.0),
                },
                SceneFlag {
                    team: Team::Blue,
                    pos: Vec2::new(1040.0, 434.0),
                    at_stand: true,
                    stand: Vec2::new(1040.0, 434.0),
                },
            ],
            ..Scene::default()
        };
        let mut figures = Figures::default();
        figures.update(0.01, &scene, &col, &[]);
        let mut batch = ShapeBatch::default();
        super::scene(
            &mut batch,
            &scene,
            &col,
            &Tuning::default(),
            &camera,
            Vec2::new(1.0, 0.0),
            &Looks {
                effects: &Effects::default(),
                figures: &figures,
                art: &FigureArt::load(),
                items: &ItemArt::load(),
                skins: &BTreeMap::new(),
                own_skin: Skin::default(),
            },
        );
        let tl = camera.top_left();
        let svg = batch.debug_svg(tl, tl + camera.size, BACKGROUND);
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/world.svg"),
            svg,
        )
        .unwrap();
    }
}

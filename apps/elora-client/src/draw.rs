//! Darstellung der Szene. Die Figur ist final (M5.3, [`crate::figure`]); Welt,
//! Waffen und Effekte sind noch Platzhalter (E-043) und folgen in M5.5/M5.6.

use elora_client::scene::Scene;

use std::collections::BTreeMap;

use elora_protocol::Skin;

use crate::figure::{FigureArt, Figures};
use crate::skins;

/// Alles, was neben der Szene zum Zeichnen der Figuren und Effekte gebraucht wird.
#[derive(Debug, Clone, Copy)]
pub struct Looks<'a> {
    pub effects: &'a Effects,
    pub figures: &'a Figures,
    pub art: &'a FigureArt,
    /// Skins der anderen Slots (online).
    pub skins: &'a BTreeMap<usize, Skin>,
    /// Eigener Skin (sofort sichtbar, ohne Umweg über den Server).
    pub own_skin: Skin,
}
use elora_render::{Camera, Color, ShapeBatch};
use elora_sim::Team;
use elora_sim::{
    Collision, Event, HookState, PHYS_SIZE, PickupKind, TILE_SIZE, Tile, Tuning, Vec2, Weapon,
};

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
const CURSOR: Color = Color::hex(0xffffff);
const HAMMER: Color = Color::hex(0x8a6a4a);
const GRENADE: Color = Color::hex(0x6fbf4f);
const LASER: Color = Color::hex(0x5ad0e0);
const HEALTH: Color = Color::hex(0xe05a7a);
const ARMOR: Color = Color::hex(0xe0b85a);
const SPAWN: Color = Color::rgba(1.0, 1.0, 1.0, 0.35);

pub fn weapon_color(w: Weapon) -> Color {
    match w {
        Weapon::Hammer => HAMMER,
        Weapon::Grenade => GRENADE,
        Weapon::Laser => LASER,
    }
}

/// Kurzlebige Effekte aus Simulations-Ereignissen.
#[derive(Debug, Default)]
pub struct Effects {
    list: Vec<Effect>,
}

#[derive(Debug)]
struct Effect {
    kind: EffectKind,
    pos: Vec2,
    /// Alter in Sekunden.
    age: f32,
}

#[derive(Debug, Clone, Copy)]
enum EffectKind {
    Explosion,
    HammerHit,
    Death,
    Spawn,
}

impl EffectKind {
    fn lifetime(self) -> f32 {
        match self {
            Self::Explosion => 0.35,
            Self::HammerHit => 0.15,
            Self::Death => 0.5,
            Self::Spawn => 0.3,
        }
    }
}

impl Effects {
    pub fn update(&mut self, dt: f32, events: &[Event]) {
        for e in &mut self.list {
            e.age += dt;
        }
        self.list.retain(|e| e.age < e.kind.lifetime());
        for event in events {
            let (kind, pos) = match *event {
                Event::Explosion { pos, .. } => (EffectKind::Explosion, pos),
                Event::HammerHit { pos, .. } => (EffectKind::HammerHit, pos),
                Event::Death { pos, .. } => (EffectKind::Death, pos),
                Event::Spawn { pos, .. } => (EffectKind::Spawn, pos),
                _ => continue,
            };
            self.list.push(Effect {
                kind,
                pos,
                age: 0.0,
            });
        }
    }

    fn draw(&self, batch: &mut ShapeBatch, explosion_radius: f32) {
        for e in &self.list {
            let t = e.age / e.kind.lifetime();
            let fade = 1.0 - t;
            match e.kind {
                EffectKind::Explosion => {
                    batch.fill_circle(
                        e.pos,
                        explosion_radius * (0.3 + 0.7 * t),
                        Color::rgba(1.0, 0.75, 0.3, 0.45 * fade),
                    );
                    batch.fill_circle(
                        e.pos,
                        explosion_radius * 0.35 * (1.0 - t * 0.5),
                        Color::rgba(1.0, 0.95, 0.7, 0.8 * fade),
                    );
                }
                EffectKind::HammerHit => {
                    batch.stroke_circle(
                        e.pos,
                        10.0 + 20.0 * t,
                        3.0,
                        Color::rgba(1.0, 1.0, 1.0, fade),
                    );
                }
                EffectKind::Death => {
                    for k in 0..8 {
                        let a = k as f32 * std::f32::consts::TAU / 8.0;
                        let p = e.pos + Vec2::new(a.cos(), a.sin()) * (10.0 + 50.0 * t);
                        batch.fill_circle(p, 5.0 * fade, Color::rgba(0.9, 0.3, 0.3, fade));
                    }
                }
                EffectKind::Spawn => {
                    batch.stroke_circle(
                        e.pos,
                        40.0 * (1.0 - t),
                        3.0,
                        Color::rgba(1.0, 1.0, 1.0, fade),
                    );
                }
            }
        }
    }
}

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
        skins,
        own_skin,
    } = *looks;
    sky(batch, camera);
    tiles(batch, collision, camera);
    spawns_and_pickups(batch, scene);

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
        weapon(batch, pos, aim, c.ch.arsenal.active);
        figures.draw(
            batch,
            art,
            c,
            aim,
            &skins::tint(skin, c.team, c.dummy, team_color),
        );
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
        flag(batch, f.pos, team_color(f.team));
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
    effects.draw(batch, tuning.explosion_radius);

    // Fadenkreuz
    if let Some(local) = scene.local() {
        let c = local.pos() + mouse_pos;
        batch.stroke_circle(c, 8.0, 2.0, CURSOR);
        batch.fill_circle(c, 1.5, CURSOR);
    }
}

pub fn team_color(t: Team) -> Color {
    match t {
        Team::Red => RED,
        Team::Blue => BLUE,
        _ => OTHER,
    }
}

/// Flagge: Stange und Wimpel, Fuß bei `pos`.
fn flag(batch: &mut ShapeBatch, pos: Vec2, color: Color) {
    let foot = pos + Vec2::new(0.0, 14.0);
    let top = foot + Vec2::new(0.0, -52.0);
    batch.stroke_line(foot, top, 4.0, OUTLINE);
    batch.fill_polygon(
        &[top, top + Vec2::new(30.0, 10.0), top + Vec2::new(0.0, 22.0)],
        color,
    );
}

fn weapon(batch: &mut ShapeBatch, pos: Vec2, aim: Vec2, w: Weapon) {
    let side = Vec2::new(-aim.y, aim.x);
    match w {
        Weapon::Hammer => {
            let handle_end = pos + aim * 26.0;
            batch.stroke_line(pos + aim * 8.0, handle_end, 4.0, OUTLINE);
            let head = [handle_end + side * 9.0, handle_end - side * 9.0];
            batch.stroke_line(head[0], head[1], 9.0, HAMMER);
        }
        Weapon::Grenade => batch.stroke_line(pos + aim * 6.0, pos + aim * 34.0, 9.0, GRENADE),
        Weapon::Laser => batch.stroke_line(pos + aim * 6.0, pos + aim * 38.0, 6.0, LASER),
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

fn spawns_and_pickups(batch: &mut ShapeBatch, scene: &Scene) {
    for &sp in &scene.spawns {
        batch.stroke_circle(sp, 10.0, 2.0, SPAWN);
    }
    for &(kind, p) in &scene.pickups {
        match kind {
            PickupKind::Health => {
                batch.fill_circle(p + Vec2::new(-4.5, -3.0), 6.0, HEALTH);
                batch.fill_circle(p + Vec2::new(4.5, -3.0), 6.0, HEALTH);
                batch.fill_polygon(
                    &[
                        p + Vec2::new(-10.5, -1.0),
                        p + Vec2::new(10.5, -1.0),
                        p + Vec2::new(0.0, 10.0),
                    ],
                    HEALTH,
                );
            }
            PickupKind::Armor => batch.fill_polygon(
                &[
                    p + Vec2::new(-9.0, -9.0),
                    p + Vec2::new(9.0, -9.0),
                    p + Vec2::new(9.0, 1.0),
                    p + Vec2::new(0.0, 10.0),
                    p + Vec2::new(-9.0, 1.0),
                ],
                ARMOR,
            ),
            PickupKind::Weapon(w) => {
                batch.stroke_circle(p, 13.0, 2.0, weapon_color(w));
                weapon(batch, p - Vec2::new(18.0, 0.0), Vec2::new(1.0, 0.0), w);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let scene = Scene {
            pickups: world.pickups.iter().map(|p| (p.kind, p.pos)).collect(),
            spawns: world.spawn_points.clone(),
            ..Scene::default()
        };
        let mut batch = ShapeBatch::default();
        sky(&mut batch, &camera);
        tiles(&mut batch, &col, &camera);
        spawns_and_pickups(&mut batch, &scene);
        let tl = camera.top_left();
        let svg = batch.debug_svg(tl, tl + camera.size, BACKGROUND);
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/world.svg"),
            svg,
        )
        .unwrap();
    }
}

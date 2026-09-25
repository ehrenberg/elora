//! Platzhalter-Darstellung für die Sandbox (E-043): einfache Vektorformen.
//! Die finale Optik folgt in M5.

use elora_render::{Camera, Color, ShapeBatch};
use elora_sim::{
    Controller, Event, HookState, PHYS_SIZE, PickupKind, TICKS_PER_SECOND, TILE_SIZE, Tile, Vec2,
    Weapon,
};

use crate::sandbox::Sandbox;

pub const BACKGROUND: Color = Color::hex(0x8fb8d9);
const SOLID: Color = Color::hex(0x5b6b7c);
const UNHOOKABLE: Color = Color::hex(0x3a4450);
const DEATH: Color = Color::hex(0xc94f4f);
const ELORA: Color = Color::hex(0xf2c14e);
const DUMMY: Color = Color::hex(0xb59fd6);
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
                Event::Explosion { pos } => (EffectKind::Explosion, pos),
                Event::HammerHit { pos } => (EffectKind::HammerHit, pos),
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
pub fn sandbox(
    batch: &mut ShapeBatch,
    s: &Sandbox,
    camera: &Camera,
    mouse_pos: Vec2,
    effects: &Effects,
) {
    tiles(batch, s, camera);
    spawns_and_pickups(batch, s);
    let alpha = s.alpha();

    for (i, slot) in s.world.players.iter().enumerate() {
        let Some(p) = slot else { continue };
        let Some(ch) = &p.character else { continue };
        let Some((cur, prev)) = s.cores(i) else {
            continue;
        };
        let pos = prev.pos.lerp(cur.pos, alpha);

        if matches!(
            cur.hook_state,
            HookState::Flying | HookState::Grabbed | HookState::Retracting(_)
        ) {
            let hook = prev.hook_pos.lerp(cur.hook_pos, alpha);
            batch.stroke_line(pos, hook, 3.0, HOOK);
            batch.fill_circle(hook, 5.0, HOOK);
        }

        // Blickrichtung: Elora folgt der Maus direkt, andere dem Winkel aus der Simulation
        let aim = if i == s.player {
            mouse_pos.normalize()
        } else {
            let a = cur.angle as f32 / 256.0;
            Vec2::new(a.cos(), a.sin())
        };
        let body = if matches!(p.controller, Controller::Human) {
            ELORA
        } else {
            DUMMY
        };
        let r = PHYS_SIZE / 2.0;
        weapon(batch, pos, aim, ch.arsenal.active);
        batch.fill_circle(pos, r + 1.5, OUTLINE);
        batch.fill_circle(pos, r, body);
        batch.fill_circle(pos + aim * (r * 0.5), 3.5, OUTLINE);

        if i != s.player {
            health_bar(batch, pos, ch.health, ch.armor, s.world.tuning.max_health);
        }
    }

    projectiles(batch, s, alpha);
    lasers(batch, s, alpha);
    effects.draw(batch, s.world.tuning.explosion_radius);

    // Fadenkreuz
    if s.character().is_some() {
        let c = s.render_pos() + mouse_pos;
        batch.stroke_circle(c, 8.0, 2.0, CURSOR);
        batch.fill_circle(c, 1.5, CURSOR);
    }
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

fn projectiles(batch: &mut ShapeBatch, s: &Sandbox, alpha: f32) {
    let tps = TICKS_PER_SECOND as f32;
    for pr in &s.world.projectiles {
        let age = (s.world.tick - pr.start_tick) as f32;
        let p = pr.pos_at((age - 1.0 + alpha).max(0.0) / tps, &s.world.tuning);
        batch.fill_circle(p, 7.0, OUTLINE);
        batch.fill_circle(p, 5.5, GRENADE);
    }
}

fn lasers(batch: &mut ShapeBatch, s: &Sandbox, alpha: f32) {
    // Sichtbar bis zum nächsten Abschnitt, dann ausblenden
    let fade_ticks = (TICKS_PER_SECOND * s.world.tuning.laser_bounce_delay) as f32 / 1000.0 + 1.0;
    for l in &s.world.lasers {
        let age = (s.world.tick - l.eval_tick) as f32 + alpha;
        let fade = (1.0 - age / fade_ticks).clamp(0.2, 1.0);
        batch.stroke_line(
            l.from,
            l.pos,
            7.0 * fade,
            Color::rgba(0.35, 0.8, 0.9, 0.6 * fade),
        );
        batch.stroke_line(l.from, l.pos, 3.0 * fade, Color::rgba(0.9, 1.0, 1.0, fade));
    }
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

fn spawns_and_pickups(batch: &mut ShapeBatch, s: &Sandbox) {
    for &sp in &s.world.spawn_points {
        batch.stroke_circle(sp, 10.0, 2.0, SPAWN);
    }
    for pk in s.world.pickups.iter().filter(|p| p.available()) {
        let p = pk.pos;
        match pk.kind {
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

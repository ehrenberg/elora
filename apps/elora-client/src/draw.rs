//! Darstellung der Szene: Karte ([`crate::map_view`], M6.4), Figuren (M5.3), Items, Geschosse und Effekte.

use elora_client::scene::Scene;

use std::collections::BTreeMap;

use elora_protocol::Skin;

use crate::creatures::CreatureArt;
use crate::effects::Effects;
use crate::emotes::Emotes;
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
    pub creatures: &'a CreatureArt,
    pub emotes: &'a Emotes,
    /// Skins der anderen Slots (online).
    pub skins: &'a BTreeMap<usize, Skin>,
    /// Eigener Skin (sofort sichtbar, ohne Umweg über den Server).
    pub own_skin: Skin,
}
use elora_render::{Camera, Color, ShapeBatch};
use elora_sim::Team;
use elora_sim::{HookState, PHYS_SIZE, Tuning, Vec2};

use crate::map_view::MapLayer;

pub const BACKGROUND: Color = Color::hex(0x8fb8d9);
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
#[allow(clippy::too_many_lines)] // eine Zeichenreihenfolge, bewusst an einem Ort
pub fn scene(
    batch: &mut ShapeBatch,
    scene: &Scene,
    map: MapLayer<'_>,
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
        creatures,
        emotes,
        skins,
        own_skin,
    } = *looks;
    let time = figures.time();
    let MapLayer {
        map,
        view: map_view,
        time: look_time,
    } = map;
    if let Some(map) = map {
        map_view.draw_back(batch, map, camera, look_time);
    }
    spawns_and_pickups(batch, scene, items, time);
    for (item, pos) in &scene.loot {
        creatures.draw_loot(batch, item, *pos, time);
    }
    for c in &scene.creatures {
        creatures.draw(batch, c, time);
    }

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
        // Schutz nach Treffer im Abenteuer: Elora blinkt (E-234)
        if c.ch.invulnerable_until > scene.tick && (scene.tick / 4).is_multiple_of(2) {
            continue;
        }
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
    for &p in &scene.creature_shots {
        CreatureArt::draw_shot(batch, p);
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
    if let Some(map) = map {
        map_view.draw_front(batch, map, camera, look_time);
    }
    effects.draw(batch);
    emotes.draw(batch, scene);

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
        let map = elora_map::decode(include_bytes!("../../../maps/sandbox.emap")).unwrap();
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
            MapLayer {
                map: Some(&map),
                view: &mut crate::map_view::MapView::default(),
                time: crate::map_view::LookTime::default(),
            },
            &Tuning::default(),
            &camera,
            Vec2::new(1.0, 0.0),
            &Looks {
                effects: &Effects::default(),
                figures: &figures,
                art: &FigureArt::load(),
                items: &ItemArt::load(),
                creatures: &CreatureArt::load(),
                emotes: &Emotes::new(),
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

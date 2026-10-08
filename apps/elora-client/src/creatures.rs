//! Drawing enemies, their projectiles and loot (A1.2) from `assets/adventure/`.
//!
//! Enemies are drawn in world units, origin in the center of the collision box,
//! facing right; to the left they are mirrored.

use std::collections::HashMap;

use elora_client::scene::SceneCreature;
use elora_render::{Affine, Color, Mesh, ShapeBatch, SvgAsset, Tint};
use elora_sim::Vec2;

/// How long the health bar stays visible after a hit (ticks, E-238).
const BAR_TICKS: u64 = 150;
const OUTLINE: Color = Color::hex(0x2b2b2b);
const BAR_BACK: Color = Color::rgba(0.118, 0.165, 0.212, 0.55);
const BAR: Color = Color::hex(0xe05a7a);
const POLLEN: Color = Color::hex(0xf8dd6e);
const POLLEN_GLOW: Color = Color::rgba(0.95, 0.76, 0.31, 0.35);
const STUN: Color = Color::hex(0xf2c14e);
const EMBER: Color = Color::hex(0xff9a3c);
const EMBER_CORE: Color = Color::hex(0xfff0b0);
const EMBER_GLOW: Color = Color::rgba(1.0, 0.6, 0.2, 0.3);

/// Parts of an enemy graphic: `idle` and optionally `air`.
#[derive(Debug)]
struct Look {
    idle: Mesh,
    air: Option<Mesh>,
    /// Guardian: dive and dazed (R2-M2.1).
    dive: Option<Mesh>,
    stunned: Option<Mesh>,
    /// Hidden in the ground (root snake, state 0).
    hidden: Option<Mesh>,
    /// Pose per state (`m<Zustand>`, e.g. `m3`): takes precedence over all others (Root Warden).
    modes: HashMap<u8, Mesh>,
}

#[derive(Debug)]
pub struct CreatureArt {
    looks: HashMap<&'static str, Look>,
    glanztropfen: Mesh,
    item: Mesh,
    /// Own images of individual items.
    items: HashMap<&'static str, Mesh>,
    /// NPCs (A1.7), origin on the ground.
    characters: HashMap<&'static str, Mesh>,
    /// Objects with two states (off, on), origin on the ground.
    objects: HashMap<&'static str, (Mesh, Mesh)>,
}

macro_rules! adventure_svgs {
    ($dir:literal: $($name:literal),* $(,)?) => {
        &[$(($name, include_bytes!(concat!("../../../assets/adventure/", $dir, "/", $name, ".svg")))),*]
    };
}

/// Tilt of a kind's flight pose in the graphic (radians, upwards negative) – `None`:
/// the pose does not rotate along (sand snake and dune worm in the jump, E-328).
fn flight_tilt(c: &SceneCreature) -> Option<f32> {
    match (c.kind.as_str(), c.mode) {
        ("sandschlange", elora_sim::creature::serpent::LEAP) => Some(-0.45),
        ("duenenwurm", elora_sim::creature::leaper::LEAP) => Some(0.0),
        _ => None,
    }
}

const CHARACTER_FILES: &[(&str, &[u8])] = adventure_svgs!("characters": "oma", "klonk", "lotte", "tueftel", "pip", "wegweiser", "wabe", "hummel", "plumm", "pilzkind", "pilzkind_froh", "pilzmama", "waechter", "sirup", "palma", "schlange", "ruinenquelle", "flocke", "bolle", "kiesel", "wicke", "kristella");
/// Figures that share an image (boards, spots of the oasis, R2-M2.3).
const SHARED_CHARACTER_FILES: &[(&str, &[u8])] = {
    const TAFEL: &[u8] = include_bytes!("../../../assets/adventure/characters/tafel.svg");
    const DRY: &[u8] = include_bytes!("../../../assets/adventure/characters/giessstelle.svg");
    const BLOOM: &[u8] =
        include_bytes!("../../../assets/adventure/characters/giessstelle_bluete.svg");
    const BOLLE: &[u8] = include_bytes!("../../../assets/adventure/characters/bolle.svg");
    const KIESEL: &[u8] = include_bytes!("../../../assets/adventure/characters/kiesel.svg");
    const WICKE: &[u8] = include_bytes!("../../../assets/adventure/characters/wicke.svg");
    const GREY: &[u8] = include_bytes!("../../../assets/adventure/characters/graue_stelle.svg");
    &[
        ("tafel-1", TAFEL),
        ("tafel-2", TAFEL),
        ("tafel-kammer", TAFEL),
        ("giessstelle-1", DRY),
        ("giessstelle-2", DRY),
        ("giessstelle-3", DRY),
        ("bluete-1", BLOOM),
        ("bluete-2", BLOOM),
        ("bluete-3", BLOOM),
        ("bolle-huette", BOLLE),
        ("kiesel-huette", KIESEL),
        ("wicke-huette", WICKE),
        ("graue-stelle", GREY),
    ]
};
/// Items with their own image (R2-M2.1); all others show `item.svg`.
const ITEM_FILES: &[(&str, &[u8])] =
    adventure_svgs!("items": "biene", "quellfunke", "wabenhut", "rune", "wasserschlauch", "wasser");
/// Object and the names of its two parts (off, on).
const OBJECT_FILES: &[(&str, &[u8], [&str; 2])] = &[
    (
        "truhe",
        include_bytes!("../../../assets/adventure/objects/truhe.svg"),
        ["closed", "open"],
    ),
    (
        "quellstein",
        include_bytes!("../../../assets/adventure/objects/quellstein.svg"),
        ["off", "on"],
    ),
    (
        "schalter",
        include_bytes!("../../../assets/adventure/objects/schalter.svg"),
        ["off", "on"],
    ),
    (
        "heilpflanze",
        include_bytes!("../../../assets/adventure/objects/heilpflanze.svg"),
        ["fresh", "used"],
    ),
];

fn load(data: &[u8], file: &str) -> SvgAsset {
    SvgAsset::load(data, 0.08).unwrap_or_else(|e| panic!("assets/adventure/{file}: {e}"))
}

macro_rules! creatures {
    ($($name:literal),* $(,)?) => {
        &[$(($name, include_bytes!(concat!("../../../assets/adventure/creatures/", $name, ".svg")))),*]
    };
}

const CREATURE_FILES: &[(&str, &[u8])] = creatures!(
    "stachelkaefer",
    "pollenblaeser",
    "grashuepfer",
    "strohpuppe",
    "brummbaer",
    "wirrbiene",
    "wurzelschlange",
    "eichhornpirat",
    "pilzwicht",
    "pilzkind",
    "wurzelwaechter",
    "sandkrabbe",
    "duenenwurm",
    "funkenmotte",
    "sandschlange",
    "eiszapfen",
    "schneebrocken",
    "schneeballrobbe",
    "fledermaus",
    "frostgeist",
    "kristella"
);

impl CreatureArt {
    /// # Panics
    /// If an embedded asset is faulty (covered by tests).
    pub fn load() -> Self {
        let looks = CREATURE_FILES
            .iter()
            .map(|&(name, data)| {
                let a = load(data, &format!("creatures/{name}.svg"));
                let idle = a
                    .part("idle")
                    .cloned()
                    .unwrap_or_else(|| panic!("{name}.svg: Teil `idle` fehlt"));
                (
                    name,
                    Look {
                        idle,
                        air: a.part("air").cloned(),
                        dive: a.part("dive").cloned(),
                        stunned: a.part("stunned").cloned(),
                        hidden: a.part("hidden").cloned(),
                        modes: (0..8u8)
                            .filter_map(|m| Some((m, a.part(&format!("m{m}"))?.clone())))
                            .collect(),
                    },
                )
            })
            .collect();
        let whole = |data: &[u8], file: &str| {
            load(data, file)
                .part("")
                .cloned()
                .unwrap_or_else(|| panic!("{file} ist leer"))
        };
        let characters = CHARACTER_FILES
            .iter()
            .chain(SHARED_CHARACTER_FILES)
            .map(|&(name, data)| {
                let a = load(data, &format!("characters/{name}.svg"));
                let m = a
                    .part("figure")
                    .cloned()
                    .unwrap_or_else(|| panic!("{name}.svg: Teil `figure` fehlt"));
                (name, m)
            })
            .collect();
        let objects = OBJECT_FILES
            .iter()
            .map(|&(name, data, [off, on])| {
                let a = load(data, &format!("objects/{name}.svg"));
                let part = |p: &str| {
                    a.part(p)
                        .cloned()
                        .unwrap_or_else(|| panic!("{name}.svg: Teil `{p}` fehlt"))
                };
                (name, (part(off), part(on)))
            })
            .collect();
        Self {
            looks,
            characters,
            objects,
            glanztropfen: whole(
                include_bytes!("../../../assets/adventure/items/glanztropfen.svg"),
                "items/glanztropfen.svg",
            ),
            items: ITEM_FILES
                .iter()
                .map(|&(name, data)| (name, whole(data, &format!("items/{name}.svg"))))
                .collect(),
            item: whole(
                include_bytes!("../../../assets/adventure/items/item.svg"),
                "items/item.svg",
            ),
        }
    }

    /// Enemies with a health bar after hits; unknown kinds as a circle. They breathe when standing,
    /// bob when walking, hover in the air and tilt flight poses into the flight direction (E-328).
    #[allow(clippy::too_many_lines)]
    pub fn draw(&self, batch: &mut ShapeBatch, c: &SceneCreature, time: f32) {
        let flip = if c.facing < 0 { -1.0 } else { 1.0 };
        #[allow(clippy::cast_precision_loss)]
        let phase = time + c.id as f32 * 1.37;
        let flash = c.since_hit.is_some_and(|t| t < 6);
        let tint = if flash {
            Tint {
                alpha: Some(0.6),
                ..Tint::default()
            }
        } else {
            Tint::default()
        };
        // frost wave of the guardian across the floor of the hall (E-341)
        if let Some(hall) = c.hall
            && c.mode == elora_sim::creature::queen::WAVE
        {
            draw_frost_wave(batch, c.goal, c.facing, hall, time);
        }
        if let Some(look) = self.looks.get(c.kind.as_str()) {
            use elora_sim::creature::diver;
            let mesh = look
                .modes
                .get(&c.mode)
                .or(match c.mode {
                    elora_sim::creature::burrow::HIDDEN => look.hidden.as_ref(),
                    diver::DIVE | diver::AIM => look.dive.as_ref(),
                    diver::STUNNED => look.stunned.as_ref(),
                    _ => None,
                })
                .or_else(|| look.air.as_ref().filter(|_| c.airborne))
                .unwrap_or(&look.idle);
            // foot of the graphic (bottom edge): breathing and growing stay on the ground
            let h = mesh.bounds().map_or(0.0, |(_, max)| max.y);
            // hidden in the ground or sand: still
            let hidden = look.hidden.as_ref().is_some_and(|m| std::ptr::eq(m, mesh))
                || (c.kind == "sandschlange" && c.mode <= 2)
                || (c.kind == "duenenwurm" && c.mode <= 1);
            let t = if c.kind == "eiszapfen" {
                // hangs still, trembles before the fall (R2-M2.4)
                let shake = if c.mode == elora_sim::creature::icicle::SHAKE {
                    (time * 70.0).sin() * 1.6
                } else {
                    0.0
                };
                Affine::translate(c.pos + Vec2::new(shake, 0.0))
            } else if c.kind == "fledermaus" && c.mode == elora_sim::creature::bat::HANG {
                // sleeps upside down: no hovering
                Affine::translate(c.pos).then(Affine::scale(flip, 1.0))
            } else if c.kind == "schneebrocken" {
                // rolls: rotates with the distance travelled
                Affine::translate(c.pos).then(Affine::rotate(c.pos.x / 18.0))
            } else if c.grow < 1.0 {
                // the root snake grows slowly out of the ground
                Affine::translate(c.pos + Vec2::new(0.0, h))
                    .then(Affine::scale(flip, c.grow.max(0.05)))
                    .then(Affine::translate(Vec2::new(0.0, -h)))
            } else if let Some(tilt) = flight_tilt(c) {
                // the flight pose points in the flight direction
                let pitch = c.vel.y.atan2(c.vel.x.abs().max(0.5));
                let a = ((pitch - tilt) * flip).clamp(-1.2, 1.2);
                Affine::translate(c.pos)
                    .then(Affine::rotate(a))
                    .then(Affine::scale(flip, 1.0))
            } else if hidden {
                Affine::translate(c.pos).then(Affine::scale(flip, 1.0))
            } else if c.airborne {
                // flyers hover up and down, jumpers tilt slightly
                let bob = (phase * 3.1).sin() * 2.5;
                let lean = (c.vel.x * 0.03).clamp(-0.25, 0.25);
                Affine::translate(c.pos + Vec2::new(0.0, bob))
                    .then(Affine::rotate(lean))
                    .then(Affine::scale(flip, 1.0))
            } else if c.vel.x.abs() > 0.3 {
                // gait: bobs with the steps
                let step = c.pos.x * 0.22;
                let bob = -step.sin().abs() * 2.5;
                let rock = step.sin() * 0.05;
                Affine::translate(c.pos + Vec2::new(0.0, h + bob))
                    .then(Affine::rotate(rock))
                    .then(Affine::scale(flip, 1.0))
                    .then(Affine::translate(Vec2::new(0.0, -h)))
            } else {
                // breathing: height and width in opposite directions, the area stays
                let sy = 1.0 + (phase * 2.4).sin() * 0.03;
                Affine::translate(c.pos + Vec2::new(0.0, h))
                    .then(Affine::scale(flip / sy, sy))
                    .then(Affine::translate(Vec2::new(0.0, -h)))
            };
            batch.draw_mesh(mesh, &t, &tint);
        } else {
            batch.fill_circle(c.pos, 16.0, OUTLINE);
            batch.fill_circle(c.pos, 13.0, Color::hex(0xc94a4a));
        }
        if c.stunned {
            for k in 0..3 {
                #[allow(clippy::cast_precision_loss)]
                let a = time * 4.0 + k as f32 * std::f32::consts::TAU / 3.0;
                let p = c.pos + Vec2::new(a.cos() * 14.0, -26.0 + a.sin() * 4.0);
                batch.fill_circle(p, 3.0, STUN);
            }
        }
        if !c.boss && c.since_hit.is_some_and(|t| t < BAR_TICKS) && c.max_health > 0 {
            let (w, h) = (36.0, 5.0);
            let top = c.pos + Vec2::new(-w / 2.0, -38.0);
            #[allow(clippy::cast_precision_loss)]
            let frac = (c.health.max(0) as f32 / c.max_health as f32).clamp(0.0, 1.0);
            batch.fill_rect(top, top + Vec2::new(w, h), BAR_BACK);
            batch.fill_rect(top, top + Vec2::new(w * frac, h), BAR);
        }
    }

    /// Inanimate figures (signs, boards, springs, plants): do not breathe, cast no shadow.
    pub fn is_still(id: &str) -> bool {
        [
            "wegweiser",
            "tafel",
            "ruinenquelle",
            "giessstelle",
            "bluete",
            "graue-stelle",
        ]
        .iter()
        .any(|p| id.starts_with(p))
    }

    /// Does the enemy cast a shadow? Not while it is stuck in the ground or sand.
    pub fn casts_shadow(c: &SceneCreature) -> bool {
        !match c.kind.as_str() {
            "wurzelschlange" => c.mode == elora_sim::creature::burrow::HIDDEN,
            "duenenwurm" => c.mode <= elora_sim::creature::leaper::WARN,
            "sandschlange" => c.mode <= elora_sim::creature::serpent::WARN,
            _ => false,
        }
    }

    /// Figure in the world: like [`Self::draw_character`], plus living figures breathe slightly
    /// (signs, boards, springs and plants stand still, E-328).
    pub fn draw_character_alive(
        &self,
        batch: &mut ShapeBatch,
        id: &str,
        ground: Vec2,
        facing: i8,
        time: f32,
    ) -> bool {
        let Some(m) = self.characters.get(id) else {
            return false;
        };
        let sy = if Self::is_still(id) {
            1.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let seed = id.bytes().map(f32::from).sum::<f32>();
            1.0 + (time * 2.2 + seed).sin() * 0.025
        };
        let flip = if facing < 0 { -1.0 } else { 1.0 };
        let t = Affine::translate(ground).then(Affine::scale(flip / sy, sy));
        batch.draw_mesh(m, &t, &Tint::default());
        true
    }

    /// Graphic of a figure (origin on the ground), e.g. for the main menu.
    pub fn character_mesh(&self, id: &str) -> Option<&Mesh> {
        self.characters.get(id)
    }

    /// Graphic of an enemy kind (origin in the center); `air` = jump pose, if present.
    pub fn creature_mesh(&self, kind: &str, air: bool) -> Option<&Mesh> {
        let look = self.looks.get(kind)?;
        Some(look.air.as_ref().filter(|_| air).unwrap_or(&look.idle))
    }

    /// NPC on the ground `ground`, facing `facing`; `scale` 1 = game size. `false` if the
    /// figure has no graphic of its own.
    pub fn draw_character(
        &self,
        batch: &mut ShapeBatch,
        id: &str,
        ground: Vec2,
        facing: i8,
        scale: f32,
    ) -> bool {
        let Some(m) = self.characters.get(id) else {
            return false;
        };
        let flip = if facing < 0 { -1.0 } else { 1.0 };
        let t = Affine::translate(ground).then(Affine::scale(flip * scale, scale));
        batch.draw_mesh(m, &t, &Tint::default());
        true
    }

    /// Object `name` (`truhe`, `quellstein`, `schalter`, `heilpflanze`) on the ground `ground`.
    pub fn draw_object(&self, batch: &mut ShapeBatch, name: &str, ground: Vec2, on: bool) {
        if let Some((off, on_mesh)) = self.objects.get(name) {
            batch.draw_mesh(
                if on { on_mesh } else { off },
                &Affine::translate(ground),
                &Tint::default(),
            );
        }
    }

    /// Projectile: pollen ball, or with `spark` a spark (`Some(true)` = glows on the ground).
    pub fn draw_shot(batch: &mut ShapeBatch, pos: Vec2, spark: Option<bool>, time: f32) {
        match spark {
            None => {
                batch.fill_circle(pos, 11.0, POLLEN_GLOW);
                batch.fill_circle(pos, 7.5, OUTLINE);
                batch.fill_circle(pos, 6.0, POLLEN);
            }
            Some(landed) => {
                // flickering spark; on the ground a flat ember spot
                let flicker = 1.0 + 0.15 * (time * 23.0 + pos.x * 0.1).sin();
                let r = if landed { 7.0 } else { 5.5 } * flicker;
                batch.fill_circle(pos, r * 2.2, EMBER_GLOW);
                if landed {
                    for dx in [-0.9, 0.9] {
                        batch.fill_circle(pos + Vec2::new(r * dx, 2.0), r * 0.7, EMBER);
                    }
                }
                batch.fill_circle(pos, r, EMBER);
                batch.fill_circle(pos, r * 0.5, EMBER_CORE);
            }
        }
    }

    /// Loot: gleam drops hover slightly, other items as a glitter stone.
    /// Icon of an item (HUD, menus), center `pos`, `scale` 1 = game size.
    /// Image of an item (its own, gleam drop or the generic one).
    fn item_mesh(&self, item: &str) -> &Mesh {
        if item == "glanztropfen" {
            &self.glanztropfen
        } else {
            self.items.get(item).unwrap_or(&self.item)
        }
    }

    pub fn draw_loot_icon(&self, batch: &mut ShapeBatch, item: &str, pos: Vec2, scale: f32) {
        let mesh = self.item_mesh(item);
        let t = Affine::translate(pos).then(Affine::scale(scale, scale));
        batch.draw_mesh(mesh, &t, &Tint::default());
    }

    pub fn draw_loot(&self, batch: &mut ShapeBatch, item: &str, pos: Vec2, time: f32) {
        let mesh = self.item_mesh(item);
        let bob = (time * 4.0 + pos.x * 0.05).sin() * 1.5;
        batch.draw_mesh(
            mesh,
            &Affine::translate(pos + Vec2::new(0.0, bob)),
            &Tint::default(),
        );
    }
}

/// Frost wave: behind the front fresh frost as ice spikes (deals damage), in front of it
/// hoarfrost creeps as a warning; only inside the hall (`hall` = edges, fresh frost length).
fn draw_frost_wave(
    batch: &mut ShapeBatch,
    front: Vec2,
    facing: i8,
    hall: (f32, f32, f32),
    time: f32,
) {
    let (left, right, fresh) = hall;
    let dir = if facing < 0 { -1.0 } else { 1.0 };
    let floor = front.y;
    let inside = |x: f32| x >= left && x <= right;
    // fresh frost
    let mut x = front.x;
    let mut k = 0u32;
    while (front.x - x) * dir <= fresh {
        if inside(x) {
            #[allow(clippy::cast_precision_loss)]
            let h = 16.0 + ((k as f32 * 1.7 + time * 9.0).sin() * 0.5 + 0.5) * 12.0;
            let w = 7.0;
            batch.fill_polygon(
                &[
                    Vec2::new(x - w - 1.5, floor + 1.0),
                    Vec2::new(x, floor - h - 2.5),
                    Vec2::new(x + w + 1.5, floor + 1.0),
                ],
                OUTLINE,
            );
            batch.fill_polygon(
                &[
                    Vec2::new(x - w, floor),
                    Vec2::new(x, floor - h),
                    Vec2::new(x + w, floor),
                ],
                Color::hex(0xe8f6ff),
            );
        }
        x -= dir * 13.0;
        k += 1;
    }
    // hoarfrost creeps ahead
    for j in 0..7 {
        #[allow(clippy::cast_precision_loss)]
        let x = front.x + dir * (10.0 + j as f32 * 13.0);
        if !inside(x) {
            continue;
        }
        #[allow(clippy::cast_precision_loss)]
        let a = (1.0 - j as f32 / 7.0) * 0.9;
        batch.stroke_polyline(
            &[
                Vec2::new(x - 4.0, floor),
                Vec2::new(x, floor - 6.0),
                Vec2::new(x + 4.0, floor),
            ],
            2.0,
            Color::rgba(0.37, 0.66, 0.82, a),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn art_loads_for_every_kind() {
        let art = CreatureArt::load();
        for k in crate::sandbox::creature_kinds() {
            assert!(
                art.looks.contains_key(k.name.as_str()),
                "Grafik für {} fehlt",
                k.name
            );
        }
        assert!(art.looks["grashuepfer"].air.is_some());
        for c in elora_adventure::Content::builtin().characters.keys() {
            assert!(
                art.characters.contains_key(c.as_str()),
                "Grafik für NPC {c} fehlt"
            );
        }
        assert_eq!(art.objects.len(), 4);
    }

    #[test]
    #[ignore = "erzeugt nur eine Datei zur Sichtprüfung"]
    #[allow(clippy::too_many_lines)]
    fn creature_sheet() {
        use elora_sim::creature::diver;
        let art = CreatureArt::load();
        let kinds = crate::sandbox::creature_kinds();
        let mut batch = ShapeBatch::default();
        let ground = 200.0;
        batch.fill_rect(
            Vec2::new(0.0, ground),
            Vec2::new(5000.0, ground + 40.0),
            Color::hex(0x8fbf7a),
        );
        let mut x = 60.0;
        let mut put =
            |name: &str, airborne: bool, stunned: bool, hit: Option<u64>, facing: i8, mode: u8| {
                let k = kinds.iter().find(|k| k.name == name).unwrap();
                let c = SceneCreature {
                    id: 0,
                    kind: name.into(),
                    pos: Vec2::new(
                        x,
                        ground - k.size[1] / 2.0 - if airborne { 30.0 } else { 0.0 },
                    ),
                    facing,
                    health: k.health / 2,
                    max_health: k.health,
                    since_hit: hit,
                    stunned,
                    airborne,
                    boss: false,
                    mode,
                    grow: 1.0,
                    // flight poses in an arc (tilt visible)
                    vel: if airborne {
                        Vec2::new(f32::from(facing) * 6.0, -4.0)
                    } else {
                        Vec2::ZERO
                    },
                    size: k.size(),
                    goal: elora_sim::Vec2::ZERO,
                    hall: None,
                };
                // collision box for checking
                let (hx, hy) = (k.size[0] / 2.0, k.size[1] / 2.0);
                let corners = [
                    c.pos + Vec2::new(-hx, -hy),
                    c.pos + Vec2::new(hx, -hy),
                    c.pos + Vec2::new(hx, hy),
                    c.pos + Vec2::new(-hx, hy),
                    c.pos + Vec2::new(-hx, -hy),
                ];
                batch.stroke_polyline(&corners, 1.0, Color::rgba(1.0, 0.0, 0.0, 0.5));
                art.draw(&mut batch, &c, 0.3);
                x += k.size[0].max(60.0) + 40.0;
            };
        put("stachelkaefer", false, false, None, 1, 0);
        put("stachelkaefer", false, true, Some(20), -1, 0);
        put("pollenblaeser", false, false, None, 1, 0);
        put("grashuepfer", false, false, None, 1, 0);
        put("grashuepfer", true, false, None, 1, 0);
        put("wirrbiene", true, false, None, 1, 0);
        put("brummbaer", true, false, None, 1, diver::CIRCLE);
        put("brummbaer", true, false, None, 1, diver::DIVE);
        put("brummbaer", false, false, Some(10), -1, diver::STUNNED);
        put("wurzelschlange", false, false, None, 1, 0);
        put("wurzelschlange", false, false, None, -1, 1);
        put("eichhornpirat", false, false, None, -1, 0);
        put("pilzwicht", false, false, None, 1, 0);
        put("pilzkind", false, false, None, 1, 0);
        put("wurzelwaechter", false, false, None, -1, 0);
        put("wurzelwaechter", false, false, None, -1, 1);
        put("wurzelwaechter", false, false, None, -1, 2);
        put("wurzelwaechter", false, false, Some(10), -1, 3);
        put("sandkrabbe", false, false, None, 1, 0);
        put("duenenwurm", false, false, None, 1, 0);
        put("duenenwurm", false, false, None, 1, 1);
        put("duenenwurm", true, false, None, 1, 2);
        put("funkenmotte", true, false, None, 1, 0);
        for mode in [1, 2, 3, 4] {
            put("sandschlange", mode == 3, false, None, 1, mode);
        }
        put("eiszapfen", false, false, None, 1, 0);
        put("schneeballrobbe", false, false, None, 1, 0);
        put("schneeballrobbe", false, false, None, -1, 1);
        put("fledermaus", true, false, None, 1, 0);
        put("fledermaus", true, false, None, 1, 1);
        put("frostgeist", true, false, None, 1, 0);
        for mode in [0, 1, 2, 3] {
            put("kristella", mode != 3, false, None, 1, mode);
        }
        put("schneebrocken", false, false, None, 1, 0);
        // Elora for size comparison (box 28)
        batch.fill_circle(Vec2::new(x, ground - 14.0), 14.0, Color::hex(0xf2c14e));
        CreatureArt::draw_shot(&mut batch, Vec2::new(x + 80.0, ground - 60.0), None, 0.0);
        CreatureArt::draw_shot(
            &mut batch,
            Vec2::new(x + 100.0, ground - 60.0),
            Some(false),
            0.0,
        );
        CreatureArt::draw_shot(
            &mut batch,
            Vec2::new(x + 120.0, ground - 6.0),
            Some(true),
            0.0,
        );
        art.draw_loot(
            &mut batch,
            "glanztropfen",
            Vec2::new(x + 140.0, ground - 8.0),
            0.0,
        );
        art.draw_loot(
            &mut batch,
            "bernstein",
            Vec2::new(x + 180.0, ground - 8.0),
            0.0,
        );
        // NPCs and objects (A1.7) on a second ground line
        let ground2 = 420.0;
        batch.fill_rect(
            Vec2::new(0.0, ground2),
            Vec2::new(900.0, ground2 + 40.0),
            Color::hex(0x8fbf7a),
        );
        for (i, c) in ["oma", "klonk", "lotte", "tueftel", "pip"]
            .iter()
            .enumerate()
        {
            #[allow(clippy::cast_precision_loss)]
            let x = 60.0 + i as f32 * 70.0;
            art.draw_character(&mut batch, c, Vec2::new(x, ground2), 1, 1.0);
        }
        batch.fill_circle(Vec2::new(420.0, ground2 - 14.0), 14.0, Color::hex(0xf2c14e));
        for (i, (name, on)) in [
            ("truhe", false),
            ("truhe", true),
            ("quellstein", false),
            ("quellstein", true),
            ("schalter", false),
            ("schalter", true),
            ("heilpflanze", false),
            ("heilpflanze", true),
        ]
        .iter()
        .enumerate()
        {
            #[allow(clippy::cast_precision_loss)]
            let x = 480.0 + i as f32 * 52.0;
            art.draw_object(&mut batch, name, Vec2::new(x, ground2), *on);
        }
        let svg = batch.debug_svg(Vec2::ZERO, Vec2::new(5000.0, 480.0), Color::hex(0xa9cde8));
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/creatures.svg"),
            svg,
        )
        .unwrap();
    }
}

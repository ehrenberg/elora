//! Gegner, ihre Geschosse und Beute zeichnen (A1.2) aus `assets/adventure/`.
//!
//! Gegner sind in Welteinheiten gezeichnet, Ursprung in der Mitte der Kollisionsbox,
//! Blick nach rechts; nach links werden sie gespiegelt.

use std::collections::HashMap;

use elora_client::scene::SceneCreature;
use elora_render::{Affine, Color, Mesh, ShapeBatch, SvgAsset, Tint};
use elora_sim::Vec2;

/// Wie lange der Lebensbalken nach einem Treffer sichtbar bleibt (Ticks, E-238).
const BAR_TICKS: u64 = 150;
const OUTLINE: Color = Color::hex(0x2b2b2b);
const BAR_BACK: Color = Color::rgba(0.118, 0.165, 0.212, 0.55);
const BAR: Color = Color::hex(0xe05a7a);
const POLLEN: Color = Color::hex(0xf8dd6e);
const POLLEN_GLOW: Color = Color::rgba(0.95, 0.76, 0.31, 0.35);
const STUN: Color = Color::hex(0xf2c14e);

/// Teile einer Gegnergrafik: `idle` und optional `air`.
#[derive(Debug)]
struct Look {
    idle: Mesh,
    air: Option<Mesh>,
    /// Hüter: Sturzflug und benommen (R2-M2.1).
    dive: Option<Mesh>,
    stunned: Option<Mesh>,
    /// Versteckt im Boden (Wurzelschlange, Zustand 0).
    hidden: Option<Mesh>,
}

#[derive(Debug)]
pub struct CreatureArt {
    looks: HashMap<&'static str, Look>,
    glanztropfen: Mesh,
    item: Mesh,
    /// Eigene Bilder einzelner Gegenstände.
    items: HashMap<&'static str, Mesh>,
    /// NPCs (A1.7), Ursprung am Boden.
    characters: HashMap<&'static str, Mesh>,
    /// Objekte mit zwei Zuständen (aus, an), Ursprung am Boden.
    objects: HashMap<&'static str, (Mesh, Mesh)>,
}

macro_rules! adventure_svgs {
    ($dir:literal: $($name:literal),* $(,)?) => {
        &[$(($name, include_bytes!(concat!("../../../assets/adventure/", $dir, "/", $name, ".svg")))),*]
    };
}

const CHARACTER_FILES: &[(&str, &[u8])] = adventure_svgs!("characters": "oma", "klonk", "lotte", "tueftel", "pip", "wegweiser", "wabe", "hummel", "plumm", "pilzkind");
/// Gegenstände mit eigenem Bild (R2-M2.1); alle anderen zeigen `item.svg`.
const ITEM_FILES: &[(&str, &[u8])] = adventure_svgs!("items": "biene", "quellfunke", "wabenhut");
/// Objekt und die Namen seiner beiden Teile (aus, an).
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
    "pilzkind"
);

impl CreatureArt {
    /// # Panics
    /// Wenn ein eingebettetes Asset fehlerhaft ist (wird von Tests abgedeckt).
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

    /// Gegner mit Lebensbalken nach Treffern; unbekannte Arten als Kreis.
    pub fn draw(&self, batch: &mut ShapeBatch, c: &SceneCreature, time: f32) {
        let flip = if c.facing < 0 { -1.0 } else { 1.0 };
        let t = Affine::translate(c.pos).then(Affine::scale(flip, 1.0));
        let flash = c.since_hit.is_some_and(|t| t < 6);
        let tint = if flash {
            Tint {
                alpha: Some(0.6),
                ..Tint::default()
            }
        } else {
            Tint::default()
        };
        if let Some(look) = self.looks.get(c.kind.as_str()) {
            use elora_sim::creature::diver;
            let mesh = match c.mode {
                elora_sim::creature::burrow::HIDDEN => look.hidden.as_ref(),
                diver::DIVE | diver::AIM => look.dive.as_ref(),
                diver::STUNNED => look.stunned.as_ref(),
                _ => None,
            }
            .or_else(|| look.air.as_ref().filter(|_| c.airborne))
            .unwrap_or(&look.idle);
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

    /// Grafik einer Figur (Ursprung am Boden), z. B. für das Hauptmenü.
    pub fn character_mesh(&self, id: &str) -> Option<&Mesh> {
        self.characters.get(id)
    }

    /// Grafik einer Gegnerart (Ursprung in der Mitte); `air` = Sprung-Pose, falls vorhanden.
    pub fn creature_mesh(&self, kind: &str, air: bool) -> Option<&Mesh> {
        let look = self.looks.get(kind)?;
        Some(look.air.as_ref().filter(|_| air).unwrap_or(&look.idle))
    }

    /// NPC am Boden `ground`, Blick `facing`; `scale` 1 = Spielgröße. `false`, wenn die
    /// Figur keine eigene Grafik hat.
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

    /// Objekt `name` (`truhe`, `quellstein`, `schalter`, `heilpflanze`) am Boden `ground`.
    pub fn draw_object(&self, batch: &mut ShapeBatch, name: &str, ground: Vec2, on: bool) {
        if let Some((off, on_mesh)) = self.objects.get(name) {
            batch.draw_mesh(
                if on { on_mesh } else { off },
                &Affine::translate(ground),
                &Tint::default(),
            );
        }
    }

    pub fn draw_shot(batch: &mut ShapeBatch, pos: Vec2) {
        batch.fill_circle(pos, 11.0, POLLEN_GLOW);
        batch.fill_circle(pos, 7.5, OUTLINE);
        batch.fill_circle(pos, 6.0, POLLEN);
    }

    /// Beute: Glanztropfen schweben leicht, andere Gegenstände als Glitzerstein.
    /// Symbol eines Gegenstands (HUD, Menüs), Mitte `pos`, `scale` 1 = Spielgröße.
    /// Bild eines Gegenstands (eigenes, Glanztropfen oder das allgemeine).
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
            Vec2::new(2000.0, ground + 40.0),
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
                };
                // Kollisionsbox zur Kontrolle
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
        // Elora zum Größenvergleich (Box 28)
        batch.fill_circle(Vec2::new(x, ground - 14.0), 14.0, Color::hex(0xf2c14e));
        CreatureArt::draw_shot(&mut batch, Vec2::new(x + 80.0, ground - 60.0));
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
        // NPCs und Objekte (A1.7) auf einer zweiten Bodenlinie
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
        let svg = batch.debug_svg(Vec2::ZERO, Vec2::new(2000.0, 480.0), Color::hex(0xa9cde8));
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/creatures.svg"),
            svg,
        )
        .unwrap();
    }
}

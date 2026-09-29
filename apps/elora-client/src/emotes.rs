//! Emotes (M5.9, E-091, E-103): Sprechblase über dem Kopf und Emote-Rad.
//!
//! Rad: Taste E halten, mit der Maus in eine Richtung zeigen, loslassen. Die
//! Richtung ist die Zielrichtung (Maus relativ zur Figur); Herz liegt oben,
//! weiter im Uhrzeigersinn.

use std::collections::HashMap;

use elora_client::scene::Scene;
use elora_protocol::msg::EMOTES;
use elora_render::{Affine, Color, Mesh, ShapeBatch, SvgAsset, Tint};
use elora_sim::{PHYS_SIZE, Vec2};

/// Anzeigedauer über dem Kopf (E-103).
const DURATION: f32 = 2.0;
const POP_IN: f32 = 0.15;
const FADE_OUT: f32 = 0.3;
/// Spitze der Blase über der Hitbox-Mitte: Figur ≈ 47 hoch, Boden bei +14.
const ABOVE_HEAD: f32 = PHYS_SIZE / 2.0 - 47.0 - 6.0;
/// Unterhalb dieser Mausentfernung (Welteinheiten) wählt das Rad nichts.
const DEAD_ZONE: f32 = 40.0;

fn load(data: &[u8], name: &str) -> Mesh {
    SvgAsset::load(data, 0.05)
        .unwrap_or_else(|e| panic!("assets/emotes/{name}: {e}"))
        .parts
        .into_iter()
        .map(|(_, m)| m)
        .next()
        .unwrap_or_else(|| panic!("assets/emotes/{name} ist leer"))
}

#[derive(Debug)]
pub struct Emotes {
    bubble: Mesh,
    symbols: Vec<Mesh>,
    /// Slot → (Emote, Alter in s).
    active: HashMap<usize, (u8, f32)>,
    /// Emote-Rad offen (Taste E gehalten).
    pub wheel_open: bool,
}

impl Emotes {
    /// # Panics
    /// Wenn ein eingebettetes Asset fehlerhaft ist (wird von Tests abgedeckt).
    pub fn new() -> Self {
        macro_rules! symbol {
            ($f:literal) => {
                load(include_bytes!(concat!("../../../assets/emotes/", $f)), $f)
            };
        }
        Self {
            bubble: symbol!("bubble.svg"),
            symbols: vec![
                symbol!("0-herz.svg"),
                symbol!("1-lachen.svg"),
                symbol!("2-wut.svg"),
                symbol!("3-traurig.svg"),
                symbol!("4-staunen.svg"),
                symbol!("5-frage.svg"),
                symbol!("6-gg.svg"),
                symbol!("7-schlaf.svg"),
            ],
            active: HashMap::new(),
            wheel_open: false,
        }
    }

    /// Emote über `slot` anzeigen (ersetzt ein laufendes).
    pub fn show(&mut self, slot: usize, emote: u8) {
        if emote < EMOTES {
            self.active.insert(slot, (emote, 0.0));
        }
    }

    pub fn update(&mut self, dt: f32) {
        for (_, age) in self.active.values_mut() {
            *age += dt;
        }
        self.active.retain(|_, (_, age)| *age < DURATION);
    }

    /// Blase mit Symbol; `scale` 1 = Weltgröße, `alpha` für Ein-/Ausblenden.
    fn draw_bubble(&self, batch: &mut ShapeBatch, tip: Vec2, emote: u8, scale: f32, alpha: f32) {
        let t = Affine::translate(tip).then(Affine::scale(scale, scale));
        let tint = Tint {
            alpha: Some(alpha),
            ..Tint::default()
        };
        batch.draw_mesh(&self.bubble, &t, &tint);
        if let Some(sym) = self.symbols.get(usize::from(emote)) {
            batch.draw_mesh(sym, &t, &tint);
        }
    }

    /// Laufende Emotes über den Figuren der Szene.
    pub fn draw(&self, batch: &mut ShapeBatch, scene: &Scene) {
        for c in &scene.chars {
            let Some(&(emote, age)) = self.active.get(&c.slot) else {
                continue;
            };
            // Aufpoppen mit leichtem Überschwingen, am Ende ausblenden
            let scale = if age < POP_IN {
                let x = age / POP_IN;
                x * (1.0 + 0.3 * (1.0 - x))
            } else {
                1.0
            };
            let alpha = ((DURATION - age) / FADE_OUT).clamp(0.0, 1.0);
            let tip = c.pos() + Vec2::new(0.0, ABOVE_HEAD);
            self.draw_bubble(batch, tip, emote, scale, alpha);
        }
    }

    /// Emote-Rad in Bildschirm-Pixeln, `s` = HUD-Skalierung.
    pub fn draw_wheel(&self, batch: &mut ShapeBatch, screen: Vec2, s: f32, selected: Option<u8>) {
        let center = screen * 0.5;
        let radius = 130.0 * s;
        batch.fill_circle(
            center,
            radius + 40.0 * s,
            Color::rgba(0.118, 0.165, 0.212, 0.45),
        );
        batch.fill_circle(center, 38.0 * s, Color::rgba(0.118, 0.165, 0.212, 0.5));
        for e in 0..EMOTES {
            let a = sector_angle(e);
            let dir = Vec2::new(a.cos(), a.sin());
            let sel = selected == Some(e);
            if sel {
                // hervorgehobener Sektor als Fächer
                let half = std::f32::consts::PI / 8.0;
                let steps = 12;
                let mut points = vec![center + dir * (40.0 * s)];
                for k in 0..=steps {
                    #[allow(clippy::cast_precision_loss)]
                    let b = a - half + 2.0 * half * k as f32 / steps as f32;
                    points.push(center + Vec2::new(b.cos(), b.sin()) * (radius + 40.0 * s));
                }
                batch.fill_polygon(&points, Color::rgba(1.0, 1.0, 1.0, 0.3));
            }
            let scale = if sel { 2.4 } else { 2.0 } * s;
            // Blase so verschieben, dass ihre Mitte (y = −17) auf dem Kreis liegt
            let tip = center + dir * radius + Vec2::new(0.0, 17.0 * scale);
            self.draw_bubble(batch, tip, e, scale, 1.0);
        }
    }
}

/// Richtung des Sektors `e` (rad): 0 oben, im Uhrzeigersinn je 45°.
fn sector_angle(e: u8) -> f32 {
    (-90.0 + f32::from(e) * 45.0).to_radians()
}

/// Gewähltes Emote für die Mausrichtung `aim` (relativ zur Figur), `None` in der Mitte.
pub fn selection(aim: Vec2) -> Option<u8> {
    if aim.length() < DEAD_ZONE {
        return None;
    }
    let deg = aim.y.atan2(aim.x).to_degrees() + 90.0;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 0..8
    let sector = ((deg / 45.0).round().rem_euclid(8.0)) as u8;
    Some(sector)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheel_selection_by_direction() {
        assert_eq!(selection(Vec2::new(0.0, -100.0)), Some(0), "oben = Herz");
        assert_eq!(selection(Vec2::new(100.0, -100.0)), Some(1), "oben rechts");
        assert_eq!(selection(Vec2::new(100.0, 0.0)), Some(2), "rechts");
        assert_eq!(selection(Vec2::new(0.0, 100.0)), Some(4), "unten");
        assert_eq!(selection(Vec2::new(-100.0, 0.0)), Some(6), "links");
        assert_eq!(selection(Vec2::new(-100.0, -90.0)), Some(7), "oben links");
        assert_eq!(selection(Vec2::new(10.0, 5.0)), None, "Mitte");
    }

    /// Sichtprüfung: `cargo test -p elora-client --bin elora emote_sheet -- --ignored`,
    /// danach `cargo xtask svg-preview target/emotes.svg target/emotes.png 1000`.
    #[test]
    #[ignore = "erzeugt nur eine Datei zur Sichtprüfung"]
    fn emote_sheet() {
        let e = Emotes::new();
        let mut batch = ShapeBatch::default();
        e.draw_wheel(&mut batch, Vec2::new(620.0, 720.0), 1.0, Some(1));
        for k in 0..EMOTES {
            let tip = Vec2::new(
                700.0 + f32::from(k % 2) * 140.0,
                130.0 + f32::from(k / 2) * 150.0,
            );
            e.draw_bubble(&mut batch, tip, k, 4.0, 1.0);
        }
        let svg = batch.debug_svg(
            Vec2::default(),
            Vec2::new(1000.0, 720.0),
            Color::hex(0x8fb8d9),
        );
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/emotes.svg"),
            svg,
        )
        .unwrap();
    }

    #[test]
    fn emotes_expire() {
        let mut e = Emotes::new();
        e.show(3, 5);
        e.show(4, EMOTES); // ungültig
        assert_eq!(e.active.len(), 1);
        e.update(DURATION - 0.1);
        assert_eq!(e.active.len(), 1);
        e.update(0.2);
        assert!(e.active.is_empty());
    }
}

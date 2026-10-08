//! Emotes (M5.9, E-091, E-103): speech bubble above the head and emote wheel.
//!
//! Wheel: hold key E, point the mouse in a direction, release. The
//! direction is the aim direction (mouse relative to the figure); heart is at the top,
//! continuing clockwise.

use std::collections::HashMap;

use elora_client::scene::Scene;
use elora_protocol::msg::EMOTES;
use elora_render::{Affine, Color, Mesh, ShapeBatch, SvgAsset, Tint};
use elora_sim::{PHYS_SIZE, Vec2};

/// Display duration above the head (E-103).
const DURATION: f32 = 2.0;
const POP_IN: f32 = 0.15;
const FADE_OUT: f32 = 0.3;
/// Tip of the bubble above the hitbox center: figure ≈ 47 high, ground at +14.
const ABOVE_HEAD: f32 = PHYS_SIZE / 2.0 - 47.0 - 6.0;
/// Below this mouse distance (world units) the wheel selects nothing.
const DEAD_ZONE: f32 = 40.0;

fn load(data: &[u8], name: &str) -> Mesh {
    SvgAsset::load(data, 0.05)
        .unwrap_or_else(|e| panic!("assets/emotes/{name}: {e}"))
        .parts
        .into_iter()
        .map(|(_, m)| m)
        .next()
        .unwrap_or_else(|| panic!("assets/emotes/{name} is empty"))
}

#[derive(Debug)]
pub struct Emotes {
    bubble: Mesh,
    symbols: Vec<Mesh>,
    /// Slot → (emote, age in s).
    active: HashMap<usize, (u8, f32)>,
    /// Emote wheel open (key E held).
    pub wheel_open: bool,
    /// Emotes shown since the last [`Emotes::take_new`] (slots, for the sound).
    new: Vec<usize>,
}

impl Emotes {
    /// # Panics
    /// If an embedded asset is faulty (covered by tests).
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
            new: Vec::new(),
        }
    }

    /// Show an emote above `slot` (replaces a running one).
    pub fn show(&mut self, slot: usize, emote: u8) {
        if emote < EMOTES {
            self.active.insert(slot, (emote, 0.0));
            self.new.push(slot);
        }
    }

    /// Slots that have received an emote since the last call.
    pub fn take_new(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.new)
    }

    pub fn update(&mut self, dt: f32) {
        for (_, age) in self.active.values_mut() {
            *age += dt;
        }
        self.active.retain(|_, (_, age)| *age < DURATION);
    }

    /// Bubble with icon; `scale` 1 = world size, `alpha` for fading in/out.
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

    /// Running emotes above the figures of the scene.
    pub fn draw(&self, batch: &mut ShapeBatch, scene: &Scene) {
        for c in &scene.chars {
            let Some(&(emote, age)) = self.active.get(&c.slot) else {
                continue;
            };
            // pop up with a slight overshoot, fade out at the end
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

    /// Emote wheel in screen pixels, `s` = HUD scale.
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
                // highlighted sector as a fan
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
            // move the bubble so that its center (y = −17) lies on the circle
            let tip = center + dir * radius + Vec2::new(0.0, 17.0 * scale);
            self.draw_bubble(batch, tip, e, scale, 1.0);
        }
    }
}

/// Direction of sector `e` (rad): 0 at the top, clockwise in steps of 45°.
fn sector_angle(e: u8) -> f32 {
    (-90.0 + f32::from(e) * 45.0).to_radians()
}

/// Chosen emote for the mouse direction `aim` (relative to the figure), `None` in the middle.
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
        assert_eq!(selection(Vec2::new(0.0, -100.0)), Some(0), "up = heart");
        assert_eq!(selection(Vec2::new(100.0, -100.0)), Some(1), "up right");
        assert_eq!(selection(Vec2::new(100.0, 0.0)), Some(2), "right");
        assert_eq!(selection(Vec2::new(0.0, 100.0)), Some(4), "down");
        assert_eq!(selection(Vec2::new(-100.0, 0.0)), Some(6), "left");
        assert_eq!(selection(Vec2::new(-100.0, -90.0)), Some(7), "up left");
        assert_eq!(selection(Vec2::new(10.0, 5.0)), None, "center");
    }

    /// Visual inspection: `cargo test -p elora-client --bin elora emote_sheet -- --ignored`,
    /// then `cargo xtask svg-preview target/emotes.svg target/emotes.png 1000`.
    #[test]
    #[ignore = "only writes a file for visual inspection"]
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
        e.show(4, EMOTES); // invalid
        assert_eq!(e.active.len(), 1);
        e.update(DURATION - 0.1);
        assert_eq!(e.active.len(), 1);
        e.update(0.2);
        assert!(e.active.is_empty());
    }
}

//! Main menu background (E-292): a piece of Tauwinkel with figures, animals and enemies,
//! varies with the time of day (E-291: morning, day, evening, night by the system clock).
//!
//! All in screen coordinates; graphics from existing assets (map decoration, figures, enemies).

#![allow(clippy::cast_precision_loss, clippy::many_single_char_names)]

use elora_render::{Affine, Color, Mesh, ShapeBatch, Tint};
use elora_sim::Vec2;

use crate::creatures::CreatureArt;
use crate::map_art::MapArt;

/// Mood of a time of day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mood {
    pub sky_top: [f32; 3],
    pub sky_bottom: [f32; 3],
    /// Light on everything (multiplied).
    pub light: [f32; 3],
    /// 0 = day, 1 = deep night (stars, windows, fireflies).
    pub night: f32,
    /// Lanterns glow (evening and night).
    pub lamps: f32,
    /// Sun height 0 (horizon) to 1 (top); below 0 = not visible.
    pub sun: f32,
}

const MORNING: Mood = Mood {
    sky_top: [0.62, 0.72, 0.88],
    sky_bottom: [0.99, 0.84, 0.74],
    light: [1.0, 0.94, 0.88],
    night: 0.0,
    lamps: 0.0,
    sun: 0.25,
};
const DAY: Mood = Mood {
    sky_top: [0.66, 0.80, 0.91],
    sky_bottom: [0.91, 0.95, 0.97],
    light: [1.0, 1.0, 1.0],
    night: 0.0,
    lamps: 0.0,
    sun: 0.85,
};
const EVENING: Mood = Mood {
    sky_top: [0.42, 0.40, 0.62],
    sky_bottom: [0.98, 0.66, 0.42],
    light: [1.0, 0.82, 0.68],
    night: 0.15,
    lamps: 1.0,
    sun: 0.12,
};
const NIGHT: Mood = Mood {
    sky_top: [0.07, 0.10, 0.22],
    sky_bottom: [0.20, 0.27, 0.45],
    light: [0.42, 0.48, 0.68],
    night: 1.0,
    lamps: 1.0,
    sun: -1.0,
};

/// Mood at clock time `hour` (0..24): four phases with soft transitions.
pub fn mood(hour: f32) -> Mood {
    // (hour, mood); blended in between
    const KEYS: [(f32, Mood); 9] = [
        (0.0, NIGHT),
        (5.5, NIGHT),
        (7.0, MORNING),
        (9.5, MORNING),
        (10.5, DAY),
        (16.5, DAY),
        (17.5, EVENING),
        (20.5, EVENING),
        (21.5, NIGHT),
    ];
    let h = hour.rem_euclid(24.0);
    let (mut a, mut b) = (KEYS[KEYS.len() - 1], (24.0, NIGHT));
    for w in KEYS.windows(2) {
        if h >= w[0].0 && h < w[1].0 {
            (a, b) = (w[0], w[1]);
        }
    }
    let t = ((h - a.0) / (b.0 - a.0)).clamp(0.0, 1.0);
    let t = t * t * (3.0 - 2.0 * t);
    let mix = |x: f32, y: f32| x + (y - x) * t;
    let mix3 = |x: [f32; 3], y: [f32; 3]| std::array::from_fn(|i| mix(x[i], y[i]));
    Mood {
        sky_top: mix3(a.1.sky_top, b.1.sky_top),
        sky_bottom: mix3(a.1.sky_bottom, b.1.sky_bottom),
        light: mix3(a.1.light, b.1.light),
        night: mix(a.1.night, b.1.night),
        lamps: mix(a.1.lamps, b.1.lamps),
        sun: mix(a.1.sun, b.1.sun),
    }
}

/// Current local time in hours.
pub fn local_hour() -> f32 {
    use chrono::Timelike;
    let now = chrono::Local::now();
    now.hour() as f32 + now.minute() as f32 / 60.0
}

fn rgb(c: [f32; 3]) -> Color {
    Color::rgb(c[0], c[1], c[2])
}

/// Drawing context of the scene.
struct Scene<'a> {
    batch: &'a mut ShapeBatch,
    map: &'a MapArt,
    creatures: &'a CreatureArt,
    w: f32,
    ground: f32,
    /// World units → pixels.
    k: f32,
    light: Tint,
    t: f32,
}

impl Scene<'_> {
    fn x(&self, frac: f32) -> f32 {
        self.w * frac
    }

    fn mesh(&mut self, mesh: &Mesh, at: Vec2, scale: f32, flip: bool, rot: f32, tint: &Tint) {
        let f = if flip { -1.0 } else { 1.0 };
        let t = Affine::translate(at)
            .then(Affine::rotate(rot.to_radians()))
            .then(Affine::scale(f * scale, scale));
        self.batch.draw_mesh(mesh, &t, tint);
    }

    /// Decoration `name` on the ground at `frac` of the width.
    fn decor(&mut self, name: &str, frac: f32, size: f32, flip: bool) {
        let at = Vec2::new(self.x(frac), self.ground);
        self.decor_at(name, at, size, flip, 0.0);
    }

    fn decor_at(&mut self, name: &str, at: Vec2, size: f32, flip: bool, rot: f32) {
        let map = self.map;
        if let Some(m) = map.builtin(name) {
            let (k, tint) = (self.k * size, self.light.clone());
            self.mesh(m, at, k, flip, rot, &tint);
        }
    }

    fn character(&mut self, id: &str, x: f32, facing: f32) {
        let art = self.creatures;
        if let Some(m) = art.character_mesh(id) {
            let (k, tint) = (self.k * 1.6, self.light.clone());
            self.mesh(m, Vec2::new(x, self.ground), k, facing < 0.0, 0.0, &tint);
        }
    }

    fn creature(&mut self, kind: &str, x: f32, lift: f32, half_height: f32, facing: f32) {
        let art = self.creatures;
        if let Some(m) = art.creature_mesh(kind, lift > 1.0) {
            let k = self.k * 1.5;
            let at = Vec2::new(x, self.ground - (half_height + lift) * k);
            let tint = self.light.clone();
            self.mesh(m, at, k, facing < 0.0, 0.0, &tint);
        }
    }

    /// Walk back and forth between `a` and `b` (fractions of the width); returns x and facing.
    fn stroll(&self, a: f32, b: f32, speed: f32, phase: f32) -> (f32, f32) {
        let s = (self.t * speed + phase).sin();
        let ds = (self.t * speed + phase).cos();
        let frac = a + (b - a) * (s * 0.5 + 0.5);
        (self.x(frac), if ds >= 0.0 { 1.0 } else { -1.0 })
    }
}

/// Windows of the buildings (center, size in world units; from `tauwinkel_buildings.py`).
/// Windows: center x, y and width, height (width 0 = unused).
type Window = (f32, f32, f32, f32);

const WINDOWS: [(&str, [Window; 3]); 4] = [
    (
        "haus-elora",
        [
            (0.0, -186.0, 22.0, 24.0),
            (34.0, -86.0, 28.0, 34.0),
            (0.0, 0.0, 0.0, 0.0),
        ],
    ),
    (
        "haus-oma",
        [
            (-38.0, -68.0, 30.0, 34.0),
            (0.0, 0.0, 0.0, 0.0),
            (0.0, 0.0, 0.0, 0.0),
        ],
    ),
    (
        "laden",
        [
            (0.0, -192.0, 22.0, 24.0),
            (-28.0, -67.0, 96.0, 58.0),
            (0.0, 0.0, 0.0, 0.0),
        ],
    ),
    (
        "werkstatt",
        [
            (44.0, -48.0, 30.0, 30.0),
            (-40.0, -132.0, 26.0, 24.0),
            (40.0, -132.0, 26.0, 24.0),
        ],
    ),
];

/// Draw the background: `screen` in pixels, `s` UI scale, `t` running time (s), `hour` 0..24.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
pub fn draw(
    batch: &mut ShapeBatch,
    map: &MapArt,
    creatures: &CreatureArt,
    screen: Vec2,
    s: f32,
    t: f32,
    hour: f32,
) -> Mood {
    let (w, h) = (screen.x, screen.y);
    let m = mood(hour);
    batch.fill_rect_vgradient(Vec2::ZERO, screen, rgb(m.sky_top), rgb(m.sky_bottom));

    // stars and moon
    if m.night > 0.05 {
        for i in 0..70u32 {
            let fx = ((i * 7919) % 1000) as f32 / 1000.0;
            let fy = ((i * 104_729) % 1000) as f32 / 1000.0;
            let twinkle = 0.6 + 0.4 * (t * 1.7 + i as f32).sin();
            let c = Color::rgba(1.0, 0.97, 0.88, m.night * twinkle);
            batch.fill_circle(
                Vec2::new(fx * w, fy * h * 0.55),
                (1.0 + (i % 3) as f32 * 0.6) * s,
                c,
            );
        }
        let moon = Vec2::new(w * 0.18, h * 0.16);
        batch.fill_circle(moon, 46.0 * s, Color::rgba(1.0, 0.97, 0.85, 0.12 * m.night));
        batch.fill_circle(moon, 26.0 * s, Color::rgba(0.98, 0.95, 0.82, m.night));
        batch.fill_circle(
            moon + Vec2::new(9.0 * s, -6.0 * s),
            22.0 * s,
            Color::rgba(m.sky_top[0], m.sky_top[1], m.sky_top[2], m.night),
        );
    }
    // sun
    if m.sun > 0.0 {
        let p = Vec2::new(w * 0.22, h * (0.62 - m.sun * 0.5));
        batch.fill_circle(p, 70.0 * s, Color::rgba(1.0, 0.92, 0.6, 0.18));
        batch.fill_circle(p, 40.0 * s, Color::rgba(1.0, 0.86, 0.45, 0.95));
    }

    let ground = h - 64.0 * s;
    let k = h / 720.0 * 0.62;
    let light = Tint {
        multiply: Some(rgb(m.light)),
        ..Tint::default()
    };
    let mut sc = Scene {
        batch,
        map,
        creatures,
        w,
        ground,
        k,
        light,
        t,
    };

    // clouds drift
    for (i, (name, y, speed, size)) in [
        ("cloud-1", 0.12, 9.0, 1.1),
        ("cloud-2", 0.24, 6.0, 0.9),
        ("cloud-3", 0.08, 12.0, 0.8),
        ("cloud-1", 0.32, 5.0, 0.7),
    ]
    .into_iter()
    .enumerate()
    {
        let span = w + 600.0 * s;
        let x = (i as f32 * 0.31 * span + t * speed * s).rem_euclid(span) - 300.0 * s;
        let cloud_tint = Tint {
            multiply: Some(Color::rgba(m.light[0], m.light[1], m.light[2], 0.9)),
            ..Tint::default()
        };
        if let Some(mesh) = map.builtin(name) {
            sc.mesh(mesh, Vec2::new(x, h * y), size * s, false, 0.0, &cloud_tint);
        }
    }
    // hills in two rows
    for (name, y, size) in [
        ("hills-far", ground - 40.0 * s, 1.1),
        ("hills-near", ground + 6.0 * s, 1.0),
    ] {
        if let Some(mesh) = map.builtin(name) {
            let step = crate::map_art::STRIP_WIDTH * size * s;
            let mut x = -(t * 2.0 * s).rem_euclid(step);
            let tint = sc.light.clone();
            while x < w + step {
                sc.mesh(mesh, Vec2::new(x, y), size * s, false, 0.0, &tint);
                x += step;
            }
        }
    }

    // ground
    let grass = Color::rgb(0.49 * m.light[0], 0.70 * m.light[1], 0.40 * m.light[2]);
    let earth = Color::rgb(0.63 * m.light[0], 0.45 * m.light[1], 0.31 * m.light[2]);
    sc.batch
        .fill_rect(Vec2::new(0.0, ground), Vec2::new(w, h), earth);
    sc.batch.fill_rect(
        Vec2::new(0.0, ground - 2.0 * s),
        Vec2::new(w, ground + 10.0 * s),
        grass,
    );

    // village: trees, houses, props
    for (name, frac, size, flip) in [
        ("tree-round", 0.02, 1.1, false),
        ("tree-pine", 0.205, 1.0, false),
        ("tree-round", 0.53, 1.0, true),
        ("tree-pine", 0.995, 1.1, true),
        ("haus-elora", 0.10, 1.0, false),
        ("brunnen", 0.255, 1.0, false),
        ("haus-oma", 0.38, 1.0, false),
        ("laden", 0.65, 1.0, false),
        ("werkstatt", 0.93, 1.0, true),
        ("fence", 0.175, 1.0, false),
        ("laterne", 0.205, 1.0, false),
        ("bank", 0.31, 1.0, false),
        ("karren", 0.47, 1.0, true),
        ("laterne", 0.565, 1.0, false),
        ("faesser", 0.72, 1.0, false),
        ("heuballen", 0.785, 1.0, false),
        ("holzstapel", 0.875, 1.0, false),
    ] {
        sc.decor(name, frac, size, flip);
    }
    // windows glow at night
    if m.night > 0.05 {
        for (name, frac, flip) in [
            ("haus-elora", 0.10, false),
            ("haus-oma", 0.38, false),
            ("laden", 0.65, false),
            ("werkstatt", 0.93, true),
        ] {
            let windows = WINDOWS
                .iter()
                .find(|w| w.0 == name)
                .map_or(&[][..], |w| &w.1[..]);
            let base = Vec2::new(sc.x(frac), ground);
            for &(cx, cy, ww, wh) in windows.iter().filter(|w| w.2 > 0.0) {
                let cx = if flip { -cx } else { cx };
                let c = base + Vec2::new(cx, cy) * k;
                let half = Vec2::new(ww, wh) * k * 0.5;
                sc.batch.fill_circle(
                    c,
                    ww.max(wh) * k,
                    Color::rgba(1.0, 0.8, 0.4, 0.12 * m.night),
                );
                sc.batch.fill_rect(
                    c - half,
                    c + half,
                    Color::rgba(1.0, 0.83, 0.45, 0.85 * m.night),
                );
            }
        }
    }
    // lanterns
    if m.lamps > 0.05 {
        for frac in [0.205, 0.565] {
            let c = Vec2::new(sc.x(frac), ground - 139.0 * k);
            sc.batch
                .fill_circle(c, 40.0 * k, Color::rgba(1.0, 0.85, 0.45, 0.18 * m.lamps));
            sc.batch
                .fill_circle(c, 9.0 * k, Color::rgba(1.0, 0.92, 0.6, 0.9 * m.lamps));
        }
    }
    // beds and flag
    for (name, frac) in [
        ("beet-bunt", 0.055),
        ("blumenkasten-bunt", 0.115),
        ("kraeuterbeet-bunt", 0.41),
        ("blumentopf-bunt", 0.62),
        ("beet-bunt", 0.69),
    ] {
        sc.decor(name, frac, 1.0, false);
    }
    let wave = (t * 1.6).sin() * 2.0;
    let at = Vec2::new(sc.x(0.455), ground);
    sc.decor_at("fahne-bunt", at, 1.0, false, wave);
    // smoke from the chimneys
    for (i, (frac, dx, dy)) in [(0.10, 48.0, -240.0), (0.65, -48.0, -246.0)]
        .into_iter()
        .enumerate()
    {
        for puff in 0..3 {
            let age = (t * 0.33 + puff as f32 / 3.0 + i as f32 * 0.17).fract();
            let p = Vec2::new(
                sc.x(frac) + dx * k + age * 30.0 * k,
                ground + dy * k - age * 120.0 * k,
            );
            let tint = Tint {
                multiply: Some(Color::rgba(
                    m.light[0],
                    m.light[1],
                    m.light[2],
                    (1.0 - age) * 0.8,
                )),
                ..Tint::default()
            };
            if let Some(mesh) = map.builtin("rauch") {
                let size = k * (0.8 + age);
                sc.mesh(mesh, p, size, false, 0.0, &tint);
            }
        }
    }

    // figures: some go for a walk
    sc.character("oma", sc.x(0.335), 1.0);
    let (x, f) = sc.stroll(0.13, 0.24, 0.35, 0.0);
    sc.character("pip", x, f);
    sc.character("lotte", sc.x(0.60), 1.0);
    sc.character("klonk", sc.x(0.745), -1.0);
    let (x, f) = sc.stroll(0.44, 0.52, 0.22, 1.3);
    sc.character("tueftel", x, f);
    // enemies: a beetle crawls, a hopper jumps now and then
    let (x, f) = sc.stroll(0.27, 0.36, 0.5, 2.0);
    sc.creature("stachelkaefer", x, 0.0, 13.0, f);
    let cycle = (t / 2.6).fract();
    let hop = if cycle < 0.3 {
        (cycle / 0.3 * std::f32::consts::PI).sin() * 60.0
    } else {
        0.0
    };
    let (x, f) = sc.stroll(0.02, 0.08, 0.18, 0.4);
    sc.creature("grashuepfer", x, hop, 14.0, f);

    // butterflies by day, fireflies at night
    for i in 0..6 {
        let fi = i as f32;
        let base = Vec2::new(
            w * (0.08 + fi * 0.16),
            ground - (60.0 + (fi * 37.0) % 70.0) * s,
        );
        let p = base
            + Vec2::new(
                (t * 0.7 + fi).sin() * 40.0 * s,
                (t * 1.3 + fi * 2.0).sin() * 18.0 * s,
            );
        if m.night < 0.5 {
            let flap = 1.0 - 0.35 * (t * 14.0 + fi).sin().abs();
            if let Some(mesh) = map.builtin("schmetterling") {
                let tint = sc.light.clone();
                let t2 = Affine::translate(p).then(Affine::scale(k * 1.2 * flap, k * 1.2));
                sc.batch.draw_mesh(mesh, &t2, &tint);
            }
        } else {
            let glow = 0.5 + 0.5 * (t * 2.0 + fi * 1.7).sin();
            sc.batch.fill_circle(
                p,
                7.0 * s,
                Color::rgba(0.9, 1.0, 0.5, 0.15 * glow * m.night),
            );
            sc.batch.fill_circle(
                p,
                2.2 * s,
                Color::rgba(0.95, 1.0, 0.6, 0.9 * glow * m.night),
            );
        }
    }
    // birds fly across the sky (not at night)
    if m.night < 0.6 {
        for (i, (y, speed)) in [(0.2, 40.0), (0.27, 55.0)].into_iter().enumerate() {
            let span = w + 200.0 * s;
            let x = (t * speed * s + i as f32 * span * 0.5).rem_euclid(span) - 100.0 * s;
            let bob = (t * 3.0 + i as f32).sin() * 6.0 * s;
            let at = Vec2::new(x, h * y + bob);
            sc.decor_at("vogel", at, 1.3, false, (t * 3.0).sin() * 8.0);
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moods_follow_the_clock() {
        assert_eq!(mood(3.0), NIGHT);
        assert_eq!(mood(8.0), MORNING);
        assert_eq!(mood(13.0), DAY);
        assert_eq!(mood(19.0), EVENING);
        assert_eq!(mood(23.0), NIGHT);
        // soft transition
        let m = mood(10.0);
        assert!(m.sun > MORNING.sun && m.sun < DAY.sun);
        assert!((mood(24.0).night - 1.0).abs() < 1e-6);
    }

    /// Visual inspection: `cargo test -p elora-client --bin elora menu_scene_sheets -- --ignored`
    /// writes `target/menu-<phase>.svg`.
    #[test]
    #[ignore = "erzeugt nur Dateien zur Sichtprüfung"]
    fn menu_scene_sheets() {
        let (map, creatures) = (MapArt::load(), CreatureArt::load());
        let screen = Vec2::new(1280.0, 720.0);
        for (name, hour) in [
            ("morgen", 8.0),
            ("tag", 13.0),
            ("abend", 19.0),
            ("nacht", 23.0),
        ] {
            let mut batch = ShapeBatch::default();
            draw(&mut batch, &map, &creatures, screen, 1.0, 3.0, hour);
            let svg = batch.debug_svg(Vec2::ZERO, screen, Color::rgb(0.0, 0.0, 0.0));
            let path = format!(
                "{}/../../target/menu-{name}.svg",
                env!("CARGO_MANIFEST_DIR")
            );
            std::fs::write(path, svg).unwrap();
        }
    }
}

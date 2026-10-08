//! Weather in the presentation (R2-W1, W1.2, E-329 to E-336): particles behind and in front of the
//! figures, splashes on surfaces, lightning and the color mood for the post shader.
//!
//! Pure presentation: particles live in world coordinates around the camera view and are
//! reused there; none of it affects the simulation.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use elora_map::{Map, Weather, WeatherKind};
use elora_render::{Camera, Color, Grade, ShapeBatch};
use elora_sim::{TILE_SIZE, Tile, Vec2};

use crate::settings::WeatherQuality;

/// Margin around the view in which particles keep living (units).
const MARGIN: f32 = 120.0;
/// At most this many new particles per frame (on entering, the picture fills immediately).
const SPAWN_PER_FRAME: usize = 600;
/// This long a lightning flash is visible (s) and a splash (s).
const BOLT_TIME: f32 = 0.22;
const SPLASH_TIME: f32 = 0.25;
/// This long the ground glows before a strike (s, like A-32).
const WARN_TIME: f32 = 0.9;

#[derive(Debug, Clone, Copy)]
struct Particle {
    kind: WeatherKind,
    pos: Vec2,
    vel: Vec2,
    /// in front of the figures (larger, stronger) or behind them
    front: bool,
    size: f32,
    /// rotation (leaves) and swaying (flakes)
    angle: f32,
    spin: f32,
    seed: f32,
    color: Color,
    /// in the ground or under a roof: do not draw (caves, houses, rock roofs)
    hidden: bool,
}

#[derive(Debug, Clone, Copy)]
struct Bolt {
    top: Vec2,
    ground: Vec2,
    age: f32,
    seed: u32,
}

/// Weather of the presentation.
#[derive(Debug, Default)]
pub struct WeatherView {
    particles: Vec<Particle>,
    splashes: Vec<(Vec2, f32)>,
    bolts: Vec<Bolt>,
    rng: u64,
    time: f32,
    next_bolt: f32,
    splash_carry: f32,
    flash: f32,
    grade: Option<Grade>,
    /// Thunder for the sound (W1.5): place and delay (s).
    thunder: Vec<(Vec2, f32)>,
    /// Wind with which the decoration currently sways (smoothed).
    wind: f32,
    /// Announced strikes from the simulation (adventure): place, age (s).
    warnings: Vec<(Vec2, f32)>,
    /// Top edge of the view (for lightning from the simulation).
    sky_top: f32,
    quality: WeatherQuality,
    /// How much the camera is under a roof or inside rock (smoothed, 0..1) – muffles the sounds.
    shelter: f32,
}

/// Top edge of the first ground (solid, platform, quicksand) below `from`, at most `reach` deep.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub fn surface_below(map: &Map, from: Vec2, reach: f32) -> Option<f32> {
    let ts = TILE_SIZE as f32;
    let (tx, ty) = ((from.x / ts).floor() as i64, (from.y / ts).floor() as i64);
    let (w, h) = (map.width as i64, map.height as i64);
    if tx < 0 || tx >= w {
        return None;
    }
    let rows = (reach / ts) as i64 + 1;
    (ty.max(0)..(ty + rows).min(h))
        .find(|&y| {
            let t = map.tiles[(y * w + tx) as usize];
            t.is_solid() || matches!(t, Tile::Platform | Tile::Quicksand)
        })
        .map(|y| y as f32 * ts)
}

/// Is `pos` inside the ground – or (falling weather) under a roof up to 8 tiles above?
#[allow(clippy::cast_possible_wrap)]
fn sheltered(map: &Map, pos: Vec2, kind: WeatherKind) -> bool {
    let ts = TILE_SIZE as f32;
    let (tx, ty) = ((pos.x / ts).floor() as i64, (pos.y / ts).floor() as i64);
    let (w, h) = (map.width as i64, map.height as i64);
    let blocks = |y: i64| {
        (0..h).contains(&y) && (0..w).contains(&tx) && {
            let t = map.tiles[(y * w + tx) as usize];
            t.is_solid() || t == Tile::Platform
        }
    };
    if blocks(ty) {
        return true;
    }
    let falls = matches!(
        kind,
        WeatherKind::Rain | WeatherKind::Storm | WeatherKind::Snow | WeatherKind::Blizzard
    );
    falls && (1..=8).any(|k| blocks(ty - k))
}

/// Base number of particles at full strength and setting "full".
fn base_count(kind: WeatherKind) -> f32 {
    match kind {
        WeatherKind::Clear => 0.0,
        WeatherKind::Rain => 420.0,
        WeatherKind::Storm => 560.0,
        WeatherKind::Fog => 12.0,
        WeatherKind::Leaves => 64.0,
        WeatherKind::Petals => 80.0,
        WeatherKind::Sandstorm => 380.0,
        WeatherKind::Snow => 260.0,
        WeatherKind::Blizzard => 520.0,
    }
}

/// Color mood per weather at full strength.
fn base_grade(kind: WeatherKind) -> Grade {
    let g = |tint: [f32; 3], tint_amount, darken, fog: [f32; 3], fog_density| Grade {
        tint,
        tint_amount,
        darken,
        fog,
        fog_density,
        flash: 0.0,
    };
    match kind {
        WeatherKind::Clear | WeatherKind::Leaves | WeatherKind::Petals => Grade::NONE,
        WeatherKind::Rain => g([0.72, 0.77, 0.86], 0.55, 0.14, [0.66, 0.7, 0.76], 0.2),
        WeatherKind::Storm => g([0.55, 0.6, 0.76], 0.65, 0.42, [0.42, 0.46, 0.54], 0.2),
        WeatherKind::Fog => g([0.95, 0.97, 1.0], 0.15, 0.0, [0.9, 0.92, 0.93], 0.72),
        WeatherKind::Sandstorm => g([1.0, 0.82, 0.55], 0.45, 0.05, [0.86, 0.72, 0.5], 0.5),
        WeatherKind::Snow => g([0.92, 0.95, 1.0], 0.2, 0.0, [0.95, 0.97, 1.0], 0.12),
        WeatherKind::Blizzard => g([0.88, 0.92, 1.0], 0.3, 0.05, [0.95, 0.97, 1.0], 0.55),
    }
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn mix_grade(a: Grade, b: Grade, t: f32) -> Grade {
    let m3 = |x: [f32; 3], y: [f32; 3]| std::array::from_fn(|i| mix(x[i], y[i], t));
    Grade {
        tint: m3(a.tint, b.tint),
        tint_amount: mix(a.tint_amount, b.tint_amount, t),
        darken: mix(a.darken, b.darken, t),
        fog: m3(a.fog, b.fog),
        fog_density: mix(a.fog_density, b.fog_density, t),
        flash: mix(a.flash, b.flash, t),
    }
}

impl WeatherView {
    fn rand(&mut self) -> f32 {
        // xorshift64*
        if self.rng == 0 {
            self.rng = 0x2545_f491_4f6c_dd1d;
        }
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        (self.rng.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 40) as f32 / (1u64 << 24) as f32
    }

    fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.rand()
    }

    /// Weather wind with gusts (−1..1; sandstorm and snowstorm always blow).
    fn gusty_wind(&self, w: Weather) -> f32 {
        let base = match w.kind {
            WeatherKind::Sandstorm | WeatherKind::Blizzard if w.wind.abs() < 0.2 => 0.7,
            _ => w.wind,
        };
        let gust = match w.kind {
            WeatherKind::Storm | WeatherKind::Blizzard | WeatherKind::Sandstorm => 0.25,
            WeatherKind::Leaves | WeatherKind::Petals => 0.2,
            _ => 0.08,
        };
        base + (self.time * 0.7).sin() * gust * base.abs().max(0.3)
    }

    /// Velocity of a new particle (units/s).
    fn velocity(&mut self, kind: WeatherKind, wind: f32) -> Vec2 {
        match kind {
            WeatherKind::Rain => Vec2::new(wind * 350.0, self.range(820.0, 980.0)),
            WeatherKind::Storm => Vec2::new(wind * 520.0, self.range(980.0, 1150.0)),
            WeatherKind::Snow => Vec2::new(wind * 120.0, self.range(55.0, 110.0)),
            WeatherKind::Blizzard => Vec2::new(wind * 700.0, self.range(180.0, 280.0)),
            WeatherKind::Sandstorm => {
                Vec2::new(wind * self.range(750.0, 950.0), self.range(-30.0, 60.0))
            }
            WeatherKind::Leaves => Vec2::new(wind * 240.0, self.range(50.0, 90.0)),
            WeatherKind::Petals => Vec2::new(wind * 200.0, self.range(35.0, 70.0)),
            WeatherKind::Fog => Vec2::new(wind * 30.0 + 12.0, 0.0),
            WeatherKind::Clear => Vec2::ZERO,
        }
    }

    fn spawn(&mut self, kind: WeatherKind, wind: f32, at: Vec2) -> Particle {
        let front = self.rand() < 0.35;
        let vel = self.velocity(kind, wind) * if front { 1.1 } else { 0.85 };
        let palette: &[u32] = match kind {
            WeatherKind::Leaves => &[0x6c_bf4a, 0xa8_c84a, 0xd9_a03a, 0xc9_6a3a],
            WeatherKind::Petals => &[0xef_7fb0, 0xf8_dd6e, 0xff_ffff, 0xa7_7be0],
            _ => &[0xff_ffff],
        };
        let pick = (self.rand() * palette.len() as f32) as usize % palette.len();
        let size = match kind {
            WeatherKind::Snow => self.range(2.0, 4.0),
            WeatherKind::Blizzard => self.range(1.6, 3.2),
            WeatherKind::Fog => self.range(160.0, 320.0),
            WeatherKind::Leaves | WeatherKind::Petals => self.range(5.0, 8.0),
            _ => self.range(0.8, 1.2),
        } * if front { 1.25 } else { 1.0 };
        Particle {
            kind,
            pos: at,
            vel,
            front,
            size,
            angle: self.range(0.0, std::f32::consts::TAU),
            spin: self.range(-4.0, 4.0),
            seed: self.range(0.0, 100.0),
            color: Color::hex(palette[pick]),
            hidden: false,
        }
    }

    /// One frame: weather `w` (with "off" everything disappears), view `camera`.
    #[allow(clippy::too_many_lines, clippy::many_single_char_names)]
    pub fn update(
        &mut self,
        dt: f32,
        w: Weather,
        quality: WeatherQuality,
        camera: &Camera,
        map: Option<&Map>,
        random_bolts: bool,
    ) {
        let dt = dt.min(0.1);
        self.time += dt;
        self.quality = quality;
        self.sky_top = camera.top_left().y - MARGIN;
        let covered = map.is_some_and(|m| sheltered(m, camera.center, WeatherKind::Rain));
        self.shelter += (f32::from(u8::from(covered)) - self.shelter) * (dt * 2.0).min(1.0);
        self.warnings.retain_mut(|w| {
            w.1 += dt;
            w.1 < WARN_TIME + 0.3
        });
        let (q, q_grade) = match quality {
            WeatherQuality::Full => (1.0, 1.0),
            WeatherQuality::Gentle => (0.35, 0.6),
            WeatherQuality::Off => (0.0, 0.0),
        };
        let strength = if w.is_clear() {
            0.0
        } else {
            0.25 + 0.75 * w.intensity.clamp(0.0, 1.0)
        };
        let wind = self.gusty_wind(w);
        self.wind += (if q > 0.0 { wind } else { 0.0 } - self.wind) * (dt * 1.5).min(1.0);
        let tl = camera.top_left() - Vec2::new(MARGIN, MARGIN);
        let br = camera.top_left() + camera.size + Vec2::new(MARGIN, MARGIN);
        let inside = |p: Vec2| p.x >= tl.x && p.x <= br.x && p.y >= tl.y && p.y <= br.y;

        // move; whatever leaves the view comes back in at the top or on the windward side
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let target = (base_count(w.kind) * strength * q) as usize;
        let mut alive = 0;
        let mut i = 0;
        while i < self.particles.len() {
            let p = &mut self.particles[i];
            let sway = match p.kind {
                WeatherKind::Snow | WeatherKind::Petals | WeatherKind::Leaves => {
                    (self.time * 1.8 + p.seed).sin() * 30.0
                }
                _ => 0.0,
            };
            p.pos += (p.vel + Vec2::new(sway, 0.0)) * dt;
            p.angle += p.spin * dt;
            p.hidden = map.is_some_and(|m| sheltered(m, p.pos, p.kind));
            let current = p.kind == w.kind;
            if current {
                alive += 1;
            }
            if inside(p.pos) {
                i += 1;
                continue;
            }
            if current && alive <= target {
                // continue around the view: whoever falls out at the bottom comes back at the top,
                // whoever stays behind on the left while running comes in on the right – so the density
                // stays the same no matter how the camera moves (playtest: leaves in bursts)
                let size = br - tl;
                p.pos = Vec2::new(
                    tl.x + (p.pos.x - tl.x).rem_euclid(size.x),
                    tl.y + (p.pos.y - tl.y).rem_euclid(size.y),
                );
                i += 1;
            } else {
                self.particles.swap_remove(i);
            }
        }
        // fill up: on entering, spread over the whole view
        let missing = target.saturating_sub(alive).min(SPAWN_PER_FRAME);
        for _ in 0..missing {
            let at = Vec2::new(self.range(tl.x, br.x), self.range(tl.y, br.y));
            let p = self.spawn(w.kind, wind, at);
            self.particles.push(p);
        }

        // splashes on surfaces
        self.splashes.retain_mut(|s| {
            s.1 += dt;
            s.1 < SPLASH_TIME
        });
        if let Some(map) = map
            && matches!(w.kind, WeatherKind::Rain | WeatherKind::Storm)
            && q > 0.0
        {
            self.splash_carry += dt * 70.0 * strength * q;
            while self.splash_carry >= 1.0 {
                self.splash_carry -= 1.0;
                let x = self.range(tl.x + MARGIN, br.x - MARGIN);
                if let Some(y) = surface_below(map, Vec2::new(x, tl.y + MARGIN), br.y - tl.y) {
                    self.splashes.push((Vec2::new(x, y), 0.0));
                }
            }
        }

        // lightning (thunderstorm); "gentle" without flashes
        self.bolts.retain_mut(|b| {
            b.age += dt;
            b.age < BOLT_TIME
        });
        self.flash *= (-dt * 7.0).exp();
        if w.kind == WeatherKind::Storm && q > 0.0 && random_bolts {
            self.next_bolt -= dt;
            if self.next_bolt <= 0.0 {
                self.next_bolt = self.range(3.0, 8.0) / strength.max(0.3);
                let x = self.range(tl.x + MARGIN, br.x - MARGIN);
                let ground = map
                    .and_then(|m| surface_below(m, Vec2::new(x, tl.y + MARGIN), br.y - tl.y))
                    .unwrap_or(br.y - MARGIN);
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let seed = (self.rand() * 1e6) as u32;
                self.strike(Vec2::new(x, ground), tl.y, seed, quality);
            }
        }

        // the color mood glides towards the target (about 2 s)
        let target = mix_grade(Grade::NONE, base_grade(w.kind), strength * q_grade);
        let current = self.grade.unwrap_or(target);
        let mut g = mix_grade(current, target, (dt * 0.8).min(1.0));
        g.flash = self.flash;
        self.grade = Some(g);
    }

    /// Lightning at `ground` (top edge of the ground) from height `top`.
    fn strike(&mut self, ground: Vec2, top: f32, seed: u32, quality: WeatherQuality) {
        self.bolts.push(Bolt {
            top: Vec2::new(ground.x + (seed % 120) as f32 - 60.0, top),
            ground,
            age: 0.0,
            seed,
        });
        if quality == WeatherQuality::Full {
            self.flash = 1.0;
        }
        let delay = 0.25 + (seed % 900) as f32 / 1000.0;
        // without a consumer (no sound yet) do not collect without limit
        if self.thunder.len() >= 8 {
            let _ = self.thunder.remove(0);
        }
        self.thunder.push((ground, delay));
    }

    /// Simulation (adventure): lightning is about to strike here – the ground glows.
    pub fn sim_warn(&mut self, pos: Vec2) {
        self.warnings.push((pos, 0.0));
    }

    /// Simulation (adventure): strike at `pos`.
    pub fn sim_strike(&mut self, pos: Vec2) {
        self.warnings.retain(|w| w.0.distance(pos) > 1.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let seed = (pos.x.abs() * 13.0 + self.time * 977.0) as u32;
        self.strike(pos, self.sky_top, seed, self.quality);
        // very close: the thunder cracks immediately
        if let Some(t) = self.thunder.last_mut() {
            t.1 = 0.0;
        }
    }

    /// Color mood for the post shader.
    pub fn grade(&self) -> Grade {
        self.grade.unwrap_or(Grade::NONE)
    }

    /// How much the camera is under a roof or in a cave (0..1, smoothed).
    pub fn shelter(&self) -> f32 {
        self.shelter
    }

    /// Wind with which the decoration currently sways.
    pub fn wind(&self) -> f32 {
        self.wind
    }

    /// Thunder since the last fetch (for the sounds, W1.5): place and delay (s).
    pub fn take_thunder(&mut self) -> Vec<(Vec2, f32)> {
        std::mem::take(&mut self.thunder)
    }

    /// Particles behind the figures and splashes.
    pub fn draw_back(&self, batch: &mut ShapeBatch) {
        // warning before the strike: the ground glows ever brighter, sparks rise (E-336)
        for (pos, age) in &self.warnings {
            let k = (age / WARN_TIME).min(1.0);
            let pulse = 0.75 + 0.25 * (age * 22.0).sin();
            let ring: Vec<Vec2> = (0..16)
                .map(|i| {
                    let a = i as f32 / 16.0 * std::f32::consts::TAU;
                    *pos + Vec2::new(a.cos() * 30.0, a.sin() * 7.0 - 1.0)
                })
                .collect();
            batch.fill_polygon(&ring, Color::rgba(1.0, 0.94, 0.6, 0.35 * k * pulse));
            batch.fill_circle(
                *pos,
                8.0 + 6.0 * k,
                Color::rgba(1.0, 0.97, 0.78, 0.5 * k * pulse),
            );
            for j in 0..5 {
                let f = j as f32;
                let rise = ((age * 1.6 + f * 0.21) % 1.0) * 34.0;
                let p = *pos + Vec2::new((f * 7.3).sin() * 18.0, -rise);
                batch.fill_circle(p, 1.8, Color::rgba(1.0, 0.97, 0.8, k));
            }
        }
        for (pos, age) in &self.splashes {
            let k = 1.0 - age / SPLASH_TIME;
            let c = Color::rgba(0.88, 0.93, 1.0, 0.7 * k);
            let h = 5.0 + 4.0 * (1.0 - k);
            batch.stroke_line(*pos, *pos + Vec2::new(-4.0, -h), 1.4, c);
            batch.stroke_line(*pos, *pos + Vec2::new(4.0, -h), 1.4, c);
        }
        for p in self.particles.iter().filter(|p| !p.front && !p.hidden) {
            draw_particle(batch, p, 0.65);
        }
    }

    /// Particles in front of the figures and lightning.
    pub fn draw_front(&self, batch: &mut ShapeBatch) {
        for p in self.particles.iter().filter(|p| p.front && !p.hidden) {
            draw_particle(batch, p, 1.0);
        }
        for b in &self.bolts {
            let k = 1.0 - b.age / BOLT_TIME;
            let points = bolt_points(b);
            batch.stroke_polyline(&points, 9.0, Color::rgba(1.0, 0.97, 0.78, 0.35 * k));
            batch.stroke_polyline(&points, 3.0, Color::rgba(1.0, 1.0, 1.0, k));
            batch.fill_circle(
                b.ground,
                14.0 * k + 4.0,
                Color::rgba(1.0, 0.96, 0.75, 0.6 * k),
            );
        }
    }
}

/// Zigzag of a lightning bolt from the top down to the ground.
fn bolt_points(b: &Bolt) -> Vec<Vec2> {
    let mut pts = vec![b.top];
    let steps = 9;
    let mut h = b.seed;
    for k in 1..steps {
        h = h.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        let t = k as f32 / steps as f32;
        let jitter = ((h >> 16) % 50) as f32 - 25.0;
        let p = b.top.lerp(b.ground, t) + Vec2::new(jitter, 0.0);
        pts.push(p);
    }
    pts.push(b.ground);
    pts
}

fn draw_particle(batch: &mut ShapeBatch, p: &Particle, alpha: f32) {
    let w = if p.front { 1.0 } else { 0.7 };
    match p.kind {
        WeatherKind::Rain | WeatherKind::Storm => {
            let tail = p.vel * 0.022;
            batch.stroke_line(
                p.pos,
                p.pos - tail,
                2.1 * w,
                Color::rgba(0.88, 0.92, 0.98, 0.78 * alpha),
            );
        }
        WeatherKind::Snow => {
            batch.fill_circle(p.pos, p.size, Color::rgba(1.0, 1.0, 1.0, 0.9 * alpha));
        }
        WeatherKind::Blizzard => {
            let tail = p.vel * 0.03;
            batch.stroke_line(
                p.pos,
                p.pos - tail,
                p.size,
                Color::rgba(1.0, 1.0, 1.0, 0.85 * alpha),
            );
        }
        WeatherKind::Sandstorm => {
            let tail = p.vel * 0.045;
            batch.stroke_line(
                p.pos,
                p.pos - tail,
                2.0 * w,
                Color::rgba(0.97, 0.88, 0.64, 0.75 * alpha),
            );
        }
        WeatherKind::Leaves | WeatherKind::Petals => {
            let (s, c) = (p.angle.sin(), p.angle.cos());
            let along = Vec2::new(c, s) * p.size;
            let across = Vec2::new(-s, c)
                * p.size
                * if p.kind == WeatherKind::Leaves {
                    0.45
                } else {
                    0.6
                };
            let mut col = p.color;
            col.0[3] = alpha;
            let pts = [p.pos - along, p.pos - across, p.pos + along, p.pos + across];
            batch.fill_polygon(&pts, Color::rgba(0.17, 0.17, 0.17, 0.8 * alpha));
            let inner: Vec<Vec2> = pts.iter().map(|q| p.pos + (*q - p.pos) * 0.78).collect();
            batch.fill_polygon(&inner, col);
        }
        WeatherKind::Fog => {
            // soft swath: rings denser towards the inside, without a hard edge
            for k in 1..=5 {
                let r = p.size * (1.0 - k as f32 * 0.17);
                batch.fill_circle(p.pos, r, Color::rgba(0.95, 0.96, 0.97, 0.025 * alpha));
            }
        }
        WeatherKind::Clear => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera() -> Camera {
        Camera {
            center: Vec2::new(500.0, 300.0),
            size: Vec2::new(1000.0, 600.0),
        }
    }

    fn rain(intensity: f32) -> Weather {
        Weather {
            kind: WeatherKind::Rain,
            intensity,
            wind: 0.3,
        }
    }

    #[test]
    fn particles_follow_kind_intensity_and_quality() {
        let mut v = WeatherView::default();
        v.update(
            0.016,
            rain(1.0),
            WeatherQuality::Full,
            &camera(),
            None,
            true,
        );
        let full = v.particles.len();
        assert!(full > 300, "{full}");
        let mut v = WeatherView::default();
        v.update(
            0.016,
            rain(1.0),
            WeatherQuality::Gentle,
            &camera(),
            None,
            true,
        );
        assert!(v.particles.len() < full / 2, "gentle: fewer");
        let mut v = WeatherView::default();
        v.update(0.016, rain(1.0), WeatherQuality::Off, &camera(), None, true);
        assert!(
            v.particles.is_empty() && v.grade().fog_density == 0.0,
            "off"
        );
        // weather change: old particles disappear gradually, new ones arrive
        let mut v = WeatherView::default();
        v.update(
            0.016,
            rain(1.0),
            WeatherQuality::Full,
            &camera(),
            None,
            true,
        );
        // about 8 s
        for _ in 0..500 {
            v.update(
                0.016,
                Weather::CLEAR,
                WeatherQuality::Full,
                &camera(),
                None,
                true,
            );
        }
        assert!(v.particles.is_empty(), "clear after the rain");
        assert!(v.grade().tint_amount < 0.01, "mood fades");
    }

    #[test]
    fn storm_strikes_with_flash_and_thunder() {
        let mut v = WeatherView::default();
        let storm = Weather {
            kind: WeatherKind::Storm,
            intensity: 1.0,
            wind: 0.0,
        };
        let mut flashes = 0;
        for _ in 0..1500 {
            v.update(0.016, storm, WeatherQuality::Full, &camera(), None, true);
            if v.grade().flash > 0.9 {
                flashes += 1;
            }
        }
        assert!(flashes > 0, "lightning strikes");
        assert!(!v.take_thunder().is_empty(), "and thunders");
        assert!(v.take_thunder().is_empty(), "taken only once");
    }

    #[test]
    fn rain_stays_out_of_caves_and_houses() {
        // ceiling in row 2: below it (row 3) it does not rain, next to it it does
        let map = Map::from_rows(
            "s",
            &[
                "S.........",
                "..........",
                "#####.....",
                "..........",
                "##########",
            ],
        )
        .unwrap();
        let under = Vec2::new(48.0, 3.5 * 32.0);
        let open = Vec2::new(8.0 * 32.0, 3.5 * 32.0);
        assert!(sheltered(&map, under, WeatherKind::Rain));
        assert!(!sheltered(&map, open, WeatherKind::Rain));
        assert!(
            !sheltered(&map, under, WeatherKind::Leaves),
            "leaves blow in"
        );
        assert!(
            sheltered(&map, Vec2::new(48.0, 2.5 * 32.0), WeatherKind::Leaves),
            "never inside rock"
        );
    }

    #[test]
    fn splashes_land_on_the_surface() {
        let map = Map::from_rows("s", &["S.........", "..........", "##########"]).unwrap();
        let cam = Camera {
            center: Vec2::new(160.0, 48.0),
            size: Vec2::new(320.0, 96.0),
        };
        let mut v = WeatherView::default();
        for _ in 0..60 {
            v.update(
                0.016,
                rain(1.0),
                WeatherQuality::Full,
                &cam,
                Some(&map),
                true,
            );
        }
        assert!(!v.splashes.is_empty());
        assert!(v.splashes.iter().all(|(p, _)| (p.y - 64.0).abs() < 0.1));
    }

    /// Playtest: leaves came in bursts depending on how the camera moved. While running
    /// and jumping, their number in the picture now stays about the same.
    #[test]
    fn leaves_stay_even_while_the_camera_moves() {
        let mut v = WeatherView::default();
        let leaves = Weather {
            kind: WeatherKind::Leaves,
            intensity: 1.0,
            wind: 0.4,
        };
        let mut cam = camera();
        for _ in 0..300 {
            v.update(0.016, leaves, WeatherQuality::Full, &cam, None, true);
        }
        let visible = |v: &WeatherView, cam: &Camera| {
            let (tl, br) = (cam.top_left(), cam.top_left() + cam.size);
            v.particles
                .iter()
                .filter(|p| {
                    p.pos.x >= tl.x && p.pos.x <= br.x && p.pos.y >= tl.y && p.pos.y <= br.y
                })
                .count()
        };
        let base = visible(&v, &cam);
        assert!(base > 20, "leaves on screen: {base}");
        // run, jump, fall
        for k in 0..240 {
            let t = k as f32 * 0.05;
            cam.center += Vec2::new(9.0, (t * 2.0).sin() * 14.0);
            v.update(0.016, leaves, WeatherQuality::Full, &cam, None, true);
            let n = visible(&v, &cam);
            assert!(
                n * 10 >= base * 7 && n * 10 <= base * 13,
                "frame {k}: {n} instead of about {base}"
            );
        }
    }
}

//! Wetter in der Darstellung (R2-W1, W1.2, E-329 bis E-336): Partikel hinter und vor den
//! Figuren, Spritzer auf Oberflächen, Blitze und die Farbstimmung für den Post-Shader.
//!
//! Reine Darstellung: Partikel leben in Weltkoordinaten um den Kameraausschnitt und werden
//! dort wiederverwendet; nichts davon wirkt auf die Simulation.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use elora_map::{Map, Weather, WeatherKind};
use elora_render::{Camera, Color, Grade, ShapeBatch};
use elora_sim::{TILE_SIZE, Tile, Vec2};

use crate::settings::WeatherQuality;

/// Rand um den Ausschnitt, in dem Partikel weiterleben (Einheiten).
const MARGIN: f32 = 120.0;
/// Höchstens so viele neue Partikel je Frame (beim Betreten füllt sich das Bild sofort).
const SPAWN_PER_FRAME: usize = 600;
/// So lange ist ein Blitz zu sehen (s) und ein Spritzer (s).
const BOLT_TIME: f32 = 0.22;
const SPLASH_TIME: f32 = 0.25;
/// So lange glimmt der Boden vor einem Einschlag (s, wie A-32).
const WARN_TIME: f32 = 0.9;

#[derive(Debug, Clone, Copy)]
struct Particle {
    kind: WeatherKind,
    pos: Vec2,
    vel: Vec2,
    /// vor den Figuren (größer, kräftiger) oder dahinter
    front: bool,
    size: f32,
    /// Drehung (Blätter) und Schaukeln (Flocken)
    angle: f32,
    spin: f32,
    seed: f32,
    color: Color,
    /// im Boden oder unter einem Dach: nicht zeichnen (Höhlen, Häuser, Felsdächer)
    hidden: bool,
}

#[derive(Debug, Clone, Copy)]
struct Bolt {
    top: Vec2,
    ground: Vec2,
    age: f32,
    seed: u32,
}

/// Wetter der Darstellung.
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
    /// Donner für den Klang (W1.5): Ort und Verzögerung (s).
    thunder: Vec<(Vec2, f32)>,
    /// Wind, mit dem die Deko gerade wiegt (geglättet).
    wind: f32,
    /// Angekündigte Einschläge aus der Simulation (Abenteuer): Ort, Alter (s).
    warnings: Vec<(Vec2, f32)>,
    /// Oberkante des Ausschnitts (für Blitze aus der Simulation).
    sky_top: f32,
    quality: WeatherQuality,
}

/// Oberkante des ersten Bodens (fest, Plattform, Treibsand) unter `from`, höchstens `reach` tief.
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

/// Steckt `pos` im Boden – oder (fallendes Wetter) unter einem Dach bis 8 Tiles darüber?
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

/// Grundmenge der Partikel bei voller Stärke und Einstellung „voll“.
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

/// Farbstimmung je Wetter bei voller Stärke.
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

    /// Wind des Wetters mit Böen (−1..1; Sandsturm und Schneesturm wehen immer).
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

    /// Geschwindigkeit eines neuen Partikels (Einheiten/s).
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

    /// Ein Frame: Wetter `w` (bei „aus“ verschwindet alles), Ausschnitt `camera`.
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

        // bewegen; was den Ausschnitt verlässt, kommt oben bzw. auf der Windseite wieder herein
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
                // wiederverwenden: oben (fallend) oder auf der Windseite (waagerecht)
                let kind = p.kind;
                let x = self.range(tl.x, br.x);
                let y = self.range(tl.y, br.y);
                let side = if wind >= 0.0 { tl.x } else { br.x };
                let at = match kind {
                    WeatherKind::Sandstorm | WeatherKind::Fog => Vec2::new(side, y),
                    _ => Vec2::new(x, tl.y),
                };
                self.particles[i] = self.spawn(kind, wind, at);
                i += 1;
            } else {
                self.particles.swap_remove(i);
            }
        }
        // auffüllen: beim Betreten über den ganzen Ausschnitt verteilt
        let missing = target.saturating_sub(alive).min(SPAWN_PER_FRAME);
        for _ in 0..missing {
            let at = Vec2::new(self.range(tl.x, br.x), self.range(tl.y, br.y));
            let p = self.spawn(w.kind, wind, at);
            self.particles.push(p);
        }

        // Spritzer auf Oberflächen
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

        // Blitze (Gewitter); „sanft“ ohne Aufblitzen
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

        // Farbstimmung gleitet zum Ziel (etwa 2 s)
        let target = mix_grade(Grade::NONE, base_grade(w.kind), strength * q_grade);
        let current = self.grade.unwrap_or(target);
        let mut g = mix_grade(current, target, (dt * 0.8).min(1.0));
        g.flash = self.flash;
        self.grade = Some(g);
    }

    /// Blitz bei `ground` (Oberkante des Bodens) aus Höhe `top`.
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
        // ohne Abholer (noch kein Klang) nicht unbegrenzt sammeln
        if self.thunder.len() >= 8 {
            let _ = self.thunder.remove(0);
        }
        self.thunder.push((ground, delay));
    }

    /// Simulation (Abenteuer): hier schlägt gleich ein Blitz ein – der Boden glimmt.
    pub fn sim_warn(&mut self, pos: Vec2) {
        self.warnings.push((pos, 0.0));
    }

    /// Simulation (Abenteuer): Einschlag bei `pos`.
    pub fn sim_strike(&mut self, pos: Vec2) {
        self.warnings.retain(|w| w.0.distance(pos) > 1.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let seed = (pos.x.abs() * 13.0 + self.time * 977.0) as u32;
        self.strike(pos, self.sky_top, seed, self.quality);
        // ganz nah: der Donner kracht sofort
        if let Some(t) = self.thunder.last_mut() {
            t.1 = 0.0;
        }
    }

    /// Farbstimmung für den Post-Shader.
    pub fn grade(&self) -> Grade {
        self.grade.unwrap_or(Grade::NONE)
    }

    /// Wind, mit dem die Deko gerade wiegt.
    pub fn wind(&self) -> f32 {
        self.wind
    }

    /// Donner seit dem letzten Abholen (für die Klänge, W1.5): Ort und Verzögerung (s).
    pub fn take_thunder(&mut self) -> Vec<(Vec2, f32)> {
        std::mem::take(&mut self.thunder)
    }

    /// Partikel hinter den Figuren und Spritzer.
    pub fn draw_back(&self, batch: &mut ShapeBatch) {
        // Warnung vor dem Einschlag: der Boden glimmt immer heller, Funken steigen (E-336)
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

    /// Partikel vor den Figuren und Blitze.
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

/// Zickzack eines Blitzes von oben bis zum Boden.
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
            // weiche Schwade: Ringe nach innen dichter, ohne harte Kante
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
        assert!(v.particles.len() < full / 2, "sanft: weniger");
        let mut v = WeatherView::default();
        v.update(0.016, rain(1.0), WeatherQuality::Off, &camera(), None, true);
        assert!(
            v.particles.is_empty() && v.grade().fog_density == 0.0,
            "aus"
        );
        // Wetterwechsel: alte Partikel verschwinden nach und nach, neue kommen
        let mut v = WeatherView::default();
        v.update(
            0.016,
            rain(1.0),
            WeatherQuality::Full,
            &camera(),
            None,
            true,
        );
        // etwa 8 s
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
        assert!(v.particles.is_empty(), "nach dem Regen klar");
        assert!(v.grade().tint_amount < 0.01, "Stimmung klingt ab");
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
        assert!(flashes > 0, "es blitzt");
        assert!(!v.take_thunder().is_empty(), "und donnert");
        assert!(v.take_thunder().is_empty(), "nur einmal abgeholt");
    }

    #[test]
    fn rain_stays_out_of_caves_and_houses() {
        // Decke in Zeile 2: darunter (Zeile 3) regnet es nicht, daneben schon
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
            "Blätter wehen hinein"
        );
        assert!(
            sheltered(&map, Vec2::new(48.0, 2.5 * 32.0), WeatherKind::Leaves),
            "im Fels nie"
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
}

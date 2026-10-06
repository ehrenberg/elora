//! Effekte (M5.6): Partikel aus Simulations-Ereignissen, Kamerawackeln und
//! Treffer-Marker (E-088, beide abschaltbar). Reine Darstellung.
//!
//! | Auslöser | Effekt |
//! |---|---|
//! | Explosion | Blitz, Rauchwolken, Funken, Wackeln in der Nähe |
//! | Hammer-Treffer | Funkenstern |
//! | Laser-Abprall | cyanfarbene Funken |
//! | Schaden | Tropfen in Körperfarbe, eigenes Wackeln |
//! | Tod | Spritzer in Körperfarbe mit Schwerkraft |
//! | Spawn, Pickup | Glitzern |
//! | Sprung / Luftsprung / Landung | Staub bzw. Wolkenring an den Füßen |
//! | Granate im Flug | Rauchspur |

use std::collections::HashMap;

use elora_client::scene::Scene;
use elora_render::{Affine, Color, Mesh, MeshBuilder, Paint, ShapeBatch, Tint, ellipse};
use elora_sim::character::events;
use elora_sim::{Event, PHYS_SIZE, Vec2};

use crate::figure::Landing;

/// Schalter (E-088), gespeichert in `tuning.toml` unter `[effects]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct EffectSettings {
    pub camera_shake: bool,
    pub hit_marker: bool,
}

impl Default for EffectSettings {
    fn default() -> Self {
        Self {
            camera_shake: true,
            hit_marker: true,
        }
    }
}

/// Größte Verschiebung der Kamera bei vollem Wackeln (Einheiten).
const SHAKE_MAX: f32 = 9.0;
/// Abbau des Wackelns je Sekunde.
const SHAKE_DECAY: f32 = 1.8;
/// Explosionen weiter weg als das wackeln nicht.
const SHAKE_RANGE: f32 = 500.0;
const HIT_MARKER_TIME: f32 = 0.18;
/// Rauchspur: Wolken je Sekunde und Granate.
const TRAIL_RATE: f32 = 40.0;

/// Vorlage für einen Partikel-Ausstoß: Anzahl, Geschwindigkeit, Lebensdauer (s) und
/// Radius (Anfang, Ende) sowie Schwerkraft und Luftwiderstand wie bei [`Particle`].
#[derive(Debug, Clone, Copy)]
struct Burst {
    count: u32,
    speed: (f32, f32),
    life: (f32, f32),
    size: (f32, f32),
    gravity: f32,
    drag: f32,
}

const GROUND_JUMP: Burst = Burst {
    count: 5,
    speed: (20.0, 60.0),
    life: (0.25, 0.4),
    size: (3.5, 1.0),
    gravity: -20.0,
    drag: 3.0,
};

/// Füllt etwa den Explosionsradius (135): Weg ≈ Geschwindigkeit / Widerstand.
const EXPLOSION_SMOKE: Burst = Burst {
    count: 18,
    speed: (150.0, 420.0),
    life: (0.45, 0.85),
    size: (12.0, 24.0),
    gravity: -30.0,
    drag: 4.0,
};

const EXPLOSION_SPARKS: Burst = Burst {
    count: 10,
    speed: (150.0, 320.0),
    life: (0.2, 0.4),
    size: (2.5, 0.5),
    gravity: 300.0,
    drag: 2.0,
};

const HAMMER_SPARKS: Burst = Burst {
    count: 10,
    speed: (160.0, 300.0),
    life: (0.18, 0.3),
    size: (3.5, 1.0),
    gravity: 0.0,
    drag: 5.0,
};

const LASER_SPARKS: Burst = Burst {
    count: 6,
    speed: (60.0, 160.0),
    life: (0.15, 0.3),
    size: (2.0, 0.5),
    gravity: 0.0,
    drag: 4.0,
};

const DAMAGE_DROPS: Burst = Burst {
    count: 5,
    speed: (60.0, 140.0),
    life: (0.3, 0.5),
    size: (3.0, 1.5),
    gravity: 500.0,
    drag: 1.0,
};

const DEATH_SPLASH: Burst = Burst {
    count: 18,
    speed: (80.0, 260.0),
    life: (0.5, 0.9),
    size: (5.0, 2.0),
    gravity: 600.0,
    drag: 1.0,
};

const DEATH_SMOKE: Burst = Burst {
    count: 6,
    speed: (20.0, 60.0),
    life: (0.4, 0.6),
    size: (8.0, 12.0),
    gravity: -20.0,
    drag: 3.0,
};

const SPAWN_GLITTER: Burst = Burst {
    count: 12,
    speed: (40.0, 110.0),
    life: (0.3, 0.6),
    size: (3.5, 1.0),
    gravity: -60.0,
    drag: 2.0,
};

const PICKUP_GLITTER: Burst = Burst {
    count: 8,
    speed: (30.0, 90.0),
    life: (0.25, 0.5),
    size: (2.0, 0.5),
    gravity: -40.0,
    drag: 2.0,
};

#[derive(Debug, Clone, Copy)]
struct Particle {
    pos: Vec2,
    vel: Vec2,
    age: f32,
    life: f32,
    /// Radius am Anfang und am Ende.
    size: (f32, f32),
    color: Color,
    /// Beschleunigung nach unten (Einheiten/s²), negativ = steigt.
    gravity: f32,
    /// Anteil der Geschwindigkeit, der pro Sekunde verloren geht.
    drag: f32,
    /// Als kleines Sternchen zeichnen (Leuchtpilze) statt als Kreis.
    star: bool,
}

/// Kurzer, heller Kreis (Explosionsblitz).
#[derive(Debug, Clone, Copy)]
struct Flash {
    pos: Vec2,
    age: f32,
    life: f32,
    radius: f32,
}

#[derive(Debug)]
pub struct Effects {
    pub settings: EffectSettings,
    particles: Vec<Particle>,
    flashes: Vec<Flash>,
    /// Wurzelstöße des Wurzelwächters: Fuß am Boden, Alter (s).
    roots: Vec<(Vec2, f32)>,
    /// Dünnes Eis mit Rissen (R2-M2.4): Ecke des Tiles, Alter (s).
    cracks: Vec<(Vec2, f32)>,
    rng: u64,
    trail: f32,
    shake: f32,
    time: f32,
    hit_marker: f32,
    /// Zuletzt gesehene Körperfarbe je Slot (für Tod und Schaden).
    colors: HashMap<usize, Color>,
    circle: Mesh,
}

impl Default for Effects {
    fn default() -> Self {
        Self {
            settings: EffectSettings::default(),
            particles: Vec::new(),
            flashes: Vec::new(),
            roots: Vec::new(),
            cracks: Vec::new(),
            rng: 0x9e37_79b9_7f4a_7c15,
            trail: 0.0,
            shake: 0.0,
            time: 0.0,
            hit_marker: 0.0,
            colors: HashMap::new(),
            circle: MeshBuilder::new(0.02)
                .fill_path(
                    &ellipse(Vec2::default(), Vec2::new(1.0, 1.0)),
                    Paint::key(1),
                )
                .build(),
        }
    }
}

const STOMP_DUST: Burst = Burst {
    count: 16,
    speed: (90.0, 260.0),
    life: (0.3, 0.55),
    size: (6.0, 2.0),
    gravity: 0.0,
    drag: 4.0,
};

/// Brocken eines zerbrochenen Bröckelbodens.
const CRUMBS: Burst = Burst {
    count: 10,
    speed: (60.0, 200.0),
    life: (0.4, 0.8),
    size: (4.0, 2.0),
    gravity: 900.0,
    drag: 1.0,
};

const SMOKE: Color = Color::rgba(0.92, 0.92, 0.9, 0.75);
const DUST: Color = Color::rgba(0.85, 0.82, 0.76, 0.7);
const SPARK: Color = Color::rgb(1.0, 0.8, 0.35);
const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
const LASER_SPARK: Color = Color::rgb(0.6, 0.95, 1.0);
const GLITTER: Color = Color::rgb(1.0, 0.95, 0.6);
const FALLBACK_BODY: Color = Color::hex(0xf2c14e);
const CRUMB: Color = Color::hex(0xb08a5e);
/// Splitter von brechendem dünnem Eis.
const ICE_SHARD: Color = Color::hex(0xcfeefa);
/// Risse bleiben höchstens so lange (s); das Eis bricht vorher (A-36).
const CRACK_LIFE: f32 = 2.0;
/// Sternchen der Leuchtpilze.
const SPORE: Color = Color::rgba(0.55, 0.85, 1.0, 0.95);
/// So lange steht ein Wurzelstoß (s).
const ROOT_LIFE: f32 = 0.7;

impl Effects {
    pub fn with_settings(settings: EffectSettings) -> Self {
        Self {
            settings,
            ..Self::default()
        }
    }

    /// Zufallszahl in 0..1 (xorshift; Darstellung muss nicht reproduzierbar sein).
    fn rand(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        #[allow(clippy::cast_precision_loss)]
        let v = (self.rng >> 40) as f32 / (1u64 << 24) as f32;
        v
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.rand()
    }

    fn dir(&mut self) -> Vec2 {
        let a = self.range(0.0, std::f32::consts::TAU);
        Vec2::new(a.cos(), a.sin())
    }

    /// Partikel nach Vorlage `b` um `pos`.
    fn burst(&mut self, pos: Vec2, color: Color, b: &Burst) {
        for _ in 0..b.count {
            let vel = self.dir() * self.range(b.speed.0, b.speed.1);
            let life = self.range(b.life.0, b.life.1);
            self.particles.push(Particle {
                pos,
                vel,
                age: 0.0,
                life,
                size: b.size,
                color,
                gravity: b.gravity,
                drag: b.drag,
                star: false,
            });
        }
    }

    /// Leuchtpilze in der Nähe: kleine blaue Sternchen steigen auf (R2-M2.2).
    pub fn glow_spores(&mut self, dt: f32, sources: &[Vec2]) {
        let dt = dt.min(0.05);
        for &p in sources {
            // etwa 3 Sternchen je Sekunde und Pilzgruppe
            if self.range(0.0, 1.0) > dt * 3.0 {
                continue;
            }
            let pos = p + Vec2::new(self.range(-26.0, 26.0), -self.range(10.0, 40.0));
            let vel = Vec2::new(self.range(-8.0, 8.0), -self.range(14.0, 30.0));
            let life = self.range(1.2, 2.2);
            self.particles.push(Particle {
                pos,
                vel,
                age: 0.0,
                life,
                size: (3.2, 1.6),
                color: SPORE,
                gravity: -4.0,
                drag: 0.3,
                star: true,
            });
        }
    }

    /// Kamera-Wackeln anstoßen (0..1, addiert, begrenzt).
    fn add_shake(&mut self, amount: f32) {
        if self.settings.camera_shake {
            self.shake = (self.shake + amount).min(1.0);
        }
    }

    /// `body_color`: aktuelle Körperfarbe einer Figur der Szene.
    pub fn update(
        &mut self,
        dt: f32,
        events_in: &[Event],
        scene: &Scene,
        landings: &[Landing],
        body_color: impl Fn(&elora_client::scene::SceneChar) -> Color,
    ) {
        let dt = dt.min(0.05);
        self.time += dt;
        self.step(dt);

        for c in &scene.chars {
            self.colors.insert(c.slot, body_color(c));
        }
        let local = scene.local().map(|c| (c.slot, c.pos()));
        let feet = |p: Vec2| p + Vec2::new(0.0, PHYS_SIZE / 2.0);

        for e in events_in {
            self.on_event(e, scene, local);
        }
        // Sprünge aus den Bits des Figurenkerns
        for c in &scene.chars {
            let bits = c.ch.core.triggered_events;
            if bits & events::GROUND_JUMP != 0 {
                self.burst(feet(c.pos()), DUST, &GROUND_JUMP);
            }
            if bits & (events::WALL_JUMP | events::HOOK_RUCK) != 0 {
                self.burst(c.pos(), SMOKE, &GROUND_JUMP);
            }
            if bits & events::AIR_JUMP != 0 {
                for k in 0..8 {
                    #[allow(clippy::cast_precision_loss)]
                    let a = k as f32 / 8.0 * std::f32::consts::TAU;
                    self.particles.push(Particle {
                        pos: feet(c.pos()),
                        vel: Vec2::new(a.cos() * 70.0, a.sin() * 25.0 + 30.0),
                        age: 0.0,
                        life: 0.35,
                        size: (4.0, 1.5),
                        color: SMOKE,
                        gravity: 0.0,
                        drag: 4.0,
                        star: false,
                    });
                }
            }
        }
        for l in landings {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)] // 0..=8
            let n = (l.strength * 10.0).clamp(0.0, 8.0) as u32;
            if n > 1 {
                for side in [-1.0, 1.0] {
                    for _ in 0..n / 2 {
                        let vel = Vec2::new(side * self.range(30.0, 90.0), -self.range(5.0, 25.0));
                        let life = self.range(0.25, 0.45);
                        self.particles.push(Particle {
                            pos: l.pos,
                            vel,
                            age: 0.0,
                            life,
                            size: (3.5, 1.0),
                            color: DUST,
                            gravity: 0.0,
                            drag: 3.0,
                            star: false,
                        });
                    }
                }
            }
        }
        // Rauchspur hinter Granaten
        self.trail += dt * TRAIL_RATE;
        while self.trail >= 1.0 {
            self.trail -= 1.0;
            for i in 0..scene.projectiles.len() {
                let p = scene.projectiles[i] + self.dir() * 2.0;
                let life = self.range(0.3, 0.5);
                self.particles.push(Particle {
                    pos: p,
                    vel: Vec2::new(0.0, -8.0),
                    age: 0.0,
                    life,
                    size: (4.0, 7.0),
                    color: Color::rgba(0.9, 0.9, 0.88, 0.45),
                    gravity: 0.0,
                    drag: 1.0,
                    star: false,
                });
            }
        }
    }

    /// Dünnes Eis bekommt Risse (`broken = false`) oder bricht in Splitter.
    fn ice_crack(&mut self, tx: i32, ty: i32, broken: bool) {
        #[allow(clippy::cast_precision_loss)]
        let ts = elora_sim::TILE_SIZE as f32;
        #[allow(clippy::cast_precision_loss)]
        let min = Vec2::new(tx as f32 * ts, ty as f32 * ts);
        if broken {
            self.cracks.retain(|c| c.0 != min);
            self.burst(min + Vec2::new(ts / 2.0, ts / 4.0), ICE_SHARD, &CRUMBS);
        } else if !self.cracks.iter().any(|c| c.0 == min) {
            self.cracks.push((min, 0.0));
        }
    }

    fn on_event(&mut self, e: &Event, scene: &Scene, local: Option<(usize, Vec2)>) {
        let is_local = |slot: usize| local.is_some_and(|(s, _)| s == slot);
        match *e {
            Event::Explosion { pos, .. } => {
                self.flashes.push(Flash {
                    pos,
                    age: 0.0,
                    life: 0.12,
                    radius: 80.0,
                });
                self.burst(pos, SMOKE, &EXPLOSION_SMOKE);
                self.burst(pos, SPARK, &EXPLOSION_SPARKS);
                if let Some((_, lp)) = local {
                    let d = (pos - lp).length();
                    if d < SHAKE_RANGE {
                        self.add_shake(0.6 * (1.0 - d / SHAKE_RANGE));
                    }
                }
            }
            Event::HammerHit { owner, pos } => {
                self.burst(pos, WHITE, &HAMMER_SPARKS);
                if is_local(owner) {
                    self.hit();
                }
            }
            Event::LaserBounce { pos, .. } => {
                self.burst(pos, LASER_SPARK, &LASER_SPARKS);
            }
            Event::Damage { player, from, .. } => {
                let color = self.body(player);
                if let Some(c) = scene.chars.iter().find(|c| c.slot == player) {
                    self.burst(c.pos(), color, &DAMAGE_DROPS);
                }
                if is_local(player) {
                    self.add_shake(0.25);
                }
                if from.is_some_and(|f| is_local(f) && f != player) {
                    self.hit();
                }
            }
            Event::Death { player, pos, .. } => {
                let color = self.body(player);
                self.burst(pos, color, &DEATH_SPLASH);
                self.burst(pos, SMOKE, &DEATH_SMOKE);
            }
            Event::Spawn { pos, .. } => {
                self.burst(pos, WHITE, &SPAWN_GLITTER);
            }
            Event::Pickup { pos, .. } | Event::LootCollect { pos, .. } => {
                self.burst(pos, GLITTER, &PICKUP_GLITTER);
            }
            Event::Stomp { player, pos } => {
                self.burst(pos, DUST, &STOMP_DUST);
                if is_local(player) {
                    self.add_shake(0.35);
                }
            }
            // abgeprallt (Hüter in der Luft): nur Sternchen, keine Treffer-Rückmeldung
            Event::CreatureHit { pos, damage: 0, .. } => self.burst(pos, GLITTER, &HAMMER_SPARKS),
            Event::CreatureHit { pos, from, .. } => {
                self.burst(pos, WHITE, &HAMMER_SPARKS);
                if from.is_some_and(is_local) {
                    self.hit();
                }
            }
            Event::CreatureDeath { pos, .. } => {
                self.burst(pos, SMOKE, &DEATH_SMOKE);
                self.burst(pos, GLITTER, &SPAWN_GLITTER);
            }
            // Wurzelwächter (R2-M2.2): Boden bebt, dann Wurzelstoß; gelöster Kern glitzert
            Event::CreatureAct { pos, act, .. } => match act {
                elora_sim::CreatureAct::Warn => {
                    self.burst(pos, CRUMB, &CRUMBS);
                    self.shake = (self.shake + 0.15).min(1.0);
                }
                elora_sim::CreatureAct::Strike => {
                    self.roots.push((pos, 0.0));
                    self.burst(pos, CRUMB, &CRUMBS);
                    self.shake = (self.shake + 0.3).min(1.0);
                }
                elora_sim::CreatureAct::Core => self.burst(pos, GLITTER, &SPAWN_GLITTER),
                // Wurzelschlange und Dünenwurm: Erde/Sand spritzt beim Auf- und Abtauchen
                elora_sim::CreatureAct::Emerge | elora_sim::CreatureAct::Burrow => {
                    self.burst(pos, CRUMB, &CRUMBS);
                }
                _ => {}
            },
            // dünnes Eis (R2-M2.4): Risse, dann Splitter
            Event::IceCrack { tx, ty, broken } => self.ice_crack(tx, ty, broken),
            // zerbrochener Boden und Wurzelwände bröseln
            Event::TileBroken { tx, ty } | Event::TileSet { tx, ty, .. } => {
                #[allow(clippy::cast_precision_loss)]
                let ts = elora_sim::TILE_SIZE as f32;
                #[allow(clippy::cast_precision_loss)]
                let center = Vec2::new((tx as f32 + 0.5) * ts, (ty as f32 + 0.5) * ts);
                self.burst(center, CRUMB, &CRUMBS);
            }
            _ => {}
        }
    }

    fn hit(&mut self) {
        if self.settings.hit_marker {
            self.hit_marker = HIT_MARKER_TIME;
        }
    }

    fn body(&self, slot: usize) -> Color {
        self.colors.get(&slot).copied().unwrap_or(FALLBACK_BODY)
    }

    fn step(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.age += dt;
            p.vel.y += p.gravity * dt;
            p.vel *= (1.0 - p.drag * dt).max(0.0);
            p.pos += p.vel * dt;
        }
        self.particles.retain(|p| p.age < p.life);
        for f in &mut self.flashes {
            f.age += dt;
        }
        self.flashes.retain(|f| f.age < f.life);
        for r in &mut self.roots {
            r.1 += dt;
        }
        self.roots.retain(|r| r.1 < ROOT_LIFE);
        for c in &mut self.cracks {
            c.1 += dt;
        }
        self.cracks.retain(|c| c.1 < CRACK_LIFE);
        self.shake = (self.shake - SHAKE_DECAY * dt).max(0.0);
        self.hit_marker = (self.hit_marker - dt).max(0.0);
    }

    /// Verschiebung der Kamera durch Wackeln (Quadrat der Stärke, glatte Schwingungen).
    pub fn camera_offset(&self) -> Vec2 {
        let s = self.shake * self.shake * SHAKE_MAX;
        let t = self.time;
        Vec2::new(
            (t * 47.0).sin() + (t * 83.0).sin() * 0.5,
            (t * 59.0).cos() + (t * 71.0).sin() * 0.5,
        ) * (s / 1.5)
    }

    pub fn draw(&self, batch: &mut ShapeBatch) {
        let mut tint = Tint::new(vec![WHITE]);
        // Risse im dünnen Eis: wachsen, bis es bricht
        #[allow(clippy::cast_precision_loss)]
        let ts = elora_sim::TILE_SIZE as f32;
        for &(min, age) in &self.cracks {
            let k = (age / 0.6).min(1.0);
            let c = Color::rgba(0.18, 0.43, 0.6, 0.9);
            let mid = min + Vec2::new(ts * 0.5, 3.0);
            for (dx, dy) in [(-12.0, 6.0), (10.0, 8.0), (-3.0, 10.0)] {
                batch.stroke_polyline(
                    &[
                        mid,
                        mid + Vec2::new(dx * 0.5, dy * 0.4) * k,
                        mid + Vec2::new(dx, dy) * k,
                    ],
                    1.6,
                    c,
                );
            }
        }
        // Wurzelstöße: schießen hoch, bleiben kurz, ziehen sich zurück
        for &(foot, age) in &self.roots {
            let t = age / ROOT_LIFE;
            let grow = if t < 0.15 {
                t / 0.15
            } else {
                1.0 - ((t - 0.6) / 0.4).max(0.0)
            };
            let h = 100.0 * grow;
            for (dx, k) in [(-16.0, 0.7), (0.0, 1.0), (16.0, 0.8)] {
                let base = foot + Vec2::new(dx, 0.0);
                let tip = base + Vec2::new(dx * 0.4, -h * k);
                let w = 10.0 * k;
                let outline = [
                    base + Vec2::new(-w - 2.0, 2.0),
                    tip + Vec2::new(0.0, -3.0),
                    base + Vec2::new(w + 2.0, 2.0),
                ];
                let fill = [base + Vec2::new(-w, 0.0), tip, base + Vec2::new(w, 0.0)];
                batch.fill_polygon(&outline, Color::hex(0x2b2b2b));
                batch.fill_polygon(&fill, Color::hex(0x8a6040));
            }
        }
        for f in &self.flashes {
            let t = f.age / f.life;
            batch.fill_circle(
                f.pos,
                f.radius * (0.6 + 0.4 * t),
                Color::rgba(1.0, 0.96, 0.8, 0.85 * (1.0 - t)),
            );
        }
        for p in &self.particles {
            let t = p.age / p.life;
            let r = p.size.0 + (p.size.1 - p.size.0) * t;
            let mut c = p.color;
            c.0[3] *= 1.0 - t * t;
            if p.star {
                // vierzackiges Sternchen, funkelt leicht
                let twinkle = 1.0 + 0.35 * (p.age * 9.0 + p.pos.x).sin();
                let (tip, side) = (r * 2.4 * twinkle, r * 0.7);
                let pts = [
                    p.pos + Vec2::new(0.0, -tip),
                    p.pos + Vec2::new(side, -side),
                    p.pos + Vec2::new(tip, 0.0),
                    p.pos + Vec2::new(side, side),
                    p.pos + Vec2::new(0.0, tip),
                    p.pos + Vec2::new(-side, side),
                    p.pos + Vec2::new(-tip, 0.0),
                    p.pos + Vec2::new(-side, -side),
                ];
                batch.fill_polygon(&pts, c);
                continue;
            }
            tint.colors[0] = c;
            batch.draw_mesh(
                &self.circle,
                &Affine::translate(p.pos).then(Affine::scale(r, r)),
                &tint,
            );
        }
    }

    /// Treffer-Marker: kurzes X um das Fadenkreuz.
    pub fn draw_hit_marker(&self, batch: &mut ShapeBatch, crosshair: Vec2) {
        if self.hit_marker <= 0.0 {
            return;
        }
        let t = self.hit_marker / HIT_MARKER_TIME;
        let c = Color::rgba(1.0, 1.0, 1.0, t);
        for (dx, dy) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
            let d = Vec2::new(dx, dy) * std::f32::consts::FRAC_1_SQRT_2;
            batch.stroke_line(
                crosshair + d * 10.0,
                crosshair + d * (16.0 + 3.0 * t),
                2.5,
                c,
            );
        }
    }

    #[cfg(test)]
    fn particle_count(&self) -> usize {
        self.particles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_sim::DeathCause;

    #[test]
    fn events_spawn_particles_that_expire() {
        let mut fx = Effects::default();
        let scene = Scene::default();
        let events = [
            Event::Explosion {
                owner: 0,
                pos: Vec2::new(10.0, 10.0),
            },
            Event::Death {
                player: 3,
                killer: None,
                cause: DeathCause::World,
                pos: Vec2::default(),
            },
        ];
        fx.update(0.016, &events, &scene, &[], |_| WHITE);
        assert!(fx.particle_count() > 20);
        for _ in 0..100 {
            fx.update(0.02, &[], &scene, &[], |_| WHITE);
        }
        assert_eq!(fx.particle_count(), 0, "alle Partikel verblasst");
    }

    /// Momentaufnahmen zur Sichtprüfung: `cargo test -p elora-client --bin elora effects_sheet -- --ignored`,
    /// danach `cargo xtask svg-preview target/effects.svg target/effects.png 1200`.
    #[test]
    #[ignore = "erzeugt nur eine Datei zur Sichtprüfung"]
    fn effects_sheet() {
        let scene = Scene::default();
        let mut batch = ShapeBatch::default();
        // je Effekt: (Ereignis, Zeitpunkte der Aufnahmen)
        let cases = [
            Event::Explosion {
                owner: 0,
                pos: Vec2::default(),
            },
            Event::Death {
                player: 1,
                killer: None,
                cause: DeathCause::World,
                pos: Vec2::default(),
            },
            Event::HammerHit {
                owner: 0,
                pos: Vec2::default(),
            },
            Event::Spawn {
                player: 0,
                pos: Vec2::default(),
            },
        ];
        for (row, e) in cases.iter().enumerate() {
            for (col, t) in [0.03_f32, 0.12, 0.3].iter().enumerate() {
                let mut fx = Effects::default();
                fx.colors.insert(1, Color::hex(0x5aaee8));
                fx.update(0.001, std::slice::from_ref(e), &scene, &[], |_| WHITE);
                let mut left = *t;
                while left > 0.0 {
                    fx.update(0.01, &[], &scene, &[], |_| WHITE);
                    left -= 0.01;
                }
                #[allow(clippy::cast_precision_loss)]
                let offset = Vec2::new(150.0 + col as f32 * 300.0, 100.0 + row as f32 * 200.0);
                for p in &mut fx.particles {
                    p.pos += offset;
                }
                for f in &mut fx.flashes {
                    f.pos += offset;
                }
                fx.draw(&mut batch);
            }
        }
        let svg = batch.debug_svg(
            Vec2::default(),
            Vec2::new(900.0, 800.0),
            Color::hex(0x8fb8d9),
        );
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/effects.svg"),
            svg,
        )
        .unwrap();
    }

    #[test]
    fn switches_disable_shake_and_marker() {
        let mut fx = Effects::with_settings(EffectSettings {
            camera_shake: false,
            hit_marker: false,
        });
        fx.add_shake(1.0);
        fx.hit();
        assert!(fx.camera_offset().length() < 1e-6);
        assert!(fx.hit_marker <= 0.0);
        fx.settings = EffectSettings::default();
        fx.add_shake(1.0);
        fx.hit();
        fx.time = 0.3;
        assert!(fx.camera_offset().length() > 0.5);
        assert!(fx.hit_marker > 0.0);
    }
}

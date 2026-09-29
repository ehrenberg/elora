//! Elora-Figur (M5.3, E-094): Entwurf B aus `assets/elora/elora.svg`, animiert.
//!
//! Reine Darstellung – Hitbox und Physik bleiben unverändert. Der Ursprung des
//! Assets ist der Bodenkontakt; er liegt an der Hitbox-Unterkante.
//!
//! - **Squash & Stretch:** gedämpfte Feder, angestoßen bei Sprung und Landung,
//!   dazu Streckung in der Luft nach Fallgeschwindigkeit. Fläche bleibt erhalten.
//! - **Neigung** in Laufrichtung, **Füße** laufen nach zurückgelegtem Weg.
//! - **Augen** folgen dem Ziel und blinzeln gelegentlich.

use std::collections::HashMap;

use elora_client::scene::{Scene, SceneChar};
use elora_render::{Affine, Mesh, ShapeBatch, SvgAsset, Tint};
use elora_sim::{Collision, Event, HookState, PHYS_SIZE, Vec2, Weapon};

/// Welteinheiten je Asset-Einheit (sichtbarer Körper ≈ 36, E-087).
const SCALE: f32 = 0.36;
/// Ruhepositionen der Teile im Asset (für Animationen um ihren Mittelpunkt).
const EYES_CENTER: Vec2 = Vec2::new(14.0, -63.0);
/// Wie weit die Augen dem Ziel folgen (Asset-Einheiten).
const EYES_LOOK: f32 = 5.0;

/// Feder für Squash & Stretch: Frequenz (Hz) und Dämpfung.
const SPRING_HZ: f32 = 3.2;
const SPRING_DAMPING: f32 = 0.35;
/// Anstoß der Feder je Welteinheit/Tick Fallgeschwindigkeit bei der Landung.
const LANDING_KICK: f32 = 0.28;
const JUMP_KICK: f32 = 3.0;
/// Hammer-Schwung: Dauer (s) und Ausholwinkel (rad).
const SWING_TIME: f32 = 0.14;
const SWING_ANGLE: f32 = 1.4;

/// Farbschlüssel im Asset (E-095).
pub const KEY_EYES: usize = 0;
pub const KEY_BODY: usize = 1;
pub const KEY_FEET: usize = 2;

/// Geladene Teile der Figur.
#[derive(Debug)]
pub struct FigureArt {
    body: Mesh,
    eyes: Mesh,
    foot_back: Mesh,
    foot_front: Mesh,
}

impl FigureArt {
    /// # Panics
    /// Wenn das eingebettete Asset fehlerhaft ist (wird von Tests abgedeckt).
    pub fn load() -> Self {
        let asset = SvgAsset::load(include_bytes!("../../../assets/elora/elora.svg"), 0.3)
            .expect("assets/elora/elora.svg lesbar");
        let part = |n: &str| {
            asset
                .part(n)
                .cloned()
                .unwrap_or_else(|| panic!("Teil `{n}` fehlt in elora.svg"))
        };
        Self {
            body: part("body"),
            eyes: part("eyes"),
            foot_back: part("foot-back"),
            foot_front: part("foot-front"),
        }
    }
}

/// Animationszustand einer Figur.
#[derive(Debug, Clone, Copy, Default)]
struct Anim {
    grounded: bool,
    vel_y: f32,
    /// Auslenkung der Feder: > 0 gestreckt, < 0 gestaucht.
    squash: f32,
    squash_vel: f32,
    /// Hammer-Schwung: 1 = gerade ausgeholt, 0 = fertig.
    swing: f32,
}

/// Landung einer Figur im letzten Frame (für Staub, M5.6).
#[derive(Debug, Clone, Copy)]
pub struct Landing {
    /// Bodenkontakt (Hitbox-Unterkante).
    pub pos: Vec2,
    /// Stärke nach Fallgeschwindigkeit, etwa 0..2.
    pub strength: f32,
}

/// Animationszustände aller sichtbaren Figuren.
#[derive(Debug, Default)]
pub struct Figures {
    anims: HashMap<usize, Anim>,
    time: f32,
    landings: Vec<Landing>,
}

impl Figures {
    pub fn update(&mut self, dt: f32, scene: &Scene, collision: &Collision, events: &[Event]) {
        let dt = dt.min(0.05);
        self.time += dt;
        self.landings.clear();
        self.anims
            .retain(|slot, _| scene.chars.iter().any(|c| c.slot == *slot));
        for c in &scene.chars {
            let core = &c.ch.core;
            let grounded = core.is_grounded(collision);
            let a = self.anims.entry(c.slot).or_insert(Anim {
                grounded,
                ..Anim::default()
            });
            if grounded && !a.grounded {
                // Landung: je schneller der Fall, desto stärker gestaucht
                a.squash_vel -= a.vel_y.clamp(0.0, 25.0) * LANDING_KICK;
                self.landings.push(Landing {
                    pos: c.pos() + Vec2::new(0.0, PHYS_SIZE / 2.0),
                    strength: a.vel_y.clamp(0.0, 25.0) / 12.0,
                });
            } else if !grounded && a.grounded && core.vel.y < 0.0 {
                a.squash_vel += JUMP_KICK;
            }
            a.grounded = grounded;
            a.vel_y = core.vel.y;
            let omega = std::f32::consts::TAU * SPRING_HZ;
            let accel = -omega * omega * a.squash - 2.0 * SPRING_DAMPING * omega * a.squash_vel;
            a.squash_vel += accel * dt;
            a.squash += a.squash_vel * dt;
            a.swing = (a.swing - dt / SWING_TIME).max(0.0);
        }
        for e in events {
            if let Event::Fire {
                player,
                weapon: Weapon::Hammer,
                ..
            } = *e
                && let Some(a) = self.anims.get_mut(&player)
            {
                a.swing = 1.0;
            }
        }
    }

    /// Winkel, um den die Waffe von `slot` gerade aus der Zielrichtung gedreht ist
    /// (Hammer-Schwung: holt nach hinten oben aus und schlägt zum Ziel).
    pub fn weapon_swing(&self, slot: usize, facing: f32) -> f32 {
        let s = self.anims.get(&slot).map_or(0.0, |a| a.swing);
        -facing * SWING_ANGLE * s * s
    }

    /// Landungen des letzten [`Figures::update`].
    pub fn landings(&self) -> &[Landing] {
        &self.landings
    }

    /// Laufzeit der Darstellung in Sekunden (für Animationen).
    pub fn time(&self) -> f32 {
        self.time
    }

    /// Zeichnet eine Figur; `aim` ist die normierte Blickrichtung.
    pub fn draw(
        &self,
        batch: &mut ShapeBatch,
        art: &FigureArt,
        c: &SceneChar,
        aim: Vec2,
        tint: &Tint,
    ) {
        let state = self.anims.get(&c.slot).copied().unwrap_or_default();
        let core = &c.ch.core;
        let vel = core.vel;
        let facing = if aim.x < 0.0 { -1.0 } else { 1.0 };

        // Streckung: Feder + Luft; Fläche erhalten
        let air = if state.grounded {
            0.0
        } else {
            (vel.y.abs() / 60.0).min(0.15)
        };
        let sy = (1.0 + state.squash + air).clamp(0.6, 1.45);
        let sx = 1.0 / sy;
        let mut lean = (vel.x * 0.012).clamp(-0.2, 0.2);
        if matches!(core.hook_state, HookState::Grabbed) {
            // am Hook in Zugrichtung neigen
            let d = (c.hook_pos() - c.pos()).normalize();
            lean = (d.x * 0.25).clamp(-0.25, 0.25);
        }

        let anchor = c.pos() + Vec2::new(0.0, PHYS_SIZE / 2.0);
        let root = Affine::translate(anchor)
            .then(Affine::rotate(lean))
            .then(Affine::scale(facing * SCALE * sx, SCALE * sy));

        // Füße in Blickrichtung (lokal: vorne = +x)
        let forward = vel.x * facing;
        let (back, front) = if !state.grounded {
            (Vec2::new(-4.0, -3.0), Vec2::new(4.0, -7.0))
        } else if vel.x.abs() > 0.5 {
            let phase = c.pos().x / 14.0 * forward.signum();
            let step = |p: f32| Vec2::new(p.sin() * 8.0, -p.cos().max(0.0) * 6.0);
            (step(phase), step(phase + std::f32::consts::PI))
        } else {
            (Vec2::default(), Vec2::default())
        };
        batch.draw_mesh(&art.foot_back, &root.then(Affine::translate(back)), tint);
        batch.draw_mesh(&art.foot_front, &root.then(Affine::translate(front)), tint);
        batch.draw_mesh(&art.body, &root, tint);

        // Augen: Blick zum Ziel, gelegentlich blinzeln
        let look = Vec2::new(aim.x * facing, aim.y) * EYES_LOOK;
        #[allow(clippy::cast_precision_loss)]
        let t = self.time + c.slot as f32 * 1.37;
        let blink = if t % 4.3 > 4.18 { 0.15 } else { 1.0 };
        let eyes = root
            .then(Affine::translate(EYES_CENTER + look))
            .then(Affine::scale(1.0, blink))
            .then(Affine::translate(-EYES_CENTER));
        batch.draw_mesh(&art.eyes, &eyes, tint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_render::Color;
    use elora_sim::{Character, Team};

    /// Posenblatt zur Sichtprüfung: `cargo test -p elora-client --bin elora pose_sheet -- --ignored`,
    /// danach `cargo xtask svg-preview target/figure-poses.svg target/figure-poses.png 1200`.
    #[test]
    #[ignore = "erzeugt nur eine Datei zur Sichtprüfung"]
    fn pose_sheet() {
        let art = FigureArt::load();
        let mut figures = Figures::default();
        let mut batch = ShapeBatch::default();
        let ground = 100.0;
        // (Bezeichnung, Geschwindigkeit, am Boden, Feder, Ziel)
        let poses = [
            ("steht", Vec2::new(0.0, 0.0), true, 0.0, Vec2::new(1.0, 0.0)),
            (
                "Blick links oben",
                Vec2::new(0.0, 0.0),
                true,
                0.0,
                Vec2::new(-0.7, -0.7),
            ),
            ("läuft", Vec2::new(8.0, 0.0), true, 0.0, Vec2::new(1.0, 0.0)),
            ("läuft", Vec2::new(8.0, 0.0), true, 0.0, Vec2::new(1.0, 0.1)),
            (
                "Sprung",
                Vec2::new(4.0, -12.0),
                false,
                0.3,
                Vec2::new(0.8, -0.6),
            ),
            (
                "fällt",
                Vec2::new(2.0, 14.0),
                false,
                0.0,
                Vec2::new(1.0, 0.3),
            ),
            (
                "Landung",
                Vec2::new(0.0, 0.0),
                true,
                -0.35,
                Vec2::new(1.0, 0.0),
            ),
        ];
        let tint = crate::skins::tint(elora_protocol::Skin::default(), Team::None, false, |_| {
            Color::hex(0)
        });
        for (i, (_, vel, grounded, squash, aim)) in poses.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let x = 40.0 + i as f32 * 60.0 + if i == 3 { 7.0 } else { 0.0 };
            let mut ch = Character::spawn(Vec2::new(x, ground - 14.0), 10);
            ch.core.vel = *vel;
            let c = SceneChar {
                slot: i,
                prev: ch.core.clone(),
                ch,
                alpha: 1.0,
                dummy: false,
                local: true,
                team: Team::None,
            };
            figures.anims.insert(
                i,
                Anim {
                    grounded: *grounded,
                    squash: *squash,
                    ..Anim::default()
                },
            );
            figures.draw(&mut batch, &art, &c, aim.normalize(), &tint);
            // Hitbox
            let r = PHYS_SIZE / 2.0;
            let p = c.pos();
            for (a, b) in [
                (Vec2::new(-r, -r), Vec2::new(r, -r)),
                (Vec2::new(r, -r), Vec2::new(r, r)),
                (Vec2::new(r, r), Vec2::new(-r, r)),
                (Vec2::new(-r, r), Vec2::new(-r, -r)),
            ] {
                batch.stroke_line(p + a, p + b, 0.6, Color::hex(0xd9534f));
            }
        }
        batch.fill_rect(
            Vec2::new(0.0, ground),
            Vec2::new(460.0, ground + 12.0),
            Color::hex(0x5b6b7c),
        );
        let svg = batch.debug_svg(
            Vec2::new(0.0, 30.0),
            Vec2::new(460.0, 115.0),
            Color::hex(0x8fb8d9),
        );
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/figure-poses.svg"),
            svg,
        )
        .expect("schreibbar");
    }

    #[test]
    fn asset_has_all_parts() {
        let art = FigureArt::load();
        for m in [&art.body, &art.eyes, &art.foot_back, &art.foot_front] {
            assert!(!m.is_empty());
        }
        // Füße berühren den Boden (Ursprung), Körper steht darüber
        let (_, max) = art.foot_front.bounds().unwrap();
        assert!(max.y.abs() < 2.5, "Fußunterkante {}", max.y);
        let (min, _) = art.body.bounds().unwrap();
        assert!(min.y < -120.0);
    }
}

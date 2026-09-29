//! Bewegung, Sprung und Hook einer Figur
//! (Referenz: Teeworlds `CCharacterCore`, E-007).

use crate::collision::{Collision, Tile};
use crate::input::PlayerInput;
use crate::math::{Vec2, round_to_int, saturated_add};
use crate::tuning::Tuning;

/// Kantenlänge der Kollisionsbox einer Figur.
pub const PHYS_SIZE: f32 = 28.0;
/// Unterhalb dieser Distanz zieht ein Wand-Hook nicht mehr.
const HOOK_MIN_DRAG_DISTANCE: f32 = 46.0;
/// Geschwindigkeits-Obergrenze als Sicherheitsnetz.
const MAX_VELOCITY: f32 = 6000.0;

/// Ereignisse eines Ticks (z. B. für Sounds und Effekte).
pub mod events {
    pub const GROUND_JUMP: u8 = 1 << 0;
    pub const AIR_JUMP: u8 = 1 << 1;
    pub const HOOK_ATTACH_PLAYER: u8 = 1 << 2;
    pub const HOOK_ATTACH_GROUND: u8 = 1 << 3;
    pub const HOOK_HIT_UNHOOKABLE: u8 = 1 << 4;
}

/// Zustand des Hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HookState {
    /// Eingefahren, Hook-Taste noch gedrückt – erst Loslassen erlaubt einen neuen Schuss.
    Retracted,
    #[default]
    Idle,
    /// Fährt zurück, Schritt 1 bis 3.
    Retracting(u8),
    Flying,
    Grabbed,
}

/// Physikalischer Zustand einer Figur.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CharacterCore {
    pub pos: Vec2,
    pub vel: Vec2,
    pub hook_pos: Vec2,
    pub hook_dir: Vec2,
    pub hook_state: HookState,
    pub hook_tick: u32,
    /// Index der gehookten Figur in der Welt.
    pub hooked_player: Option<usize>,
    /// Bit 0: Sprung mit aktuellem Tastendruck ausgeführt; Bit 1: Luftsprung verbraucht.
    pub jumped: u8,
    pub direction: i8,
    /// Blickwinkel in 1/256 Radiant.
    pub angle: i32,
    /// Im letzten Tick ein Todes-Tile berührt.
    pub death: bool,
    /// Ereignisse des letzten Ticks, siehe [`events`].
    pub triggered_events: u8,
    /// Von anderen Figuren per Hook aufgeprägte Geschwindigkeit (wird in `move` angewendet).
    pub hook_drag_vel: Vec2,
}

impl CharacterCore {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            hook_pos: pos,
            ..Self::default()
        }
    }

    /// Steht die Figur auf festem Boden?
    pub fn is_grounded(&self, col: &Collision) -> bool {
        let y = self.pos.y + PHYS_SIZE / 2.0 + 5.0;
        col.is_solid(Vec2::new(self.pos.x + PHYS_SIZE / 2.0, y))
            || col.is_solid(Vec2::new(self.pos.x - PHYS_SIZE / 2.0, y))
    }

    /// Erste Tick-Phase: Eingabe, Kräfte, Hook.
    ///
    /// `others` enthält die Positionen aller Figuren der Welt (Index = Figur-Index,
    /// `None` = kein Spieler); der eigene Eintrag wird über `self_index` übersprungen.
    /// Hook-Kräfte auf andere Figuren werden in `drag_out` addiert.
    #[allow(clippy::too_many_lines)] // bewusst nah an der Referenz gehalten
    /// `input = None`: ohne Eingabe weiterrechnen (wie `Tick(false)` im Original:
    /// Laufrichtung bleibt, Sprung und Hook ändern sich nicht). Für fremde
    /// Figuren in der Client-Vorhersage.
    pub(crate) fn tick(
        &mut self,
        input: Option<&PlayerInput>,
        tuning: &Tuning,
        col: &Collision,
        self_index: usize,
        others: &[Option<Vec2>],
        drag_out: &mut [Vec2],
    ) {
        self.triggered_events = 0;
        let grounded = self.is_grounded(col);

        self.vel.y += tuning.gravity;

        let (max_speed, accel, friction) = if grounded {
            (
                tuning.ground_control_speed,
                tuning.ground_control_accel,
                tuning.ground_friction,
            )
        } else {
            (
                tuning.air_control_speed,
                tuning.air_control_accel,
                tuning.air_friction,
            )
        };

        // Eingabe
        if let Some(input) = input {
            let target = Vec2::new(input.target_x as f32, input.target_y as f32);
            let target_dir = target.normalize();
            self.direction = input.direction.signum();
            self.angle = (target.angle() * 256.0) as i32;

            if input.jump {
                if self.jumped & 1 == 0 {
                    if grounded {
                        self.triggered_events |= events::GROUND_JUMP;
                        self.vel.y = -tuning.ground_jump_impulse;
                        self.jumped |= 1;
                    } else if self.jumped & 2 == 0 {
                        self.triggered_events |= events::AIR_JUMP;
                        self.vel.y = -tuning.air_jump_impulse;
                        self.jumped |= 3;
                    }
                }
            } else {
                self.jumped &= !1;
            }

            if input.hook {
                if self.hook_state == HookState::Idle {
                    self.hook_state = HookState::Flying;
                    self.hook_pos = self.pos + target_dir * PHYS_SIZE * 1.5;
                    self.hook_dir = target_dir;
                    self.hooked_player = None;
                    self.hook_tick = 0;
                }
            } else {
                self.release_hook(HookState::Idle);
            }
        }

        // Laufen
        match self.direction {
            d if d < 0 => self.vel.x = saturated_add(-max_speed, max_speed, self.vel.x, -accel),
            d if d > 0 => self.vel.x = saturated_add(-max_speed, max_speed, self.vel.x, accel),
            _ => self.vel.x *= friction,
        }

        if grounded {
            self.jumped &= !2;
        }

        // Hook-Zustandsautomat
        match self.hook_state {
            HookState::Idle => self.release_hook(HookState::Idle),
            HookState::Retracting(step) if step < 3 => {
                self.hook_state = HookState::Retracting(step + 1);
            }
            HookState::Retracting(_) => self.hook_state = HookState::Retracted,
            HookState::Flying => self.tick_flying_hook(tuning, col, self_index, others),
            HookState::Retracted | HookState::Grabbed => {}
        }

        if self.hook_state == HookState::Grabbed {
            self.tick_grabbed_hook(tuning, others);
        }

        // Spieler untereinander: Kollision und Hook-Zug
        for (i, other) in others.iter().enumerate() {
            let Some(other_pos) = *other else { continue };
            if i == self_index {
                continue;
            }
            let distance = self.pos.distance(other_pos);
            let dir = (self.pos - other_pos).normalize();

            if tuning.player_collision && distance < PHYS_SIZE * 1.25 && distance > 0.0 {
                let a = PHYS_SIZE * 1.45 - distance;
                let mut velocity = 0.5;
                if self.vel.length() > 0.0001 {
                    // bewusst nicht `f32::midpoint`: gleiche Rundung wie die Referenz
                    #[allow(clippy::manual_midpoint)]
                    {
                        velocity = 1.0 - (self.vel.normalize().dot(dir) + 1.0) / 2.0;
                    }
                }
                self.vel += dir * a * (velocity * 0.75);
                self.vel *= 0.85;
            }

            if self.hooked_player == Some(i) && tuning.player_hooking && distance > PHYS_SIZE * 1.5
            {
                let accel = tuning.hook_drag_accel * (distance / tuning.hook_length);
                drag_out[i] += dir * accel * tuning.player_hook_force;
                self.hook_drag_vel -= dir * accel * 0.25;
            }
        }

        if self.vel.length() > MAX_VELOCITY {
            self.vel = self.vel.normalize() * MAX_VELOCITY;
        }
    }

    fn release_hook(&mut self, state: HookState) {
        self.hooked_player = None;
        self.hook_state = state;
        self.hook_pos = self.pos;
    }

    fn tick_flying_hook(
        &mut self,
        tuning: &Tuning,
        col: &Collision,
        self_index: usize,
        others: &[Option<Vec2>],
    ) {
        let mut new_pos = self.hook_pos + self.hook_dir * tuning.hook_fire_speed;
        if self.pos.distance(new_pos) > tuning.hook_length {
            self.hook_state = HookState::Retracting(1);
            new_pos = self.pos + (new_pos - self.pos).normalize() * tuning.hook_length;
        }

        // nicht durch den Boden
        let mut hit_ground = false;
        let mut hit_unhookable = false;
        if let Some((hit_pos, tile)) = col.intersect_line(self.hook_pos, new_pos) {
            new_pos = hit_pos;
            if tile == Tile::Unhookable {
                hit_unhookable = true;
            } else {
                hit_ground = true;
            }
        }

        // zuerst andere Spieler prüfen
        if tuning.player_hooking {
            let mut best = f32::MAX;
            for (i, other) in others.iter().enumerate() {
                let Some(other_pos) = *other else { continue };
                if i == self_index {
                    continue;
                }
                let closest = Vec2::closest_point_on_segment(self.hook_pos, new_pos, other_pos);
                if other_pos.distance(closest) < PHYS_SIZE + 2.0 {
                    let d = self.hook_pos.distance(other_pos);
                    if self.hooked_player.is_none() || d < best {
                        self.triggered_events |= events::HOOK_ATTACH_PLAYER;
                        self.hook_state = HookState::Grabbed;
                        self.hooked_player = Some(i);
                        best = d;
                    }
                }
            }
        }

        if self.hook_state == HookState::Flying {
            if hit_ground {
                self.triggered_events |= events::HOOK_ATTACH_GROUND;
                self.hook_state = HookState::Grabbed;
            } else if hit_unhookable {
                self.triggered_events |= events::HOOK_HIT_UNHOOKABLE;
                self.hook_state = HookState::Retracting(1);
            }
            self.hook_pos = new_pos;
        }
    }

    fn tick_grabbed_hook(&mut self, tuning: &Tuning, others: &[Option<Vec2>]) {
        if let Some(i) = self.hooked_player {
            let Some(p) = others.get(i).copied().flatten() else {
                self.release_hook(HookState::Retracted);
                return;
            };
            self.hook_pos = p;
        }

        // Wand-Hook zieht die Figur
        if self.hooked_player.is_none() && self.hook_pos.distance(self.pos) > HOOK_MIN_DRAG_DISTANCE
        {
            let mut hook_vel = (self.hook_pos - self.pos).normalize() * tuning.hook_drag_accel;
            // nach oben zieht der Hook stärker als nach unten (leichter auf Plattformen)
            if hook_vel.y > 0.0 {
                hook_vel.y *= 0.3;
            }
            // Zug in Laufrichtung wird verstärkt, sonst gedämpft
            let same_dir = (hook_vel.x < 0.0 && self.direction < 0)
                || (hook_vel.x > 0.0 && self.direction > 0);
            hook_vel.x *= if same_dir { 0.95 } else { 0.75 };

            let new_vel = self.vel + hook_vel;
            if new_vel.length() < tuning.hook_drag_speed || new_vel.length() < self.vel.length() {
                self.vel = new_vel;
            }
        }

        self.hook_tick += 1;
        if self.hooked_player.is_some() && self.hook_tick > tuning.player_hook_ticks {
            self.release_hook(HookState::Retracted);
        }
    }

    /// Zweite Tick-Phase: aufgeprägte Hook-Kräfte anwenden und bewegen.
    ///
    /// `others` sind die aktuellen Positionen der anderen Figuren (für Spielerkollision).
    pub(crate) fn apply_drag_and_move(
        &mut self,
        tuning: &Tuning,
        col: &Collision,
        self_index: usize,
        others: &[Option<Vec2>],
    ) {
        let drag = tuning.hook_drag_speed;
        self.vel.x = saturated_add(-drag, drag, self.vel.x, self.hook_drag_vel.x);
        self.vel.y = saturated_add(-drag, drag, self.vel.y, self.hook_drag_vel.y);
        self.hook_drag_vel = Vec2::ZERO;

        self.do_move(tuning, col, self_index, others);
        self.quantize();
    }

    fn do_move(
        &mut self,
        tuning: &Tuning,
        col: &Collision,
        self_index: usize,
        others: &[Option<Vec2>],
    ) {
        let ramp = velocity_ramp(
            self.vel.length() * crate::TICKS_PER_SECOND as f32,
            tuning.velramp_start,
            tuning.velramp_range,
            tuning.velramp_curvature,
        );

        self.vel.x *= ramp;
        let mut new_pos = self.pos;
        self.death = col.move_box(
            &mut new_pos,
            &mut self.vel,
            Vec2::new(PHYS_SIZE, PHYS_SIZE),
            0.0,
        );
        self.vel.x *= 1.0 / ramp;

        if tuning.player_collision {
            let distance = self.pos.distance(new_pos);
            let end = distance as i32 + 1;
            let mut last = self.pos;
            for step in 0..end {
                let a = if distance > 0.0 {
                    step as f32 / distance
                } else {
                    0.0
                };
                let p = self.pos.lerp(new_pos, a);
                for (i, other) in others.iter().enumerate() {
                    let Some(other_pos) = *other else { continue };
                    if i == self_index {
                        continue;
                    }
                    let d = p.distance(other_pos);
                    if d < PHYS_SIZE {
                        if a > 0.0 {
                            self.pos = last;
                        } else if new_pos.distance(other_pos) > d {
                            self.pos = new_pos;
                        }
                        return;
                    }
                }
                last = p;
            }
        }

        self.pos = new_pos;
    }

    /// Rundet den Zustand auf Netzwerk-Genauigkeit (E-021): Positionen auf ganze
    /// Einheiten, Geschwindigkeit und Hook-Richtung auf 1/256.
    pub fn quantize(&mut self) {
        let q = |v: f32| round_to_int(v) as f32;
        let q256 = |v: f32| round_to_int(v * 256.0) as f32 / 256.0;
        self.pos = Vec2::new(q(self.pos.x), q(self.pos.y));
        self.vel = Vec2::new(q256(self.vel.x), q256(self.vel.y));
        self.hook_pos = Vec2::new(q(self.hook_pos.x), q(self.hook_pos.y));
        self.hook_dir = Vec2::new(q256(self.hook_dir.x), q256(self.hook_dir.y));
    }
}

/// Dämpft die Bewegung oberhalb von `start` exponentiell.
pub fn velocity_ramp(value: f32, start: f32, range: f32, curvature: f32) -> f32 {
    if value < start {
        return 1.0;
    }
    1.0 / curvature.powf((value - start) / range)
}

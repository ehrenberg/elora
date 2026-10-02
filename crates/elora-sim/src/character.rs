//! Bewegung, Sprung und Hook einer Figur
//! (Referenz: Teeworlds `CCharacterCore`, E-007).

use crate::ability::{Abilities, Ability};
use crate::collision::{Collision, TILE_SIZE, Tile};
use crate::creature::HookTarget;
use crate::input::PlayerInput;
use crate::math::{Vec2, round_to_int, saturated_add};
use crate::tuning::{Tuning, ms_to_ticks};

/// Kantenlänge der Kollisionsbox einer Figur.
pub const PHYS_SIZE: f32 = 28.0;
/// Unterhalb dieser Distanz zieht ein Wand-Hook nicht mehr.
const HOOK_MIN_DRAG_DISTANCE: f32 = 46.0;
/// Geschwindigkeits-Obergrenze als Sicherheitsnetz.
const MAX_VELOCITY: f32 = 6000.0;

/// Ereignisse eines Ticks (z. B. für Sounds und Effekte).
pub mod events {
    pub const GROUND_JUMP: u16 = 1 << 0;
    pub const AIR_JUMP: u16 = 1 << 1;
    pub const HOOK_ATTACH_PLAYER: u16 = 1 << 2;
    pub const HOOK_ATTACH_GROUND: u16 = 1 << 3;
    pub const HOOK_HIT_UNHOOKABLE: u16 = 1 << 4;
    /// Von einem Sprungfeld geworfen (M6.1).
    pub const JUMP_PAD: u16 = 1 << 5;
    /// Hook-Ruck ausgelöst (E-226).
    pub const HOOK_RUCK: u16 = 1 << 6;
    /// Stampfen begonnen (E-227).
    pub const STOMP: u16 = 1 << 7;
    /// Stampfen aufgeprallt – die Welt wertet die Stoßwelle aus.
    pub const STOMP_LAND: u16 = 1 << 8;
    /// An einer Kletterwand festgehalten (E-228).
    pub const WALL_GRIP: u16 = 1 << 9;
    /// Von einer Kletterwand abgesprungen.
    pub const WALL_JUMP: u16 = 1 << 10;
    /// Gleiten begonnen (E-229).
    pub const GLIDE: u16 = 1 << 11;
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
#[allow(clippy::struct_excessive_bools)] // unabhängige Zustände, keine Zustandsmaschine
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
    pub triggered_events: u16,
    /// Von anderen Figuren per Hook aufgeprägte Geschwindigkeit (wird in `move` angewendet).
    pub hook_drag_vel: Vec2,
    /// „Runter“ gehalten: durch Plattformen fallen (E-141).
    pub drop_through: bool,
    /// Geschwindigkeit des Untergrunds (Beschleuniger, T-35), in `move` angewendet.
    pub belt: f32,
    /// Freigeschaltete Fähigkeiten (Abenteuer, Quellenkampf); leer im Mehrspieler.
    pub abilities: Abilities,
    /// Fähigkeitstaste im letzten Tick gehalten (Flanke für den Hook-Ruck).
    pub ability_held: bool,
    /// Ticks bis zum nächsten Hook-Ruck (A-02).
    pub ruck_cooldown: u32,
    /// Stampft gerade (bis zum Aufprall).
    pub stomping: bool,
    /// Haftet an einer Kletterwand: -1 links, 1 rechts, 0 nicht.
    pub grip: i8,
    /// Verbrauchte Haftzeit seit dem letzten Boden oder Wandsprung (A-06).
    pub grip_ticks: u32,
    /// Gleitet gerade.
    pub gliding: bool,
    /// Gehakte Kreatur (Id, E-233).
    pub hooked_creature: Option<u32>,
    /// Die gehakte Kreatur wird zu Elora gezogen (Heranhooken), statt Elora zu ihr.
    pub pulling: bool,
}

impl CharacterCore {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            hook_pos: pos,
            ..Self::default()
        }
    }

    /// Steht die Figur auf festem Boden oder auf einer Plattform (nicht beim Durchfallen)?
    pub fn is_grounded(&self, col: &Collision) -> bool {
        self.ground_tile(col).is_some()
    }

    /// Tile unter den Füßen, wenn die Figur steht. Spezial-Tiles haben Vorrang
    /// (Sprungfeld vor Beschleuniger vor Eis), damit ein Fuß darauf genügt.
    pub fn ground_tile(&self, col: &Collision) -> Option<Tile> {
        let bottom = self.pos.y + PHYS_SIZE / 2.0;
        let y = bottom + 5.0;
        let feet = [
            col.tile_at(Vec2::new(self.pos.x + PHYS_SIZE / 2.0, y)),
            col.tile_at(Vec2::new(self.pos.x - PHYS_SIZE / 2.0, y)),
        ];
        #[allow(clippy::cast_precision_loss)]
        let platform_top = (crate::math::round_to_int(y).div_euclid(TILE_SIZE) * TILE_SIZE) as f32;
        let stands = |t: Tile| {
            t.is_solid()
                || (t == Tile::Platform && !self.drop_through && bottom <= platform_top + 0.5)
        };
        let rank = |t: Tile| match t {
            Tile::JumpPad(_) => 3,
            Tile::Conveyor(_) => 2,
            Tile::Ice => 1,
            _ => 0,
        };
        feet.into_iter()
            .filter(|t| stands(*t))
            .max_by_key(|t| rank(*t))
    }

    /// Erste Tick-Phase: Eingabe, Kräfte, Hook.
    ///
    /// `others` enthält die Positionen aller Figuren der Welt (Index = Figur-Index,
    /// `None` = kein Spieler); der eigene Eintrag wird über `self_index` übersprungen.
    /// Hook-Kräfte auf andere Figuren werden in `drag_out` addiert.
    /// `creatures` sind die Kreaturen als Hook-Ziele (im Mehrspieler leer).
    #[allow(clippy::too_many_lines, clippy::too_many_arguments)] // bewusst nah an der Referenz gehalten
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
        creatures: &[HookTarget],
        drag_out: &mut [Vec2],
    ) {
        self.triggered_events = 0;
        // „Runter“ frisch gedrückt (vor dem Überschreiben von `drop_through`)
        let down_pressed = input.is_some_and(|i| i.down && !self.drop_through);
        // Stand die Figur vor diesem Druck? Dann fällt sie durch die Plattform statt zu stampfen
        let stood = self.is_grounded(col);
        if let Some(input) = input {
            // beim Stampfen landet die Figur auch auf Plattformen
            self.drop_through = input.down && !self.stomping;
        }
        let ground = self.ground_tile(col);
        let grounded = ground.is_some();

        self.vel.y += tuning.gravity;

        if let Some(input) = input {
            self.tick_abilities(input, tuning, col, grounded, down_pressed && !stood);
        }

        let (max_speed, accel, friction) = if ground == Some(Tile::Ice) {
            (
                tuning.ground_control_speed,
                tuning.ice_accel,
                tuning.ice_friction,
            )
        } else if grounded {
            (
                tuning.ground_control_speed,
                tuning.ground_control_accel,
                tuning.ground_friction,
            )
        } else if self.gliding {
            (
                tuning.glide_control_speed,
                tuning.air_control_accel,
                tuning.air_friction,
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
                    if self.grip != 0 {
                        // Wandsprung: weg von der Wand, Doppelsprung bleibt erhalten
                        self.triggered_events |= events::WALL_JUMP;
                        self.vel = Vec2::new(
                            -f32::from(self.grip) * tuning.wall_jump_x,
                            -tuning.wall_jump_y,
                        );
                        self.jumped |= 1;
                        self.grip = 0;
                        self.grip_ticks = 0;
                    } else if grounded {
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
            self.grip_ticks = 0;
        }
        if self.stomping {
            self.vel = Vec2::new(0.0, self.vel.y.max(tuning.stomp_speed));
        }

        // Spezial-Tiles unter den Füßen (M6.1)
        self.belt = match ground {
            Some(Tile::Conveyor(dir)) => dir.sign() * tuning.conveyor_speed,
            _ => 0.0,
        };
        if let Some(Tile::JumpPad(dir)) = ground
            && self.vel.y >= 0.0
        {
            self.vel = dir.vector() * tuning.jump_pad_force;
            self.triggered_events |= events::JUMP_PAD;
        }

        // Hook-Zustandsautomat
        match self.hook_state {
            HookState::Idle => self.release_hook(HookState::Idle),
            HookState::Retracting(step) if step < 3 => {
                self.hook_state = HookState::Retracting(step + 1);
            }
            HookState::Retracting(_) => self.hook_state = HookState::Retracted,
            HookState::Flying => {
                self.tick_flying_hook(tuning, col, self_index, others, creatures);
            }
            HookState::Retracted | HookState::Grabbed => {}
        }

        if self.hook_state == HookState::Grabbed {
            self.tick_grabbed_hook(tuning, others, creatures);
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
        self.hooked_creature = None;
        self.pulling = false;
        self.hook_state = state;
        self.hook_pos = self.pos;
    }

    fn tick_flying_hook(
        &mut self,
        tuning: &Tuning,
        col: &Collision,
        self_index: usize,
        others: &[Option<Vec2>],
        creatures: &[HookTarget],
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
            if tile.is_hookable() {
                hit_ground = true;
            } else {
                hit_unhookable = true;
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

        // dann Kreaturen (E-233)
        if self.hook_state == HookState::Flying {
            let mut best = f32::MAX;
            for c in creatures {
                let closest = Vec2::closest_point_on_segment(self.hook_pos, new_pos, c.pos);
                let d = self.hook_pos.distance(c.pos);
                if c.pos.distance(closest) < c.radius + 2.0 && d < best {
                    best = d;
                    self.triggered_events |= events::HOOK_ATTACH_PLAYER;
                    self.hook_state = HookState::Grabbed;
                    self.hooked_creature = Some(c.id);
                    self.pulling = c.small && self.abilities.has(Ability::Pull);
                    self.hook_tick = 0;
                    self.hook_pos = c.pos;
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

    fn tick_grabbed_hook(
        &mut self,
        tuning: &Tuning,
        others: &[Option<Vec2>],
        creatures: &[HookTarget],
    ) {
        if let Some(id) = self.hooked_creature {
            let Some(c) = creatures.iter().find(|c| c.id == id) else {
                self.release_hook(HookState::Retracted);
                return;
            };
            self.hook_pos = c.pos;
        }
        if let Some(i) = self.hooked_player {
            let Some(p) = others.get(i).copied().flatten() else {
                self.release_hook(HookState::Retracted);
                return;
            };
            self.hook_pos = p;
        }

        // Wand-Hook (oder Kreatur ohne Heranhooken) zieht die Figur
        if self.hooked_player.is_none()
            && !self.pulling
            && self.hook_pos.distance(self.pos) > HOOK_MIN_DRAG_DISTANCE
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
        if (self.hooked_player.is_some() || self.hooked_creature.is_some())
            && self.hook_tick > tuning.player_hook_ticks
        {
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
        if self.stomping && self.is_grounded(col) {
            self.stomping = false;
            self.triggered_events |= events::STOMP_LAND;
        }
    }

    /// Fähigkeiten vor der Laufsteuerung: Hook-Ruck, Stampfen, Eisgriff, Gleiten.
    fn tick_abilities(
        &mut self,
        input: &PlayerInput,
        tuning: &Tuning,
        col: &Collision,
        grounded: bool,
        down_pressed: bool,
    ) {
        let a = self.abilities;
        let ability_pressed = input.ability && !self.ability_held;
        self.ability_held = input.ability;
        self.ruck_cooldown = self.ruck_cooldown.saturating_sub(1);

        // Hook-Ruck (E-226): nur an einer Wand, nicht an Spielern
        if a.has(Ability::HookRuck)
            && ability_pressed
            && self.ruck_cooldown == 0
            && self.hook_state == HookState::Grabbed
            && self.hooked_player.is_none()
            && !self.pulling
            && self.hook_pos.distance(self.pos) > HOOK_MIN_DRAG_DISTANCE
        {
            self.vel = (self.hook_pos - self.pos).normalize() * tuning.ruck_speed;
            self.ruck_cooldown = ms_to_ticks(tuning.ruck_cooldown);
            self.triggered_events |= events::HOOK_RUCK;
        }

        // Stampfen (E-227): „Runter“ in der Luft; der Hook lässt los
        if grounded {
            self.stomping = false;
        } else if a.has(Ability::Stomp) && down_pressed && !self.stomping {
            self.stomping = true;
            self.drop_through = false;
            self.release_hook(HookState::Retracted);
            self.triggered_events |= events::STOMP;
        }

        // Eisgriff (E-228): in der Luft gegen eine Kletterwand laufen
        let was_gripping = self.grip != 0;
        self.grip = 0;
        if a.has(Ability::Grip)
            && !grounded
            && !self.stomping
            && input.direction != 0
            && self.vel.y >= 0.0
            && self.grip_ticks < ms_to_ticks(tuning.grip_time)
            && self.touches_climb(col, input.direction.signum())
        {
            self.grip = input.direction.signum();
            self.grip_ticks += 1;
            self.vel.y = self.vel.y.min(tuning.grip_slide_speed);
            self.jumped &= !2;
            if !was_gripping {
                self.triggered_events |= events::WALL_GRIP;
            }
        }

        // Gleiten (E-229): Springen halten beim Fallen, wenn der Doppelsprung verbraucht ist
        let was_gliding = self.gliding;
        self.gliding = a.has(Ability::Glide)
            && !grounded
            && !self.stomping
            && self.grip == 0
            && input.jump
            && self.jumped & 2 != 0
            && self.vel.y > 0.0;
        if self.gliding {
            self.vel.y = self.vel.y.min(tuning.glide_fall_speed);
            if !was_gliding {
                self.triggered_events |= events::GLIDE;
            }
        }
    }

    /// Berührt die Figur seitlich (`side` -1/1) eine Kletterwand?
    fn touches_climb(&self, col: &Collision, side: i8) -> bool {
        let x = self.pos.x + f32::from(side) * (PHYS_SIZE / 2.0 + 1.0);
        let h = PHYS_SIZE / 2.0 - 2.0;
        [self.pos.y - h, self.pos.y + h]
            .into_iter()
            .any(|y| col.tile_at(Vec2::new(x, y)) == Tile::Climb)
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
        // Laufband: Untergrund bewegt die Figur mit, ohne ihre eigene Geschwindigkeit zu ändern
        let mut moved = self.vel + Vec2::new(self.belt, 0.0);
        self.death = col.move_box_platforms(
            &mut new_pos,
            &mut moved,
            Vec2::new(PHYS_SIZE, PHYS_SIZE),
            0.0,
            !self.drop_through,
        );
        self.vel.y = moved.y;
        // an einer Wand gestoppt: auch die eigene Geschwindigkeit ist weg
        self.vel.x = if moved.x == 0.0 && self.vel.x + self.belt != 0.0 {
            0.0
        } else {
            moved.x - self.belt
        };
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

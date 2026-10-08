//! Movement, jump and hook of a character
//! (reference: Teeworlds `CCharacterCore`, E-007).

use crate::ability::{Abilities, Ability};
use crate::collision::{Collision, TILE_SIZE, Tile};
use crate::creature::HookTarget;
use crate::input::PlayerInput;
use crate::math::{Vec2, round_to_int, saturated_add};
use crate::tuning::{Tuning, ms_to_ticks};

/// Edge length of a character's collision box.
pub const PHYS_SIZE: f32 = 28.0;
/// Below this distance a wall hook no longer pulls.
const HOOK_MIN_DRAG_DISTANCE: f32 = 46.0;
/// Speed cap as a safety net.
const MAX_VELOCITY: f32 = 6000.0;

/// Events of a tick (e.g. for sounds and effects).
pub mod events {
    pub const GROUND_JUMP: u16 = 1 << 0;
    pub const AIR_JUMP: u16 = 1 << 1;
    pub const HOOK_ATTACH_PLAYER: u16 = 1 << 2;
    pub const HOOK_ATTACH_GROUND: u16 = 1 << 3;
    pub const HOOK_HIT_UNHOOKABLE: u16 = 1 << 4;
    /// Launched by a jump pad (M6.1).
    pub const JUMP_PAD: u16 = 1 << 5;
    /// Hook jerk triggered (E-226).
    pub const HOOK_RUCK: u16 = 1 << 6;
    /// Stomp started (E-227).
    pub const STOMP: u16 = 1 << 7;
    /// Stomp landed – the world evaluates the shockwave.
    pub const STOMP_LAND: u16 = 1 << 8;
    /// Clung to a climbing wall (E-228).
    pub const WALL_GRIP: u16 = 1 << 9;
    /// Jumped off a climbing wall.
    pub const WALL_JUMP: u16 = 1 << 10;
    /// Glide started (E-229).
    pub const GLIDE: u16 = 1 << 11;
}

/// State of the hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HookState {
    /// Retracted, hook key still pressed – only releasing allows a new shot.
    Retracted,
    #[default]
    Idle,
    /// Retracting, step 1 to 3.
    Retracting(u8),
    Flying,
    Grabbed,
}

/// Physical state of a character.
#[derive(Debug, Clone, PartialEq, Default)]
#[allow(clippy::struct_excessive_bools)] // independent states, no state machine
pub struct CharacterCore {
    pub pos: Vec2,
    pub vel: Vec2,
    pub hook_pos: Vec2,
    pub hook_dir: Vec2,
    pub hook_state: HookState,
    pub hook_tick: u32,
    /// Index of the hooked character in the world.
    pub hooked_player: Option<usize>,
    /// Bit 0: jump performed with the current key press; bit 1: air jump used up.
    pub jumped: u8,
    pub direction: i8,
    /// View angle in 1/256 radians.
    pub angle: i32,
    /// Touched a death tile in the last tick.
    pub death: bool,
    /// Events of the last tick, see [`events`].
    pub triggered_events: u16,
    /// Velocity imposed by other characters via hook (applied in `move`).
    pub hook_drag_vel: Vec2,
    /// "Down" held: fall through platforms (E-141).
    pub drop_through: bool,
    /// Velocity of the ground (accelerator, T-35), applied in `move`.
    pub belt: f32,
    /// Unlocked abilities (adventure, spring battle); empty in multiplayer.
    pub abilities: Abilities,
    /// Ability key held in the last tick (edge detection for the hook jerk).
    pub ability_held: bool,
    /// Ticks until the next hook jerk (A-02).
    pub ruck_cooldown: u32,
    /// Ability key pressed while the hook was still flying: jerk as soon as it grabs.
    pub ruck_queued: bool,
    /// Ticks the jerk keeps pulling (A-28).
    pub ruck_ticks: u32,
    /// Currently stomping (until impact).
    pub stomping: bool,
    /// Clinging to a climbing wall: -1 left, 1 right, 0 not.
    pub grip: i8,
    /// Cling time used since the last ground contact or wall jump (A-06).
    pub grip_ticks: u32,
    /// Currently gliding.
    pub gliding: bool,
    /// Hooked creature (id, E-233).
    pub hooked_creature: Option<u32>,
    /// The hooked creature is pulled towards Elora (pull hook) instead of Elora towards it.
    pub pulling: bool,
    /// Colorful rush (mushroom imp, E-311): slower for this many more ticks.
    pub dazed: u32,
    /// Frozen (frost ghost, R2-M2.4): this many more ticks without input (aiming only).
    pub frozen: u32,
    /// Heat bar full (E-320, set by the adventure): slower.
    pub overheated: bool,
    /// Ticks in quicksand (positions are integers: sinking in whole units).
    pub sand_ticks: u32,
}

impl CharacterCore {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            hook_pos: pos,
            ..Self::default()
        }
    }

    /// Is the character standing on solid ground or on a platform (not while falling through)?
    pub fn is_grounded(&self, col: &Collision) -> bool {
        self.ground_tile(col).is_some()
    }

    /// Tile under the feet when the character is standing. Special tiles take precedence
    /// (jump pad before accelerator before ice), so that one foot on it is enough.
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

    /// Are the feet stuck in quicksand (E-318)?
    pub fn in_quicksand(&self, col: &Collision) -> bool {
        col.tile_at(Vec2::new(self.pos.x, self.pos.y + PHYS_SIZE / 2.0 - 1.0)) == Tile::Quicksand
    }

    /// Is the character in ice water (R2-M2.4)?
    pub fn in_ice_water(&self, col: &Collision) -> bool {
        col.tile_at(Vec2::new(self.pos.x, self.pos.y + PHYS_SIZE / 2.0 - 4.0)) == Tile::IceWater
    }

    /// Has the character sunk in completely (head in quicksand)?
    pub fn buried(&self, col: &Collision) -> bool {
        col.tile_at(Vec2::new(self.pos.x, self.pos.y - PHYS_SIZE / 2.0 + 4.0)) == Tile::Quicksand
    }

    /// First tick phase: input, forces, hook.
    ///
    /// `others` contains the positions of all characters in the world (index = character index,
    /// `None` = no player); the own entry is skipped via `self_index`.
    /// Hook forces on other characters are added into `drag_out`.
    /// `creatures` are the creatures as hook targets (empty in multiplayer).
    #[allow(clippy::too_many_lines, clippy::too_many_arguments)] // kept close to the reference
    /// `input = None`: continue without input (like `Tick(false)` in the original:
    /// walking direction stays, jump and hook do not change). For foreign
    /// characters in client prediction.
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
        // frozen: only aiming works, not walking, jumping, hook or fire (D-M24-07)
        let still;
        let input = if self.frozen > 0 {
            self.frozen -= 1;
            still = input.map(|i| PlayerInput {
                target_x: i.target_x,
                target_y: i.target_y,
                ..PlayerInput::default()
            });
            still.as_ref()
        } else {
            input
        };
        // "down" freshly pressed (before `drop_through` is overwritten)
        let down_pressed = input.is_some_and(|i| i.down && !self.drop_through);
        // Was the character standing before this press? Then it falls through the platform
        // instead of stomping
        let stood = self.is_grounded(col);
        if let Some(input) = input {
            // while stomping the character lands on platforms too
            self.drop_through = input.down && !self.stomping;
        }
        let ground = self.ground_tile(col);
        let grounded = ground.is_some();
        let in_sand = self.in_quicksand(col);

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
        } else if grounded || in_sand {
            // wet or snowy ground brakes more softly (R2-W1, A-35)
            let slip = col.wet * tuning.wet_slip;
            (
                tuning.ground_control_speed,
                tuning.ground_control_accel
                    + (tuning.ice_accel - tuning.ground_control_accel) * slip,
                tuning.ground_friction + (tuning.ice_friction - tuning.ground_friction) * slip,
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

        // colorful rush: walk slower (A-23)
        self.dazed = self.dazed.saturating_sub(1);
        let max_speed = if self.dazed > 0 {
            max_speed * tuning.daze_speed
        } else {
            max_speed
        };
        // quicksand and full heat bar (E-318, E-320)
        let max_speed = max_speed
            * if in_sand { tuning.quicksand_speed } else { 1.0 }
            * if self.overheated {
                tuning.heat_speed
            } else {
                1.0
            };

        // input
        if let Some(input) = input {
            let target = Vec2::new(input.target_x as f32, input.target_y as f32);
            let target_dir = target.normalize();
            self.direction = input.direction.signum();
            self.angle = (target.angle() * 256.0) as i32;

            if input.jump {
                if self.jumped & 1 == 0 {
                    if self.grip != 0 {
                        // wall jump: away from the wall, the double jump is kept
                        self.triggered_events |= events::WALL_JUMP;
                        self.vel = Vec2::new(
                            -f32::from(self.grip) * tuning.wall_jump_x,
                            -tuning.wall_jump_y,
                        );
                        self.jumped |= 1;
                        self.grip = 0;
                        self.grip_ticks = 0;
                    } else if grounded || in_sand {
                        // a jump frees from the quicksand (E-318)
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

        // walking
        match self.direction {
            d if d < 0 => self.vel.x = saturated_add(-max_speed, max_speed, self.vel.x, -accel),
            d if d > 0 => self.vel.x = saturated_add(-max_speed, max_speed, self.vel.x, accel),
            _ => self.vel.x *= friction,
        }

        if grounded || in_sand {
            self.jumped &= !2;
            self.grip_ticks = 0;
        } else if col.wind != 0.0 {
            // wind pushes in the air (R2-W1, A-29); barely on the ground
            self.vel.x += col.wind * tuning.wind_push;
        }
        if self.stomping {
            self.vel = Vec2::new(0.0, self.vel.y.max(tuning.stomp_speed));
        }
        // slowly sink in quicksand: one whole unit every few ticks
        if in_sand {
            self.sand_ticks += 1;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let every = (1.0 / tuning.quicksand_sink.max(0.01)).round().max(1.0) as u32;
            let step = if self.sand_ticks.is_multiple_of(every) {
                1.0
            } else {
                0.0
            };
            self.vel.y = self.vel.y.min(step);
        } else {
            self.sand_ticks = 0;
        }

        // special tiles under the feet (M6.1)
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

        // hook state machine
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

        // hook blossom wilts: let go
        if self.hook_state == HookState::Grabbed
            && self.hooked_player.is_none()
            && self.hooked_creature.is_none()
            && col.tile_at(self.hook_pos) == Tile::HookPoint
            && !col.hook_point_active(
                crate::math::round_to_int(self.hook_pos.x).div_euclid(crate::TILE_SIZE),
            )
        {
            self.release_hook(HookState::Retracting(1));
        }
        if self.hook_state == HookState::Grabbed {
            self.tick_grabbed_hook(tuning, others, creatures);
        }

        // players among each other: collision and hook pull
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
                    // deliberately not `f32::midpoint`: same rounding as the reference
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
        self.ruck_ticks = 0;
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

        // not through the ground
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
        // hook blossom in front: grabs in its center (R2-M2.1)
        if let Some(p) = col.intersect_hook_point(self.hook_pos, new_pos) {
            new_pos = p;
            hit_ground = true;
            hit_unhookable = false;
        }

        // check other players first
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

        // then creatures (E-233)
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
                    self.pulling = c.anchor || (c.small && self.abilities.has(Ability::Pull));
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

        // hook jerk: pulls straight to the hook point at full force for a while (E-226, A-28)
        if self.ruck_ticks > 0 {
            self.ruck_ticks -= 1;
            let to = self.hook_pos - self.pos;
            if self.hooked_player.is_none() && !self.pulling && to.length() > PHYS_SIZE {
                self.vel = to.normalize() * tuning.ruck_speed;
            } else {
                self.ruck_ticks = 0;
            }
        }
        // wall hook (or creature without pull hook) pulls the character
        else if self.hooked_player.is_none()
            && !self.pulling
            && self.hook_pos.distance(self.pos) > HOOK_MIN_DRAG_DISTANCE
        {
            let mut hook_vel = (self.hook_pos - self.pos).normalize() * tuning.hook_drag_accel;
            // the hook pulls harder upwards than downwards (easier onto platforms)
            if hook_vel.y > 0.0 {
                hook_vel.y *= 0.3;
            }
            // pull in the walking direction is amplified, otherwise damped
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

    /// Second tick phase: apply imposed hook forces and move.
    ///
    /// `others` are the current positions of the other characters (for player collision).
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

    /// Abilities before the walking control: hook jerk, stomp, ice grip, glide.
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

        // hook jerk (E-226): only on a wall, not on players; pressed early (hook still
        // flying) counts as soon as it grabs
        if ability_pressed && self.hook_state == HookState::Flying {
            self.ruck_queued = true;
        }
        if !matches!(self.hook_state, HookState::Flying | HookState::Grabbed) {
            self.ruck_queued = false;
        }
        if a.has(Ability::HookRuck)
            && (ability_pressed || self.ruck_queued)
            && self.ruck_cooldown == 0
            && self.hook_state == HookState::Grabbed
            && self.hooked_player.is_none()
            && !self.pulling
            && self.hook_pos.distance(self.pos) > HOOK_MIN_DRAG_DISTANCE
        {
            self.vel = (self.hook_pos - self.pos).normalize() * tuning.ruck_speed;
            self.ruck_ticks = ms_to_ticks(tuning.ruck_time);
            self.ruck_cooldown = ms_to_ticks(tuning.ruck_cooldown);
            self.ruck_queued = false;
            self.triggered_events |= events::HOOK_RUCK;
        }

        // stomp (E-227): "down" in the air; the hook lets go
        if grounded {
            self.stomping = false;
        } else if a.has(Ability::Stomp) && down_pressed && !self.stomping {
            self.stomping = true;
            self.drop_through = false;
            self.release_hook(HookState::Retracted);
            self.triggered_events |= events::STOMP;
        }

        // ice grip (E-228): walk against a climbing wall in the air
        let was_gripping = self.grip != 0;
        self.grip = 0;
        // strengthened (A-42): whoever keeps pushing towards the wall pulls up instead of sliding
        let climbing = tuning.grip_climb > 0.0 && was_gripping;
        if a.has(Ability::Grip)
            && !grounded
            && !self.stomping
            && input.direction != 0
            && (self.vel.y >= 0.0 || climbing)
            && self.grip_ticks < ms_to_ticks(tuning.grip_time)
            && self.touches_climb(col, input.direction.signum())
        {
            self.grip = input.direction.signum();
            self.grip_ticks += 1;
            self.vel.y = if tuning.grip_climb > 0.0 {
                -tuning.grip_climb
            } else {
                self.vel.y.min(tuning.grip_slide_speed)
            };
            self.jumped &= !2;
            if !was_gripping {
                self.triggered_events |= events::WALL_GRIP;
            }
        }

        // glide (E-229): hold jump while falling once the double jump is used up
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

    /// Does the character touch a climbing wall on the side (`side` -1/1)?
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
        // conveyor belt: the ground carries the character along without changing its own velocity
        let mut moved = self.vel + Vec2::new(self.belt, 0.0);
        self.death = col.move_box_platforms(
            &mut new_pos,
            &mut moved,
            Vec2::new(PHYS_SIZE, PHYS_SIZE),
            0.0,
            !self.drop_through,
        );
        self.vel.y = moved.y;
        // stopped at a wall: the own velocity is gone too
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

    /// Rounds the state to network precision (E-021): positions to whole
    /// units, velocity and hook direction to 1/256.
    pub fn quantize(&mut self) {
        let q = |v: f32| round_to_int(v) as f32;
        let q256 = |v: f32| round_to_int(v * 256.0) as f32 / 256.0;
        self.pos = Vec2::new(q(self.pos.x), q(self.pos.y));
        self.vel = Vec2::new(q256(self.vel.x), q256(self.vel.y));
        self.hook_pos = Vec2::new(q(self.hook_pos.x), q(self.hook_pos.y));
        self.hook_dir = Vec2::new(q256(self.hook_dir.x), q256(self.hook_dir.y));
    }
}

/// Damps the movement above `start` exponentially.
pub fn velocity_ramp(value: f32, start: f32, range: f32, curvature: f32) -> f32 {
    if value < start {
        return 1.0;
    }
    1.0 / curvature.powf((value - start) / range)
}

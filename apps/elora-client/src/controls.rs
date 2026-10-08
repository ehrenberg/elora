//! Input: bound actions and mouse → [`PlayerInput`] (bindings see [`crate::bindings`]).

use elora_sim::input::INPUT_STATE_MASK;
use elora_sim::{PlayerInput, Vec2};

use crate::bindings::GameAction;

/// Maximum distance of the crosshair with a static camera (original: 400).
pub const MOUSE_MAX_DISTANCE: f32 = 400.0;

/// Key state; the individual `bool`s represent pressed keys.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Controls {
    left: bool,
    right: bool,
    jump: bool,
    down: bool,
    hook: bool,
    ability: bool,
    /// Counter as in the original: every state change +1, odd = pressed.
    fire: u8,
    next_weapon: u8,
    prev_weapon: u8,
    /// Weapon chosen via number key, sent with the next input.
    wanted_weapon: u8,
    /// Crosshair relative to the figure (world units).
    pub mouse_pos: Vec2,
    /// Mouse sensitivity in percent (original: `inp_mousesens`, 100).
    pub sensitivity: f32,
    /// Switch to the picked-up weapon (E-287).
    pub auto_switch: crate::settings::AutoSwitch,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            left: false,
            right: false,
            jump: false,
            down: false,
            hook: false,
            ability: false,
            fire: 0,
            next_weapon: 0,
            prev_weapon: 0,
            wanted_weapon: 0,
            mouse_pos: Vec2::new(100.0, 0.0),
            sensitivity: 100.0,
            auto_switch: crate::settings::AutoSwitch::New,
        }
    }
}

/// Increments a counter (with mask).
fn bump(counter: &mut u8) {
    *counter = counter.wrapping_add(1) & INPUT_STATE_MASK;
}

impl Controls {
    /// Game control for a bound action (M7.5). Returns `true` if it belongs to the
    /// figure (movement, hook, weapons); chat, emotes etc. are handled by the app.
    pub fn action(&mut self, action: GameAction, down: bool) -> bool {
        match action {
            GameAction::Left => self.left = down,
            GameAction::Right => self.right = down,
            GameAction::Jump => self.jump = down,
            GameAction::Down => self.down = down,
            GameAction::Hook => self.hook = down,
            GameAction::Ability => self.ability = down,
            GameAction::Fire => {
                if down != self.fire_held() {
                    bump(&mut self.fire);
                }
            }
            // E-051: 1 hammer, 2 grenade, 3 laser
            GameAction::Hammer if down => self.wanted_weapon = 1,
            GameAction::Grenade if down => self.wanted_weapon = 2,
            GameAction::Laser if down => self.wanted_weapon = 3,
            GameAction::Hammer | GameAction::Grenade | GameAction::Laser => {}
            // counter as in the original: every state change +1
            GameAction::NextWeapon => bump(&mut self.next_weapon),
            GameAction::PrevWeapon => bump(&mut self.prev_weapon),
            _ => return false,
        }
        true
    }

    /// Raw mouse movement: added directly in world units (as in the original).
    /// Choose a weapon as with the weapon key (switch on pickup, E-287).
    pub fn want_weapon(&mut self, w: elora_sim::Weapon) {
        self.wanted_weapon = u8::try_from(w.index() + 1).unwrap_or(0);
    }

    pub fn mouse_motion(&mut self, dx: f64, dy: f64) {
        let factor = self.sensitivity / 100.0;
        self.mouse_pos += Vec2::new(dx as f32, dy as f32) * factor;
        if self.mouse_pos.length() > MOUSE_MAX_DISTANCE {
            self.mouse_pos = self.mouse_pos.normalize() * MOUSE_MAX_DISTANCE;
        }
    }

    fn fire_held(&self) -> bool {
        self.fire & 1 == 1
    }

    /// Release all keys (e.g. on focus loss). Counters keep running.
    pub fn release_all(&mut self) {
        self.left = false;
        self.right = false;
        self.jump = false;
        self.down = false;
        self.hook = false;
        self.ability = false;
        if self.fire_held() {
            bump(&mut self.fire);
        }
    }

    /// Input for the next tick. A number key choice is sent exactly once.
    pub fn player_input(&mut self) -> PlayerInput {
        let mut target_x = self.mouse_pos.x as i32;
        let target_y = self.mouse_pos.y as i32;
        // the aim vector must never be (0, 0)
        if target_x == 0 && target_y == 0 {
            target_x = 1;
        }
        PlayerInput {
            direction: i8::from(self.right) - i8::from(self.left),
            target_x,
            target_y,
            jump: self.jump,
            down: self.down,
            hook: self.hook,
            ability: self.ability,
            fire: self.fire,
            wanted_weapon: std::mem::take(&mut self.wanted_weapon),
            next_weapon: self.next_weapon,
            prev_weapon: self.prev_weapon,
        }
    }
}

//! Eingabe: belegte Aktionen und Maus → [`PlayerInput`] (Belegung siehe [`crate::bindings`]).

use elora_sim::input::INPUT_STATE_MASK;
use elora_sim::{PlayerInput, Vec2};

use crate::bindings::GameAction;

/// Maximale Entfernung des Fadenkreuzes bei statischer Kamera (Original: 400).
pub const MOUSE_MAX_DISTANCE: f32 = 400.0;

/// Tastenzustand; die einzelnen `bool`s bilden gedrückte Tasten ab.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Controls {
    left: bool,
    right: bool,
    jump: bool,
    down: bool,
    hook: bool,
    /// Zähler wie im Original: jede Zustandsänderung +1, ungerade = gedrückt.
    fire: u8,
    next_weapon: u8,
    prev_weapon: u8,
    /// Per Zahlentaste gewählte Waffe, wird mit der nächsten Eingabe gesendet.
    wanted_weapon: u8,
    /// Fadenkreuz relativ zur Figur (Welteinheiten).
    pub mouse_pos: Vec2,
    /// Maus-Empfindlichkeit in Prozent (Original: `inp_mousesens`, 100).
    pub sensitivity: f32,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            left: false,
            right: false,
            jump: false,
            down: false,
            hook: false,
            fire: 0,
            next_weapon: 0,
            prev_weapon: 0,
            wanted_weapon: 0,
            mouse_pos: Vec2::new(100.0, 0.0),
            sensitivity: 100.0,
        }
    }
}

/// Erhöht einen Zähler (mit Maske).
fn bump(counter: &mut u8) {
    *counter = counter.wrapping_add(1) & INPUT_STATE_MASK;
}

impl Controls {
    /// Spielsteuerung für eine belegte Aktion (M7.5). Liefert `true`, wenn sie zur
    /// Figur gehört (Bewegung, Hook, Waffen); Chat, Emotes usw. behandelt die App.
    pub fn action(&mut self, action: GameAction, down: bool) -> bool {
        match action {
            GameAction::Left => self.left = down,
            GameAction::Right => self.right = down,
            GameAction::Jump => self.jump = down,
            GameAction::Down => self.down = down,
            GameAction::Hook => self.hook = down,
            GameAction::Fire => {
                if down != self.fire_held() {
                    bump(&mut self.fire);
                }
            }
            // E-051: 1 Hammer, 2 Granate, 3 Laser
            GameAction::Hammer if down => self.wanted_weapon = 1,
            GameAction::Grenade if down => self.wanted_weapon = 2,
            GameAction::Laser if down => self.wanted_weapon = 3,
            GameAction::Hammer | GameAction::Grenade | GameAction::Laser => {}
            // Zähler wie im Original: jede Zustandsänderung +1
            GameAction::NextWeapon => bump(&mut self.next_weapon),
            GameAction::PrevWeapon => bump(&mut self.prev_weapon),
            _ => return false,
        }
        true
    }

    /// Rohe Maus-Bewegung: wird direkt in Welteinheiten addiert (wie im Original).
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

    /// Alle Tasten loslassen (z. B. bei Fokusverlust). Zähler laufen weiter.
    pub fn release_all(&mut self) {
        self.left = false;
        self.right = false;
        self.jump = false;
        self.down = false;
        self.hook = false;
        if self.fire_held() {
            bump(&mut self.fire);
        }
    }

    /// Eingabe für den nächsten Tick. Eine Zahlentasten-Wahl wird genau einmal gesendet.
    pub fn player_input(&mut self) -> PlayerInput {
        let mut target_x = self.mouse_pos.x as i32;
        let target_y = self.mouse_pos.y as i32;
        // Zielvektor darf nie (0, 0) sein
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
            fire: self.fire,
            wanted_weapon: std::mem::take(&mut self.wanted_weapon),
            next_weapon: self.next_weapon,
            prev_weapon: self.prev_weapon,
        }
    }
}

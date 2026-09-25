//! Eingabe: Tasten und Maus → [`PlayerInput`] (Standardbelegung E-043).

use elora_sim::{PlayerInput, Vec2};
use winit::event::{ElementState, MouseButton};
use winit::keyboard::KeyCode;

/// Maximale Entfernung des Fadenkreuzes bei statischer Kamera (Original: 400).
pub const MOUSE_MAX_DISTANCE: f32 = 400.0;

/// Tastenzustand; die einzelnen `bool`s bilden gedrückte Tasten ab.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Controls {
    left: bool,
    right: bool,
    jump: bool,
    hook: bool,
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
            hook: false,
            mouse_pos: Vec2::new(100.0, 0.0),
            sensitivity: 100.0,
        }
    }
}

impl Controls {
    /// Verarbeitet eine Taste. Liefert `true`, wenn sie zur Spielsteuerung gehört.
    pub fn key(&mut self, code: KeyCode, state: ElementState) -> bool {
        let down = state.is_pressed();
        match code {
            KeyCode::KeyA => self.left = down,
            KeyCode::KeyD => self.right = down,
            KeyCode::Space => self.jump = down,
            _ => return false,
        }
        true
    }

    pub fn mouse_button(&mut self, button: MouseButton, state: ElementState) {
        let down = state.is_pressed();
        // Linke Maustaste (Schießen) folgt in M2
        if button == MouseButton::Right {
            self.hook = down;
        }
    }

    /// Rohe Maus-Bewegung: wird direkt in Welteinheiten addiert (wie im Original).
    pub fn mouse_motion(&mut self, dx: f64, dy: f64) {
        let factor = self.sensitivity / 100.0;
        self.mouse_pos += Vec2::new(dx as f32, dy as f32) * factor;
        if self.mouse_pos.length() > MOUSE_MAX_DISTANCE {
            self.mouse_pos = self.mouse_pos.normalize() * MOUSE_MAX_DISTANCE;
        }
    }

    /// Alle Tasten loslassen (z. B. bei Fokusverlust).
    pub fn release_all(&mut self) {
        let mouse_pos = self.mouse_pos;
        let sensitivity = self.sensitivity;
        *self = Self {
            mouse_pos,
            sensitivity,
            ..Self::default()
        };
    }

    pub fn player_input(&self) -> PlayerInput {
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
            hook: self.hook,
        }
    }
}

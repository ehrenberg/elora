//! Spielwelt: fester Tick über alle Figuren.

use crate::character::CharacterCore;
use crate::collision::Collision;
use crate::input::PlayerInput;
use crate::math::Vec2;
use crate::tuning::Tuning;

/// Ein Slot pro Spieler; `None` = unbesetzt.
#[derive(Debug, Clone)]
pub struct World {
    pub tuning: Tuning,
    pub collision: Collision,
    pub characters: Vec<Option<CharacterCore>>,
    pub tick: u64,
}

impl World {
    pub fn new(tuning: Tuning, collision: Collision) -> Self {
        Self {
            tuning,
            collision,
            characters: Vec::new(),
            tick: 0,
        }
    }

    /// Fügt eine Figur hinzu und liefert ihren Index.
    pub fn spawn(&mut self, pos: Vec2) -> usize {
        let core = CharacterCore::new(pos);
        if let Some(i) = self.characters.iter().position(Option::is_none) {
            self.characters[i] = Some(core);
            i
        } else {
            self.characters.push(Some(core));
            self.characters.len() - 1
        }
    }

    pub fn remove(&mut self, index: usize) {
        if let Some(slot) = self.characters.get_mut(index) {
            *slot = None;
        }
    }

    fn positions(&self) -> Vec<Option<Vec2>> {
        self.characters
            .iter()
            .map(|c| c.as_ref().map(|c| c.pos))
            .collect()
    }

    /// Simuliert einen Tick. `inputs[i]` gehört zur Figur `i`; fehlende Einträge
    /// gelten als leere Eingabe.
    ///
    /// Ablauf wie im Original: erst alle Figuren `tick` (Kräfte, Hook), dann
    /// nacheinander Hook-Zug anwenden, bewegen, quantisieren.
    pub fn step(&mut self, inputs: &[PlayerInput]) {
        let positions = self.positions();
        let mut drag = vec![Vec2::ZERO; self.characters.len()];

        for (i, slot) in self.characters.iter_mut().enumerate() {
            if let Some(core) = slot {
                let input = inputs.get(i).copied().unwrap_or_default();
                core.tick(
                    &input,
                    &self.tuning,
                    &self.collision,
                    i,
                    &positions,
                    &mut drag,
                );
            }
        }

        for (slot, d) in self.characters.iter_mut().zip(drag) {
            if let Some(core) = slot {
                core.hook_drag_vel += d;
            }
        }

        for i in 0..self.characters.len() {
            let positions = self.positions();
            if let Some(core) = &mut self.characters[i] {
                core.apply_drag_and_move(&self.tuning, &self.collision, i, &positions);
            }
        }

        self.tick += 1;
    }
}

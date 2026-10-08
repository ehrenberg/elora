//! Training dummies (E-053): fixed movement patterns, no AI (bots follow after
//! Release 1, E-035). The inputs are derived deterministically from the state.

use crate::Vec2;
use crate::character::{CharacterCore, PHYS_SIZE};
use crate::collision::Collision;
use crate::input::PlayerInput;

/// Movement pattern (map character E-054).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DummyPattern {
    /// `D`
    Stand,
    /// `W`: walks to the wall or edge, then turns around
    Walk,
    /// `J`: jumps regularly, double jump at the apex
    Jump,
    /// `X`: walks and jumps
    WalkJump,
}

/// Interval between two jumps (ticks).
const JUMP_INTERVAL: u64 = 60;

/// Internal state of a dummy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DummyBrain {
    direction: i8,
    /// Tick of the last ground jump, if the double jump is still pending.
    jumped_at: Option<u64>,
}

impl Default for DummyBrain {
    fn default() -> Self {
        Self {
            direction: 1,
            jumped_at: None,
        }
    }
}

impl DummyBrain {
    /// Input for this tick.
    pub fn input(
        &mut self,
        pattern: DummyPattern,
        core: &CharacterCore,
        col: &Collision,
        tick: u64,
    ) -> PlayerInput {
        let walk = matches!(pattern, DummyPattern::Walk | DummyPattern::WalkJump);
        let jump = matches!(pattern, DummyPattern::Jump | DummyPattern::WalkJump);
        let grounded = core.is_grounded(col);

        if walk && grounded {
            let d = f32::from(self.direction);
            let ahead = Vec2::new(core.pos.x + d * (PHYS_SIZE / 2.0 + 4.0), core.pos.y);
            let below_ahead = Vec2::new(
                core.pos.x + d * (PHYS_SIZE / 2.0 + 6.0),
                core.pos.y + PHYS_SIZE,
            );
            if col.is_solid(ahead) || !col.is_solid(below_ahead) {
                self.direction = -self.direction;
            }
        }

        let mut press_jump = false;
        if jump {
            if grounded && tick.is_multiple_of(JUMP_INTERVAL) {
                press_jump = true;
                self.jumped_at = Some(tick);
            } else if let Some(t) = self.jumped_at {
                // Double jump at the apex (key released before)
                if tick > t + 1 && core.vel.y >= 0.0 {
                    press_jump = true;
                    self.jumped_at = None;
                }
            }
        }

        PlayerInput {
            direction: if walk { self.direction } else { 0 },
            target_x: i32::from(self.direction) * 100,
            target_y: 0,
            jump: press_jump,
            ..PlayerInput::default()
        }
    }
}

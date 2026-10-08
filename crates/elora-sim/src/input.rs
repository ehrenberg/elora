//! Input of a player for one tick.

/// Mask of the key counters (like `INPUT_STATE_MASK` in the original).
pub const INPUT_STATE_MASK: u8 = 0x3f;

/// Input of a player. Integer-based, so that it can later be sent losslessly over the
/// network and simulations stay reproducible.
///
/// Fire and weapon switching are **counters** like in the original: every change
/// (press or release) increments the counter by 1; odd = pressed. This way even
/// clicks shorter than a tick are not lost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::struct_excessive_bools)] // keys are independent
pub struct PlayerInput {
    /// Walking direction: -1 left, 0 none, 1 right.
    pub direction: i8,
    /// Target point relative to the character (world units).
    pub target_x: i32,
    pub target_y: i32,
    pub jump: bool,
    pub hook: bool,
    /// Fire counter (odd = pressed).
    pub fire: u8,
    /// Direct weapon selection: 0 = none, otherwise weapon number 1..=3 (E-051).
    pub wanted_weapon: u8,
    /// "Next weapon" counter (mouse wheel).
    pub next_weapon: u8,
    /// "Previous weapon" counter (mouse wheel).
    pub prev_weapon: u8,
    /// "Down" held: fall through platforms (E-141).
    #[cfg_attr(feature = "serde", serde(default))]
    pub down: bool,
    /// "Ability" key held (hook jerk, E-226).
    #[cfg_attr(feature = "serde", serde(default))]
    pub ability: bool,
}

impl Default for PlayerInput {
    fn default() -> Self {
        // Aim direction must never be (0, 0)
        Self {
            direction: 0,
            target_x: 1,
            target_y: 0,
            jump: false,
            hook: false,
            fire: 0,
            wanted_weapon: 0,
            next_weapon: 0,
            prev_weapon: 0,
            down: false,
            ability: false,
        }
    }
}

impl PlayerInput {
    /// Is the fire key currently pressed?
    pub fn fire_held(&self) -> bool {
        self.fire & 1 == 1
    }
}

/// Number of key presses between two counter values (`CountInput` in the original).
pub fn count_presses(prev: u8, cur: u8) -> u32 {
    let (mut i, cur) = (prev & INPUT_STATE_MASK, cur & INPUT_STATE_MASK);
    let mut presses = 0;
    while i != cur {
        i = (i + 1) & INPUT_STATE_MASK;
        if i & 1 == 1 {
            presses += 1;
        }
    }
    presses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_presses_including_wraparound() {
        assert_eq!(count_presses(0, 0), 0);
        assert_eq!(count_presses(0, 1), 1); // pressed
        assert_eq!(count_presses(1, 2), 0); // released
        assert_eq!(count_presses(0, 4), 2); // two short clicks in one tick
        assert_eq!(count_presses(63, 1), 1); // overflow of the mask
    }
}

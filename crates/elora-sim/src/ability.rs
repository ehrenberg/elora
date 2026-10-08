//! Abilities of a character in the adventure and in the spring battle (R2-M1, E-223 to E-230).
//!
//! Without abilities a character behaves exactly like in multiplayer.

/// Set of unlocked abilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Abilities(u8);

/// A single ability, in the order of the regions (E-212).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Ability {
    /// "Ability" key while the hook is attached: jerk towards the hook point (E-226).
    HookJerk,
    /// The hook pulls items and small enemies closer (comes with the creatures, A1.2).
    Pull,
    /// "Down" in the air: thrust downwards, breaks crumbling floor (E-227, E-230).
    Stomp,
    /// Cling to climbing walls and jump off (E-228).
    Grip,
    /// Holding jump while falling after the double jump (E-229).
    Glide,
}

impl Ability {
    pub const ALL: [Self; 5] = [
        Self::HookJerk,
        Self::Pull,
        Self::Stomp,
        Self::Grip,
        Self::Glide,
    ];

    fn bit(self) -> u8 {
        1 << self as u8
    }
}

impl Abilities {
    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(0b1_1111);

    pub fn has(self, a: Ability) -> bool {
        self.0 & a.bit() != 0
    }

    pub fn set(&mut self, a: Ability, on: bool) {
        if on {
            self.0 |= a.bit();
        } else {
            self.0 &= !a.bit();
        }
    }

    #[must_use]
    pub fn with(mut self, a: Ability) -> Self {
        self.set(a, true);
        self
    }

    /// Both together.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn bits(self) -> u8 {
        self.0
    }

    /// From bits; unknown bits are dropped.
    pub fn from_bits(bits: u8) -> Self {
        Self(bits & Self::ALL.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_query() {
        let mut a = Abilities::NONE.with(Ability::Stomp);
        assert!(a.has(Ability::Stomp) && !a.has(Ability::Glide));
        a.set(Ability::Glide, true);
        a.set(Ability::Stomp, false);
        assert_eq!(a, Abilities::NONE.with(Ability::Glide));
        assert_eq!(Abilities::from_bits(0xff), Abilities::ALL);
        assert!(Ability::ALL.iter().all(|&x| Abilities::ALL.has(x)));
    }
}

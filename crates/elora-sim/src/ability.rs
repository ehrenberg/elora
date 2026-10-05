//! Fähigkeiten einer Figur im Abenteuer und im Quellenkampf (R2-M1, E-223 bis E-230).
//!
//! Ohne Fähigkeiten verhält sich eine Figur genau wie im Mehrspieler.

/// Menge freigeschalteter Fähigkeiten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Abilities(u8);

/// Eine einzelne Fähigkeit, in der Reihenfolge der Gebiete (E-212).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Ability {
    /// Taste „Fähigkeit“ bei hängendem Hook: Ruck zum Hook-Punkt (E-226).
    HookRuck,
    /// Hook zieht Gegenstände und kleine Gegner heran (kommt mit den Kreaturen, A1.2).
    Pull,
    /// „Runter“ in der Luft: Stoß nach unten, bricht Bröckelboden (E-227, E-230).
    Stomp,
    /// An Kletterwänden haften und abspringen (E-228).
    Grip,
    /// Springen halten beim Fallen nach dem Doppelsprung (E-229).
    Glide,
}

impl Ability {
    pub const ALL: [Self; 5] = [
        Self::HookRuck,
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

    /// Beide zusammen.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn bits(self) -> u8 {
        self.0
    }

    /// Aus Bits; unbekannte Bits fallen weg.
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

//! Weapons: types, ownership and ammo (E-016, E-051).

use crate::tuning::{Tuning, ms_to_ticks};

/// Weapons in Release 1, in the order of keys 1–3 (E-051).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Weapon {
    Hammer,
    Grenade,
    Laser,
}

impl Weapon {
    pub const ALL: [Self; 3] = [Self::Hammer, Self::Grenade, Self::Laser];

    pub fn index(self) -> usize {
        self as usize
    }

    /// Weapon for key number 1..=3.
    pub fn from_number(n: u8) -> Option<Self> {
        Self::ALL.get(usize::from(n).checked_sub(1)?).copied()
    }

    /// Automatic fire while the key is held (original: grenade, shotgun, laser).
    pub fn full_auto(self) -> bool {
        matches!(self, Self::Grenade | Self::Laser)
    }

    /// Fire delay in ticks.
    pub fn fire_delay(self, t: &Tuning) -> u32 {
        ms_to_ticks(match self {
            Self::Hammer => t.hammer_fire_delay,
            Self::Grenade => t.grenade_fire_delay,
            Self::Laser => t.laser_fire_delay,
        })
    }
}

/// Ammo: `None` = unlimited (hammer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WeaponSlot {
    pub got: bool,
    pub ammo: Option<i32>,
}

/// Weapon state of a character.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arsenal {
    pub slots: [WeaponSlot; 3],
    pub active: Weapon,
    /// Desired weapon; the switch only happens once the reload timer has expired.
    pub queued: Option<Weapon>,
    /// Ticks until the next possible shot.
    pub reload_timer: u32,
}

impl Default for Arsenal {
    /// Starting equipment: only the hammer (E-025).
    fn default() -> Self {
        let mut slots = [WeaponSlot::default(); 3];
        slots[Weapon::Hammer.index()] = WeaponSlot {
            got: true,
            ammo: None,
        };
        Self {
            slots,
            active: Weapon::Hammer,
            queued: None,
            reload_timer: 0,
        }
    }
}

impl Arsenal {
    pub fn slot(&self, w: Weapon) -> &WeaponSlot {
        &self.slots[w.index()]
    }

    pub fn has(&self, w: Weapon) -> bool {
        self.slot(w).got
    }

    /// Grants or removes the hammer (adventure: Elora only gets it from Klonk). Without any
    /// owned weapon the character cannot attack; with the hammer it becomes active if the
    /// active weapon is not owned.
    pub fn set_hammer(&mut self, owned: bool) {
        self.slots[Weapon::Hammer.index()].got = owned;
        if owned && !self.has(self.active) {
            self.active = Weapon::Hammer;
        }
    }

    /// Gives a weapon with `ammo` shots (at most `max`). Returns `false` if the
    /// weapon is already owned and full (then the pickup is not consumed).
    pub fn give(&mut self, w: Weapon, ammo: i32, max: i32) -> bool {
        let slot = &mut self.slots[w.index()];
        if slot.got && slot.ammo.is_none_or(|a| a >= max) {
            return false;
        }
        slot.got = true;
        slot.ammo = Some(ammo.min(max));
        true
    }

    /// Selects the next or previous owned weapon `steps` times.
    pub fn cycle(&self, from: Weapon, steps: u32, forward: bool) -> Weapon {
        let n = Weapon::ALL.len();
        let mut i = from.index();
        let mut left = steps.min(128);
        while left > 0 {
            i = if forward {
                (i + 1) % n
            } else {
                (i + n - 1) % n
            };
            if self.slots[i].got {
                left -= 1;
            }
        }
        Weapon::ALL[i]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_follow_e051() {
        assert_eq!(Weapon::from_number(1), Some(Weapon::Hammer));
        assert_eq!(Weapon::from_number(2), Some(Weapon::Grenade));
        assert_eq!(Weapon::from_number(3), Some(Weapon::Laser));
        assert_eq!(Weapon::from_number(0), None);
        assert_eq!(Weapon::from_number(4), None);
    }

    #[test]
    fn give_respects_full_ammo() {
        let mut a = Arsenal::default();
        assert!(a.give(Weapon::Laser, 10, 10));
        assert!(!a.give(Weapon::Laser, 10, 10));
        a.slots[Weapon::Laser.index()].ammo = Some(3);
        assert!(a.give(Weapon::Laser, 10, 10));
        assert_eq!(a.slot(Weapon::Laser).ammo, Some(10));
        // Hammer (unlimited) cannot be "refilled"
        assert!(!a.give(Weapon::Hammer, 10, 10));
    }

    #[test]
    fn cycle_skips_missing_weapons() {
        let mut a = Arsenal::default();
        assert_eq!(a.cycle(Weapon::Hammer, 1, true), Weapon::Hammer);
        a.give(Weapon::Laser, 10, 10);
        assert_eq!(a.cycle(Weapon::Hammer, 1, true), Weapon::Laser);
        assert_eq!(a.cycle(Weapon::Hammer, 1, false), Weapon::Laser);
        assert_eq!(a.cycle(Weapon::Laser, 2, true), Weapon::Laser);
    }

    #[test]
    fn fire_delays_in_ticks() {
        let t = Tuning::default();
        assert_eq!(Weapon::Hammer.fire_delay(&t), 6);
        assert_eq!(Weapon::Grenade.fire_delay(&t), 25);
        assert_eq!(Weapon::Laser.fire_delay(&t), 37);
    }
}

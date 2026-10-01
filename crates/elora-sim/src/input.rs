//! Eingabe eines Spielers für einen Tick.

/// Maske der Tastenzähler (wie `INPUT_STATE_MASK` im Original).
pub const INPUT_STATE_MASK: u8 = 0x3f;

/// Eingabe eines Spielers. Ganzzahlig, damit sie später verlustfrei über das
/// Netzwerk geht und Simulationen reproduzierbar bleiben.
///
/// Feuer und Waffenwechsel sind **Zähler** wie im Original: Jede Änderung
/// (Drücken oder Loslassen) erhöht den Zähler um 1; ungerade = gedrückt. So gehen
/// auch Klicks, die kürzer als ein Tick sind, nicht verloren.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::struct_excessive_bools)] // Tasten sind unabhängig
pub struct PlayerInput {
    /// Laufrichtung: -1 links, 0 keine, 1 rechts.
    pub direction: i8,
    /// Zielpunkt relativ zur Figur (Welteinheiten).
    pub target_x: i32,
    pub target_y: i32,
    pub jump: bool,
    pub hook: bool,
    /// Feuer-Zähler (ungerade = gedrückt).
    pub fire: u8,
    /// Direkte Waffenwahl: 0 = keine, sonst Waffennummer 1..=3 (E-051).
    pub wanted_weapon: u8,
    /// Zähler „nächste Waffe“ (Mausrad).
    pub next_weapon: u8,
    /// Zähler „vorige Waffe“ (Mausrad).
    pub prev_weapon: u8,
    /// „Runter“ gehalten: durch Plattformen fallen (E-141).
    #[cfg_attr(feature = "serde", serde(default))]
    pub down: bool,
    /// Taste „Fähigkeit“ gehalten (Hook-Ruck, E-226).
    #[cfg_attr(feature = "serde", serde(default))]
    pub ability: bool,
}

impl Default for PlayerInput {
    fn default() -> Self {
        // Zielrichtung darf nie (0, 0) sein
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
    /// Ist die Feuertaste gerade gedrückt?
    pub fn fire_held(&self) -> bool {
        self.fire & 1 == 1
    }
}

/// Anzahl der Tastendrücke zwischen zwei Zählerständen (`CountInput` im Original).
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
        assert_eq!(count_presses(0, 1), 1); // gedrückt
        assert_eq!(count_presses(1, 2), 0); // losgelassen
        assert_eq!(count_presses(0, 4), 2); // zwei kurze Klicks in einem Tick
        assert_eq!(count_presses(63, 1), 1); // Überlauf der Maske
    }
}

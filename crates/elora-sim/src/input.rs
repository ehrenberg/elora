//! Eingabe eines Spielers für einen Tick.

/// Eingabe eines Spielers. Ganzzahlig, damit sie später verlustfrei über das
/// Netzwerk geht und Simulationen reproduzierbar bleiben.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerInput {
    /// Laufrichtung: -1 links, 0 keine, 1 rechts.
    pub direction: i8,
    /// Zielpunkt relativ zur Figur (Welteinheiten).
    pub target_x: i32,
    pub target_y: i32,
    pub jump: bool,
    pub hook: bool,
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
        }
    }
}

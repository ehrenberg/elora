//! Deterministische Spielsimulation von Elora.
//!
//! Enthält reine Spiellogik ohne Abhängigkeiten zu Fenster, Grafik oder Netzwerk
//! (siehe `docs/03-architektur.md`). Server, Client-Vorhersage und Tests nutzen
//! denselben Code.

/// Feste Simulationsrate in Ticks pro Sekunde (T-01).
pub const TICKS_PER_SECOND: u32 = 50;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_rate_matches_tuning() {
        assert_eq!(TICKS_PER_SECOND, 50);
    }
}

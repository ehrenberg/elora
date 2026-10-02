//! Abenteuer von Elora (R2-M1): Inhalte als Daten, Spielstand mit Stufen, Fähigkeitenbaum,
//! Inventar, Ausrüstung, Läden, Waffen-Ausbau, Tod und Speichern.
//!
//! Reine Logik ohne Fenster, Grafik oder Netz – wie `elora-game` für die Mehrspieler-Regeln.
//! Die Simulation (`elora-sim`) bekommt daraus ein Tuning, die Fähigkeiten und den Stand der
//! Figur; Ereignisse der Welt fließen über [`SaveGame::on_event`] zurück.

pub mod check;
pub mod data;
pub mod dialog;
pub mod quest;
pub mod save;
pub mod script;
pub mod session;
pub mod state;
pub mod stats;

pub use data::{Content, GLANZTROPFEN};
pub use dialog::{Conversation, Turn};
pub use quest::Outcome;
pub use session::{Session, SessionEvent};
pub use state::{Location, Notice, Refusal, SaveGame};
pub use stats::Stats;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_content_is_valid() {
        let c = Content::builtin();
        assert!(c.items.len() >= 10);
        assert_eq!(c.skills.len(), 16, "E-242: 16 Knoten");
        assert_eq!(c.upgrades.len(), 9, "3 Waffen × 3 Stufen");
        assert_eq!(c.creatures.len(), 3);
    }

    #[test]
    fn content_from_dir_matches_builtin() {
        let dir = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/adventure"
        ));
        let c = Content::from_dir(dir).unwrap();
        let b = Content::builtin();
        assert_eq!(
            c.dialogs.len(),
            b.dialogs.len(),
            "alle Gespräche eingetragen (data.rs)"
        );
        assert_eq!(c.quests, b.quests);
    }

    #[test]
    fn broken_references_are_rejected() {
        let mut src = data::Sources::builtin();
        let shops = "[[shop]]\nid = \"x\"\nstock = [\"gibt-es-nicht\"]\n";
        src.shops = shops;
        assert!(Content::load(&src).is_err());
    }
}

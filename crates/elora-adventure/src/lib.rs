//! Elora's adventure (R2-M1): content as data, save game with levels, skill tree,
//! inventory, equipment, shops, weapon upgrades, death and saving.
//!
//! Pure logic without window, graphics or network – like `elora-game` for the multiplayer rules.
//! The simulation (`elora-sim`) gets a tuning, the abilities and the state of the character from
//! it; world events flow back via [`SaveGame::on_event`].

#![warn(missing_docs)]

pub mod avalanche;
pub mod check;
pub mod data;
pub mod dialog;
pub mod quest;
pub mod save;
pub mod script;
pub mod session;
pub mod state;
pub mod stats;

pub use data::{Content, GLEAM_DROPS};
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
        assert_eq!(c.skills.len(), 16, "E-242: 16 nodes");
        assert_eq!(c.upgrades.len(), 9, "3 weapons × 3 levels");
        assert_eq!(
            c.creatures.len(),
            21,
            "up to chapter 4 with terrain, enemies and Kristella"
        );
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
            "all dialog files embedded (build.rs)"
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

    #[test]
    fn broken_temperature_bars_are_rejected() {
        let builtin = data::Sources::builtin();
        for (from, to) in [
            ("relief_zones = [\"oasis\"]", "relief_zones = []"),
            (
                "faster_weather = [\"blizzard\"]",
                "faster_weather = [\"hail\"]",
            ),
            ("fill_ms = 20_000", "fill_ms = 0"),
            ("recover = 0.5", "recover = 1.5"),
            ("hot = true", "hot = true\ncold = true"),
        ] {
            let world_map = builtin.world_map.replacen(from, to, 1);
            assert_ne!(world_map, builtin.world_map, "{from}");
            let mut src = data::Sources::builtin();
            src.world_map = &world_map;
            assert!(Content::load(&src).is_err(), "{to}");
        }
    }
}

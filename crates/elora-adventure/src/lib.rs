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
pub mod world_map;

pub use data::{Content, GLEAM_DROPS};
pub use dialog::{Conversation, Turn};
pub use quest::Outcome;
pub use session::{MapSpot, Session, SessionEvent};
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
        // dark mirrors send out existing kinds (R2-M2.5)
        for k in &c.creatures {
            if let elora_sim::Behavior::Mirror { spawn, .. } = &k.behavior {
                assert!(
                    c.creatures.iter().any(|o| &o.name == spawn),
                    "{}: unknown kind `{spawn}`",
                    k.name
                );
            }
        }
        assert_eq!(
            c.creatures.len(),
            27,
            "up to chapter 5 with terrain, enemies, Kristella, the creatures of the night and the \
             climax figures"
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

    /// The bundled quests plus one tune (`[[tune]]` in `quests.toml`).
    fn load_with_tune(tune: &str) -> Result<Content, data::ContentError> {
        let builtin = data::Sources::builtin();
        let quests = format!("{}\n[[tune]]\n{tune}\n", builtin.quests);
        Content::load(&data::Sources {
            quests: &quests,
            ..builtin
        })
    }

    #[test]
    fn tunes_are_read_from_the_quest_file() {
        let c = load_with_tune(
            "id = \"verse\"\nmap = \"gorge-1\"\nnotes = [\"e\", \"g\", \"c2\"]\n\
             if = \"quest well done\"\ndo = [\"flag luma_song.verses +1\"]",
        )
        .unwrap();
        let tune = c.tunes.last().unwrap();
        assert_eq!(tune.pitches(), Some(vec![4, 7, 12]));
        assert_eq!((tune.id.as_str(), tune.map.as_str()), ("verse", "gorge-1"));
    }

    #[test]
    fn broken_tunes_are_rejected() {
        let tune = |map: &str, notes: &str, cond: &str, action: &str| {
            format!(
                "id = \"verse\"\nmap = \"{map}\"\nnotes = [{notes}]\nif = \"{cond}\"\n\
                 do = [\"{action}\"]"
            )
        };
        let (map, notes, cond, action) =
            ("gorge-1", "\"e\", \"g\"", "quest well done", "flag x +1");
        assert!(load_with_tune(&tune(map, notes, cond, action)).is_ok());
        let long = vec!["\"c\""; session::NOTES_KEPT + 1].join(", ");
        for (broken, why) in [
            (tune("nowhere-1", notes, cond, action), "of no area"),
            (tune(map, "", cond, action), "no notes"),
            (tune(map, "\"e\", \"h\"", cond, action), "unknown note `h`"),
            (tune(map, &long, cond, action), "longer than"),
            (tune(map, notes, "quest nope done", action), "unknown quest"),
            (tune(map, notes, cond, "give nothing 1"), "unknown item"),
            (
                format!(
                    "{}\n[[tune]]\n{}",
                    tune(map, notes, cond, action),
                    tune(map, notes, cond, action)
                ),
                "duplicated",
            ),
        ] {
            let e = load_with_tune(&broken).unwrap_err().to_string();
            assert!(e.contains(why), "{why}: {e}");
        }
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

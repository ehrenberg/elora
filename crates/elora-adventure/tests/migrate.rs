//! Save games from 0.9.x (format version 1, German ids) keep working (RF-12, RF-41).

use elora_adventure::save::{Slots, decode};
use elora_adventure::{Content, Session};

const PLAYTEST: &[u8] = include_bytes!("fixtures/0.9.2/playtest.esav");
const NEW_GAME: &[u8] = include_bytes!("fixtures/0.9.2/new-game.esav");

#[test]
fn playtest_save_from_0_9_2_gets_english_ids() {
    let g = decode(PLAYTEST).expect("old save loads");
    assert_eq!((g.level, g.gleam_drops, g.gleam_since_save), (20, 1097, 88));
    assert_eq!(g.inventory.get("amber"), Some(&7));
    assert_eq!(g.inventory.get("clear_crystal"), Some(&8));
    assert_eq!(
        g.equipped.values().next().map(String::as_str),
        Some("fur_boots")
    );
    assert!(g.skills.contains_key("quick_jerk") && g.skills.contains_key("far_hook"));
    assert!(g.defeated.contains("enemy:frost-arena:ice_queen"));
    for flag in [
        "freed.blossom_spring",
        "defeated.root_warden",
        "visited:meadow-1",
        "find:wiese-1:glitter_stone",
        "chest:tauwinkel:chest-treehouse",
        "yard.pull_gate",
        "springs_freed",
    ] {
        let flag = elora_map::rename::translate_id(flag);
        assert!(g.flags.contains_key(flag.as_ref()), "{flag}");
    }
    assert!(
        !g.flags
            .keys()
            .any(|k| k.contains("truhe") || k.contains("befreit"))
    );
}

/// Everything the converted save refers to exists in the new content and maps.
#[test]
fn converted_saves_fit_the_new_content() {
    let c = Content::builtin();
    for data in [PLAYTEST, NEW_GAME] {
        let g = decode(data).expect("old save loads");
        for item in g.inventory.keys().chain(g.equipped.values()) {
            assert!(c.item(item).is_some(), "item {item}");
        }
        for skill in g.skills.keys() {
            assert!(c.skills.iter().any(|n| &n.id == skill), "skill {skill}");
        }
        for quest in g.quests.keys() {
            assert!(c.quests.iter().any(|q| &q.id == quest), "quest {quest}");
        }
        let map = format!(
            "{}/../../maps/adventure/{}.emap",
            env!("CARGO_MANIFEST_DIR"),
            g.location.map
        );
        let map = elora_map::Map::load(std::path::Path::new(&map)).expect("map of the save");
        let mut s = Session::new(c.clone(), g.clone());
        let spawn = g.location.spawn.clone();
        s.enter(&g.location.map, map, &spawn, &elora_sim::Tuning::default());
    }
}

#[test]
fn old_slot_files_are_converted_on_first_read() {
    let dir = std::env::temp_dir().join(format!("elora-saves-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("platz-1.esav"), PLAYTEST).unwrap();
    std::fs::write(dir.join("platz-2.esav"), b"broken").unwrap();
    let slots = Slots::new(&dir);
    let g = slots.load(0).expect("converted").expect("not empty");
    assert_eq!(g.gleam_drops, 1097);
    assert!(dir.join("slot-1.esav").is_file() && !dir.join("platz-1.esav").exists());
    // written in the new format: reads back the same
    assert_eq!(
        decode(&std::fs::read(dir.join("slot-1.esav")).unwrap()).unwrap(),
        g
    );
    // a damaged old slot is reported, not lost
    assert!(slots.load(1).is_err());
    assert!(dir.join("platz-2.esav").exists());
    std::fs::remove_dir_all(&dir).unwrap();
}

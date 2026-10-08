//! Maps from 0.9.x (format version 1, German ids) are converted when they are read (RF-13).

use elora_map::adventure::ObjectKind;
use elora_map::look::Art;

fn load(name: &str) -> elora_map::Map {
    let path = format!(
        "{}/tests/fixtures/0.9.2/{name}.emap",
        env!("CARGO_MANIFEST_DIR")
    );
    elora_map::decode(&std::fs::read(path).expect("fixture")).expect("old map loads")
}

fn arts(map: &elora_map::Map) -> Vec<String> {
    map.decor_back
        .iter()
        .chain(&map.decor_front)
        .chain(map.backgrounds.iter().flat_map(|b| &b.items))
        .filter_map(|d| match &d.art {
            Art::Builtin(n) => Some(n.clone()),
            Art::Image(_) => None,
        })
        .collect()
}

#[test]
fn old_adventure_map_gets_english_ids() {
    let map = load("tauwinkel");
    let ids: Vec<&str> = map
        .adventure
        .objects
        .iter()
        .map(|o| o.id.as_str())
        .collect();
    assert!(ids.contains(&"sign-start"), "{ids:?}");
    assert!(ids.contains(&"chest-treehouse"));
    assert!(
        !ids.iter()
            .any(|i| i.starts_with("schild") || i.starts_with("truhe"))
    );
    let exits: Vec<(&str, &str)> = map
        .adventure
        .objects
        .iter()
        .filter_map(|o| match &o.kind {
            ObjectKind::Exit { map, spawn, .. } => Some((map.as_str(), spawn.as_str())),
            _ => None,
        })
        .collect();
    assert!(exits.contains(&("meadow-1", "west")), "{exits:?}");
    let doors: Vec<&str> = map
        .adventure
        .objects
        .iter()
        .filter_map(|o| match &o.kind {
            ObjectKind::Door { open_if, .. } => Some(open_if.as_str()),
            _ => None,
        })
        .collect();
    assert!(doors.contains(&"not quest glutsand new"), "{doors:?}");
    let art = arts(&map);
    assert!(art.iter().any(|a| a == "house-oma"), "{art:?}");
    assert!(!art.iter().any(|a| a == "haus-oma"));
}

#[test]
fn old_multiplayer_maps_load_with_english_decoration() {
    for name in ["dm-wiese", "ctf-nacht", "wueste-2"] {
        let map = load(name);
        for a in arts(&map) {
            assert_eq!(
                elora_map::rename::translate_id(&a),
                a,
                "{name}: {a} still German"
            );
        }
        // written again, the map is version 2 and reads back the same
        let again = elora_map::decode(&elora_map::encode(&map)).expect("re-encoded");
        assert_eq!(again, map, "{name}");
    }
}

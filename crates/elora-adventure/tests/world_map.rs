//! World map (R2-WM): the goal of the current main quest step on the shipped maps.

use elora_adventure::Content;
use elora_adventure::quest::{Goal, QuestKind, QuestState, QuestStatus};
use elora_adventure::world_map::quest_goal;
use elora_map::Map;

fn maps() -> Vec<(String, Map)> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../maps/adventure");
    let mut v: Vec<(String, Map)> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| {
            let p = e.unwrap().path();
            let name = p.file_stem().unwrap().to_string_lossy().into_owned();
            (name, Map::load(&p).unwrap())
        })
        .collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

#[test]
fn first_step_leads_to_tueftel_in_tauwinkel() {
    let c = Content::builtin();
    let mut save = elora_adventure::Session::new_game(c.clone()).save;
    assert!(
        quest_goal(&c, &save, &[], "tauwinkel").is_none(),
        "no quest before Oma"
    );
    save.quests.insert(
        "well".into(),
        QuestState {
            status: QuestStatus::Active,
            step: 0,
            progress: 0,
        },
    );
    let all = maps();
    let refs: Vec<(&str, &Map)> = all.iter().map(|(n, m)| (n.as_str(), m)).collect();
    let g = quest_goal(&c, &save, &refs, "tauwinkel").expect("a main quest is active");
    assert_eq!(g.quest, "well");
    assert_eq!(g.map.as_deref(), Some("tauwinkel"));
    assert!(g.pos.is_some());
}

#[test]
fn main_quest_steps_find_their_map() {
    let c = Content::builtin();
    let all = maps();
    let refs: Vec<(&str, &Map)> = all.iter().map(|(n, m)| (n.as_str(), m)).collect();
    let mut missing = Vec::new();
    for q in c.quests.iter().filter(|q| q.kind == QuestKind::Main) {
        for (i, step) in q.step.iter().enumerate() {
            if matches!(step.goal, Goal::Manual) {
                continue;
            }
            let mut save = elora_adventure::Session::new_game(c.clone()).save;
            save.quests.clear();
            save.quests.insert(
                q.id.clone(),
                QuestState {
                    status: QuestStatus::Active,
                    step: i,
                    progress: 0,
                },
            );
            let g = quest_goal(&c, &save, &refs, "tauwinkel").unwrap();
            if g.map.is_none() {
                missing.push(format!("{} / {}: {:?}", q.id, step.id, step.goal));
            }
        }
    }
    assert!(missing.is_empty(), "no place for:\n{}", missing.join("\n"));
}

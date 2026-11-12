//! The goal of the current main quest step on the world map (R2-WM, E-372).

use elora_map::{Map, ObjectKind};
use elora_sim::Vec2;

use crate::data::Text;
use crate::quest::{Goal, QuestKind, QuestStatus};
use crate::{Content, SaveGame};

/// Where the current main quest step leads.
#[derive(Debug, Clone, PartialEq)]
pub struct QuestGoal {
    /// Quest id.
    pub quest: String,
    /// Text of the current step.
    pub text: Text,
    /// Map of the goal, if it can be found.
    pub map: Option<String>,
    /// Position on that map (character, enemy, find, zone), if known.
    pub pos: Option<Vec2>,
}

/// The current step of the first active main quest and where its goal is: the step's own `map`,
/// otherwise the map with the character (`talk`, `bring`), the enemy kind (`defeat`), the find
/// or a chest with it (`collect`), the zone (`reach`) or the switch (`flag`). `maps` are all
/// adventure maps; `here` is preferred when it has the goal too.
pub fn quest_goal(
    content: &Content,
    save: &SaveGame,
    maps: &[(&str, &Map)],
    here: &str,
) -> Option<QuestGoal> {
    let (quest, state) = content
        .quests
        .iter()
        .filter(|q| q.kind == QuestKind::Main)
        .find_map(|q| {
            let s = save.quest(&q.id)?;
            (s.status == QuestStatus::Active).then_some((q, s))
        })?;
    let step = quest.step.get(state.step)?;
    let found = |test: &dyn Fn(&ObjectKind, &str) -> bool| -> Option<(String, Option<Vec2>)> {
        let mut hits = maps.iter().flat_map(|(name, m)| {
            m.adventure
                .objects
                .iter()
                .filter(|o| test(&o.kind, &o.id))
                .map(move |o| (*name, o.pos + o.kind.area().unwrap_or(Vec2::ZERO) * 0.5))
        });
        let all: Vec<(&str, Vec2)> = hits.by_ref().collect();
        all.iter()
            .find(|(n, _)| *n == here)
            .or_else(|| all.first())
            .map(|&(n, p)| (n.to_owned(), Some(p)))
    };
    let place = match &step.goal {
        Goal::Talk { who } | Goal::Bring { to: who, .. } => {
            found(&|k, _| matches!(k, ObjectKind::Npc { character, .. } if character == who))
        }
        Goal::Defeat { kind, map, .. } => {
            found(&|k, _| matches!(k, ObjectKind::Creature { kind: c, .. } if c == kind))
                .filter(|(m, _)| map.as_ref().is_none_or(|want| want == m))
                .or_else(|| map.clone().map(|m| (m, None)))
        }
        Goal::Collect { item, .. } => found(&|k, _| match k {
            ObjectKind::Collectible { item: i } => i == item,
            ObjectKind::Chest { contents, .. } => contents.iter().any(|(i, _)| i == item),
            _ => false,
        }),
        Goal::Reach { map, zone } => match zone {
            Some(z) => maps
                .iter()
                .find(|(n, _)| n == map)
                .and_then(|(_, m)| m.adventure.object(z))
                .map(|o| {
                    (
                        map.clone(),
                        Some(o.pos + o.kind.area().unwrap_or(Vec2::ZERO) * 0.5),
                    )
                })
                .or_else(|| Some((map.clone(), None))),
            None => Some((map.clone(), None)),
        },
        Goal::Flag { flag, .. } => {
            found(&|k, _| matches!(k, ObjectKind::Switch { flag: f, .. } if f == flag))
        }
        Goal::Manual => None,
    };
    let place = step.map.clone().map(|m| (m, None)).or(place);
    Some(QuestGoal {
        quest: quest.id.clone(),
        text: step.text.clone(),
        map: place.as_ref().map(|(m, _)| m.clone()),
        pos: place.and_then(|(_, p)| p),
    })
}

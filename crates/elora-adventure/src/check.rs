//! Checks of dialogs, quests and characters when loading (A1.4): references, conditions,
//! effects and translations (E-247). Errors name the file and location.

use std::collections::BTreeSet;

use crate::data::{Content, Text};
use crate::quest::Goal;
use crate::script::{Action, Cond, Open, QuestCheck};

/// Elora's own lines need no entry in `characters.toml`.
pub const ELORA: &str = "elora";

/// Required text: both languages filled.
fn text(t: &Text, at: &str) -> Result<(), String> {
    match (t.de.trim().is_empty(), t.en.trim().is_empty()) {
        (false, false) => Ok(()),
        (true, true) => Err(format!("{at}: text missing")),
        (true, false) => Err(format!("{at}: German text missing")),
        (false, true) => Err(format!("{at}: English text missing")),
    }
}

/// Optional text: empty or in both languages.
fn optional(t: &Text, at: &str) -> Result<(), String> {
    if t.de.trim().is_empty() && t.en.trim().is_empty() {
        Ok(())
    } else {
        text(t, at)
    }
}

fn speaker(c: &Content, who: &str, at: &str) -> Result<(), String> {
    if who == ELORA || c.characters.contains_key(who) {
        Ok(())
    } else {
        Err(format!("{at}: unknown character `{who}`"))
    }
}

fn cond(c: &Content, src: &str, at: &str) -> Result<(), String> {
    let parsed = Cond::parse(src).map_err(|e| format!("{at}: {e}"))?;
    refs_cond(c, &parsed).map_err(|e| format!("{at}: {e}"))
}

fn refs_cond(c: &Content, cond: &Cond) -> Result<(), String> {
    match cond {
        Cond::Quest(id, check) => {
            let q = c.quest(id).ok_or_else(|| format!("unknown quest `{id}`"))?;
            if let QuestCheck::Step(s) = check
                && !q.step.iter().any(|st| &st.id == s)
            {
                return Err(format!("quest `{id}` has no step `{s}`"));
            }
            Ok(())
        }
        Cond::Affection(who, ..) => speaker(c, who, "affection"),
        Cond::Has(item, _) => c
            .item(item)
            .map(|_| ())
            .ok_or_else(|| format!("unknown item `{item}`")),
        Cond::Not(inner) => refs_cond(c, inner),
        Cond::All(all) => all.iter().try_for_each(|x| refs_cond(c, x)),
        _ => Ok(()),
    }
}

fn actions(c: &Content, list: &[String], at: &str) -> Result<(), String> {
    for src in list {
        let a = Action::parse(src).map_err(|e| format!("{at}: {e}"))?;
        let bad = |m: String| Err(format!("{at}: `{src}`: {m}"));
        match &a {
            Action::Quest(id, _) if c.quest(id).is_none() => {
                return bad("unknown quest".into());
            }
            Action::Affection(who, _) if !c.characters.contains_key(who) => {
                return bad("unknown character".into());
            }
            Action::Give(item, _) | Action::Take(item, _) if c.item(item).is_none() => {
                return bad("unknown item".into());
            }
            Action::Open(Open::Shop(id)) if !c.shops.contains_key(id) => {
                return bad("unknown shop".into());
            }
            _ => {}
        }
    }
    Ok(())
}

/// Check everything.
///
/// # Errors
/// First error found, with location.
pub fn story(c: &Content) -> Result<(), String> {
    for ch in c.characters.values() {
        text(&ch.name, &format!("characters.toml `{}`", ch.id))?;
    }
    for s in c.shops.values() {
        if let Some(o) = &s.owner {
            speaker(c, o, &format!("shops.toml `{}`", s.id))?;
        }
    }
    dialogs(c)?;
    quests(c)?;
    tunes(c)?;
    temperature(c)
}

/// Tunes (`[[tune]]` in `quests.toml`): unique ids, a map of an area, notes that exist and fit
/// into what the session remembers, valid condition and effects.
fn tunes(c: &Content) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for t in &c.tunes {
        let at = format!("quests.toml tune `{}`", t.id);
        if !ids.insert(t.id.as_str()) {
            return Err(format!("{at}: duplicated"));
        }
        if c.area_of(&t.map).is_none() {
            return Err(format!("{at}: map `{}` is of no area", t.map));
        }
        if t.notes.is_empty() {
            return Err(format!("{at}: no notes"));
        }
        if t.notes.len() > crate::session::NOTES_KEPT {
            return Err(format!(
                "{at}: longer than {} notes",
                crate::session::NOTES_KEPT
            ));
        }
        if let Some(n) = t
            .notes
            .iter()
            .find(|n| elora_sim::notes::pitch_of(n).is_none())
        {
            return Err(format!(
                "{at}: unknown note `{n}` (`c d e f g a b`, `2` for the octave above)"
            ));
        }
        if let Some(src) = &t.cond {
            cond(c, src, &at)?;
        }
        actions(c, &t.actions, &at)?;
    }
    Ok(())
}

/// Heat and cold bar (`[temperature]` in `worldmap.toml`): times, factors, zone prefixes and
/// weather keys; an area is hot or cold, not both.
/// Map sections of the world map (R2-WM): a path inside 0..1, maps of the own area, each map
/// once.
fn sections(c: &Content) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    for a in &c.areas {
        for s in &a.sections {
            let at = format!("worldmap.toml area `{}`, section `{}`", a.id, s.map);
            if !s.map.starts_with(&a.maps) {
                return Err(format!("{at}: map does not belong to the area"));
            }
            if s.path.is_empty() || s.path.iter().flatten().any(|v| !(0.0..=1.0).contains(v)) {
                return Err(format!("{at}: path empty or outside 0..1"));
            }
            if !seen.insert(s.map.as_str()) {
                return Err(format!("{at}: map has two sections"));
            }
        }
    }
    Ok(())
}

fn temperature(c: &Content) -> Result<(), String> {
    if let Some(a) = c.areas.iter().find(|a| {
        std::iter::once(&a.darkness)
            .chain(a.map_darkness.values())
            .any(|d| !(0.0..=1.0).contains(d))
    }) {
        return Err(format!(
            "worldmap.toml area `{}`: darkness must be 0 to 1",
            a.id
        ));
    }
    sections(c)?;
    let t = &c.temperature;
    for (name, bar) in [("heat", &t.heat), ("cold", &t.cold)] {
        let at = format!("worldmap.toml [temperature.{name}]");
        if bar.fill_ms == 0 || bar.shelter_ms == 0 || bar.relief_ms == 0 {
            return Err(format!(
                "{at}: fill_ms, shelter_ms and relief_ms must be above 0"
            ));
        }
        if !(bar.faster.is_finite() && bar.faster > 0.0) {
            return Err(format!("{at}: faster must be above 0"));
        }
        if !(bar.recover > 0.0 && bar.recover <= 1.0) {
            return Err(format!("{at}: recover must lie in (0, 1]"));
        }
        if bar.relief_zones.is_empty() {
            return Err(format!("{at}: relief_zones missing"));
        }
        if let Some(z) = bar
            .relief_zones
            .iter()
            .chain(&bar.shelter_zones)
            .find(|z| z.trim().is_empty())
        {
            return Err(format!("{at}: empty zone prefix `{z}`"));
        }
        if let Some(w) = bar
            .faster_weather
            .iter()
            .chain(&bar.shelter_weather)
            .find(|w| elora_map::WeatherKind::from_key(w).is_none())
        {
            return Err(format!("{at}: unknown weather `{w}`"));
        }
    }
    if let Some(a) = c.areas.iter().find(|a| a.hot && a.cold) {
        return Err(format!("worldmap.toml area `{}`: both hot and cold", a.id));
    }
    Ok(())
}

fn dialogs(c: &Content) -> Result<(), String> {
    for d in c.dialogs.values() {
        let file = format!("dialogs/{}.toml", d.id);
        speaker(c, &d.speaker, &file)?;
        let mut ids = BTreeSet::new();
        for n in &d.node {
            if !ids.insert(n.id.as_str()) {
                return Err(format!("{file}: node `{}` duplicated", n.id));
            }
        }
        let exists = |id: &str, at: &str| {
            if ids.contains(id) {
                Ok(())
            } else {
                Err(format!("{at}: node `{id}` does not exist"))
            }
        };
        if d.start.is_empty() {
            return Err(format!("{file}: no entry point"));
        }
        let mut reached: BTreeSet<&str> = BTreeSet::new();
        for (i, s) in d.start.iter().enumerate() {
            let at = format!("{file} entry {}", i + 1);
            exists(&s.node, &at)?;
            reached.insert(&s.node);
            if let Some(x) = &s.cond {
                cond(c, x, &at)?;
            }
        }
        for n in &d.node {
            let at = format!("{file} node `{}`", n.id);
            text(&n.text, &at)?;
            speaker(c, d.speaker_of(n), &at)?;
            actions(c, &n.actions, &at)?;
            if let Some(next) = &n.next {
                exists(next, &at)?;
                reached.insert(next);
            }
            for (k, ch) in n.choice.iter().enumerate() {
                let at = format!("{at} choice {}", k + 1);
                text(&ch.text, &at)?;
                actions(c, &ch.actions, &at)?;
                if let Some(x) = &ch.cond {
                    cond(c, x, &at)?;
                }
                if let Some(next) = &ch.next {
                    exists(next, &at)?;
                    reached.insert(next);
                }
            }
        }
        if let Some(n) = d.node.iter().find(|n| !reached.contains(n.id.as_str())) {
            return Err(format!("{file}: node `{}` is never reachable", n.id));
        }
        for (i, b) in d.bark.iter().enumerate() {
            let at = format!("{file} bark {}", i + 1);
            text(&b.text, &at)?;
            if let Some(x) = &b.cond {
                cond(c, x, &at)?;
            }
        }
    }
    Ok(())
}

fn quests(c: &Content) -> Result<(), String> {
    let mut quest_ids = BTreeSet::new();
    for q in &c.quests {
        let at = format!("quests.toml `{}`", q.id);
        if !quest_ids.insert(q.id.as_str()) {
            return Err(format!("{at}: duplicated"));
        }
        text(&q.name, &at)?;
        optional(&q.desc, &at)?;
        if let Some(g) = &q.giver {
            speaker(c, g, &at)?;
        }
        if q.step.is_empty() {
            return Err(format!("{at}: no steps"));
        }
        let mut steps = BTreeSet::new();
        for s in &q.step {
            let at = format!("{at} step `{}`", s.id);
            if !steps.insert(s.id.as_str()) {
                return Err(format!("{at}: duplicated"));
            }
            text(&s.text, &at)?;
            match &s.goal {
                Goal::Talk { who } => speaker(c, who, &at)?,
                Goal::Defeat { kind, count, .. } => {
                    if !c.creatures.iter().any(|k| &k.name == kind) {
                        return Err(format!("{at}: unknown enemy kind `{kind}`"));
                    }
                    if *count == 0 {
                        return Err(format!("{at}: count 0"));
                    }
                }
                Goal::Collect { item, .. } | Goal::Bring { item, .. } if c.item(item).is_none() => {
                    return Err(format!("{at}: unknown item `{item}`"));
                }
                Goal::Bring { to, .. } => speaker(c, to, &at)?,
                _ => {}
            }
        }
        for it in &q.reward.items {
            if c.item(&it.item).is_none() {
                return Err(format!("{at} reward: unknown item `{}`", it.item));
            }
        }
        if let Some(n) = &q.next
            && c.quest(n).is_none()
        {
            return Err(format!("{at}: next quest `{n}` does not exist"));
        }
        if let Some(f) = &q.fail_if {
            cond(c, f, &format!("{at} fail_if"))?;
        }
    }
    for (id, ch) in &c.characters {
        if let Some(s) = &ch.show_if {
            cond(c, s, &format!("characters.toml `{id}` show_if"))?;
        }
        if let Some(s) = &ch.follow_if {
            cond(c, s, &format!("characters.toml `{id}` follow_if"))?;
        }
        if let Some(k) = &ch.follower
            && !c.creatures.iter().any(|x| &x.name == k)
        {
            return Err(format!(
                "characters.toml `{id}`: follower `{k}` does not exist"
            ));
        }
    }
    Ok(())
}

/// Check a map's adventure objects against the content: enemy kinds, characters,
/// dialogs, items, conditions. Returns all errors (for the editor, A1.8).
pub fn map_objects(c: &Content, map: &elora_map::Map) -> Vec<String> {
    use elora_map::ObjectKind as K;
    let mut errors = Vec::new();
    for o in &map.adventure.objects {
        let at = format!("map `{}`, object `{}`", map.name, o.id);
        let mut push = |r: Result<(), String>| {
            if let Err(e) = r {
                errors.push(e);
            }
        };
        let item = |id: &str| {
            c.item(id)
                .map(|_| ())
                .ok_or_else(|| format!("{at}: unknown item `{id}`"))
        };
        match &o.kind {
            K::Creature { kind, .. } => {
                if !c.creatures.iter().any(|k| &k.name == kind) {
                    push(Err(format!("{at}: unknown enemy kind `{kind}`")));
                }
            }
            K::Npc {
                character, dialog, ..
            } => {
                push(speaker(c, character, &at));
                if c.dialog(dialog).is_none() {
                    push(Err(format!("{at}: unknown dialog `{dialog}`")));
                }
            }
            K::Chest { contents, lock } => {
                for (i, n) in contents {
                    push(item(i));
                    if *n == 0 {
                        push(Err(format!("{at}: count 0")));
                    }
                }
                if !lock.is_empty() {
                    push(cond(c, lock, &at));
                }
            }
            K::Switch { flag, .. } if flag.is_empty() => {
                push(Err(format!("{at}: flag missing")));
            }
            K::Door { open_if, .. } => push(cond(c, open_if, &at)),
            K::Collectible { item: i } => push(item(i)),
            K::HealPlant { heal } if *heal <= 0 => push(Err(format!("{at}: does not heal"))),
            // note stones (M2.5.3): the pitch comes from the id
            K::Zone { .. }
                if o.id.starts_with(elora_map::islands::NOTE)
                    && !o.id.contains(elora_map::islands::BRIDGE)
                    && elora_sim::notes::pitch(&o.id).is_none() =>
            {
                push(Err(format!(
                    "{at}: note stone without pitch (`note-<c d e f g a b>[2]…`)"
                )));
            }
            _ => {}
        }
    }
    errors.extend(climax_zones(map));
    errors.extend(tune_stones(c, map));
    errors
}

/// Tunes of the map (M2.5.6): every note needs a note stone with its pitch.
fn tune_stones(c: &Content, map: &elora_map::Map) -> Vec<String> {
    let (stones, _) = elora_map::islands::map_notes(map);
    c.tunes
        .iter()
        .filter(|t| t.map == map.name)
        .flat_map(|t| {
            let missing: BTreeSet<&str> = t
                .notes
                .iter()
                .filter(|n| {
                    elora_sim::notes::pitch_of(n)
                        .is_some_and(|p| !stones.iter().any(|s| s.note == p))
                })
                .map(String::as_str)
                .collect();
            missing.into_iter().map(move |n| {
                format!(
                    "map `{}`: tune `{}` needs a note stone `note-{n}…` (`{n}`)",
                    t.map, t.id
                )
            })
        })
        .collect()
}

/// Climax “Protect Luma” (M2.5.5): a map with `climax-start` needs Luma's place, at least one
/// shadow spawn zone and the Withered One's first place.
fn climax_zones(map: &elora_map::Map) -> Vec<String> {
    use crate::session::climax as z;
    let has = |id: &str| {
        map.adventure
            .objects
            .iter()
            .any(|o| o.id.starts_with(id) && matches!(o.kind, elora_map::ObjectKind::Zone { .. }))
    };
    if !has(z::START_ZONE) {
        return Vec::new();
    }
    [
        (z::LUMA_ZONE, "Luma's place"),
        (z::SPAWN_ZONE, "a zone for the grey shadows"),
        ("withered-1", "the Withered One's first place"),
    ]
    .into_iter()
    .filter(|(id, _)| !has(id))
    .map(|(id, what)| format!("map `{}`: climax without {what} (zone `{id}…`)", map.name))
    .collect()
}

/// Check transitions between maps: target map and target entrance must exist.
pub fn map_links(maps: &[(&str, &elora_map::Map)]) -> Vec<String> {
    let mut errors = Vec::new();
    for (name, m) in maps {
        for o in &m.adventure.objects {
            if let elora_map::ObjectKind::Exit { map, spawn, .. } = &o.kind {
                match maps.iter().find(|(n, _)| n == map) {
                    None => errors.push(format!(
                        "map `{name}`, exit `{}`: target map `{map}` missing",
                        o.id
                    )),
                    Some((_, target)) => {
                        let ok = target.adventure.objects.iter().any(|t| {
                            &t.id == spawn && matches!(t.kind, elora_map::ObjectKind::Spawn)
                        });
                        if !ok {
                            errors.push(format!(
                                "map `{name}`, exit `{}`: entrance `{spawn}` missing on `{map}`",
                                o.id
                            ));
                        }
                    }
                }
            }
        }
    }
    errors
}

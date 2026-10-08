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
        (true, true) => Err(format!("{at}: Text fehlt")),
        (true, false) => Err(format!("{at}: deutscher Text fehlt")),
        (false, true) => Err(format!("{at}: englischer Text fehlt")),
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
        Err(format!("{at}: unbekannte Figur `{who}`"))
    }
}

fn cond(c: &Content, src: &str, at: &str) -> Result<(), String> {
    let parsed = Cond::parse(src).map_err(|e| format!("{at}: {e}"))?;
    refs_cond(c, &parsed).map_err(|e| format!("{at}: {e}"))
}

fn refs_cond(c: &Content, cond: &Cond) -> Result<(), String> {
    match cond {
        Cond::Quest(id, check) => {
            let q = c
                .quest(id)
                .ok_or_else(|| format!("unbekannte Aufgabe `{id}`"))?;
            if let QuestCheck::Step(s) = check
                && !q.step.iter().any(|st| &st.id == s)
            {
                return Err(format!("Aufgabe `{id}` hat keinen Schritt `{s}`"));
            }
            Ok(())
        }
        Cond::Affection(who, ..) => speaker(c, who, "Zuneigung"),
        Cond::Has(item, _) => c
            .item(item)
            .map(|_| ())
            .ok_or_else(|| format!("unbekannter Gegenstand `{item}`")),
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
                return bad("unbekannte Aufgabe".into());
            }
            Action::Affection(who, _) if !c.characters.contains_key(who) => {
                return bad("unbekannte Figur".into());
            }
            Action::Give(item, _) | Action::Take(item, _) if c.item(item).is_none() => {
                return bad("unbekannter Gegenstand".into());
            }
            Action::Open(Open::Shop(id)) if !c.shops.contains_key(id) => {
                return bad("unbekannter Laden".into());
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
    quests(c)
}

fn dialogs(c: &Content) -> Result<(), String> {
    for d in c.dialogs.values() {
        let file = format!("dialogs/{}.toml", d.id);
        speaker(c, &d.speaker, &file)?;
        let mut ids = BTreeSet::new();
        for n in &d.node {
            if !ids.insert(n.id.as_str()) {
                return Err(format!("{file}: Knoten `{}` doppelt", n.id));
            }
        }
        let exists = |id: &str, at: &str| {
            if ids.contains(id) {
                Ok(())
            } else {
                Err(format!("{at}: Knoten `{id}` gibt es nicht"))
            }
        };
        if d.start.is_empty() {
            return Err(format!("{file}: kein Einstieg"));
        }
        let mut reached: BTreeSet<&str> = BTreeSet::new();
        for (i, s) in d.start.iter().enumerate() {
            let at = format!("{file} Einstieg {}", i + 1);
            exists(&s.node, &at)?;
            reached.insert(&s.node);
            if let Some(x) = &s.cond {
                cond(c, x, &at)?;
            }
        }
        for n in &d.node {
            let at = format!("{file} Knoten `{}`", n.id);
            text(&n.text, &at)?;
            speaker(c, d.speaker_of(n), &at)?;
            actions(c, &n.actions, &at)?;
            if let Some(next) = &n.next {
                exists(next, &at)?;
                reached.insert(next);
            }
            for (k, ch) in n.choice.iter().enumerate() {
                let at = format!("{at} Antwort {}", k + 1);
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
            return Err(format!("{file}: Knoten `{}` ist nie erreichbar", n.id));
        }
        for (i, b) in d.bark.iter().enumerate() {
            let at = format!("{file} Zuruf {}", i + 1);
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
            return Err(format!("{at}: doppelt"));
        }
        text(&q.name, &at)?;
        optional(&q.desc, &at)?;
        if let Some(g) = &q.giver {
            speaker(c, g, &at)?;
        }
        if q.step.is_empty() {
            return Err(format!("{at}: keine Schritte"));
        }
        let mut steps = BTreeSet::new();
        for s in &q.step {
            let at = format!("{at} Schritt `{}`", s.id);
            if !steps.insert(s.id.as_str()) {
                return Err(format!("{at}: doppelt"));
            }
            text(&s.text, &at)?;
            match &s.goal {
                Goal::Talk { who } => speaker(c, who, &at)?,
                Goal::Defeat { kind, count, .. } => {
                    if !c.creatures.iter().any(|k| &k.name == kind) {
                        return Err(format!("{at}: unbekannte Gegnerart `{kind}`"));
                    }
                    if *count == 0 {
                        return Err(format!("{at}: Anzahl 0"));
                    }
                }
                Goal::Collect { item, .. } | Goal::Bring { item, .. } if c.item(item).is_none() => {
                    return Err(format!("{at}: unbekannter Gegenstand `{item}`"));
                }
                Goal::Bring { to, .. } => speaker(c, to, &at)?,
                _ => {}
            }
        }
        for it in &q.reward.items {
            if c.item(&it.item).is_none() {
                return Err(format!(
                    "{at} Belohnung: unbekannter Gegenstand `{}`",
                    it.item
                ));
            }
        }
        if let Some(n) = &q.next
            && c.quest(n).is_none()
        {
            return Err(format!("{at}: nächste Aufgabe `{n}` gibt es nicht"));
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
                "characters.toml `{id}`: Begleiter `{k}` gibt es nicht"
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
        let at = format!("Karte `{}`, Objekt `{}`", map.name, o.id);
        let mut push = |r: Result<(), String>| {
            if let Err(e) = r {
                errors.push(e);
            }
        };
        let item = |id: &str| {
            c.item(id)
                .map(|_| ())
                .ok_or_else(|| format!("{at}: unbekannter Gegenstand `{id}`"))
        };
        match &o.kind {
            K::Creature { kind, .. } => {
                if !c.creatures.iter().any(|k| &k.name == kind) {
                    push(Err(format!("{at}: unbekannte Gegnerart `{kind}`")));
                }
            }
            K::Npc {
                character, dialog, ..
            } => {
                push(speaker(c, character, &at));
                if c.dialog(dialog).is_none() {
                    push(Err(format!("{at}: unbekanntes Gespräch `{dialog}`")));
                }
            }
            K::Chest { contents, lock } => {
                for (i, n) in contents {
                    push(item(i));
                    if *n == 0 {
                        push(Err(format!("{at}: Anzahl 0")));
                    }
                }
                if !lock.is_empty() {
                    push(cond(c, lock, &at));
                }
            }
            K::Switch { flag, .. } if flag.is_empty() => {
                push(Err(format!("{at}: Merker fehlt")));
            }
            K::Door { open_if, .. } => push(cond(c, open_if, &at)),
            K::Collectible { item: i } => push(item(i)),
            K::HealPlant { heal } if *heal <= 0 => push(Err(format!("{at}: heilt nicht"))),
            _ => {}
        }
    }
    errors
}

/// Check transitions between maps: target map and target entrance must exist.
pub fn map_links(maps: &[(&str, &elora_map::Map)]) -> Vec<String> {
    let mut errors = Vec::new();
    for (name, m) in maps {
        for o in &m.adventure.objects {
            if let elora_map::ObjectKind::Exit { map, spawn, .. } = &o.kind {
                match maps.iter().find(|(n, _)| n == map) {
                    None => errors.push(format!(
                        "Karte `{name}`, Übergang `{}`: Zielkarte `{map}` fehlt",
                        o.id
                    )),
                    Some((_, target)) => {
                        let ok = target.adventure.objects.iter().any(|t| {
                            &t.id == spawn && matches!(t.kind, elora_map::ObjectKind::Spawn)
                        });
                        if !ok {
                            errors.push(format!(
                                "Karte `{name}`, Übergang `{}`: Eingang `{spawn}` fehlt auf `{map}`",
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

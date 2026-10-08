//! German → English ids (R2-RF, RF-10 to RF-13, E-358).
//!
//! One dictionary, `assets/renames.toml`, translates ids word by word: an id is split at `-`,
//! `_`, `.` and `:` and a trailing number (`fest2` → `fest` + `2`); every word in `[words]` is
//! replaced, all others stay (proper names, words that are English already). A translation
//! with `_` takes the separator of the id it lands in: `stachelkaefer-1` → `spike-beetle-1`,
//! `munition_granate` → `ammo_grenade`. Composite keys translate like their parts:
//! `truhe:wiese-1:hof-truhe` → `chest:meadow-1:yard-chest`.
//!
//! The same table converts saves and maps from 0.9.x when they are loaded, and the condition
//! language (`[keywords]`) still accepts the German words for one release (D-RF-02).

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;

const SOURCE: &str = include_str!("../../../assets/renames.toml");

struct Table {
    words: HashMap<String, String>,
    keywords: HashMap<String, String>,
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        let raw: HashMap<String, HashMap<String, String>> =
            toml::from_str(SOURCE).expect("assets/renames.toml is valid");
        let take = |k: &str| raw.get(k).cloned().unwrap_or_default();
        Table {
            words: take("words"),
            keywords: take("keywords"),
        }
    })
}

fn is_sep(c: char) -> bool {
    matches!(c, '-' | '_' | '.' | ':')
}

/// Translates one id; unchanged ids are borrowed.
pub fn translate_id(id: &str) -> Cow<'_, str> {
    let words = &table().words;
    // pieces alternate: word, separator, word, …
    let mut pieces: Vec<&str> = Vec::new();
    let mut start = 0;
    for (i, c) in id.char_indices() {
        if is_sep(c) {
            pieces.push(&id[start..i]);
            pieces.push(&id[i..=i]);
            start = i + 1;
        }
    }
    pieces.push(&id[start..]);
    let mut changed = false;
    let mut out = String::with_capacity(id.len() + 8);
    for (i, piece) in pieces.iter().enumerate() {
        if i % 2 == 1 {
            out.push_str(piece);
            continue;
        }
        let letters = piece.trim_end_matches(|c: char| c.is_ascii_digit());
        let Some(word) = words.get(letters) else {
            out.push_str(piece);
            continue;
        };
        changed = true;
        let near_dash = (i > 0 && pieces[i - 1] == "-") || pieces.get(i + 1) == Some(&"-");
        if near_dash {
            out.push_str(&word.replace('_', "-"));
        } else {
            out.push_str(word);
        }
        out.push_str(&piece[letters.len()..]);
    }
    if changed {
        Cow::Owned(out)
    } else {
        Cow::Borrowed(id)
    }
}

/// The English keyword of a German one of the condition language (`merker` → `flag`).
pub fn translate_keyword(word: &str) -> Option<&'static str> {
    table().keywords.get(word).map(String::as_str)
}

/// Translates a condition or effect (`nicht merker oma.frech und quest brunnen schritt
/// kaefer` → `not flag oma.cheeky and quest well step beetle`): keywords where the language
/// has them, ids everywhere else. English input stays the same.
pub fn translate_script(src: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    // position within the current clause (after `und` / `and` a new clause starts)
    let mut pos = 0;
    let mut quest = false;
    for token in src.split(' ') {
        let kw = translate_keyword(token);
        let keyword_here = match pos {
            0 => true,
            // `quest <id> <status>` / `quest <id> schritt <id>`
            2 => quest,
            _ => false,
        };
        let word = match kw {
            Some(k) if keyword_here || matches!(k, "and" | "not") => Cow::Borrowed(k),
            _ if token.is_empty() => Cow::Borrowed(token),
            _ => translate_id(token),
        };
        match word.as_ref() {
            "and" => {
                pos = 0;
                quest = false;
            }
            "not" if pos == 0 => {}
            w => {
                if pos == 0 {
                    quest = w == "quest";
                }
                pos += 1;
            }
        }
        out.push(word.into_owned());
    }
    out.join(" ")
}

/// Converts a map of format version 1 (0.9.x, German ids) in place: decoration, background
/// and material names and every id and condition of the adventure objects (RF-13).
pub fn migrate_map(map: &mut crate::Map) {
    use crate::adventure::ObjectKind;
    use crate::look::Art;
    fn id(s: &mut String) {
        if let Cow::Owned(new) = translate_id(s) {
            *s = new;
        }
    }
    fn script(s: &mut String) {
        if !s.is_empty() {
            *s = translate_script(s);
        }
    }
    let decor = map
        .decor_back
        .iter_mut()
        .chain(map.decor_front.iter_mut())
        .chain(map.backgrounds.iter_mut().flat_map(|b| b.items.iter_mut()));
    for d in decor {
        if let Art::Builtin(name) = &mut d.art {
            id(name);
        }
    }
    map.materials.iter_mut().for_each(id);
    for o in &mut map.adventure.objects {
        id(&mut o.id);
        match &mut o.kind {
            ObjectKind::Creature { kind, .. } => id(kind),
            ObjectKind::Npc {
                character, dialog, ..
            } => {
                id(character);
                id(dialog);
            }
            ObjectKind::Chest { contents, lock } => {
                for (item, _) in contents.iter_mut() {
                    id(item);
                }
                script(lock);
            }
            ObjectKind::Switch { flag, .. } => id(flag),
            ObjectKind::Door { open_if, .. } => script(open_if),
            ObjectKind::Collectible { item } => id(item),
            ObjectKind::Exit { map, spawn, .. } => {
                id(map);
                id(spawn);
            }
            ObjectKind::SavePoint
            | ObjectKind::HealPlant { .. }
            | ObjectKind::Spawn
            | ObjectKind::Zone { .. }
            | ObjectKind::Camera { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_word_by_word() {
        for (old, new) in [
            ("stachelkaefer-1", "spike-beetle-1"),
            ("munition_granate", "ammo_grenade"),
            ("glanztropfen", "gleam_drops"),
            ("besiegt.brummbaer", "defeated.bumblebear"),
            ("fest2", "party2"),
            ("wiese-arena", "meadow-arena"),
            ("dm-wiese", "dm-meadow"),
            ("truhe:wiese-1:hof-truhe", "chest:meadow-1:yard-chest"),
            ("fund:wiese-1:glitzerstein", "find:meadow-1:glitter_stone"),
            ("bluetenquelle-verdorrt", "blossom-spring-withered"),
            ("eisgriff.stark", "ice_grip.strong"),
            ("schoen", "clear"),
        ] {
            assert_eq!(translate_id(old), new, "{old}");
        }
    }

    #[test]
    fn english_ids_and_proper_names_stay() {
        for id in [
            "klonk",
            "tauwinkel",
            "frost-1",
            "rock-2",
            "spike-beetle-1",
            "",
            "x",
        ] {
            assert!(matches!(translate_id(id), Cow::Borrowed(_)), "{id}");
        }
    }

    #[test]
    fn conditions_and_effects() {
        for (old, new) in [
            (
                "nicht merker oma.frech und quest brunnen schritt kaefer",
                "not flag oma.cheeky and quest well step beetle",
            ),
            ("quest glutsand neu", "quest glutsand new"),
            ("faehigkeit hook-ruck", "ability hook-jerk"),
            ("waffe hammer", "weapon hammer"),
            ("gib heiltrank 2", "give healing_potion 2"),
            ("merker tor >= 2", "flag gate >= 2"),
            ("zuneigung oma +1", "affection oma +1"),
            ("stufe >= 3 und glanz < 50", "level >= 3 and gleam < 50"),
            ("laden lotte", "shop lotte"),
            ("baum", "skills"),
            ("quest brunnen start", "quest well start"),
        ] {
            assert_eq!(translate_script(old), new, "{old}");
        }
        // English stays English
        let english = "not flag oma.cheeky and quest well step beetle";
        assert_eq!(translate_script(english), english);
    }
}

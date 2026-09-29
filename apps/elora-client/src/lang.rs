//! Übersetzungen der Oberfläche (M7.2, E-114): Deutsch und Englisch, umschaltbar.
//!
//! Texte stehen in `assets/lang/<code>.toml`, gegliedert in Abschnitte
//! (`[hud] dead = "…"` → Schlüssel `hud.dead`). Platzhalter heißen `{name}` und
//! werden mit [`Lang::f`] ersetzt. Fehlt ein Text, gilt der deutsche, sonst der Schlüssel.

use std::collections::HashMap;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    De,
    En,
}

impl Language {
    pub const ALL: [Self; 2] = [Self::De, Self::En];

    /// Name in der eigenen Sprache (für die Auswahl).
    pub fn name(self) -> &'static str {
        match self {
            Self::De => "Deutsch",
            Self::En => "English",
        }
    }

    fn source(self) -> &'static str {
        match self {
            Self::De => include_str!("../../../assets/lang/de.toml"),
            Self::En => include_str!("../../../assets/lang/en.toml"),
        }
    }

    /// Startwert ohne Einstellungsdatei: Deutsch, wenn die Systemsprache Deutsch ist.
    pub fn from_env() -> Self {
        let lang = ["LC_ALL", "LC_MESSAGES", "LANG"]
            .iter()
            .find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
            .unwrap_or_default();
        if lang.starts_with("en") {
            Self::En
        } else {
            Self::De
        }
    }
}

fn flatten(prefix: &str, table: &toml::Table, out: &mut HashMap<String, String>) {
    for (k, v) in table {
        let key = if prefix.is_empty() {
            k.clone()
        } else {
            format!("{prefix}.{k}")
        };
        match v {
            toml::Value::Table(t) => flatten(&key, t, out),
            toml::Value::String(s) => {
                out.insert(key, s.clone());
            }
            _ => {}
        }
    }
}

fn parse(src: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    if let Ok(table) = src.parse::<toml::Table>() {
        flatten("", &table, &mut out);
    }
    out
}

#[derive(Debug)]
pub struct Lang {
    #[allow(dead_code)] // Sprachauswahl in den Einstellungen (M7.4)
    pub language: Language,
    texts: HashMap<String, String>,
    fallback: HashMap<String, String>,
}

impl Lang {
    pub fn new(language: Language) -> Self {
        Self {
            language,
            texts: parse(language.source()),
            fallback: parse(Language::De.source()),
        }
    }

    /// Text zu `key`.
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.texts
            .get(key)
            .or_else(|| self.fallback.get(key))
            .map_or(key, String::as_str)
    }

    /// Text mit ersetzten Platzhaltern, z. B. `f("hud.red", &[("n", &3)])`.
    pub fn f(&self, key: &str, args: &[(&str, &dyn Display)]) -> String {
        let mut s = self.t(key).to_owned();
        for (name, value) in args {
            s = s.replace(&format!("{{{name}}}"), &value.to_string());
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placeholders(s: &str) -> Vec<&str> {
        let mut v: Vec<&str> = s
            .split('{')
            .skip(1)
            .filter_map(|p| p.split_once('}').map(|(n, _)| n))
            .collect();
        v.sort_unstable();
        v
    }

    #[test]
    fn languages_have_same_keys_and_placeholders() {
        let de = parse(Language::De.source());
        let en = parse(Language::En.source());
        assert!(de.len() > 20);
        let mut dk: Vec<&String> = de.keys().collect();
        let mut ek: Vec<&String> = en.keys().collect();
        dk.sort();
        ek.sort();
        assert_eq!(dk, ek, "gleiche Schlüssel in de.toml und en.toml");
        for (k, v) in &de {
            assert_eq!(
                placeholders(v),
                placeholders(&en[k]),
                "Platzhalter von `{k}`"
            );
        }
    }

    #[test]
    fn lookup_format_and_fallback() {
        let en = Lang::new(Language::En);
        assert_eq!(en.t("hud.round_over"), "Round over");
        assert_eq!(en.f("hud.red", &[("n", &3)]), "Red 3");
        assert_eq!(en.t("gibt.es.nicht"), "gibt.es.nicht");
        let de = Lang::new(Language::De);
        assert_eq!(
            de.f(
                "hud.goal",
                &[("n", &20), ("unit", &de.t("hud.unit_points"))]
            ),
            "Ziel: 20 Punkte"
        );
    }
}

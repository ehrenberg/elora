//! UI translations (M7.2, E-114): German and English, switchable.
//!
//! Texts live in `assets/lang/<code>.toml`, organized in sections
//! (`[hud] dead = "…"` → key `hud.dead`). Placeholders are called `{name}` and
//! are replaced with [`Lang::f`]. If a text is missing, the German one applies, otherwise the key.

use std::collections::HashMap;
use std::fmt::Display;

use elora_protocol::{Message, VoteSubject, WinnerName};
use elora_sim::Team;
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

    /// Name in its own language (for the selection).
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

    /// Initial value without a settings file: German if the system language is German.
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
    #[allow(dead_code)] // language selection in the settings (M7.4)
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

    /// Text for `key`.
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.texts
            .get(key)
            .or_else(|| self.fallback.get(key))
            .map_or(key, String::as_str)
    }

    /// Text with replaced placeholders, e.g. `f("hud.red", &[("n", &3)])`.
    pub fn f(&self, key: &str, args: &[(&str, &dyn Display)]) -> String {
        let mut s = self.t(key).to_owned();
        for (name, value) in args {
            s = s.replace(&format!("{{{name}}}"), &value.to_string());
        }
        s
    }
}

/// Translate server messages (M8.1, E-164).
impl Lang {
    fn team(&self, t: Team) -> &str {
        self.t(match t {
            Team::Red => "msg.team_red",
            Team::Blue => "msg.team_blue",
            Team::Spectator => "msg.team_spectators",
            Team::None => "msg.team_none",
        })
    }

    fn winner(&self, w: &WinnerName) -> String {
        match w {
            WinnerName::Player(p) => p.clone(),
            WinnerName::Team(Team::Red) => self.t("msg.winner_red").to_owned(),
            WinnerName::Team(Team::Blue) => self.t("msg.winner_blue").to_owned(),
            WinnerName::Team(_) | WinnerName::Nobody => self.t("msg.nobody").to_owned(),
        }
    }

    /// Subject of a vote.
    pub fn vote_subject(&self, s: &VoteSubject) -> String {
        match s {
            VoteSubject::Map(m) => self.f("vote.map", &[("map", m)]),
            VoteSubject::Mode { mode, instagib } => self.f(
                "vote.mode",
                &[("mode", &VoteSubject::mode_label(*mode, *instagib))],
            ),
            VoteSubject::Kick(n) => self.f("vote.kick", &[("name", n)]),
            VoteSubject::Spectate(n) => self.f("vote.spectate", &[("name", n)]),
        }
    }

    /// Server message in this language.
    pub fn message(&self, m: &Message) -> String {
        match m {
            Message::Text(t) => t.clone(),
            Message::Joined { name } => self.f("msg.joined", &[("name", name)]),
            Message::Left { name } => self.f("msg.left", &[("name", name)]),
            Message::TeamJoined { name, team } => self.f(
                "msg.team_joined",
                &[("name", name), ("team", &self.team(*team))],
            ),
            Message::TeamBalanced { name, team } => self.f(
                "msg.team_balanced",
                &[("name", name), ("team", &self.team(*team))],
            ),
            Message::MatchStarted { mode } => self.f("msg.match_started", &[("mode", mode)]),
            Message::RoundWon(w) => self.f("msg.round_won", &[("winner", &self.winner(w))]),
            Message::RoundDraw => self.t("msg.round_draw").to_owned(),
            Message::MatchWon(w) => self.f("msg.match_won", &[("winner", &self.winner(w))]),
            Message::SuddenDeath => self.t("msg.sudden_death").to_owned(),
            Message::MapChanged { map } => self.f("msg.map_changed", &[("map", map)]),
            Message::ModeChanged { mode } => self.f("msg.mode_changed", &[("mode", mode)]),
            Message::VoteStarted { who, subject } => self.f(
                "msg.vote_started",
                &[("who", who), ("subject", &self.vote_subject(subject))],
            ),
            Message::VoteFailed(s) => {
                self.f("msg.vote_failed", &[("subject", &self.vote_subject(s))])
            }
            Message::VotePassed(s) => {
                self.f("msg.vote_passed", &[("subject", &self.vote_subject(s))])
            }
            Message::VoteCancelled => self.t("msg.vote_cancelled").to_owned(),
            Message::VotesDisabled => self.t("msg.votes_disabled").to_owned(),
            Message::VoteRunning => self.t("msg.vote_running").to_owned(),
            Message::UnknownMap { map } => self.f("msg.unknown_map", &[("map", map)]),
            Message::InvalidPlayer => self.t("msg.invalid_player").to_owned(),
            Message::MapChangeFailed { map } => self.f("msg.map_change_failed", &[("map", map)]),
        }
    }

    /// Disconnect reason: translate the code, leave free text unchanged. Client-side codes
    /// ([`elora_client::online::fail_code`]) carry a technical detail after the code.
    pub fn reason(&self, text: &str) -> String {
        if let Some(k) = elora_protocol::reason::key(text) {
            return self.t(&k).to_owned();
        }
        let (code, detail) = text.split_once(' ').unwrap_or((text, ""));
        match code.strip_prefix('#').map(|c| format!("reason.{c}")) {
            Some(key) if self.t(&key) != key => self.f(&key, &[("detail", &detail)]),
            _ => text.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_client::online::fail_code;

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
    fn server_messages_are_translated() {
        use elora_protocol::{Message, VoteSubject, WinnerName, reason};
        let samples = [
            Message::Joined { name: "A".into() },
            Message::TeamJoined {
                name: "A".into(),
                team: Team::Spectator,
            },
            Message::TeamBalanced {
                name: "A".into(),
                team: Team::Red,
            },
            Message::MatchStarted { mode: "CTF".into() },
            Message::RoundWon(WinnerName::Team(Team::Blue)),
            Message::MatchWon(WinnerName::Nobody),
            Message::VoteStarted {
                who: "A".into(),
                subject: VoteSubject::Kick("B".into()),
            },
            Message::VotePassed(VoteSubject::Mode {
                mode: elora_game::Mode::Ctf,
                instagib: true,
            }),
            Message::VoteFailed(VoteSubject::Spectate("B".into())),
            Message::UnknownMap { map: "x".into() },
            Message::MapChangeFailed { map: "x".into() },
            Message::Left { name: "A".into() },
            Message::RoundDraw,
            Message::SuddenDeath,
            Message::MapChanged { map: "x".into() },
            Message::ModeChanged { mode: "x".into() },
            Message::VoteCancelled,
            Message::VotesDisabled,
            Message::VoteRunning,
            Message::InvalidPlayer,
        ];
        for lang in Language::ALL.map(Lang::new) {
            for m in &samples {
                let text = lang.message(m);
                assert!(!text.contains('{') && !text.contains("msg."), "{text}");
            }
            for code in reason::ALL {
                let text = lang.reason(code);
                assert!(
                    !text.starts_with('#') && !text.starts_with("reason."),
                    "{text}"
                );
            }
            for code in [
                fail_code::MAP_DAMAGED,
                fail_code::MAP_INVALID,
                fail_code::MAP_MISMATCH,
            ] {
                let text = lang.reason(&format!("{code} detail"));
                assert!(!text.starts_with('#') && text.contains("detail"), "{text}");
            }
            assert_eq!(lang.reason("free text"), "free text");
            assert_eq!(lang.reason("#unknown-code x"), "#unknown-code x");
        }
        let en = Lang::new(Language::En);
        assert_eq!(
            en.message(&Message::Joined {
                name: "Nimbus".into()
            }),
            "Nimbus joined"
        );
        assert_eq!(
            en.message(&Message::RoundWon(WinnerName::Team(Team::Red))),
            "Team Red wins the round"
        );
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
        assert_eq!(dk, ek, "same keys in de.toml and en.toml");
        for (k, v) in &de {
            assert_eq!(
                placeholders(v),
                placeholders(&en[k]),
                "placeholders of `{k}`"
            );
        }
    }

    #[test]
    fn lookup_format_and_fallback() {
        let en = Lang::new(Language::En);
        assert_eq!(en.t("hud.round_over"), "Round over");
        assert_eq!(en.f("hud.red", &[("n", &3)]), "Red 3");
        assert_eq!(en.t("does.not.exist"), "does.not.exist");
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

//! Tastenbelegung (M7.5, E-117): eine Taste, Maustaste oder Mausrad-Richtung je Aktion.
//!
//! Gespeichert in `settings.toml` unter `[bindings]` als lesbare Namen, z. B.
//! `jump = "space"`, `fire = "mouse_left"`, `next_weapon = "wheel_down"`. Unbekannte
//! oder ungültige Einträge behalten die Standardbelegung. Fest belegt bleiben Esc
//! (Pause), F1 (Debug-Panel) und in der Sandbox R (Respawn) und F5 (Aufzeichnung).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use crate::lang::Lang;

/// Auslöser einer Aktion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    Key(KeyCode),
    Mouse(MouseButton),
    WheelUp,
    WheelDown,
}

/// Belegbare Aktionen im Spiel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameAction {
    Left,
    Right,
    Jump,
    Down,
    Hook,
    /// Fähigkeit (Hook-Ruck, E-226) – nur im Abenteuer und im Quellenkampf.
    Ability,
    Fire,
    Hammer,
    Grenade,
    Laser,
    NextWeapon,
    PrevWeapon,
    Chat,
    TeamChat,
    Scoreboard,
    Emote,
    Kill,
    VoteYes,
    VoteNo,
}

impl GameAction {
    pub const ALL: [Self; 19] = [
        Self::Left,
        Self::Right,
        Self::Jump,
        Self::Down,
        Self::Hook,
        Self::Ability,
        Self::Fire,
        Self::Hammer,
        Self::Grenade,
        Self::Laser,
        Self::NextWeapon,
        Self::PrevWeapon,
        Self::Chat,
        Self::TeamChat,
        Self::Scoreboard,
        Self::Emote,
        Self::Kill,
        Self::VoteYes,
        Self::VoteNo,
    ];

    /// Schlüssel in `settings.toml` und für die Übersetzung (`bind.<name>`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Jump => "jump",
            Self::Down => "down",
            Self::Hook => "hook",
            Self::Ability => "ability",
            Self::Fire => "fire",
            Self::Hammer => "hammer",
            Self::Grenade => "grenade",
            Self::Laser => "laser",
            Self::NextWeapon => "next_weapon",
            Self::PrevWeapon => "prev_weapon",
            Self::Chat => "chat",
            Self::TeamChat => "team_chat",
            Self::Scoreboard => "scoreboard",
            Self::Emote => "emote",
            Self::Kill => "kill",
            Self::VoteYes => "vote_yes",
            Self::VoteNo => "vote_no",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.name() == name)
    }

    /// Standardbelegung (E-043, E-051, E-078, E-091, E-141, E-226).
    fn default_trigger(self) -> Trigger {
        use KeyCode as K;
        match self {
            Self::Left => Trigger::Key(K::KeyA),
            Self::Right => Trigger::Key(K::KeyD),
            Self::Jump => Trigger::Key(K::Space),
            Self::Down => Trigger::Key(K::KeyS),
            Self::Hook => Trigger::Mouse(MouseButton::Right),
            Self::Ability => Trigger::Key(K::ShiftLeft),
            Self::Fire => Trigger::Mouse(MouseButton::Left),
            Self::Hammer => Trigger::Key(K::Digit1),
            Self::Grenade => Trigger::Key(K::Digit2),
            Self::Laser => Trigger::Key(K::Digit3),
            Self::NextWeapon => Trigger::WheelDown,
            Self::PrevWeapon => Trigger::WheelUp,
            Self::Chat => Trigger::Key(K::KeyT),
            Self::TeamChat => Trigger::Key(K::KeyY),
            Self::Scoreboard => Trigger::Key(K::Tab),
            Self::Emote => Trigger::Key(K::KeyE),
            Self::Kill => Trigger::Key(K::KeyK),
            Self::VoteYes => Trigger::Key(K::F3),
            Self::VoteNo => Trigger::Key(K::F4),
        }
    }
}

/// Tasten, die nicht belegt werden können (fest vergeben).
pub fn reserved(t: Trigger) -> bool {
    matches!(t, Trigger::Key(KeyCode::Escape | KeyCode::F1))
}

/// Belegbare Tasten mit Namen (Datei) und Anzeige.
const KEYS: &[(KeyCode, &str, &str)] = {
    use KeyCode as K;
    &[
        (K::KeyA, "a", "A"),
        (K::KeyB, "b", "B"),
        (K::KeyC, "c", "C"),
        (K::KeyD, "d", "D"),
        (K::KeyE, "e", "E"),
        (K::KeyF, "f", "F"),
        (K::KeyG, "g", "G"),
        (K::KeyH, "h", "H"),
        (K::KeyI, "i", "I"),
        (K::KeyJ, "j", "J"),
        (K::KeyK, "k", "K"),
        (K::KeyL, "l", "L"),
        (K::KeyM, "m", "M"),
        (K::KeyN, "n", "N"),
        (K::KeyO, "o", "O"),
        (K::KeyP, "p", "P"),
        (K::KeyQ, "q", "Q"),
        (K::KeyR, "r", "R"),
        (K::KeyS, "s", "S"),
        (K::KeyT, "t", "T"),
        (K::KeyU, "u", "U"),
        (K::KeyV, "v", "V"),
        (K::KeyW, "w", "W"),
        (K::KeyX, "x", "X"),
        (K::KeyY, "y", "Y"),
        (K::KeyZ, "z", "Z"),
        (K::Digit0, "0", "0"),
        (K::Digit1, "1", "1"),
        (K::Digit2, "2", "2"),
        (K::Digit3, "3", "3"),
        (K::Digit4, "4", "4"),
        (K::Digit5, "5", "5"),
        (K::Digit6, "6", "6"),
        (K::Digit7, "7", "7"),
        (K::Digit8, "8", "8"),
        (K::Digit9, "9", "9"),
        (K::F2, "f2", "F2"),
        (K::F3, "f3", "F3"),
        (K::F4, "f4", "F4"),
        (K::F5, "f5", "F5"),
        (K::F6, "f6", "F6"),
        (K::F7, "f7", "F7"),
        (K::F8, "f8", "F8"),
        (K::F9, "f9", "F9"),
        (K::F10, "f10", "F10"),
        (K::F11, "f11", "F11"),
        (K::F12, "f12", "F12"),
        (K::Space, "space", "keys.space"),
        (K::Tab, "tab", "Tab"),
        (K::Enter, "enter", "Enter"),
        (K::Backspace, "backspace", "keys.backspace"),
        (K::ShiftLeft, "shift_left", "keys.shift_left"),
        (K::ShiftRight, "shift_right", "keys.shift_right"),
        (K::ControlLeft, "ctrl_left", "keys.ctrl_left"),
        (K::ControlRight, "ctrl_right", "keys.ctrl_right"),
        (K::AltLeft, "alt_left", "keys.alt_left"),
        (K::AltRight, "alt_right", "keys.alt_right"),
        (K::CapsLock, "caps_lock", "keys.caps_lock"),
        (K::ArrowUp, "up", "↑"),
        (K::ArrowDown, "down", "↓"),
        (K::ArrowLeft, "left", "←"),
        (K::ArrowRight, "right", "→"),
        (K::Numpad0, "num0", "Num 0"),
        (K::Numpad1, "num1", "Num 1"),
        (K::Numpad2, "num2", "Num 2"),
        (K::Numpad3, "num3", "Num 3"),
        (K::Numpad4, "num4", "Num 4"),
        (K::Numpad5, "num5", "Num 5"),
        (K::Numpad6, "num6", "Num 6"),
        (K::Numpad7, "num7", "Num 7"),
        (K::Numpad8, "num8", "Num 8"),
        (K::Numpad9, "num9", "Num 9"),
        (K::Minus, "minus", "-"),
        (K::Equal, "equal", "="),
        (K::Comma, "comma", ","),
        (K::Period, "period", "."),
        (K::Slash, "slash", "/"),
        (K::Semicolon, "semicolon", ";"),
        (K::Quote, "quote", "'"),
        (K::BracketLeft, "bracket_left", "["),
        (K::BracketRight, "bracket_right", "]"),
        (K::Backslash, "backslash", "\\"),
        (K::Backquote, "backquote", "`"),
        (K::Insert, "insert", "Ins"),
        (K::Delete, "delete", "Del"),
        (K::Home, "home", "Home"),
        (K::End, "end", "End"),
        (K::PageUp, "page_up", "PgUp"),
        (K::PageDown, "page_down", "PgDn"),
    ]
};

impl Trigger {
    /// Name in `settings.toml`; `None` für nicht belegbare Tasten.
    pub fn name(self) -> Option<&'static str> {
        match self {
            Self::Key(k) => KEYS.iter().find(|(c, _, _)| *c == k).map(|(_, n, _)| *n),
            Self::Mouse(MouseButton::Left) => Some("mouse_left"),
            Self::Mouse(MouseButton::Right) => Some("mouse_right"),
            Self::Mouse(MouseButton::Middle) => Some("mouse_middle"),
            Self::Mouse(MouseButton::Back) => Some("mouse_back"),
            Self::Mouse(MouseButton::Forward) => Some("mouse_forward"),
            Self::Mouse(_) => None,
            Self::WheelUp => Some("wheel_up"),
            Self::WheelDown => Some("wheel_down"),
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let t = match name {
            "mouse_left" => Self::Mouse(MouseButton::Left),
            "mouse_right" => Self::Mouse(MouseButton::Right),
            "mouse_middle" => Self::Mouse(MouseButton::Middle),
            "mouse_back" => Self::Mouse(MouseButton::Back),
            "mouse_forward" => Self::Mouse(MouseButton::Forward),
            "wheel_up" => Self::WheelUp,
            "wheel_down" => Self::WheelDown,
            _ => Self::Key(KEYS.iter().find(|(_, n, _)| *n == name)?.0),
        };
        Some(t)
    }

    /// Anzeige in der Oberfläche (Maus, Mausrad und einige Tasten übersetzt).
    pub fn label(self, lang: &Lang) -> String {
        let key = match self {
            Self::Key(k) => KEYS
                .iter()
                .find(|(c, _, _)| *c == k)
                .map_or("?", |(_, _, l)| l),
            Self::Mouse(MouseButton::Left) => "keys.mouse_left",
            Self::Mouse(MouseButton::Right) => "keys.mouse_right",
            Self::Mouse(MouseButton::Middle) => "keys.mouse_middle",
            Self::Mouse(MouseButton::Back) => "keys.mouse_back",
            Self::Mouse(MouseButton::Forward) => "keys.mouse_forward",
            Self::Mouse(_) => "?",
            Self::WheelUp => "keys.wheel_up",
            Self::WheelDown => "keys.wheel_down",
        };
        lang.t(key).to_owned()
    }
}

/// Belegung aller Aktionen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "BTreeMap<String, String>", into = "BTreeMap<String, String>")]
pub struct Bindings {
    map: BTreeMap<GameAction, Trigger>,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            map: GameAction::ALL
                .into_iter()
                .map(|a| (a, a.default_trigger()))
                .collect(),
        }
    }
}

impl From<BTreeMap<String, String>> for Bindings {
    fn from(raw: BTreeMap<String, String>) -> Self {
        let mut b = Self::default();
        for (action, trigger) in raw {
            if let (Some(a), Some(t)) =
                (GameAction::from_name(&action), Trigger::from_name(&trigger))
                && !reserved(t)
            {
                b.map.insert(a, t);
            }
        }
        b
    }
}

impl From<Bindings> for BTreeMap<String, String> {
    fn from(b: Bindings) -> Self {
        b.map
            .iter()
            .filter_map(|(a, t)| Some((a.name().to_owned(), t.name()?.to_owned())))
            .collect()
    }
}

impl Bindings {
    pub fn trigger(&self, a: GameAction) -> Trigger {
        self.map
            .get(&a)
            .copied()
            .unwrap_or_else(|| a.default_trigger())
    }

    /// Aktionen, die `t` auslöst (bei doppelter Belegung mehrere).
    pub fn actions(&self, t: Trigger) -> Vec<GameAction> {
        self.map
            .iter()
            .filter(|(_, tt)| **tt == t)
            .map(|(a, _)| *a)
            .collect()
    }

    /// Neu belegen; `false` bei fest vergebenen oder nicht speicherbaren Tasten.
    pub fn set(&mut self, a: GameAction, t: Trigger) -> bool {
        if reserved(t) || t.name().is_none() {
            return false;
        }
        self.map.insert(a, t);
        true
    }

    /// Ist die Taste von `a` noch einer anderen Aktion zugeordnet?
    pub fn conflict(&self, a: GameAction) -> bool {
        let t = self.trigger(a);
        self.map.iter().any(|(other, tt)| *other != a && *tt == t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_have_no_conflicts_and_roundtrip() {
        let b = Bindings::default();
        assert!(GameAction::ALL.iter().all(|a| !b.conflict(*a)));
        let raw: BTreeMap<String, String> = b.clone().into();
        assert_eq!(raw.len(), GameAction::ALL.len());
        assert_eq!(raw["jump"], "space");
        assert_eq!(raw["next_weapon"], "wheel_down");
        assert_eq!(Bindings::from(raw), b);
    }

    #[test]
    fn set_conflict_and_bad_entries() {
        let mut b = Bindings::default();
        assert!(b.set(GameAction::Jump, Trigger::Key(KeyCode::KeyW)));
        assert!(!b.conflict(GameAction::Jump));
        assert!(b.set(GameAction::Hook, Trigger::Key(KeyCode::KeyW)));
        assert!(b.conflict(GameAction::Jump) && b.conflict(GameAction::Hook));
        assert_eq!(b.actions(Trigger::Key(KeyCode::KeyW)).len(), 2);
        assert!(
            !b.set(GameAction::Jump, Trigger::Key(KeyCode::Escape)),
            "Esc ist fest"
        );
        let raw: BTreeMap<String, String> = [
            ("jump".to_owned(), "gibt_es_nicht".to_owned()),
            ("fire".to_owned(), "f1".to_owned()),
            ("left".to_owned(), "q".to_owned()),
            ("unbekannt".to_owned(), "a".to_owned()),
        ]
        .into_iter()
        .collect();
        let b = Bindings::from(raw);
        assert_eq!(
            b.trigger(GameAction::Jump),
            Trigger::Key(KeyCode::Space),
            "ungültig → Standard"
        );
        assert_eq!(
            b.trigger(GameAction::Fire),
            Trigger::Mouse(MouseButton::Left)
        );
        assert_eq!(b.trigger(GameAction::Left), Trigger::Key(KeyCode::KeyQ));
    }

    #[test]
    fn names_are_unique() {
        let mut names: Vec<&str> = KEYS.iter().map(|(_, n, _)| *n).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(names.len(), before);
        for (k, n, _) in KEYS {
            assert_eq!(Trigger::from_name(n), Some(Trigger::Key(*k)));
        }
    }
}

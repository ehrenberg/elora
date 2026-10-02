//! Bedingungen (`if`) und Folgen (`do`) in Gesprächen und Aufgaben (A1.4, E-248).
//!
//! Kurze deutsche Sätze, beim Laden geprüft:
//!
//! | Bedingung | Bedeutung |
//! |---|---|
//! | `stufe >= 3`, `glanz < 50` | Stufe, Glanztropfen (Vergleiche `= != < <= > >=`) |
//! | `quest brunnen neu/aktiv/erledigt/gescheitert` | Zustand einer Aufgabe |
//! | `quest brunnen schritt bruecke` | aktueller Schritt |
//! | `merker oma.frech`, `merker tor >= 2` | Weltzustand (ohne Vergleich: nicht 0) |
//! | `zuneigung lotte >= 5` | Zuneigung einer Figur |
//! | `hat bernstein 3` | Gegenstand (ohne Zahl: mindestens 1) |
//! | `faehigkeit gleiten` | Gebietsfähigkeit |
//!
//! Mehrere Bedingungen mit ` und `, Verneinung mit `nicht ` davor.
//!
//! | Folge | Bedeutung |
//! |---|---|
//! | `quest brunnen start/weiter/fertig/scheitern` | Aufgabe steuern (`weiter` = aktueller Schritt erledigt) |
//! | `zuneigung oma +1` | Zuneigung ändern |
//! | `merker oma.frech = 1`, `merker tor +1` | Weltzustand setzen oder ändern |
//! | `gib heiltrank 2`, `nimm bernstein 3` | Gegenstand an Elora geben bzw. abnehmen |
//! | `erfahrung 50`, `punkte 1` | Erfahrung, Tautropfen-Punkte |
//! | `faehigkeit hook-ruck`, `waffe granate` | Fähigkeit oder Waffe freischalten |
//! | `laden lotte`, `schmied`, `baum` | Laden, Schmiede, Fähigkeitenbaum öffnen |

use std::fmt;

use elora_sim::{Ability, Weapon};

/// Vergleich.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Cmp {
    fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "=" | "==" => Self::Eq,
            "!=" => Self::Ne,
            "<" => Self::Lt,
            "<=" => Self::Le,
            ">" => Self::Gt,
            ">=" => Self::Ge,
            _ => return None,
        })
    }

    pub fn holds(self, a: i64, b: i64) -> bool {
        match self {
            Self::Eq => a == b,
            Self::Ne => a != b,
            Self::Lt => a < b,
            Self::Le => a <= b,
            Self::Gt => a > b,
            Self::Ge => a >= b,
        }
    }
}

/// Zustand einer Aufgabe in Bedingungen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestCheck {
    New,
    Active,
    Done,
    Failed,
    Step(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cond {
    Level(Cmp, i64),
    Glanz(Cmp, i64),
    Quest(String, QuestCheck),
    Flag(String, Cmp, i64),
    Affection(String, Cmp, i64),
    Has(String, u32),
    Ability(Ability),
    Not(Box<Cond>),
    All(Vec<Cond>),
}

/// Steuerung einer Aufgabe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestOp {
    Start,
    Advance,
    Finish,
    Fail,
}

/// Was die Oberfläche öffnen soll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Open {
    Shop(String),
    Forge,
    Skills,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quest(String, QuestOp),
    Affection(String, i32),
    SetFlag(String, i64),
    AddFlag(String, i64),
    Give(String, u32),
    Take(String, u32),
    Xp(u32),
    Points(u32),
    Ability(Ability),
    Weapon(Weapon),
    Open(Open),
}

/// Fehler beim Lesen einer Bedingung oder Folge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptError(pub String);

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn err<T>(src: &str, what: &str) -> Result<T, ScriptError> {
    Err(ScriptError(format!("`{src}`: {what}")))
}

fn int(src: &str, s: &str) -> Result<i64, ScriptError> {
    s.parse().or_else(|_| err(src, "Zahl erwartet"))
}

/// Gebietsfähigkeit nach deutschem Namen.
pub fn ability_by_name(s: &str) -> Option<Ability> {
    Some(match s {
        "hook-ruck" => Ability::HookRuck,
        "heranhooken" => Ability::Pull,
        "stampfen" => Ability::Stomp,
        "eisgriff" => Ability::Grip,
        "gleiten" => Ability::Glide,
        _ => return None,
    })
}

fn weapon_by_name(s: &str) -> Option<Weapon> {
    Some(match s {
        "hammer" => Weapon::Hammer,
        "granate" | "granatwerfer" => Weapon::Grenade,
        "laser" => Weapon::Laser,
        _ => return None,
    })
}

impl Cond {
    /// # Errors
    /// Bei unbekannten Wörtern oder fehlenden Werten.
    pub fn parse(src: &str) -> Result<Self, ScriptError> {
        let parts: Vec<&str> = src.split(" und ").map(str::trim).collect();
        if parts.len() > 1 {
            return parts
                .iter()
                .map(|p| Self::parse(p))
                .collect::<Result<Vec<_>, _>>()
                .map(Self::All);
        }
        let s = src.trim();
        if let Some(rest) = s.strip_prefix("nicht ") {
            return Ok(Self::Not(Box::new(Self::parse(rest)?)));
        }
        let t: Vec<&str> = s.split_whitespace().collect();
        let cmp = |op: &str| {
            Cmp::parse(op).ok_or_else(|| ScriptError(format!("`{src}`: Vergleich erwartet")))
        };
        Ok(match t.as_slice() {
            ["stufe", op, n] => Self::Level(cmp(op)?, int(src, n)?),
            ["glanz", op, n] => Self::Glanz(cmp(op)?, int(src, n)?),
            ["quest", id, state] => Self::Quest(
                (*id).to_owned(),
                match *state {
                    "neu" => QuestCheck::New,
                    "aktiv" => QuestCheck::Active,
                    "erledigt" => QuestCheck::Done,
                    "gescheitert" => QuestCheck::Failed,
                    _ => return err(src, "neu, aktiv, erledigt oder gescheitert erwartet"),
                },
            ),
            ["quest", id, "schritt", step] => {
                Self::Quest((*id).to_owned(), QuestCheck::Step((*step).to_owned()))
            }
            ["merker", name] => Self::Flag((*name).to_owned(), Cmp::Ne, 0),
            ["merker", name, op, n] => Self::Flag((*name).to_owned(), cmp(op)?, int(src, n)?),
            ["zuneigung", who, op, n] => Self::Affection((*who).to_owned(), cmp(op)?, int(src, n)?),
            ["hat", item] => Self::Has((*item).to_owned(), 1),
            ["hat", item, n] => Self::Has(
                (*item).to_owned(),
                u32::try_from(int(src, n)?).or_else(|_| err(src, "Anzahl ab 0"))?,
            ),
            ["faehigkeit", name] => {
                Self::Ability(ability_by_name(name).ok_or_else(|| {
                    ScriptError(format!("`{src}`: unbekannte Fähigkeit `{name}`"))
                })?)
            }
            _ => return err(src, "unbekannte Bedingung"),
        })
    }
}

impl Action {
    /// # Errors
    /// Bei unbekannten Wörtern oder fehlenden Werten.
    pub fn parse(src: &str) -> Result<Self, ScriptError> {
        let t: Vec<&str> = src.split_whitespace().collect();
        let count = |n: Option<&&str>| -> Result<u32, ScriptError> {
            n.map_or(Ok(1), |n| {
                u32::try_from(int(src, n)?).or_else(|_| err(src, "Anzahl ab 0"))
            })
        };
        let signed =
            |n: &str| -> Result<i64, ScriptError> { int(src, n.strip_prefix('+').unwrap_or(n)) };
        Ok(match t.as_slice() {
            ["quest", id, op] => Self::Quest(
                (*id).to_owned(),
                match *op {
                    "start" => QuestOp::Start,
                    "weiter" => QuestOp::Advance,
                    "fertig" => QuestOp::Finish,
                    "scheitern" => QuestOp::Fail,
                    _ => return err(src, "start, weiter, fertig oder scheitern erwartet"),
                },
            ),
            ["zuneigung", who, n] => Self::Affection(
                (*who).to_owned(),
                i32::try_from(signed(n)?).or_else(|_| err(src, "Zahl zu groß"))?,
            ),
            ["merker", name, "=", n] => Self::SetFlag((*name).to_owned(), int(src, n)?),
            ["merker", name, n] if n.starts_with(['+', '-']) => {
                Self::AddFlag((*name).to_owned(), signed(n)?)
            }
            ["gib", item, rest @ ..] if rest.len() <= 1 => {
                Self::Give((*item).to_owned(), count(rest.first())?)
            }
            ["nimm", item, rest @ ..] if rest.len() <= 1 => {
                Self::Take((*item).to_owned(), count(rest.first())?)
            }
            ["erfahrung", n] => Self::Xp(count(Some(n))?),
            ["punkte", n] => Self::Points(count(Some(n))?),
            ["faehigkeit", name] => {
                Self::Ability(ability_by_name(name).ok_or_else(|| {
                    ScriptError(format!("`{src}`: unbekannte Fähigkeit `{name}`"))
                })?)
            }
            ["waffe", name] => Self::Weapon(
                weapon_by_name(name)
                    .ok_or_else(|| ScriptError(format!("`{src}`: unbekannte Waffe `{name}`")))?,
            ),
            ["laden", id] => Self::Open(Open::Shop((*id).to_owned())),
            ["schmied"] => Self::Open(Open::Forge),
            ["baum"] => Self::Open(Open::Skills),
            _ => return err(src, "unbekannte Folge"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditions_parse() {
        assert_eq!(Cond::parse("stufe >= 3"), Ok(Cond::Level(Cmp::Ge, 3)));
        assert_eq!(
            Cond::parse("nicht merker oma.frech und hat bernstein 2"),
            Ok(Cond::All(vec![
                Cond::Not(Box::new(Cond::Flag("oma.frech".into(), Cmp::Ne, 0))),
                Cond::Has("bernstein".into(), 2),
            ]))
        );
        assert_eq!(
            Cond::parse("quest brunnen schritt bruecke"),
            Ok(Cond::Quest(
                "brunnen".into(),
                QuestCheck::Step("bruecke".into())
            ))
        );
        assert_eq!(
            Cond::parse("faehigkeit gleiten"),
            Ok(Cond::Ability(Ability::Glide))
        );
        assert!(Cond::parse("stufe ungefähr 3").is_err());
        assert!(Cond::parse("quest brunnen bald").is_err());
        assert!(Cond::parse("wetter schön").is_err());
    }

    #[test]
    fn actions_parse() {
        assert_eq!(
            Action::parse("quest brunnen start"),
            Ok(Action::Quest("brunnen".into(), QuestOp::Start))
        );
        assert_eq!(
            Action::parse("zuneigung oma +1"),
            Ok(Action::Affection("oma".into(), 1))
        );
        assert_eq!(
            Action::parse("zuneigung pip -2"),
            Ok(Action::Affection("pip".into(), -2))
        );
        assert_eq!(
            Action::parse("merker tor = 2"),
            Ok(Action::SetFlag("tor".into(), 2))
        );
        assert_eq!(
            Action::parse("merker tor +1"),
            Ok(Action::AddFlag("tor".into(), 1))
        );
        assert_eq!(
            Action::parse("gib heiltrank"),
            Ok(Action::Give("heiltrank".into(), 1))
        );
        assert_eq!(
            Action::parse("waffe granate"),
            Ok(Action::Weapon(Weapon::Grenade))
        );
        assert_eq!(
            Action::parse("laden lotte"),
            Ok(Action::Open(Open::Shop("lotte".into())))
        );
        assert!(Action::parse("gib").is_err());
        assert!(Action::parse("fliegen").is_err());
    }
}

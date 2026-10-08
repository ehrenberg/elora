//! Conditions (`if`) and effects (`do`) in dialogs and quests (A1.4, E-248, RF-11).
//!
//! Short English phrases, checked when loading. The German keywords of 0.9.x (`merker`,
//! `quest … schritt`, `gib`, `nimm`, `faehigkeit` …) are still accepted for one release
//! (D-RF-02): they are translated with the id dictionary before parsing.
//!
//! | Condition | Meaning |
//! |---|---|
//! | `level >= 3`, `gleam < 50` | level, gleam drops (comparisons `= != < <= > >=`) |
//! | `quest well new/active/done/failed` | state of a quest |
//! | `quest well step bridge` | current step |
//! | `flag oma.cheeky`, `flag gate >= 2` | world state (without comparison: not 0) |
//! | `affection lotte >= 5` | affection of a character |
//! | `has amber 3` | item (without number: at least 1) |
//! | `ability glide` | area ability: `hook-jerk`, `pull_hook`, `stomp`, `ice_grip`, `glide` |
//! | `weapon hammer` | Elora owns the weapon (E-354) |
//!
//! Several conditions with ` and `, negation with `not ` in front.
//!
//! | Effect | Meaning |
//! |---|---|
//! | `quest well start/advance/finish/fail` | control a quest (`advance` = current step done) |
//! | `affection oma +1` | change affection |
//! | `flag oma.cheeky = 1`, `flag gate +1` | set or change world state |
//! | `give healing_potion 2`, `take amber 3` | give an item to Elora or take it away |
//! | `xp 50`, `points 1` | experience, dewdrop points |
//! | `ability hook-jerk`, `weapon grenade` | unlock an ability or weapon |
//! | `shop lotte`, `forge`, `skills` | open shop, smithy, skill tree |

use std::fmt;

use elora_sim::{Ability, Weapon};

/// Comparison.
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

/// State of a quest in conditions.
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
    Gleam(Cmp, i64),
    Quest(String, QuestCheck),
    Flag(String, Cmp, i64),
    Affection(String, Cmp, i64),
    Has(String, u32),
    Ability(Ability),
    Weapon(Weapon),
    Not(Box<Cond>),
    All(Vec<Cond>),
}

/// Control of a quest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestOp {
    Start,
    Advance,
    Finish,
    Fail,
}

/// What the UI should open.
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

/// Error when reading a condition or effect.
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
    s.parse().or_else(|_| err(src, "number expected"))
}

/// Area ability by its name in the condition language.
pub fn ability_by_name(s: &str) -> Option<Ability> {
    Some(match s {
        "hook-jerk" => Ability::HookJerk,
        "pull_hook" => Ability::Pull,
        "stomp" => Ability::Stomp,
        "ice_grip" => Ability::Grip,
        "glide" => Ability::Glide,
        _ => return None,
    })
}

fn weapon_by_name(s: &str) -> Option<Weapon> {
    Some(match s {
        "hammer" => Weapon::Hammer,
        "grenade" | "grenade_launcher" => Weapon::Grenade,
        "laser" => Weapon::Laser,
        _ => return None,
    })
}

impl Cond {
    /// # Errors
    /// On unknown words or missing values.
    pub fn parse(src: &str) -> Result<Self, ScriptError> {
        let english = elora_map::rename::translate_script(src.trim());
        Self::parse_english(&english).map_err(|e| ScriptError(e.0.replace(&english, src)))
    }

    fn parse_english(src: &str) -> Result<Self, ScriptError> {
        let parts: Vec<&str> = src.split(" and ").map(str::trim).collect();
        if parts.len() > 1 {
            return parts
                .iter()
                .map(|p| Self::parse_english(p))
                .collect::<Result<Vec<_>, _>>()
                .map(Self::All);
        }
        let s = src.trim();
        if let Some(rest) = s.strip_prefix("not ") {
            return Ok(Self::Not(Box::new(Self::parse_english(rest)?)));
        }
        let t: Vec<&str> = s.split_whitespace().collect();
        let cmp = |op: &str| {
            Cmp::parse(op).ok_or_else(|| ScriptError(format!("`{src}`: comparison expected")))
        };
        Ok(match t.as_slice() {
            ["level", op, n] => Self::Level(cmp(op)?, int(src, n)?),
            ["gleam", op, n] => Self::Gleam(cmp(op)?, int(src, n)?),
            ["quest", id, state] => Self::Quest(
                (*id).to_owned(),
                match *state {
                    "new" => QuestCheck::New,
                    "active" => QuestCheck::Active,
                    "done" => QuestCheck::Done,
                    "failed" => QuestCheck::Failed,
                    _ => return err(src, "expected new, active, done or failed"),
                },
            ),
            ["quest", id, "step", step] => {
                Self::Quest((*id).to_owned(), QuestCheck::Step((*step).to_owned()))
            }
            ["flag", name] => Self::Flag((*name).to_owned(), Cmp::Ne, 0),
            ["flag", name, op, n] => Self::Flag((*name).to_owned(), cmp(op)?, int(src, n)?),
            ["affection", who, op, n] => Self::Affection((*who).to_owned(), cmp(op)?, int(src, n)?),
            ["has", item] => Self::Has((*item).to_owned(), 1),
            ["has", item, n] => Self::Has(
                (*item).to_owned(),
                u32::try_from(int(src, n)?).or_else(|_| err(src, "count must be 0 or more"))?,
            ),
            ["ability", name] => Self::Ability(
                ability_by_name(name)
                    .ok_or_else(|| ScriptError(format!("`{src}`: unknown ability `{name}`")))?,
            ),
            ["weapon", name] => Self::Weapon(
                weapon_by_name(name)
                    .ok_or_else(|| ScriptError(format!("`{src}`: unknown weapon `{name}`")))?,
            ),
            _ => return err(src, "unknown condition"),
        })
    }
}

impl Action {
    /// # Errors
    /// On unknown words or missing values.
    pub fn parse(src: &str) -> Result<Self, ScriptError> {
        let english = elora_map::rename::translate_script(src.trim());
        Self::parse_english(&english).map_err(|e| ScriptError(e.0.replace(&english, src)))
    }

    fn parse_english(src: &str) -> Result<Self, ScriptError> {
        let t: Vec<&str> = src.split_whitespace().collect();
        let count = |n: Option<&&str>| -> Result<u32, ScriptError> {
            n.map_or(Ok(1), |n| {
                u32::try_from(int(src, n)?).or_else(|_| err(src, "count must be 0 or more"))
            })
        };
        let signed =
            |n: &str| -> Result<i64, ScriptError> { int(src, n.strip_prefix('+').unwrap_or(n)) };
        Ok(match t.as_slice() {
            ["quest", id, op] => Self::Quest(
                (*id).to_owned(),
                match *op {
                    "start" => QuestOp::Start,
                    "advance" => QuestOp::Advance,
                    "finish" => QuestOp::Finish,
                    "fail" => QuestOp::Fail,
                    _ => return err(src, "expected start, advance, finish or fail"),
                },
            ),
            ["affection", who, n] => Self::Affection(
                (*who).to_owned(),
                i32::try_from(signed(n)?).or_else(|_| err(src, "number too large"))?,
            ),
            ["flag", name, "=", n] => Self::SetFlag((*name).to_owned(), int(src, n)?),
            ["flag", name, n] if n.starts_with(['+', '-']) => {
                Self::AddFlag((*name).to_owned(), signed(n)?)
            }
            ["give", item, rest @ ..] if rest.len() <= 1 => {
                Self::Give((*item).to_owned(), count(rest.first())?)
            }
            ["take", item, rest @ ..] if rest.len() <= 1 => {
                Self::Take((*item).to_owned(), count(rest.first())?)
            }
            ["xp", n] => Self::Xp(count(Some(n))?),
            ["points", n] => Self::Points(count(Some(n))?),
            ["ability", name] => Self::Ability(
                ability_by_name(name)
                    .ok_or_else(|| ScriptError(format!("`{src}`: unknown ability `{name}`")))?,
            ),
            ["weapon", name] => Self::Weapon(
                weapon_by_name(name)
                    .ok_or_else(|| ScriptError(format!("`{src}`: unknown weapon `{name}`")))?,
            ),
            ["shop", id] => Self::Open(Open::Shop((*id).to_owned())),
            ["forge"] => Self::Open(Open::Forge),
            ["skills"] => Self::Open(Open::Skills),
            _ => return err(src, "unknown action"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditions_parse() {
        assert_eq!(Cond::parse("level >= 3"), Ok(Cond::Level(Cmp::Ge, 3)));
        assert_eq!(
            Cond::parse("not flag oma.cheeky and has amber 2"),
            Ok(Cond::All(vec![
                Cond::Not(Box::new(Cond::Flag("oma.cheeky".into(), Cmp::Ne, 0))),
                Cond::Has("amber".into(), 2),
            ]))
        );
        assert_eq!(
            Cond::parse("quest well step bridge"),
            Ok(Cond::Quest(
                "well".into(),
                QuestCheck::Step("bridge".into())
            ))
        );
        assert_eq!(
            Cond::parse("ability glide"),
            Ok(Cond::Ability(Ability::Glide))
        );
        assert_eq!(
            Cond::parse("weapon hammer"),
            Ok(Cond::Weapon(Weapon::Hammer))
        );
        assert!(Cond::parse("stufe ungefähr 3").is_err());
        assert!(Cond::parse("quest well bald").is_err());
        assert!(Cond::parse("wetter schön").is_err());
    }

    #[test]
    fn actions_parse() {
        assert_eq!(
            Action::parse("quest well start"),
            Ok(Action::Quest("well".into(), QuestOp::Start))
        );
        assert_eq!(
            Action::parse("affection oma +1"),
            Ok(Action::Affection("oma".into(), 1))
        );
        assert_eq!(
            Action::parse("affection pip -2"),
            Ok(Action::Affection("pip".into(), -2))
        );
        assert_eq!(
            Action::parse("flag gate = 2"),
            Ok(Action::SetFlag("gate".into(), 2))
        );
        assert_eq!(
            Action::parse("flag gate +1"),
            Ok(Action::AddFlag("gate".into(), 1))
        );
        assert_eq!(
            Action::parse("give healing_potion"),
            Ok(Action::Give("healing_potion".into(), 1))
        );
        assert_eq!(
            Action::parse("weapon grenade"),
            Ok(Action::Weapon(Weapon::Grenade))
        );
        assert_eq!(
            Action::parse("shop lotte"),
            Ok(Action::Open(Open::Shop("lotte".into())))
        );
        assert!(Action::parse("gib").is_err());
        assert!(Action::parse("fly").is_err());
    }

    /// The German words of 0.9.x still work for one release (D-RF-02).
    #[test]
    fn german_keywords_are_still_accepted() {
        assert_eq!(
            Cond::parse("nicht merker oma.frech und hat bernstein 2"),
            Cond::parse("not flag oma.cheeky and has amber 2")
        );
        assert_eq!(
            Cond::parse("quest brunnen schritt bruecke"),
            Cond::parse("quest well step bridge")
        );
        assert_eq!(
            Cond::parse("faehigkeit hook-ruck"),
            Ok(Cond::Ability(Ability::HookJerk))
        );
        assert_eq!(
            Action::parse("gib heiltrank 2"),
            Action::parse("give healing_potion 2")
        );
        assert_eq!(
            Action::parse("quest brunnen weiter"),
            Action::parse("quest well advance")
        );
        assert_eq!(Action::parse("baum"), Ok(Action::Open(Open::Skills)));
        let e = Cond::parse("merker").unwrap_err();
        assert!(
            e.0.contains("`merker`"),
            "error names the original text: {e}"
        );
    }
}

//! Aufgaben (A1.4, E-249 bis E-251): Schritte mit Zielen, Belohnung, Scheitern – und das
//! Auswerten von Bedingungen und Folgen auf dem Spielstand.

use serde::{Deserialize, Serialize};

use crate::data::{Content, Cost, GLANZTROPFEN, Text};
use crate::script::{Action, Cond, Open, QuestCheck, QuestOp};
use crate::state::{Notice, SaveGame};

/// Haupt- oder Nebenaufgabe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestKind {
    #[default]
    Main,
    Side,
}

/// Ziel eines Schritts (E-249).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Goal {
    /// Mit einer Figur sprechen.
    Talk { who: String },
    /// Karte (und optional eine Zone darin) erreichen.
    Reach {
        map: String,
        #[serde(default)]
        zone: Option<String>,
    },
    /// Gegner einer Art besiegen, optional nur auf einer Karte.
    Defeat {
        kind: String,
        count: u32,
        #[serde(default)]
        map: Option<String>,
    },
    /// Gegenstände haben.
    Collect { item: String, count: u32 },
    /// Gegenstände bei einer Figur abgeben.
    Bring {
        item: String,
        count: u32,
        to: String,
    },
    /// Merker im Weltzustand (Schalter, Truhe, Tür …).
    Flag {
        flag: String,
        #[serde(default = "one")]
        value: i64,
    },
    /// Nur über Gespräche (`quest x weiter`).
    Manual,
}

fn one() -> i64 {
    1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub text: Text,
    pub goal: Goal,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Reward {
    #[serde(default)]
    pub xp: u32,
    #[serde(default)]
    pub glanztropfen: u32,
    #[serde(default)]
    pub items: Vec<Cost>,
    /// Tautropfen-Punkte.
    #[serde(default)]
    pub points: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestDef {
    pub id: String,
    #[serde(default)]
    pub kind: QuestKind,
    /// Wer die Aufgabe vergibt (Anzeige im Aufgabenbuch).
    #[serde(default)]
    pub giver: Option<String>,
    pub name: Text,
    #[serde(default)]
    pub desc: Text,
    pub step: Vec<Step>,
    #[serde(default)]
    pub reward: Reward,
    /// Scheitert, sobald diese Bedingung gilt (E-250).
    #[serde(default)]
    pub fail_if: Option<String>,
}

/// Zustand einer begonnenen Aufgabe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestStatus {
    Active,
    Done,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestState {
    pub status: QuestStatus,
    /// Index des aktuellen Schritts.
    pub step: usize,
    /// Fortschritt im Schritt (z. B. besiegte Gegner).
    pub progress: u32,
}

/// Ergebnis einer Folge für die Oberfläche.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Notice(Notice),
    Open(Open),
}

impl SaveGame {
    pub fn quest(&self, id: &str) -> Option<&QuestState> {
        self.quests.get(id)
    }

    pub fn affection(&self, who: &str) -> i32 {
        self.affection.get(who).copied().unwrap_or(0)
    }

    /// Gilt die Bedingung (Text siehe [`crate::script`])? Ungültige Bedingungen gelten nie
    /// (die Inhalte werden beim Laden geprüft).
    pub fn holds(&self, content: &Content, src: &str) -> bool {
        Cond::parse(src).is_ok_and(|c| self.check(content, &c))
    }

    pub fn check(&self, content: &Content, c: &Cond) -> bool {
        match c {
            Cond::Level(op, n) => op.holds(i64::from(self.level), *n),
            Cond::Glanz(op, n) => op.holds(i64::from(self.glanztropfen), *n),
            Cond::Quest(id, q) => {
                let st = self.quests.get(id);
                match q {
                    QuestCheck::New => st.is_none(),
                    QuestCheck::Active => st.is_some_and(|s| s.status == QuestStatus::Active),
                    QuestCheck::Done => st.is_some_and(|s| s.status == QuestStatus::Done),
                    QuestCheck::Failed => st.is_some_and(|s| s.status == QuestStatus::Failed),
                    QuestCheck::Step(step) => st.is_some_and(|s| {
                        s.status == QuestStatus::Active
                            && content
                                .quest(id)
                                .and_then(|d| d.step.get(s.step))
                                .is_some_and(|d| &d.id == step)
                    }),
                }
            }
            Cond::Flag(name, op, n) => op.holds(self.flag(name), *n),
            Cond::Affection(who, op, n) => op.holds(i64::from(self.affection(who)), *n),
            Cond::Has(item, n) => self.count(item) >= *n,
            Cond::Ability(a) => self.abilities().has(*a),
            Cond::Not(c) => !self.check(content, c),
            Cond::All(cs) => cs.iter().all(|c| self.check(content, c)),
        }
    }

    /// Folgen ausführen (Texte siehe [`crate::script`]); ungültige werden übersprungen.
    pub fn run(&mut self, content: &Content, actions: &[String]) -> Vec<Outcome> {
        let mut out = Vec::new();
        for a in actions {
            if let Ok(a) = Action::parse(a) {
                out.extend(self.apply(content, &a));
            }
        }
        out.extend(self.update_quests(content));
        out
    }

    fn apply(&mut self, content: &Content, a: &Action) -> Vec<Outcome> {
        let notices = |v: Vec<Notice>| v.into_iter().map(Outcome::Notice).collect();
        match a {
            Action::Quest(id, op) => match op {
                QuestOp::Start => self.start_quest(content, id),
                QuestOp::Advance => self.advance_quest(content, id),
                QuestOp::Finish => self.finish_quest(content, id),
                QuestOp::Fail => self.fail_quest(id),
            },
            Action::Affection(who, d) => {
                let v = (self.affection(who) + d).clamp(AFFECTION_MIN, AFFECTION_MAX);
                self.affection.insert(who.clone(), v);
                Vec::new()
            }
            Action::SetFlag(n, v) => {
                self.set_flag(n, *v);
                Vec::new()
            }
            Action::AddFlag(n, d) => {
                self.set_flag(n, self.flag(n) + d);
                Vec::new()
            }
            Action::Give(item, n) => {
                if self.add_item(content, item, *n).is_ok() {
                    vec![Outcome::Notice(Notice::Item {
                        id: item.clone(),
                        count: *n,
                    })]
                } else {
                    Vec::new()
                }
            }
            Action::Take(item, n) => {
                let have = self.count(item).min(*n);
                let _ = self.remove_item(item, have);
                Vec::new()
            }
            Action::Xp(n) => notices(self.add_xp(content, *n)),
            Action::Points(n) => {
                self.bonus_points += n;
                Vec::new()
            }
            Action::Ability(ab) => {
                self.grant_ability(*ab);
                Vec::new()
            }
            Action::Weapon(w) => {
                self.give_weapon(*w);
                Vec::new()
            }
            Action::Open(o) => vec![Outcome::Open(o.clone())],
        }
    }

    // ------------------------------------------------------------ Aufgaben steuern

    fn start_quest(&mut self, content: &Content, id: &str) -> Vec<Outcome> {
        if self.quests.contains_key(id) || content.quest(id).is_none() {
            return Vec::new();
        }
        self.quests.insert(
            id.to_owned(),
            QuestState {
                status: QuestStatus::Active,
                step: 0,
                progress: 0,
            },
        );
        vec![Outcome::Notice(Notice::QuestStarted(id.to_owned()))]
    }

    /// Aktueller Schritt erledigt; nach dem letzten ist die Aufgabe fertig.
    fn advance_quest(&mut self, content: &Content, id: &str) -> Vec<Outcome> {
        let Some(def) = content.quest(id) else {
            return Vec::new();
        };
        let Some(st) = self
            .quests
            .get_mut(id)
            .filter(|s| s.status == QuestStatus::Active)
        else {
            return Vec::new();
        };
        st.step += 1;
        st.progress = 0;
        if st.step >= def.step.len() {
            return self.finish_quest(content, id);
        }
        vec![Outcome::Notice(Notice::QuestStep(id.to_owned()))]
    }

    fn finish_quest(&mut self, content: &Content, id: &str) -> Vec<Outcome> {
        let Some(def) = content.quest(id) else {
            return Vec::new();
        };
        if self
            .quests
            .get(id)
            .is_some_and(|s| s.status != QuestStatus::Active)
        {
            return Vec::new();
        }
        self.quests.insert(
            id.to_owned(),
            QuestState {
                status: QuestStatus::Done,
                step: def.step.len(),
                progress: 0,
            },
        );
        let mut out = vec![Outcome::Notice(Notice::QuestDone(id.to_owned()))];
        let r = def.reward.clone();
        if r.glanztropfen > 0 && self.add_item(content, GLANZTROPFEN, r.glanztropfen).is_ok() {
            out.push(Outcome::Notice(Notice::Item {
                id: GLANZTROPFEN.into(),
                count: r.glanztropfen,
            }));
        }
        for c in &r.items {
            if self.add_item(content, &c.item, c.count).is_ok() {
                out.push(Outcome::Notice(Notice::Item {
                    id: c.item.clone(),
                    count: c.count,
                }));
            }
        }
        self.bonus_points += r.points;
        if r.xp > 0 {
            out.extend(self.add_xp(content, r.xp).into_iter().map(Outcome::Notice));
        }
        out
    }

    fn fail_quest(&mut self, id: &str) -> Vec<Outcome> {
        match self.quests.get_mut(id) {
            Some(st) if st.status == QuestStatus::Active => {
                st.status = QuestStatus::Failed;
                vec![Outcome::Notice(Notice::QuestFailed(id.to_owned()))]
            }
            _ => Vec::new(),
        }
    }

    /// Aktive Aufgaben mit ihrem aktuellen Schritt.
    fn active_steps<'c>(&self, content: &'c Content) -> Vec<(String, &'c Step)> {
        self.quests
            .iter()
            .filter(|(_, s)| s.status == QuestStatus::Active)
            .filter_map(|(id, s)| Some((id.clone(), content.quest(id)?.step.get(s.step)?)))
            .collect()
    }

    /// Selbsttätige Ziele (Sammeln, Merker) und Scheitern prüfen; nach jeder Änderung.
    pub fn update_quests(&mut self, content: &Content) -> Vec<Outcome> {
        let mut out = Vec::new();
        // Mehrere Schritte können nacheinander erfüllt sein
        for _ in 0..16 {
            let mut changed = false;
            for (id, step) in self.active_steps(content) {
                let fail = content
                    .quest(&id)
                    .and_then(|d| d.fail_if.as_deref())
                    .is_some_and(|f| self.holds(content, f));
                if fail {
                    out.extend(self.fail_quest(&id));
                    changed = true;
                    continue;
                }
                let done = match &step.goal {
                    Goal::Collect { item, count } => self.count(item) >= *count,
                    Goal::Flag { flag, value } => self.flag(flag) == *value,
                    _ => false,
                };
                if done {
                    out.extend(self.advance_quest(content, &id));
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        out
    }

    /// Gespräch mit `who` begonnen: Sprechen- und Bringen-Ziele.
    pub fn on_talk(&mut self, content: &Content, who: &str) -> Vec<Outcome> {
        let mut out = Vec::new();
        for (id, step) in self.active_steps(content) {
            match &step.goal {
                Goal::Talk { who: w } if w == who => out.extend(self.advance_quest(content, &id)),
                Goal::Bring { item, count, to } if to == who && self.count(item) >= *count => {
                    let _ = self.remove_item(item, *count);
                    out.extend(self.advance_quest(content, &id));
                }
                _ => {}
            }
        }
        out.extend(self.update_quests(content));
        out
    }

    /// Karte oder Zone erreicht.
    pub fn on_reach(&mut self, content: &Content, map: &str, zone: Option<&str>) -> Vec<Outcome> {
        let mut out = Vec::new();
        for (id, step) in self.active_steps(content) {
            if let Goal::Reach { map: m, zone: z } = &step.goal
                && m == map
                && (z.is_none() || z.as_deref() == zone)
            {
                out.extend(self.advance_quest(content, &id));
            }
        }
        out.extend(self.update_quests(content));
        out
    }

    /// Gegner der Art `kind` auf Karte `map` besiegt.
    pub fn on_defeat(&mut self, content: &Content, kind: &str, map: &str) -> Vec<Outcome> {
        let mut out = Vec::new();
        for (id, step) in self.active_steps(content) {
            if let Goal::Defeat {
                kind: k,
                count,
                map: m,
            } = &step.goal
                && k == kind
                && m.as_deref().is_none_or(|m| m == map)
                && let Some(st) = self.quests.get_mut(&id)
            {
                st.progress += 1;
                if st.progress >= *count {
                    out.extend(self.advance_quest(content, &id));
                }
            }
        }
        out.extend(self.update_quests(content));
        out
    }
}

/// Zuneigung je Figur von −10 bis 10 (E-248).
pub const AFFECTION_MIN: i32 = -10;
pub const AFFECTION_MAX: i32 = 10;

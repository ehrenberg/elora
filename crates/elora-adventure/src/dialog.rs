//! Dialogs (A1.4, E-213, E-218, E-222, E-246 to E-248): nodes with text in both languages,
//! choices with tone, conditions and effects; plus short calls (speech bubbles).
//!
//! One file per dialog under `assets/adventure/dialogs/<id>.toml`; characters (name, picture)
//! in `assets/adventure/characters.toml`.

use serde::{Deserialize, Serialize};

use crate::data::{Content, Text};
use crate::quest::Outcome;
use crate::state::SaveGame;

/// Tone of an answer (world book §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    #[serde(rename = "freundlich")]
    Friendly,
    #[serde(rename = "neugierig")]
    Curious,
    #[serde(rename = "frech")]
    Cheeky,
}

/// A character that speaks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterDef {
    pub id: String,
    pub name: Text,
    /// Picture of the character in the dialog panel (graphic name).
    #[serde(default)]
    pub portrait: Option<String>,
    /// Does not turn towards Elora (signs).
    #[serde(default)]
    pub fixed: bool,
    /// Character is only visible while the condition holds (e.g. only after a fight).
    #[serde(default)]
    pub show_if: Option<String>,
    /// Companion (E-308): enemy kind that follows Elora while `follow_if` holds – also across
    /// map changes. When the companion reaches the zone `home_zone`, the flag `<id>.daheim` is
    /// set and it stays there.
    #[serde(default)]
    pub follower: Option<String>,
    #[serde(default)]
    pub follow_if: Option<String>,
    #[serde(default)]
    pub home_zone: Option<String>,
    /// Pitch of the babble sounds (E-286): 1 = medium, smaller = lower; 0 = mute (signs).
    #[serde(default = "default_voice")]
    pub voice: f32,
}

fn default_voice() -> f32 {
    1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Start {
    #[serde(default, rename = "if")]
    pub cond: Option<String>,
    pub node: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    #[serde(default)]
    pub tone: Option<Tone>,
    #[serde(default, rename = "if")]
    pub cond: Option<String>,
    pub text: Text,
    /// Next node; if missing, the dialog ends.
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default, rename = "do")]
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    /// Speaker; if missing, the dialog's speaker, `elora` for Elora herself.
    #[serde(default)]
    pub speaker: Option<String>,
    pub text: Text,
    /// Next node without a choice; if missing (and without choices), the dialog ends.
    #[serde(default)]
    pub next: Option<String>,
    /// Effects when reaching the node.
    #[serde(default, rename = "do")]
    pub actions: Vec<String>,
    #[serde(default)]
    pub choice: Vec<Choice>,
}

/// Short call as a speech bubble (E-222).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bark {
    #[serde(default, rename = "if")]
    pub cond: Option<String>,
    pub text: Text,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dialog {
    /// From the file name.
    #[serde(skip)]
    pub id: String,
    /// Default speaker of the nodes.
    pub speaker: String,
    /// Entries: the first whose condition holds.
    pub start: Vec<Start>,
    pub node: Vec<Node>,
    #[serde(default)]
    pub bark: Vec<Bark>,
}

impl Dialog {
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.node.iter().find(|n| n.id == id)
    }

    pub fn speaker_of<'a>(&'a self, n: &'a Node) -> &'a str {
        n.speaker.as_deref().unwrap_or(&self.speaker)
    }
}

/// A running dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conversation {
    pub dialog: String,
    pub node: String,
}

/// What a step in the dialog causes.
#[derive(Debug, Clone, PartialEq)]
pub struct Turn {
    /// `false`: the dialog is over.
    pub open: bool,
    pub outcomes: Vec<Outcome>,
}

fn cond_ok(save: &SaveGame, content: &Content, cond: Option<&String>) -> bool {
    cond.is_none_or(|c| save.holds(content, c))
}

impl Conversation {
    /// Starts the dialog `dialog`: first matching entry, talk goals of the quests, effects of
    /// the first node. `None` if no entry matches.
    pub fn start(content: &Content, save: &mut SaveGame, dialog: &str) -> Option<(Self, Turn)> {
        let d = content.dialog(dialog)?;
        let entry = d
            .start
            .iter()
            .find(|s| cond_ok(save, content, s.cond.as_ref()))?;
        let mut outcomes = save.on_talk(content, &d.speaker);
        let mut c = Self {
            dialog: dialog.to_owned(),
            node: entry.node.clone(),
        };
        let turn = c.enter(content, save, entry.node.clone());
        outcomes.extend(turn.outcomes);
        Some((
            c,
            Turn {
                open: turn.open,
                outcomes,
            },
        ))
    }

    fn enter(&mut self, content: &Content, save: &mut SaveGame, node: String) -> Turn {
        let Some(n) = content.dialog(&self.dialog).and_then(|d| d.node(&node)) else {
            return Turn {
                open: false,
                outcomes: Vec::new(),
            };
        };
        self.node = node;
        Turn {
            open: true,
            outcomes: save.run(content, &n.actions),
        }
    }

    pub fn current<'c>(&self, content: &'c Content) -> Option<(&'c Dialog, &'c Node)> {
        let d = content.dialog(&self.dialog)?;
        Some((d, d.node(&self.node)?))
    }

    /// Visible answers (index into `node.choice`).
    pub fn choices(&self, content: &Content, save: &SaveGame) -> Vec<usize> {
        self.current(content).map_or_else(Vec::new, |(_, n)| {
            n.choice
                .iter()
                .enumerate()
                .filter(|(_, c)| cond_ok(save, content, c.cond.as_ref()))
                .map(|(i, _)| i)
                .collect()
        })
    }

    /// Continue without a choice (node without visible answers).
    pub fn advance(&mut self, content: &Content, save: &mut SaveGame) -> Turn {
        let next = self.current(content).and_then(|(_, n)| n.next.clone());
        match next {
            Some(n) => self.enter(content, save, n),
            None => Turn {
                open: false,
                outcomes: Vec::new(),
            },
        }
    }

    /// Choose answer `index` (from [`Self::choices`]).
    pub fn choose(&mut self, content: &Content, save: &mut SaveGame, index: usize) -> Turn {
        if !self.choices(content, save).contains(&index) {
            return Turn {
                open: true,
                outcomes: Vec::new(),
            };
        }
        let Some((_, n)) = self.current(content) else {
            return Turn {
                open: false,
                outcomes: Vec::new(),
            };
        };
        let choice = n.choice[index].clone();
        let mut outcomes = save.run(content, &choice.actions);
        let turn = match choice.next {
            Some(next) => self.enter(content, save, next),
            None => Turn {
                open: false,
                outcomes: Vec::new(),
            },
        };
        outcomes.extend(turn.outcomes);
        Turn {
            open: turn.open,
            outcomes,
        }
    }
}

/// Call of a character (first matching one from its dialog).
pub fn bark<'c>(content: &'c Content, save: &SaveGame, dialog: &str) -> Option<&'c Text> {
    content
        .dialog(dialog)?
        .bark
        .iter()
        .find(|b| cond_ok(save, content, b.cond.as_ref()))
        .map(|b| &b.text)
}

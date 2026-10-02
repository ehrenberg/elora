//! Gespräche (A1.4, E-213, E-218, E-222, E-246 bis E-248): Knoten mit Text in beiden
//! Sprachen, Auswahl mit Ton, Bedingungen und Folgen; dazu kurze Zurufe (Sprechblasen).
//!
//! Eine Datei je Gespräch unter `assets/adventure/dialogs/<id>.toml`; Figuren (Name, Bild)
//! in `assets/adventure/characters.toml`.

use serde::{Deserialize, Serialize};

use crate::data::{Content, Text};
use crate::quest::Outcome;
use crate::state::SaveGame;

/// Ton einer Antwort (Weltbuch §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    Freundlich,
    Neugierig,
    Frech,
}

/// Eine Figur, die spricht.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterDef {
    pub id: String,
    pub name: Text,
    /// Bild der Figur im Gesprächsfeld (Grafikname).
    #[serde(default)]
    pub portrait: Option<String>,
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
    /// Nächster Knoten; ohne Angabe endet das Gespräch.
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default, rename = "do")]
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    /// Sprecher; ohne Angabe der des Gesprächs, `elora` für Elora selbst.
    #[serde(default)]
    pub speaker: Option<String>,
    pub text: Text,
    /// Nächster Knoten ohne Auswahl; ohne Angabe (und ohne Auswahl) endet das Gespräch.
    #[serde(default)]
    pub next: Option<String>,
    /// Folgen beim Erreichen des Knotens.
    #[serde(default, rename = "do")]
    pub actions: Vec<String>,
    #[serde(default)]
    pub choice: Vec<Choice>,
}

/// Kurzer Zuruf als Sprechblase (E-222).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bark {
    #[serde(default, rename = "if")]
    pub cond: Option<String>,
    pub text: Text,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dialog {
    /// Aus dem Dateinamen.
    #[serde(skip)]
    pub id: String,
    /// Standard-Sprecher der Knoten.
    pub speaker: String,
    /// Einstiege: der erste, dessen Bedingung gilt.
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

/// Ein laufendes Gespräch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conversation {
    pub dialog: String,
    pub node: String,
}

/// Was ein Schritt im Gespräch bewirkt.
#[derive(Debug, Clone, PartialEq)]
pub struct Turn {
    /// `false`: Gespräch ist zu Ende.
    pub open: bool,
    pub outcomes: Vec<Outcome>,
}

fn cond_ok(save: &SaveGame, content: &Content, cond: Option<&String>) -> bool {
    cond.is_none_or(|c| save.holds(content, c))
}

impl Conversation {
    /// Beginnt das Gespräch `dialog`: erster passender Einstieg, Sprechen-Ziele der Aufgaben,
    /// Folgen des ersten Knotens. `None`, wenn kein Einstieg passt.
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

    /// Sichtbare Antworten (Index in `node.choice`).
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

    /// Weiter ohne Auswahl (Knoten ohne sichtbare Antworten).
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

    /// Antwort `index` (aus [`Self::choices`]) wählen.
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

/// Zuruf einer Figur (erster passender aus ihrem Gespräch).
pub fn bark<'c>(content: &'c Content, save: &SaveGame, dialog: &str) -> Option<&'c Text> {
    content
        .dialog(dialog)?
        .bark
        .iter()
        .find(|b| cond_ok(save, content, b.cond.as_ref()))
        .map(|b| &b.text)
}

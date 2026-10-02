//! Gespräche und Aufgaben (A1.4) an den mitgelieferten Entwürfen und an kaputten Inhalten.

use elora_adventure::data::Sources;
use elora_adventure::dialog::bark;
use elora_adventure::quest::{Outcome, QuestStatus};
use elora_adventure::state::Notice;
use elora_adventure::{Content, Conversation, Location, SaveGame};
use elora_sim::{Event, Vec2};

fn game() -> (Content, SaveGame) {
    let c = Content::builtin();
    let g = SaveGame::new(
        &c,
        Location {
            map: "tauwinkel".into(),
            spawn: "brunnen".into(),
        },
    );
    (c, g)
}

fn text(c: &Content, conv: &Conversation) -> String {
    conv.current(c).unwrap().1.text.de.clone()
}

#[test]
fn oma_starts_the_well_quest_with_a_choice() {
    let (c, mut g) = game();
    let (mut conv, turn) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert!(turn.open);
    assert!(text(&c, &conv).starts_with("Ach, Elora"));
    assert_eq!(conv.choices(&c, &g), vec![0, 1, 2]);
    let turn = conv.choose(&c, &mut g, 0);
    assert!(
        turn.outcomes
            .contains(&Outcome::Notice(Notice::QuestStarted("brunnen".into())))
    );
    assert!(turn.outcomes.contains(&Outcome::Notice(Notice::Item {
        id: "heiltrank".into(),
        count: 1
    })));
    assert_eq!(g.affection("oma"), 1);
    assert_eq!(g.count("heiltrank"), 1);
    assert!(!conv.advance(&c, &mut g).open, "Ende nach der Zusage");
    // nächstes Gespräch: Erinnerung statt Begrüßung
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "erinnerung");
    assert_eq!(bark(&c, &g, "oma").unwrap().de, "Pass auf dich auf!");
}

#[test]
fn cheeky_answer_is_remembered() {
    let (c, mut g) = game();
    let (mut conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    conv.choose(&c, &mut g, 2);
    assert_eq!(g.flag("oma.frech"), 1);
    assert_eq!(conv.node, "frech");
    conv.advance(&c, &mut g);
    assert_eq!(conv.node, "bitte");
    assert_eq!(g.affection("oma"), 0, "frech gibt keine Zuneigung");
}

#[test]
fn well_quest_runs_through_all_goal_types() {
    let (c, mut g) = game();
    let (mut conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    conv.choose(&c, &mut g, 0);
    // Sprechen: Tüftel
    let (conv, turn) = Conversation::start(&c, &mut g, "tueftel").unwrap();
    // Einstieg wird vor dem Sprechen-Ziel gewählt: Tüftel gibt seinen Rat, der Schritt ist erledigt
    assert_eq!(conv.node, "rat");
    assert!(
        turn.outcomes
            .contains(&Outcome::Notice(Notice::QuestStep("brunnen".into())))
    );
    // Ort erreichen
    assert!(g.on_reach(&c, "wiese-1", Some("falsch")).is_empty());
    g.on_reach(&c, "wiese-1", Some("bruecke"));
    assert!(g.holds(&c, "quest brunnen schritt kaefer"));
    // Gegner besiegen (über Ereignisse der Welt)
    g.location.map = "wiese-1".into();
    let kind = c
        .creatures
        .iter()
        .position(|k| k.name == "stachelkaefer")
        .unwrap();
    for _ in 0..3 {
        let e = Event::CreatureDeath {
            id: 1,
            kind,
            pos: Vec2::ZERO,
            killer: Some(0),
        };
        g.on_event(&c, &c.creatures, 0, &e);
    }
    assert!(g.holds(&c, "quest brunnen schritt quelle"));
    let out = g.on_reach(&c, "wiese-1", Some("quelle"));
    assert!(out.contains(&Outcome::Notice(Notice::QuestDone("brunnen".into()))));
    assert_eq!(g.quest("brunnen").unwrap().status, QuestStatus::Done);
    assert!(g.glanztropfen >= 30, "Belohnung");
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "danach");
}

#[test]
fn side_quest_bring_and_fail() {
    let (c, mut g) = game();
    g.run(&c, &["quest pips_stein start".into()]);
    g.on_talk(&c, "pip");
    assert!(
        g.holds(&c, "quest pips_stein aktiv"),
        "ohne Bernstein nichts abgegeben"
    );
    g.add_item(&c, "bernstein", 3).unwrap();
    g.on_talk(&c, "pip");
    assert!(g.holds(&c, "quest pips_stein erledigt"));
    assert_eq!(g.count("bernstein"), 0, "abgegeben");

    let (c, mut g) = game();
    g.run(&c, &["quest pips_stein start".into()]);
    let out = g.run(&c, &["merker pip.verpetzt = 1".into()]);
    assert!(out.contains(&Outcome::Notice(Notice::QuestFailed("pips_stein".into()))));
    assert!(g.holds(&c, "quest pips_stein gescheitert"));
}

#[test]
fn hidden_choice_until_condition_holds() {
    let (c, mut g) = game();
    let (conv, _) = Conversation::start(&c, &mut g, "tueftel").unwrap();
    assert_eq!(conv.choices(&c, &g), vec![1], "Baum erst ab Stufe 2");
    g.add_xp(&c, 25);
    let (mut conv, _) = Conversation::start(&c, &mut g, "tueftel").unwrap();
    assert_eq!(conv.choices(&c, &g), vec![0, 1]);
    let turn = conv.choose(&c, &mut g, 0);
    assert!(
        turn.outcomes
            .contains(&Outcome::Open(elora_adventure::script::Open::Skills))
    );
}

#[test]
fn affection_gives_discount() {
    let (c, mut g) = game();
    assert_eq!(g.price(&c, "lotte", "strohhut"), Some(200));
    g.run(&c, &["zuneigung lotte +5".into()]);
    assert_eq!(g.price(&c, "lotte", "strohhut"), Some(180));
    g.run(&c, &["zuneigung lotte +20".into()]);
    assert_eq!(g.affection("lotte"), 10, "begrenzt");
    assert_eq!(g.price(&c, "lotte", "strohhut"), Some(160));
}

/// Mitgelieferte Inhalte mit einem ausgetauschten Gespräch laden.
fn load_with_dialog(src: &'static str) -> Result<Content, String> {
    let dialogs: &'static [(&'static str, &'static str)] = Box::leak(Box::new([("test", src)]));
    let s = Sources {
        dialogs,
        ..Sources::builtin()
    };
    Content::load(&s)
        .map(|_| Content::builtin())
        .map_err(|e| e.to_string())
}

#[test]
fn broken_dialogs_are_reported() {
    let base = "speaker = \"oma\"\n[[start]]\nnode = \"a\"\n";
    let cases = [
        (
            "[[node]]\nid = \"a\"\ntext = { de = \"Hallo\", en = \"\" }\n",
            "englischer Text fehlt",
        ),
        (
            "[[node]]\nid = \"a\"\nnext = \"b\"\ntext = { de = \"x\", en = \"x\" }\n",
            "Knoten `b` gibt es nicht",
        ),
        (
            "[[node]]\nid = \"a\"\ntext = { de = \"x\", en = \"x\" }\ndo = [\"quest gibtsnicht start\"]\n",
            "unbekannte Aufgabe",
        ),
        (
            "[[node]]\nid = \"a\"\ntext = { de = \"x\", en = \"x\" }\n[[node.choice]]\nif = \"stufe ungefähr 2\"\ntext = { de = \"x\", en = \"x\" }\n",
            "Vergleich erwartet",
        ),
        (
            "[[node]]\nid = \"a\"\ntext = { de = \"x\", en = \"x\" }\n[[node]]\nid = \"z\"\ntext = { de = \"x\", en = \"x\" }\n",
            "nie erreichbar",
        ),
        (
            "[[node]]\nid = \"a\"\nspeaker = \"niemand\"\ntext = { de = \"x\", en = \"x\" }\n",
            "unbekannte Figur",
        ),
    ];
    for (body, expected) in cases {
        let src: &'static str = Box::leak(format!("{base}{body}").into_boxed_str());
        let e = load_with_dialog(src).expect_err(expected);
        assert!(e.contains(expected), "{expected}: {e}");
    }
    let ok: &'static str = Box::leak(
        format!("{base}[[node]]\nid = \"a\"\ntext = {{ de = \"x\", en = \"x\" }}\n")
            .into_boxed_str(),
    );
    assert!(load_with_dialog(ok).is_ok());
}

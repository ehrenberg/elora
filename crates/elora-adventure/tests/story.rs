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
    assert_eq!(g.affection("oma"), 1);
    assert!(!conv.advance(&c, &mut g).open, "Ende nach der Zusage");
    // nächstes Gespräch: Erinnerung statt Begrüßung
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "erinnerung");
    assert_eq!(bark(&c, &g, "oma").unwrap().de, "Pass auf dich auf!");
}

#[test]
fn lotte_gives_a_potion_once() {
    let (c, mut g) = game();
    let (conv, turn) = Conversation::start(&c, &mut g, "lotte").unwrap();
    assert_eq!(conv.node, "erster");
    assert!(text(&c, &conv).contains("{taste:quick_heal}"));
    assert!(turn.outcomes.contains(&Outcome::Notice(Notice::Item {
        id: "heiltrank".into(),
        count: 1
    })));
    assert_eq!(g.count("heiltrank"), 1);
    let (conv, _) = Conversation::start(&c, &mut g, "lotte").unwrap();
    assert_eq!(conv.node, "laden");
    assert_eq!(g.count("heiltrank"), 1, "nur einmal");
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
    // Sprechen: Klonk mit der Hammer-Übung
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "uebung");
    assert!(g.holds(&c, "quest brunnen schritt wiese"));
    // Karte erreichen
    assert!(g.on_reach(&c, "tauwinkel", None).is_empty());
    g.on_reach(&c, "wiese-1", None);
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
    assert!(g.holds(&c, "quest brunnen schritt wiesenrand"));
    assert!(g.on_reach(&c, "wiese-1", Some("falsch")).is_empty());
    let out = g.on_reach(&c, "wiese-1", Some("wiesenrand"));
    assert!(out.contains(&Outcome::Notice(Notice::QuestDone("brunnen".into()))));
    assert_eq!(g.quest("brunnen").unwrap().status, QuestStatus::Done);
    assert!(g.glanztropfen >= 30, "Belohnung");
    assert!(
        g.holds(&c, "quest bluetenquelle aktiv"),
        "Kapitel 1 beginnt"
    );
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "danach");
}

#[test]
fn side_quest_bring_and_fail() {
    let (c, mut g) = game();
    g.run(&c, &["quest pips_stein start".into()]);
    g.on_talk(&c, "pip");
    assert!(g.holds(&c, "quest pips_stein schritt finden"));
    g.add_item(&c, "glitzerstein", 1).unwrap();
    g.update_quests(&c);
    assert!(g.holds(&c, "quest pips_stein schritt bringen"));
    let (conv, _) = Conversation::start(&c, &mut g, "pip").unwrap();
    assert_eq!(conv.node, "stein_da");
    assert!(g.holds(&c, "quest pips_stein erledigt"));
    assert_eq!(g.count("glitzerstein"), 0, "abgegeben");

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

// ---------------------------------------------------------------- Karten (A1.5)

use elora_adventure::check::{map_links, map_objects};
use elora_map::{Map, Object, ObjectKind};

fn obj(id: &str, kind: ObjectKind) -> Object {
    Object {
        id: id.into(),
        pos: Vec2::new(40.0, 40.0),
        kind,
    }
}

fn small(name: &str, objects: Vec<Object>) -> Map {
    let mut m = Map::from_rows(name, &["#####", "#S..#", "#...#", "#####"]).unwrap();
    m.adventure.objects = objects;
    m
}

#[test]
fn map_objects_are_checked_against_content() {
    let c = Content::builtin();
    let good = small(
        "wiese-1",
        vec![
            obj(
                "kaefer",
                ObjectKind::Creature {
                    kind: "stachelkaefer".into(),
                    persistent: false,
                },
            ),
            obj(
                "oma",
                ObjectKind::Npc {
                    character: "oma".into(),
                    dialog: "oma".into(),
                    facing: 1,
                    walk: 0.0,
                },
            ),
            obj(
                "truhe",
                ObjectKind::Chest {
                    contents: vec![("glanztropfen".into(), 20)],
                    lock: String::new(),
                },
            ),
            obj(
                "stein",
                ObjectKind::Collectible {
                    item: "glitzerstein".into(),
                },
            ),
            obj(
                "tor",
                ObjectKind::Door {
                    size: (1, 1),
                    open_if: "merker tor.wiese".into(),
                },
            ),
        ],
    );
    assert!(
        map_objects(&c, &good).is_empty(),
        "{:?}",
        map_objects(&c, &good)
    );
    let bad = small(
        "wiese-1",
        vec![
            obj(
                "a",
                ObjectKind::Creature {
                    kind: "drache".into(),
                    persistent: false,
                },
            ),
            obj(
                "b",
                ObjectKind::Npc {
                    character: "oma".into(),
                    dialog: "fehlt".into(),
                    facing: 1,
                    walk: 0.0,
                },
            ),
            obj(
                "c",
                ObjectKind::Chest {
                    contents: vec![("gold".into(), 1)],
                    lock: "hat".into(),
                },
            ),
            obj(
                "d",
                ObjectKind::Door {
                    size: (1, 1),
                    open_if: "merker".into(),
                },
            ),
        ],
    );
    let errors = map_objects(&c, &bad);
    assert_eq!(errors.len(), 5, "{errors:?}");
    assert!(
        errors[0].contains("drache") && errors[1].contains("fehlt") && errors[2].contains("gold")
    );
}

#[test]
fn exits_must_lead_to_existing_entrances() {
    let exit = |map: &str, spawn: &str| {
        obj(
            "weg",
            ObjectKind::Exit {
                size: Vec2::new(32.0, 32.0),
                map: map.into(),
                spawn: spawn.into(),
                on_touch: true,
            },
        )
    };
    let a = small("dorf", vec![exit("wiese-1", "west")]);
    let b = small(
        "wiese-1",
        vec![obj("west", ObjectKind::Spawn), exit("dorf", "ost")],
    );
    let errors = map_links(&[("dorf", &a), ("wiese-1", &b)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("Eingang `ost` fehlt"));
    let errors = map_links(&[("dorf", &a)]);
    assert!(errors[0].contains("Zielkarte `wiese-1` fehlt"));
}

/// Kapitel 1 (R2-M2.1): Wabe, Hummel, Quellfunke bei Tüftel, Fest bei Oma, Granatwerfer.
#[test]
fn chapter_one_runs_from_wabe_to_the_party() {
    let (c, mut g) = game();
    g.run(&c, &["quest bluetenquelle start".into()]);
    g.on_reach(&c, "wiese-2", None);
    assert!(g.holds(&c, "quest bluetenquelle schritt wabe"));
    // Wabe: Hauptaufgabe weiter, Nebenaufgabe beginnt
    let (mut conv, _) = Conversation::start(&c, &mut g, "wabe").unwrap();
    assert_eq!(conv.node, "begruessung");
    conv.choose(&c, &mut g, 0);
    assert!(g.holds(&c, "quest wabes_bienen aktiv"));
    assert!(g.holds(&c, "quest bluetenquelle schritt wurzeln"));
    g.on_reach(&c, "wiese-3", None);
    g.on_reach(&c, "wiese-arena", None);
    assert!(g.holds(&c, "quest bluetenquelle schritt hueter"));
    // Hüter besiegt (Merker setzt die Sitzung), Quellfunke als Beute
    g.location.map = "wiese-arena".into();
    g.set_flag("besiegt.brummbaer", 1);
    let kind = c
        .creatures
        .iter()
        .position(|k| k.name == "brummbaer")
        .unwrap();
    g.on_event(
        &c,
        &c.creatures,
        0,
        &Event::CreatureDeath {
            id: 1,
            kind,
            pos: Vec2::ZERO,
            killer: Some(0),
        },
    );
    g.add_item(&c, "quellfunke", 1).unwrap();
    assert!(g.holds(&c, "quest bluetenquelle schritt funke"));
    // die Hummel spricht und deutet auf den Dürren
    let (mut conv, _) = Conversation::start(&c, &mut g, "hummel").unwrap();
    conv.choose(&c, &mut g, 1);
    assert_eq!(g.flag("duerrer.gesehen"), 1);
    // Tüftel baut den Hook-Ruck
    let (conv, _) = Conversation::start(&c, &mut g, "tueftel").unwrap();
    assert_eq!(conv.node, "funke");
    assert!(g.abilities().has(elora_sim::Ability::HookRuck));
    assert_eq!(g.count("quellfunke"), 0, "abgegeben");
    assert_eq!(g.flag("quellen_befreit"), 1);
    assert_eq!(g.flag("fest"), 1);
    assert!(g.holds(&c, "quest bluetenquelle schritt fest"));
    // Fest bei Oma: Kapitel fertig, Kapitel 2 angekündigt
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "fest");
    assert!(g.holds(&c, "quest bluetenquelle erledigt"));
    assert!(g.holds(&c, "quest murmelwald aktiv"));
    // Klonk gibt den Granatwerfer (E-243), nur einmal
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "granate");
    assert!(g.weapons.contains_key(&elora_sim::Weapon::Grenade));
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_ne!(conv.node, "granate");
}

#[test]
fn wabes_bees_give_the_honeycomb_hat() {
    let (c, mut g) = game();
    g.run(&c, &["quest wabes_bienen start".into()]);
    g.add_item(&c, "biene", 4).unwrap();
    g.update_quests(&c);
    assert!(g.holds(&c, "quest wabes_bienen schritt sammeln"));
    g.add_item(&c, "biene", 1).unwrap();
    g.update_quests(&c);
    let (conv, _) = Conversation::start(&c, &mut g, "wabe").unwrap();
    assert_eq!(conv.node, "bienen_da");
    assert!(g.holds(&c, "quest wabes_bienen erledigt"));
    assert_eq!(g.count("wabenhut"), 1);
    assert_eq!(g.count("biene"), 0);
}

/// Kapitel 2 (R2-M2.2): Westhang, Plumm, Wächter, Heranhooken bei Tüftel, Fest, Runen.
#[test]
fn chapter_two_runs_from_the_slope_to_the_party() {
    let (c, mut g) = game();
    g.run(&c, &["quest murmelwald start".into()]);
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "westhang");
    g.on_reach(&c, "wald-1", None);
    let (mut conv, _) = Conversation::start(&c, &mut g, "plumm").unwrap();
    assert_eq!(conv.node, "begruessung");
    assert!(g.holds(&c, "quest murmelwald schritt wurzeln"));
    conv.choose(&c, &mut g, 0);
    conv.choose(&c, &mut g, 0);
    assert!(g.holds(&c, "quest runen aktiv"));
    g.on_reach(&c, "wald-3", None);
    g.on_reach(&c, "wald-arena", None);
    g.location.map = "wald-arena".into();
    g.set_flag("besiegt.wurzelwaechter", 1);
    let kind = c
        .creatures
        .iter()
        .position(|k| k.name == "wurzelwaechter")
        .unwrap();
    g.on_event(
        &c,
        &c.creatures,
        0,
        &Event::CreatureDeath {
            id: 1,
            kind,
            pos: Vec2::ZERO,
            killer: Some(0),
        },
    );
    g.add_item(&c, "quellfunke", 1).unwrap();
    assert!(g.holds(&c, "quest murmelwald schritt funke"));
    let (mut conv, _) = Conversation::start(&c, &mut g, "waechter").unwrap();
    conv.choose(&c, &mut g, 0);
    conv.advance(&c, &mut g);
    assert_eq!(g.flag("befreit.waldquelle"), 1);
    let (conv, _) = Conversation::start(&c, &mut g, "tueftel").unwrap();
    assert_eq!(conv.node, "funke2");
    assert!(g.abilities().has(elora_sim::Ability::Pull));
    assert_eq!(g.flag("quellen_befreit"), 2);
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "fest2");
    assert!(g.holds(&c, "quest murmelwald erledigt"));
    assert!(g.holds(&c, "quest glutsand aktiv"));
    assert_eq!(g.flag("befreit.waldquelle"), 1, "Weltkarte: Quelle befreit");
    // Dorf nach Kapitel 2: Lotte schenkt Pilzsuppe, Klonk spricht vom Harz
    let (conv, _) = Conversation::start(&c, &mut g, "lotte").unwrap();
    assert_eq!(conv.node, "wald");
    assert_eq!(g.count("pilzsuppe"), 1);
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "harz");
    // Runen: Plumm liest vor, Feder und Tautropfen-Punkt (E-309)
    let points = g.bonus_points;
    g.add_item(&c, "rune", 5).unwrap();
    g.update_quests(&c);
    let (conv, _) = Conversation::start(&c, &mut g, "plumm").unwrap();
    assert_eq!(conv.node, "runen_da");
    assert!(g.holds(&c, "quest runen erledigt"));
    assert_eq!(g.count("eulenfeder"), 1);
    assert_eq!(g.bonus_points, points + 1);
    assert_eq!(g.flag("sechste_quelle"), 1);
}

#[test]
fn mushroom_child_quest_ends_with_mama() {
    let (c, mut g) = game();
    let (mut conv, _) = Conversation::start(&c, &mut g, "pilzkind").unwrap();
    conv.choose(&c, &mut g, 0);
    assert!(g.holds(&c, "quest pilzkind aktiv"));
    assert_eq!(g.flag("pilzkind.unterwegs"), 1);
    // die Sitzung setzt den Merker, sobald der Begleiter daheim ist
    g.set_flag("pilzkind.daheim", 1);
    g.update_quests(&c);
    assert!(g.holds(&c, "quest pilzkind schritt danke"));
    let (conv, _) = Conversation::start(&c, &mut g, "pilzmama").unwrap();
    assert_eq!(conv.node, "danke");
    assert!(g.holds(&c, "quest pilzkind erledigt"));
}

#[test]
fn chapter_three_runs_from_the_desert_to_the_party() {
    let (c, mut g) = game();
    g.run(&c, &["quest glutsand start".into()]);
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "glutsand");
    g.on_reach(&c, "wueste-1", None);
    assert!(g.holds(&c, "quest glutsand schritt sirup"));
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "unterwegs");
    // Sirup erzählt vom grauen Wanderer und kennt die Kammer
    let (mut conv, _) = Conversation::start(&c, &mut g, "sirup").unwrap();
    assert_eq!(conv.node, "begruessung");
    assert!(g.holds(&c, "quest glutsand schritt ruinen"));
    conv.choose(&c, &mut g, 0);
    assert_eq!(conv.node, "wanderer");
    conv.advance(&c, &mut g);
    assert_eq!(conv.node, "spuren");
    conv.choose(&c, &mut g, 1);
    assert_eq!(conv.node, "ruine");
    conv.choose(&c, &mut g, 0);
    assert!(g.holds(&c, "quest ruine aktiv"));
    let (conv, _) = Conversation::start(&c, &mut g, "sirup").unwrap();
    assert_eq!(conv.node, "danach", "kennt Elora schon");
    g.on_reach(&c, "wueste-3", None);
    g.on_reach(&c, "wueste-arena", None);
    g.location.map = "wueste-arena".into();
    g.set_flag("besiegt.sandschlange", 1);
    let kind = c
        .creatures
        .iter()
        .position(|k| k.name == "sandschlange")
        .unwrap();
    g.on_event(
        &c,
        &c.creatures,
        0,
        &Event::CreatureDeath {
            id: 1,
            kind,
            pos: Vec2::ZERO,
            killer: Some(0),
        },
    );
    g.add_item(&c, "quellfunke", 1).unwrap();
    assert!(g.holds(&c, "quest glutsand schritt funke"));
    let (mut conv, _) = Conversation::start(&c, &mut g, "schlange").unwrap();
    conv.advance(&c, &mut g);
    assert_eq!(g.flag("befreit.glutquelle"), 1);
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "funke");
    let (conv, _) = Conversation::start(&c, &mut g, "tueftel").unwrap();
    assert_eq!(conv.node, "funke3");
    assert!(g.abilities().has(elora_sim::Ability::Stomp));
    assert_eq!(g.flag("quellen_befreit"), 3);
    assert_eq!(g.count("quellfunke"), 0, "abgegeben");
    let (mut conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "fest3");
    conv.advance(&c, &mut g);
    assert_eq!(conv.node, "fest3_lied");
    assert!(g.holds(&c, "quest glutsand erledigt"));
    assert!(g.holds(&c, "quest frostspitzen aktiv"));
    // Klonk gibt den Laser (E-243)
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "laser");
    assert!(g.weapons.contains_key(&elora_sim::Weapon::Laser));
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_ne!(conv.node, "laser", "nur einmal");
    // Dorf nach Kapitel 3: Lotte schenkt Kaktusfrüchte von Sirup, Pip staunt
    let fruit = g.count("kaktusfrucht");
    let (conv, _) = Conversation::start(&c, &mut g, "lotte").unwrap();
    assert_eq!(conv.node, "wueste");
    assert_eq!(g.count("kaktusfrucht"), fruit + 2);
    assert!(bark(&c, &g, "pip").unwrap().de.contains("Schlange"));
    // Kammer der Ruine: Tafel lesen, Sirup berichten
    g.on_reach(&c, "wueste-3", Some("ruinenkammer"));
    assert!(g.holds(&c, "quest ruine schritt tafel"));
    Conversation::start(&c, &mut g, "tafel-kammer").unwrap();
    assert!(g.holds(&c, "quest ruine schritt bericht"));
    let glanz = g.glanztropfen;
    let (conv, _) = Conversation::start(&c, &mut g, "sirup").unwrap();
    assert_eq!(conv.node, "bericht");
    assert!(g.holds(&c, "quest ruine erledigt"));
    assert_eq!(g.glanztropfen, glanz + 60);
}

#[test]
fn oasis_quest_fills_the_skin_once_and_waters_three_patches() {
    let (c, mut g) = game();
    let (mut conv, _) = Conversation::start(&c, &mut g, "palma").unwrap();
    assert_eq!(conv.node, "begruessung");
    conv.choose(&c, &mut g, 0);
    assert_eq!(conv.node, "auftrag");
    conv.choose(&c, &mut g, 0);
    assert!(g.holds(&c, "quest oase aktiv"));
    assert_eq!(g.count("wasserschlauch"), 1);
    // ohne Wasser bleibt die Stelle trocken
    let (conv, _) = Conversation::start(&c, &mut g, "giessstelle-1").unwrap();
    assert_eq!(conv.node, "trocken");
    let (conv, _) = Conversation::start(&c, &mut g, "ruinenquelle").unwrap();
    assert_eq!(conv.node, "fuellen");
    assert_eq!(g.count("wasser"), 3, "drei Füllungen (E-322)");
    assert!(g.holds(&c, "quest oase schritt giessen"));
    let (conv, _) = Conversation::start(&c, &mut g, "ruinenquelle").unwrap();
    assert_eq!(conv.node, "voll");
    for n in 1..=3 {
        let (conv, _) = Conversation::start(&c, &mut g, &format!("giessstelle-{n}")).unwrap();
        assert_eq!(conv.node, "giessen");
        assert_eq!(g.flag(&format!("befreit.giessstelle-{n}")), 1);
    }
    assert_eq!(g.count("wasser"), 0);
    assert_eq!(g.flag("oase.gegossen"), 3);
    assert!(g.holds(&c, "quest oase schritt danke"));
    let points = g.bonus_points;
    let (conv, _) = Conversation::start(&c, &mut g, "palma").unwrap();
    assert_eq!(conv.node, "danke");
    assert!(g.holds(&c, "quest oase erledigt"));
    assert_eq!(g.count("kaktusfrucht"), 3);
    assert_eq!(g.bonus_points, points + 1);
}

#[test]
fn cactus_fruit_heals_and_sun_veil_slows_the_heat() {
    let (c, mut g) = game();
    g.add_item(&c, "kaktusfrucht", 1).unwrap();
    g.health = 2;
    assert_eq!(
        g.use_item(&c, "kaktusfrucht"),
        Ok(elora_adventure::data::Effect::Cool(3))
    );
    assert_eq!(g.health, 5);
    g.add_item(&c, "sonnenschleier", 1).unwrap();
    g.equip(&c, "sonnenschleier").unwrap();
    assert!((g.stats(&c).heat_pct + 40.0).abs() < 1e-6);
    assert!(c.shops["sirup"].stock.contains(&"kaktusfrucht".to_owned()));
}

/// Playtest 2026-10-06: Wer nach Kapitel 1 nicht bei Klonk war, bekam nach Kapitel 2 nur das
/// Harz-Gespräch und nie den Granatwerfer. Waffen kommen jetzt zuerst.
#[test]
fn klonk_hands_out_weapons_before_talking_about_resin() {
    let (c, mut g) = game();
    g.run(
        &c,
        &[
            "quest bluetenquelle start".into(),
            "quest bluetenquelle fertig".into(),
            "quest murmelwald fertig".into(),
            "quest glutsand fertig".into(),
        ],
    );
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "granate");
    assert!(g.weapons.contains_key(&elora_sim::Weapon::Grenade));
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "laser");
    assert!(g.weapons.contains_key(&elora_sim::Weapon::Laser));
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "harz");
}

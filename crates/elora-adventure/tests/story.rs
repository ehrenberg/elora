//! Dialogs and quests (A1.4) on the bundled drafts and on broken content.

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
            spawn: "well".into(),
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
            .contains(&Outcome::Notice(Notice::QuestStarted("well".into())))
    );
    assert_eq!(g.affection("oma"), 1);
    assert!(!conv.advance(&c, &mut g).open, "ends after the promise");
    // next dialog: reminder instead of greeting
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "memory");
    assert_eq!(bark(&c, &g, "oma").unwrap().de, "Pass auf dich auf!");
}

#[test]
fn lotte_gives_a_potion_once() {
    let (c, mut g) = game();
    let (conv, turn) = Conversation::start(&c, &mut g, "lotte").unwrap();
    assert_eq!(conv.node, "first");
    assert!(text(&c, &conv).contains("{taste:quick_heal}"));
    assert!(turn.outcomes.contains(&Outcome::Notice(Notice::Item {
        id: "healing_potion".into(),
        count: 1
    })));
    assert_eq!(g.count("healing_potion"), 1);
    let (conv, _) = Conversation::start(&c, &mut g, "lotte").unwrap();
    assert_eq!(conv.node, "shop");
    assert_eq!(g.count("healing_potion"), 1, "only once");
}

#[test]
fn cheeky_answer_is_remembered() {
    let (c, mut g) = game();
    let (mut conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    conv.choose(&c, &mut g, 2);
    assert_eq!(g.flag("oma.cheeky"), 1);
    assert_eq!(conv.node, "cheeky");
    conv.advance(&c, &mut g);
    assert_eq!(conv.node, "please");
    assert_eq!(g.affection("oma"), 0, "cheeky gives no affection");
}

#[test]
fn well_quest_runs_through_all_goal_types() {
    let (c, mut g) = game();
    let (mut conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    conv.choose(&c, &mut g, 0);
    // talk: Tüftel
    let (conv, turn) = Conversation::start(&c, &mut g, "tueftel").unwrap();
    // the entry is chosen before the talk goal: Tüftel gives his advice, the step is done
    assert_eq!(conv.node, "advice");
    assert!(
        turn.outcomes
            .contains(&Outcome::Notice(Notice::QuestStep("well".into())))
    );
    // talk: Klonk with the hammer practice
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "practice");
    assert!(g.holds(&c, "quest well step meadow"));
    // reach a map
    assert!(g.on_reach(&c, "tauwinkel", None).is_empty());
    g.on_reach(&c, "meadow-1", None);
    assert!(g.holds(&c, "quest well step beetle"));
    // defeat enemies (via world events)
    g.location.map = "meadow-1".into();
    let kind = c
        .creatures
        .iter()
        .position(|k| k.name == "spike_beetle")
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
    assert!(g.holds(&c, "quest well step meadow_edge"));
    assert!(g.on_reach(&c, "meadow-1", Some("wrong")).is_empty());
    let out = g.on_reach(&c, "meadow-1", Some("meadow_edge"));
    assert!(out.contains(&Outcome::Notice(Notice::QuestDone("well".into()))));
    assert_eq!(g.quest("well").unwrap().status, QuestStatus::Done);
    assert!(g.gleam_drops >= 30, "Belohnung");
    assert!(
        g.holds(&c, "quest blossom_spring active"),
        "chapter 1 begins"
    );
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "after");
}

#[test]
fn side_quest_bring_and_fail() {
    let (c, mut g) = game();
    g.run(&c, &["quest pips_stone start".into()]);
    g.on_talk(&c, "pip");
    assert!(g.holds(&c, "quest pips_stone step find"));
    g.add_item(&c, "glitter_stone", 1).unwrap();
    g.update_quests(&c);
    assert!(g.holds(&c, "quest pips_stone step bring"));
    let (conv, _) = Conversation::start(&c, &mut g, "pip").unwrap();
    assert_eq!(conv.node, "stone_here");
    assert!(g.holds(&c, "quest pips_stone done"));
    assert_eq!(g.count("glitter_stone"), 0, "handed_in");

    let (c, mut g) = game();
    g.run(&c, &["quest pips_stone start".into()]);
    let out = g.run(&c, &["flag pip.told_on = 1".into()]);
    assert!(out.contains(&Outcome::Notice(Notice::QuestFailed("pips_stone".into()))));
    assert!(g.holds(&c, "quest pips_stone failed"));
}

#[test]
fn hidden_choice_until_condition_holds() {
    let (c, mut g) = game();
    let (conv, _) = Conversation::start(&c, &mut g, "tueftel").unwrap();
    assert_eq!(
        conv.choices(&c, &g),
        vec![1],
        "skill tree only from level 2"
    );
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
    assert_eq!(g.price(&c, "lotte", "straw_hat"), Some(200));
    g.run(&c, &["affection lotte +5".into()]);
    assert_eq!(g.price(&c, "lotte", "straw_hat"), Some(180));
    g.run(&c, &["affection lotte +20".into()]);
    assert_eq!(g.affection("lotte"), 10, "limited");
    assert_eq!(g.price(&c, "lotte", "straw_hat"), Some(160));
}

/// Load the bundled content with one dialog replaced.
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
            "English text missing",
        ),
        (
            "[[node]]\nid = \"a\"\nnext = \"b\"\ntext = { de = \"x\", en = \"x\" }\n",
            "node `b` does not exist",
        ),
        (
            "[[node]]\nid = \"a\"\ntext = { de = \"x\", en = \"x\" }\ndo = [\"quest gibtsnicht start\"]\n",
            "unknown quest",
        ),
        (
            "[[node]]\nid = \"a\"\ntext = { de = \"x\", en = \"x\" }\n[[node.choice]]\nif = \"stufe ungefähr 2\"\ntext = { de = \"x\", en = \"x\" }\n",
            "comparison expected",
        ),
        (
            "[[node]]\nid = \"a\"\ntext = { de = \"x\", en = \"x\" }\n[[node]]\nid = \"z\"\ntext = { de = \"x\", en = \"x\" }\n",
            "never reachable",
        ),
        (
            "[[node]]\nid = \"a\"\nspeaker = \"niemand\"\ntext = { de = \"x\", en = \"x\" }\n",
            "unknown character",
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

// ---------------------------------------------------------------- Maps (A1.5)

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
        "meadow-1",
        vec![
            obj(
                "beetle",
                ObjectKind::Creature {
                    kind: "spike_beetle".into(),
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
                "chest",
                ObjectKind::Chest {
                    contents: vec![("gleam_drops".into(), 20)],
                    lock: String::new(),
                },
            ),
            obj(
                "stone",
                ObjectKind::Collectible {
                    item: "glitter_stone".into(),
                },
            ),
            obj(
                "gate",
                ObjectKind::Door {
                    size: (1, 1),
                    open_if: "flag gate.meadow".into(),
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
        "meadow-1",
        vec![
            obj(
                "a",
                ObjectKind::Creature {
                    kind: "dragon".into(),
                    persistent: false,
                },
            ),
            obj(
                "b",
                ObjectKind::Npc {
                    character: "oma".into(),
                    dialog: "missing".into(),
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
        errors[0].contains("dragon") && errors[1].contains("missing") && errors[2].contains("gold")
    );
}

#[test]
fn exits_must_lead_to_existing_entrances() {
    let exit = |map: &str, spawn: &str| {
        obj(
            "path",
            ObjectKind::Exit {
                size: Vec2::new(32.0, 32.0),
                map: map.into(),
                spawn: spawn.into(),
                on_touch: true,
            },
        )
    };
    let a = small("village", vec![exit("meadow-1", "west")]);
    let b = small(
        "meadow-1",
        vec![obj("west", ObjectKind::Spawn), exit("village", "east")],
    );
    let errors = map_links(&[("village", &a), ("meadow-1", &b)]);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("entrance `east` missing"), "{errors:?}");
    let errors = map_links(&[("village", &a)]);
    assert!(
        errors[0].contains("target map `meadow-1` missing"),
        "{errors:?}"
    );
}

/// Chapter 1 (R2-M2.1): honeycomb, bumblebee, spring spark at Tüftel, festival at Oma,
/// grenade launcher.
#[test]
fn chapter_one_runs_from_wabe_to_the_party() {
    let (c, mut g) = game();
    g.run(&c, &["quest blossom_spring start".into()]);
    g.on_reach(&c, "meadow-2", None);
    assert!(g.holds(&c, "quest blossom_spring step wabe"));
    // honeycomb: main quest continues, side quest starts
    let (mut conv, _) = Conversation::start(&c, &mut g, "wabe").unwrap();
    assert_eq!(conv.node, "greeting");
    conv.choose(&c, &mut g, 0);
    assert!(g.holds(&c, "quest wabes_bees active"));
    assert!(g.holds(&c, "quest blossom_spring step roots"));
    g.on_reach(&c, "meadow-3", None);
    g.on_reach(&c, "meadow-arena", None);
    assert!(g.holds(&c, "quest blossom_spring step guardian"));
    // guardian defeated (the session sets the flag), spring spark as loot
    g.location.map = "meadow-arena".into();
    g.set_flag("defeated.bumblebear", 1);
    let kind = c
        .creatures
        .iter()
        .position(|k| k.name == "bumblebear")
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
    g.add_item(&c, "spring_spark", 1).unwrap();
    assert!(g.holds(&c, "quest blossom_spring step spark"));
    // the bumblebee speaks and points to the Dürrer
    let (mut conv, _) = Conversation::start(&c, &mut g, "bumblebee").unwrap();
    conv.choose(&c, &mut g, 1);
    assert_eq!(g.flag("withered_one.seen"), 1);
    // Tüftel builds the hook jerk
    let (conv, _) = Conversation::start(&c, &mut g, "tueftel").unwrap();
    assert_eq!(conv.node, "spark");
    assert!(g.abilities().has(elora_sim::Ability::HookJerk));
    assert_eq!(g.count("spring_spark"), 0, "handed_in");
    assert_eq!(g.flag("springs_freed"), 1);
    assert_eq!(g.flag("party"), 1);
    assert!(g.holds(&c, "quest blossom_spring step party"));
    // festival at Oma: chapter done, chapter 2 announced
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "party");
    assert!(g.holds(&c, "quest blossom_spring done"));
    assert!(g.holds(&c, "quest murmelwald active"));
    // Klonk gives the grenade launcher (E-243), only once
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "grenade");
    assert!(g.weapons.contains_key(&elora_sim::Weapon::Grenade));
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_ne!(conv.node, "grenade");
}

#[test]
fn wabes_bees_give_the_honeycomb_hat() {
    let (c, mut g) = game();
    g.run(&c, &["quest wabes_bees start".into()]);
    g.add_item(&c, "bee", 4).unwrap();
    g.update_quests(&c);
    assert!(g.holds(&c, "quest wabes_bees step collect"));
    g.add_item(&c, "bee", 1).unwrap();
    g.update_quests(&c);
    let (conv, _) = Conversation::start(&c, &mut g, "wabe").unwrap();
    assert_eq!(conv.node, "bees_here");
    assert!(g.holds(&c, "quest wabes_bees done"));
    assert_eq!(g.count("honeycomb_hat"), 1);
    assert_eq!(g.count("bee"), 0);
}

/// Chapter 2 (R2-M2.2): west slope, Plumm, warden, pull hook at Tüftel, festival, runes.
#[test]
fn chapter_two_runs_from_the_slope_to_the_party() {
    let (c, mut g) = game();
    g.run(&c, &["quest murmelwald start".into()]);
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "west_slope");
    g.on_reach(&c, "forest-1", None);
    let (mut conv, _) = Conversation::start(&c, &mut g, "plumm").unwrap();
    assert_eq!(conv.node, "greeting");
    assert!(g.holds(&c, "quest murmelwald step roots"));
    conv.choose(&c, &mut g, 0);
    conv.choose(&c, &mut g, 0);
    assert!(g.holds(&c, "quest runes active"));
    g.on_reach(&c, "forest-3", None);
    g.on_reach(&c, "forest-arena", None);
    g.location.map = "forest-arena".into();
    g.set_flag("defeated.root_warden", 1);
    let kind = c
        .creatures
        .iter()
        .position(|k| k.name == "root_warden")
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
    g.add_item(&c, "spring_spark", 1).unwrap();
    assert!(g.holds(&c, "quest murmelwald step spark"));
    let (mut conv, _) = Conversation::start(&c, &mut g, "warden").unwrap();
    conv.choose(&c, &mut g, 0);
    conv.advance(&c, &mut g);
    assert_eq!(g.flag("freed.moss_spring"), 1);
    let (conv, _) = Conversation::start(&c, &mut g, "tueftel").unwrap();
    assert_eq!(conv.node, "spark2");
    assert!(g.abilities().has(elora_sim::Ability::Pull));
    assert_eq!(g.flag("springs_freed"), 2);
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "party2");
    assert!(g.holds(&c, "quest murmelwald done"));
    assert!(g.holds(&c, "quest glutsand active"));
    assert_eq!(g.flag("freed.moss_spring"), 1, "world map: spring freed");
    // village after chapter 2: Lotte gives mushroom soup, Klonk talks about the resin
    let (conv, _) = Conversation::start(&c, &mut g, "lotte").unwrap();
    assert_eq!(conv.node, "forest");
    assert_eq!(g.count("mushroom_soup"), 1);
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "resin");
    // runes: Plumm reads aloud, feather and dewdrop point (E-309)
    let points = g.bonus_points;
    g.add_item(&c, "rune", 5).unwrap();
    g.update_quests(&c);
    let (conv, _) = Conversation::start(&c, &mut g, "plumm").unwrap();
    assert_eq!(conv.node, "runes_here");
    assert!(g.holds(&c, "quest runes done"));
    assert_eq!(g.count("owl_feather"), 1);
    assert_eq!(g.bonus_points, points + 1);
    assert_eq!(g.flag("sixth_spring"), 1);
}

#[test]
fn mushroom_child_quest_ends_with_mama() {
    let (c, mut g) = game();
    let (mut conv, _) = Conversation::start(&c, &mut g, "mushroom_child").unwrap();
    conv.choose(&c, &mut g, 0);
    assert!(g.holds(&c, "quest mushroom_child active"));
    assert_eq!(g.flag("mushroom_child.on_the_way"), 1);
    // the session sets the flag as soon as the companion is home
    g.set_flag("mushroom_child.home", 1);
    g.update_quests(&c);
    assert!(g.holds(&c, "quest mushroom_child step thanks"));
    let (conv, _) = Conversation::start(&c, &mut g, "mushroom_mama").unwrap();
    assert_eq!(conv.node, "thanks");
    assert!(g.holds(&c, "quest mushroom_child done"));
}

#[test]
fn chapter_three_runs_from_the_desert_to_the_party() {
    let (c, mut g) = game();
    g.run(&c, &["quest glutsand start".into()]);
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "glutsand");
    g.on_reach(&c, "desert-1", None);
    assert!(g.holds(&c, "quest glutsand step sirup"));
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "on_the_way");
    // Sirup tells about the grey wanderer and knows the chamber
    let (mut conv, _) = Conversation::start(&c, &mut g, "sirup").unwrap();
    assert_eq!(conv.node, "greeting");
    assert!(g.holds(&c, "quest glutsand step ruins"));
    conv.choose(&c, &mut g, 0);
    assert_eq!(conv.node, "wanderer");
    conv.advance(&c, &mut g);
    assert_eq!(conv.node, "tracks");
    conv.choose(&c, &mut g, 1);
    assert_eq!(conv.node, "ruin");
    conv.choose(&c, &mut g, 0);
    assert!(g.holds(&c, "quest ruin active"));
    let (conv, _) = Conversation::start(&c, &mut g, "sirup").unwrap();
    assert_eq!(conv.node, "after", "already knows Elora");
    g.on_reach(&c, "desert-3", None);
    g.on_reach(&c, "desert-arena", None);
    g.location.map = "desert-arena".into();
    g.set_flag("defeated.sand_serpent", 1);
    let kind = c
        .creatures
        .iter()
        .position(|k| k.name == "sand_serpent")
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
    g.add_item(&c, "spring_spark", 1).unwrap();
    assert!(g.holds(&c, "quest glutsand step spark"));
    let (mut conv, _) = Conversation::start(&c, &mut g, "serpent").unwrap();
    conv.advance(&c, &mut g);
    assert_eq!(g.flag("freed.ember_spring"), 1);
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "spark");
    let (conv, _) = Conversation::start(&c, &mut g, "tueftel").unwrap();
    assert_eq!(conv.node, "spark3");
    assert!(g.abilities().has(elora_sim::Ability::Stomp));
    assert_eq!(g.flag("springs_freed"), 3);
    assert_eq!(g.count("spring_spark"), 0, "handed_in");
    let (mut conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "party3");
    conv.advance(&c, &mut g);
    assert_eq!(conv.node, "party3_song");
    assert!(g.holds(&c, "quest glutsand done"));
    assert!(g.holds(&c, "quest frostspitzen active"));
    // Klonk gives the laser (E-243)
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "laser");
    assert!(g.weapons.contains_key(&elora_sim::Weapon::Laser));
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_ne!(conv.node, "laser", "only once");
    // village after chapter 3: Lotte gives cactus fruits from Sirup, Pip is amazed
    let fruit = g.count("cactus_fruit");
    let (conv, _) = Conversation::start(&c, &mut g, "lotte").unwrap();
    assert_eq!(conv.node, "desert");
    assert_eq!(g.count("cactus_fruit"), fruit + 2);
    assert!(bark(&c, &g, "pip").unwrap().de.contains("Schlange"));
    // chamber of the ruin: read the tablet, report to Sirup
    g.on_reach(&c, "desert-3", Some("ruin_chamber"));
    assert!(g.holds(&c, "quest ruin step tablet"));
    Conversation::start(&c, &mut g, "tablet-chamber").unwrap();
    assert!(g.holds(&c, "quest ruin step report"));
    let gleam = g.gleam_drops;
    let (conv, _) = Conversation::start(&c, &mut g, "sirup").unwrap();
    assert_eq!(conv.node, "report");
    assert!(g.holds(&c, "quest ruin done"));
    assert_eq!(g.gleam_drops, gleam + 60);
}

#[test]
fn oasis_quest_fills_the_skin_once_and_waters_three_patches() {
    let (c, mut g) = game();
    let (mut conv, _) = Conversation::start(&c, &mut g, "palma").unwrap();
    assert_eq!(conv.node, "greeting");
    conv.choose(&c, &mut g, 0);
    assert_eq!(conv.node, "task");
    conv.choose(&c, &mut g, 0);
    assert!(g.holds(&c, "quest oasis active"));
    assert_eq!(g.count("waterskin"), 1);
    // without water the spot stays dry
    let (conv, _) = Conversation::start(&c, &mut g, "watering-spot-1").unwrap();
    assert_eq!(conv.node, "dry");
    let (conv, _) = Conversation::start(&c, &mut g, "ruin_spring").unwrap();
    assert_eq!(conv.node, "fill");
    assert_eq!(g.count("water"), 3, "three fillings (E-322)");
    assert!(g.holds(&c, "quest oasis step water"));
    let (conv, _) = Conversation::start(&c, &mut g, "ruin_spring").unwrap();
    assert_eq!(conv.node, "full");
    for n in 1..=3 {
        let (conv, _) = Conversation::start(&c, &mut g, &format!("watering-spot-{n}")).unwrap();
        assert_eq!(conv.node, "water");
        assert_eq!(g.flag(&format!("freed.watering-spot-{n}")), 1);
    }
    assert_eq!(g.count("water"), 0);
    assert_eq!(g.flag("oasis.watered"), 3);
    assert!(g.holds(&c, "quest oasis step thanks"));
    let points = g.bonus_points;
    let (conv, _) = Conversation::start(&c, &mut g, "palma").unwrap();
    assert_eq!(conv.node, "thanks");
    assert!(g.holds(&c, "quest oasis done"));
    assert_eq!(g.count("cactus_fruit"), 3);
    assert_eq!(g.bonus_points, points + 1);
}

#[test]
fn cactus_fruit_heals_and_sun_veil_slows_the_heat() {
    let (c, mut g) = game();
    g.add_item(&c, "cactus_fruit", 1).unwrap();
    g.health = 2;
    assert_eq!(
        g.use_item(&c, "cactus_fruit"),
        Ok(elora_adventure::data::Effect::Cool(3))
    );
    assert_eq!(g.health, 5);
    g.add_item(&c, "sun_veil", 1).unwrap();
    g.equip(&c, "sun_veil").unwrap();
    assert!((g.stats(&c).heat_pct + 40.0).abs() < 1e-6);
    assert!(c.shops["sirup"].stock.contains(&"cactus_fruit".to_owned()));
}

/// Playtest 2026-10-06: whoever had not visited Klonk after chapter 1 only got the resin
/// dialog after chapter 2 and never the grenade launcher. Weapons now come first.
#[test]
fn klonk_hands_out_weapons_before_talking_about_resin() {
    let (c, mut g) = game();
    g.run(
        &c,
        &[
            "quest blossom_spring start".into(),
            "quest blossom_spring finish".into(),
            "quest murmelwald finish".into(),
            "quest glutsand finish".into(),
        ],
    );
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "grenade");
    assert!(g.weapons.contains_key(&elora_sim::Weapon::Grenade));
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "laser");
    assert!(g.weapons.contains_key(&elora_sim::Weapon::Laser));
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "resin");
}

#[test]
fn chapter_four_runs_from_the_mountain_path_to_the_party() {
    let (c, mut g) = game();
    g.run(&c, &["quest frostspitzen start".into()]);
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "frost");
    g.on_reach(&c, "frost-1", None);
    assert!(g.holds(&c, "quest frostspitzen step flocke"));
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "on_the_way4");
    // Flocke: greeting, then the rope
    let (mut conv, _) = Conversation::start(&c, &mut g, "flocke").unwrap();
    assert_eq!(conv.node, "greeting");
    assert!(g.holds(&c, "quest frostspitzen step rope"));
    conv.choose(&c, &mut g, 0);
    assert_eq!(conv.node, "rope");
    let (conv, _) = Conversation::start(&c, &mut g, "flocke").unwrap();
    assert_eq!(conv.node, "memory");
    // rope from the cellar: climbing claws (ice grip) in the middle of the chapter (E-340)
    g.add_item(&c, "rope", 1).unwrap();
    let (mut conv, _) = Conversation::start(&c, &mut g, "flocke").unwrap();
    assert_eq!(conv.node, "rope_back");
    assert!(g.abilities().has(elora_sim::Ability::Grip));
    assert_eq!(g.count("rope"), 0, "handed_in");
    assert!(g.holds(&c, "quest frostspitzen step ridge"));
    conv.choose(&c, &mut g, 0);
    assert_eq!(conv.node, "climbers");
    conv.choose(&c, &mut g, 0);
    assert!(g.holds(&c, "quest climbers active"));
    g.on_reach(&c, "frost-3", None);
    g.on_reach(&c, "frost-arena", None);
    g.location.map = "frost-arena".into();
    g.set_flag("defeated.kristella", 1);
    let kind = c
        .creatures
        .iter()
        .position(|k| k.name == "kristella")
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
    g.add_item(&c, "spring_spark", 1).unwrap();
    assert!(g.holds(&c, "quest frostspitzen step spark"));
    let (mut conv, _) = Conversation::start(&c, &mut g, "kristella").unwrap();
    assert!(text(&c, &conv).contains("Genug"));
    conv.advance(&c, &mut g);
    assert!(text(&c, &conv).contains("einsam, nicht böse"));
    conv.advance(&c, &mut g);
    assert_eq!(g.flag("freed.frost_spring"), 1);
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "spark");
    let (conv, _) = Conversation::start(&c, &mut g, "tueftel").unwrap();
    assert_eq!(conv.node, "spark4");
    assert_eq!(g.flag("ice_grip.strong"), 1);
    assert_eq!(g.flag("springs_freed"), 4);
    let (mut conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "party4");
    conv.advance(&c, &mut g);
    assert_eq!(conv.node, "party4_song");
    assert!(g.holds(&c, "quest frostspitzen done"));
    assert!(g.holds(&c, "quest sternschlucht active"));
    let (conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    assert_eq!(conv.node, "star");
    // village after chapter 4: calls, herbal tea at Lotte, strengthened ice grip (D-M24-03)
    assert!(bark(&c, &g, "pip").unwrap().de.contains("Kristella"));
    assert!(bark(&c, &g, "lotte").unwrap().de.contains("Kräutertee"));
    assert!(bark(&c, &g, "tueftel").unwrap().de.contains("Steigkrallen"));
    let base = elora_sim::Tuning::default();
    let t = g.tuning(&c, &base);
    assert_eq!(t.grip_time, base.grip_time * 2);
    assert!(t.grip_climb > 0.0 && base.grip_climb.abs() < f32::EPSILON);
}

#[test]
fn lost_climbers_go_home_and_flocke_gives_the_bobble_hat() {
    let (c, mut g) = game();
    g.run(&c, &["quest climbers start".into()]);
    for who in ["bolle", "kiesel", "wicke"] {
        assert!(g.holds(&c, &format!("nicht merker kletterer.{who}")));
        Conversation::start(&c, &mut g, who).unwrap();
        assert!(g.holds(&c, &format!("merker kletterer.{who}")));
    }
    assert!(g.holds(&c, "quest climbers step report"));
    let (conv, _) = Conversation::start(&c, &mut g, "flocke").unwrap();
    assert_eq!(conv.node, "report");
    assert!(g.holds(&c, "quest climbers done"));
    assert_eq!(g.count("bobble_hat"), 1);
    assert!(bark(&c, &g, "wicke-hut").is_some());
}

#[test]
fn klonk_cuts_one_crystal_pendant_of_your_choice() {
    let (c, mut g) = game();
    g.run(&c, &["quest frostspitzen start".into()]);
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "crystals");
    assert!(g.holds(&c, "quest clear_crystals active"));
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "crystals_memory");
    g.add_item(&c, "clear_crystal", 8).unwrap();
    let _ = g.update_quests(&c);
    assert!(g.holds(&c, "quest clear_crystals step bring"));
    let (mut conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "crystals_here");
    assert_eq!(g.count("clear_crystal"), 0, "handed_in");
    conv.choose(&c, &mut g, 2);
    assert_eq!(g.count("light_crystal"), 1);
    assert_eq!(g.count("force_crystal") + g.count("blast_crystal"), 0);
    assert!(g.holds(&c, "quest clear_crystals done"));
}

#[test]
fn herbal_tea_heals_and_the_bobble_hat_slows_the_cold() {
    let (c, mut g) = game();
    g.health = 3;
    g.add_item(&c, "herbal_tea", 1).unwrap();
    let effect = g.use_item(&c, "herbal_tea").unwrap();
    assert_eq!(effect, elora_adventure::data::Effect::Warm(2));
    assert_eq!(g.health, 5);
    g.add_item(&c, "bobble_hat", 1).unwrap();
    g.equip(&c, "bobble_hat").unwrap();
    assert!((g.stats(&c).cold_pct + 40.0).abs() < f32::EPSILON);
}

/// Signposts only explain what Elora can already do (E-354): before that they say "later"
/// or what is still missing.
#[test]
fn signposts_explain_only_what_elora_already_has() {
    for (sign, unlock) in [
        ("sign-jerk", "ability hook-jerk"),
        ("sign-pull", "ability pull_hook"),
        ("sign-stomp", "ability stomp"),
        ("sign-chimney", "ability ice_grip"),
        ("sign-hammer", "weapon hammer"),
        ("sign-mountain-path", "ability stomp"),
        ("sign-desert", "quest glutsand start"),
    ] {
        let (c, mut g) = game();
        let (talk, _) = Conversation::start(&c, &mut g, sign).expect("sign");
        assert_eq!(talk.node, "later", "{sign} before");
        g.run(&c, &[unlock.into()]);
        let (talk, _) = Conversation::start(&c, &mut g, sign).expect("sign");
        assert_eq!(talk.node, "text", "{sign} after `{unlock}`");
    }
}

#[test]
fn klonk_sends_elora_to_tueftel_before_the_hammer() {
    let (c, mut g) = game();
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "first_tueftel", "no hammer, not sent yet");
    // Oma, then Tüftel: now Klonk is on the step and hands out the hammer
    let (mut conv, _) = Conversation::start(&c, &mut g, "oma").unwrap();
    conv.choose(&c, &mut g, 0);
    Conversation::start(&c, &mut g, "tueftel").unwrap();
    assert!(g.holds(&c, "quest well step klonk"));
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "practice");
    let (conv, _) = Conversation::start(&c, &mut g, "klonk").unwrap();
    assert_eq!(conv.node, "smithy", "with the hammer back to the forge");
}

#[test]
fn buried_chamber_sign_waits_for_the_stomp() {
    let (c, mut g) = game();
    let (talk, _) = Conversation::start(&c, &mut g, "sign-chamber").expect("sign");
    assert_eq!(talk.node, "later");
    g.run(&c, &["ability stomp".into()]);
    let (talk, _) = Conversation::start(&c, &mut g, "sign-chamber").expect("sign");
    assert_eq!(talk.node, "text");
}

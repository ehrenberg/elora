//! Durchlauf des Prologs (A1.9/A1.10) auf den mitgelieferten Karten `maps/abenteuer/`:
//! Gespräche, Aufgaben, Übergänge, Sammeln, Speichern und Fortsetzen.

use elora_adventure::quest::QuestStatus;
use elora_adventure::{Content, Conversation, Session, SessionEvent, save};
use elora_map::Map;
use elora_sim::{Event, PlayerInput, Tuning, Vec2, World};

fn load(name: &str) -> Map {
    let path = format!(
        "{}/../../maps/abenteuer/{name}.emap",
        env!("CARGO_MANIFEST_DIR")
    );
    elora_map::decode(&std::fs::read(path).unwrap()).unwrap()
}

fn step(s: &mut Session, w: &mut World, input: PlayerInput, interact: bool) -> Vec<SessionEvent> {
    w.step(&[input]);
    s.tick(w, interact)
}

/// Elora neben ein Objekt stellen und kurz landen lassen.
fn go_to(s: &mut Session, w: &mut World, id: &str, dx: f32) {
    let o = s
        .map
        .adventure
        .object(id)
        .unwrap_or_else(|| panic!("{id} fehlt"));
    let pos = o.kind.area().map_or(o.pos, |a| o.pos + a * 0.5);
    w.spawn_character(s.player, pos + Vec2::new(dx, -4.0));
    for _ in 0..30 {
        step(s, w, PlayerInput::default(), false);
    }
}

/// Mit einer Figur sprechen und immer die erste Antwort wählen; liefert die Knoten.
fn talk(s: &mut Session, w: &mut World, npc: &str) -> Vec<String> {
    go_to(s, w, npc, -36.0);
    // Figuren mit Laufweg: an ihre aktuelle Stelle
    let at = s.anchor(npc).unwrap();
    w.spawn_character(s.player, at + Vec2::new(-20.0, -4.0));
    let ev = step(s, w, PlayerInput::default(), true);
    let dialog = ev
        .iter()
        .find_map(|e| match e {
            SessionEvent::Talk { dialog, .. } => Some(dialog.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("kein Gespräch mit {npc}: {ev:?}"));
    let content = s.content.clone();
    let (mut conv, mut turn) = Conversation::start(&content, &mut s.save, &dialog).unwrap();
    let mut nodes = vec![conv.node.clone()];
    while turn.open {
        turn = if conv.choices(&content, &s.save).is_empty() {
            conv.advance(&content, &mut s.save)
        } else {
            let first = conv.choices(&content, &s.save)[0];
            conv.choose(&content, &mut s.save, first)
        };
        nodes.push(conv.node.clone());
    }
    nodes
}

fn walk_until_travel(s: &mut Session, w: &mut World, direction: i8) -> (String, String) {
    for _ in 0..400 {
        let input = PlayerInput {
            direction,
            ..PlayerInput::default()
        };
        for e in step(s, w, input, false) {
            if let SessionEvent::Travel { map, spawn } = e {
                return (map, spawn);
            }
        }
    }
    panic!("kein Übergang");
}

fn holds(s: &Session, cond: &str) -> bool {
    s.save.holds(&s.content, cond)
}

#[test]
fn prologue_from_start_to_the_meadow_edge_and_back() {
    let tuning = Tuning::default();
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("tauwinkel", load("tauwinkel"), "start", &tuning);

    // Pip weckt Elora, Oma gibt die Hauptaufgabe
    assert_eq!(talk(&mut s, &mut w, "pip")[0], "wecken");
    talk(&mut s, &mut w, "oma");
    assert!(holds(&s, "quest brunnen schritt tueftel"));
    // Wegweiser lesen
    talk(&mut s, &mut w, "schild-start");
    // Tüftel, Klonk, Lotte
    assert_eq!(talk(&mut s, &mut w, "tueftel")[0], "rat");
    assert_eq!(talk(&mut s, &mut w, "klonk")[0], "uebung");
    assert!(holds(&s, "quest brunnen schritt wiese"));
    assert_eq!(talk(&mut s, &mut w, "lotte")[0], "erster");
    assert_eq!(s.save.count("heiltrank"), 1);
    // Pips Nebenaufgabe
    talk(&mut s, &mut w, "pip");
    assert!(holds(&s, "quest pips_stein aktiv"));

    // Ostpfad: Übergang beim Hineinlaufen
    go_to(&mut s, &mut w, "ost", 0.0);
    let (map, spawn) = walk_until_travel(&mut s, &mut w, 1);
    assert_eq!((map.as_str(), spawn.as_str()), ("wiese-1", "west"));
    let mut w = s.enter(&map, load(&map), &spawn, &tuning);
    assert!(holds(&s, "quest brunnen schritt kaefer"));

    // drei Stachelkäfer
    let content = s.content.clone();
    let kind = content
        .creatures
        .iter()
        .position(|k| k.name == "stachelkaefer")
        .unwrap();
    for id in 0..3 {
        let e = Event::CreatureDeath {
            id,
            kind,
            pos: Vec2::ZERO,
            killer: Some(s.player),
        };
        s.save.on_event(&content, &content.creatures, s.player, &e);
    }
    assert!(holds(&s, "quest brunnen schritt wiesenrand"));

    // Glitzerstein auf dem Plateau
    go_to(&mut s, &mut w, "glitzerstein", 0.0);
    assert_eq!(s.save.count("glitzerstein"), 1);
    assert!(holds(&s, "quest pips_stein schritt bringen"));

    // Wiesenrand: Aufgabe fertig, Kapitel 1 beginnt
    go_to(&mut s, &mut w, "wiesenrand", 0.0);
    assert_eq!(s.save.quest("brunnen").unwrap().status, QuestStatus::Done);
    assert!(holds(&s, "quest bluetenquelle aktiv"));

    // Quellstein: speichern, Spielstand prüfen und fortsetzen
    go_to(&mut s, &mut w, "quellstein", -36.0);
    let ev = step(&mut s, &mut w, PlayerInput::default(), true);
    assert!(ev.contains(&SessionEvent::Save), "{ev:?}");
    let file = save::encode(&s.save);
    let loaded = save::decode(&file).unwrap();
    assert_eq!(loaded, s.save);
    assert_eq!(loaded.location.map, "wiese-1");
    assert_eq!(loaded.location.spawn, "quellstein");

    let mut s = Session::new(Content::builtin(), loaded);
    let loc = s.save.location.clone();
    let mut w = s.enter(&loc.map, load(&loc.map), &loc.spawn, &tuning);
    assert!(w.character(s.player).is_some());

    // zurück nach Tauwinkel und Pip den Stein bringen
    go_to(&mut s, &mut w, "west", 0.0);
    let (map, spawn) = walk_until_travel(&mut s, &mut w, -1);
    assert_eq!((map.as_str(), spawn.as_str()), ("tauwinkel", "ost"));
    let mut w = s.enter(&map, load(&map), &spawn, &tuning);
    assert_eq!(talk(&mut s, &mut w, "pip")[0], "stein_da");
    assert!(holds(&s, "quest pips_stein erledigt"));
}

/// Zugschalter in `wald-3` mit einem echten Hook-Schuss nach oben (R2-M2.2).
#[test]
fn pull_lever_in_the_root_caves_opens_the_chamber() {
    let tuning = Tuning::default();
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("wald-3", load("wald-3"), "ost", &tuning);
    s.save
        .run(&s.content.clone(), &["faehigkeit heranhooken".into()]);
    s.sync_world(&mut w);
    let lever = s.map.adventure.object("zug").unwrap().pos;
    // Elora unter den Schalter auf den Höhlenboden
    w.spawn_character(s.player, Vec2::new(lever.x, lever.y + 200.0));
    for _ in 0..40 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    let me = w.character(s.player).unwrap().core.pos;
    let aim = lever - me;
    let mut events = Vec::new();
    for _ in 0..40 {
        let input = PlayerInput {
            hook: true,
            target_x: aim.x as i32,
            target_y: aim.y as i32,
            ..PlayerInput::default()
        };
        events.extend(step(&mut s, &mut w, input, false));
    }
    assert_eq!(s.save.flag("zug.wald3"), 1, "Schalter umgelegt: {events:?}");
}

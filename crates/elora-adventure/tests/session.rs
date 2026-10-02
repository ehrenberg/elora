//! Abenteuer-Sitzung (A1.6) auf den Test-Karten (`fixture`): Truhen, Schalter und Türen, Speicherpunkt,
//! Übergänge, Zonen, Tod.

use elora_adventure::session::Prompt;
use elora_adventure::state::Notice;
use elora_adventure::{Content, Session, SessionEvent};
use elora_sim::{PlayerInput, Tile, Tuning, Vec2, World};

mod fixture;

use fixture::load;

fn start() -> (Session, World) {
    let mut s = Session::new_game(Content::builtin());
    let loc = s.save.location.clone();
    let w = s.enter(&loc.map, load(&loc.map), &loc.spawn, &Tuning::default());
    (s, w)
}

fn step(s: &mut Session, w: &mut World, input: PlayerInput, interact: bool) -> Vec<SessionEvent> {
    w.step(&[input]);
    s.tick(w, interact)
}

/// Elora an ein Objekt stellen und ein paar Ticks stehen lassen.
fn go_to(s: &mut Session, w: &mut World, id: &str) {
    let pos = s.map.adventure.object(id).unwrap().pos;
    let target = s
        .map
        .adventure
        .object(id)
        .unwrap()
        .kind
        .area()
        .map_or(pos, |a| pos + a * 0.5);
    w.spawn_character(s.player, Vec2::new(target.x, pos.y.max(target.y)));
    for _ in 0..10 {
        step(s, w, PlayerInput::default(), false);
    }
}

#[test]
fn new_game_starts_in_tauwinkel() {
    let (s, w) = start();
    assert_eq!(s.map_name, "tauwinkel");
    let ch = w.character(s.player).unwrap();
    assert_eq!(ch.health, 10);
    assert!(w.adventure && !w.creature_kinds.is_empty());
    // Tor ist zu
    let door = s.map.adventure.object("tor").unwrap().pos;
    assert_eq!(
        w.collision.tile_at(door + Vec2::new(16.0, 16.0)),
        Tile::Unhookable
    );
}

#[test]
fn chest_switch_and_door() {
    let (mut s, mut w) = start();
    go_to(&mut s, &mut w, "truhe-1");
    assert_eq!(
        s.interactable(w.character(s.player).unwrap().core.pos)
            .map(|x| x.1),
        Some(Prompt::Open)
    );
    let ev = step(&mut s, &mut w, PlayerInput::default(), true);
    assert!(ev.contains(&SessionEvent::Notice(Notice::Item {
        id: "glanztropfen".into(),
        count: 20
    })));
    assert_eq!(s.save.glanztropfen, 20);
    assert!(s.object_done("truhe-1"));
    // zweites Öffnen nicht möglich
    assert!(step(&mut s, &mut w, PlayerInput::default(), true).is_empty());

    go_to(&mut s, &mut w, "hebel");
    let ev = step(&mut s, &mut w, PlayerInput::default(), true);
    assert!(
        ev.contains(&SessionEvent::TilesChanged),
        "Tor öffnet sich: {ev:?}"
    );
    let door = s.map.adventure.object("tor").unwrap().pos;
    assert_eq!(w.collision.tile_at(door + Vec2::new(16.0, 16.0)), Tile::Air);
    // Hebel zurück: Tor bleibt offen (E-254)
    step(&mut s, &mut w, PlayerInput::default(), true);
    assert_eq!(s.save.flag("tor.dorf"), 0);
    assert_eq!(w.collision.tile_at(door + Vec2::new(16.0, 16.0)), Tile::Air);
}

#[test]
fn save_point_rests_and_asks_to_save() {
    let (mut s, mut w) = start();
    w.character_mut(s.player).unwrap().health = 3;
    go_to(&mut s, &mut w, "brunnen");
    let ev = step(&mut s, &mut w, PlayerInput::default(), true);
    assert!(ev.contains(&SessionEvent::Save));
    assert_eq!(w.character(s.player).unwrap().health, 10);
    assert_eq!(s.save.location.spawn, "brunnen");
}

#[test]
fn walking_into_the_exit_travels_and_zones_count() {
    let (mut s, mut w) = start();
    s.save.run(
        &s.content.clone(),
        &[
            "quest brunnen start".into(),
            "quest brunnen weiter".into(),
            "quest brunnen weiter".into(),
        ],
    );
    let exit = s.map.adventure.object("weg-wiese").unwrap().pos;
    w.spawn_character(s.player, exit + Vec2::new(-40.0, 150.0));
    let mut travel = None;
    for _ in 0..80 {
        for e in step(
            &mut s,
            &mut w,
            PlayerInput {
                direction: 1,
                ..PlayerInput::default()
            },
            false,
        ) {
            if let SessionEvent::Travel { map, spawn } = e {
                travel = Some((map, spawn));
            }
        }
    }
    let (map, spawn) = travel.expect("Übergang beim Hineinlaufen");
    assert_eq!((map.as_str(), spawn.as_str()), ("wiese-1", "west"));
    let mut w = s.enter(&map, load(&map), &spawn, &Tuning::default());
    assert_eq!(s.save.location.map, "wiese-1");
    assert_eq!(w.creatures.len(), 5);
    assert!(s.save.holds(&s.content, "quest brunnen schritt kaefer"));
    s.save
        .run(&s.content.clone(), &["quest brunnen weiter".into()]);
    // Zone „Wiesenrand“ erreichen: Aufgabe fertig, die nächste beginnt
    let zone = s.map.adventure.object("wiesenrand").unwrap().pos;
    w.spawn_character(s.player, zone + Vec2::new(100.0, 200.0));
    let mut notes = Vec::new();
    for _ in 0..3 {
        notes.extend(step(&mut s, &mut w, PlayerInput::default(), false));
    }
    assert!(
        s.save.holds(&s.content, "quest brunnen erledigt"),
        "{notes:?}"
    );
    assert!(s.save.holds(&s.content, "quest bluetenquelle aktiv"));
}

#[test]
fn death_loses_some_glanz_and_marks_dead() {
    let (mut s, mut w) = start();
    let c = s.content.clone();
    s.save.add_item(&c, "glanztropfen", 40).unwrap();
    w.die(s.player, None, elora_sim::DeathCause::World);
    w.events.push(elora_sim::Event::Death {
        player: s.player,
        killer: None,
        cause: elora_sim::DeathCause::World,
        pos: Vec2::ZERO,
    });
    let ev = s.tick(&mut w, false);
    assert!(ev.contains(&SessionEvent::Died { lost: 10 }), "{ev:?}");
    assert!(s.dead);
    assert_eq!(s.save.glanztropfen, 30);
}

#[test]
fn locked_chest_needs_condition() {
    let (mut s, _) = start();
    let mut w = s.enter("wiese-1", load("wiese-1"), "west", &Tuning::default());
    go_to(&mut s, &mut w, "truhe-quelle");
    let ev = step(&mut s, &mut w, PlayerInput::default(), true);
    assert_eq!(
        ev,
        vec![SessionEvent::Locked {
            object: "truhe-quelle".into()
        }]
    );
    let c = s.content.clone();
    s.save.add_item(&c, "glitzerstein", 1).unwrap();
    let ev = step(&mut s, &mut w, PlayerInput::default(), true);
    assert!(ev.contains(&SessionEvent::Notice(Notice::Item {
        id: "bernstein".into(),
        count: 3
    })));
}

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

#[test]
fn defeated_boss_sets_a_flag() {
    let (mut s, mut w) = start();
    let kind = w
        .creature_kind("brummbaer")
        .expect("Hüter in creatures.toml");
    let id = w.add_creature(kind, Vec2::new(300.0, 200.0)).unwrap();
    w.step(&[PlayerInput::default()]);
    w.events.push(elora_sim::Event::CreatureDeath {
        id,
        kind,
        pos: Vec2::ZERO,
        killer: Some(s.player),
    });
    s.tick(&mut w, false);
    assert_eq!(s.save.flag("besiegt.brummbaer"), 1);
    assert!(s.save.holds(&s.content, "merker besiegt.brummbaer"));
}

#[test]
fn unlocks_from_dialogs_reach_the_running_world() {
    let (mut s, mut w) = start();
    let c = s.content.clone();
    s.save
        .run(&c, &["faehigkeit hook-ruck".into(), "waffe granate".into()]);
    let ch = w.character(s.player).unwrap();
    assert!(
        !ch.core.abilities.has(elora_sim::Ability::HookRuck),
        "noch nicht"
    );
    s.sync_world(&mut w);
    let ch = w.character(s.player).unwrap();
    assert!(ch.core.abilities.has(elora_sim::Ability::HookRuck));
    assert!(ch.arsenal.has(elora_sim::Weapon::Grenade));
}

/// Heranhooken (R2-M2.2): Sammelstücke am Hook werden eingesammelt, Beute fliegt zu Elora.
#[test]
fn pulling_hook_grabs_collectibles_and_loot() {
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("wiese-1", load("wiese-1"), "west", &Tuning::default());
    let stone = s.map.adventure.object("stein").unwrap().pos;
    let hook_at = |w: &mut World, s: &Session, at: Vec2| {
        let ch = w.character_mut(s.player).unwrap();
        ch.core.hook_state = elora_sim::HookState::Flying;
        ch.core.hook_pos = at;
    };
    // ohne Heranhooken: nichts
    hook_at(&mut w, &s, stone);
    s.tick(&mut w, false);
    assert_eq!(s.save.count("glitzerstein"), 0);
    // mit Heranhooken: eingesammelt
    s.save
        .run(&s.content.clone(), &["faehigkeit heranhooken".into()]);
    s.sync_world(&mut w);
    hook_at(&mut w, &s, stone);
    s.tick(&mut w, false);
    assert_eq!(s.save.count("glitzerstein"), 1);
    // Beute am Hook landet bei Elora
    let far = Vec2::new(40.0 * 32.0, 10.0 * 32.0);
    w.loot.push(elora_sim::creature::Loot {
        id: 999,
        item: "glanztropfen".into(),
        count: 5,
        pos: far,
        vel: Vec2::ZERO,
        age: 100,
    });
    hook_at(&mut w, &s, far);
    s.tick(&mut w, false);
    let elora = w.character(s.player).unwrap().core.pos;
    assert!(
        w.loot
            .iter()
            .find(|l| l.id == 999)
            .unwrap()
            .pos
            .distance(elora)
            < 1.0
    );
}

/// Begleiter (E-308): erscheint, folgt über Kartenwechsel und bleibt in seiner Heimat-Zone.
#[test]
fn follower_appears_follows_and_stays_home() {
    use elora_map::{Object, ObjectKind};
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("wiese-1", load("wiese-1"), "west", &Tuning::default());
    assert!(
        w.creatures
            .iter()
            .all(|c| w.creature_kinds[c.kind].name != "pilzkind")
    );
    s.save.run(
        &s.content.clone(),
        &["merker pilzkind.unterwegs = 1".into()],
    );
    s.sync_world(&mut w);
    let kid = |w: &World| {
        w.creatures
            .iter()
            .find(|c| w.creature_kinds[c.kind].name == "pilzkind")
            .map(|c| c.pos)
    };
    assert!(kid(&w).is_some(), "folgt");
    // neue Karte mit Pilzring: das Kind kommt mit
    let mut home = load("wiese-1");
    let spawn = home.adventure.object("west").unwrap().pos;
    home.adventure.objects.push(Object {
        id: "pilzring".into(),
        pos: spawn - Vec2::new(200.0, 200.0),
        kind: ObjectKind::Zone {
            size: Vec2::new(400.0, 400.0),
        },
    });
    let mut w = s.enter("wald-1", home, "west", &Tuning::default());
    assert!(kid(&w).is_some(), "über den Kartenwechsel");
    for _ in 0..5 {
        w.step(&[PlayerInput::default()]);
        s.tick(&mut w, false);
    }
    assert_eq!(s.save.flag("pilzkind.daheim"), 1);
    assert!(kid(&w).is_none(), "bleibt daheim");
}

/// 1,2 s in Leuchtpilzen: bunter Rausch (E-311).
#[test]
fn standing_in_glowing_mushrooms_dazes() {
    let mut map = load("wiese-1");
    let spawn = map.adventure.object("west").unwrap().pos;
    map.decor_front.push(elora_map::Decor::new(
        elora_map::Art::Builtin("leuchtpilze".into()),
        spawn + Vec2::new(0.0, 14.0),
    ));
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("wiese-1", map, "west", &Tuning::default());
    for _ in 0..50 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert_eq!(w.character(s.player).unwrap().core.dazed, 0, "noch nicht");
    for _ in 0..15 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(w.character(s.player).unwrap().core.dazed > 0);
}

/// Hitze-Leiste (E-320): Sonne füllt, voll = langsamer; Oase kühlt; Dach spendet Schatten.
#[test]
fn heat_fills_in_the_sun_and_cools_in_shade_and_at_the_oasis() {
    use elora_map::ObjectKind;
    // eine Karte der Wiese, aber als Wüste
    let mut map = load("wiese-1");
    let spawn = map.adventure.object("west").unwrap().pos;
    map.adventure.objects.push(elora_map::Object {
        id: "oase-1".into(),
        pos: spawn + Vec2::new(-80.0, -2000.0),
        kind: ObjectKind::Zone {
            size: Vec2::new(40.0, 40.0),
        },
    });
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("wueste-1", map, "west", &Tuning::default());
    // ohne gewürfelten Sandsturm (er verdeckt die Sonne, R2-W1)
    s.map.weather = elora_map::Weather::CLEAR;
    assert!(s.hot());
    let sky_above = (1..=10).all(|k| {
        #[allow(clippy::cast_precision_loss)]
        let p = w.character(s.player).unwrap().core.pos - Vec2::new(0.0, k as f32 * 32.0);
        !w.collision.tile_at(p).is_solid()
    });
    assert!(sky_above, "Testkarte: freier Himmel über dem Eingang");
    for _ in 0..500 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(
        s.in_sun && s.heat > 0.45 && s.heat < 0.55,
        "halb voll: {}",
        s.heat
    );
    for _ in 0..520 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(s.overheated && w.character(s.player).unwrap().core.overheated);
    // Dach über Elora: Schatten kühlt
    let pos = w.character(s.player).unwrap().core.pos;
    #[allow(clippy::cast_possible_truncation)]
    let (tx, ty) = ((pos.x / 32.0) as i32, (pos.y / 32.0) as i32 - 4);
    w.collision.set_tile(tx, ty, Tile::Solid);
    for _ in 0..100 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(!s.in_sun && s.heat < 0.9, "kühlt im Schatten: {}", s.heat);
    assert!(s.overheated, "bleibt langsam, bis die Hälfte erreicht ist");
    // Oase kühlt schnell
    w.collision.set_tile(tx, ty, Tile::Air);
    s.map.adventure.objects.last_mut().unwrap().pos = pos - Vec2::new(20.0, 20.0);
    for _ in 0..50 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(s.heat < 0.5 && !s.overheated, "an der Oase: {}", s.heat);
    assert!(!w.character(s.player).unwrap().core.overheated);
}

#[test]
fn no_heat_outside_the_desert() {
    let (mut s, mut w) = start();
    assert!(!s.hot());
    for _ in 0..200 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(s.heat == 0.0 && !s.in_sun);
}

/// Kaktusfrucht (E-320): heilt und leert die Hitze-Leiste, Elora ist wieder schnell.
#[test]
fn cactus_fruit_cools_elora_down() {
    let map = load("wiese-1");
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("wueste-1", map, "west", &Tuning::default());
    // ohne gewürfelten Sandsturm (er verdeckt die Sonne, R2-W1)
    s.map.weather = elora_map::Weather::CLEAR;
    for _ in 0..1100 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(s.overheated);
    s.save
        .add_item(&s.content.clone(), "kaktusfrucht", 1)
        .unwrap();
    s.use_item(&mut w, "kaktusfrucht").unwrap();
    assert!(s.heat == 0.0 && !s.overheated);
    assert!(!w.character(s.player).unwrap().core.overheated);
}

/// Wetter beim Betreten (R2-W1, E-331): Trüb, solange die Quelle schweigt; danach die
/// bunteren Wetter des Gebiets; Arenen schön; eigenes Kartenwetter geht vor.
#[test]
fn weather_follows_the_springs() {
    use elora_map::{Weather, WeatherKind};
    let kinds = |freed: bool, map: &str| {
        let mut seen = std::collections::BTreeSet::new();
        for t in 0..40 {
            let mut s = Session::new_game(Content::builtin());
            s.save.play_time_secs = t * 37;
            if freed {
                s.save.set_flag("befreit.bluetenquelle", 1);
                s.save.set_flag("quellen_befreit", 1);
            }
            let w = s.enter(map, load("wiese-1"), "west", &Tuning::default());
            drop(w);
            seen.insert(s.map.weather.kind.key());
        }
        seen
    };
    let gloomy = kinds(false, "wiese-1");
    assert!(
        gloomy.iter().all(|k| ["regen", "gewitter"].contains(k)),
        "{gloomy:?}"
    );
    let bright = kinds(true, "wiese-1");
    assert!(
        bright
            .iter()
            .all(|k| ["schoen", "blueten", "regen"].contains(k)),
        "{bright:?}"
    );
    assert!(bright.contains("schoen"), "meist schön");
    assert_eq!(
        kinds(false, "wiese-arena").into_iter().collect::<Vec<_>>(),
        ["schoen"],
        "Arena"
    );
    // eigenes Wetter der Karte (Editor) geht vor
    let mut map = load("wiese-1");
    map.weather = Weather {
        kind: WeatherKind::Fog,
        intensity: 0.5,
        wind: 0.0,
    };
    let mut s = Session::new_game(Content::builtin());
    s.enter("wiese-1", map, "west", &Tuning::default());
    assert_eq!(s.map.weather.kind, WeatherKind::Fog);
}

/// Tauwinkel nieselt bis zur ersten Quelle; wird sie befreit, klart es auf, ohne die Karte zu
/// verlassen (nach dem Gespräch mit Tüftel).
#[test]
fn tauwinkel_clears_after_the_first_spring() {
    use elora_map::WeatherKind;
    let (mut s, _w) = start();
    assert_eq!(s.map.weather.kind, WeatherKind::Rain, "Niesel");
    assert!(s.map.weather.intensity < 0.5);
    s.save.set_flag("quellen_befreit", 1);
    s.refresh_decor();
    assert!(
        matches!(
            s.map.weather.kind,
            WeatherKind::Clear | WeatherKind::Petals | WeatherKind::Rain
        ),
        "{:?}",
        s.map.weather
    );
    assert_ne!(
        (s.map.weather.kind, s.map.weather.intensity < 0.5),
        (WeatherKind::Rain, true),
        "kein Niesel mehr"
    );
}

/// Sand verdeckt die Sonne: im Sandsturm füllt sich die Hitze-Leiste nicht (R2-W1, E-320).
#[test]
fn sandstorm_hides_the_sun() {
    let mut map = load("wiese-1");
    map.weather = elora_map::Weather {
        kind: elora_map::WeatherKind::Sandstorm,
        intensity: 0.8,
        wind: 0.6,
    };
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("wueste-1", map, "west", &Tuning::default());
    for _ in 0..300 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(!s.in_sun && s.heat == 0.0, "{}", s.heat);
}

#[test]
fn avalanche_rolls_rocks_then_rests() {
    use elora_map::ObjectKind;
    let mut map = load("wiese-1");
    let spawn = map.adventure.object("west").unwrap().pos;
    let zone = |id: &str, pos: Vec2, size: Vec2| elora_map::Object {
        id: id.into(),
        pos,
        kind: ObjectKind::Zone { size },
    };
    // Hang weit weg, die Auslöse-Stelle genau am Eingang
    map.adventure.objects.push(zone(
        "lawine-test",
        spawn + Vec2::new(900.0, -600.0),
        Vec2::new(500.0, 500.0),
    ));
    map.adventure.objects.push(zone(
        "lawine-test-tritt",
        spawn - Vec2::new(100.0, 100.0),
        Vec2::new(200.0, 200.0),
    ));
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("wiese-1", map, "west", &Tuning::default());
    let rock = w.creature_kind("schneebrocken").expect("in creatures.toml");
    let mut seen = std::collections::BTreeSet::new();
    let mut count_after = |s: &mut Session, w: &mut World, ticks: u32| {
        for _ in 0..ticks {
            step(s, w, PlayerInput::default(), false);
            seen.extend(w.creatures.iter().filter(|c| c.kind == rock).map(|c| c.id));
        }
        seen.len()
    };
    let t = Tuning::default();
    let rocks = usize::try_from(t.avalanche_rocks).unwrap();
    assert_eq!(count_after(&mut s, &mut w, 200), rocks, "eine Lawine");
    // Elora steht weiter auf der Auslöse-Stelle: der Hang ruht erst
    assert_eq!(count_after(&mut s, &mut w, 100), rocks, "Ruhe nach A-41");
    let rest = elora_sim::tuning::ms_to_ticks(t.avalanche_rest);
    assert_eq!(
        count_after(&mut s, &mut w, rest),
        rocks * 2,
        "danach wieder"
    );
}

#[test]
fn cold_fills_outside_faster_in_blizzards_and_warms_at_fire_and_roofs() {
    use elora_map::{ObjectKind, Weather, WeatherKind};
    // eine Karte der Wiese, aber in den Frostspitzen
    let mut map = load("wiese-1");
    let spawn = map.adventure.object("west").unwrap().pos;
    map.adventure.objects.push(elora_map::Object {
        id: "feuer-1".into(),
        pos: spawn + Vec2::new(-80.0, -2000.0),
        kind: ObjectKind::Zone {
            size: Vec2::new(40.0, 40.0),
        },
    });
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("frost-1", map, "west", &Tuning::default());
    s.map.weather = Weather::CLEAR;
    assert!(s.chilly() && !s.hot());
    let run = |s: &mut Session, w: &mut World, ticks: u32| {
        for _ in 0..ticks {
            step(s, w, PlayerInput::default(), false);
        }
    };
    // draußen: etwa 60 s bis voll
    run(&mut s, &mut w, 1500);
    assert!(s.cold > 0.45 && s.cold < 0.55, "halb voll: {}", s.cold);
    // im Schneesturm doppelt so schnell
    s.map.weather = Weather {
        kind: WeatherKind::Blizzard,
        intensity: 1.0,
        wind: 0.0,
    };
    run(&mut s, &mut w, 760);
    assert!(s.frozen && w.character(s.player).unwrap().core.overheated);
    // Dach über Elora wärmt langsam
    s.map.weather = Weather::CLEAR;
    let pos = w.character(s.player).unwrap().core.pos;
    #[allow(clippy::cast_possible_truncation)]
    let (tx, ty) = ((pos.x / 32.0) as i32, (pos.y / 32.0) as i32 - 4);
    w.collision.set_tile(tx, ty, Tile::Solid);
    run(&mut s, &mut w, 100);
    assert!(
        s.cold < 0.95 && s.frozen,
        "wärmt unter dem Dach: {}",
        s.cold
    );
    // am Feuer schnell warm
    w.collision.set_tile(tx, ty, Tile::Air);
    s.map.adventure.objects.last_mut().unwrap().pos = pos - Vec2::new(20.0, 20.0);
    run(&mut s, &mut w, 100);
    assert!(s.cold < 0.4 && !s.frozen, "am Feuer: {}", s.cold);
    assert!(!w.character(s.player).unwrap().core.overheated);
}

#[test]
fn no_cold_outside_the_mountains() {
    let (mut s, mut w) = start();
    for _ in 0..300 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(!s.chilly());
    assert!(s.cold.abs() < f32::EPSILON && !s.frozen);
}

#[test]
fn kristellas_storm_fills_the_hall_until_she_is_calmed() {
    use elora_map::WeatherKind;
    let map = load("wiese-1");
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("frost-arena", map, "west", &Tuning::default());
    assert!(s.map.weather.is_clear(), "Arenen bleiben schön");
    let me = w.character(s.player).unwrap().core.pos;
    let queen = w.creature_kind("kristella").expect("in creatures.toml");
    let id = w.add_creature(queen, me - Vec2::new(0.0, 200.0)).unwrap();
    // im letzten Viertel
    w.creatures.iter_mut().find(|c| c.id == id).unwrap().health = 8;
    for _ in 0..5 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert_eq!(s.map.weather.kind, WeatherKind::Blizzard, "Schneesturm");
    assert!(
        w.weather.is_some_and(|e| e.wind.abs() > 0.5),
        "Wind in der Halle"
    );
    // beruhigt: der Sturm legt sich
    let c = w.creatures.iter_mut().find(|c| c.id == id).unwrap();
    c.mode = elora_sim::creature::queen::TIRED;
    w.hurt_creature(id, 8);
    s.tick(&mut w, false);
    assert!(s.map.weather.is_clear());
}

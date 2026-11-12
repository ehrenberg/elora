//! Adventure session (A1.6) on the test maps (`fixture`): chests, switches and doors, save
//! point, transitions, zones, death.

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

/// Put Elora at an object and let her stand there for a few ticks.
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
    // gate is closed
    let door = s.map.adventure.object("gate").unwrap().pos;
    assert_eq!(
        w.collision.tile_at(door + Vec2::new(16.0, 16.0)),
        Tile::Unhookable
    );
}

#[test]
fn chest_switch_and_door() {
    let (mut s, mut w) = start();
    go_to(&mut s, &mut w, "chest-1");
    assert_eq!(
        s.interactable(w.character(s.player).unwrap().core.pos)
            .map(|x| x.1),
        Some(Prompt::Open)
    );
    let ev = step(&mut s, &mut w, PlayerInput::default(), true);
    assert!(ev.contains(&SessionEvent::Notice(Notice::Item {
        id: "gleam_drops".into(),
        count: 20
    })));
    // sparkles and sound at the chest (playtest)
    let at = s.map.adventure.object("chest-1").unwrap().pos;
    assert!(ev.contains(&SessionEvent::ChestOpened { pos: at }));
    assert_eq!(s.save.gleam_drops, 20);
    assert!(s.object_done("chest-1"));
    // opening a second time is not possible
    assert!(step(&mut s, &mut w, PlayerInput::default(), true).is_empty());

    go_to(&mut s, &mut w, "lever");
    let ev = step(&mut s, &mut w, PlayerInput::default(), true);
    assert!(
        ev.contains(&SessionEvent::TilesChanged),
        "gate opens: {ev:?}"
    );
    let door = s.map.adventure.object("gate").unwrap().pos;
    assert_eq!(w.collision.tile_at(door + Vec2::new(16.0, 16.0)), Tile::Air);
    // lever back: gate stays open (E-254)
    step(&mut s, &mut w, PlayerInput::default(), true);
    assert_eq!(s.save.flag("gate.village"), 0);
    assert_eq!(w.collision.tile_at(door + Vec2::new(16.0, 16.0)), Tile::Air);
}

#[test]
fn save_point_rests_and_asks_to_save() {
    let (mut s, mut w) = start();
    w.character_mut(s.player).unwrap().health = 3;
    go_to(&mut s, &mut w, "well");
    let ev = step(&mut s, &mut w, PlayerInput::default(), true);
    assert!(ev.contains(&SessionEvent::Save));
    assert_eq!(w.character(s.player).unwrap().health, 10);
    assert_eq!(s.save.location.spawn, "well");
}

#[test]
fn walking_into_the_exit_travels_and_zones_count() {
    let (mut s, mut w) = start();
    s.save.run(
        &s.content.clone(),
        &[
            "quest well start".into(),
            "quest well advance".into(),
            "quest well advance".into(),
        ],
    );
    let exit = s.map.adventure.object("path-meadow").unwrap().pos;
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
    let (map, spawn) = travel.expect("exit when walking in");
    assert_eq!((map.as_str(), spawn.as_str()), ("meadow-1", "west"));
    let mut w = s.enter(&map, load(&map), &spawn, &Tuning::default());
    assert_eq!(s.save.location.map, "meadow-1");
    assert_eq!(w.creatures.len(), 5);
    assert!(s.save.holds(&s.content, "quest well step beetle"));
    s.save
        .run(&s.content.clone(), &["quest well advance".into()]);
    // reach zone „Wiesenrand“: quest done, the next one starts
    let zone = s.map.adventure.object("meadow_edge").unwrap().pos;
    w.spawn_character(s.player, zone + Vec2::new(100.0, 200.0));
    let mut notes = Vec::new();
    for _ in 0..3 {
        notes.extend(step(&mut s, &mut w, PlayerInput::default(), false));
    }
    assert!(s.save.holds(&s.content, "quest well done"), "{notes:?}");
    assert!(s.save.holds(&s.content, "quest blossom_spring active"));
}

#[test]
fn death_loses_some_gleam_and_marks_dead() {
    let (mut s, mut w) = start();
    let c = s.content.clone();
    s.save.add_item(&c, "gleam_drops", 40).unwrap();
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
    assert_eq!(s.save.gleam_drops, 30);
}

#[test]
fn locked_chest_needs_condition() {
    let (mut s, _) = start();
    let mut w = s.enter("meadow-1", load("meadow-1"), "west", &Tuning::default());
    go_to(&mut s, &mut w, "chest-spring");
    let ev = step(&mut s, &mut w, PlayerInput::default(), true);
    assert_eq!(
        ev,
        vec![SessionEvent::Locked {
            object: "chest-spring".into()
        }]
    );
    let c = s.content.clone();
    s.save.add_item(&c, "glitter_stone", 1).unwrap();
    let ev = step(&mut s, &mut w, PlayerInput::default(), true);
    assert!(ev.contains(&SessionEvent::Notice(Notice::Item {
        id: "amber".into(),
        count: 3
    })));
}

#[test]
fn defeated_boss_sets_a_flag() {
    let (mut s, mut w) = start();
    let kind = w
        .creature_kind("bumblebear")
        .expect("guardian in creatures.toml");
    let id = w.add_creature(kind, Vec2::new(300.0, 200.0)).unwrap();
    w.step(&[PlayerInput::default()]);
    w.events.push(elora_sim::Event::CreatureDeath {
        id,
        kind,
        pos: Vec2::ZERO,
        killer: Some(s.player),
    });
    s.tick(&mut w, false);
    assert_eq!(s.save.flag("defeated.bumblebear"), 1);
    assert!(s.save.holds(&s.content, "flag defeated.bumblebear"));
}

#[test]
fn unlocks_from_dialogs_reach_the_running_world() {
    let (mut s, mut w) = start();
    let c = s.content.clone();
    s.save
        .run(&c, &["ability hook-jerk".into(), "weapon grenade".into()]);
    let ch = w.character(s.player).unwrap();
    assert!(
        !ch.core.abilities.has(elora_sim::Ability::HookJerk),
        "not yet"
    );
    s.sync_world(&mut w);
    let ch = w.character(s.player).unwrap();
    assert!(ch.core.abilities.has(elora_sim::Ability::HookJerk));
    assert!(ch.arsenal.has(elora_sim::Weapon::Grenade));
}

/// The star spark (D-M25-04): Tüftel's work on the star cape applies right away and stays.
#[test]
fn strengthened_star_cape_reaches_the_running_world_and_later_ones() {
    use elora_adventure::state::{STRONG_GLIDE, STRONG_GLIDE_FLAP, STRONG_GLIDE_UPDRAFT};
    let (mut s, mut w) = start();
    let base = Tuning::default();
    assert!(w.tuning.glide_flap.abs() < f32::EPSILON, "no flap yet");
    assert!((w.tuning.glide_updraft - base.glide_updraft).abs() < f32::EPSILON);
    let c = s.content.clone();
    s.save.run(&c, &[format!("flag {STRONG_GLIDE} = 1")]);
    s.sync_world(&mut w);
    let strong = |w: &World| {
        (w.tuning.glide_flap - STRONG_GLIDE_FLAP).abs() < f32::EPSILON
            && (w.tuning.glide_updraft - STRONG_GLIDE_UPDRAFT).abs() < f32::EPSILON
    };
    assert!(strong(&w), "right after Tüftel's work");
    let w = s.enter("meadow-1", load("meadow-1"), "west", &base);
    assert!(strong(&w), "and on every map afterwards");
}

/// Pull hook (R2-M2.2): collectibles on the hook are collected, loot flies to Elora.
#[test]
fn pulling_hook_grabs_collectibles_and_loot() {
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("meadow-1", load("meadow-1"), "west", &Tuning::default());
    let stone = s.map.adventure.object("stone").unwrap().pos;
    let hook_at = |w: &mut World, s: &Session, at: Vec2| {
        let ch = w.character_mut(s.player).unwrap();
        ch.core.hook_state = elora_sim::HookState::Flying;
        ch.core.hook_pos = at;
    };
    // without pull hook: nothing
    hook_at(&mut w, &s, stone);
    s.tick(&mut w, false);
    assert_eq!(s.save.count("glitter_stone"), 0);
    // with pull hook: collected
    s.save
        .run(&s.content.clone(), &["ability pull_hook".into()]);
    s.sync_world(&mut w);
    hook_at(&mut w, &s, stone);
    s.tick(&mut w, false);
    assert_eq!(s.save.count("glitter_stone"), 1);
    // loot on the hook ends up with Elora
    let far = Vec2::new(40.0 * 32.0, 10.0 * 32.0);
    w.loot.push(elora_sim::creature::Loot {
        id: 999,
        item: "gleam_drops".into(),
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

/// Companion (E-308): appears, follows across map changes and stays in its home zone.
#[test]
fn follower_appears_follows_and_stays_home() {
    use elora_map::{Object, ObjectKind};
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("meadow-1", load("meadow-1"), "west", &Tuning::default());
    assert!(
        w.creatures
            .iter()
            .all(|c| w.creature_kinds[c.kind].name != "mushroom_child")
    );
    s.save.run(
        &s.content.clone(),
        &["flag mushroom_child.on_the_way = 1".into()],
    );
    s.sync_world(&mut w);
    let kid = |w: &World| {
        w.creatures
            .iter()
            .find(|c| w.creature_kinds[c.kind].name == "mushroom_child")
            .map(|c| c.pos)
    };
    assert!(kid(&w).is_some(), "follows");
    // new map with mushroom ring: the child comes along
    let mut home = load("meadow-1");
    let spawn = home.adventure.object("west").unwrap().pos;
    home.adventure.objects.push(Object {
        id: "mushroom_ring".into(),
        pos: spawn - Vec2::new(200.0, 200.0),
        kind: ObjectKind::Zone {
            size: Vec2::new(400.0, 400.0),
        },
    });
    let mut w = s.enter("forest-1", home, "west", &Tuning::default());
    assert!(kid(&w).is_some(), "across the map change");
    for _ in 0..5 {
        w.step(&[PlayerInput::default()]);
        s.tick(&mut w, false);
    }
    assert_eq!(s.save.flag("mushroom_child.home"), 1);
    assert!(kid(&w).is_none(), "stays home");
}

/// 1.2 s in glow mushrooms: colourful daze (E-311).
#[test]
fn standing_in_glowing_mushrooms_dazes() {
    let mut map = load("meadow-1");
    let spawn = map.adventure.object("west").unwrap().pos;
    map.decor_front.push(elora_map::Decor::new(
        elora_map::Art::Builtin("glow_mushrooms".into()),
        spawn + Vec2::new(0.0, 14.0),
    ));
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("meadow-1", map, "west", &Tuning::default());
    for _ in 0..50 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert_eq!(w.character(s.player).unwrap().core.dazed, 0, "not yet");
    for _ in 0..15 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(w.character(s.player).unwrap().core.dazed > 0);
}

/// Heat bar (E-320): sun fills it, full = slower; oasis cools; roof gives shade.
#[test]
fn heat_fills_in_the_sun_and_cools_in_shade_and_at_the_oasis() {
    use elora_map::ObjectKind;
    // a meadow map, but as desert
    let mut map = load("meadow-1");
    let spawn = map.adventure.object("west").unwrap().pos;
    map.adventure.objects.push(elora_map::Object {
        id: "oasis-1".into(),
        pos: spawn + Vec2::new(-80.0, -2000.0),
        kind: ObjectKind::Zone {
            size: Vec2::new(40.0, 40.0),
        },
    });
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("desert-1", map, "west", &Tuning::default());
    // without a rolled sandstorm (it hides the sun, R2-W1) and without the beetles
    s.map.weather = elora_map::Weather::CLEAR;
    w.creatures.clear();
    assert!(s.hot());
    let sky_above = (1..=10).all(|k| {
        let p = w.character(s.player).unwrap().core.pos - Vec2::new(0.0, k as f32 * 32.0);
        !w.collision.tile_at(p).is_solid()
    });
    assert!(sky_above, "test map: open sky above the entrance");
    for _ in 0..500 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(
        s.in_sun && s.heat > 0.45 && s.heat < 0.55,
        "half full: {}",
        s.heat
    );
    for _ in 0..520 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(s.overheated && w.character(s.player).unwrap().core.overheated);
    // roof above Elora: shade cools
    let pos = w.character(s.player).unwrap().core.pos;
    let (tx, ty) = ((pos.x / 32.0) as i32, (pos.y / 32.0) as i32 - 4);
    w.collision.set_tile(tx, ty, Tile::Solid);
    for _ in 0..100 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(!s.in_sun && s.heat < 0.9, "cools in the shade: {}", s.heat);
    assert!(s.overheated, "stays slow until half is reached");
    // oasis cools fast
    w.collision.set_tile(tx, ty, Tile::Air);
    s.map.adventure.objects.last_mut().unwrap().pos = pos - Vec2::new(20.0, 20.0);
    for _ in 0..50 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(s.heat < 0.5 && !s.overheated, "at the oasis: {}", s.heat);
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

/// Cactus fruit (E-320): heals and empties the heat bar, Elora is fast again.
#[test]
fn cactus_fruit_cools_elora_down() {
    let map = load("meadow-1");
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("desert-1", map, "west", &Tuning::default());
    // without a rolled sandstorm (it hides the sun, R2-W1) and without the beetles
    s.map.weather = elora_map::Weather::CLEAR;
    w.creatures.clear();
    for _ in 0..1100 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(s.overheated);
    s.save
        .add_item(&s.content.clone(), "cactus_fruit", 1)
        .unwrap();
    s.use_item(&mut w, "cactus_fruit").unwrap();
    assert!(s.heat == 0.0 && !s.overheated);
    assert!(!w.character(s.player).unwrap().core.overheated);
}

/// Weather on entering (R2-W1, E-331): gloomy while the spring is silent; afterwards the
/// more colourful weather of the area; arenas fair; the map's own weather comes first.
#[test]
fn weather_follows_the_springs() {
    use elora_map::{Weather, WeatherKind};
    let kinds = |freed: bool, map: &str| {
        let mut seen = std::collections::BTreeSet::new();
        for t in 0..40 {
            let mut s = Session::new_game(Content::builtin());
            s.save.play_time_secs = t * 37;
            if freed {
                s.save.set_flag("freed.blossom_spring", 1);
                s.save.set_flag("springs_freed", 1);
            }
            let w = s.enter(map, load("meadow-1"), "west", &Tuning::default());
            drop(w);
            seen.insert(s.map.weather.kind.key());
        }
        seen
    };
    // mostly dry and grey, sometimes rain or a thunderstorm (E-353)
    let gloomy = kinds(false, "meadow-1");
    assert!(
        gloomy
            .iter()
            .all(|k| ["clear", "rain", "storm"].contains(k))
            && gloomy.contains("clear")
            && gloomy.contains("rain"),
        "{gloomy:?}"
    );
    let bright = kinds(true, "meadow-1");
    assert!(
        bright
            .iter()
            .all(|k| ["clear", "blossoms", "rain"].contains(k)),
        "{bright:?}"
    );
    assert!(bright.contains("clear"), "mostly fair");
    assert_eq!(
        kinds(false, "meadow-arena").into_iter().collect::<Vec<_>>(),
        ["clear"],
        "Arena"
    );
    // the map's own weather (editor) comes first
    let mut map = load("meadow-1");
    map.weather = Weather {
        kind: WeatherKind::Fog,
        intensity: 0.5,
        wind: 0.0,
    };
    let mut s = Session::new_game(Content::builtin());
    s.enter("meadow-1", map, "west", &Tuning::default());
    assert_eq!(s.map.weather.kind, WeatherKind::Fog);
}

/// Tauwinkel drizzles until the first spring; once it is freed, the sky clears without
/// leaving the map (after the dialog with Tüftel).
#[test]
fn tauwinkel_clears_after_the_first_spring() {
    use elora_map::WeatherKind;
    let (mut s, _w) = start();
    // gloomy: dry, drizzle or a light thunderstorm (E-353)
    let w = s.map.weather;
    assert!(
        w.kind == WeatherKind::Clear
            || w.kind == WeatherKind::Storm
            || (w.kind == WeatherKind::Rain && w.intensity < 0.5),
        "{w:?}"
    );
    s.save.set_flag("springs_freed", 1);
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
        "no more drizzle"
    );
}

/// Sand hides the sun: in a sandstorm the heat bar does not fill (R2-W1, E-320).
#[test]
fn sandstorm_hides_the_sun() {
    let mut map = load("meadow-1");
    map.weather = elora_map::Weather {
        kind: elora_map::WeatherKind::Sandstorm,
        intensity: 0.8,
        wind: 0.6,
    };
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("desert-1", map, "west", &Tuning::default());
    for _ in 0..300 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(!s.in_sun && s.heat == 0.0, "{}", s.heat);
}

#[test]
fn avalanche_rolls_rocks_then_rests() {
    use elora_map::ObjectKind;
    let mut map = load("meadow-1");
    let spawn = map.adventure.object("west").unwrap().pos;
    let zone = |id: &str, pos: Vec2, size: Vec2| elora_map::Object {
        id: id.into(),
        pos,
        kind: ObjectKind::Zone { size },
    };
    // slope far away, the trigger spot right at the entrance
    map.adventure.objects.push(zone(
        "avalanche-test",
        spawn + Vec2::new(900.0, -600.0),
        Vec2::new(500.0, 500.0),
    ));
    map.adventure.objects.push(zone(
        "avalanche-test-step",
        spawn - Vec2::new(100.0, 100.0),
        Vec2::new(200.0, 200.0),
    ));
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("meadow-1", map, "west", &Tuning::default());
    let rock = w.creature_kind("snow_chunk").expect("in creatures.toml");
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
    assert_eq!(count_after(&mut s, &mut w, 200), rocks, "one avalanche");
    // Elora keeps standing on the trigger spot: the slope rests first
    assert_eq!(count_after(&mut s, &mut w, 100), rocks, "quiet after A-41");
    let rest = elora_sim::tuning::ms_to_ticks(t.avalanche_rest);
    assert_eq!(
        count_after(&mut s, &mut w, rest),
        rocks * 2,
        "again afterwards"
    );
}

#[test]
fn cold_fills_outside_faster_in_blizzards_and_warms_at_fire_and_roofs() {
    use elora_map::{ObjectKind, Weather, WeatherKind};
    // a meadow map, but in the Frostspitzen
    let mut map = load("meadow-1");
    let spawn = map.adventure.object("west").unwrap().pos;
    map.adventure.objects.push(elora_map::Object {
        id: "fire-1".into(),
        pos: spawn + Vec2::new(-80.0, -2000.0),
        kind: ObjectKind::Zone {
            size: Vec2::new(40.0, 40.0),
        },
    });
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("frost-1", map, "west", &Tuning::default());
    s.map.weather = Weather::CLEAR;
    // the beetles would walk over the bridge and interrupt the measurement
    w.creatures.clear();
    assert!(s.chilly() && !s.hot());
    let run = |s: &mut Session, w: &mut World, ticks: u32| {
        for _ in 0..ticks {
            step(s, w, PlayerInput::default(), false);
        }
    };
    // outside: about 60 s until full
    run(&mut s, &mut w, 1500);
    assert!(s.cold > 0.45 && s.cold < 0.55, "half full: {}", s.cold);
    // twice as fast in a blizzard
    s.map.weather = Weather {
        kind: WeatherKind::Blizzard,
        intensity: 1.0,
        wind: 0.0,
    };
    run(&mut s, &mut w, 760);
    assert!(s.frozen && w.character(s.player).unwrap().core.overheated);
    // roof above Elora warms slowly
    s.map.weather = Weather::CLEAR;
    let pos = w.character(s.player).unwrap().core.pos;
    let (tx, ty) = ((pos.x / 32.0) as i32, (pos.y / 32.0) as i32 - 4);
    w.collision.set_tile(tx, ty, Tile::Solid);
    run(&mut s, &mut w, 100);
    assert!(
        s.cold < 0.95 && s.frozen,
        "warms under the roof: {}",
        s.cold
    );
    // warm quickly at the fire
    w.collision.set_tile(tx, ty, Tile::Air);
    s.map.adventure.objects.last_mut().unwrap().pos = pos - Vec2::new(20.0, 20.0);
    run(&mut s, &mut w, 100);
    assert!(s.cold < 0.4 && !s.frozen, "at the fire: {}", s.cold);
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
    let map = load("meadow-1");
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("frost-arena", map, "west", &Tuning::default());
    assert!(s.map.weather.is_clear(), "arenas stay fair");
    let me = w.character(s.player).unwrap().core.pos;
    let queen = w.creature_kind("kristella").expect("in creatures.toml");
    let id = w.add_creature(queen, me - Vec2::new(0.0, 200.0)).unwrap();
    // in the last quarter
    w.creatures.iter_mut().find(|c| c.id == id).unwrap().health = 8;
    for _ in 0..5 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert_eq!(s.map.weather.kind, WeatherKind::Blizzard, "blizzard");
    assert!(
        w.weather.is_some_and(|e| e.wind.abs() > 0.5),
        "wind in the hall"
    );
    // calmed: the storm settles
    let c = w.creatures.iter_mut().find(|c| c.id == id).unwrap();
    c.mode = elora_sim::creature::queen::TIRED;
    w.hurt_creature(id, 8);
    s.tick(&mut w, false);
    assert!(s.map.weather.is_clear());
}

/// Night in the Sternschlucht (R2-M2.5, M2.5.1): the area is dark, glowing decoration lights it.
#[test]
fn night_maps_are_dark_and_glowing_decoration_lights_them() {
    let mut map = load("meadow-1");
    let spawn = map.adventure.object("west").unwrap().pos;
    map.decor_back.push(elora_map::Decor::new(
        elora_map::Art::Builtin("lantern".into()),
        spawn + Vec2::new(64.0, 0.0),
    ));
    let mut s = Session::new_game(Content::builtin());
    let w = s.enter("gorge-1", map.clone(), "west", &Tuning::default());
    assert!(s.darkness() > 0.5, "the Sternschlucht is night");
    assert!(w.is_lit(spawn + Vec2::new(64.0, 0.0)));
    assert!(!w.is_lit(spawn + Vec2::new(2000.0, 0.0)));
    // a day map has no lights at all
    let w = s.enter("meadow-1", map.clone(), "west", &Tuning::default());
    assert!(s.darkness() == 0.0 && w.lights.is_empty());
    // a single map can be darker than its area
    let mut content = Content::builtin();
    let gorge = content
        .areas
        .iter_mut()
        .find(|a| a.id == "sternschlucht")
        .unwrap();
    gorge.map_darkness.insert("gorge-3".into(), 1.0);
    let mut s = Session::new_game(content);
    s.enter("gorge-1", map.clone(), "west", &Tuning::default());
    assert!((s.darkness() - 0.9).abs() < 1e-6);
    s.enter("gorge-3", map, "west", &Tuning::default());
    assert!((s.darkness() - 1.0).abs() < 1e-6);
}

/// Star islands and updrafts (R2-M2.5, M2.5.2) come from the zones of the map.
#[test]
fn island_and_updraft_zones_become_part_of_the_world() {
    let mut map = load("meadow-1");
    let spawn = map.adventure.object("west").unwrap().pos;
    for (id, at) in [
        ("island-sink-a", spawn + Vec2::new(200.0, -64.0)),
        ("island-sink-a-to", spawn + Vec2::new(400.0, -64.0)),
        ("updraft-a", spawn + Vec2::new(600.0, -300.0)),
    ] {
        map.adventure.objects.push(elora_map::Object {
            id: id.into(),
            pos: at,
            kind: elora_map::ObjectKind::Zone {
                size: Vec2::new(128.0, 32.0),
            },
        });
    }
    let mut s = Session::new_game(Content::builtin());
    let w = s.enter("meadow-1", map, "west", &Tuning::default());
    assert_eq!(w.islands.len(), 1);
    assert!(w.islands[0].sinks);
    assert_eq!(w.islands[0].end, spawn + Vec2::new(400.0, -64.0));
    assert_eq!(w.collision.movers.updrafts.len(), 1);
}

/// The session remembers the notes Elora plays (R2-M2.5, M2.5.3) for Luma's song.
#[test]
fn played_notes_are_remembered_for_lumas_song() {
    let map = load("meadow-1");
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("meadow-1", map.clone(), "west", &Tuning::default());
    for note in [0, 4, 7, 4, 12] {
        w.events.push(elora_sim::Event::Note {
            player: s.player,
            note,
            pos: Vec2::ZERO,
        });
        s.tick(&mut w, false);
        w.events.clear();
    }
    assert!(s.played(&[7, 4, 12]));
    assert!(!s.played(&[0, 4, 7]), "only the last notes count");
    // another figure's notes do not count, a new map starts afresh
    w.events.push(elora_sim::Event::Note {
        player: s.player + 1,
        note: 2,
        pos: Vec2::ZERO,
    });
    s.tick(&mut w, false);
    assert!(s.played(&[4, 12]));
    s.enter("meadow-1", map, "west", &Tuning::default());
    assert!(s.notes.is_empty());
}

/// World map (R2-WM, E-370): Elora's section and her place along it.
#[test]
fn map_spot_follows_elora_along_her_section() {
    let map = load("meadow-1");
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("meadow-1", map.clone(), "west", &Tuning::default());
    let start = s.map_spot(&w).expect("on the world map");
    assert_eq!(
        (start.area.as_str(), start.map.as_str()),
        ("bluetenwiesen", "meadow-1")
    );
    assert!(
        start.along < 0.2,
        "enters from the side of Tauwinkel: {}",
        start.along
    );
    let section = s.content.areas[1].section("meadow-1").unwrap().clone();
    let first = section.path[0];
    assert!((start.point[0] - first[0]).abs() < 0.05 && (start.point[1] - first[1]).abs() < 0.05);
    // further right in the map = further along the path
    let far = Vec2::new(map.width as f32 * 32.0 - 100.0, 100.0);
    w.character_mut(s.player).unwrap().core.pos = far;
    let end = s.map_spot(&w).unwrap();
    assert!(end.along > 0.9);
    let last = section.path[section.path.len() - 1];
    assert!((end.point[0] - last[0]).abs() < 0.03 && (end.point[1] - last[1]).abs() < 0.03);
}

#[test]
fn every_adventure_map_with_a_file_has_a_section() {
    let c = Content::builtin();
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../maps/adventure");
    for e in std::fs::read_dir(dir).unwrap() {
        let name = e
            .unwrap()
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let area = c
            .area_of(&name)
            .unwrap_or_else(|| panic!("{name}: no area"));
        assert!(
            area.section(&name).is_some(),
            "{name}: no section on the world map"
        );
    }
}

// ---------------------------------------------------------------- Tunes (M2.5.6)

/// The bundled content plus one tune on `meadow-1`: e g c2 counts `luma_song.verses` up.
fn content_with_tune(cond: &str) -> Content {
    let mut c = Content::builtin();
    c.tunes.push(elora_adventure::quest::TuneDef {
        id: "verse-1".into(),
        map: "meadow-1".into(),
        notes: vec!["e".into(), "g".into(), "c2".into()],
        cond: (!cond.is_empty()).then(|| cond.to_owned()),
        actions: vec!["flag luma_song.verses +1".into()],
    });
    c
}

/// Elora touches note stones with these pitches, one per tick.
fn play(s: &mut Session, w: &mut World, notes: &[u8]) -> Vec<SessionEvent> {
    let mut all = Vec::new();
    for &note in notes {
        w.events.push(elora_sim::Event::Note {
            player: s.player,
            note,
            pos: Vec2::ZERO,
        });
        all.extend(s.tick(w, false));
        w.events.clear();
    }
    all
}

fn tune_played(ev: &[SessionEvent]) -> bool {
    ev.contains(&SessionEvent::TunePlayed {
        id: "verse-1".into(),
    })
}

#[test]
fn a_tune_on_the_note_stones_sets_off_its_effects_once() {
    let mut s = Session::new_game(content_with_tune(""));
    let mut w = s.enter("meadow-1", load("meadow-1"), "west", &Tuning::default());
    let ev = play(&mut s, &mut w, &[0, 4, 7]);
    assert!(!tune_played(&ev), "not the tune yet");
    assert_eq!(s.save.flag("luma_song.verses"), 0);
    // the tune may start anywhere in what she plays
    let ev = play(&mut s, &mut w, &[4, 7, 12]);
    assert!(tune_played(&ev));
    assert_eq!(s.save.flag("luma_song.verses"), 1);
    // playing it again does nothing, also after coming back
    let ev = play(&mut s, &mut w, &[4, 7, 12]);
    assert!(!tune_played(&ev));
    let mut w = s.enter("meadow-1", load("meadow-1"), "west", &Tuning::default());
    play(&mut s, &mut w, &[4, 7, 12]);
    assert_eq!(s.save.flag("luma_song.verses"), 1);
}

#[test]
fn a_tune_counts_only_on_its_map_and_while_its_condition_holds() {
    let mut s = Session::new_game(content_with_tune("flag knows_song"));
    let mut w = s.enter("tauwinkel", load("tauwinkel"), "west", &Tuning::default());
    s.save.set_flag("knows_song", 1);
    assert!(
        !tune_played(&play(&mut s, &mut w, &[4, 7, 12])),
        "other map"
    );
    s.save.set_flag("knows_song", 0);
    let mut w = s.enter("meadow-1", load("meadow-1"), "west", &Tuning::default());
    let ev = play(&mut s, &mut w, &[4, 7, 12]);
    assert!(!tune_played(&ev), "condition does not hold");
    assert_eq!(s.save.flag("luma_song.verses"), 0);
    // it is not used up: once the condition holds, playing it counts
    s.save.set_flag("knows_song", 1);
    assert!(tune_played(&play(&mut s, &mut w, &[4, 7, 12])));
    assert_eq!(s.save.flag("luma_song.verses"), 1);
}

// ---------------------------------------------------------------- Night tea (M2.5.6)

#[test]
fn night_tea_heals_and_makes_eloras_light_reach_further_for_a_while() {
    use elora_adventure::data::{Effect, ItemDef, ItemKind, Text};
    let mut c = Content::builtin();
    c.items.insert(
        "night_tea".into(),
        ItemDef {
            id: "night_tea".into(),
            name: Text::default(),
            desc: Text::default(),
            price: 18,
            kind: ItemKind::Consumable {
                effect: Effect::Glow(2),
            },
        },
    );
    let mut s = Session::new_game(c);
    let mut w = s.enter("meadow-1", load("meadow-1"), "west", &Tuning::default());
    step(&mut s, &mut w, PlayerInput::default(), false);
    assert!((s.light_reach() - 1.0).abs() < 1e-6);
    let full = s.save.max_health(&s.content);
    s.save.health = full - 3;
    s.save.inventory.insert("night_tea".into(), 1);
    s.use_item(&mut w, "night_tea").unwrap();
    assert_eq!(s.save.health, full - 1, "heals 2");
    assert_eq!(w.character(s.player).unwrap().health, full - 1);
    assert!((s.light_reach() - 2.0).abs() < 1e-6, "twice as far");
    // it lasts across a map change and fades out at the end
    for _ in 0..20 * 50 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    let mut w = s.enter("tauwinkel", load("tauwinkel"), "west", &Tuning::default());
    assert!((s.light_reach() - 2.0).abs() < 1e-6);
    let mut last = 2.0;
    for _ in 0..10 * 50 {
        step(&mut s, &mut w, PlayerInput::default(), false);
        assert!(s.light_reach() <= last, "never brighter again");
        last = s.light_reach();
    }
    assert!((s.light_reach() - 1.0).abs() < 1e-6, "over after 30 s");
}

// ---------------------------------------------------------------- Climax (M2.5.5)

/// `meadow-1` without enemies, with the zones of “Protect Luma”: Elora starts inside
/// `climax-start`, Luma stands in the middle, shadows come from the right, a lantern next to
/// Luma.
fn climax_map() -> elora_map::Map {
    let mut map = load("meadow-1");
    map.adventure
        .objects
        .retain(|o| !matches!(o.kind, elora_map::ObjectKind::Creature { .. }));
    let zone = |id: &str, x: f32, y: f32, w: f32, h: f32| elora_map::Object {
        id: id.into(),
        pos: Vec2::new(x * 32.0, y * 32.0),
        kind: elora_map::ObjectKind::Zone {
            size: Vec2::new(w * 32.0, h * 32.0),
        },
    };
    map.adventure.objects.extend([
        zone("climax-start", 0.0, 10.0, 12.0, 12.0),
        zone("climax-luma", 40.0, 19.0, 2.0, 2.0),
        zone("shade-spawn-1", 70.0, 10.0, 10.0, 4.0),
        zone("withered-1", 60.0, 8.0, 2.0, 3.0),
        zone("withered-2", 20.0, 8.0, 2.0, 3.0),
        zone("lantern-1", 41.0, 19.0, 1.0, 2.0),
    ]);
    map
}

fn climax_start() -> (Session, World) {
    let mut s = Session::new_game(Content::builtin());
    s.save.set_flag(elora_adventure::session::LUMA_SINGS, 1);
    let w = s.enter("meadow-1", climax_map(), "west", &Tuning::default());
    (s, w)
}

/// Steps until `done` holds (at most `ticks`); the lantern burns while `lantern` is set.
fn climax_run(
    s: &mut Session,
    w: &mut World,
    ticks: u32,
    lantern: bool,
    mut done: impl FnMut(&Session, &[SessionEvent]) -> bool,
) -> Vec<SessionEvent> {
    let mut all = Vec::new();
    for _ in 0..ticks {
        if lantern {
            w.lanterns[0].lit_until = w.tick + 100;
        }
        let ev = step(s, w, PlayerInput::default(), false);
        let stop = done(s, &ev);
        all.extend(ev);
        if stop {
            break;
        }
    }
    all
}

#[test]
fn climax_waits_until_luma_is_ready_to_sing() {
    let (mut s, mut w) = climax_start();
    s.save.set_flag(elora_adventure::session::LUMA_SINGS, 0);
    step(&mut s, &mut w, PlayerInput::default(), false);
    assert!(
        s.climax.is_none(),
        "Elora has not talked to Luma at the ruins yet"
    );
    s.save.set_flag(elora_adventure::session::LUMA_SINGS, 1);
    step(&mut s, &mut w, PlayerInput::default(), false);
    assert!(
        s.climax.is_some(),
        "standing in the zone is enough afterwards"
    );
}

#[test]
fn climax_starts_in_its_zone_with_the_withered_one() {
    let (mut s, mut w) = climax_start();
    assert!(s.climax.is_none());
    step(&mut s, &mut w, PlayerInput::default(), false);
    let c = s.climax.as_ref().expect("Elora stands in climax-start");
    assert_eq!((c.verse, c.light), (0, 1.0));
    assert_eq!(c.luma, Vec2::new(41.0 * 32.0, 20.0 * 32.0));
    let withered = w.creature_kind("withered").unwrap();
    assert!(w.creatures.iter().any(|x| x.kind == withered));
    // shadows come and drift towards Luma
    climax_run(&mut s, &mut w, 200, true, |_, _| false);
    let shade = w.creature_kind("grey_shade").unwrap();
    assert!(
        w.creatures
            .iter()
            .any(|x| x.kind == shade && x.goal == c_luma(&s))
    );
}

fn c_luma(s: &Session) -> Vec2 {
    s.climax.as_ref().unwrap().luma
}

#[test]
fn shadows_on_luma_put_her_light_out_and_send_elora_back_without_loss() {
    let (mut s, mut w) = climax_start();
    s.save.gleam_drops = 40;
    s.save.gleam_since_save = 40;
    let mut dimmed = false;
    let ev = climax_run(&mut s, &mut w, 60 * 50, false, |s, ev| {
        if let Some(c) = &s.climax
            && c.light < 1.0
        {
            dimmed = true;
        }
        ev.iter().any(|e| matches!(e, SessionEvent::Died { .. }))
    });
    assert!(dimmed, "Luma's light went down shadow by shadow");
    assert!(ev.contains(&SessionEvent::Died { lost: 0 }));
    assert!(s.dead && s.climax.is_none());
    assert_eq!(s.save.gleam_drops, 40, "no loss (E-362)");
    assert_eq!(s.save.flag(elora_adventure::session::LUMA_PROTECTED), 0);
    let shade = w.creature_kind("grey_shade").unwrap();
    assert!(w.creatures.iter().all(|x| x.kind != shade), "shadows gone");
    // back at the spring stone the climax is ready again
    let loc = s.save.location.clone();
    let mut w = s.enter(&loc.map, climax_map(), &loc.spawn, &Tuning::default());
    step(&mut s, &mut w, PlayerInput::default(), false);
    assert!(s.climax.is_some());
}

#[test]
fn lit_lanterns_protect_luma_through_three_verses_then_he_flees() {
    use elora_adventure::session::{ClimaxPhase, LUMA_PROTECTED};
    let (mut s, mut w) = climax_start();
    let mut phases = Vec::new();
    let kind = w.creature_kind("withered").unwrap();
    climax_run(&mut s, &mut w, 140 * 50, true, |s, _| {
        if let Some(c) = &s.climax {
            let p = std::mem::discriminant(&c.phase);
            if phases.last() != Some(&p) {
                phases.push(p);
            }
        }
        s.climax.is_none()
    });
    assert!(s.climax.is_none(), "over");
    assert_eq!(s.save.flag(LUMA_PROTECTED), 1);
    let names = [
        ClimaxPhase::Song,
        ClimaxPhase::Moving(0),
        ClimaxPhase::Song,
        ClimaxPhase::Moving(0),
        ClimaxPhase::Song,
        ClimaxPhase::Looking(0),
        ClimaxPhase::Fleeing(0),
    ]
    .map(|p| std::mem::discriminant(&p));
    assert_eq!(phases, names);
    assert!(w.creatures.iter().all(|x| x.kind != kind), "he is gone");
    // walking into the zone again does not start it again
    climax_run(&mut s, &mut w, 10, false, |_, _| false);
    assert!(s.climax.is_none());
}

#[test]
fn withered_one_is_heard_when_he_appears_moves_on_and_flees() {
    use elora_sim::{CreatureAct, Event};
    let (mut s, mut w) = climax_start();
    let mut heard = Vec::new();
    for _ in 0..140 * 50 {
        w.lanterns[0].lit_until = w.tick + 100;
        step(&mut s, &mut w, PlayerInput::default(), false);
        let him = w.creature_kind("withered");
        heard.extend(w.events.iter().filter_map(|e| {
            match *e {
                Event::CreatureAct { id, act, .. }
                    if w.creatures
                        .iter()
                        .any(|c| c.id == id && Some(c.kind) == him) =>
                {
                    Some(act)
                }
                _ => None,
            }
        }));
        if s.climax.is_none() {
            break;
        }
    }
    // he appears, moves on after the first and the second verse, and flees after the third
    assert_eq!(
        heard,
        [
            CreatureAct::Wake,
            CreatureAct::Dive,
            CreatureAct::Dive,
            CreatureAct::Flee
        ]
    );
}

#[test]
fn withered_one_moves_to_his_next_place_between_verses() {
    use elora_adventure::session::ClimaxPhase;
    let (mut s, mut w) = climax_start();
    climax_run(&mut s, &mut w, 60 * 50, true, |s, _| {
        s.climax
            .as_ref()
            .is_some_and(|c| c.verse == 1 && c.phase == ClimaxPhase::Song)
    });
    let kind = w.creature_kind("withered").unwrap();
    let him = w.creatures.iter().find(|x| x.kind == kind).unwrap();
    let place = Vec2::new(21.0 * 32.0, 9.5 * 32.0);
    assert!(
        him.pos.distance(place) < 1.0,
        "at withered-2: {:?}",
        him.pos
    );
}

#[test]
fn complete_song_makes_the_verse_faster() {
    let song_after = |bonus: bool| {
        let (mut s, mut w) = climax_start();
        if bonus {
            s.save.set_flag(elora_adventure::session::LUMA_SONG, 1);
        }
        climax_run(&mut s, &mut w, 500, true, |_, _| false);
        s.climax.as_ref().unwrap().song
    };
    let (plain, strong) = (song_after(false), song_after(true));
    assert!(strong > plain * 1.4, "{plain} → {strong}");
}

#[test]
fn leaving_the_map_ends_a_running_climax() {
    let (mut s, mut w) = climax_start();
    step(&mut s, &mut w, PlayerInput::default(), false);
    assert!(s.climax.is_some());
    s.enter("tauwinkel", load("tauwinkel"), "west", &Tuning::default());
    assert!(s.climax.is_none());
}

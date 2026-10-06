//! Wetter in der Simulation (R2-W1, W1.4, E-330, E-336): Wind, Nässe, Granaten, Blitze –
//! nur im Abenteuer.

use elora_sim::{Collision, Event, PlayerInput, Tile, Tuning, Vec2, Weapon, WeatherEnv, World};

const W: usize = 80;
const H: usize = 30;
const FLOOR: usize = 26;

fn world(adventure: bool, weather: Option<WeatherEnv>) -> World {
    let mut tiles = vec![Tile::Air; W * H];
    for y in FLOOR..H {
        for x in 0..W {
            tiles[y * W + x] = Tile::Solid;
        }
    }
    let mut w = World::new(Tuning::default(), Collision::new(W, H, tiles));
    w.adventure = adventure;
    w.weather = weather;
    let i = w.join();
    w.spawn_character(i, Vec2::new(40.0 * 32.0, FLOOR as f32 * 32.0 - 15.0));
    w.character_mut(0).unwrap().invulnerable_until = 0;
    w
}

#[allow(clippy::unnecessary_wraps)]
fn wind(v: f32) -> Option<WeatherEnv> {
    Some(WeatherEnv {
        wind: v,
        ..WeatherEnv::default()
    })
}

fn run(w: &mut World, input: PlayerInput, ticks: u32) -> Vec<Event> {
    let mut all = Vec::new();
    for _ in 0..ticks {
        w.step(&[input]);
        all.extend(w.events.iter().cloned());
    }
    all
}

fn x(w: &World) -> f32 {
    w.character(0).unwrap().core.pos.x
}

/// Gerader Sprung: wie weit treibt Elora seitlich ab?
fn drift(adventure: bool, weather: Option<WeatherEnv>) -> f32 {
    let mut w = world(adventure, weather);
    run(&mut w, PlayerInput::default(), 10);
    let start = x(&w);
    let jump = PlayerInput {
        jump: true,
        ..PlayerInput::default()
    };
    run(&mut w, jump, 1);
    run(&mut w, PlayerInput::default(), 25);
    x(&w) - start
}

#[test]
fn wind_pushes_elora_in_the_air_only_in_the_adventure() {
    let calm = drift(true, None);
    let windy = drift(true, wind(1.0));
    assert!(calm.abs() < 0.5, "ohne Wind gerade: {calm}");
    assert!(windy > 20.0, "Wind treibt nach rechts: {windy}");
    assert!(drift(true, wind(-1.0)) < -20.0, "und nach links");
    assert!(
        drift(false, wind(1.0)).abs() < 0.5,
        "Mehrspieler: kein Wind"
    );
    // am Boden schiebt er nicht
    let mut w = world(true, wind(1.0));
    run(&mut w, PlayerInput::default(), 10);
    let start = x(&w);
    run(&mut w, PlayerInput::default(), 50);
    assert!((x(&w) - start).abs() < 0.5, "steht fest");
}

/// Bremsweg nach dem Loslassen: nasser Boden bremst weicher.
fn stop_distance(weather: Option<WeatherEnv>) -> f32 {
    let mut w = world(true, weather);
    let right = PlayerInput {
        direction: 1,
        ..PlayerInput::default()
    };
    run(&mut w, right, 40);
    let start = x(&w);
    run(&mut w, PlayerInput::default(), 40);
    x(&w) - start
}

#[test]
fn wet_ground_brakes_softer() {
    let dry = stop_distance(None);
    let wet = stop_distance(Some(WeatherEnv {
        wet: 1.0,
        ..WeatherEnv::default()
    }));
    assert!(wet > dry * 1.2, "nass {wet} statt trocken {dry}");
}

#[test]
fn wind_bends_grenades() {
    let landing = |weather| {
        let mut w = world(true, weather);
        w.character_mut(0)
            .unwrap()
            .arsenal
            .give(Weapon::Grenade, 10, 10);
        w.character_mut(0).unwrap().arsenal.active = Weapon::Grenade;
        let fire = PlayerInput {
            fire: 1,
            target_x: 0,
            target_y: -100,
            ..PlayerInput::default()
        };
        // erst ein paar Ticks: das Wetter liegt ab dem ersten Tick in der Kollision
        run(&mut w, PlayerInput::default(), 5);
        run(&mut w, fire, 1);
        let ev = run(&mut w, PlayerInput::default(), 200);
        ev.iter()
            .find_map(|e| match e {
                Event::Explosion { pos, .. } => Some(pos.x),
                _ => None,
            })
            .expect("Granate explodiert")
    };
    let calm = landing(None);
    let windy = landing(wind(1.0));
    assert!(
        windy - calm > 30.0,
        "Wind trägt die Granate: {calm} → {windy}"
    );
}

#[test]
fn lightning_warns_then_strikes_and_hurts() {
    let storm = Some(WeatherEnv {
        lightning: 1.0,
        ..WeatherEnv::default()
    });
    let mut w = world(true, storm);
    let mut warn = None;
    let mut strike = None;
    for t in 0..3000u32 {
        w.step(&[PlayerInput::default()]);
        for e in &w.events {
            match e {
                Event::LightningWarn { pos } if warn.is_none() => warn = Some((t, *pos)),
                Event::Lightning { pos } if warn.is_some() && strike.is_none() => {
                    strike = Some((t, *pos));
                }
                _ => {}
            }
        }
        if strike.is_some() {
            break;
        }
    }
    let (tw, pw) = warn.expect("Warnung");
    let (ts, ps) = strike.expect("Einschlag");
    assert_eq!(pw, ps, "schlägt dort ein, wo es glimmt");
    assert!(ts - tw >= 40, "Zeit zum Ausweichen: {} Ticks", ts - tw);
    assert!((ps.y - FLOOR as f32 * 32.0).abs() < 0.5, "am Boden");
    // Elora genau an der Einschlagstelle: Schaden
    let mut w = world(true, storm);
    let before = w.character(0).unwrap().health;
    let mut hurt = false;
    for _ in 0..3000 {
        w.step(&[PlayerInput::default()]);
        let warned = w.events.iter().find_map(|e| match e {
            Event::LightningWarn { pos } => Some(*pos),
            _ => None,
        });
        if let Some(p) = warned {
            w.spawn_character(0, p - Vec2::new(0.0, 15.0));
        }
        if w.events
            .iter()
            .any(|e| matches!(e, Event::Lightning { .. }))
        {
            hurt = w.character(0).is_none_or(|c| c.health < before);
            break;
        }
    }
    assert!(hurt, "Blitz trifft Elora");
    // im Mehrspieler keine Blitze
    let mut w = world(false, storm);
    let ev = run(&mut w, PlayerInput::default(), 3000);
    assert!(!ev.iter().any(|e| matches!(e, Event::Lightning { .. })));
}

//! Verhaltenstests der Bewegungsphysik gegen die Werte aus docs/04-tuning.md.

use elora_sim::{Collision, HookState, PlayerInput, Tile, Tuning, Vec2, World};

/// Karte aus ASCII: `#` Wand, `%` unhookable, `^` Tod, sonst Luft.
fn world(rows: &[&str]) -> World {
    let width = rows[0].len();
    let tiles = rows
        .iter()
        .flat_map(|r| {
            assert_eq!(r.len(), width);
            r.chars().map(|c| match c {
                '#' => Tile::Solid,
                '%' => Tile::Unhookable,
                '^' => Tile::Death,
                _ => Tile::Air,
            })
        })
        .collect();
    World::new(Tuning::default(), Collision::new(width, rows.len(), tiles))
}

/// Große offene Arena (60×40 Tiles) mit Boden in Zeile 38.
fn arena() -> World {
    let mut rows = vec!["#".to_string() + &".".repeat(58) + "#"; 40];
    rows[0] = "#".repeat(60);
    rows[38] = "#".repeat(60);
    rows[39] = "#".repeat(60);
    let refs: Vec<&str> = rows.iter().map(String::as_str).collect();
    world(&refs)
}

/// Mittelpunkt des Tiles (tx, ty).
fn tile_center(tx: i32, ty: i32) -> Vec2 {
    Vec2::new(tx as f32 * 32.0 + 16.0, ty as f32 * 32.0 + 16.0)
}

fn run(w: &mut World, input: PlayerInput, ticks: u32) {
    for _ in 0..ticks {
        w.step(&[input]);
    }
}

fn core(w: &World) -> &elora_sim::CharacterCore {
    w.core(0).expect("Figur 0 lebt")
}

/// Lässt die Figur landen und zur Ruhe kommen.
fn landed(w: &mut World, tx: i32) {
    w.spawn(tile_center(tx, 36));
    run(w, PlayerInput::default(), 60);
    assert!(core(w).is_grounded(&w.collision), "Figur sollte stehen");
}

#[test]
fn stands_on_ground() {
    let mut w = arena();
    landed(&mut w, 30);
    // Boden-Oberkante 38*32 = 1216, halbe Box 14
    assert!((core(&w).pos.y - (1216.0 - 14.0)).abs() <= 1.0);
    assert!(core(&w).vel.length() < 0.01);
}

#[test]
fn ground_speed_is_capped() {
    let mut w = arena();
    landed(&mut w, 5);
    let right = PlayerInput {
        direction: 1,
        ..PlayerInput::default()
    };
    run(&mut w, right, 20);
    let vx = core(&w).vel.x;
    assert!((vx - 10.5).abs() < 0.01, "T-02: erwartet 10.5, war {vx}");
}

#[test]
fn jump_and_double_jump_heights() {
    let mut w = arena();
    landed(&mut w, 30);
    let start_y = core(&w).pos.y;

    // einfacher Sprung (T-05)
    let jump = PlayerInput {
        jump: true,
        ..PlayerInput::default()
    };
    let mut min_y = start_y;
    for _ in 0..60 {
        w.step(&[jump]);
        min_y = min_y.min(core(&w).pos.y);
    }
    let single = start_y - min_y;
    assert!((180.0..=200.0).contains(&single), "Sprunghöhe {single}");

    // Doppelsprung (T-06): am Scheitelpunkt loslassen und erneut drücken
    run(&mut w, PlayerInput::default(), 60);
    let mut min_y = start_y;
    for t in 0..120 {
        let input = if t == 28 {
            PlayerInput::default()
        } else {
            jump
        };
        w.step(&[input]);
        min_y = min_y.min(core(&w).pos.y);
    }
    let double = start_y - min_y;
    assert!(
        (310.0..=340.0).contains(&double),
        "Doppelsprunghöhe {double}"
    );
}

#[test]
fn holding_jump_does_not_rejump() {
    let mut w = arena();
    landed(&mut w, 30);
    let jump = PlayerInput {
        jump: true,
        ..PlayerInput::default()
    };
    run(&mut w, jump, 200);
    // nach der Landung bleibt die Figur trotz gehaltener Taste am Boden
    assert!(core(&w).is_grounded(&w.collision));
    assert!(core(&w).vel.y.abs() < 0.01);
}

#[test]
fn hook_in_open_air_retracts_at_max_length() {
    let mut w = arena();
    landed(&mut w, 5);
    let hook = PlayerInput {
        hook: true,
        target_x: 100,
        target_y: 0,
        ..PlayerInput::default()
    };
    let start = core(&w).pos;
    let mut max = 0.0f32;
    for _ in 0..10 {
        w.step(&[hook]);
        max = max.max(core(&w).hook_pos.distance(start));
    }
    // Wie im Original wird die Hook-Position beim Erreichen der Maximallänge nicht
    // mehr aktualisiert (sichtbar: letzter Flugschritt); die Treffer-Prüfung reicht
    // aber bis zur vollen Länge, siehe `hook_reaches_full_length`.
    assert!(max <= 400.0, "Hook zu lang: {max}");
    assert_eq!(core(&w).hook_state, HookState::Retracted);

    // loslassen → Idle, erneutes Drücken schießt wieder
    w.step(&[PlayerInput::default()]);
    assert_eq!(core(&w).hook_state, HookState::Idle);
    w.step(&[hook]);
    assert_eq!(core(&w).hook_state, HookState::Flying);
}

/// Wie im Original greift der Hook im Tick, in dem er die Maximallänge
/// überschreitet, nicht an Wänden. Effektive Wand-Reichweite ist daher der letzte
/// volle Flugschritt: 42 + 4 · 85 = 382 (Original: 42 + 4 · 80 = 362).
#[test]
fn hook_wall_reach_follows_flight_steps() {
    let hook = PlayerInput {
        hook: true,
        target_x: 100,
        target_y: 0,
        ..PlayerInput::default()
    };
    let mut rows = vec!["#..............#####"; 6];
    rows[5] = "####################";

    // Wand ab x = 480, Abstand 380 → greift
    let mut w = world(&rows);
    w.spawn(Vec2::new(100.0, 4.0 * 32.0 + 16.0));
    run(&mut w, PlayerInput::default(), 20);
    run(&mut w, hook, 6);
    assert_eq!(core(&w).hook_state, HookState::Grabbed);
    assert!((core(&w).hook_pos.x - 480.0).abs() <= 1.0);

    // Abstand 395 → liegt im gekappten Schritt, greift nicht
    let mut w = world(&rows);
    w.spawn(Vec2::new(85.0, 4.0 * 32.0 + 16.0));
    run(&mut w, PlayerInput::default(), 20);
    let mut grabbed = false;
    for _ in 0..8 {
        w.step(&[hook]);
        grabbed |= core(&w).hook_state == HookState::Grabbed;
    }
    assert!(!grabbed);
}

#[test]
fn hook_grabs_ceiling_and_pulls_up() {
    let mut rows = vec!["#......................#"; 14];
    rows[0] = "########################";
    rows[13] = "########################";
    let mut w = world(&rows);
    w.spawn(tile_center(10, 11));
    run(&mut w, PlayerInput::default(), 30);
    let ground_y = core(&w).pos.y;

    let hook = PlayerInput {
        hook: true,
        target_x: 0,
        target_y: -100,
        ..PlayerInput::default()
    };
    run(&mut w, hook, 6);
    assert_eq!(core(&w).hook_state, HookState::Grabbed);
    run(&mut w, hook, 40);
    assert!(core(&w).pos.y < ground_y - 150.0, "Hook sollte hochziehen");
}

#[test]
fn unhookable_makes_hook_retract() {
    let mut rows = vec!["#......................#"; 14];
    rows[0] = "%%%%%%%%%%%%%%%%%%%%%%%%";
    rows[13] = "########################";
    let mut w = world(&rows);
    w.spawn(tile_center(10, 11));
    run(&mut w, PlayerInput::default(), 30);
    let hook = PlayerInput {
        hook: true,
        target_x: 0,
        target_y: -100,
        ..PlayerInput::default()
    };
    let mut grabbed = false;
    for _ in 0..10 {
        w.step(&[hook]);
        grabbed |= core(&w).hook_state == HookState::Grabbed;
    }
    assert!(!grabbed);
    assert_eq!(core(&w).hook_state, HookState::Retracted);
}

#[test]
fn death_tile_kills() {
    let mut w = world(&["#.....#", "#.....#", "#.....#", "#^^^^^#", "#######"]);
    w.spawn(tile_center(3, 1));
    let mut died = false;
    for _ in 0..30 {
        w.step(&[PlayerInput::default()]);
        died |= w.events.iter().any(|e| {
            matches!(
                e,
                elora_sim::Event::Death {
                    player: 0,
                    cause: elora_sim::DeathCause::World,
                    ..
                }
            )
        });
    }
    assert!(died);
    assert!(w.character(0).is_none());
}

#[test]
fn player_hook_releases_after_limit() {
    let mut w = arena();
    w.spawn(tile_center(10, 36));
    w.spawn(tile_center(18, 36));
    let idle = PlayerInput::default();
    for _ in 0..60 {
        w.step(&[idle, idle]);
    }
    let hook = PlayerInput {
        hook: true,
        target_x: 100,
        target_y: 0,
        ..PlayerInput::default()
    };
    let mut grabbed_ticks = 0;
    for _ in 0..120 {
        w.step(&[hook, idle]);
        if core(&w).hooked_player == Some(1) {
            grabbed_ticks += 1;
        }
    }
    // T-16: 55 Ticks (+1 wegen `>`-Vergleich wie im Original)
    assert!(
        (55..=57).contains(&grabbed_ticks),
        "gehalten: {grabbed_ticks} Ticks"
    );
    assert_eq!(core(&w).hook_state, HookState::Retracted);
}

#[test]
fn simulation_is_deterministic() {
    let script = |t: i32| PlayerInput {
        direction: [1, 1, 0, -1][usize::try_from(t / 17 % 4).unwrap()],
        target_x: (t * 37 % 200) - 100,
        target_y: -((t * 13) % 150) - 1,
        jump: t % 23 < 5,
        hook: t % 40 > 12,
        ..PlayerInput::default()
    };
    let simulate = || {
        let mut w = arena();
        w.spawn(tile_center(20, 30));
        for t in 0..1000 {
            w.step(&[script(t)]);
        }
        core(&w).clone()
    };
    assert_eq!(simulate(), simulate());
}

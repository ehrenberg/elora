use elora_sim::{Collision, DeathCause, PlayerInput, Tile, Tuning, Vec2, Weapon};

use super::*;

/// Offene Arena 60×20 Tiles mit Boden in Zeile 18, Spawns links (Rot) und rechts (Blau),
/// Flaggenstände an beiden Enden.
fn arena() -> World {
    let (w, h) = (60, 20);
    let mut tiles = vec![Tile::Air; w * h];
    for x in 0..w {
        tiles[x] = Tile::Solid;
        tiles[18 * w + x] = Tile::Solid;
        tiles[19 * w + x] = Tile::Solid;
    }
    for y in 0..h {
        tiles[y * w] = Tile::Solid;
        tiles[y * w + w - 1] = Tile::Solid;
    }
    let mut world = World::new(Tuning::default(), Collision::new(w, h, tiles));
    let ground = |tx: f32| Vec2::new(tx * 32.0 + 16.0, 18.0 * 32.0 - 15.0);
    world.spawn_points.push(ground(30.0));
    world.team_spawns[0].push(ground(5.0));
    world.team_spawns[1].push(ground(54.0));
    world.flag_stands = [Some(ground(3.0)), Some(ground(56.0))];
    world
}

fn cfg(mode: Mode) -> RulesConfig {
    RulesConfig {
        mode,
        ..RulesConfig::default()
    }
}

/// Welt mit `n` Spielern und laufenden Regeln (Countdown übersprungen).
fn game(mode: Mode, n: usize) -> (World, Rules) {
    let mut w = arena();
    for _ in 0..n {
        w.join();
    }
    let mut r = Rules::new(cfg(mode), &mut w, false);
    run(&mut w, &mut r, 3 * 50 + 2);
    assert_eq!(r.phase, Phase::Running, "Match sollte laufen");
    (w, r)
}

fn run(w: &mut World, r: &mut Rules, ticks: u32) {
    for _ in 0..ticks {
        w.step(&[]);
        r.update(w);
    }
}

/// Tod mit anschließender Auswertung (wie nach einem Tick).
fn kill(w: &mut World, r: &mut Rules, victim: usize, killer: Option<usize>) {
    w.events.clear();
    w.die(victim, killer, DeathCause::Weapon(Weapon::Laser));
    r.update(w);
}

fn score(r: &Rules, i: usize) -> i32 {
    r.stats[&i].score
}

#[test]
fn waits_for_players_then_counts_down() {
    let mut w = arena();
    w.join();
    let mut r = Rules::new(cfg(Mode::Dm), &mut w, false);
    run(&mut w, &mut r, 10);
    assert_eq!(
        r.phase,
        Phase::Warmup { until: None },
        "allein: unbegrenztes Aufwärmen"
    );
    let i = w.join();
    r.on_join(&mut w, i);
    run(&mut w, &mut r, 1);
    assert!(matches!(r.phase, Phase::Countdown { .. }));
    assert!(w.paused, "Countdown friert die Welt ein");
    let before = w.core(0).unwrap().pos;
    run(&mut w, &mut r, 100);
    assert_eq!(w.core(0).unwrap().pos, before, "eingefroren");
    run(&mut w, &mut r, 60);
    assert_eq!(r.phase, Phase::Running);
    assert!(!w.paused);
}

#[test]
fn fresh_map_warms_up_ten_seconds() {
    let mut w = arena();
    w.join();
    w.join();
    let mut r = Rules::new(cfg(Mode::Dm), &mut w, true);
    run(&mut w, &mut r, 9 * 50);
    assert!(matches!(r.phase, Phase::Warmup { until: Some(_) }));
    kill(&mut w, &mut r, 1, Some(0));
    assert_eq!(score(&r, 0), 0, "Punkte zählen im Aufwärmen nicht");
    run(&mut w, &mut r, 60);
    assert!(matches!(r.phase, Phase::Countdown { .. }));
}

#[test]
fn dm_scoring_and_limit() {
    let (mut w, mut r) = game(Mode::Dm, 2);
    kill(&mut w, &mut r, 1, Some(0));
    assert_eq!(score(&r, 0), 1);
    run(&mut w, &mut r, 200);
    w.events.clear();
    w.kill(0);
    r.update(&mut w);
    assert_eq!(score(&r, 0), 0, "Selbstmord −1");
    r.stats.get_mut(&0).unwrap().score = 19;
    run(&mut w, &mut r, 200);
    kill(&mut w, &mut r, 1, Some(0));
    run(&mut w, &mut r, 1);
    assert!(matches!(r.phase, Phase::MatchOver { .. }));
    assert!(
        r.take_events()
            .contains(&GameEvent::MatchOver(Winner::Player(0)))
    );
}

#[test]
fn sudden_death_on_tie_at_time_limit() {
    let mut w = arena();
    w.join();
    w.join();
    let mut r = Rules::new(
        RulesConfig {
            time_limit: 1,
            ..cfg(Mode::Dm)
        },
        &mut w,
        false,
    );
    run(&mut w, &mut r, 3 * 50 + 60 * 50 + 5);
    assert_eq!(r.phase, Phase::Running, "Gleichstand → weiter");
    assert!(r.sudden_death);
    kill(&mut w, &mut r, 1, Some(0));
    run(&mut w, &mut r, 1);
    assert!(matches!(r.phase, Phase::MatchOver { .. }));
}

#[test]
fn tdm_teams_scores_friendly_fire_and_respawn_delay() {
    let (mut w, mut r) = game(Mode::Tdm, 4);
    let teams: Vec<Team> = (0..4).map(|i| w.team(i)).collect();
    assert_eq!(
        teams,
        [Team::Red, Team::Blue, Team::Red, Team::Blue],
        "abwechselnd ins kleinere Team"
    );
    kill(&mut w, &mut r, 1, Some(0));
    assert_eq!((score(&r, 0), r.team_score), (1, [1, 0]));
    // Teamkill (Friendly Fire an, E-069)
    kill(&mut w, &mut r, 2, Some(0));
    assert_eq!((score(&r, 0), r.team_score), (0, [0, 0]));
    // Respawn frühestens nach 3 s (E-070)
    run(&mut w, &mut r, 100);
    let fire = PlayerInput {
        fire: 1,
        ..PlayerInput::default()
    };
    for _ in 0..30 {
        w.step(&[fire, fire, fire, fire]);
        r.update(&mut w);
    }
    assert!(w.character(1).is_none(), "vor 3 s kein Respawn");
    run(&mut w, &mut r, 30);
    assert!(w.character(1).is_some());
}

#[test]
fn friendly_fire_off_keeps_knockback_only() {
    for ff in [false, true] {
        let mut w = arena();
        for _ in 0..3 {
            w.join();
        }
        let mut r = Rules::new(
            RulesConfig {
                friendly_fire: ff,
                ..cfg(Mode::Tdm)
            },
            &mut w,
            false,
        );
        run(&mut w, &mut r, 160);
        // 0 und 2 sind beide Rot; nebeneinander stellen
        assert!(w.team(0).is_mate(w.team(2)));
        let p = Vec2::new(20.0 * 32.0, 18.0 * 32.0 - 15.0);
        w.character_mut(0).unwrap().core.pos = p;
        w.character_mut(2).unwrap().core.pos = p + Vec2::new(30.0, 0.0);
        run(&mut w, &mut r, 2);
        let hammer = PlayerInput {
            fire: 1,
            target_x: 100,
            target_y: 0,
            ..PlayerInput::default()
        };
        w.step(&[hammer]);
        r.update(&mut w);
        let mate = w.character(2).unwrap();
        assert!(mate.core.vel.y < -3.0, "Rückstoß wirkt immer");
        let expected = if ff { 10 - 3 } else { 10 };
        assert_eq!(mate.health, expected, "Friendly Fire {ff}");
    }
}

#[test]
fn ctf_grab_capture_drop_return() {
    let (mut w, mut r) = game(Mode::Ctf, 2);
    assert_eq!(w.flags.len(), 2);
    let (red, blue) = (0, 1);
    assert_eq!((w.team(red), w.team(blue)), (Team::Red, Team::Blue));
    // Rot holt die blaue Flagge …
    let blue_stand = w.flags[1].stand;
    w.character_mut(red).unwrap().core.pos = blue_stand;
    run(&mut w, &mut r, 1);
    assert_eq!(w.flags[1].carrier, Some(red));
    assert_eq!(score(&r, red), 1, "Aufnehmen +1");
    // … und bringt sie zur eigenen Flagge
    let red_stand = w.flags[0].stand;
    w.character_mut(red).unwrap().core.pos = red_stand;
    run(&mut w, &mut r, 2);
    assert_eq!(r.team_score, [1, 0], "Eroberung = Teampunkt (E-067)");
    assert_eq!(score(&r, red), 1 + 5);
    assert!(w.flags[1].at_stand);

    // Blau holt die rote Flagge, stirbt → Flagge fällt, Mörder +1
    w.character_mut(blue).unwrap().core.pos = red_stand;
    run(&mut w, &mut r, 1);
    assert_eq!(w.flags[0].carrier, Some(blue));
    w.character_mut(blue).unwrap().core.pos = Vec2::new(30.0 * 32.0, 18.0 * 32.0 - 15.0);
    run(&mut w, &mut r, 1);
    let before = score(&r, red);
    kill(&mut w, &mut r, blue, Some(red));
    assert_eq!(w.flags[0].carrier, None);
    assert_eq!(score(&r, red), before + 2, "Kill +1, Flaggenträger +1");
    // Rot bringt die eigene Flagge zurück
    let dropped = w.flags[0].pos;
    w.character_mut(red).unwrap().core.pos = dropped;
    run(&mut w, &mut r, 2);
    assert!(w.flags[0].at_stand, "zurückgebracht");
}

#[test]
fn ctf_flag_returns_after_30_seconds() {
    let (mut w, mut r) = game(Mode::Ctf, 2);
    let red_stand = w.flags[0].stand;
    w.character_mut(1).unwrap().core.pos = red_stand;
    run(&mut w, &mut r, 1);
    w.character_mut(1).unwrap().core.pos = Vec2::new(30.0 * 32.0, 18.0 * 32.0 - 15.0);
    run(&mut w, &mut r, 1);
    kill(&mut w, &mut r, 1, None);
    w.character_mut(0).unwrap().core.pos = Vec2::new(50.0 * 32.0, 18.0 * 32.0 - 15.0);
    run(&mut w, &mut r, 29 * 50);
    assert!(!w.flags[0].at_stand);
    run(&mut w, &mut r, 60);
    assert!(w.flags[0].at_stand, "nach 30 s zurück");
}

#[test]
fn lms_no_respawn_and_round_winner() {
    let (mut w, mut r) = game(Mode::Lms, 3);
    kill(&mut w, &mut r, 1, Some(0));
    run(&mut w, &mut r, 200);
    assert!(w.character(1).is_none(), "kein Respawn in der Runde");
    kill(&mut w, &mut r, 2, Some(0));
    run(&mut w, &mut r, 1);
    assert!(matches!(r.phase, Phase::RoundOver { .. }));
    assert_eq!(score(&r, 0), 2 + 1, "2 Kills + Rundensieg");
    run(&mut w, &mut r, 5 * 50 + 1);
    assert!(
        matches!(r.phase, Phase::Countdown { .. }),
        "neue Runde mit Countdown (E-068)"
    );
    assert!(
        w.character(1).is_some() && w.character(2).is_some(),
        "alle wieder da"
    );
}

#[test]
fn lts_team_elimination() {
    let (mut w, mut r) = game(Mode::Lts, 3);
    // Blau hat nur Spieler 1
    kill(&mut w, &mut r, 1, Some(0));
    run(&mut w, &mut r, 1);
    assert_eq!(r.team_score, [1, 0]);
    assert!(
        r.take_events()
            .contains(&GameEvent::RoundOver(Winner::Team(Team::Red)))
    );
}

#[test]
fn instagib_laser_only_one_hit_no_pickups() {
    let mut w = arena();
    w.add_pickup(elora_sim::PickupKind::Health, Vec2::new(200.0, 561.0));
    w.join();
    w.join();
    let mut r = Rules::new(
        RulesConfig {
            instagib: true,
            ..cfg(Mode::Dm)
        },
        &mut w,
        false,
    );
    run(&mut w, &mut r, 160);
    let a = &w.character(0).unwrap().arsenal;
    assert_eq!(a.active, Weapon::Laser);
    assert!(!a.has(Weapon::Hammer));
    assert_eq!(a.slot(Weapon::Laser).ammo, None, "unbegrenzt");
    assert!(!w.pickups_enabled);
    assert!(w.tuning.laser_damage > w.tuning.max_health + w.tuning.max_armor);
}

#[test]
fn team_change_and_spectator() {
    let (mut w, mut r) = game(Mode::Tdm, 2);
    r.set_team(&mut w, 1, Team::Spectator);
    assert!(w.character(1).is_none());
    assert_eq!(score(&r, 1), 0, "Team-Wechsel wird nicht gewertet");
    run(&mut w, &mut r, 400);
    assert!(w.character(1).is_none(), "Zuschauer spawnt nie");
    assert_eq!(
        r.phase,
        Phase::Warmup { until: None },
        "Team leer → Aufwärmen"
    );
}

#[test]
fn balance_moves_player_after_a_minute() {
    let (mut w, mut r) = game(Mode::Tdm, 4);
    r.set_team(&mut w, 1, Team::Red);
    r.set_team(&mut w, 3, Team::Red);
    // 3 : 1 ist unausgewogen … aber Blau hat noch Spieler? nein → 4 Rot, 0 Blau
    r.set_team(&mut w, 2, Team::Red);
    r.set_team(&mut w, 0, Team::Blue);
    run(&mut w, &mut r, 10);
    // jetzt 3 Rot, 1 Blau
    run(&mut w, &mut r, 61 * 50);
    let sizes = Rules::team_sizes(&w);
    assert_eq!(sizes, [2, 2], "nach 1 min ausgeglichen");
}

#[test]
fn match_end_swaps_teams_and_requests_next_map() {
    let (mut w, mut r) = game(Mode::Tdm, 2);
    r.team_score = [19, 0];
    kill(&mut w, &mut r, 1, Some(0));
    run(&mut w, &mut r, 1);
    assert!(matches!(r.phase, Phase::MatchOver { .. }));
    let _ = r.take_events();
    run(&mut w, &mut r, 10 * 50 + 1);
    assert_eq!(w.team(0), Team::Blue, "Teamtausch nach dem Match (E-074)");
    assert!(r.take_events().contains(&GameEvent::NextMap));
    assert_eq!(r.team_score, [0, 0]);
}

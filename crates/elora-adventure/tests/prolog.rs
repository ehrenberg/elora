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

/// Zugschalter mit einem echten Hook-Schuss von unten umlegen (R2-M2.2): in `wald-3` und an
/// den Zugtruhen in Tüftels Hof, `wiese-1` und `wiese-2`.
#[test]
fn pull_levers_flip_with_a_real_hook_shot() {
    for (map, spawn, lever, flag) in [
        ("wald-3", "ost", "zug", "zug.wald3"),
        ("tauwinkel", "start", "hof-zug", "hof.zugtor"),
        ("wiese-1", "west", "wiese1-zug", "wiese1.zug"),
        ("wiese-2", "west", "wiese2-zug", "wiese2.zug"),
    ] {
        let tuning = Tuning::default();
        let mut s = Session::new_game(Content::builtin());
        let mut w = s.enter(map, load(map), spawn, &tuning);
        s.save
            .run(&s.content.clone(), &["faehigkeit heranhooken".into()]);
        s.sync_world(&mut w);
        let at = s.map.adventure.object(lever).unwrap().pos;
        // Elora darunter auf den Boden fallen lassen
        w.spawn_character(s.player, Vec2::new(at.x - 40.0, at.y + 120.0));
        for _ in 0..60 {
            step(&mut s, &mut w, PlayerInput::default(), false);
        }
        let me = w.character(s.player).unwrap().core.pos;
        let aim = at - me;
        for _ in 0..40 {
            let input = PlayerInput {
                hook: true,
                target_x: aim.x as i32,
                target_y: aim.y as i32,
                ..PlayerInput::default()
            };
            step(&mut s, &mut w, input, false);
        }
        assert_eq!(s.save.flag(flag), 1, "{map}: Schalter {lever}");
    }
}

fn travel(s: &mut Session, w: &mut World, from: &str, direction: i8) -> World {
    go_to(s, w, from, 0.0);
    let (map, spawn) = walk_until_travel(s, w, direction);
    s.enter(&map, load(&map), &spawn, &Tuning::default())
}

/// Durchlauf von Kapitel 3 (R2-M2.3) auf den mitgelieferten Karten: Hohlweg am Ostpfad,
/// Karawane, Oase mit Wasser aus den Ruinen, Kammer unter dem Bröckelboden (Stampfen),
/// Kampf an der Glutquelle, Tor und Weg zurück nach Tauwinkel.
#[test]
#[allow(clippy::too_many_lines)] // ein Durchlauf in der Reihenfolge des Kapitels
fn chapter_three_from_the_sunken_path_to_the_spring_and_home() {
    let tuning = Tuning::default();
    // vor Kapitel 3 ist der Hohlweg zu
    let mut s = Session::new_game(Content::builtin());
    let mut w = s.enter("tauwinkel", load("tauwinkel"), "hohlweg", &tuning);
    step(&mut s, &mut w, PlayerInput::default(), false);
    let lid = s.map.adventure.object("hohlweg-deckel").unwrap().pos;
    assert!(w.collision.tile_at(lid + Vec2::new(16.0, 16.0)).is_solid());

    let mut s = Session::new_game(Content::builtin());
    s.save
        .run(&s.content.clone(), &["quest glutsand start".into()]);
    let mut w = s.enter("tauwinkel", load("tauwinkel"), "hohlweg", &tuning);
    step(&mut s, &mut w, PlayerInput::default(), false);
    assert!(!w.collision.tile_at(lid + Vec2::new(16.0, 16.0)).is_solid());
    let (map, spawn) = walk_until_travel(&mut s, &mut w, 1);
    assert_eq!((map.as_str(), spawn.as_str()), ("wueste-1", "nord"));
    let mut w = s.enter(&map, load(&map), &spawn, &tuning);
    assert!(holds(&s, "quest glutsand schritt sirup"));
    assert!(s.hot());

    // Karawanenlager: Sirup und Palma
    let mut w = travel(&mut s, &mut w, "ost", 1);
    assert_eq!(s.map_name, "wueste-2");
    assert_eq!(talk(&mut s, &mut w, "sirup")[0], "begruessung");
    assert!(holds(&s, "quest glutsand schritt ruinen"));
    talk(&mut s, &mut w, "palma");
    assert!(holds(&s, "quest oase aktiv"));

    // Ruinen: Wasser holen, mit Stampfen in die Kammer
    let mut w = travel(&mut s, &mut w, "ost", 1);
    assert_eq!(s.map_name, "wueste-3");
    assert!(holds(&s, "quest glutsand schritt quelle"));
    talk(&mut s, &mut w, "ruinenquelle");
    assert_eq!(s.save.count("wasser"), 3);
    s.save
        .run(&s.content.clone(), &["quest ruine start".into()]);
    w.set_abilities(
        s.player,
        elora_sim::Abilities::NONE.with(elora_sim::Ability::Stomp),
    );
    // mitten auf dem Bröckelboden (Spalten 175 bis 179, Oberkante Zeile 46)
    let top = Vec2::new(177.5 * 32.0, 46.0 * 32.0 - 20.0);
    w.spawn_character(s.player, top);
    w.set_abilities(
        s.player,
        elora_sim::Abilities::NONE.with(elora_sim::Ability::Stomp),
    );
    for _ in 0..20 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(holds(&s, "quest ruine schritt kammer"), "der Boden trägt");
    let down = PlayerInput {
        down: true,
        ..PlayerInput::default()
    };
    step(
        &mut s,
        &mut w,
        PlayerInput {
            jump: true,
            ..PlayerInput::default()
        },
        false,
    );
    for _ in 0..12 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    for _ in 0..80 {
        step(&mut s, &mut w, down, false);
    }
    for _ in 0..40 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert!(
        holds(&s, "quest ruine schritt tafel"),
        "durch den Boden gestampft"
    );
    talk(&mut s, &mut w, "tafel-kammer");
    assert!(holds(&s, "quest ruine schritt bericht"));

    // zurück zur Oase und gießen
    let mut w = travel(&mut s, &mut w, "west", -1);
    assert_eq!(s.map_name, "wueste-2");
    for n in 1..=3 {
        assert_eq!(
            talk(&mut s, &mut w, &format!("giessstelle-{n}"))[0],
            "giessen"
        );
    }
    assert!(holds(&s, "quest oase schritt danke"));
    // die Blüte steht jetzt dort
    assert!(s.npcs(&w).iter().any(|n| n.id == "bluete-1"));
    assert!(!s.npcs(&w).iter().any(|n| n.id == "giessstelle-1"));
    talk(&mut s, &mut w, "palma");
    assert!(holds(&s, "quest oase erledigt"));

    // Glutquelle: im Schatten des Kessels, Kampf, Tor geht auf
    let mut w = s.enter("wueste-arena", load("wueste-arena"), "west", &tuning);
    assert!(holds(&s, "quest glutsand schritt hueter"));
    go_to(&mut s, &mut w, "sandschlange", -200.0);
    assert!(!s.in_sun, "Kessel liegt im Schatten");
    // benommen am Boden, ein Hammerschlag beruhigt sie
    assert_eq!(w.creatures.len(), 1, "nur die Sandschlange im Kessel");
    let snake = &mut w.creatures[0];
    snake.mode = elora_sim::creature::serpent::STUNNED;
    snake.timer = 0;
    snake.health = 1;
    let at = snake.pos;
    w.spawn_character(s.player, at + Vec2::new(-60.0, 0.0));
    w.character_mut(s.player).unwrap().invulnerable_until = u64::MAX;
    let hit = PlayerInput {
        fire: 1,
        target_x: 100,
        target_y: 0,
        ..PlayerInput::default()
    };
    let mut done = Vec::new();
    for _ in 0..3 {
        done.extend(step(&mut s, &mut w, hit, false));
    }
    for _ in 0..80 {
        done.extend(step(&mut s, &mut w, PlayerInput::default(), false));
    }
    assert!(
        done.contains(&SessionEvent::ChapterDone {
            area: "glutsandwueste".into()
        }),
        "Gewinn-Bildschirm nach dem Hüter"
    );
    assert_eq!(s.save.flag("besiegt.sandschlange"), 1);
    assert!(s.save.count("quellfunke") >= 1, "Funke eingesammelt");
    assert!(holds(&s, "quest glutsand schritt funke"));
    let gate = s.map.adventure.object("tor").unwrap().pos;
    assert!(!w.collision.tile_at(gate + Vec2::new(16.0, 16.0)).is_solid());
    assert_eq!(talk(&mut s, &mut w, "schlange")[0], "erwacht");
    let (map, spawn) = {
        go_to(&mut s, &mut w, "ost", 0.0);
        walk_until_travel(&mut s, &mut w, 1)
    };
    assert_eq!((map.as_str(), spawn.as_str()), ("tauwinkel", "hohlweg"));
}

/// Stampfkammern (R2-M2.3, M2.3.6): Der Bröckelboden trägt, bis Elora mit Stampfen
/// daraufspringt; dann fällt sie in die Kammer zur Truhe. Übung in Tüftels Hof und
/// Belohnungen für die Rückkehr nach Kapitel 1 und 2.
#[test]
fn stomp_vaults_open_only_with_a_stomp() {
    let tuning = Tuning::default();
    for (map, spawn, id) in [
        ("tauwinkel", "start", "stampf"),
        ("wiese-2", "west", "wiese2-stampf"),
        ("wald-1", "ost", "wald1-stampf"),
        ("wald-3", "ost", "wald3-stampf"),
    ] {
        let mut s = Session::new_game(Content::builtin());
        let mut w = s.enter(map, load(map), spawn, &tuning);
        let chest = s
            .map
            .adventure
            .object(&format!("{id}-truhe"))
            .unwrap_or_else(|| panic!("{map}: Truhe fehlt"))
            .pos;
        // Bröckelboden neben der Truhe: Mitte und Oberkante
        let crumbs: Vec<(i32, i32)> = (-6..=6)
            .flat_map(|dx| (-8..=0).map(move |dy| (dx, dy)))
            .map(|(dx, dy)| {
                #[allow(clippy::cast_possible_truncation)]
                let (tx, ty) = ((chest.x / 32.0) as i32 + dx, (chest.y / 32.0) as i32 + dy);
                (tx, ty)
            })
            .filter(|&(tx, ty)| w.collision.tile(tx, ty) == elora_sim::Tile::Crumble)
            .collect();
        assert!(!crumbs.is_empty(), "{map}: kein Bröckelboden");
        let top = crumbs.iter().map(|c| c.1).min().unwrap();
        #[allow(clippy::cast_precision_loss)]
        let mid = crumbs.iter().map(|c| c.0 as f32).sum::<f32>() / crumbs.len() as f32;
        #[allow(clippy::cast_precision_loss)]
        let above = Vec2::new((mid + 0.5) * 32.0, top as f32 * 32.0 - 20.0);
        let run = |s: &mut Session, w: &mut World, input: PlayerInput, n: usize| {
            for _ in 0..n {
                step(s, w, input, false);
            }
        };
        let jump = PlayerInput {
            jump: true,
            ..PlayerInput::default()
        };
        let down = PlayerInput {
            down: true,
            ..PlayerInput::default()
        };
        // ohne Stampfen trägt der Boden
        w.spawn_character(s.player, above);
        w.set_abilities(s.player, elora_sim::Abilities::NONE);
        run(&mut s, &mut w, PlayerInput::default(), 20);
        run(&mut s, &mut w, jump, 1);
        run(&mut s, &mut w, PlayerInput::default(), 12);
        run(&mut s, &mut w, down, 60);
        #[allow(clippy::cast_precision_loss)]
        let surface = top as f32 * 32.0;
        let y = w.character(s.player).unwrap().core.pos.y;
        assert!(y < surface, "{map}: ohne Stampfen nicht hinein ({y})");
        // mit Stampfen bricht er
        w.spawn_character(s.player, above);
        w.set_abilities(
            s.player,
            elora_sim::Abilities::NONE.with(elora_sim::Ability::Stomp),
        );
        run(&mut s, &mut w, PlayerInput::default(), 20);
        run(&mut s, &mut w, jump, 1);
        run(&mut s, &mut w, PlayerInput::default(), 12);
        run(&mut s, &mut w, down, 60);
        run(&mut s, &mut w, PlayerInput::default(), 30);
        let p = w.character(s.player).unwrap().core.pos;
        assert!(p.y > surface + 64.0, "{map}: in der Kammer ({p:?})");
        assert!((p.y - chest.y).abs() < 24.0, "{map}: unten bei der Truhe");
        // ein Sprung (mit Doppelsprung) führt wieder hinaus
        run(&mut s, &mut w, jump, 1);
        run(&mut s, &mut w, PlayerInput::default(), 14);
        run(&mut s, &mut w, jump, 1);
        let mut out = false;
        for _ in 0..30 {
            step(&mut s, &mut w, PlayerInput::default(), false);
            out |= w.character(s.player).unwrap().core.pos.y < surface - 14.0;
        }
        assert!(out, "{map}: wieder hinaus");
    }
}

/// Durchlauf von Kapitel 4 (R2-M2.4) auf den mitgelieferten Karten: Eisdeckel am Bergsteig
/// (Stampfen), Gletscherfuß mit Kälte und Feuer, Bergdorf mit Flocke, Keller und Seil,
/// Steigkrallen, Kletterer, Gipfelgrat im Schneesturm, Kampf in der Eishalle, Tor und Weg
/// zurück nach Tauwinkel.
#[test]
#[allow(clippy::too_many_lines)] // ein Durchlauf in der Reihenfolge des Kapitels
fn chapter_four_from_the_mountain_path_to_the_ice_hall_and_home() {
    let tuning = Tuning::default();
    let run = |s: &mut Session, w: &mut World, input: PlayerInput, n: usize| {
        for _ in 0..n {
            step(s, w, input, false);
        }
    };
    let jump = PlayerInput {
        jump: true,
        ..PlayerInput::default()
    };
    let down = PlayerInput {
        down: true,
        ..PlayerInput::default()
    };
    let mut s = Session::new_game(Content::builtin());
    s.save
        .run(&s.content.clone(), &["quest frostspitzen start".into()]);
    let mut w = s.enter("tauwinkel", load("tauwinkel"), "bergsteig", &tuning);
    // Eisdeckel (Spalten 343–345, Zeilen 32–33): trägt ohne Stampfen
    let lid = Vec2::new(344.5 * 32.0, 32.0 * 32.0 - 20.0);
    w.spawn_character(s.player, lid);
    w.set_abilities(s.player, elora_sim::Abilities::NONE);
    run(&mut s, &mut w, PlayerInput::default(), 20);
    run(&mut s, &mut w, jump, 1);
    run(&mut s, &mut w, PlayerInput::default(), 12);
    run(&mut s, &mut w, down, 60);
    assert!(
        w.collision.tile(344, 32) == elora_sim::Tile::Crumble,
        "Deckel hält"
    );
    // mit Stampfen bricht er, Elora fällt in den Gang und kommt zum Übergang
    w.spawn_character(s.player, lid);
    w.set_abilities(
        s.player,
        elora_sim::Abilities::NONE.with(elora_sim::Ability::Stomp),
    );
    run(&mut s, &mut w, PlayerInput::default(), 20);
    run(&mut s, &mut w, jump, 1);
    run(&mut s, &mut w, PlayerInput::default(), 12);
    run(&mut s, &mut w, down, 60);
    assert!(!w.collision.tile_at(lid + Vec2::new(0.0, 40.0)).is_solid());
    let (map, spawn) = walk_until_travel(&mut s, &mut w, 1);
    assert_eq!((map.as_str(), spawn.as_str()), ("frost-1", "west"));
    // der Deckel bleibt offen
    assert!(
        s.save
            .broken
            .get("tauwinkel")
            .is_some_and(|b| b.contains(&(344, 32)))
    );
    let mut w = s.enter(&map, load(&map), &spawn, &tuning);
    assert!(holds(&s, "quest frostspitzen schritt flocke"));
    assert!(s.chilly());
    // am Eingang brennt ein Feuer: dort bleibt es warm
    s.cold = 0.8;
    go_to(&mut s, &mut w, "feuer-eingang", 0.0);
    assert!(s.cold < 0.8, "am Feuer wärmer: {}", s.cold);

    // Bergdorf: Flocke, Seil aus dem Keller, Steigkrallen
    let mut w = travel(&mut s, &mut w, "ost", 1);
    assert_eq!(s.map_name, "frost-2");
    assert_eq!(talk(&mut s, &mut w, "flocke")[0], "begruessung");
    assert!(holds(&s, "quest frostspitzen schritt seil"));
    go_to(&mut s, &mut w, "truhe-seil", -30.0);
    step(&mut s, &mut w, PlayerInput::default(), true);
    for _ in 0..40 {
        step(&mut s, &mut w, PlayerInput::default(), false);
    }
    assert_eq!(s.save.count("seil"), 1, "Seil aus der Truhe im Keller");
    assert_eq!(talk(&mut s, &mut w, "flocke")[0], "seil_zurueck");
    assert!(s.save.abilities().has(elora_sim::Ability::Grip));
    assert!(holds(&s, "quest kletterer aktiv"));
    talk(&mut s, &mut w, "kiesel");
    assert_eq!(s.save.flag("kletterer.gefunden"), 1);
    // Kiesel sitzt jetzt in der Hütte
    assert!(s.npcs(&w).iter().any(|n| n.id == "kiesel-huette"));
    assert!(!s.npcs(&w).iter().any(|n| n.id == "kiesel"));

    // Gipfelgrat: immer im Schneesturm, die graue Stelle
    let mut w = travel(&mut s, &mut w, "ost", 1);
    assert_eq!(s.map_name, "frost-3");
    assert!(holds(&s, "quest frostspitzen schritt quelle"));
    assert_eq!(s.map.weather.kind, elora_map::WeatherKind::Blizzard);
    talk(&mut s, &mut w, "graue-stelle");
    assert_eq!(s.save.flag("duerrer.grat"), 1);
    talk(&mut s, &mut w, "wicke");

    // Eishalle: Kampf, Tor geht auf
    let mut w = travel(&mut s, &mut w, "ost", 1);
    assert_eq!(s.map_name, "frost-arena");
    assert!(s.map.weather.is_clear(), "die Halle bleibt schön");
    assert!(holds(&s, "quest frostspitzen schritt hueter"));
    go_to(&mut s, &mut w, "eiskoenigin", -150.0);
    let kind = w.creature_kind("kristella").expect("Art");
    let queen = w
        .creatures
        .iter_mut()
        .find(|c| c.kind == kind)
        .expect("Kristella in der Halle");
    queen.mode = elora_sim::creature::queen::TIRED;
    queen.timer = 0;
    queen.health = 1;
    let at = queen.pos;
    w.spawn_character(s.player, at + Vec2::new(-60.0, 0.0));
    w.character_mut(s.player).unwrap().invulnerable_until = u64::MAX;
    let hit = PlayerInput {
        fire: 1,
        target_x: 100,
        target_y: 0,
        ..PlayerInput::default()
    };
    let mut done = Vec::new();
    for _ in 0..3 {
        done.extend(step(&mut s, &mut w, hit, false));
    }
    for _ in 0..80 {
        done.extend(step(&mut s, &mut w, PlayerInput::default(), false));
    }
    assert!(
        done.contains(&SessionEvent::ChapterDone {
            area: "frostspitzen".into()
        }),
        "Gewinn-Bildschirm nach der Hüterin"
    );
    assert_eq!(s.save.flag("besiegt.kristella"), 1);
    assert!(s.save.count("quellfunke") >= 1, "Funke eingesammelt");
    let gate = s.map.adventure.object("tor").unwrap().pos;
    assert!(!w.collision.tile_at(gate + Vec2::new(16.0, 16.0)).is_solid());
    assert_eq!(talk(&mut s, &mut w, "kristella")[0], "erwacht");
    assert_eq!(s.save.flag("befreit.frostquelle"), 1);
    let (map, spawn) = {
        go_to(&mut s, &mut w, "ost", 0.0);
        walk_until_travel(&mut s, &mut w, 1)
    };
    assert_eq!((map.as_str(), spawn.as_str()), ("tauwinkel", "bergsteig"));
}

/// Kletterstellen für die Rückkehr (R2-M2.4, M2.4.7): Unter dem hängenden Kamin läuft man
/// durch; mit dem Eisgriff klettert Elora (nur mit Eingaben) bis auf das Sims mit der Truhe,
/// ohne ihn nicht.
#[test]
fn climb_vaults_need_the_grip() {
    let tuning = Tuning::default();
    for (map, spawn, id) in [
        ("wiese-2", "west", "wiese2-kletter"),
        ("wald-1", "ost", "wald1-kletter"),
        ("wueste-2", "west", "wueste2-kletter"),
    ] {
        let highest = |abilities: elora_sim::Abilities| {
            let mut s = Session::new_game(Content::builtin());
            let mut w = s.enter(map, load(map), spawn, &tuning);
            // nur das Klettern zählt: Gegner in der Nähe stören den Bot
            w.creatures.clear();
            let chest = s
                .map
                .adventure
                .object(&format!("{id}-truhe"))
                .unwrap_or_else(|| panic!("{map}: Truhe fehlt"))
                .pos;
            // unter dem Kamin: fünf Spalten neben der Truhe ist die Mitte zwischen den Wänden
            // (die Wald-Karten sind gespiegelt: dort liegt das Sims links)
            let floor = chest.y + 19.0 * 32.0 + 13.0;
            let wall = |x: f32| {
                w.collision.tile_at(Vec2::new(x, floor - 10.0 * 32.0)) == elora_sim::Tile::Climb
            };
            let side: f32 = if wall(chest.x - 3.0 * 32.0) {
                -1.0
            } else {
                1.0
            };
            let start = Vec2::new(chest.x + side * 5.0 * 32.0, floor - 15.0);
            #[allow(clippy::cast_possible_truncation)]
            let to_ledge = -side as i8;
            w.spawn_character(s.player, start);
            w.set_abilities(s.player, abilities);
            w.character_mut(s.player).unwrap().invulnerable_until = u64::MAX;
            for _ in 0..20 {
                step(&mut s, &mut w, PlayerInput::default(), false);
            }
            let mut toward: i8 = 1;
            let mut held = false;
            let mut best = f32::MAX;
            // vom Boden gerade hochspringen und erst oben zur Wand lenken
            let mut rising = false;
            for _ in 0..1500 {
                let c = w.character(s.player).unwrap().core.clone();
                best = best.min(c.pos.y);
                let grounded = c.is_grounded(&w.collision);
                let jump = if c.grip != 0 && !held {
                    toward = -c.grip;
                    rising = false;
                    true
                } else {
                    !held && grounded
                };
                if jump && grounded {
                    rising = true;
                }
                held = jump;
                // oben angekommen: nach rechts aufs Sims
                let dir = if c.pos.y < chest.y + 20.0 {
                    to_ledge
                } else if rising && c.vel.y < -3.0 {
                    0
                } else {
                    toward
                };
                step(
                    &mut s,
                    &mut w,
                    PlayerInput {
                        direction: dir,
                        jump,
                        ..PlayerInput::default()
                    },
                    false,
                );
            }
            (best, chest.y)
        };
        let grip = elora_sim::Abilities::NONE.with(elora_sim::Ability::Grip);
        let (best, top) = highest(grip);
        assert!(
            best < top + 8.0,
            "{map}: mit Eisgriff auf dem Sims ({best} / {top})"
        );
        let (best, top) = highest(elora_sim::Abilities::NONE);
        assert!(
            best > top + 6.0 * 32.0,
            "{map}: ohne Eisgriff nicht ({best} / {top})"
        );
    }
}

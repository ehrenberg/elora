//! Karten des Prologs (A1.9, `docs/release-2/prolog.md`): Tauwinkel und Blütenwiesen 1 –
//! Gelände, Objekte des Abenteuers und Deko (Gebäude E-278, verblasste Blumen E-277).
//!
//! `cargo test -p elora-client --bin elora write_prolog_maps -- --ignored` schreibt
//! `maps/abenteuer/*.emap`.

#![allow(
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::too_many_lines
)]

use std::time::Instant;

use elora_map::look::{Curve, EnvKind, EnvPoint, EnvRef, Envelope};
use elora_map::{Art, Decor, Map, Object, ObjectKind};
use elora_sim::{TILE_SIZE, Vec2};

use super::Editor;
use super::look::Preset;
use super::release::{self, Theme};

pub(super) const T: f32 = TILE_SIZE as f32;

/// Zeichenraster der Karte; unten ab `floor` Boden, an den Seiten offen (E-279).
pub(super) struct Grid(pub(super) Vec<Vec<char>>);

impl Grid {
    pub(super) fn new(w: usize, h: usize, floor: usize) -> Self {
        let mut g = vec![vec!['.'; w]; h];
        for (y, row) in g.iter_mut().enumerate() {
            for c in row.iter_mut() {
                if y >= floor {
                    *c = '#';
                }
            }
        }
        Self(g)
    }

    /// Rechteck (Spalten `x`, Zeilen `y`, jeweils einschließlich) füllen.
    pub(super) fn fill(&mut self, x: (usize, usize), y: (usize, usize), c: char) {
        for row in &mut self.0[y.0..=y.1] {
            for cell in &mut row[x.0..=x.1] {
                *cell = c;
            }
        }
    }

    /// Boden der Spalten `x0..=x1` auf Zeile `top` setzen (darüber Luft).
    pub(super) fn ground(&mut self, x0: usize, x1: usize, top: usize) {
        for (y, row) in self.0.iter_mut().enumerate() {
            for cell in &mut row[x0..=x1] {
                *cell = if y >= top { '#' } else { '.' };
            }
        }
    }

    /// Karte aus dem Raster; das `S` braucht nur das Textformat, die Karte nutzt Eingänge.
    pub(super) fn map(&self, name: &str) -> Map {
        let mut g = self.0.clone();
        g[0][1] = 'S';
        let rows: Vec<String> = g.iter().map(|r| r.iter().collect()).collect();
        let r: Vec<&str> = rows.iter().map(String::as_str).collect();
        let mut m = Map::from_rows(name, &r).expect("Layout gültig");
        m.entities.clear();
        m
    }
}

/// Ruck-Stelle (M2.1.5): Steinwand links, Schacht 5 Tiles breit,
/// Hook-Blüte 12 Tiles über dem Boden an der rechten Seite, rechts ein Steinturm 7 Tiles breit,
/// dessen Oberkante 9 Tiles über der Blüte liegt – nur mit Hook-Ruck erreichbar. Geprüft in
/// `crates/elora-sim/tests/abilities.rs` (`ruck_gate_needs_the_hook_ruck`). Unten führt ein
/// Durchgang (3 Tiles hoch) durch Wand und Turm, damit der Weg frei bleibt. Liefert die Zeile
/// der Turm-Oberkante.
pub(super) fn ruck_gate(g: &mut Grid, x0: usize, floor: usize) -> usize {
    let top = floor - 21;
    g.fill((x0, x0), (top, floor - 4), '%');
    g.fill((x0 + 1, x0 + 5), (top, floor - 1), '.');
    g.fill((x0 + 5, x0 + 5), (floor - 12, floor - 12), '*');
    g.fill((x0 + 6, x0 + 12), (top, floor - 4), '%');
    top
}

/// Mitte über dem Boden (Oberkante von Zeile `ty`) in Spalte `tx` für ein Objekt der Höhe `h`.
pub(super) fn at(tx: usize, ty: usize, h: f32) -> Vec2 {
    Vec2::new(tx as f32 * T + T / 2.0, ty as f32 * T - h / 2.0 - 1.0)
}

pub(super) fn corner(tx: usize, ty: usize) -> Vec2 {
    Vec2::new(tx as f32 * T, ty as f32 * T)
}

pub(super) fn o(id: &str, pos: Vec2, kind: ObjectKind) -> Object {
    Object {
        id: id.into(),
        pos,
        kind,
    }
}

pub(super) fn npc(id: &str, tx: usize, ty: usize, facing: i8, walk: f32) -> Object {
    o(
        id,
        at(tx, ty, 28.0),
        ObjectKind::Npc {
            character: id.into(),
            dialog: id.into(),
            facing,
            walk,
        },
    )
}

/// Wegweiser-Schild mit Hinweis (E-273).
pub(super) fn sign(dialog: &str, tx: usize, ty: usize) -> Object {
    o(
        dialog,
        at(tx, ty, 28.0),
        ObjectKind::Npc {
            character: "wegweiser".into(),
            dialog: dialog.into(),
            facing: 1,
            walk: 0.0,
        },
    )
}

pub(super) fn creature(id: &str, kind: &str, tx: usize, ty: usize, h: f32) -> Object {
    o(
        id,
        at(tx, ty, h),
        ObjectKind::Creature {
            kind: kind.into(),
            persistent: false,
        },
    )
}

pub(super) fn chest(id: &str, tx: usize, ty: usize, contents: &[(&str, u32)]) -> Object {
    o(
        id,
        at(tx, ty, 26.0),
        ObjectKind::Chest {
            contents: contents.iter().map(|(i, n)| ((*i).into(), *n)).collect(),
            lock: String::new(),
        },
    )
}

pub(super) fn plant(id: &str, tx: usize, ty: usize) -> Object {
    o(id, at(tx, ty, 16.0), ObjectKind::HealPlant { heal: 2 })
}

/// Deko auf dem Boden (Oberkante von Zeile `ty`), Mitte in Spalte `tx` (halbe Tiles erlaubt).
pub(super) fn decor(name: &str, tx: f32, ty: usize) -> Decor {
    Decor::new(
        Art::Builtin(name.into()),
        Vec2::new(tx * T + T / 2.0, ty as f32 * T),
    )
}

/// Deko an einer Weltposition (Pixel), z. B. auf Dächern und Bänken.
pub(super) fn decor_px(name: &str, x: f32, y: f32) -> Decor {
    Decor::new(Art::Builtin(name.into()), Vec2::new(x, y))
}

/// Animation anhängen; liefert ihren Index.
fn envelope(map: &mut Map, name: &str, kind: EnvKind, points: &[(u32, [f32; 4], Curve)]) -> u16 {
    map.envelopes.push(Envelope {
        name: name.into(),
        kind,
        synced: false,
        points: points
            .iter()
            .map(|&(time_ms, value, curve)| EnvPoint {
                time_ms,
                value,
                curve,
            })
            .collect(),
    });
    u16::try_from(map.envelopes.len() - 1).expect("wenige Animationen")
}

/// Kleine Bewegungen (Detail): Schmetterlinge flattern, Rauch steigt, Fahnen wehen.
pub(super) fn animate(map: &mut Map, smoke: &[Vec2]) {
    use Curve::{Linear, Smooth};
    let flutter = envelope(
        map,
        "Flattern",
        EnvKind::Position,
        &[
            (0, [0.0, 0.0, 0.0, 0.0], Smooth),
            (1500, [30.0, -24.0, 8.0, 0.0], Smooth),
            (3000, [64.0, -4.0, -6.0, 0.0], Smooth),
            (4500, [30.0, 18.0, 6.0, 0.0], Smooth),
            (6000, [0.0, 0.0, 0.0, 0.0], Smooth),
        ],
    );
    let wave = envelope(
        map,
        "Wehen",
        EnvKind::Position,
        &[
            (0, [0.0, 0.0, -1.5, 0.0], Smooth),
            (1800, [0.0, 0.0, 1.5, 0.0], Smooth),
            (3600, [0.0, 0.0, -1.5, 0.0], Smooth),
        ],
    );
    let rise = envelope(
        map,
        "Rauch",
        EnvKind::Position,
        &[
            (0, [0.0, 0.0, 0.0, 0.0], Linear),
            (3000, [14.0, -90.0, 0.0, 0.0], Linear),
        ],
    );
    let fade = envelope(
        map,
        "Rauch verblasst",
        EnvKind::Color,
        &[
            (0, [1.0, 1.0, 1.0, 0.0], Linear),
            (400, [1.0, 1.0, 1.0, 0.9], Linear),
            (3000, [1.0, 1.0, 1.0, 0.0], Linear),
        ],
    );
    let mut k = 0;
    for d in map.decor_front.iter_mut().chain(map.decor_back.iter_mut()) {
        let Art::Builtin(name) = &d.art else {
            continue;
        };
        k += 1;
        let offset_ms = (k * 977) % 6000;
        match name.as_str() {
            "schmetterling" => {
                d.pos_env = Some(EnvRef {
                    index: flutter,
                    offset_ms,
                });
            }
            n if n.starts_with("fahne-") => {
                d.pos_env = Some(EnvRef {
                    index: wave,
                    offset_ms,
                });
            }
            _ => {}
        }
    }
    for (i, &at) in smoke.iter().enumerate() {
        for puff in 0..2 {
            let offset_ms = i32::try_from(i * 700 + puff * 1500).unwrap_or(0);
            let mut d = decor_px("rauch", at.x, at.y);
            d.pos_env = Some(EnvRef {
                index: rise,
                offset_ms,
            });
            d.color_env = Some(EnvRef {
                index: fade,
                offset_ms,
            });
            map.decor_back.push(d);
        }
    }
}

/// Gras vor der Spielfläche (Tauwinkel: keine bunten Blumen, E-210).
const GRASS: Theme = Theme {
    file: "",
    name: "",
    rows: &[],
    material: None,
    preset: Preset::Day,
    sky: None,
    background_tint: None,
    no_forest: false,
    back: &[("bush-1", 2)],
    front: &["grass-1", "grass-2"],
    decor_tint: None,
    front_density: 25,
    back_spacing: 1000,
};

pub(super) fn finish(map: Map, theme: &Theme) -> Map {
    let mut editor = Editor::new(None, std::path::PathBuf::from("maps"));
    let (back, front) = (map.decor_back.clone(), map.decor_front.clone());
    editor.map = map;
    editor.map.author = Some("Elora-Team".into());
    editor.map.decor_back.clear();
    editor.map.decor_front.clear();
    editor.apply_preset(Preset::Day, Instant::now());
    release::place(theme, &mut editor.map);
    // große Deko nur auf durchgehendem Boden (nicht auf Hook-Felsen und Simsen)
    let auto = std::mem::take(&mut editor.map.decor_back);
    let map = &editor.map;
    let grounded = |d: &Decor| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let (tx, ty) = ((d.pos.x / T) as usize, (d.pos.y / T) as usize);
        (ty..map.height).all(|y| map.tiles[y * map.width + tx] != elora_sim::Tile::Air)
    };
    let back: Vec<Decor> = back
        .into_iter()
        .chain(auto.into_iter().filter(|d| grounded(d)))
        .collect();
    editor.map.decor_back = back;
    // eigene Deko zuerst (hinter der automatischen)
    editor.map.decor_front.splice(0..0, front);
    editor.map
}

/// Tauwinkel, 450 × 50 (E-280): Steilhang im Westen, Eloras Garten mit Baumhaus, Hecke,
/// Steg, Brunnenplatz, Werkstatt und Tüftels Hof (E-281), Aufstieg ins Oberdorf mit Schmiede,
/// Strohpuppen und Laden, Ostpfad hinunter zum offenen Übergang (E-279).
pub fn tauwinkel() -> Map {
    let (w, h) = (450, 50);
    let mut g = Grid::new(w, h, 40);
    // Westen: Steilhang aus Stein als natürliches Ende
    g.fill((0, 3), (0, h - 1), '%');
    g.fill((4, 6), (18, h - 1), '%');
    g.fill((7, 8), (29, h - 1), '%');
    // Baumhaus: Plattform 5 Tiles über dem Boden
    g.fill((40, 44), (35, 35), '=');
    // Hecke: nur mit Doppelsprung
    g.fill((54, 56), (33, 39), '#');
    // Wiesenweg zwei Tiles höher, Mulde mit Steg (Runter zum Durchfallen)
    g.ground(67, 180, 38);
    g.fill((75, 85), (38, 40), '.');
    g.fill((75, 85), (38, 38), '=');
    // Tüftels Hof: Grube unter einer Decke, Felsbogen mit Stein- und Erd-Überhang,
    // Bröckelbrücke, Block und hoher Sitz unter einem Überhang
    g.ground(181, 252, 38);
    // Ruck-Strecke hinter der Werkstatt: Tor öffnet sich, wenn Tüftel den Hook-Ruck gebaut hat
    let ruck_top = ruck_gate(&mut g, 170, 38);
    g.fill((186, 203), (38, 41), '.');
    g.fill((185, 204), (29, 30), '#');
    g.fill((211, 221), (26, 34), '#');
    g.fill((205, 212), (15, 16), '%');
    g.fill((213, 225), (15, 16), '#');
    g.fill((222, 229), (26, 26), ':');
    g.fill((230, 236), (26, 27), '#');
    g.fill((241, 247), (15, 15), '#');
    g.fill((236, 251), (5, 6), '#');
    // Aufstieg ins Oberdorf und Ostpfad hinunter
    g.ground(253, 256, 36);
    g.ground(257, 260, 34);
    g.ground(261, 350, 32);
    g.ground(351, 356, 34);
    g.ground(357, 362, 36);
    g.ground(363, w - 1, 38);
    g.ground(404, 417, 36);
    let mut m = g.map("Tauwinkel");
    m.adventure.objects = vec![
        o("start", at(23, 40, 28.0), ObjectKind::Spawn),
        sign("schild-start", 26, 40),
        npc("pip", 37, 40, -1, 0.0),
        chest("truhe-baumhaus", 41, 35, &[("glanztropfen", 10)]),
        sign("schild-hecke", 51, 40),
        sign("schild-plattform", 72, 38),
        chest("truhe-1", 83, 41, &[("glanztropfen", 20)]),
        sign("schild-brunnen", 102, 38),
        o("brunnen", at(110, 38, 40.0), ObjectKind::SavePoint),
        npc("oma", 123, 38, -1, 0.0),
        npc("tueftel", 158, 38, 1, 32.0),
        sign("schild-hook", 184, 38),
        sign("schild-ruck", 172, 38),
        chest(
            "truhe-ruck",
            179,
            ruck_top,
            &[("glanztropfen", 40), ("tautrank", 1)],
        ),
        chest(
            "truhe-hook",
            244,
            15,
            &[("glanztropfen", 25), ("heiltrank", 1)],
        ),
        npc("klonk", 281, 32, 1, 0.0),
        sign("schild-hammer", 285, 32),
        creature("puppe-1", "strohpuppe", 290, 32, 40.0),
        creature("puppe-2", "strohpuppe", 295, 32, 40.0),
        creature("puppe-3", "strohpuppe", 300, 32, 40.0),
        npc("lotte", 321, 32, -1, 0.0),
        sign("schild-ostpfad", 398, 38),
        o("ost", at(436, 38, 28.0), ObjectKind::Spawn),
        o(
            "weg-wiese",
            corner(w - 3, 0),
            ObjectKind::Exit {
                size: Vec2::new(3.0 * T, h as f32 * T),
                map: "wiese-1".into(),
                spawn: "west".into(),
                on_touch: true,
            },
        ),
    ];
    let mut baumhaus = decor("baumhaus", 42.0, 40);
    baumhaus.pos.y += 8.0;
    m.decor_back = vec![
        decor("tree-pine", 5.0, 18),
        decor("tree-round", 8.0, 29),
        decor("tree-round", 11.0, 40),
        decor("haus-elora", 18.0, 40),
        decor("waescheleine", 31.0, 40),
        baumhaus,
        decor("fence", 48.0, 40),
        decor("tree-round", 62.0, 40),
        decor("fence", 69.0, 38),
        decor("bank", 90.0, 38),
        decor("laterne", 93.0, 38),
        decor("anschlagbrett", 99.0, 38),
        decor("fahne-blass", 106.0, 38),
        decor("brunnen", 115.0, 38),
        decor("haus-oma", 132.0, 38),
        decor("bank", 140.0, 38),
        decor("fahne-blass", 144.0, 38),
        decor("laterne", 148.0, 38),
        decor("werkstatt", 166.0, 38),
        decor("holzstapel", 148.0, 38),
        decor("faesser", 151.5, 38),
        decor("tree-round", 208.0, 38),
        decor("tree-pine", 255.0, 36),
        decor("schmiede", 272.0, 32),
        decor("holzstapel", 309.0, 32),
        decor("laden", 328.0, 32),
        decor("faesser", 336.0, 32),
        decor("karren", 341.0, 32),
        decor("bank", 347.0, 32),
        decor("laterne", 350.0, 32),
        decor("tree-round", 368.0, 38),
        decor("tree-pine", 384.0, 38),
        decor("fence", 390.0, 38),
        decor("fahne-blass", 401.0, 38),
        decor("tree-round", 411.0, 36),
        decor("tree-pine", 422.0, 38),
        decor("tree-round", 444.0, 38),
    ];
    m.decor_front = vec![
        decor("bush-2", 9.0, 40),
        decor("beet-blass", 14.0, 40),
        decor("blumenkasten-blass", 21.5, 40),
        decor("kraeuterbeet-blass", 28.0, 40),
        decor("bush-1", 55.0, 33),
        decor("heuballen", 79.0, 41),
        decor("beet-blass", 113.0, 38),
        decor("beet-blass", 119.0, 38),
        decor("kraeuterbeet-blass", 128.5, 38),
        decor("kraeuterbeet-blass", 135.5, 38),
        decor("mauer", 151.0, 38),
        decor("heuballen", 191.0, 42),
        decor("heuballen", 198.0, 42),
        decor("heuballen", 306.0, 32),
        decor("blumenkasten-blass", 325.5, 32),
        decor("beet-blass", 332.0, 32),
        decor("mauer", 374.0, 38),
        decor("rock-2", 380.0, 38),
        decor("bush-2", 395.0, 38),
        decor("heuballen", 407.0, 36),
        decor("rock-1", 415.0, 36),
        decor("bush-1", 419.0, 38),
        decor("fence", 427.0, 38),
        decor("fence", 429.0, 38),
        decor("bush-1", 430.0, 38),
        decor("rock-2", 440.0, 38),
    ];
    // Hecke aus Büschen vor dem Block
    for k in 0..6 {
        let y = 40.0 * T - k as f32 * 38.0;
        let x = 55.0 * T + T / 2.0 + if k % 2 == 0 { -12.0 } else { 12.0 };
        let mut d = Decor::new(Art::Builtin("bush-2".into()), Vec2::new(x, y));
        d.flip_x = k % 2 == 1;
        m.decor_front.push(d);
    }
    let ground = |tx: f32, ty: usize| Vec2::new(tx * T + T / 2.0, ty as f32 * T);
    m.decor_front.extend([
        decor("blumentopf-blass", 13.0, 40),
        decor("giesskanne", 15.0, 40),
        decor("briefkasten", 24.5, 40),
        decor("kuerbisse", 35.0, 40),
        decor("vogelhaus", 47.0, 40),
        decor("katze", 46.0, 35),
        decor("beerenbusch", 64.5, 40),
        decor("trittsteine", 96.0, 38),
        decor("korb", 126.0, 38),
        decor("blumentopf-blass", 130.5, 38),
        decor("blumentopf-blass", 133.5, 38),
        decor_px("katze", ground(140.0, 38).x, ground(140.0, 38).y - 28.0),
        decor("trittsteine", 160.0, 38),
        decor("giesskanne", 170.0, 38),
        decor("baumstumpf", 207.0, 38),
        decor("farn", 224.0, 38),
        decor_px("vogel", ground(244.0, 15).x + 40.0, ground(244.0, 15).y),
        decor("kuerbisse", 264.0, 32),
        decor("korb", 324.0, 32),
        decor("blumentopf-blass", 331.0, 32),
        decor("briefkasten", 318.0, 32),
        decor("baumstumpf", 376.0, 38),
        decor("farn", 386.0, 38),
        decor("loewenzahn", 392.0, 38),
        decor("beerenbusch", 414.0, 36),
        decor_px("vogel", ground(428.0, 38).x, ground(428.0, 38).y - 28.0),
        decor("schmetterling", 395.0, 36),
        decor("schmetterling", 420.0, 35),
        decor("loewenzahn", 433.0, 38),
    ]);
    // Festschmuck nach Kapitel 1 (E-301): nur mit Merker `fest` zu sehen
    for tx in [104.0, 119.0, 141.0, 286.0, 312.0] {
        m.decor_back
            .push(decor("girlande-fest", tx, if tx > 250.0 { 32 } else { 38 }));
    }
    // Laternen hängen an den Girlanden (Schnur bei −120, Leine dort bei etwa −133)
    for (k, tx) in [104.0, 119.0, 141.0, 286.0, 312.0].into_iter().enumerate() {
        let g = ground(tx, if tx > 250.0 { 32 } else { 38 });
        for (side, name) in [(-40.0, "festlaterne-fest"), (40.0, "festlaterne-gelb-fest")] {
            let flip = if k % 2 == 0 { side } else { -side };
            m.decor_back.push(decor_px(name, g.x + flip, g.y - 13.0));
        }
    }
    // Vögel auf den Dächern
    for (tx, ty, dy) in [(18.0, 40, 246.0), (115.0, 38, 178.0)] {
        let p = ground(tx, ty);
        m.decor_back.push(decor_px("vogel", p.x - 10.0, p.y - dy));
    }
    let chimneys = [
        ground(18.0, 40) + Vec2::new(56.0, -240.0),
        ground(166.0, 38) + Vec2::new(-42.0, -258.0),
        ground(328.0, 32) + Vec2::new(-40.0, -246.0),
    ];
    let mut map = finish(m, &GRASS);
    animate(&mut map, &chimneys);
    map
}

/// Blütenwiesen 1, 300 × 60 (E-282): Hügel, Tal mit Dornengrube und Hook-Decke darüber,
/// Brücke über einer Dornenschlucht (Stachelkäfer), Hügelkamm mit Pollenbläser,
/// Bröckelboden, Hook-Felsen zum hohen Plateau mit dem Glitzerstein, Quellstein am Wiesenrand.
pub fn wiese() -> Map {
    let (w, h) = (300, 60);
    let mut g = Grid::new(w, h, 44);
    // Hügel
    g.ground(22, 29, 42);
    g.ground(30, 38, 39);
    g.ground(39, 47, 41);
    // Tal mit Dornengrube; oben eine Decke zum Hooken und ein Sims mit Truhe
    g.ground(60, 95, 50);
    g.fill((72, 78), (50, 52), '.');
    g.fill((72, 78), (53, 53), '^');
    g.ground(96, 99, 47);
    g.fill((62, 69), (32, 33), '#');
    g.fill((72, 80), (32, 33), '#');
    g.fill((84, 92), (38, 39), '#');
    // Dornenschlucht mit Brücke
    g.ground(112, 134, 55);
    g.fill((113, 133), (54, 54), '^');
    g.fill((112, 134), (44, 44), '=');
    // Hügelkamm, dahinter ein Tal mit Bröckelboden über Dornen
    g.ground(150, 160, 40);
    g.ground(161, 170, 36);
    g.ground(171, 186, 32);
    g.ground(187, 230, 46);
    g.fill((196, 203), (46, 46), ':');
    g.fill((196, 203), (47, 49), '.');
    g.fill((196, 203), (50, 50), '^');
    // Hook-Felsen hinüber zum hohen Plateau (mit Durchgang darunter)
    g.fill((189, 192), (22, 23), '#');
    g.fill((197, 200), (22, 23), '#');
    g.fill((205, 208), (22, 23), '#');
    g.fill((215, 230), (30, 40), '#');
    // Anstieg zum Wiesenrand, im Osten dichter Wald am Hang
    g.ground(241, 250, 43);
    g.ground(251, w - 1, 40);
    // Ruck-Stelle mit Biene 5 (Rückkehr nach Kapitel 1)
    let ruck_top = ruck_gate(&mut g, 252, 40);
    let mut m = g.map("Blütenwiesen 1");
    m.adventure.objects = vec![
        o(
            "weg-dorf",
            corner(0, 0),
            ObjectKind::Exit {
                size: Vec2::new(2.0 * T, h as f32 * T),
                map: "tauwinkel".into(),
                spawn: "ost".into(),
                on_touch: true,
            },
        ),
        o("west", at(6, 44, 28.0), ObjectKind::Spawn),
        sign("schild-wiese", 10, 44),
        plant("blume-1", 34, 39),
        creature("huepfer-1", "grashuepfer", 43, 41, 28.0),
        creature("kaefer-1", "stachelkaefer", 53, 44, 26.0),
        creature("kaefer-2", "stachelkaefer", 66, 50, 26.0),
        plant("blume-2", 83, 50),
        creature("huepfer-2", "grashuepfer", 89, 50, 28.0),
        chest(
            "truhe-oben-1",
            88,
            38,
            &[("glanztropfen", 30), ("heiltrank", 1)],
        ),
        o(
            "bruecke",
            corner(108, 34),
            ObjectKind::Zone {
                size: Vec2::new(30.0 * T, 10.0 * T),
            },
        ),
        creature("kaefer-3", "stachelkaefer", 117, 44, 26.0),
        creature("kaefer-4", "stachelkaefer", 128, 44, 26.0),
        creature("kaefer-5", "stachelkaefer", 140, 44, 26.0),
        plant("blume-3", 146, 44),
        creature("blaeser", "pollenblaeser", 180, 32, 60.0),
        creature("huepfer-3", "grashuepfer", 209, 46, 28.0),
        o(
            "glitzerstein",
            at(224, 30, 28.0),
            ObjectKind::Collectible {
                item: "glitzerstein".into(),
            },
        ),
        chest(
            "truhe-oben-2",
            228,
            30,
            &[("glanztropfen", 40), ("bernstein", 2)],
        ),
        plant("blume-4", 236, 46),
        creature("kaefer-6", "stachelkaefer", 245, 43, 26.0),
        creature("huepfer-4", "grashuepfer", 270, 40, 28.0),
        o(
            "wiesenrand",
            corner(266, 30),
            ObjectKind::Zone {
                size: Vec2::new(28.0 * T, 10.0 * T),
            },
        ),
        o("quellstein", at(280, 40, 40.0), ObjectKind::SavePoint),
        // weiter in die Blütenwiesen (Kapitel 1)
        o("ost", at(292, 40, 28.0), ObjectKind::Spawn),
        o(
            "biene-5",
            at(261, ruck_top, 24.0),
            ObjectKind::Collectible {
                item: "biene".into(),
            },
        ),
        o(
            "weg-wiese-2",
            corner(w - 2, 0),
            ObjectKind::Exit {
                size: Vec2::new(2.0 * T, h as f32 * T),
                map: "wiese-2".into(),
                spawn: "west".into(),
                on_touch: true,
            },
        ),
    ];
    m.decor_back = vec![
        decor("tree-round", 15.0, 44),
        decor("tree-round", 64.0, 50),
        decor("tree-round", 104.0, 44),
        decor("tree-round", 165.0, 36),
        decor("tree-round", 239.0, 46),
        decor("tree-pine", 286.0, 40),
        decor("riesenblume-rosa", 296.0, 40),
    ];
    m.decor_front = vec![
        decor("bush-2", 222.5, 30),
        decor("bush-1", 225.5, 30),
        decor("bush-2", 289.0, 40),
    ];
    for (y, x0, x1) in [(54, 72, 78), (55, 113, 133), (51, 196, 203)] {
        let mut x = x0;
        while x < x1 {
            m.decor_front.push(decor("dornen", x as f32 + 0.5, y));
            x += 2;
        }
    }
    m.decor_front.extend([
        decor("farn", 18.0, 44),
        decor("loewenzahn", 26.0, 42),
        decor("baumstumpf", 45.0, 41),
        decor("beerenbusch", 57.0, 44),
        decor("farn", 62.0, 50),
        decor("loewenzahn", 86.0, 50),
        decor("farn", 92.0, 50),
        decor("trittsteine", 102.0, 44),
        decor("beerenbusch", 138.0, 44),
        decor("loewenzahn", 155.0, 40),
        decor("baumstumpf", 168.0, 36),
        decor("farn", 176.0, 32),
        decor("beerenbusch", 190.0, 46),
        decor("loewenzahn", 212.0, 46),
        decor("farn", 219.0, 30),
        decor("baumstumpf", 233.0, 46),
        decor("beerenbusch", 248.0, 43),
        decor("loewenzahn", 268.0, 40),
        decor("farn", 274.0, 40),
        decor("beerenbusch", 287.0, 40),
    ]);
    for (tx, ty) in [
        (14, 41),
        (33, 37),
        (70, 47),
        (90, 47),
        (122, 41),
        (146, 41),
        (176, 29),
        (206, 43),
        (226, 27),
        (252, 37),
        (270, 37),
        (284, 36),
    ] {
        m.decor_front.push(decor("schmetterling", tx as f32, ty));
    }
    let mut map = finish(m, &release::THEMES[0]);
    animate(&mut map, &[]);
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_adventure::Content;
    use elora_adventure::check::{map_links, map_objects};

    fn shipped(name: &str) -> String {
        format!(
            "{}/../../maps/abenteuer/{name}.{}",
            env!("CARGO_MANIFEST_DIR"),
            elora_map::EXTENSION
        )
    }

    #[test]
    fn prolog_maps_match_content_and_link_up() {
        let c = Content::builtin();
        let (a, b) = (tauwinkel(), wiese());
        for m in [&a, &b] {
            let back = elora_map::decode(&elora_map::encode(m)).expect("Karte gültig");
            let errors = map_objects(&c, &back);
            assert!(errors.is_empty(), "{}: {errors:?}", m.name);
            assert!(!back.decor_back.is_empty() && !back.backgrounds.is_empty());
        }
        // Übergänge prüft kapitel1::tests über alle Abenteuer-Karten
        let _ = map_links;
        assert!(a.adventure.object(&c.progression.start_spawn).is_some());
        // Zonen und Gegner der Hauptaufgabe
        assert!(b.adventure.object("wiesenrand").is_some());
        let kaefer = b
            .adventure
            .objects
            .iter()
            .filter(
                |o| matches!(&o.kind, ObjectKind::Creature { kind, .. } if kind == "stachelkaefer"),
            )
            .count();
        assert!(kaefer >= 3);
    }

    #[test]
    fn shipped_prolog_maps_are_current() {
        for (name, map) in [("tauwinkel", tauwinkel()), ("wiese-1", wiese())] {
            let file = std::fs::read(shipped(name)).expect("Karte vorhanden");
            assert_eq!(
                elora_map::decode(&file).expect("gültig"),
                map,
                "{name} veraltet – write_prolog_maps -- --ignored"
            );
        }
    }

    #[test]
    #[ignore = "schreibt maps/abenteuer/*.emap"]
    fn write_prolog_maps() {
        for (name, map) in [("tauwinkel", tauwinkel()), ("wiese-1", wiese())] {
            map.save(std::path::Path::new(&shipped(name))).unwrap();
        }
    }

    /// Übersicht: `… prolog_sheets -- --ignored` → `target/prolog-<karte>.svg`.
    #[test]
    #[ignore = "erzeugt nur Dateien zur Sichtprüfung"]
    fn prolog_sheets() {
        use crate::editor::panel::Preview;
        use crate::editor::view;
        for (name, map) in [("tauwinkel", tauwinkel()), ("wiese-1", wiese())] {
            let mut editor = Editor::new(None, std::path::PathBuf::from("maps"));
            editor.map = map;
            editor.center_view();
            editor.visible.grid = false;
            let size =
                Vec2::new(editor.map.width as f32, editor.map.height as f32) * TILE_SIZE as f32;
            let window = Vec2::new(2400.0, 2400.0 * size.y / size.x);
            editor.zoom = size.x / window.x;
            let cam = view::camera(&editor, window, window * 0.5);
            let mut batch = elora_render::ShapeBatch::default();
            view::draw(
                &mut batch,
                &editor,
                &mut crate::map_view::MapView::default(),
                &crate::items::ItemArt::load(),
                &cam,
                0.0,
                Preview::None,
            );
            let tl = cam.top_left();
            let svg = batch.debug_svg(tl, tl + cam.size, view::OUTSIDE);
            let path = format!(
                "{}/../../target/prolog-{name}.svg",
                env!("CARGO_MANIFEST_DIR")
            );
            std::fs::write(path, svg).unwrap();
        }
    }
}

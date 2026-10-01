//! Eingabe-Aufzeichnungen für Determinismus-Tests (M1.6).
//!
//! Eine Aufzeichnung ist in sich geschlossen: Kollisionsraster, Spawnpunkt, Tuning
//! und alle Eingaben stehen in einer Datei (`.erec.toml`). Beim Abspielen entsteht
//! ein Zustandsprotokoll, das bit-genau mit einer Golden-Datei verglichen wird.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::player::Controller;
use crate::{Collision, DummyPattern, PickupKind, PlayerInput, Tile, Tuning, Vec2, World};

/// Formatversion von `.erec.toml`.
pub const RECORDING_FORMAT: u32 = 2;

/// Alle wie viele Ticks der Zustand protokolliert wird.
const DUMP_INTERVAL: u64 = 25;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recording {
    pub format: u32,
    /// Kollisionsraster, Zeichen siehe [`Tile::to_char`].
    pub tiles: String,
    /// Spawnpunkte für den Respawn menschlicher Spieler.
    pub spawn_points: Vec<[f32; 2]>,
    /// Spieler-Slots in Reihenfolge; `dummy = None` ist der aufgezeichnete Mensch.
    pub players: Vec<RecordedPlayer>,
    pub pickups: Vec<RecordedPickup>,
    /// Eingaben des Menschen, eine Zeile pro Tick:
    /// `richtung ziel_x ziel_y sprung hook feuer waffe nächste vorige`.
    pub inputs: String,
    pub tuning: Tuning,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedPlayer {
    pub pos: [f32; 2],
    pub dummy: Option<DummyPattern>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedPickup {
    pub kind: PickupKind,
    pub pos: [f32; 2],
}

/// Formatversion 1 (M1): ein Spieler ohne Waffen, Eingaben mit 5 Feldern.
#[derive(Debug, Deserialize)]
struct RecordingV1 {
    spawn: [f32; 2],
    tiles: String,
    inputs: String,
    tuning: Tuning,
}

impl From<RecordingV1> for Recording {
    fn from(v1: RecordingV1) -> Self {
        let mut inputs = String::with_capacity(v1.inputs.len() * 2);
        for line in v1.inputs.lines() {
            let _ = writeln!(inputs, "{line} 0 0 0 0");
        }
        Self {
            format: RECORDING_FORMAT,
            tiles: v1.tiles,
            spawn_points: vec![v1.spawn],
            players: vec![RecordedPlayer {
                pos: v1.spawn,
                dummy: None,
            }],
            pickups: Vec::new(),
            inputs,
            tuning: v1.tuning,
        }
    }
}

/// Fehler beim Lesen einer Aufzeichnung.
#[derive(Debug)]
pub struct RecordingError(pub String);

impl std::fmt::Display for RecordingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RecordingError {}

impl Recording {
    /// Beginnt eine Aufzeichnung für den Zustand von `world`. Erwartet eine
    /// frische Welt (alle Figuren lebend an ihrer Startposition, Pickups verfügbar,
    /// keine Projektile) mit genau einem menschlichen Spieler.
    pub fn new(world: &World) -> Self {
        let collision = &world.collision;
        let mut tiles = String::new();
        for y in 0..collision.height() {
            for x in 0..collision.width() {
                #[allow(clippy::cast_possible_wrap)]
                tiles.push(collision.tile(x as i32, y as i32).to_char());
            }
            tiles.push('\n');
        }
        let players = world
            .players
            .iter()
            .flatten()
            .filter_map(|p| {
                let c = p.character.as_ref()?;
                let dummy = match &p.controller {
                    Controller::Human | Controller::Remote => None,
                    Controller::Dummy { pattern, .. } => Some(*pattern),
                };
                Some(RecordedPlayer {
                    pos: [c.core.pos.x, c.core.pos.y],
                    dummy,
                })
            })
            .collect();
        Self {
            format: RECORDING_FORMAT,
            tiles,
            spawn_points: world.spawn_points.iter().map(|p| [p.x, p.y]).collect(),
            players,
            pickups: world
                .pickups
                .iter()
                .map(|p| RecordedPickup {
                    kind: p.kind,
                    pos: [p.pos.x, p.pos.y],
                })
                .collect(),
            inputs: String::new(),
            tuning: world.tuning.clone(),
        }
    }

    pub fn push(&mut self, input: &PlayerInput) {
        let _ = writeln!(
            self.inputs,
            "{} {} {} {} {} {} {} {} {} {}",
            input.direction,
            input.target_x,
            input.target_y,
            u8::from(input.jump),
            u8::from(input.hook),
            input.fire,
            input.wanted_weapon,
            input.next_weapon,
            input.prev_weapon,
            u8::from(input.down),
        );
    }

    pub fn len(&self) -> usize {
        self.inputs.lines().count()
    }

    pub fn is_empty(&self) -> bool {
        self.inputs.is_empty()
    }

    /// # Errors
    /// Wenn `src` kein gültiges TOML einer Aufzeichnung ist.
    pub fn from_toml(src: &str) -> Result<Self, RecordingError> {
        let err = |e: toml::de::Error| RecordingError(e.to_string());
        let value: toml::Table = toml::from_str(src).map_err(err)?;
        match value.get("format").and_then(toml::Value::as_integer) {
            Some(1) => Ok(toml::from_str::<RecordingV1>(src).map_err(err)?.into()),
            Some(2) => toml::from_str(src).map_err(err),
            other => Err(RecordingError(format!(
                "Formatversion {other:?} nicht unterstützt"
            ))),
        }
    }

    /// # Errors
    /// Wenn die Serialisierung fehlschlägt.
    pub fn to_toml(&self) -> Result<String, RecordingError> {
        toml::to_string(self).map_err(|e| RecordingError(e.to_string()))
    }

    fn collision(&self) -> Result<Collision, RecordingError> {
        let rows: Vec<&str> = self.tiles.lines().filter(|l| !l.is_empty()).collect();
        let width = rows.first().map_or(0, |r| r.chars().count());
        let mut tiles = Vec::with_capacity(width * rows.len());
        for (y, row) in rows.iter().enumerate() {
            if row.chars().count() != width {
                return Err(RecordingError(format!(
                    "Rasterzeile {} hat falsche Länge",
                    y + 1
                )));
            }
            for c in row.chars() {
                tiles.push(
                    Tile::from_char(c)
                        .ok_or_else(|| RecordingError(format!("unbekanntes Tile `{c}`")))?,
                );
            }
        }
        Ok(Collision::new(width, rows.len(), tiles))
    }

    fn parse_inputs(&self) -> Result<Vec<PlayerInput>, RecordingError> {
        self.inputs
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let err = || RecordingError(format!("Eingabezeile {}: `{line}`", i + 1));
                let f: Vec<i32> = line
                    .split_whitespace()
                    .map(str::parse)
                    .collect::<Result<_, _>>()
                    .map_err(|_| err())?;
                // 9 Felder (ältere Aufzeichnungen) oder 10 mit „Runter“ (M6.1)
                let (fields, down) = match f.len() {
                    9 => (&f[..], 0),
                    10 => (&f[..9], f[9]),
                    _ => return Err(err()),
                };
                let [
                    direction,
                    target_x,
                    target_y,
                    jump,
                    hook,
                    fire,
                    wanted,
                    next,
                    prev,
                ] = fields[..]
                else {
                    return Err(err());
                };
                let byte = |v: i32| u8::try_from(v).map_err(|_| err());
                Ok(PlayerInput {
                    direction: i8::try_from(direction).map_err(|_| err())?,
                    target_x,
                    target_y,
                    jump: jump != 0,
                    hook: hook != 0,
                    fire: byte(fire)?,
                    wanted_weapon: byte(wanted)?,
                    next_weapon: byte(next)?,
                    prev_weapon: byte(prev)?,
                    down: down != 0,
                    ability: false,
                })
            })
            .collect()
    }

    /// Baut die Welt zum Start der Aufzeichnung.
    ///
    /// # Errors
    /// Bei ungültigem Raster.
    pub fn world(&self) -> Result<(World, usize), RecordingError> {
        let v = |p: [f32; 2]| Vec2::new(p[0], p[1]);
        let mut world = World::new(self.tuning.clone(), self.collision()?);
        world.spawn_points = self.spawn_points.iter().copied().map(v).collect();
        let mut human = None;
        for p in &self.players {
            match p.dummy {
                Some(pattern) => {
                    world.add_dummy(v(p.pos), pattern);
                }
                None => human = Some(world.spawn(v(p.pos))),
            }
        }
        for p in &self.pickups {
            world.add_pickup(p.kind, v(p.pos));
        }
        let human = human.ok_or_else(|| RecordingError("kein menschlicher Spieler".into()))?;
        Ok((world, human))
    }

    /// Spielt die Aufzeichnung ab und liefert das Zustandsprotokoll.
    ///
    /// # Errors
    /// Bei ungültigem Raster oder ungültigen Eingabezeilen.
    pub fn replay(&self) -> Result<String, RecordingError> {
        let (mut world, human) = self.world()?;
        let mut inputs = vec![PlayerInput::default(); world.players.len()];
        let mut log = String::new();
        for input in self.parse_inputs()? {
            inputs[human] = input;
            world.step(&inputs);
            if world.tick.is_multiple_of(DUMP_INTERVAL) {
                dump(&mut log, &world);
            }
        }
        let _ = write!(log, "ende ");
        dump(&mut log, &world);
        Ok(log)
    }
}

/// Zustand aller Slots. `{:?}` gibt f32 verlustfrei aus.
fn dump(log: &mut String, world: &World) {
    let _ = writeln!(
        log,
        "tick {}: projektile {} laser {}",
        world.tick,
        world.projectiles.len(),
        world.lasers.len()
    );
    for (i, p) in world.players.iter().enumerate() {
        let Some(p) = p else { continue };
        let Some(ch) = &p.character else {
            let _ = writeln!(log, "  p{i}: tot seit {}", p.die_tick);
            continue;
        };
        let c = &ch.core;
        let _ = writeln!(
            log,
            "  p{i}: pos ({:?}, {:?}) vel ({:?}, {:?}) hook {:?} ({:?}, {:?}) sprung {} hp {} rüstung {} waffe {:?} munition {:?}",
            c.pos.x,
            c.pos.y,
            c.vel.x,
            c.vel.y,
            c.hook_state,
            c.hook_pos.x,
            c.hook_pos.y,
            c.jumped,
            ch.health,
            ch.armor,
            ch.arsenal.active,
            ch.arsenal.slots.iter().map(|s| s.ammo).collect::<Vec<_>>(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_replay() {
        let col = Collision::new(3, 3, {
            let mut t = vec![Tile::Solid; 9];
            t[4] = Tile::Air;
            t
        });
        let mut world = World::new(Tuning::default(), col);
        world.spawn(Vec2::new(48.0, 48.0));
        let mut rec = Recording::new(&world);
        rec.push(&PlayerInput {
            direction: 1,
            jump: true,
            fire: 1,
            ..PlayerInput::default()
        });
        rec.push(&PlayerInput::default());
        let text = rec.to_toml().unwrap();
        let back = Recording::from_toml(&text).unwrap();
        assert_eq!(back, rec);
        assert_eq!(back.len(), 2);
        assert!(back.replay().unwrap().starts_with("ende tick 2:"));
    }
}

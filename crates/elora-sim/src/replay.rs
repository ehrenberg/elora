//! Eingabe-Aufzeichnungen für Determinismus-Tests (M1.6).
//!
//! Eine Aufzeichnung ist in sich geschlossen: Kollisionsraster, Spawnpunkt, Tuning
//! und alle Eingaben stehen in einer Datei (`.erec.toml`). Beim Abspielen entsteht
//! ein Zustandsprotokoll, das bit-genau mit einer Golden-Datei verglichen wird.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::{CharacterCore, Collision, PlayerInput, Tile, Tuning, Vec2, World};

/// Formatversion von `.erec.toml`.
pub const RECORDING_FORMAT: u32 = 1;

/// Alle wie viele Ticks der Zustand protokolliert wird.
const DUMP_INTERVAL: u64 = 25;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recording {
    pub format: u32,
    pub spawn: [f32; 2],
    /// Kollisionsraster: `.` Luft, `#` Wand, `%` unhookable, `^` Tod.
    pub tiles: String,
    /// Eingaben, eine Zeile pro Tick: `richtung ziel_x ziel_y sprung hook`.
    pub inputs: String,
    pub tuning: Tuning,
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
    /// Beginnt eine Aufzeichnung für die aktuelle Welt.
    pub fn new(collision: &Collision, spawn: Vec2, tuning: Tuning) -> Self {
        let mut tiles = String::new();
        for y in 0..collision.height() {
            for x in 0..collision.width() {
                #[allow(clippy::cast_possible_wrap)]
                tiles.push(match collision.tile(x as i32, y as i32) {
                    Tile::Air => '.',
                    Tile::Solid => '#',
                    Tile::Unhookable => '%',
                    Tile::Death => '^',
                });
            }
            tiles.push('\n');
        }
        Self {
            format: RECORDING_FORMAT,
            spawn: [spawn.x, spawn.y],
            tiles,
            inputs: String::new(),
            tuning,
        }
    }

    pub fn push(&mut self, input: &PlayerInput) {
        let _ = writeln!(
            self.inputs,
            "{} {} {} {} {}",
            input.direction,
            input.target_x,
            input.target_y,
            u8::from(input.jump),
            u8::from(input.hook),
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
        let rec: Self = toml::from_str(src).map_err(|e| RecordingError(e.to_string()))?;
        if rec.format != RECORDING_FORMAT {
            return Err(RecordingError(format!(
                "Formatversion {} nicht unterstützt",
                rec.format
            )));
        }
        Ok(rec)
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
                tiles.push(match c {
                    '.' => Tile::Air,
                    '#' => Tile::Solid,
                    '%' => Tile::Unhookable,
                    '^' => Tile::Death,
                    other => return Err(RecordingError(format!("unbekanntes Tile `{other}`"))),
                });
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
                let [direction, target_x, target_y, jump, hook] = f[..] else {
                    return Err(err());
                };
                Ok(PlayerInput {
                    direction: i8::try_from(direction).map_err(|_| err())?,
                    target_x,
                    target_y,
                    jump: jump != 0,
                    hook: hook != 0,
                })
            })
            .collect()
    }

    /// Spielt die Aufzeichnung ab und liefert das Zustandsprotokoll.
    ///
    /// # Errors
    /// Bei ungültigem Raster oder ungültigen Eingabezeilen.
    ///
    /// # Panics
    /// Nie im Normalbetrieb: Der Spieler-Slot wird hier selbst angelegt.
    pub fn replay(&self) -> Result<String, RecordingError> {
        let mut world = World::new(self.tuning.clone(), self.collision()?);
        let player = world.spawn(Vec2::new(self.spawn[0], self.spawn[1]));
        let mut log = String::new();
        for input in self.parse_inputs()? {
            world.step(&[input]);
            let core = world.characters[player]
                .as_ref()
                .expect("Spieler existiert");
            if world.tick.is_multiple_of(DUMP_INTERVAL) {
                dump(&mut log, world.tick, core);
            }
            // wie in der Sandbox: Tod → Respawn am Startpunkt
            if core.death {
                world.characters[player] =
                    Some(CharacterCore::new(Vec2::new(self.spawn[0], self.spawn[1])));
            }
        }
        let core = world.characters[player]
            .as_ref()
            .expect("Spieler existiert");
        let _ = write!(log, "ende ");
        dump(&mut log, world.tick, core);
        Ok(log)
    }
}

/// Eine Zeile Zustand. `{:?}` gibt f32 verlustfrei aus.
fn dump(log: &mut String, tick: u64, c: &CharacterCore) {
    let _ = writeln!(
        log,
        "tick {tick}: pos ({:?}, {:?}) vel ({:?}, {:?}) hook {:?} ({:?}, {:?}) sprung {}",
        c.pos.x, c.pos.y, c.vel.x, c.vel.y, c.hook_state, c.hook_pos.x, c.hook_pos.y, c.jumped
    );
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
        let mut rec = Recording::new(&col, Vec2::new(48.0, 48.0), Tuning::default());
        rec.push(&PlayerInput {
            direction: 1,
            jump: true,
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

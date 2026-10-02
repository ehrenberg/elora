//! Werkzeug „Abenteuer“ (A1.8, E-268): Gegner, NPCs, Objekte, Zonen und Übergänge setzen,
//! wählen, verschieben und löschen – Zustand und Logik ohne egui (testbar).

// Raster-Code: `x`/`y` (Tile), `c` (Mitte), `h` (Höhe) wie in tools.rs
#![allow(clippy::many_single_char_names)]

use std::time::Instant;

use elora_map::adventure::{CameraMode, SwitchTrigger};
use elora_map::{Object, ObjectKind};
use elora_sim::{TILE_SIZE, Vec2};

use super::Editor;

/// Arten in der Auswahl der Seitenleiste.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Kind {
    #[default]
    Creature,
    Npc,
    Chest,
    Switch,
    Door,
    Collectible,
    SavePoint,
    HealPlant,
    Spawn,
    Exit,
    Zone,
    Camera,
}

impl Kind {
    pub const ALL: [Self; 12] = [
        Self::Creature,
        Self::Npc,
        Self::Chest,
        Self::Switch,
        Self::Door,
        Self::Collectible,
        Self::SavePoint,
        Self::HealPlant,
        Self::Spawn,
        Self::Exit,
        Self::Zone,
        Self::Camera,
    ];

    /// Sprachschlüssel (`editor.adv_<…>`), gleich dem Namen in der Karte.
    pub fn key(self) -> &'static str {
        match self {
            Self::Creature => "editor.adv_gegner",
            Self::Npc => "editor.adv_npc",
            Self::Chest => "editor.adv_truhe",
            Self::Switch => "editor.adv_schalter",
            Self::Door => "editor.adv_tuer",
            Self::Collectible => "editor.adv_sammelstueck",
            Self::SavePoint => "editor.adv_speicherpunkt",
            Self::HealPlant => "editor.adv_heilpflanze",
            Self::Spawn => "editor.adv_eingang",
            Self::Exit => "editor.adv_uebergang",
            Self::Zone => "editor.adv_zone",
            Self::Camera => "editor.adv_kamera",
        }
    }

    /// Bereiche werden aufgezogen, alles andere per Klick gesetzt.
    pub fn is_area(self) -> bool {
        matches!(self, Self::Door | Self::Exit | Self::Zone | Self::Camera)
    }

    pub fn of(k: &ObjectKind) -> Self {
        match k {
            ObjectKind::Creature { .. } => Self::Creature,
            ObjectKind::Npc { .. } => Self::Npc,
            ObjectKind::Chest { .. } => Self::Chest,
            ObjectKind::Switch { .. } => Self::Switch,
            ObjectKind::Door { .. } => Self::Door,
            ObjectKind::Collectible { .. } => Self::Collectible,
            ObjectKind::SavePoint => Self::SavePoint,
            ObjectKind::HealPlant { .. } => Self::HealPlant,
            ObjectKind::Spawn => Self::Spawn,
            ObjectKind::Exit { .. } => Self::Exit,
            ObjectKind::Zone { .. } => Self::Zone,
            ObjectKind::Camera { .. } => Self::Camera,
        }
    }
}

/// Höhe eines Punkt-Objekts (Mitte über dem Boden, wie im Spiel gezeichnet).
pub fn height(k: &ObjectKind, creature_height: impl Fn(&str) -> Option<f32>) -> Option<f32> {
    Some(match k {
        ObjectKind::Creature { kind, .. } => creature_height(kind).unwrap_or(28.0),
        ObjectKind::Npc { .. } | ObjectKind::Spawn => 28.0,
        ObjectKind::Chest { .. } => 26.0,
        ObjectKind::Switch { .. } => 30.0,
        ObjectKind::SavePoint => 40.0,
        ObjectKind::HealPlant { .. } => 16.0,
        // Sammelstücke schweben in der Tile-Mitte, Bereiche haben keine Höhe
        _ => return None,
    })
}

/// Vorgaben für ein neues Objekt; Listen kommen aus den Inhalten (erste Gegnerart usw.).
#[derive(Debug, Clone, Default)]
pub struct Defaults {
    pub creature: String,
    pub character: String,
    pub item: String,
}

pub fn new_kind(kind: Kind, d: &Defaults, size: Vec2) -> ObjectKind {
    match kind {
        Kind::Creature => ObjectKind::Creature {
            kind: d.creature.clone(),
            persistent: false,
        },
        Kind::Npc => ObjectKind::Npc {
            character: d.character.clone(),
            dialog: d.character.clone(),
            facing: 1,
            walk: 0.0,
        },
        Kind::Chest => ObjectKind::Chest {
            contents: vec![("glanztropfen".into(), 10)],
            lock: String::new(),
        },
        Kind::Switch => ObjectKind::Switch {
            flag: "schalter".into(),
            once: false,
            trigger: SwitchTrigger::Interact,
        },
        Kind::Door => {
            let ts = TILE_SIZE as f32;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let tiles = |v: f32| (v / ts).round().clamp(1.0, 255.0) as u8;
            ObjectKind::Door {
                size: (tiles(size.x), tiles(size.y)),
                open_if: "merker schalter".into(),
            }
        }
        Kind::Collectible => ObjectKind::Collectible {
            item: d.item.clone(),
        },
        Kind::SavePoint => ObjectKind::SavePoint,
        Kind::HealPlant => ObjectKind::HealPlant { heal: 2 },
        Kind::Spawn => ObjectKind::Spawn,
        Kind::Exit => ObjectKind::Exit {
            size,
            map: String::new(),
            spawn: String::new(),
            on_touch: true,
        },
        Kind::Zone => ObjectKind::Zone { size },
        Kind::Camera => ObjectKind::Camera {
            size,
            mode: CameraMode::Fixed,
        },
    }
}

/// Zustand des Werkzeugs.
#[derive(Debug, Clone, Default)]
pub struct AdventureTool {
    pub kind: Kind,
    pub selected: Option<String>,
    /// Ziehen: Weltpunkt beim Drücken und Lage des Objekts davor.
    pub drag: Option<(Vec2, Vec2)>,
    /// Bereich aufziehen: Start-Tile.
    pub area_start: Option<(usize, usize)>,
    pub defaults: Defaults,
}

fn tile_center(x: usize, y: usize) -> Vec2 {
    let ts = TILE_SIZE as f32;
    #[allow(clippy::cast_precision_loss)]
    Vec2::new(x as f32 * ts + ts / 2.0, y as f32 * ts + ts / 2.0)
}

impl Editor {
    /// Neue eindeutige Id wie `truhe-3`.
    pub fn new_object_id(&self, kind: &ObjectKind) -> String {
        let base = kind.name();
        let n = self.map.adventure.objects.len() + 1;
        (1..=n)
            .map(|k| format!("{base}-{k}"))
            .find(|id| self.map.adventure.object(id).is_none())
            .unwrap_or_else(|| format!("{base}-{}", n + 1))
    }

    /// Lage eines Punkt-Objekts auf Tile (x, y): auf dem ersten festen Boden darunter.
    pub fn snap_point(&self, x: usize, y: usize, height: Option<f32>) -> Vec2 {
        let c = tile_center(x, y);
        let Some(h) = height else { return c };
        let ts = TILE_SIZE as f32;
        let floor = (y..self.map.height)
            .find(|&ty| {
                let t = self.map.tiles[ty * self.map.width + x];
                t.is_solid() || t == elora_sim::Tile::Platform
            })
            .unwrap_or(self.map.height);
        #[allow(clippy::cast_precision_loss)]
        let top = floor as f32 * ts;
        Vec2::new(c.x, top - h / 2.0 - 1.0)
    }

    /// Punkt-Objekt der gewählten Art auf Tile (x, y) setzen und wählen.
    pub fn place_object(
        &mut self,
        x: usize,
        y: usize,
        height: impl Fn(&str) -> Option<f32>,
        now: Instant,
    ) {
        let kind = new_kind(self.adventure.kind, &self.adventure.defaults, Vec2::ZERO);
        let pos = self.snap_point(x, y, super::adventure::height(&kind, height));
        self.add_object(pos, kind, now);
    }

    /// Bereich über die Tiles `a`–`b` (einschließlich) setzen.
    pub fn place_area(&mut self, a: (usize, usize), b: (usize, usize), now: Instant) {
        let ts = TILE_SIZE as f32;
        let (x0, x1) = (a.0.min(b.0), a.0.max(b.0));
        let (y0, y1) = (a.1.min(b.1), a.1.max(b.1));
        #[allow(clippy::cast_precision_loss)]
        let pos = Vec2::new(x0 as f32 * ts, y0 as f32 * ts);
        #[allow(clippy::cast_precision_loss)]
        let size = Vec2::new((x1 - x0 + 1) as f32 * ts, (y1 - y0 + 1) as f32 * ts);
        let kind = new_kind(self.adventure.kind, &self.adventure.defaults, size);
        self.add_object(pos, kind, now);
    }

    fn add_object(&mut self, pos: Vec2, kind: ObjectKind, now: Instant) {
        self.begin_edit("adv-add", now);
        self.end_edit();
        let id = self.new_object_id(&kind);
        self.map.adventure.objects.push(Object {
            id: id.clone(),
            pos,
            kind,
        });
        self.adventure.selected = Some(id);
    }

    /// Objekt unter dem Weltpunkt (Punkte vor Bereichen, kleinere Bereiche zuerst).
    pub fn object_at(&self, p: Vec2) -> Option<String> {
        let objects = &self.map.adventure.objects;
        let point = objects
            .iter()
            .filter(|o| o.kind.area().is_none())
            .map(|o| (o, o.pos.distance(p)))
            .filter(|(_, d)| *d < 22.0)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((o, _)) = point {
            return Some(o.id.clone());
        }
        objects
            .iter()
            .filter_map(|o| Some((o, o.kind.area()?)))
            .filter(|(o, s)| {
                p.x >= o.pos.x && p.y >= o.pos.y && p.x <= o.pos.x + s.x && p.y <= o.pos.y + s.y
            })
            .min_by(|a, b| (a.1.x * a.1.y).total_cmp(&(b.1.x * b.1.y)))
            .map(|(o, _)| o.id.clone())
    }

    pub fn selected_object(&self) -> Option<&Object> {
        self.map
            .adventure
            .object(self.adventure.selected.as_deref()?)
    }

    /// Gewähltes Objekt zum Bearbeiten (ein Rückgängig-Schritt je Feld und kurzer Zeit).
    pub fn edit_object(&mut self, what: &str, now: Instant) -> Option<&mut Object> {
        let id = self.adventure.selected.clone()?;
        self.map.adventure.object(&id)?;
        self.begin_edit(&format!("adv-{id}-{what}"), now);
        self.map.adventure.objects.iter_mut().find(|o| o.id == id)
    }

    /// Gewähltes Objekt um `delta` verschieben, auf ganze Tiles gerastert; Türen bleiben auf
    /// dem Raster (E-254).
    pub fn move_object(&mut self, start: Vec2, delta: Vec2, now: Instant) {
        let ts = TILE_SIZE as f32;
        let snapped = Vec2::new((delta.x / ts).round() * ts, (delta.y / ts).round() * ts);
        let size = Vec2::new(self.map.width as f32 * ts, self.map.height as f32 * ts);
        let Some(o) = self.edit_object("move", now) else {
            return;
        };
        let area = o.kind.area().unwrap_or(Vec2::ZERO);
        let p = start + snapped;
        o.pos = Vec2::new(
            p.x.clamp(0.0, size.x - area.x),
            p.y.clamp(0.0, size.y - area.y),
        );
    }

    /// Gewähltes Objekt umbenennen; `false`, wenn die Id leer, ungültig oder vergeben ist.
    pub fn rename_object(&mut self, new_id: &str, now: Instant) -> bool {
        let valid = !new_id.is_empty() && !new_id.contains(':') && new_id.len() <= 128;
        if !valid || self.map.adventure.object(new_id).is_some() {
            return false;
        }
        let Some(o) = self.edit_object("id", now) else {
            return false;
        };
        new_id.clone_into(&mut o.id);
        self.adventure.selected = Some(new_id.to_owned());
        true
    }

    pub fn remove_object(&mut self, id: &str, now: Instant) {
        if self.map.adventure.object(id).is_some() {
            self.begin_edit("adv-remove", now);
            self.end_edit();
            self.map.adventure.objects.retain(|o| o.id != id);
            if self.adventure.selected.as_deref() == Some(id) {
                self.adventure.selected = None;
            }
        }
    }

    /// Ist die Karte eine Abenteuer-Karte (hat Objekte)?
    pub fn is_adventure_map(&self) -> bool {
        !self.map.adventure.objects.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_map::Map;

    fn editor() -> Editor {
        let mut e = Editor::new(None, std::path::PathBuf::new());
        e.map = Map::from_rows(
            "T",
            &[
                "#########",
                "#S......#",
                "#.......#",
                "#.......#",
                "#########",
            ],
        )
        .unwrap();
        e.adventure.defaults = Defaults {
            creature: "stachelkaefer".into(),
            character: "oma".into(),
            item: "glitzerstein".into(),
        };
        e
    }

    #[test]
    fn place_snaps_to_ground_and_gets_unique_ids() {
        let mut e = editor();
        let now = Instant::now();
        e.adventure.kind = Kind::Chest;
        e.place_object(3, 1, |_| None, now);
        e.place_object(5, 2, |_| None, now);
        let objs = &e.map.adventure.objects;
        assert_eq!(objs.len(), 2);
        assert_eq!(objs[0].id, "truhe-1");
        assert_eq!(objs[1].id, "truhe-2");
        // Boden in Zeile 4: Oberkante 128, Truhe 26 hoch
        assert!((objs[0].pos.y - (128.0 - 13.0 - 1.0)).abs() < 0.01);
        assert_eq!(e.adventure.selected.as_deref(), Some("truhe-2"));
        e.undo();
        assert_eq!(e.map.adventure.objects.len(), 1);
    }

    #[test]
    fn areas_select_move_rename_remove() {
        let mut e = editor();
        let now = Instant::now();
        e.adventure.kind = Kind::Door;
        e.place_area((4, 1), (4, 3), now);
        let door = e.selected_object().unwrap().clone();
        assert_eq!(
            door.kind,
            ObjectKind::Door {
                size: (1, 3),
                open_if: "merker schalter".into()
            }
        );
        assert_eq!(
            e.object_at(door.pos + Vec2::new(10.0, 50.0)),
            Some(door.id.clone())
        );
        e.move_object(door.pos, Vec2::new(40.0, -10.0), now);
        assert_eq!(
            e.selected_object().unwrap().pos,
            door.pos + Vec2::new(32.0, 0.0),
            "auf Tiles"
        );
        assert!(e.rename_object("tor", now));
        assert!(!e.rename_object("a:b", now));
        e.remove_object("tor", now);
        assert!(e.map.adventure.objects.is_empty() && e.adventure.selected.is_none());
        // Karte bleibt gültig speicherbar
        e.adventure.kind = Kind::Spawn;
        e.place_object(2, 1, |_| None, now);
        assert!(elora_map::decode(&elora_map::encode(&e.map)).is_ok());
    }
}

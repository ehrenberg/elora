//! Werkzeuge des Editors (M6.7): Pinsel, Rechteck, Füllen, Radierer, Auswahl mit Kopieren und
//! Einfügen, Entities, Material. Reine Logik auf [`Editor`]; jede Aktion ist rückgängig machbar.

// Raster-Code: `x`/`y`/`w`/`h`/`i` sind hier lesbarer als lange Namen
#![allow(clippy::many_single_char_names)]

use std::time::Instant;

use elora_map::{Entity, EntityKind};
use elora_sim::{DummyPattern, Tile};

use super::Editor;

/// Gewähltes Werkzeug (Tasten 1–7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    Brush,
    Rect,
    Fill,
    Eraser,
    Select,
    Entity,
    Material,
    /// Deko platzieren und bearbeiten (M6.8).
    Decor,
}

impl Tool {
    pub const ALL: [Self; 8] = [
        Self::Brush,
        Self::Rect,
        Self::Fill,
        Self::Eraser,
        Self::Select,
        Self::Entity,
        Self::Material,
        Self::Decor,
    ];

    /// Sprachschlüssel.
    pub fn key(self) -> &'static str {
        match self {
            Self::Brush => "editor.tool_brush",
            Self::Rect => "editor.tool_rect",
            Self::Fill => "editor.tool_fill",
            Self::Eraser => "editor.tool_eraser",
            Self::Select => "editor.tool_select",
            Self::Entity => "editor.tool_entity",
            Self::Material => "editor.tool_material",
            Self::Decor => "editor.tool_decor",
        }
    }
}

/// Setzbare Entities mit Sprachschlüssel.
pub const ENTITIES: [(EntityKind, &str); 13] = [
    (EntityKind::Spawn, "editor.ent_spawn"),
    (EntityKind::SpawnRed, "editor.ent_spawn_red"),
    (EntityKind::SpawnBlue, "editor.ent_spawn_blue"),
    (EntityKind::FlagRed, "editor.ent_flag_red"),
    (EntityKind::FlagBlue, "editor.ent_flag_blue"),
    (EntityKind::Health, "editor.ent_health"),
    (EntityKind::Armor, "editor.ent_armor"),
    (EntityKind::Laser, "editor.ent_laser"),
    (EntityKind::Grenade, "editor.ent_grenade"),
    (EntityKind::Dummy(DummyPattern::Stand), "editor.ent_dummy"),
    (
        EntityKind::Dummy(DummyPattern::Walk),
        "editor.ent_dummy_walk",
    ),
    (
        EntityKind::Dummy(DummyPattern::Jump),
        "editor.ent_dummy_jump",
    ),
    (
        EntityKind::Dummy(DummyPattern::WalkJump),
        "editor.ent_dummy_walk_jump",
    ),
];

/// Rechteck in Tiles (einschließlich).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cells {
    pub x0: usize,
    pub y0: usize,
    pub x1: usize,
    pub y1: usize,
}

impl Cells {
    /// Aus zwei beliebigen Ecken.
    pub fn span(a: (usize, usize), b: (usize, usize)) -> Self {
        Self {
            x0: a.0.min(b.0),
            y0: a.1.min(b.1),
            x1: a.0.max(b.0),
            y1: a.1.max(b.1),
        }
    }

    pub fn width(self) -> usize {
        self.x1 - self.x0 + 1
    }

    pub fn height(self) -> usize {
        self.y1 - self.y0 + 1
    }

    pub fn contains(self, x: usize, y: usize) -> bool {
        (self.x0..=self.x1).contains(&x) && (self.y0..=self.y1).contains(&y)
    }
}

/// Kopierter Ausschnitt: Tiles, Material (Name, `None` = Standard) und Entities relativ zur Ecke.
#[derive(Debug, Clone, PartialEq)]
pub struct Clip {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
    pub materials: Vec<Option<String>>,
    pub entities: Vec<(EntityKind, usize, usize)>,
}

impl Editor {
    /// Materialname eines Tiles (`None` = Standard der Tile-Art).
    fn material_name(&self, i: usize) -> Option<String> {
        let m = *self.map.material_map.get(i)?;
        self.map
            .materials
            .get(usize::from(m).checked_sub(1)?)
            .cloned()
    }

    /// Material-Index für `name` (legt den Namen in der Karte an); Standard → 0.
    fn material_code(&mut self, name: Option<&str>) -> u8 {
        let Some(name) = name.filter(|n| self.solid_materials.first().is_none_or(|d| d != n))
        else {
            return 0;
        };
        let i = self
            .map
            .materials
            .iter()
            .position(|m| m == name)
            .unwrap_or_else(|| {
                self.map.materials.push(name.to_owned());
                self.map.materials.len() - 1
            });
        u8::try_from(i + 1).unwrap_or(0)
    }

    /// Ein Feld setzen, ohne Verlauf. Material gilt nur für feste Tiles (Stein und Eis sind fest
    /// zugeordnet, E-148). Liefert, ob sich etwas geändert hat.
    fn set_cell(&mut self, x: usize, y: usize, tile: Tile, material: Option<&str>) -> bool {
        let i = y * self.map.width + x;
        let code = if tile == Tile::Solid {
            self.material_code(material)
        } else {
            0
        };
        let old_code = self.map.material_map.get(i).copied().unwrap_or(0);
        if self.map.tiles[i] == tile && old_code == code {
            return false;
        }
        self.map.tiles[i] = tile;
        if code != 0 && self.map.material_map.is_empty() {
            self.map.material_map = vec![0; self.map.tiles.len()];
        }
        if let Some(m) = self.map.material_map.get_mut(i) {
            *m = code;
        }
        true
    }

    /// Material für neu gemalte feste Tiles.
    fn brush_material(&self) -> String {
        self.solid_material.clone()
    }

    /// Felder eines quadratischen Pinsels um (x, y), auf die Karte begrenzt.
    pub fn brush_cells(&self, x: usize, y: usize, size: usize) -> Cells {
        let r = size.max(1) - 1;
        let (lo, hi) = (r / 2, r - r / 2);
        Cells {
            x0: x.saturating_sub(lo),
            y0: y.saturating_sub(lo),
            x1: (x + hi).min(self.map.width - 1),
            y1: (y + hi).min(self.map.height - 1),
        }
    }

    /// Bereich mit einer Tile-Art füllen (ein Rückgängig-Schritt je `kind`).
    pub fn fill_cells(&mut self, cells: Cells, tile: Tile, kind: &str, now: Instant) {
        let material = self.brush_material();
        let mut changed = false;
        let before = self.map.clone();
        for y in cells.y0..=cells.y1 {
            for x in cells.x0..=cells.x1 {
                changed |= self.set_cell(x, y, tile, Some(&material));
            }
        }
        if changed {
            self.record(before, kind, now);
        }
    }

    /// Stand vor einer Änderung in den Verlauf übernehmen (wie [`Editor::begin_edit`], nur nachträglich).
    fn record(&mut self, before: elora_map::Map, kind: &str, now: Instant) {
        let after = std::mem::replace(&mut self.map, before);
        self.begin_edit(kind, now);
        self.map = after;
    }

    /// Zusammenhängende Fläche gleicher Tile-Art (und gleichen Materials) ab (x, y) füllen.
    pub fn flood_fill(&mut self, x: usize, y: usize, tile: Tile, now: Instant) {
        let (w, h) = (self.map.width, self.map.height);
        let start = y * w + x;
        let target = (self.map.tiles[start], self.material_name(start));
        let material = self.brush_material();
        let wanted_material = if tile == Tile::Solid {
            Some(material.clone()).filter(|m| self.solid_materials.first() != Some(m))
        } else {
            None
        };
        if target == (tile, wanted_material) {
            return;
        }
        let before = self.map.clone();
        let mut seen = vec![false; w * h];
        let mut stack = vec![(x, y)];
        seen[start] = true;
        while let Some((cx, cy)) = stack.pop() {
            self.set_cell(cx, cy, tile, Some(&material));
            let mut push = |nx: usize, ny: usize| {
                let i = ny * w + nx;
                if !seen[i] && (before.tiles[i], material_of(&before, i)) == target {
                    seen[i] = true;
                    stack.push((nx, ny));
                }
            };
            if cx > 0 {
                push(cx - 1, cy);
            }
            if cx + 1 < w {
                push(cx + 1, cy);
            }
            if cy > 0 {
                push(cx, cy - 1);
            }
            if cy + 1 < h {
                push(cx, cy + 1);
            }
        }
        self.end_edit();
        self.record(before, "fill", now);
        self.end_edit();
    }

    /// Material auf vorhandene feste Tiles malen (andere Tiles bleiben unberührt).
    pub fn paint_material(&mut self, cells: Cells, name: &str, kind: &str, now: Instant) {
        let before = self.map.clone();
        let mut changed = false;
        for y in cells.y0..=cells.y1 {
            for x in cells.x0..=cells.x1 {
                if self.map.tiles[y * self.map.width + x] == Tile::Solid {
                    changed |= self.set_cell(x, y, Tile::Solid, Some(name));
                }
            }
        }
        if changed {
            self.record(before, kind, now);
        }
    }

    /// Entity setzen (ersetzt eines auf demselben Feld). Flaggen gibt es je Team nur einmal:
    /// eine neue verschiebt die alte.
    pub fn place_entity(&mut self, x: usize, y: usize, kind: EntityKind, now: Instant) {
        if self
            .map
            .entities
            .iter()
            .any(|e| e.tx == x && e.ty == y && e.kind == kind)
        {
            return;
        }
        self.begin_edit("entity", now);
        self.end_edit();
        self.insert_entity(x, y, kind);
    }

    fn insert_entity(&mut self, x: usize, y: usize, kind: EntityKind) {
        let flag = matches!(kind, EntityKind::FlagRed | EntityKind::FlagBlue);
        self.map
            .entities
            .retain(|e| !((e.tx == x && e.ty == y) || (flag && e.kind == kind)));
        self.map.entities.push(Entity { kind, tx: x, ty: y });
    }

    /// Entity auf einem Feld entfernen.
    pub fn remove_entity(&mut self, x: usize, y: usize, now: Instant) {
        if self.map.entities.iter().any(|e| e.tx == x && e.ty == y) {
            self.begin_edit("entity", now);
            self.end_edit();
            self.map.entities.retain(|e| !(e.tx == x && e.ty == y));
        }
    }

    /// Ausschnitt kopieren.
    pub fn copy(&self, cells: Cells) -> Clip {
        let w = self.map.width;
        let mut clip = Clip {
            width: cells.width(),
            height: cells.height(),
            tiles: Vec::with_capacity(cells.width() * cells.height()),
            materials: Vec::new(),
            entities: Vec::new(),
        };
        for y in cells.y0..=cells.y1 {
            for x in cells.x0..=cells.x1 {
                clip.tiles.push(self.map.tiles[y * w + x]);
                clip.materials.push(self.material_name(y * w + x));
            }
        }
        clip.entities = self
            .map
            .entities
            .iter()
            .filter(|e| cells.contains(e.tx, e.ty))
            .map(|e| (e.kind, e.tx - cells.x0, e.ty - cells.y0))
            .collect();
        clip
    }

    /// Auswahl in die Zwischenablage.
    pub fn copy_selection(&mut self) {
        if let Some(sel) = self.selection {
            self.clipboard = Some(self.copy(sel));
        }
    }

    /// Auswahl leeren.
    pub fn delete_selection(&mut self, now: Instant) {
        if let Some(sel) = self.selection {
            self.clear_cells(sel, now);
        }
    }

    /// Ausschnitt leeren (Luft, keine Entities).
    pub fn clear_cells(&mut self, cells: Cells, now: Instant) {
        let before = self.map.clone();
        for y in cells.y0..=cells.y1 {
            for x in cells.x0..=cells.x1 {
                self.set_cell(x, y, Tile::Air, None);
            }
        }
        self.map.entities.retain(|e| !cells.contains(e.tx, e.ty));
        if self.map != before {
            self.end_edit();
            self.record(before, "clear", now);
            self.end_edit();
        }
    }

    /// Ausschnitt mit der linken oberen Ecke bei (x, y) einfügen; was über den Rand ragt, fällt weg.
    pub fn paste(&mut self, clip: &Clip, x: usize, y: usize, now: Instant) {
        let before = self.map.clone();
        let (w, h) = (self.map.width, self.map.height);
        let area = Cells {
            x0: x,
            y0: y,
            x1: (x + clip.width - 1).min(w - 1),
            y1: (y + clip.height - 1).min(h - 1),
        };
        self.map.entities.retain(|e| !area.contains(e.tx, e.ty));
        for cy in 0..clip.height {
            for cx in 0..clip.width {
                let (tx, ty) = (x + cx, y + cy);
                if tx < w && ty < h {
                    let i = cy * clip.width + cx;
                    let material = clip.materials[i]
                        .clone()
                        .or_else(|| self.solid_materials.first().cloned());
                    self.set_cell(tx, ty, clip.tiles[i], material.as_deref());
                }
            }
        }
        for &(kind, ex, ey) in &clip.entities {
            if x + ex < w && y + ey < h {
                self.insert_entity(x + ex, y + ey, kind);
            }
        }
        self.end_edit();
        self.record(before, "paste", now);
        self.end_edit();
    }
}

fn material_of(map: &elora_map::Map, i: usize) -> Option<String> {
    let m = *map.material_map.get(i)?;
    map.materials.get(usize::from(m).checked_sub(1)?).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::Editor;
    use std::path::PathBuf;

    fn editor() -> Editor {
        let mut e = Editor::new(None, PathBuf::from("maps"));
        e.solid_materials = vec!["earth".into(), "sand".into(), "snow".into()];
        e.solid_material = "earth".into();
        e
    }

    fn at(e: &Editor, x: usize, y: usize) -> Tile {
        e.map.tiles[y * e.map.width + x]
    }

    #[test]
    fn rect_and_brush_sizes() {
        let mut e = editor();
        let t = Instant::now();
        e.fill_cells(Cells::span((5, 5), (2, 3)), Tile::Ice, "rect", t);
        assert_eq!(at(&e, 2, 3), Tile::Ice);
        assert_eq!(at(&e, 5, 5), Tile::Ice);
        assert_eq!(at(&e, 6, 5), Tile::Air);
        let c = e.brush_cells(0, 0, 3);
        assert_eq!(
            (c.x0, c.y0, c.x1, c.y1),
            (0, 0, 1, 1),
            "mittig, am Rand abgeschnitten"
        );
        let c = e.brush_cells(10, 10, 2);
        assert_eq!((c.width(), c.height()), (2, 2));
        e.undo();
        assert_eq!(at(&e, 2, 3), Tile::Air);
    }

    #[test]
    fn flood_fill_stays_inside_walls() {
        let mut e = editor();
        let t = Instant::now();
        // Kasten aus Wänden 0..=4, innen frei
        e.fill_cells(Cells::span((0, 0), (4, 4)), Tile::Solid, "a", t);
        e.end_edit();
        e.fill_cells(Cells::span((1, 1), (3, 3)), Tile::Air, "b", t);
        e.end_edit();
        e.flood_fill(2, 2, Tile::Death, t);
        assert_eq!(at(&e, 1, 1), Tile::Death);
        assert_eq!(at(&e, 3, 3), Tile::Death);
        assert_eq!(at(&e, 0, 0), Tile::Solid);
        assert_eq!(at(&e, 6, 6), Tile::Air, "außen unberührt");
        e.undo();
        assert_eq!(at(&e, 2, 2), Tile::Air, "Füllen ist ein Schritt");
        // gleiche Füllung: nichts zu tun
        let steps = e.undo_len();
        e.flood_fill(10, 10, Tile::Air, t);
        assert_eq!(e.undo_len(), steps);
    }

    #[test]
    fn materials_only_on_solid_and_default_is_zero() {
        let mut e = editor();
        let t = Instant::now();
        e.solid_material = "sand".into();
        e.fill_cells(Cells::span((0, 0), (1, 0)), Tile::Solid, "a", t);
        e.fill_cells(Cells::span((2, 0), (2, 0)), Tile::Unhookable, "b", t);
        assert_eq!(e.map.materials, vec!["sand".to_owned()]);
        assert_eq!(&e.map.material_map[..3], &[1, 1, 0]);
        e.paint_material(Cells::span((0, 0), (2, 0)), "earth", "m", t);
        assert_eq!(
            &e.map.material_map[..3],
            &[0, 0, 0],
            "Standard = 0, Stein bleibt"
        );
        // Karte bleibt gültig
        let data = elora_map::encode(&e.map);
        assert!(elora_map::decode_draft(&data).is_ok());
    }

    #[test]
    fn entities_replace_and_flags_are_unique() {
        let mut e = editor();
        let t = Instant::now();
        e.place_entity(3, 3, EntityKind::Spawn, t);
        e.place_entity(3, 3, EntityKind::Health, t);
        assert_eq!(e.map.entities.len(), 1);
        assert_eq!(e.map.entities[0].kind, EntityKind::Health);
        e.place_entity(5, 5, EntityKind::FlagRed, t);
        e.place_entity(9, 5, EntityKind::FlagRed, t);
        let reds: Vec<_> = e
            .map
            .entities
            .iter()
            .filter(|x| x.kind == EntityKind::FlagRed)
            .collect();
        assert_eq!(reds.len(), 1);
        assert_eq!(reds[0].tx, 9);
        e.remove_entity(3, 3, t);
        assert_eq!(e.map.entities.len(), 1);
        e.undo();
        assert_eq!(e.map.entities.len(), 2);
    }

    #[test]
    fn copy_paste_and_clear() {
        let mut e = editor();
        let t = Instant::now();
        e.solid_material = "snow".into();
        e.fill_cells(Cells::span((1, 1), (2, 2)), Tile::Solid, "a", t);
        e.place_entity(1, 0, EntityKind::Spawn, t);
        let clip = e.copy(Cells::span((1, 0), (2, 2)));
        assert_eq!((clip.width, clip.height), (2, 3));
        assert_eq!(clip.entities, vec![(EntityKind::Spawn, 0, 0)]);
        e.paste(&clip, 20, 10, t);
        assert_eq!(at(&e, 21, 12), Tile::Solid);
        assert_eq!(
            e.material_name(12 * e.map.width + 21).as_deref(),
            Some("snow")
        );
        assert!(e.map.entities.iter().any(|x| (x.tx, x.ty) == (20, 10)));
        // am Rand: was übersteht, fällt weg
        let w = e.map.width;
        e.paste(&clip, w - 1, 0, t);
        assert_eq!(at(&e, w - 1, 1), Tile::Solid);
        e.clear_cells(Cells::span((20, 10), (21, 12)), t);
        assert_eq!(at(&e, 21, 12), Tile::Air);
        assert!(!e.map.entities.iter().any(|x| (x.tx, x.ty) == (20, 10)));
        e.undo();
        assert_eq!(at(&e, 21, 12), Tile::Solid);
    }
}

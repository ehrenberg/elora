//! Map editor (M6.6, E-028, E-031): egui dark (E-150), one sidebar on the right (E-151),
//! custom maps in the user directory (E-152).
//!
//! State and logic live here (without egui, testable); [`panel`] builds the UI,
//! [`view`] draws map, grid and entities.

pub mod adventure;
#[cfg(test)]
mod chapter1;
#[cfg(test)]
mod chapter2;
#[cfg(test)]
mod chapter3;
#[cfg(test)]
mod chapter4;
pub mod look;
pub mod panel;
pub mod panel_adventure;
pub mod panel_look;
#[cfg(test)]
mod prologue;
#[cfg(test)]
mod release;
#[cfg(test)]
mod release_layouts;
pub mod tools;
#[cfg(test)]
mod training;
pub mod view;
#[cfg(test)]
mod weather;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use elora_map::Map;
use elora_sim::{TILE_SIZE, Tile, Vec2};

use crate::map_view::Layers;
use tools::{Cells, Clip, Tool};

/// Undo steps that are kept.
const HISTORY: usize = 100;
/// Changes of the same kind within this time become one step (typing, dragging a color).
const MERGE_WINDOW: Duration = Duration::from_millis(800);
/// Zoom: world units per screen pixel.
pub const ZOOM_MIN: f32 = 0.25;
pub const ZOOM_MAX: f32 = 8.0;
/// Size of a new map (tiles).
pub const NEW_SIZE: (usize, usize) = (60, 30);

/// What is visible (layer list).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Visible {
    pub layers: Layers,
    pub entities: bool,
    pub grid: bool,
}

impl Default for Visible {
    fn default() -> Self {
        Self {
            layers: Layers::ALL,
            entities: true,
            grid: true,
        }
    }
}

/// Open window over the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dialog {
    New {
        name: String,
        width: usize,
        height: usize,
    },
    Open,
    /// Unsaved changes: where to go afterwards.
    Discard(AfterDiscard),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AfterDiscard {
    New,
    Open(PathBuf),
    Leave,
}

/// Request to the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    /// Back to the main menu.
    Leave,
    /// Playtest the map (M6.9).
    Test,
    /// Test an adventure map in the adventure (A1.8, E-269).
    TestAdventure,
}

/// A map to open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapFile {
    pub path: PathBuf,
    pub name: String,
    /// From the user directory (otherwise bundled; saving creates a custom copy).
    pub own: bool,
}

#[derive(Debug)]
pub struct Editor {
    pub map: Map,
    /// Tool „Abenteuer“ (A1.8).
    pub adventure: adventure::AdventureTool,
    /// Name of the adventure map (file name under `adventure/`, target of transitions, E-271).
    pub adventure_id: String,
    /// World point under the mouse (playtest „an der Maus“).
    pub mouse_world: Option<Vec2>,
    /// Contents of the adventure (reloadable) or the error while loading.
    pub adventure_content: Option<Result<elora_adventure::Content, String>>,
    /// Test state for the playtest and the dialogue test (E-269).
    pub adventure_test: panel_adventure::TestSetup,
    /// Open test window of a dialogue (E-270).
    pub dialog_test: Option<panel_adventure::DialogTest>,
    /// File in the user directory that was last saved to.
    pub file: Option<PathBuf>,
    /// Map point in the center of the map area.
    pub center: Vec2,
    /// World units per screen pixel.
    pub zoom: f32,
    undo: Vec<Map>,
    redo: Vec<Map>,
    /// Kind and time of the last change (for merging).
    last_edit: Option<(String, Instant)>,
    pub dirty: bool,
    /// Selected tile kind of the brush.
    pub brush: Tile,
    pub tool: Tool,
    /// Edge length of the brush in tiles.
    pub brush_size: usize,
    /// Material for newly painted solid tiles.
    pub solid_material: String,
    /// Selectable materials of solid tiles; the first is the default (from `materials.toml`).
    pub solid_materials: Vec<String>,
    /// Selected entity.
    pub entity: elora_map::EntityKind,
    /// Selection (selection tool).
    pub selection: Option<Cells>,
    pub clipboard: Option<Clip>,
    /// Pasting in progress: preview follows the mouse, click places.
    pub pasting: bool,
    /// Start of a rectangle or selection and whether with the left button.
    pub drag: Option<((usize, usize), bool)>,
    /// Target layer for new decoration (M6.8).
    pub decor_layer: look::DecorLayer,
    /// Graphic for new decoration.
    pub decor_art: elora_map::Art,
    pub selected_decor: Option<look::DecorRef>,
    /// Decoration is being dragged: last world point of the mouse.
    pub decor_drag: Option<Vec2>,
    pub selected_bg: Option<usize>,
    pub selected_env: Option<usize>,
    /// Path for embedding an SVG.
    pub image_path: String,
    pub visible: Visible,
    pub dialog: Option<Dialog>,
    /// Last message: language key and value (`{arg}`).
    pub status: Option<(&'static str, String)>,
    /// Current brush stroke (number), ends on release.
    pub stroke: Option<u32>,
    next_stroke: u32,
    /// Inputs for resizing (sidebar).
    pub resize: (usize, usize),
    /// What the app should do after the frame (back to the menu, playtest).
    pub request: Option<Request>,
    /// Folder for custom maps.
    pub user_dir: Option<PathBuf>,
    /// Folder of the bundled maps.
    pub bundled_dir: PathBuf,
}

/// Empty map (air only).
pub fn blank(name: &str, width: usize, height: usize) -> Map {
    let mut m = Map::new(
        name,
        width.clamp(1, elora_map::MAX_SIZE),
        height.clamp(1, elora_map::MAX_SIZE),
    );
    m.author = None;
    m
}

impl Editor {
    pub fn new(user_dir: Option<PathBuf>, bundled_dir: PathBuf) -> Self {
        let map = blank("new", NEW_SIZE.0, NEW_SIZE.1);
        let mut e = Self {
            resize: (map.width, map.height),
            map,
            adventure: adventure::AdventureTool::default(),
            adventure_id: String::new(),
            mouse_world: None,
            adventure_content: None,
            adventure_test: panel_adventure::TestSetup::default(),
            dialog_test: None,
            file: None,
            center: Vec2::ZERO,
            zoom: 1.0,
            undo: Vec::new(),
            redo: Vec::new(),
            last_edit: None,
            dirty: false,
            brush: Tile::Solid,
            tool: Tool::Brush,
            brush_size: 1,
            solid_material: "earth".into(),
            solid_materials: vec!["earth".into()],
            entity: elora_map::EntityKind::Spawn,
            selection: None,
            clipboard: None,
            pasting: false,
            drag: None,
            decor_layer: look::DecorLayer::Front,
            decor_art: elora_map::Art::Builtin("bush-1".into()),
            selected_decor: None,
            decor_drag: None,
            selected_bg: None,
            selected_env: None,
            image_path: String::new(),
            visible: Visible::default(),
            dialog: None,
            status: None,
            stroke: None,
            next_stroke: 0,
            request: None,
            user_dir,
            bundled_dir,
        };
        e.center_view();
        e
    }

    pub fn note(&mut self, key: &'static str, arg: impl Into<String>) {
        self.status = Some((key, arg.into()));
    }

    /// Begin a brush stroke; returns its key for [`Editor::paint`].
    pub fn start_stroke(&mut self) -> u32 {
        self.end_edit();
        self.next_stroke += 1;
        self.stroke = Some(self.next_stroke);
        self.next_stroke
    }

    pub fn finish_stroke(&mut self) {
        if self.stroke.take().is_some() {
            self.end_edit();
        }
    }

    /// View on the map center.
    pub fn center_view(&mut self) {
        let ts = TILE_SIZE as f32;
        #[allow(clippy::cast_precision_loss)]
        {
            self.center = Vec2::new(self.map.width as f32 * ts, self.map.height as f32 * ts) * 0.5;
        }
    }

    /// Call before a change. The same `kind` in quick succession becomes one step.
    pub fn begin_edit(&mut self, kind: &str, now: Instant) {
        let merge = self
            .last_edit
            .as_ref()
            .is_some_and(|(k, t)| k == kind && now.duration_since(*t) < MERGE_WINDOW);
        if !merge {
            self.undo.push(self.map.clone());
            if self.undo.len() > HISTORY {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
        self.last_edit = Some((kind.to_owned(), now));
        self.dirty = true;
    }

    /// Finish the current change (e.g. brush stroke ended): the next one becomes a new step.
    pub fn end_edit(&mut self) {
        self.last_edit = None;
    }

    /// Number of undo steps.
    #[cfg(test)]
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.map, prev));
            self.after_history();
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.map, next));
            self.after_history();
        }
    }

    fn after_history(&mut self) {
        // indices could point into nothing after undo
        self.selected_decor = None;
        self.selected_bg = self.selected_bg.filter(|&i| i < self.map.backgrounds.len());
        self.selected_env = self.selected_env.filter(|&i| i < self.map.envelopes.len());
        self.last_edit = None;
        self.dirty = true;
        self.resize = (self.map.width, self.map.height);
    }

    /// Tile under a world point, if inside the map.
    pub fn tile_at(&self, world: Vec2) -> Option<(usize, usize)> {
        let ts = TILE_SIZE as f32;
        let (x, y) = ((world.x / ts).floor(), (world.y / ts).floor());
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        (x >= 0.0 && y >= 0.0 && x < self.map.width as f32 && y < self.map.height as f32)
            .then_some((x as usize, y as usize))
    }

    /// Resize the map; the content stays at the top left, entities outside are dropped.
    pub fn resize_map(&mut self, width: usize, height: usize, now: Instant) {
        let (w, h) = (
            width.clamp(1, elora_map::MAX_SIZE),
            height.clamp(1, elora_map::MAX_SIZE),
        );
        if (w, h) == (self.map.width, self.map.height) {
            return;
        }
        self.begin_edit("resize", now);
        self.end_edit();
        let old = &self.map;
        let mut tiles = vec![Tile::Air; w * h];
        let mut materials = if old.material_map.is_empty() {
            Vec::new()
        } else {
            vec![0; w * h]
        };
        for y in 0..h.min(old.height) {
            for x in 0..w.min(old.width) {
                tiles[y * w + x] = old.tiles[y * old.width + x];
                if let Some(m) = materials.get_mut(y * w + x) {
                    *m = old.material_map[y * old.width + x];
                }
            }
        }
        self.map.tiles = tiles;
        self.map.material_map = materials;
        self.map.width = w;
        self.map.height = h;
        self.map.entities.retain(|e| e.tx < w && e.ty < h);
        self.resize = (w, h);
    }

    /// Zoom around a screen point (`offset` = distance to the center of the map area in pixels).
    pub fn zoom_at(&mut self, factor: f32, offset: Vec2) {
        let before = self.center + offset * self.zoom;
        self.zoom = (self.zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX);
        let after = self.center + offset * self.zoom;
        self.center += before - after;
    }

    /// New empty map.
    pub fn new_map(&mut self, name: &str, width: usize, height: usize) {
        self.replace_map(blank(name, width, height), None);
        let size = format!("{}×{}", self.map.width, self.map.height);
        self.note("editor.new_done", size);
    }

    fn replace_map(&mut self, map: Map, file: Option<PathBuf>) {
        self.map = map;
        self.adventure.selected = None;
        self.adventure_id.clear();
        self.file = file;
        self.undo.clear();
        self.redo.clear();
        self.last_edit = None;
        self.dirty = false;
        self.resize = (self.map.width, self.map.height);
        self.selection = None;
        self.selected_decor = None;
        self.selected_bg = None;
        self.selected_env = None;
        self.decor_layer = look::DecorLayer::Front;
        self.center_view();
    }

    /// Open a map (also drafts without spawn).
    pub fn open(&mut self, file: &MapFile) {
        match std::fs::read(&file.path)
            .map_err(|e| e.to_string())
            .and_then(|d| elora_map::decode_draft(&d).map_err(|e| e.to_string()))
        {
            Ok(map) => {
                self.note("editor.opened", file.path.display().to_string());
                self.replace_map(map, file.own.then(|| file.path.clone()));
                // adventure maps are named like their file (target of transitions)
                if file
                    .path
                    .parent()
                    .and_then(Path::file_name)
                    .is_some_and(|d| d == "adventure")
                {
                    self.adventure_id = file
                        .path
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                }
            }
            Err(e) => self.note("editor.open_failed", e),
        }
    }

    /// Target file in the user directory (after the map name); adventure maps under
    /// `adventure/` after their name (E-271).
    pub fn target_path(&self) -> Option<PathBuf> {
        let dir = self.user_dir.as_ref()?;
        if self.is_adventure_map() {
            let id = if self.adventure_id.is_empty() {
                elora_client::map_store::safe_name(&self.map.name)
            } else {
                elora_client::map_store::safe_name(&self.adventure_id)
            };
            return Some(
                dir.join("adventure")
                    .join(format!("{id}.{}", elora_map::EXTENSION)),
            );
        }
        Some(dir.join(format!(
            "{}.{}",
            elora_client::map_store::safe_name(&self.map.name),
            elora_map::EXTENSION
        )))
    }

    /// Save to the user directory. Unplayable drafts are saved, but reported.
    pub fn save(&mut self) {
        let Some(path) = self.target_path() else {
            self.note("editor.no_user_dir", "");
            return;
        };
        let result = path
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| self.map.save(&path));
        match result {
            Ok(()) => {
                self.dirty = false;
                self.file = Some(path.clone());
                match elora_map::validate(&self.map) {
                    Ok(()) => self.note("editor.saved", path.display().to_string()),
                    Err(e) => self.note("editor.saved_draft", e.to_string()),
                }
            }
            Err(e) => self.note("editor.save_failed", e.to_string()),
        }
    }

    /// Maps to open: custom ones first, then bundled ones.
    pub fn map_files(&self) -> Vec<MapFile> {
        let list = |dir: &Path, own: bool| -> Vec<MapFile> {
            let mut v: Vec<MapFile> = std::fs::read_dir(dir)
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == elora_map::EXTENSION))
                .map(|path| MapFile {
                    name: path
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                    path,
                    own,
                })
                .collect();
            v.sort_by(|a, b| a.name.cmp(&b.name));
            v
        };
        let mut all = self
            .user_dir
            .as_deref()
            .map(|d| list(d, true))
            .unwrap_or_default();
        all.extend(list(&self.bundled_dir, false));
        // adventure maps (E-262, E-271)
        let adventure = |dir: &Path, own: bool| {
            list(&dir.join("adventure"), own).into_iter().map(|mut f| {
                f.name = format!("adventure/{}", f.name);
                f
            })
        };
        if let Some(d) = self.user_dir.as_deref() {
            all.extend(adventure(d, true));
        }
        all.extend(adventure(&self.bundled_dir, false));
        all
    }

    /// Request a playtest; an unplayable map (e.g. without spawn) is reported.
    pub fn request_test(&mut self) {
        if self.is_adventure_map() {
            match elora_map::validate(&self.map) {
                Ok(()) => self.request = Some(Request::TestAdventure),
                Err(e) => self.note("editor.test_unplayable", e.to_string()),
            }
            return;
        }
        match elora_map::validate(&self.map) {
            Ok(()) => self.request = Some(Request::Test),
            Err(e) => self.note("editor.test_unplayable", e.to_string()),
        }
    }

    /// Action that would discard unsaved changes: ask first.
    pub fn request(&mut self, after: AfterDiscard) {
        if self.dirty {
            self.dialog = Some(Dialog::Discard(after));
        } else {
            self.proceed(after);
        }
    }

    /// Execute after confirmation (or without changes).
    pub fn proceed(&mut self, after: AfterDiscard) {
        self.dialog = None;
        match after {
            AfterDiscard::New => {
                self.dialog = Some(Dialog::New {
                    name: "new".into(),
                    width: NEW_SIZE.0,
                    height: NEW_SIZE.1,
                });
            }
            AfterDiscard::Open(path) => {
                let own = self.user_dir.as_ref().is_some_and(|d| path.starts_with(d));
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.open(&MapFile { path, name, own });
            }
            AfterDiscard::Leave => self.request = Some(Request::Leave),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor() -> Editor {
        Editor::new(None, PathBuf::from("maps"))
    }

    #[test]
    fn strokes_are_single_undo_steps() {
        let mut e = editor();
        let t = Instant::now();
        e.fill_cells(Cells::span((1, 1), (1, 1)), Tile::Solid, "line-1", t);
        e.fill_cells(Cells::span((2, 1), (2, 1)), Tile::Solid, "line-1", t);
        e.end_edit();
        e.fill_cells(Cells::span((3, 1), (3, 1)), Tile::Ice, "line-2", t);
        assert!(e.dirty);
        e.undo();
        assert_eq!(e.map.tiles[e.map.width + 3], Tile::Air);
        assert_eq!(
            e.map.tiles[e.map.width + 2],
            Tile::Solid,
            "first stroke stays"
        );
        e.undo();
        assert_eq!(e.map.tiles[e.map.width + 1], Tile::Air);
        assert!(!e.can_undo());
        e.redo();
        e.redo();
        assert_eq!(e.map.tiles[e.map.width + 3], Tile::Ice);
        assert!(!e.can_redo());
        // a new change clears redo
        e.undo();
        e.fill_cells(Cells::span((5, 5), (5, 5)), Tile::Death, "line-3", t);
        assert!(!e.can_redo());
    }

    #[test]
    fn same_kind_merges_only_within_window() {
        let mut e = editor();
        let t = Instant::now();
        e.begin_edit("name", t);
        e.map.name = "a".into();
        e.begin_edit("name", t + Duration::from_millis(100));
        e.map.name = "ab".into();
        e.begin_edit("name", t + Duration::from_secs(5));
        e.map.name = "abc".into();
        e.undo();
        assert_eq!(e.map.name, "ab");
        e.undo();
        assert_eq!(e.map.name, "new");
    }

    #[test]
    fn resize_keeps_top_left_and_drops_outside_entities() {
        let mut e = editor();
        let t = Instant::now();
        e.fill_cells(Cells::span((2, 2), (2, 2)), Tile::Solid, "s", t);
        e.map.entities.push(elora_map::Entity {
            kind: elora_map::EntityKind::Spawn,
            tx: 50,
            ty: 2,
        });
        e.resize_map(10, 5, t);
        assert_eq!((e.map.width, e.map.height), (10, 5));
        assert_eq!(e.map.tiles[2 * 10 + 2], Tile::Solid);
        assert!(e.map.entities.is_empty());
        e.undo();
        assert_eq!((e.map.width, e.map.height), NEW_SIZE);
        assert_eq!(e.map.entities.len(), 1);
    }

    #[test]
    fn tile_lookup_and_zoom_keep_point_under_cursor() {
        let mut e = editor();
        assert_eq!(e.tile_at(Vec2::new(40.0, 70.0)), Some((1, 2)));
        assert_eq!(e.tile_at(Vec2::new(-1.0, 5.0)), None);
        assert_eq!(e.tile_at(Vec2::new(32.0 * 60.0, 5.0)), None);
        let offset = Vec2::new(100.0, -50.0);
        let before = e.center + offset * e.zoom;
        e.zoom_at(2.0, offset);
        assert!((e.center + offset * e.zoom - before).length() < 1e-3);
        e.zoom_at(1000.0, offset);
        assert!((e.zoom - ZOOM_MAX).abs() < 1e-6);
    }

    #[test]
    fn save_and_open_roundtrip_including_drafts() {
        let dir = std::env::temp_dir().join(format!("elora-editor-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut e = Editor::new(Some(dir.clone()), dir.join("bundled"));
        e.map.name = "Meine Karte!".into();
        e.fill_cells(
            Cells::span((0, 0), (0, 0)),
            Tile::Solid,
            "s",
            Instant::now(),
        );
        e.save();
        assert!(!e.dirty);
        assert_eq!(e.status.as_ref().map(|s| s.0), Some("editor.saved_draft"));
        let files = e.map_files();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "Meine_Karte_");
        let mut f = Editor::new(Some(dir.clone()), dir.join("bundled"));
        f.open(&files[0]);
        assert_eq!(f.map, e.map);
        assert_eq!(f.file.as_ref(), Some(&files[0].path));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_play_needs_a_playable_map() {
        let mut e = editor();
        e.request_test();
        assert_eq!(e.request, None);
        assert_eq!(
            e.status.as_ref().map(|s| s.0),
            Some("editor.test_unplayable")
        );
        e.map.entities.push(elora_map::Entity {
            kind: elora_map::EntityKind::Spawn,
            tx: 1,
            ty: 1,
        });
        e.request_test();
        assert_eq!(e.request, Some(Request::Test));
    }

    #[test]
    fn unsaved_changes_ask_before_leaving() {
        let mut e = editor();
        e.request(AfterDiscard::Leave);
        assert_eq!(
            e.request,
            Some(Request::Leave),
            "immediately without changes"
        );
        let mut e = editor();
        e.fill_cells(
            Cells::span((0, 0), (0, 0)),
            Tile::Solid,
            "s",
            Instant::now(),
        );
        e.request(AfterDiscard::Leave);
        assert_eq!(e.request, None);
        assert_eq!(e.dialog, Some(Dialog::Discard(AfterDiscard::Leave)));
        e.proceed(AfterDiscard::Leave);
        assert_eq!(e.request, Some(Request::Leave));
    }
}

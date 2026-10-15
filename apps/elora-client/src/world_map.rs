//! World map of the Tauland (R2-WM, E-370 to E-374): a painted overview with the map sections,
//! Elora on hers, clouds over undiscovered parts and markers; a detail window of the current
//! section drawn from its tiles; a legend with the current quest step.

use std::path::Path;

use elora_adventure::data::{Area, Section};
use elora_adventure::world_map::QuestGoal;
use elora_adventure::{Content, MapSpot, SaveGame};
use elora_map::{Map, ObjectKind};
use elora_render::{Affine, Align, Color, Mesh, SvgAsset, Tint};
use elora_sim::{TILE_SIZE, Tile, Vec2};

use crate::figure::FigureArt;
use crate::lang::Lang;
use crate::ui::{self, Rect, Ui};

/// Size of the overview space (`worldmap.toml` 0..1 × this, units of the ground graphic).
const SPACE: Vec2 = Vec2::new(1480.0, 800.0);
const INK: Color = Color::hex(0x6b5a43);
const ROAD: Color = Color::hex(0xa8946e);
const GLOW: Color = Color::rgba(1.0, 0.95, 0.69, 0.8);
const LABEL: Color = Color::hex(0x4a3b28);
const PARCHMENT: Color = Color::hex(0xf6eedb);
const UNKNOWN: Color = Color::hex(0x7f93a3);

/// The graphics of the world map: painted ground and markers.
#[derive(Debug)]
pub struct WorldMapArt {
    ground: Mesh,
    icons: Vec<(String, Mesh)>,
}

impl WorldMapArt {
    /// # Panics
    /// If an embedded asset is faulty (covered by tests).
    pub fn load() -> Self {
        let ground = SvgAsset::load(
            include_bytes!("../../../assets/adventure/world-map.svg"),
            0.5,
        )
        .expect("assets/adventure/world-map.svg")
        .part("")
        .cloned()
        .expect("world map ground");
        let icons = SvgAsset::load(
            include_bytes!("../../../assets/adventure/world-map-icons.svg"),
            0.1,
        )
        .expect("assets/adventure/world-map-icons.svg")
        .parts
        .into_iter()
        .filter(|(id, _)| !id.is_empty())
        .collect();
        Self { ground, icons }
    }

    fn icon(&self, name: &str) -> Option<&Mesh> {
        self.icons.iter().find(|(n, _)| n == name).map(|(_, m)| m)
    }
}

/// All adventure maps, loaded once (markers and quest goals of the world map).
#[derive(Debug, Default)]
pub struct Atlas {
    /// Map id and map.
    pub maps: Vec<(String, Map)>,
}

impl Atlas {
    /// Loads every map of `maps/adventure/` with `load` (which prefers the player's own copy).
    pub fn load(load: impl Fn(&str) -> anyhow::Result<Map>) -> Self {
        let dir = elora_server::paths::resolve(Path::new("maps/adventure"));
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter_map(|e| {
                let p = e.path();
                if p.extension()? != elora_map::EXTENSION {
                    return None;
                }
                Some(p.file_stem()?.to_string_lossy().into_owned())
            })
            .collect();
        names.sort();
        Self {
            maps: names
                .into_iter()
                .filter_map(|n| load(&n).ok().map(|m| (n, m)))
                .collect(),
        }
    }

    fn map(&self, name: &str) -> Option<&Map> {
        self.maps.iter().find(|(n, _)| n == name).map(|(_, m)| m)
    }

    /// The maps as references (for [`elora_adventure::world_map::quest_goal`]).
    pub fn refs(&self) -> Vec<(&str, &Map)> {
        self.maps.iter().map(|(n, m)| (n.as_str(), m)).collect()
    }
}

/// Everything the world map shows.
pub struct View<'a> {
    pub art: &'a WorldMapArt,
    pub atlas: &'a Atlas,
    pub content: &'a Content,
    pub save: &'a SaveGame,
    pub lang: &'a Lang,
    pub code: &'a str,
    pub figure: &'a FigureArt,
    pub tint: &'a Tint,
    /// Elora's place on the overview.
    pub spot: Option<MapSpot>,
    /// Current main quest goal.
    pub goal: Option<QuestGoal>,
    /// The current map (detail window) and Elora's position in it.
    pub here: Option<(&'a str, &'a Map, Vec2)>,
    /// Running time (s) for the glow.
    pub time: f32,
}

impl View<'_> {
    fn visited(&self, map: &str) -> bool {
        self.save.flag(&format!("visited:{map}")) != 0
    }

    fn known(&self, a: &Area) -> bool {
        a.id == self.content.areas[0].id
            || self
                .save
                .flags
                .keys()
                .filter_map(|k| k.strip_prefix("visited:"))
                .any(|m| m.starts_with(&a.maps))
    }

    /// Point on the overview for `pos` on map `map` (its section; otherwise the area point).
    fn place(&self, map: &str, pos: Option<Vec2>) -> Option<[f32; 2]> {
        let area = self.content.area_of(map)?;
        let Some(section) = area.section(map) else {
            return Some(area.pos);
        };
        let along = match (pos, self.atlas.map(map)) {
            (Some(p), Some(m)) => {
                let ts = TILE_SIZE as f32;
                match section.axis {
                    elora_adventure::data::Axis::X => p.x / (m.width as f32 * ts),
                    elora_adventure::data::Axis::Y => 1.0 - p.y / (m.height as f32 * ts),
                }
            }
            _ => 0.5,
        };
        Some(section.point(along))
    }
}

/// Full layout: overview on the left, detail window and legend on the right.
pub fn draw(ui: &mut Ui<'_>, area: Rect, view: &View<'_>) {
    let s = ui.s;
    let gap = 14.0 * s;
    let side = (area.w() * 0.3).clamp(200.0 * s, 360.0 * s);
    let room = Rect::new(area.min.x, area.min.y, area.w() - side - gap, area.h());
    // the overview keeps its proportions
    let k = (room.w() / SPACE.x).min(room.h() / SPACE.y);
    let ov = Rect::new(
        room.min.x + (room.w() - SPACE.x * k) / 2.0,
        room.min.y + (room.h() - SPACE.y * k) / 2.0,
        SPACE.x * k,
        SPACE.y * k,
    );
    overview(ui, ov, view);
    let x = area.max.x - side;
    let detail_h = (side * 0.62).min(area.h() * 0.5);
    detail(ui, Rect::new(x, area.min.y, side, detail_h), view);
    legend(
        ui,
        Rect::new(
            x,
            area.min.y + detail_h + gap,
            side,
            area.h() - detail_h - gap,
        ),
        view,
    );
}

/// Drawing on the overview: its frame, the scale of the painted ground and of the markers
/// (they stay readable when the overview is small).
struct Sheet {
    frame: Rect,
    zoom: f32,
    marker: f32,
}

impl Sheet {
    fn new(frame: Rect, ui_scale: f32) -> Self {
        let zoom = frame.w() / SPACE.x;
        Self {
            frame,
            zoom,
            marker: zoom.max(0.62 * ui_scale),
        }
    }

    /// Screen point of an overview point (0..1).
    fn at(&self, p: [f32; 2]) -> Vec2 {
        Vec2::new(
            self.frame.min.x + p[0] * self.frame.w(),
            self.frame.min.y + p[1] * self.frame.h(),
        )
    }

    /// A marker; clouds scale with the ground, the others with the marker scale.
    fn icon(
        &self,
        ui: &mut Ui<'_>,
        art: &WorldMapArt,
        name: &str,
        pos: Vec2,
        size: f32,
        alpha: f32,
    ) {
        let Some(mesh) = art.icon(name) else { return };
        let size = size
            * if name == "cloud" {
                self.zoom
            } else {
                self.marker
            };
        let tint = Tint {
            alpha: (alpha < 1.0).then_some(alpha),
            ..Tint::default()
        };
        ui.batch.draw_mesh(
            mesh,
            &Affine::translate(pos).then(Affine::scale(size, size)),
            &tint,
        );
    }
}

/// Painted overview with sections, clouds, markers, names and Elora.
pub fn overview(ui: &mut Ui<'_>, frame: Rect, view: &View<'_>) {
    let sheet = Sheet::new(frame, ui.s);
    ui.batch.draw_mesh(
        &view.art.ground,
        &Affine::translate(frame.min).then(Affine::scale(sheet.zoom, sheet.zoom)),
        &Tint::default(),
    );
    paths(ui, &sheet, view);
    area_markers(ui, &sheet, view);
    rest_stones(ui, &sheet, view);
    people(ui, &sheet, view);
    quest_star(ui, &sheet, view);
    clouds(ui, &sheet, view);
    names(ui, &sheet, view);
    elora(ui, &sheet, view);
    outline(ui, frame, 3.0 * ui.s, INK);
}

/// The path pieces: visited ones in the area colour (the current one glowing), others dotted.
fn paths(ui: &mut Ui<'_>, sheet: &Sheet, view: &View<'_>) {
    let zoom = sheet.zoom;
    for area in view.content.areas.iter().skip(1) {
        let color = crate::adventure_menu::hex(&area.color);
        for sec in &area.sections {
            let pts = curve(sec, sheet);
            let Some(&end) = pts.last().filter(|_| pts.len() >= 2) else {
                continue;
            };
            if view.visited(&sec.map) {
                if view.spot.as_ref().is_some_and(|spot| spot.map == sec.map) {
                    let mut glow = GLOW;
                    glow.0[3] = 0.65 + 0.2 * (view.time * 3.0).sin();
                    ui.batch.stroke_polyline(&pts, 20.0 * zoom, glow);
                }
                ui.batch.stroke_polyline(&pts, 11.0 * zoom, INK);
                ui.batch
                    .stroke_polyline(&pts, 7.0 * zoom, elora_render::shade(color, -0.1));
            } else {
                dotted(ui, &pts, 4.0 * zoom, (4.0 * zoom, 7.0 * zoom), ROAD);
            }
            ui.batch.fill_circle(end, 7.0 * zoom, INK);
            ui.batch.fill_circle(end, 5.0 * zoom, ui::CARD);
        }
    }
}

/// Guardians at the arenas, springs of the known areas.
fn area_markers(ui: &mut Ui<'_>, sheet: &Sheet, view: &View<'_>) {
    for area in view.content.areas.iter().skip(1) {
        if let Some(arena) = area
            .sections
            .iter()
            .find(|sec| sec.path.len() == 1 && sec.map.ends_with("arena"))
        {
            let calm = area
                .guardian
                .as_ref()
                .is_some_and(|g| view.save.flag(&format!("defeated.{g}")) != 0);
            let name = if calm { "guardian_calm" } else { "guardian" };
            sheet.icon(ui, view.art, name, sheet.at(arena.path[0]), 1.1, 1.0);
        }
        if view.known(area) && area.spring.is_some() {
            let name = if area.freed(view.save) {
                "spring_freed"
            } else {
                "spring_silent"
            };
            let pos = sheet.at(area.pos) + Vec2::new(60.0, -46.0) * sheet.zoom;
            sheet.icon(ui, view.art, name, pos, 1.0, 1.0);
        }
    }
}

/// Spring stones Elora rested at.
fn rest_stones(ui: &mut Ui<'_>, sheet: &Sheet, view: &View<'_>) {
    for (name, map) in &view.atlas.maps {
        for obj in &map.adventure.objects {
            if matches!(obj.kind, ObjectKind::SavePoint)
                && view.save.flag(&format!("rested:{name}:{}", obj.id)) != 0
                && let Some(point) = view.place(name, Some(obj.pos))
            {
                let pos = sheet.at(point) + Vec2::new(0.0, -16.0) * sheet.zoom;
                sheet.icon(ui, view.art, "spring_stone", pos, 0.9, 1.0);
            }
        }
    }
}

/// Shops in Tauwinkel and one character per visited section of the other areas.
fn people(ui: &mut Ui<'_>, sheet: &Sheet, view: &View<'_>) {
    let hub = &view.content.areas[0];
    let owners: Vec<&str> = view
        .content
        .shops
        .values()
        .filter_map(|shop| shop.owner.as_deref())
        .collect();
    let mut shops = 0.0;
    for (name, map) in &view.atlas.maps {
        let in_hub = view.content.area_of(name).is_some_and(|a| a.id == hub.id);
        let mut person = None;
        for obj in &map.adventure.objects {
            let ObjectKind::Npc { character, .. } = &obj.kind else {
                continue;
            };
            if crate::creatures::CreatureArt::is_still(character) {
                continue;
            }
            if in_hub && owners.contains(&character.as_str()) {
                let offset = Vec2::new(50.0 + 24.0 * shops, -34.0 + 22.0 * shops);
                let pos = sheet.at(hub.pos) + offset * sheet.zoom;
                sheet.icon(ui, view.art, "shop", pos, 0.9, 1.0);
                shops += 1.0;
            } else if !in_hub && person.is_none() {
                person = Some(obj.pos);
            }
        }
        if let Some(found) = person
            && view.visited(name)
            && let Some(point) = view.place(name, Some(found))
        {
            let pos = sheet.at(point) + Vec2::new(18.0, -22.0) * sheet.zoom;
            sheet.icon(ui, view.art, "person", pos, 0.8, 1.0);
        }
    }
}

/// The goal of the current main quest step.
fn quest_star(ui: &mut Ui<'_>, sheet: &Sheet, view: &View<'_>) {
    if let Some(goal) = &view.goal
        && let Some(map) = &goal.map
        && let Some(point) = view.place(map, goal.pos)
    {
        let bob = (view.time * 2.5).sin() * 3.0 * sheet.zoom;
        let pos = sheet.at(point) + Vec2::new(4.0, -6.0) * sheet.zoom + Vec2::new(0.0, bob);
        sheet.icon(ui, view.art, "quest", pos, 1.2, 1.0);
    }
}

/// Clouds: thick over unknown areas, thin over unvisited sections of known ones.
fn clouds(ui: &mut Ui<'_>, sheet: &Sheet, view: &View<'_>) {
    for area in view.content.areas.iter().skip(1) {
        if view.known(area) {
            for sec in area.sections.iter().filter(|sec| !view.visited(&sec.map)) {
                sheet.icon(ui, view.art, "cloud", sheet.at(sec.point(1.0)), 0.75, 0.55);
            }
        } else {
            let centre = sheet.at(area.pos);
            for (dx, dy, size) in CLOUDS {
                let pos = centre + Vec2::new(dx, dy) * sheet.zoom;
                sheet.icon(ui, view.art, "cloud", pos, size, 0.95);
            }
        }
    }
}

/// Names of the known areas (with a light rim), `?` for the others.
fn names(ui: &mut Ui<'_>, sheet: &Sheet, view: &View<'_>) {
    let ui_scale = ui.s;
    let size = 13.0 * (sheet.frame.w() / (900.0 * ui_scale)).clamp(0.7, 1.4);
    for (i, area) in view.content.areas.iter().enumerate() {
        let centre = sheet.at(area.pos);
        if view.known(area) {
            let below = if i == 0 { 56.0 } else { 112.0 };
            let pos = centre + Vec2::new(0.0, below * sheet.zoom);
            let name = area.name.get(view.code);
            for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
                let rim = pos + Vec2::new(dx, dy) * 1.5 * ui_scale;
                ui.label(name, rim, size, PARCHMENT, Align::Center);
            }
            ui.label(name, pos, size, LABEL, Align::Center);
        } else {
            let pos = centre - Vec2::new(0.0, 14.0 * sheet.zoom);
            ui.label("?", pos, 22.0, UNKNOWN, Align::Center);
        }
    }
}

/// Elora's pin on her section.
fn elora(ui: &mut Ui<'_>, sheet: &Sheet, view: &View<'_>) {
    let Some(spot) = &view.spot else { return };
    let pos = sheet.at(spot.point);
    let size = sheet.marker * 1.3;
    sheet.icon(ui, view.art, "pin", pos, 1.3, 1.0);
    view.figure.draw_pose(
        ui.batch,
        pos - Vec2::new(0.0, 24.0 * size),
        34.0 * size,
        1.0,
        view.tint,
    );
}

/// A rectangle's outline.
fn outline(ui: &mut Ui<'_>, frame: Rect, width: f32, color: Color) {
    ui.batch.stroke_polyline(
        &[
            frame.min,
            Vec2::new(frame.max.x, frame.min.y),
            frame.max,
            Vec2::new(frame.min.x, frame.max.y),
            frame.min,
        ],
        width,
        color,
    );
}

/// Cloud cluster over an unknown area (offset in overview units, size).
const CLOUDS: [(f32, f32, f32); 8] = [
    (-100.0, -30.0, 1.5),
    (-20.0, -50.0, 1.4),
    (70.0, -30.0, 1.5),
    (0.0, 30.0, 1.7),
    (100.0, 30.0, 1.3),
    (-100.0, 45.0, 1.2),
    (130.0, -10.0, 1.0),
    (-140.0, 5.0, 1.0),
];

/// The section as screen points (a curve through its points).
fn curve(sec: &Section, sheet: &Sheet) -> Vec<Vec2> {
    match sec.path.len() {
        0 | 1 => Vec::new(),
        3 => {
            // quadratic curve through the middle point
            let [start, middle, end] = [sec.path[0], sec.path[1], sec.path[2]].map(|p| sheet.at(p));
            let control = middle * 2.0 - (start + end) * 0.5;
            (0..=16)
                .map(|i| {
                    let t = i as f32 / 16.0;
                    start * ((1.0 - t) * (1.0 - t))
                        + control * (2.0 * (1.0 - t) * t)
                        + end * (t * t)
                })
                .collect()
        }
        _ => sec.path.iter().map(|p| sheet.at(*p)).collect(),
    }
}

/// A polyline as dashes (`pattern` = dash and gap length).
fn dotted(ui: &mut Ui<'_>, pts: &[Vec2], width: f32, pattern: (f32, f32), color: Color) {
    let (dash, gap) = pattern;
    let mut left = 0.0;
    for seg in pts.windows(2) {
        let (from, to) = (seg[0], seg[1]);
        let len = from.distance(to);
        let mut done = left;
        while done < len {
            let a = from.lerp(to, done / len);
            let b = from.lerp(to, ((done + dash) / len).min(1.0));
            ui.batch.stroke_line(a, b, width, color);
            done += dash + gap;
        }
        left = done - len;
    }
}

/// Detail window: the current map drawn small from its tiles, Elora, exits, spring stones,
/// characters and the quest goal.
pub fn detail(ui: &mut Ui<'_>, frame: Rect, view: &View<'_>) {
    let ui_scale = ui.s;
    ui.batch
        .fill_rounded_rect(frame.min, frame.max, 12.0 * ui_scale, Color::hex(0xe9f3e2));
    let Some((name, map, elora_pos)) = view.here else {
        return;
    };
    ui.label(
        &section_title(view, name),
        frame.min + Vec2::new(12.0, 10.0) * ui_scale,
        11.0,
        ui::TEXT,
        Align::Left,
    );
    let inner = Rect::new(
        frame.min.x + 10.0 * ui_scale,
        frame.min.y + 32.0 * ui_scale,
        frame.w() - 20.0 * ui_scale,
        frame.h() - 42.0 * ui_scale,
    );
    let tile = TILE_SIZE as f32;
    let world = Vec2::new(map.width as f32 * tile, map.height as f32 * tile);
    let zoom = (inner.w() / world.x).min(inner.h() / world.y);
    let origin = inner.center() - world * (zoom * 0.5);
    let to = |p: Vec2| origin + p * zoom;
    ui.batch
        .fill_rect(to(Vec2::ZERO), to(world), Color::hex(0xcfe6f2));
    tiles(ui, map, &to, tile * zoom);
    objects(ui, view, map, &to, inner.center().x);
    if let Some(goal) = &view.goal
        && goal.map.as_deref() == Some(name)
        && let Some(pos) = goal.pos
    {
        let bob = (view.time * 2.5).sin() * 2.0 * ui_scale;
        detail_icon(
            ui,
            view,
            "quest",
            to(pos) + Vec2::new(0.0, bob - 10.0 * ui_scale),
            0.8,
        );
    }
    let dot = to(elora_pos);
    ui.batch
        .fill_circle(dot, 10.0 * ui_scale, Color::rgba(1.0, 0.95, 0.69, 0.7));
    ui.batch.fill_circle(dot, 7.0 * ui_scale, ui::OUTLINE);
    ui.batch
        .fill_circle(dot, 5.0 * ui_scale, Color::hex(0xf2c14e));
    outline(ui, frame, 2.5 * ui_scale, ui::OUTLINE);
}

/// “Area · section n” (or only the area for arenas and the village).
fn section_title(view: &View<'_>, name: &str) -> String {
    let Some(area) = view.content.area_of(name) else {
        return name.to_owned();
    };
    let area_name = area.name.get(view.code);
    match area.sections.iter().position(|sec| sec.map == name) {
        Some(i) if !name.ends_with("arena") && area.sections.len() > 1 => view.lang.f(
            "adventure.map_section",
            &[("area", &area_name), ("n", &(i + 1))],
        ),
        _ => area_name.to_owned(),
    }
}

/// The tiles of the map in rows of equal kind.
fn tiles(ui: &mut Ui<'_>, map: &Map, to: &dyn Fn(Vec2) -> Vec2, cell: f32) {
    let tile = TILE_SIZE as f32;
    for row in 0..map.height {
        let mut col = 0;
        while col < map.width {
            let kind = map.tiles[row * map.width + col];
            let mut end = col + 1;
            while end < map.width && map.tiles[row * map.width + end] == kind {
                end += 1;
            }
            if let Some(color) = tile_color(kind) {
                let min = to(Vec2::new(col as f32 * tile, row as f32 * tile));
                let height = if kind == Tile::Platform {
                    cell * 0.35
                } else {
                    cell
                };
                ui.batch.fill_rect(
                    min,
                    min + Vec2::new((end - col) as f32 * cell, height.max(1.0)),
                    color,
                );
            }
            col = end;
        }
    }
}

/// Spring stones, characters and exits in the detail window.
fn objects(ui: &mut Ui<'_>, view: &View<'_>, map: &Map, to: &dyn Fn(Vec2) -> Vec2, middle: f32) {
    let ui_scale = ui.s;
    for obj in &map.adventure.objects {
        let pos = to(obj.pos);
        match &obj.kind {
            ObjectKind::SavePoint => detail_icon(ui, view, "spring_stone", pos, 0.7),
            ObjectKind::Npc { character, .. }
                if !crate::creatures::CreatureArt::is_still(character) =>
            {
                detail_icon(
                    ui,
                    view,
                    "person",
                    pos - Vec2::new(0.0, 12.0 * ui_scale),
                    0.55,
                );
            }
            ObjectKind::Exit { size, .. } => {
                let centre = to(obj.pos + *size * 0.5);
                let dir = if centre.x < middle { -1.0 } else { 1.0 };
                exit_arrow(ui, centre + Vec2::new(dir * 8.0 * ui_scale, 0.0), dir);
            }
            _ => {}
        }
    }
}

/// Arrow pointing out of the map at `tip` (`dir` −1 left, 1 right).
fn exit_arrow(ui: &mut Ui<'_>, tip: Vec2, dir: f32) {
    let ui_scale = ui.s;
    let tri = |reach: f32, half: f32, from: f32| {
        [
            tip + Vec2::new(dir * from * ui_scale, 0.0),
            tip - Vec2::new(dir * reach * ui_scale, half * ui_scale),
            tip - Vec2::new(dir * reach * ui_scale, -half * ui_scale),
        ]
    };
    ui.batch.fill_polygon(&tri(14.0, 10.0, 2.0), ui::OUTLINE);
    ui.batch.fill_polygon(&tri(11.0, 7.0, 0.0), ui::CARD);
}

fn detail_icon(ui: &mut Ui<'_>, view: &View<'_>, name: &str, pos: Vec2, size: f32) {
    if let Some(mesh) = view.art.icon(name) {
        let size = size * ui.s;
        ui.batch.draw_mesh(
            mesh,
            &Affine::translate(pos).then(Affine::scale(size, size)),
            &Tint::default(),
        );
    }
}

fn tile_color(t: Tile) -> Option<Color> {
    Some(match t {
        Tile::Air | Tile::HookPoint => return None,
        Tile::Platform => Color::hex(0xa87a52),
        Tile::Death => Color::hex(0xc94a4a),
        Tile::Ice | Tile::ThinIce => Color::hex(0xbfe4f4),
        Tile::Quicksand => Color::hex(0xe2bf7c),
        Tile::IceWater => Color::hex(0x5fa8d8),
        _ => Color::hex(0x6aae4a),
    })
}

/// Legend of the markers and the current quest step.
pub fn legend(ui: &mut Ui<'_>, frame: Rect, view: &View<'_>) {
    let ui_scale = ui.s;
    ui.batch
        .fill_rounded_rect(frame.min, frame.max, 12.0 * ui_scale, ui::CARD);
    let left = frame.min.x + 12.0 * ui_scale;
    let mut y = frame.min.y + 12.0 * ui_scale;
    if let Some(goal) = &view.goal {
        let title = view.lang.t("adventure.map_goal");
        ui.label(title, Vec2::new(left, y), 10.0, ui::TEXT_DIM, Align::Left);
        y += 16.0 * ui_scale;
        let text = goal.text.get(view.code);
        ui.label(text, Vec2::new(left, y), 11.0, ui::TEXT, Align::Left);
        y += 26.0 * ui_scale;
    }
    let rows = [
        ("spring_stone", "adventure.map_legend_stone"),
        ("quest", "adventure.map_legend_goal"),
        ("spring_freed", "adventure.map_legend_spring"),
        ("guardian", "adventure.map_legend_guardian"),
        ("shop", "adventure.map_legend_shop"),
        ("person", "adventure.map_legend_person"),
    ];
    let step = ((frame.max.y - y - 8.0 * ui_scale) / rows.len() as f32)
        .clamp(14.0 * ui_scale, 26.0 * ui_scale);
    for (icon, key) in rows {
        if y + step > frame.max.y {
            break;
        }
        let pos = Vec2::new(frame.min.x + 22.0 * ui_scale, y + step * 0.5);
        detail_icon(ui, view, icon, pos, 0.6);
        let text_pos = Vec2::new(
            frame.min.x + 40.0 * ui_scale,
            y + step * 0.5 - 7.0 * ui_scale,
        );
        ui.label(view.lang.t(key), text_pos, 10.0, ui::TEXT, Align::Left);
        y += step;
    }
}

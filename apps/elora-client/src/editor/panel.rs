//! Editor-Oberfläche (egui dunkel, E-150): Seitenleiste rechts (E-151), Kartenfläche mit
//! Verschieben, Zoomen und Pinsel, Dialoge für Neu, Öffnen und ungespeicherte Änderungen.

use std::time::Instant;

use egui::{Key, KeyboardShortcut, Modifiers, PointerButton};
use elora_map::Rgba;
use elora_sim::{BeltDir, JumpDir, Tile, Vec2};

use super::tools::{Cells, ENTITIES, Tool};
use super::{AfterDiscard, Dialog, Editor};
use crate::lang::Lang;
use crate::map_view::MapView;

use super::panel_look;

/// Höchstlänge von Name und Autor in Zeichen (die Datei erlaubt 128 Bytes, UTF-8 bis 4 Bytes je Zeichen).
const NAME_CHARS: usize = 32;
/// Größter Pinsel (Tiles).
const MAX_BRUSH: usize = 9;
/// Breite der Seitenleiste (Punkte).
const PANEL_WIDTH: f32 = 290.0;

/// Tile-Arten des Pinsels mit Sprachschlüssel und Farbe der Vorschau.
pub const BRUSHES: [(Tile, &str, u32); 17] = [
    (Tile::Air, "editor.tile_air", 0x3a3f47),
    (Tile::Solid, "editor.tile_solid", 0xa87a52),
    (Tile::Unhookable, "editor.tile_unhookable", 0x566068),
    (Tile::Death, "editor.tile_death", 0xc94f4f),
    (Tile::Platform, "editor.tile_platform", 0xc9955c),
    (Tile::Ice, "editor.tile_ice", 0xbfe6f5),
    (Tile::JumpPad(JumpDir::Up), "editor.tile_jump_up", 0xef7fb0),
    (
        Tile::JumpPad(JumpDir::UpLeft),
        "editor.tile_jump_left",
        0xef7fb0,
    ),
    (
        Tile::JumpPad(JumpDir::UpRight),
        "editor.tile_jump_right",
        0xef7fb0,
    ),
    (
        Tile::Conveyor(BeltDir::Left),
        "editor.tile_conveyor_left",
        0x6c7a89,
    ),
    (
        Tile::Conveyor(BeltDir::Right),
        "editor.tile_conveyor_right",
        0x6c7a89,
    ),
    (Tile::Climb, "editor.tile_climb", 0x5f8a9a),
    (Tile::Crumble, "editor.tile_crumble", 0xc49a68),
    (Tile::HookPoint, "editor.tile_hook_point", 0xef7fb0),
    (Tile::Quicksand, "editor.tile_quicksand", 0xe2bf7c),
    (Tile::ThinIce, "editor.tile_thin_ice", 0xc7ebf8),
    (Tile::IceWater, "editor.tile_ice_water", 0x2f6f9a),
];

/// Ergebnis eines Frames für die Ansicht.
#[derive(Debug, Clone, Copy, Default)]
pub struct AreaInfo {
    /// Mitte der Kartenfläche in Pixeln.
    pub center_px: Vec2,
    /// Tile unter der Maus.
    pub hover: Option<(usize, usize)>,
    /// Was das Werkzeug unter der Maus zeigt.
    pub preview: Preview,
}

/// Vorschau des Werkzeugs in der Kartenansicht.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Preview {
    #[default]
    None,
    /// Felder, die ein Klick ändern würde (Pinsel, Rechteck beim Ziehen, Füllen-Startfeld).
    Cells(Cells),
    /// Einzufügender Ausschnitt mit der linken oberen Ecke hier.
    Stamp(usize, usize),
}

fn color32(c: Rgba) -> egui::Color32 {
    let [red, green, blue, alpha] = c.0;
    egui::Color32::from_rgba_unmultiplied(red, green, blue, alpha)
}

fn rgba(c: egui::Color32) -> Rgba {
    Rgba(c.to_srgba_unmultiplied())
}

fn swatch(ui: &mut egui::Ui, color: u32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
    let [_, r, g, b] = color.to_be_bytes();
    ui.painter()
        .rect_filled(rect, 3.0, egui::Color32::from_rgb(r, g, b));
}

/// Baut die ganze Editor-Oberfläche. `window` = Fenstergröße in Pixeln, `ppp` = Pixel je Punkt.
#[allow(clippy::too_many_arguments)]
pub fn ui(
    ui: &mut egui::Ui,
    editor: &mut Editor,
    map_view: &mut MapView,
    lang: &Lang,
    window: Vec2,
    ppp: f32,
    time_ms: i64,
    now: Instant,
) -> AreaInfo {
    ui.ctx().set_visuals(egui::Visuals::dark());
    shortcuts(ui, editor, now);
    egui::Panel::right("editor")
        .exact_size(PANEL_WIDTH)
        .resizable(false)
        .show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| side(ui, editor, lang, time_ms, now));
        });
    let info = egui::CentralPanel::no_frame()
        .show(ui, |ui| area(ui, editor, map_view, window, ppp, now))
        .inner;
    dialogs(ui.ctx(), editor, lang);
    super::panel_adventure::dialog_window(ui.ctx(), editor, lang);
    info
}

fn shortcuts(ui: &egui::Ui, editor: &mut Editor, now: Instant) {
    let cmd = |k| KeyboardShortcut::new(Modifiers::COMMAND, k);
    let cmd_shift = |k| KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, k);
    ui.input_mut(|i| {
        if i.consume_shortcut(&cmd_shift(Key::Z)) || i.consume_shortcut(&cmd(Key::Y)) {
            editor.redo();
        }
        if i.consume_shortcut(&cmd(Key::Z)) {
            editor.undo();
        }
        if i.consume_shortcut(&cmd(Key::S)) {
            editor.save();
        }
        if i.consume_shortcut(&cmd(Key::N)) {
            editor.request(AfterDiscard::New);
        }
        if i.consume_shortcut(&cmd(Key::O)) {
            editor.dialog = Some(Dialog::Open);
        }
        if i.consume_key(Modifiers::NONE, Key::F5) {
            editor.request_test();
        }
        if i.consume_shortcut(&cmd(Key::C)) {
            editor.copy_selection();
        }
        if i.consume_shortcut(&cmd(Key::X)) {
            editor.copy_selection();
            editor.delete_selection(now);
        }
        if i.consume_shortcut(&cmd(Key::V)) && editor.clipboard.is_some() {
            editor.tool = Tool::Select;
            editor.pasting = true;
        }
        if i.consume_key(Modifiers::NONE, Key::Escape) {
            if editor.pasting {
                editor.pasting = false;
            } else if editor.dialog.is_some() {
                editor.dialog = None;
            } else {
                editor.request(AfterDiscard::Leave);
            }
        }
    });
    if ui.ctx().egui_wants_keyboard_input() {
        return; // Ziffern und Klammern gehören dem Textfeld
    }
    ui.input_mut(|i| {
        let digits = [
            Key::Num1,
            Key::Num2,
            Key::Num3,
            Key::Num4,
            Key::Num5,
            Key::Num6,
            Key::Num7,
            Key::Num8,
            Key::Num9,
        ];
        for (k, tool) in digits.into_iter().zip(Tool::ALL) {
            if i.consume_key(Modifiers::NONE, k) {
                editor.tool = tool;
                editor.pasting = false;
            }
        }
        if i.consume_key(Modifiers::NONE, Key::OpenBracket) {
            editor.brush_size = editor.brush_size.saturating_sub(1).max(1);
        }
        if i.consume_key(Modifiers::NONE, Key::CloseBracket) {
            editor.brush_size = (editor.brush_size + 1).min(MAX_BRUSH);
        }
        if i.consume_key(Modifiers::NONE, Key::Delete) {
            match (editor.tool, editor.selected_decor) {
                (Tool::Decor, Some(r)) => editor.remove_decor(r, now),
                (Tool::Adventure, _) => {
                    if let Some(id) = editor.adventure.selected.clone() {
                        editor.remove_object(&id, now);
                    }
                }
                _ => editor.delete_selection(now),
            }
        }
    });
}

fn side(ui: &mut egui::Ui, editor: &mut Editor, lang: &Lang, time_ms: i64, now: Instant) {
    ui.heading(lang.t("editor.title"));
    let title = if editor.dirty {
        format!("{}  {}", editor.map.name, lang.t("editor.unsaved"))
    } else {
        editor.map.name.clone()
    };
    ui.label(title);
    ui.separator();

    ui.strong(lang.t("editor.file"));
    ui.horizontal_wrapped(|ui| {
        if ui.button(lang.t("editor.new")).clicked() {
            editor.request(AfterDiscard::New);
        }
        if ui.button(lang.t("editor.open")).clicked() {
            editor.dialog = Some(Dialog::Open);
        }
        if ui.button(lang.t("editor.save")).clicked() {
            editor.save();
        }
    });
    if let Some(path) = editor.target_path() {
        ui.small(lang.f("editor.save_target", &[("arg", &path.display())]));
    }
    ui.horizontal(|ui| {
        if ui.button(lang.t("editor.test")).clicked() {
            editor.request_test();
        }
        if ui.button(lang.t("editor.back")).clicked() {
            editor.request(AfterDiscard::Leave);
        }
    });
    ui.separator();

    ui.strong(lang.t("editor.edit"));
    ui.horizontal(|ui| {
        if ui
            .add_enabled(editor.can_undo(), egui::Button::new(lang.t("editor.undo")))
            .clicked()
        {
            editor.undo();
        }
        if ui
            .add_enabled(editor.can_redo(), egui::Button::new(lang.t("editor.redo")))
            .clicked()
        {
            editor.redo();
        }
    });
    ui.separator();

    tools(ui, editor, lang, now);
    ui.separator();
    panel_look::backgrounds(ui, editor, lang, now);
    panel_look::envelopes(ui, editor, lang, time_ms, now);
    ui.separator();

    ui.strong(lang.t("editor.layers"));
    let v = &mut editor.visible;
    ui.checkbox(
        &mut v.layers.backgrounds,
        lang.t("editor.layer_backgrounds"),
    );
    ui.checkbox(&mut v.layers.decor_back, lang.t("editor.layer_decor_back"));
    ui.checkbox(&mut v.layers.terrain, lang.t("editor.layer_terrain"));
    ui.checkbox(&mut v.entities, lang.t("editor.layer_entities"));
    ui.checkbox(
        &mut v.layers.decor_front,
        lang.t("editor.layer_decor_front"),
    );
    ui.checkbox(&mut v.grid, lang.t("editor.layer_grid"));
    ui.separator();

    map_properties(ui, editor, lang, now);
    ui.separator();

    ui.strong(lang.t("editor.controls"));
    ui.small(lang.t("editor.controls_text"));
    ui.small(lang.f(
        "editor.zoom",
        &[("arg", &format!("{:.0}", 100.0 / editor.zoom))],
    ));
    if let Some((key, arg)) = &editor.status {
        ui.separator();
        ui.label(lang.f(key, &[("arg", arg)]));
    }
}

fn tools(ui: &mut egui::Ui, editor: &mut Editor, lang: &Lang, now: Instant) {
    ui.strong(lang.t("editor.tools"));
    ui.horizontal_wrapped(|ui| {
        for (n, tool) in Tool::ALL.into_iter().enumerate() {
            let label = format!("{} {}", n + 1, lang.t(tool.key()));
            if ui.selectable_label(editor.tool == tool, label).clicked() {
                editor.tool = tool;
                editor.pasting = false;
            }
        }
    });
    ui.small(lang.t(&format!("{}_hint", editor.tool.key())));
    match editor.tool {
        Tool::Brush | Tool::Rect | Tool::Fill => tile_choice(ui, editor, lang),
        Tool::Eraser => {}
        Tool::Entity => {
            for (kind, key) in ENTITIES {
                ui.radio_value(&mut editor.entity, kind, lang.t(key));
            }
        }
        Tool::Material => material_choice(ui, editor, lang),
        Tool::Decor => panel_look::decor_tool(ui, editor, lang, now),
        Tool::Adventure => super::panel_adventure::tool(ui, editor, lang, now),
        Tool::Select => {
            ui.horizontal_wrapped(|ui| {
                let has = editor.selection.is_some();
                if ui
                    .add_enabled(has, egui::Button::new(lang.t("editor.copy")))
                    .clicked()
                {
                    editor.copy_selection();
                }
                if ui
                    .add_enabled(has, egui::Button::new(lang.t("editor.cut")))
                    .clicked()
                {
                    editor.copy_selection();
                    editor.delete_selection(now);
                }
                if ui
                    .add_enabled(
                        editor.clipboard.is_some(),
                        egui::Button::new(lang.t("editor.paste")),
                    )
                    .clicked()
                {
                    editor.pasting = true;
                }
                if ui
                    .add_enabled(has, egui::Button::new(lang.t("editor.delete")))
                    .clicked()
                {
                    editor.delete_selection(now);
                }
            });
        }
    }
    if matches!(editor.tool, Tool::Brush | Tool::Eraser | Tool::Material) {
        ui.add(
            egui::Slider::new(&mut editor.brush_size, 1..=MAX_BRUSH)
                .text(lang.t("editor.brush_size")),
        );
    }
}

fn tile_choice(ui: &mut egui::Ui, editor: &mut Editor, lang: &Lang) {
    for (tile, key, color) in BRUSHES {
        ui.horizontal(|ui| {
            swatch(ui, color);
            ui.radio_value(&mut editor.brush, tile, lang.t(key));
        });
    }
    if editor.brush == Tile::Solid {
        material_choice(ui, editor, lang);
    }
}

fn material_choice(ui: &mut egui::Ui, editor: &mut Editor, lang: &Lang) {
    ui.horizontal_wrapped(|ui| {
        ui.label(lang.t("editor.material"));
        for m in editor.solid_materials.clone() {
            let label = lang.t(&format!("editor.mat_{m}")).to_owned();
            ui.radio_value(&mut editor.solid_material, m, label);
        }
    });
}

fn map_properties(ui: &mut egui::Ui, editor: &mut Editor, lang: &Lang, now: Instant) {
    ui.strong(lang.t("editor.map"));
    egui::Grid::new("map_props").num_columns(2).show(ui, |ui| {
        ui.label(lang.t("editor.name"));
        let mut name = editor.map.name.clone();
        if ui.text_edit_singleline(&mut name).changed() {
            editor.begin_edit("name", now);
            editor.map.name = name.chars().take(NAME_CHARS).collect();
        }
        ui.end_row();
        ui.label(lang.t("editor.author"));
        let mut author = editor.map.author.clone().unwrap_or_default();
        if ui.text_edit_singleline(&mut author).changed() {
            editor.begin_edit("author", now);
            let author: String = author.chars().take(NAME_CHARS).collect();
            editor.map.author = (!author.trim().is_empty()).then_some(author);
        }
        ui.end_row();
        ui.label(lang.t("editor.size"));
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut editor.resize.0).range(1..=elora_map::MAX_SIZE));
            ui.label("×");
            ui.add(egui::DragValue::new(&mut editor.resize.1).range(1..=elora_map::MAX_SIZE));
        });
        ui.end_row();
        ui.label("");
        let changed = editor.resize != (editor.map.width, editor.map.height);
        if ui
            .add_enabled(changed, egui::Button::new(lang.t("editor.apply_size")))
            .clicked()
        {
            let (w, h) = editor.resize;
            editor.resize_map(w, h, now);
        }
        ui.end_row();
        for (key, top) in [("editor.sky_top", true), ("editor.sky_bottom", false)] {
            ui.label(lang.t(key));
            let current = if top {
                editor.map.sky.top
            } else {
                editor.map.sky.bottom
            };
            let mut c = color32(current);
            if ui.color_edit_button_srgba(&mut c).changed() {
                editor.begin_edit(key, now);
                let target = if top {
                    &mut editor.map.sky.top
                } else {
                    &mut editor.map.sky.bottom
                };
                *target = rgba(c);
            }
            ui.end_row();
        }
        weather_properties(ui, editor, lang, now);
    });
}

/// Wetter der Karte (R2-W1, E-329): Art, Stärke und Wind.
fn weather_properties(ui: &mut egui::Ui, editor: &mut Editor, lang: &Lang, now: Instant) {
    use elora_map::WeatherKind;
    let name = |k: WeatherKind| lang.t(&format!("weather.{}", k.key())).to_owned();
    ui.label(lang.t("editor.weather"));
    let mut kind = editor.map.weather.kind;
    egui::ComboBox::from_id_salt("weather_kind")
        .selected_text(name(kind))
        .show_ui(ui, |ui| {
            for k in WeatherKind::ALL {
                ui.selectable_value(&mut kind, k, name(k));
            }
        });
    if kind != editor.map.weather.kind {
        editor.begin_edit("weather", now);
        let w = &mut editor.map.weather;
        w.kind = kind;
        if kind == WeatherKind::Clear {
            *w = elora_map::Weather::CLEAR;
        } else if w.intensity <= 0.0 {
            w.intensity = 0.7;
        }
    }
    ui.end_row();
    let clear = editor.map.weather.is_clear();
    let mut w = editor.map.weather;
    ui.label(lang.t("editor.weather_intensity"));
    let a = ui.add_enabled(!clear, egui::Slider::new(&mut w.intensity, 0.0..=1.0));
    ui.end_row();
    ui.label(lang.t("editor.weather_wind"));
    let b = ui.add_enabled(!clear, egui::Slider::new(&mut w.wind, -1.0..=1.0));
    ui.end_row();
    if a.changed() || b.changed() {
        editor.begin_edit("weather_values", now);
        editor.map.weather = w;
    }
}

/// Kartenfläche: Verschieben (mittlere Maustaste oder Leertaste + Ziehen), Zoomen (Mausrad),
/// Malen (links: Pinsel, rechts: Luft).
fn area(
    ui: &mut egui::Ui,
    editor: &mut Editor,
    map_view: &mut MapView,
    window: Vec2,
    ppp: f32,
    now: Instant,
) -> AreaInfo {
    let rect = ui.max_rect();
    let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
    let px = |p: egui::Pos2| Vec2::new(p.x * ppp, p.y * ppp);
    let center_px = px(rect.center());
    let mut info = AreaInfo {
        center_px,
        hover: None,
        preview: Preview::None,
    };
    let typing = ui.ctx().egui_wants_keyboard_input();
    let space = !typing && ui.input(|i| i.key_down(Key::Space));
    let pan = response.dragged_by(PointerButton::Middle)
        || (space && response.dragged_by(PointerButton::Primary));
    if pan {
        let d = response.drag_delta();
        editor.center -= Vec2::new(d.x, d.y) * ppp * editor.zoom;
    }
    let Some(pos) = response.hover_pos() else {
        if !ui.input(|i| i.pointer.any_down()) {
            editor.finish_stroke();
        }
        return info;
    };
    let scroll = ui.input(|i| i.smooth_scroll_delta.y);
    if scroll.abs() > f32::EPSILON {
        editor.zoom_at((-scroll * 0.0025).exp(), px(pos) - center_px);
    }
    let camera = super::view::camera(editor, window, center_px);
    let world = super::view::to_world(&camera, window, px(pos));
    info.hover = editor.tile_at(world);
    let (primary, secondary) = ui.input(|i| (i.pointer.primary_down(), i.pointer.secondary_down()));
    let down = (primary || secondary) && !space && response.is_pointer_button_down_on();
    let pressed = !space
        && response.hovered()
        && ui.input(|i| i.pointer.primary_pressed() || i.pointer.secondary_pressed());
    editor.mouse_world = Some(world);
    if editor.tool == Tool::Adventure || editor.is_adventure_map() {
        super::panel_adventure::labels(ui, editor, &camera, window, ppp);
    }
    if editor.tool == Tool::Adventure {
        super::panel_adventure::interact(editor, &mut info, world, (pressed, primary, down), now);
    } else if editor.tool == Tool::Decor {
        panel_look::decor_interact(
            editor,
            map_view,
            world,
            camera.center,
            (pressed, primary, down),
            now,
        );
    } else {
        use_tool(editor, &mut info, down, pressed, primary, now);
    }
    info
}

/// Werkzeug anwenden: `down` = Taste über der Karte gehalten, `pressed` = in diesem Frame gedrückt.
fn use_tool(
    editor: &mut Editor,
    info: &mut AreaInfo,
    down: bool,
    pressed: bool,
    primary: bool,
    now: Instant,
) {
    let Some((x, y)) = info.hover else {
        if !down {
            editor.finish_stroke();
        }
        return;
    };
    let size = editor.brush_size;
    match editor.tool {
        Tool::Brush | Tool::Eraser | Tool::Material => {
            let cells = editor.brush_cells(x, y, size);
            info.preview = Preview::Cells(cells);
            if down {
                let stroke = editor.stroke.unwrap_or_else(|| editor.start_stroke());
                let kind = format!("strich-{stroke}");
                match editor.tool {
                    Tool::Material if primary => {
                        let m = editor.solid_material.clone();
                        editor.paint_material(cells, &m, &kind, now);
                    }
                    Tool::Material => {}
                    Tool::Brush if primary => editor.fill_cells(cells, editor.brush, &kind, now),
                    _ => editor.fill_cells(cells, Tile::Air, &kind, now),
                }
            } else {
                editor.finish_stroke();
            }
        }
        Tool::Rect | Tool::Select if editor.pasting => {
            info.preview = Preview::Stamp(x, y);
            if pressed {
                if let Some(clip) = editor.clipboard.clone() {
                    editor.paste(&clip, x, y, now);
                }
                editor.pasting = false;
            }
        }
        Tool::Rect | Tool::Select => {
            if pressed {
                editor.drag = Some(((x, y), primary));
            }
            if let Some((start, left)) = editor.drag {
                let cells = Cells::span(start, (x, y));
                if editor.tool == Tool::Select {
                    editor.selection = Some(cells);
                } else {
                    info.preview = Preview::Cells(cells);
                }
                if !down {
                    editor.drag = None;
                    if editor.tool == Tool::Rect {
                        let tile = if left { editor.brush } else { Tile::Air };
                        editor.end_edit();
                        editor.fill_cells(cells, tile, "rect", now);
                        editor.end_edit();
                    }
                }
            } else {
                info.preview = Preview::Cells(Cells::span((x, y), (x, y)));
            }
        }
        Tool::Fill => {
            info.preview = Preview::Cells(Cells::span((x, y), (x, y)));
            if pressed {
                let tile = if primary { editor.brush } else { Tile::Air };
                editor.flood_fill(x, y, tile, now);
            }
        }
        Tool::Decor | Tool::Adventure => {}
        Tool::Entity => {
            info.preview = Preview::Cells(Cells::span((x, y), (x, y)));
            if pressed {
                if primary {
                    editor.place_entity(x, y, editor.entity, now);
                } else {
                    editor.remove_entity(x, y, now);
                }
            }
        }
    }
}

/// Fenster über der Kartenfläche (etwas nach links versetzt, weg von der Seitenleiste).
fn dialog_window(title: &str) -> egui::Window<'static> {
    egui::Window::new(title.to_owned())
        .collapsible(false)
        .resizable(false)
        .anchor(
            egui::Align2::CENTER_CENTER,
            egui::vec2(-PANEL_WIDTH / 2.0, 0.0),
        )
}

fn dialogs(ctx: &egui::Context, editor: &mut Editor, lang: &Lang) {
    match editor.dialog.clone() {
        Some(Dialog::New {
            name,
            width,
            height,
        }) => new_dialog(ctx, editor, lang, name, (width, height)),
        Some(Dialog::Open) => open_dialog(ctx, editor, lang),
        Some(Dialog::Discard(after)) => discard_dialog(ctx, editor, lang, after),
        None => {}
    }
}

fn new_dialog(
    ctx: &egui::Context,
    editor: &mut Editor,
    lang: &Lang,
    mut name: String,
    (mut width, mut height): (usize, usize),
) {
    let mut close = None;
    dialog_window(lang.t("editor.new_title")).show(ctx, |ui| {
        egui::Grid::new("new_map").num_columns(2).show(ui, |ui| {
            ui.label(lang.t("editor.name"));
            ui.text_edit_singleline(&mut name);
            ui.end_row();
            ui.label(lang.t("editor.width"));
            ui.add(egui::DragValue::new(&mut width).range(1..=elora_map::MAX_SIZE));
            ui.end_row();
            ui.label(lang.t("editor.height"));
            ui.add(egui::DragValue::new(&mut height).range(1..=elora_map::MAX_SIZE));
            ui.end_row();
        });
        ui.horizontal(|ui| {
            if ui.button(lang.t("editor.create")).clicked() {
                close = Some(true);
            }
            if ui.button(lang.t("editor.cancel")).clicked() {
                close = Some(false);
            }
        });
    });
    match close {
        Some(true) => {
            editor.dialog = None;
            let name: String = if name.trim().is_empty() {
                "neu".into()
            } else {
                name.chars().take(NAME_CHARS).collect()
            };
            editor.new_map(&name, width, height);
        }
        Some(false) => editor.dialog = None,
        None => {
            editor.dialog = Some(Dialog::New {
                name,
                width,
                height,
            });
        }
    }
}

fn open_dialog(ctx: &egui::Context, editor: &mut Editor, lang: &Lang) {
    let files = editor.map_files();
    let mut chosen = None;
    let mut cancel = false;
    dialog_window(lang.t("editor.open_title")).show(ctx, |ui| {
        if files.is_empty() {
            ui.label(lang.t("editor.none"));
        }
        for (own, key) in [(true, "editor.own"), (false, "editor.bundled")] {
            let group: Vec<_> = files.iter().filter(|f| f.own == own).collect();
            if group.is_empty() {
                continue;
            }
            ui.strong(lang.t(key));
            for f in group {
                if ui.button(&f.name).clicked() {
                    chosen = Some(f.path.clone());
                }
            }
        }
        ui.separator();
        cancel = ui.button(lang.t("editor.cancel")).clicked();
    });
    if let Some(path) = chosen {
        editor.dialog = None;
        editor.request(AfterDiscard::Open(path));
    } else if cancel {
        editor.dialog = None;
    }
}

fn discard_dialog(ctx: &egui::Context, editor: &mut Editor, lang: &Lang, after: AfterDiscard) {
    let mut choice = None;
    dialog_window(lang.t("editor.discard_title")).show(ctx, |ui| {
        ui.label(lang.t("editor.discard_text"));
        ui.horizontal(|ui| {
            if ui.button(lang.t("editor.save")).clicked() {
                choice = Some(0);
            }
            if ui.button(lang.t("editor.discard")).clicked() {
                choice = Some(1);
            }
            if ui.button(lang.t("editor.cancel")).clicked() {
                choice = Some(2);
            }
        });
    });
    match choice {
        Some(0) => {
            editor.save();
            if !editor.dirty {
                editor.proceed(after);
            }
        }
        Some(1) => editor.proceed(after),
        Some(2) => editor.dialog = None,
        _ => {}
    }
}

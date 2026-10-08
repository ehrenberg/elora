//! Editor UI for the look (M6.8): decoration tool, background layers, animations.

// UI code: short names for object, copy and color
#![allow(clippy::many_single_char_names)]

use std::time::Instant;

use elora_map::look::{Curve, EnvKind, EnvRef};
use elora_map::{Art, Decor, Rgba};
use elora_sim::Vec2;

use super::Editor;
use super::look::{self, DecorLayer, DecorRef, Preset};
use crate::lang::Lang;
use crate::map_art::{BACKGROUND_FILES, DECOR_FILES};
use crate::map_view::MapView;

const CURVES: [(Curve, &str); 5] = [
    (Curve::Step, "editor.curve_step"),
    (Curve::Linear, "editor.curve_linear"),
    (Curve::Slow, "editor.curve_slow"),
    (Curve::Fast, "editor.curve_fast"),
    (Curve::Smooth, "editor.curve_smooth"),
];

fn color32(c: Rgba) -> egui::Color32 {
    let [red, green, blue, alpha] = c.0;
    egui::Color32::from_rgba_unmultiplied(red, green, blue, alpha)
}

fn layer_name(editor: &Editor, layer: DecorLayer, lang: &Lang) -> String {
    match layer {
        DecorLayer::Back => lang.t("editor.layer_decor_back").to_owned(),
        DecorLayer::Front => lang.t("editor.layer_decor_front").to_owned(),
        DecorLayer::Background(i) => editor.map.backgrounds.get(i).map_or_else(String::new, |b| {
            format!("{}: {}", lang.t("editor.background"), b.name)
        }),
    }
}

fn art_name(editor: &Editor, art: &Art, lang: &Lang) -> String {
    match art {
        Art::Builtin(n) => lang.t(&format!("editor.art_{n}")).to_owned(),
        Art::Image(i) => editor
            .map
            .images
            .get(usize::from(*i))
            .map_or_else(|| "?".into(), |img| format!("🖼 {}", img.name)),
    }
}

/// Sidebar for the decoration tool: target layer, graphic, selected object, custom SVGs.
pub fn decor_tool(ui: &mut egui::Ui, editor: &mut Editor, lang: &Lang, now: Instant) {
    let mut layers = vec![DecorLayer::Back, DecorLayer::Front];
    layers.extend((0..editor.map.backgrounds.len()).map(DecorLayer::Background));
    let current = layer_name(editor, editor.decor_layer, lang);
    egui::ComboBox::from_label(lang.t("editor.target_layer"))
        .selected_text(current)
        .show_ui(ui, |ui| {
            for l in layers {
                let name = layer_name(editor, l, lang);
                ui.selectable_value(&mut editor.decor_layer, l, name);
            }
        });
    egui::CollapsingHeader::new(lang.t("editor.palette"))
        .default_open(true)
        .show(ui, |ui| {
            let mut arts: Vec<Art> = DECOR_FILES
                .iter()
                .chain(BACKGROUND_FILES)
                .map(|(n, _)| Art::Builtin((*n).into()))
                .collect();
            arts.extend(
                (0..editor.map.images.len()).filter_map(|i| u16::try_from(i).ok().map(Art::Image)),
            );
            ui.horizontal_wrapped(|ui| {
                for a in arts {
                    let label = art_name(editor, &a, lang);
                    ui.selectable_value(&mut editor.decor_art, a, label);
                }
            });
        });
    if let Some(r) = editor.selected_decor
        && editor.decor(r).is_some()
    {
        ui.separator();
        selected_decor(ui, editor, r, lang, now);
    }
    ui.separator();
    images(ui, editor, lang, now);
}

fn env_choice(
    ui: &mut egui::Ui,
    editor: &Editor,
    id: &str,
    kind: EnvKind,
    current: Option<EnvRef>,
    lang: &Lang,
) -> (Option<EnvRef>, bool) {
    let mut chosen = current.map(|r| r.index);
    let name = |i: Option<u16>| {
        i.and_then(|i| editor.map.envelopes.get(usize::from(i)))
            .map_or_else(|| lang.t("editor.none_env").to_owned(), |e| e.name.clone())
    };
    egui::ComboBox::from_id_salt(id)
        .selected_text(name(chosen))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut chosen, None, lang.t("editor.none_env"));
            for (i, e) in editor.map.envelopes.iter().enumerate() {
                if e.kind == kind {
                    let i = u16::try_from(i).ok();
                    ui.selectable_value(&mut chosen, i, e.name.clone());
                }
            }
        });
    let mut offset = current.map_or(0, |r| r.offset_ms);
    let off_changed = chosen.is_some()
        && ui
            .add(egui::DragValue::new(&mut offset).suffix(" ms").speed(10.0))
            .changed();
    let new = chosen.map(|index| EnvRef {
        index,
        offset_ms: offset,
    });
    (new, new != current || off_changed)
}

fn selected_decor(ui: &mut egui::Ui, editor: &mut Editor, r: DecorRef, lang: &Lang, now: Instant) {
    let Some(d) = editor.decor(r).cloned() else {
        return;
    };
    ui.strong(format!(
        "{} – {}",
        art_name(editor, &d.art, lang),
        layer_name(editor, r.layer, lang)
    ));
    let mut n = d.clone();
    egui::Grid::new("decor_props")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label(lang.t("editor.position"));
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut n.pos.x).speed(1.0));
                ui.add(egui::DragValue::new(&mut n.pos.y).speed(1.0));
            });
            ui.end_row();
            ui.label(lang.t("editor.scale"));
            ui.add(egui::Slider::new(&mut n.scale, 0.1..=8.0).logarithmic(true));
            ui.end_row();
            ui.label(lang.t("editor.rotation"));
            ui.add(egui::Slider::new(&mut n.rotation, -180.0..=180.0).suffix("°"));
            ui.end_row();
            ui.label(lang.t("editor.flip"));
            ui.checkbox(&mut n.flip_x, "");
            ui.end_row();
            ui.label(lang.t("editor.tint"));
            let mut c = color32(n.tint);
            if ui.color_edit_button_srgba(&mut c).changed() {
                n.tint = Rgba(c.to_srgba_unmultiplied());
            }
            ui.end_row();
            ui.label(lang.t("editor.motion"));
            ui.horizontal(|ui| {
                let (v, changed) =
                    env_choice(ui, editor, "pos_env", EnvKind::Position, n.pos_env, lang);
                if changed {
                    n.pos_env = v;
                }
            });
            ui.end_row();
            ui.label(lang.t("editor.color_anim"));
            ui.horizontal(|ui| {
                let (v, changed) =
                    env_choice(ui, editor, "color_env", EnvKind::Color, n.color_env, lang);
                if changed {
                    n.color_env = v;
                }
            });
            ui.end_row();
        });
    if n != d
        && let Some(target) = editor.edit_decor(r, "decor-props", now)
    {
        *target = n;
    }
    if ui.button(lang.t("editor.delete")).clicked() {
        editor.remove_decor(r, now);
    }
}

fn images(ui: &mut egui::Ui, editor: &mut Editor, lang: &Lang, now: Instant) {
    egui::CollapsingHeader::new(lang.t("editor.images")).show(ui, |ui| {
        ui.small(lang.t("editor.images_hint"));
        ui.text_edit_singleline(&mut editor.image_path);
        if ui.button(lang.t("editor.embed")).clicked() {
            let path = std::path::PathBuf::from(editor.image_path.trim());
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let result = std::fs::read(&path)
                .map_err(|e| ("editor.embed_failed", e.to_string()))
                .and_then(|data| editor.embed_image(&name, data, now));
            editor.status = Some(match result {
                Ok(i) => {
                    editor.decor_art = Art::Image(i);
                    ("editor.embedded", name)
                }
                Err(e) => e,
            });
        }
        let mut remove = None;
        for (i, img) in editor.map.images.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{} ({} KiB)",
                    img.name,
                    img.svg.len().div_ceil(1024)
                ));
                if ui.small_button("✖").clicked() {
                    remove = Some(i);
                }
            });
        }
        if let Some(i) = remove {
            editor.remove_image(i, now);
        }
    });
}

/// Background layers: presets, list with order, properties of the selected layer.
pub fn backgrounds(ui: &mut egui::Ui, editor: &mut Editor, lang: &Lang, now: Instant) {
    egui::CollapsingHeader::new(lang.t("editor.backgrounds"))
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button(lang.t("editor.preset_day")).clicked() {
                    editor.apply_preset(Preset::Day, now);
                }
                if ui.button(lang.t("editor.preset_night")).clicked() {
                    editor.apply_preset(Preset::Night, now);
                }
                if ui.button(lang.t("editor.add_layer")).clicked() {
                    editor.add_background(now);
                }
            });
            ui.small(lang.t("editor.backgrounds_hint"));
            for i in (0..editor.map.backgrounds.len()).rev() {
                let name = editor.map.backgrounds[i].name.clone();
                if ui
                    .selectable_label(editor.selected_bg == Some(i), format!("{}. {name}", i + 1))
                    .clicked()
                {
                    editor.selected_bg = Some(i);
                    editor.decor_layer = DecorLayer::Background(i);
                }
            }
            if let Some(i) = editor
                .selected_bg
                .filter(|&i| i < editor.map.backgrounds.len())
            {
                ui.separator();
                background_props(ui, editor, i, lang, now);
            }
        });
}

fn background_props(ui: &mut egui::Ui, editor: &mut Editor, i: usize, lang: &Lang, now: Instant) {
    let old = editor.map.backgrounds[i].clone();
    let mut b = old.clone();
    egui::Grid::new("bg_props").num_columns(2).show(ui, |ui| {
        ui.label(lang.t("editor.name"));
        ui.text_edit_singleline(&mut b.name);
        ui.end_row();
        ui.label(lang.t("editor.parallax"));
        ui.horizontal(|ui| {
            ui.add(
                egui::DragValue::new(&mut b.parallax.x)
                    .range(0.0..=2.0)
                    .speed(0.01),
            );
            ui.add(
                egui::DragValue::new(&mut b.parallax.y)
                    .range(0.0..=2.0)
                    .speed(0.01),
            );
        });
        ui.end_row();
        ui.label(lang.t("editor.offset"));
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut b.offset.x).speed(1.0));
            ui.add(egui::DragValue::new(&mut b.offset.y).speed(1.0));
        });
        ui.end_row();
        ui.label(lang.t("editor.repeat"));
        ui.horizontal(|ui| {
            let mut on = b.repeat_x.is_some();
            ui.checkbox(&mut on, "");
            let mut w = b.repeat_x.unwrap_or(1024.0);
            if on {
                ui.add(egui::DragValue::new(&mut w).range(16.0..=8192.0).speed(4.0));
            }
            b.repeat_x = on.then_some(w);
        });
        ui.end_row();
    });
    b.name = b.name.chars().take(32).collect();
    if b != old {
        editor.begin_edit("bg-props", now);
        editor.map.backgrounds[i] = b;
    }
    ui.horizontal(|ui| {
        if ui.button(lang.t("editor.forward")).clicked() {
            editor.move_background(i, true, now);
        }
        if ui.button(lang.t("editor.backward")).clicked() {
            editor.move_background(i, false, now);
        }
        if ui.button(lang.t("editor.delete")).clicked() {
            editor.remove_background(i, now);
        }
    });
}

/// Animations: list, points as a table, curve plot with time marker.
pub fn envelopes(ui: &mut egui::Ui, editor: &mut Editor, lang: &Lang, time_ms: i64, now: Instant) {
    egui::CollapsingHeader::new(lang.t("editor.animations"))
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button(lang.t("editor.add_motion")).clicked() {
                    editor.add_envelope(EnvKind::Position, now);
                }
                if ui.button(lang.t("editor.add_color")).clicked() {
                    editor.add_envelope(EnvKind::Color, now);
                }
            });
            for (i, e) in editor.map.envelopes.iter().enumerate() {
                let kind = match e.kind {
                    EnvKind::Position => lang.t("editor.motion"),
                    EnvKind::Color => lang.t("editor.color_anim"),
                };
                if ui
                    .selectable_label(
                        editor.selected_env == Some(i),
                        format!("{} ({kind})", e.name),
                    )
                    .clicked()
                {
                    editor.selected_env = Some(i);
                }
            }
            if let Some(i) = editor
                .selected_env
                .filter(|&i| i < editor.map.envelopes.len())
            {
                ui.separator();
                envelope_props(ui, editor, i, lang, time_ms, now);
            }
        });
}

fn envelope_props(
    ui: &mut egui::Ui,
    editor: &mut Editor,
    i: usize,
    lang: &Lang,
    time_ms: i64,
    now: Instant,
) {
    let old = editor.map.envelopes[i].clone();
    let mut e = old.clone();
    ui.horizontal(|ui| {
        ui.label(lang.t("editor.name"));
        ui.text_edit_singleline(&mut e.name);
    });
    ui.checkbox(&mut e.synced, lang.t("editor.synced"));
    plot(ui, &e, time_ms);
    let channels: &[&str] = match e.kind {
        EnvKind::Position => &["x", "y", "°"],
        EnvKind::Color => &["R", "G", "B", "A"],
    };
    let mut remove = None;
    egui::Grid::new("env_points")
        .num_columns(channels.len() + 3)
        .show(ui, |ui| {
            ui.label("ms");
            for c in channels {
                ui.label(*c);
            }
            ui.label(lang.t("editor.curve"));
            ui.end_row();
            let count = e.points.len();
            for (k, p) in e.points.iter_mut().enumerate() {
                ui.add(egui::DragValue::new(&mut p.time_ms).speed(10.0));
                for v in p.value.iter_mut().take(channels.len()) {
                    let speed = if e.kind == EnvKind::Color { 0.01 } else { 0.5 };
                    ui.add(egui::DragValue::new(v).speed(speed));
                }
                let curve_label = CURVES
                    .iter()
                    .find(|(c, _)| *c == p.curve)
                    .map_or("", |(_, k)| lang.t(k));
                egui::ComboBox::from_id_salt(("curve", k))
                    .selected_text(curve_label)
                    .show_ui(ui, |ui| {
                        for (c, key) in CURVES {
                            ui.selectable_value(&mut p.curve, c, lang.t(key));
                        }
                    });
                if count > 1 && ui.small_button("✖").clicked() {
                    remove = Some(k);
                }
                ui.end_row();
            }
        });
    if let Some(k) = remove {
        e.points.remove(k);
    }
    ui.horizontal(|ui| {
        if ui.button(lang.t("editor.add_point")).clicked()
            && let Some(last) = e.points.last().copied()
            && e.points.len() < 1024
        {
            e.points.push(elora_map::look::EnvPoint {
                time_ms: last.time_ms + 500,
                ..last
            });
        }
        if ui.button(lang.t("editor.delete")).clicked() {
            remove = Some(usize::MAX);
        }
    });
    if remove == Some(usize::MAX) {
        editor.remove_envelope(i, now);
        return;
    }
    e.name = e.name.chars().take(32).collect();
    if e != old {
        editor.begin_edit("env-props", now);
        editor.map.envelopes[i] = e;
        editor.normalize_envelope(i);
    }
}

/// Curve plot: all channels over one loop, scaled to the common value range.
fn plot(ui: &mut egui::Ui, e: &elora_map::Envelope, time_ms: i64) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 90.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, egui::Color32::from_gray(28));
    let dur = i64::from(e.duration_ms().max(1));
    let channels = match e.kind {
        EnvKind::Position => 3,
        EnvKind::Color => 4,
    };
    let samples: Vec<[f32; 4]> = (0..=120).map(|k| e.eval(dur * k / 120)).collect();
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for s in &samples {
        for v in &s[..channels] {
            lo = lo.min(*v);
            hi = hi.max(*v);
        }
    }
    if hi - lo < 1e-3 {
        lo -= 1.0;
        hi += 1.0;
    }
    let colors = [
        egui::Color32::from_rgb(230, 90, 90),
        egui::Color32::from_rgb(90, 200, 120),
        egui::Color32::from_rgb(90, 150, 230),
        egui::Color32::from_gray(220),
    ];
    #[allow(clippy::cast_precision_loss)]
    let point = |k: usize, v: f32| {
        egui::pos2(
            rect.left() + rect.width() * k as f32 / 120.0,
            rect.bottom() - 6.0 - (rect.height() - 12.0) * (v - lo) / (hi - lo),
        )
    };
    for c in 0..channels {
        let line: Vec<egui::Pos2> = samples
            .iter()
            .enumerate()
            .map(|(k, s)| point(k, s[c]))
            .collect();
        painter.add(egui::Shape::line(line, egui::Stroke::new(1.5, colors[c])));
    }
    #[allow(clippy::cast_precision_loss)]
    let x = rect.left() + rect.width() * (time_ms.rem_euclid(dur) as f32 / dur as f32);
    painter.line_segment(
        [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
        egui::Stroke::new(1.0, egui::Color32::from_rgb(242, 193, 78)),
    );
}

/// Decoration tool on the map area: clicking an object selects it (dragging moves it),
/// clicking next to it places the selected graphic, right-click removes.
#[allow(clippy::fn_params_excessive_bools)]
pub fn decor_interact(
    editor: &mut Editor,
    map_view: &mut MapView,
    world: Vec2,
    camera: Vec2,
    (pressed, primary, down): (bool, bool, bool),
    now: Instant,
) {
    if pressed {
        let found = pick(editor, map_view, world, camera);
        if primary {
            if let Some(r) = found {
                editor.selected_decor = Some(r);
                editor.decor_drag = Some(world);
                editor.end_edit();
            } else {
                let art = editor.decor_art.clone();
                editor.add_decor(editor.decor_layer, art, world, camera, now);
                editor.decor_drag = Some(world);
            }
        } else if let Some(r) = found {
            editor.remove_decor(r, now);
        }
        return;
    }
    match (editor.decor_drag, editor.selected_decor) {
        (Some(last), Some(r)) if down => {
            if world != last {
                editor.move_decor(r, world - last, now);
                editor.decor_drag = Some(world);
            }
        }
        _ => {
            if editor.decor_drag.take().is_some() {
                editor.end_edit();
            }
        }
    }
}

/// Topmost visible object under `world`: front, back, then backgrounds from front to back.
fn pick(editor: &Editor, map_view: &mut MapView, world: Vec2, camera: Vec2) -> Option<DecorRef> {
    let v = editor.visible.layers;
    let mut order = Vec::new();
    if v.decor_front {
        order.push(DecorLayer::Front);
    }
    if v.decor_back {
        order.push(DecorLayer::Back);
    }
    if v.backgrounds {
        order.extend(
            (0..editor.map.backgrounds.len())
                .rev()
                .map(DecorLayer::Background),
        );
    }
    for layer in order {
        let items = editor.layer_items(layer)?;
        for index in (0..items.len()).rev() {
            let r = DecorRef { layer, index };
            let d: &Decor = &items[index];
            let Some(bounds) = map_view.decor_bounds(&editor.map, d) else {
                continue;
            };
            let Some(at) = editor.decor_world_pos(r, camera, world) else {
                continue;
            };
            if look::hit(d, at, bounds, world) {
                return Some(r);
            }
        }
    }
    None
}

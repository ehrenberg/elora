//! Einstellungsseiten im Hauptmenü (M7.4): Spieler & Skin, Steuerung, Grafik, Ton,
//! Sprache. Änderungen wirken sofort; die App speichert sie (E-116).

// Layout-Code: `s` (Skalierung), `x`/`y` sind hier lesbarer als lange Namen
#![allow(clippy::many_single_char_names)]

use elora_audio::AudioSettings;
use elora_protocol::Skin;
use elora_render::{Align, Color};
use elora_sim::{Team, Vec2};

use crate::effects::EffectSettings;
use crate::lang::Language;
use crate::menu::MenuCtx;
use crate::settings::GraphicsSettings;
use crate::skins::{BODY, EYES};
use crate::ui::{BLUE, ORANGE, Rect, TEXT_DIM, Ui};

/// Was die Einstellungsseiten bearbeiten (Verweise in die App).
pub struct SettingsEdit<'a> {
    pub name: &'a mut String,
    pub skin: &'a mut Skin,
    pub graphics: &'a mut GraphicsSettings,
    pub audio: &'a mut AudioSettings,
    pub effects: &'a mut EffectSettings,
    /// Maus-Empfindlichkeit in Prozent.
    pub sensitivity: &'a mut f32,
    pub language: &'a mut Language,
    /// Audiogerät vorhanden?
    pub audio_device: bool,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn percent(v: f32) -> u32 {
    (v * 100.0).round() as u32
}

/// Zeichnet die Seite `tab` in `page`; `true`, wenn sich etwas geändert hat.
pub fn page(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    page: Rect,
    tab: usize,
    edit: &mut SettingsEdit<'_>,
) -> bool {
    match tab {
        0 => player(ui, cx, page, edit),
        1 => controls(ui, cx, page, edit),
        2 => graphics(ui, cx, page, edit),
        3 => audio(ui, cx, page, edit),
        _ => language(ui, cx, page, edit),
    }
}

fn player(ui: &mut Ui<'_>, cx: &MenuCtx<'_>, page: Rect, edit: &mut SettingsEdit<'_>) -> bool {
    let s = cx.s;
    let lang = cx.lang;
    let x = page.min.x + 20.0 * s;
    let mut y = page.min.y + 60.0 * s;
    let mut changed = false;
    ui.label(
        lang.t("settings.name"),
        Vec2::new(x, y),
        11.0,
        TEXT_DIM,
        Align::Left,
    );
    changed |= ui.text_field(
        "set_name",
        Rect::new(x, y + 12.0 * s, 240.0 * s, 30.0 * s),
        edit.name,
        16,
        "Elora",
    ) != crate::ui::FieldEvent::None;
    y += 64.0 * s;
    let body: Vec<Color> = BODY.iter().map(|(_, c)| *c).collect();
    let eyes: Vec<Color> = EYES.iter().map(|(_, c)| *c).collect();
    for (label, id, colors, value) in [
        ("settings.body", "set_body", &body, &mut edit.skin.body),
        ("settings.feet", "set_feet", &body, &mut edit.skin.feet),
        ("settings.eyes", "set_eyes", &eyes, &mut edit.skin.eyes),
    ] {
        ui.label(lang.t(label), Vec2::new(x, y), 11.0, TEXT_DIM, Align::Left);
        changed |= ui.swatches(id, Vec2::new(x, y + 10.0 * s), colors, value);
        y += 46.0 * s;
    }
    ui.label(
        lang.t("settings.team_note"),
        Vec2::new(x, y + 4.0 * s),
        11.0,
        TEXT_DIM,
        Align::Left,
    );
    // Vorschau
    let center = Vec2::new(page.max.x - 100.0 * s, page.min.y + 190.0 * s);
    ui.batch
        .fill_circle(center, 78.0 * s, Color::rgba(0.949, 0.757, 0.306, 0.2));
    let tint = crate::skins::tint(*edit.skin, Team::None, false, crate::draw::team_color);
    cx.art.draw_pose(
        ui.batch,
        center + Vec2::new(0.0, 70.0 * s),
        140.0 * s,
        1.0,
        &tint,
    );
    changed
}

fn controls(ui: &mut Ui<'_>, cx: &MenuCtx<'_>, page: Rect, edit: &mut SettingsEdit<'_>) -> bool {
    let s = cx.s;
    let lang = cx.lang;
    let x = page.min.x + 20.0 * s;
    let y = page.min.y + 60.0 * s;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = edit.sensitivity.round() as u32;
    ui.label(
        &lang.f("settings.mouse", &[("n", &n)]),
        Vec2::new(x, y),
        11.0,
        TEXT_DIM,
        Align::Left,
    );
    let changed = ui.slider(
        "sens",
        Rect::new(x, y + 10.0 * s, 300.0 * s, 24.0 * s),
        edit.sensitivity,
        10.0,
        400.0,
    );
    ui.label(
        lang.t("settings.bindings_soon"),
        Vec2::new(x, y + 60.0 * s),
        11.0,
        TEXT_DIM,
        Align::Left,
    );
    changed
}

fn graphics(ui: &mut Ui<'_>, cx: &MenuCtx<'_>, page: Rect, edit: &mut SettingsEdit<'_>) -> bool {
    let s = cx.s;
    let lang = cx.lang;
    let x = page.min.x + 20.0 * s;
    let mut y = page.min.y + 52.0 * s;
    let mut changed = false;
    let g = &mut *edit.graphics;
    for (id, key, value) in [
        ("fullscreen", "settings.fullscreen", &mut g.fullscreen),
        ("vsync", "settings.vsync", &mut g.vsync),
        ("msaa", "settings.msaa", &mut g.msaa),
    ] {
        changed |= ui.toggle(id, Vec2::new(x, y), lang.t(key), value);
        y += 34.0 * s;
    }
    y += 10.0 * s;
    ui.label(
        &lang.f("settings.ui_scale", &[("n", &percent(g.ui_scale))]),
        Vec2::new(x, y),
        11.0,
        TEXT_DIM,
        Align::Left,
    );
    // Regler in 5-%-Schritten, damit die Größe nicht bei jedem Pixel springt
    let mut scale = g.ui_scale;
    if ui.slider(
        "ui_scale",
        Rect::new(x, y + 10.0 * s, 300.0 * s, 24.0 * s),
        &mut scale,
        0.5,
        2.0,
    ) {
        let stepped = (scale * 20.0).round() / 20.0;
        if (stepped - g.ui_scale).abs() > f32::EPSILON {
            g.ui_scale = stepped;
            changed = true;
        }
    }
    y += 56.0 * s;
    let e = &mut *edit.effects;
    changed |= ui.toggle(
        "shake",
        Vec2::new(x, y),
        lang.t("settings.shake"),
        &mut e.camera_shake,
    );
    changed |= ui.toggle(
        "marker",
        Vec2::new(x, y + 34.0 * s),
        lang.t("settings.hit_marker"),
        &mut e.hit_marker,
    );
    changed
}

fn audio(ui: &mut Ui<'_>, cx: &MenuCtx<'_>, page: Rect, edit: &mut SettingsEdit<'_>) -> bool {
    let s = cx.s;
    let lang = cx.lang;
    let x = page.min.x + 20.0 * s;
    let mut y = page.min.y + 60.0 * s;
    let mut changed = false;
    let a = &mut *edit.audio;
    if !edit.audio_device {
        ui.label(
            lang.t("settings.no_device"),
            Vec2::new(x, y - 10.0 * s),
            11.0,
            ORANGE,
            Align::Left,
        );
        y += 20.0 * s;
    }
    for (id, key, value) in [
        ("volume", "settings.volume", &mut a.volume),
        ("music", "settings.music", &mut a.music_volume),
    ] {
        ui.label(
            &lang.f(key, &[("n", &percent(*value))]),
            Vec2::new(x, y),
            11.0,
            TEXT_DIM,
            Align::Left,
        );
        changed |= ui.slider(
            id,
            Rect::new(x, y + 10.0 * s, 300.0 * s, 24.0 * s),
            value,
            0.0,
            1.0,
        );
        y += 56.0 * s;
    }
    changed |= ui.toggle(
        "muted",
        Vec2::new(x, y),
        lang.t("settings.muted"),
        &mut a.muted,
    );
    changed
}

fn language(ui: &mut Ui<'_>, cx: &MenuCtx<'_>, page: Rect, edit: &mut SettingsEdit<'_>) -> bool {
    let s = cx.s;
    let x = page.min.x + 20.0 * s;
    let y = page.min.y + 52.0 * s;
    let names: Vec<&str> = Language::ALL.iter().map(|l| l.name()).collect();
    let current = Language::ALL
        .iter()
        .position(|l| l == edit.language)
        .unwrap_or(0);
    let changed =
        if let Some(i) = ui.tabs("lang", Vec2::new(x, y), 30.0 * s, &names, current, &[BLUE]) {
            *edit.language = Language::ALL[i];
            true
        } else {
            false
        };
    ui.label(
        cx.lang.t("settings.language_note"),
        Vec2::new(x, y + 56.0 * s),
        11.0,
        TEXT_DIM,
        Align::Left,
    );
    changed
}

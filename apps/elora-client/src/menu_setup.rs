//! Setup screen on the first start (RF-30, E-351, E-360): name, language and Elora's look on
//! one page, shown once while there is no settings file yet; it cannot be skipped.

use elora_render::{Align, Color};
use elora_sim::{Team, Vec2};

use crate::lang::Language;
use crate::menu::{MenuAction, MenuCtx};
use crate::menu_settings::SettingsEdit;
use crate::skins::{BODY, EYES};
use crate::ui::{GREEN, ORANGE, Rect, SAND, TEXT, TEXT_DIM, Ui};

/// Red of the missing-name hint.
const WARN: Color = Color::hex(0xd94a4a);

/// Draws the page; returns `SetupDone` once a valid name is confirmed, and whether a setting
/// changed (the language switches the screen at once).
pub fn page(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    edit: &mut SettingsEdit<'_>,
) -> (Option<MenuAction>, bool) {
    let scale = cx.s;
    let width = (760.0 * scale).min(cx.screen.x - 40.0 * scale);
    let height = (440.0 * scale).min(cx.screen.y - 40.0 * scale);
    let card = Rect::new(
        (cx.screen.x - width) / 2.0,
        (cx.screen.y - height) / 2.0,
        width,
        height,
    );
    ui.card(card);
    let mut changed = header(ui, cx, card, edit.language);
    changed |= name_and_colours(ui, cx, card, edit);
    let name = crate::settings::player_name(edit.name);
    preview(ui, cx, card, edit, name.as_deref());
    let go = Rect::new(
        card.max.x - 210.0 * scale,
        card.max.y - 66.0 * scale,
        160.0 * scale,
        40.0 * scale,
    );
    let color = if name.is_some() { GREEN } else { SAND };
    let clicked = ui.button("setup_go", go, cx.lang.t("setup.go"), color);
    let action = match name {
        Some(n) if clicked => {
            *edit.name = n;
            Some(MenuAction::SetupDone)
        }
        _ => None,
    };
    (action, changed)
}

/// Title, introduction and the language switch (top right).
fn header(ui: &mut Ui<'_>, cx: &MenuCtx<'_>, card: Rect, language: &mut Language) -> bool {
    let (scale, lang) = (cx.s, cx.lang);
    let x = card.min.x + 32.0 * scale;
    ui.label(
        lang.t("setup.title"),
        Vec2::new(x, card.min.y + 44.0 * scale),
        26.0,
        TEXT,
        Align::Left,
    );
    ui.label(
        lang.t("setup.sub"),
        Vec2::new(x, card.min.y + 70.0 * scale),
        12.0,
        TEXT_DIM,
        Align::Left,
    );
    let names: Vec<&str> = Language::ALL.iter().map(|l| l.name()).collect();
    let current = Language::ALL
        .iter()
        .position(|l| l == language)
        .unwrap_or(0);
    let lx = card.max.x - 220.0 * scale;
    ui.label(
        lang.t("setup.language"),
        Vec2::new(lx, card.min.y + 34.0 * scale),
        11.0,
        TEXT_DIM,
        Align::Left,
    );
    match ui.tabs(
        "setup_lang",
        Vec2::new(lx, card.min.y + 42.0 * scale),
        26.0 * scale,
        &names,
        current,
        &[ORANGE],
    ) {
        Some(idx) if idx != current => {
            *language = Language::ALL[idx];
            true
        }
        _ => false,
    }
}

/// Name field (required) and the colours, with the palettes of the player settings.
fn name_and_colours(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    card: Rect,
    edit: &mut SettingsEdit<'_>,
) -> bool {
    let (scale, lang) = (cx.s, cx.lang);
    let x = card.min.x + 32.0 * scale;
    let mut y = card.min.y + 110.0 * scale;
    ui.label(
        lang.t("settings.name"),
        Vec2::new(x, y),
        11.0,
        TEXT_DIM,
        Align::Left,
    );
    let mut changed = ui.text_field(
        "setup_name",
        Rect::new(x, y + 10.0 * scale, 260.0 * scale, 32.0 * scale),
        edit.name,
        crate::settings::NAME_MAX,
        "",
    ) != crate::ui::FieldEvent::None;
    if crate::settings::player_name(edit.name).is_none() {
        ui.label(
            lang.t("setup.hint"),
            Vec2::new(x, y + 58.0 * scale),
            11.0,
            WARN,
            Align::Left,
        );
    }
    y += 84.0 * scale;
    let body: Vec<Color> = BODY.iter().map(|(_, c)| *c).collect();
    let eyes: Vec<Color> = EYES.iter().map(|(_, c)| *c).collect();
    for (label, id, colors, value) in [
        ("settings.body", "setup_body", &body, &mut edit.skin.body),
        ("settings.feet", "setup_feet", &body, &mut edit.skin.feet),
        ("settings.eyes", "setup_eyes", &eyes, &mut edit.skin.eyes),
    ] {
        ui.label(lang.t(label), Vec2::new(x, y), 11.0, TEXT_DIM, Align::Left);
        changed |= ui.swatches(id, Vec2::new(x, y + 10.0 * scale), colors, value);
        y += 46.0 * scale;
    }
    ui.label(
        lang.t("setup.later"),
        Vec2::new(x, card.max.y - 26.0 * scale),
        11.0,
        TEXT_DIM,
        Align::Left,
    );
    changed
}

/// Live preview of Elora with the chosen colours and name.
fn preview(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    card: Rect,
    edit: &SettingsEdit<'_>,
    name: Option<&str>,
) {
    let scale = cx.s;
    let center = Vec2::new(card.max.x - 130.0 * scale, card.min.y + 210.0 * scale);
    ui.batch
        .fill_circle(center, 84.0 * scale, Color::rgba(0.949, 0.757, 0.306, 0.2));
    let tint = crate::skins::tint(*edit.skin, Team::None, false, crate::draw::team_color);
    cx.art.draw_pose(
        ui.batch,
        center + Vec2::new(0.0, 74.0 * scale),
        150.0 * scale,
        1.0,
        &tint,
    );
    ui.label(
        name.unwrap_or("…"),
        Vec2::new(center.x, center.y + 104.0 * scale),
        14.0,
        TEXT,
        Align::Center,
    );
}

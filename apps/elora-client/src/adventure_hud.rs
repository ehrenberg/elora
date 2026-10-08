//! Adventure displays (A1.7, E-222, E-225): level with experience ring, gleam drops,
//! current quest, conversation box with portrait and choices, speech bubbles, hints.
//!
//! Everything in screen pixels via [`crate::ui`]; style like the drafts
//! (`docs/release-2/design/adventure-ui.png`).

// Layout code: `s` (scale), `w`/`h`/`x`/`y` as in menu.rs
#![allow(clippy::many_single_char_names)]

use elora_adventure::data::Text;
use elora_adventure::dialog::{Choice, Tone};
use elora_adventure::quest::{Goal, QuestKind, QuestStatus};
use elora_adventure::{Content, SaveGame};
use elora_render::{Align, Color};
use elora_sim::Vec2;

use crate::creatures::CreatureArt;
use crate::lang::Lang;
use crate::ui::{self, Rect, Ui};

/// Dark HUD surface like the bar at the bottom (hud.rs).
const PANEL: Color = Color::rgba(0.118, 0.165, 0.212, 0.6);
const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
const WHITE_DIM: Color = Color::rgba(1.0, 1.0, 1.0, 0.75);
const GOLD: Color = Color::hex(0xf2c14e);
const XP: Color = Color::hex(0x7fd99a);
/// Health bar of a guardian.
const BOSS: Color = Color::hex(0xe8685a);
const TONE_FRIENDLY: Color = Color::hex(0xf2c14e);
const TONE_CURIOUS: Color = Color::hex(0x5aaee8);
const TONE_CHEEKY: Color = Color::hex(0xe8685a);

/// Width of the bar at the bottom (hud.rs), for the level badge to the left of it.
const BAR_WIDTH: f32 = 300.0;
const BAR_HEIGHT: f32 = 54.0;
const BAR_MARGIN: f32 = 14.0;

/// Circular arc as a polyline (angle in radians, 0 = right, clockwise).
fn arc(center: Vec2, r: f32, from: f32, to: f32) -> Vec<Vec2> {
    let n = 32;
    (0..=n)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let a = from + (to - from) * i as f32 / n as f32;
            center + Vec2::new(a.cos() * r, a.sin() * r)
        })
        .collect()
}

/// Level with experience ring left of the bar, gleam drops top left, quest top right.
#[allow(clippy::too_many_lines)] // three small displays in one place
pub fn status(
    ui: &mut Ui<'_>,
    art: &CreatureArt,
    lang: &Lang,
    code: &str,
    c: &Content,
    save: &SaveGame,
    screen: Vec2,
) {
    let s = ui.s;
    // Level
    let center = Vec2::new(
        (screen.x - BAR_WIDTH * s) / 2.0 - 34.0 * s,
        screen.y - (BAR_MARGIN + BAR_HEIGHT / 2.0) * s,
    );
    ui.batch.fill_circle(center, 25.0 * s, PANEL);
    let need = c.xp_to_next(save.level).max(1);
    #[allow(clippy::cast_precision_loss)]
    let frac = if save.level >= c.progression.max_level {
        1.0
    } else {
        (save.xp as f32 / need as f32).clamp(0.0, 1.0)
    };
    let top = -std::f32::consts::FRAC_PI_2;
    ui.batch.stroke_polyline(
        &arc(center, 21.0 * s, top, top + std::f32::consts::TAU),
        4.0 * s,
        Color::rgba(1.0, 1.0, 1.0, 0.2),
    );
    if frac > 0.0 {
        ui.batch.stroke_polyline(
            &arc(center, 21.0 * s, top, top + std::f32::consts::TAU * frac),
            4.0 * s,
            XP,
        );
    }
    ui.batch.fill_circle(center, 16.0 * s, GOLD);
    ui.label(
        &save.level.to_string(),
        center,
        15.0,
        ui::TEXT,
        Align::Center,
    );

    // Gleam drops
    let pill = Rect::new(14.0 * s, 14.0 * s, 104.0 * s, 32.0 * s);
    ui.batch
        .fill_rounded_rect(pill.min, pill.max, 16.0 * s, PANEL);
    art.draw_loot_icon(
        ui.batch,
        "gleam_drops",
        pill.min + Vec2::new(19.0 * s, 17.0 * s),
        1.1 * s,
    );
    ui.label(
        &save.gleam_drops.to_string(),
        Vec2::new(pill.min.x + 38.0 * s, pill.center().y),
        14.0,
        WHITE,
        Align::Left,
    );

    // Quest: main quest first, otherwise the first active one
    let active = c
        .quests
        .iter()
        .filter(|q| {
            save.quest(&q.id)
                .is_some_and(|st| st.status == QuestStatus::Active)
        })
        .min_by_key(|q| q.kind != QuestKind::Main);
    if let Some(q) = active
        && let Some(st) = save.quest(&q.id)
        && let Some(step) = q.step.get(st.step)
    {
        let mut line = step.text.get(code).to_owned();
        if let Goal::Defeat { count, .. } = step.goal {
            line = format!("{line} ({}/{count})", st.progress);
        }
        let w = (ui.text_width(&line, 11.0) + 50.0 * s)
            .max(ui.text_width(q.name.get(code), 13.0) + 30.0 * s)
            .min(screen.x * 0.4);
        let card = Rect::new(screen.x - w - 14.0 * s, 14.0 * s, w, 66.0 * s);
        ui.batch
            .fill_rounded_rect(card.min, card.max, 12.0 * s, PANEL);
        let x = card.min.x + 14.0 * s;
        ui.label(
            lang.t("adventure.quest_label"),
            Vec2::new(x, card.min.y + 15.0 * s),
            9.0,
            GOLD,
            Align::Left,
        );
        ui.label(
            q.name.get(code),
            Vec2::new(x, card.min.y + 33.0 * s),
            13.0,
            WHITE,
            Align::Left,
        );
        let box_min = Vec2::new(x, card.min.y + 46.0 * s);
        ui.batch.fill_rounded_rect(
            box_min,
            box_min + Vec2::new(10.0 * s, 10.0 * s),
            2.0 * s,
            WHITE_DIM,
        );
        ui.batch.fill_rounded_rect(
            box_min + Vec2::new(1.5 * s, 1.5 * s),
            box_min + Vec2::new(8.5 * s, 8.5 * s),
            1.5 * s,
            PANEL,
        );
        ui.label(
            &line,
            Vec2::new(x + 16.0 * s, card.min.y + 51.0 * s),
            11.0,
            WHITE_DIM,
            Align::Left,
        );
    }
}

/// Heat bar below the gleam drops (E-320): sun and fill level from yellow to red;
/// when full (Elora is slower) it pulses until it drops below half again.
pub fn heat_bar(ui: &mut Ui<'_>, heat: f32, overheated: bool, time: f32) {
    temperature_bar(ui, heat, overheated, time, false);
}

/// Cold bar (E-342): snowflake and fill level from light blue to deep blue; pulses when full.
pub fn cold_bar(ui: &mut Ui<'_>, cold: f32, frozen: bool, time: f32) {
    temperature_bar(ui, cold, frozen, time, true);
}

fn temperature_bar(ui: &mut Ui<'_>, value: f32, full: bool, time: f32, cold: bool) {
    let s = ui.s;
    let pill = Rect::new(14.0 * s, 52.0 * s, 104.0 * s, 24.0 * s);
    ui.batch
        .fill_rounded_rect(pill.min, pill.max, 12.0 * s, PANEL);
    let icon = pill.min + Vec2::new(15.0 * s, 12.0 * s);
    if cold {
        // snowflake, rotates slowly
        let c = Color::hex(0xbfe6f5);
        for k in 0..6 {
            #[allow(clippy::cast_precision_loss)]
            let a = k as f32 * std::f32::consts::TAU / 6.0 + time * 0.3;
            let d = Vec2::new(a.cos(), a.sin());
            ui.batch.stroke_line(icon, icon + d * (9.0 * s), 1.8 * s, c);
            let b = icon + d * (5.5 * s);
            let n = Vec2::new(-d.y, d.x);
            ui.batch.stroke_line(b, b + (d + n) * (2.4 * s), 1.4 * s, c);
            ui.batch.stroke_line(b, b + (d - n) * (2.4 * s), 1.4 * s, c);
        }
    } else {
        // sun
        for k in 0..8 {
            #[allow(clippy::cast_precision_loss)]
            let a = k as f32 * std::f32::consts::TAU / 8.0 + time * 0.6;
            let d = Vec2::new(a.cos(), a.sin());
            ui.batch
                .stroke_line(icon + d * (7.0 * s), icon + d * (10.0 * s), 1.6 * s, GOLD);
        }
        ui.batch.fill_circle(icon, 5.5 * s, GOLD);
    }
    // fill level
    let (x0, x1) = (pill.min.x + 30.0 * s, pill.max.x - 10.0 * s);
    let (y0, y1) = (pill.min.y + 8.0 * s, pill.max.y - 8.0 * s);
    let r = (y1 - y0) / 2.0;
    ui.batch.fill_rounded_rect(
        Vec2::new(x0, y0),
        Vec2::new(x1, y1),
        r,
        Color::rgba(1.0, 1.0, 1.0, 0.2),
    );
    let h = value.clamp(0.0, 1.0);
    if h > 0.0 {
        let mix = |a: f32, b: f32| a + (b - a) * h;
        let (strong, mild) = if cold {
            (Color::hex(0x3f7fc8).0, Color::hex(0xbfe6f5).0)
        } else {
            (Color::hex(0xe8685a).0, GOLD.0)
        };
        let mut c = Color::rgb(
            mix(mild[0], strong[0]),
            mix(mild[1], strong[1]),
            mix(mild[2], strong[2]),
        );
        if full {
            c.0[3] = 0.65 + 0.35 * (time * 8.0).sin();
        }
        ui.batch.fill_rounded_rect(
            Vec2::new(x0, y0),
            Vec2::new((x0 + (x1 - x0) * h).max(x0 + 2.0 * r), y1),
            r,
            c,
        );
    }
}

/// Content of the victory screen after a guardian.
pub struct VictoryView<'a> {
    pub chapter: u32,
    pub area: &'a str,
    pub line: &'a str,
    pub honor: &'a str,
    pub color: Color,
    /// Colours of the five springs, `None` = still silent.
    pub springs: [Option<Color>; 5],
    pub stats: &'a str,
    /// Seconds since appearing.
    pub time: f32,
}

/// From then on the victory screen can be closed (seconds).
pub const VICTORY_READY: f32 = 1.2;

fn ease_out_back(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    let (c1, c3) = (1.70158, 2.70158);
    1.0 + c3 * (x - 1.0).powi(3) + c1 * (x - 1.0).powi(2)
}

/// Victory screen “Chapter X complete!”: rotating sun rays in the colour of the region,
/// confetti, the five springs as drops, honorary title. Returns `true` when “Continue” is pressed.
#[allow(clippy::too_many_lines)] // one scene in one piece
pub fn victory(ui: &mut Ui<'_>, lang: &Lang, v: &VictoryView<'_>, screen: Vec2) -> bool {
    use std::f32::consts::TAU;
    let s = ui.s;
    let t = v.time;
    let fade = (t * 3.0).min(1.0);
    ui.batch.fill_rect(
        Vec2::ZERO,
        screen,
        Color::rgba(0.07, 0.09, 0.13, 0.62 * fade),
    );
    let center = Vec2::new(screen.x / 2.0, screen.y * 0.44);
    // sun rays
    let reach = screen.length();
    let rays = 18;
    for k in 0..rays {
        #[allow(clippy::cast_precision_loss)]
        let a = k as f32 / rays as f32 * TAU + t * 0.22;
        let w = TAU / rays as f32 * 0.28;
        let dir = |a: f32| Vec2::new(a.cos(), a.sin()) * reach;
        let mut c = v.color;
        c.0[3] = if k % 2 == 0 { 0.22 } else { 0.12 } * fade;
        ui.batch
            .fill_polygon(&[center, center + dir(a - w), center + dir(a + w)], c);
    }
    let mut glow = v.color;
    glow.0[3] = 0.25 * fade;
    ui.batch.fill_circle(center, 230.0 * s, glow);
    // confetti
    let palette = [
        v.color,
        GOLD,
        Color::hex(0xef7fb0),
        Color::hex(0x5aaee8),
        Color::hex(0x7fd99a),
    ];
    for i in 0..70u32 {
        let h = i.wrapping_mul(2_654_435_761).rotate_left(i % 13);
        // own random number 0..1 per size (previously the bits for x only reached the middle)
        let r = |k: u32| {
            let mut z = h ^ k.wrapping_mul(0x9e37_79b9);
            z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b);
            z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35);
            z ^= z >> 16;
            #[allow(clippy::cast_precision_loss)]
            let v = (z & 0xffff) as f32 / 65535.0;
            v
        };
        let speed = 0.12 + r(3) * 0.18;
        let y = ((t * speed + r(13)) % 1.15 - 0.08) * screen.y;
        let x = r(23) * screen.x + (t * (1.0 + r(5) * 2.0) + r(7) * 6.0).sin() * 18.0 * s;
        let ang = t * (2.0 + r(9) * 4.0) + r(11) * TAU;
        let (dx, dy) = (
            Vec2::new(ang.cos(), ang.sin()) * 5.0 * s,
            Vec2::new(-ang.sin(), ang.cos()) * 2.5 * s,
        );
        let p = Vec2::new(x, y);
        let mut c = palette[(h % 5) as usize];
        c.0[3] = fade;
        ui.batch
            .fill_polygon(&[p - dx - dy, p + dx - dy, p + dx + dy, p - dx + dy], c);
    }
    // card
    let (w, h) = (560.0 * s, 400.0 * s);
    let card = Rect::new(center.x - w / 2.0, center.y - h / 2.0, w, h);
    ui.card(card);
    let x = card.center().x;
    let pop = ease_out_back(t / 0.7);
    ui.label(
        &lang.f("victory.title", &[("n", &v.chapter)]),
        Vec2::new(x, card.min.y + 52.0 * s),
        (30.0 * pop).max(1.0),
        ui::TEXT,
        Align::Center,
    );
    ui.label(
        v.area,
        Vec2::new(x, card.min.y + 88.0 * s),
        15.0,
        v.color,
        Align::Center,
    );
    let mut y = card.min.y + 120.0 * s;
    for line in wrap(ui, v.line, 13.0, w - 80.0 * s) {
        ui.label(&line, Vec2::new(x, y), 13.0, ui::TEXT_DIM, Align::Center);
        y += 20.0 * s;
    }
    // the five springs
    let dy = card.min.y + 210.0 * s;
    for (k, spring) in v.springs.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let cx = x + (k as f32 - 2.0) * 54.0 * s;
        let bob = if spring.is_some() {
            (t * 3.0 + k as f32).sin() * 3.0 * s
        } else {
            0.0
        };
        let c = spring.unwrap_or(Color::hex(0xc8ccd2));
        let base = Vec2::new(cx, dy + bob);
        ui.batch.fill_circle(base, 15.0 * s, ui::OUTLINE);
        ui.batch.fill_polygon(
            &[
                base + Vec2::new(-12.5 * s, -6.0 * s),
                base + Vec2::new(0.0, -30.0 * s),
                base + Vec2::new(12.5 * s, -6.0 * s),
            ],
            ui::OUTLINE,
        );
        ui.batch.fill_circle(base, 12.0 * s, c);
        ui.batch.fill_polygon(
            &[
                base + Vec2::new(-10.0 * s, -6.0 * s),
                base + Vec2::new(0.0, -25.0 * s),
                base + Vec2::new(10.0 * s, -6.0 * s),
            ],
            c,
        );
        if spring.is_some() {
            ui.batch.fill_circle(
                base + Vec2::new(-4.0 * s, -4.0 * s),
                3.0 * s,
                Color::rgba(1.0, 1.0, 1.0, 0.8),
            );
        }
    }
    let freed = v.springs.iter().filter(|c| c.is_some()).count();
    ui.label(
        &lang.f("victory.springs", &[("n", &freed)]),
        Vec2::new(x, dy + 34.0 * s),
        12.0,
        ui::TEXT_DIM,
        Align::Center,
    );
    // honorary title
    let badge_text = format!("{}: {}", lang.t("victory.honor"), v.honor);
    let bw = ui.text_width(&badge_text, 13.0) + 36.0 * s;
    let badge = Rect::new(x - bw / 2.0, dy + 54.0 * s, bw, 30.0 * s);
    ui.batch
        .fill_rounded_rect(badge.min, badge.max, 15.0 * s, ui::OUTLINE);
    ui.batch.fill_rounded_rect(
        badge.min + Vec2::new(2.5 * s, 2.5 * s),
        badge.max - Vec2::new(2.5 * s, 2.5 * s),
        12.5 * s,
        GOLD,
    );
    ui.label(&badge_text, badge.center(), 13.0, ui::TEXT, Align::Center);
    ui.label(
        v.stats,
        Vec2::new(x, dy + 104.0 * s),
        11.0,
        ui::TEXT_DIM,
        Align::Center,
    );
    ui.label(
        lang.t("victory.next"),
        Vec2::new(x, dy + 124.0 * s),
        11.0,
        ui::TEXT_DIM,
        Align::Center,
    );
    // Continue (after a moment, so nobody clicks the screen away)
    if t >= VICTORY_READY {
        let b = Rect::new(x - 90.0 * s, card.max.y - 46.0 * s, 180.0 * s, 32.0 * s);
        return ui.button("victory_continue", b, lang.t("victory.continue"), ui::GREEN);
    }
    false
}

/// Replace the placeholder `{taste:<aktion>}` with the bound key (signs, E-273),
/// e.g. `{taste:jump}` → “Space”.
/// Health bar of a guardian at the top centre with name (R2-M2.1); `frac` 0..1.
pub fn boss_bar(ui: &mut Ui<'_>, name: &str, frac: f32, screen: Vec2) {
    let s = ui.s;
    let w = (screen.x * 0.5).min(520.0 * s);
    let bar = Rect::new((screen.x - w) / 2.0, 54.0 * s, w, 16.0 * s);
    ui.label(
        name,
        Vec2::new(screen.x / 2.0, bar.min.y - 12.0 * s),
        14.0,
        WHITE,
        Align::Center,
    );
    let pad = Vec2::new(3.0 * s, 3.0 * s);
    ui.batch
        .fill_rounded_rect(bar.min - pad, bar.max + pad, 11.0 * s, PANEL);
    ui.batch
        .fill_rounded_rect(bar.min, bar.max, 8.0 * s, Color::rgba(1.0, 1.0, 1.0, 0.15));
    let f = frac.clamp(0.0, 1.0);
    if f > 0.0 {
        ui.batch.fill_rounded_rect(
            bar.min,
            Vec2::new(bar.min.x + w * f, bar.max.y),
            8.0 * s,
            BOSS,
        );
    }
}

pub fn with_keys(text: &str, keys: &crate::bindings::Bindings, lang: &Lang) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("{taste:") {
        out.push_str(&rest[..i]);
        let after = &rest[i + 7..];
        let Some(end) = after.find('}') else {
            out.push_str(&rest[i..]);
            return out;
        };
        match crate::bindings::GameAction::from_name(&after[..end]) {
            Some(a) => out.push_str(&keys.trigger(a).label(lang)),
            None => out.push_str(&rest[i..i + 8 + end]),
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Draw lines of which only `shown` characters are visible; returns the height below.
fn revealed_lines(ui: &mut Ui<'_>, lines: &[String], shown: usize, at: Vec2) -> f32 {
    let mut y = at.y;
    let mut left = shown;
    for line in lines {
        let n = line.chars().count();
        if left >= n {
            ui.label(line, Vec2::new(at.x, y), 13.0, ui::TEXT, Align::Left);
        } else if left > 0 {
            let part: String = line.chars().take(left).collect();
            ui.label(&part, Vec2::new(at.x, y), 13.0, ui::TEXT, Align::Left);
        }
        // wrapping swallows the space between the lines
        left = left.saturating_sub(n + 1);
        y += 19.0 * ui.s;
    }
    y
}

/// Line wrapping at word boundaries for the width `max` (pixels).
pub fn wrap(ui: &Ui<'_>, text: &str, size: f32, max: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let next = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if ui.text_width(&next, size) > max && !line.is_empty() {
            lines.push(std::mem::replace(&mut line, word.to_owned()));
        } else {
            line = next;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// What the conversation box shows.
pub struct DialogView<'a> {
    /// Character (graphics and colour of the name tag); `elora` for Elora.
    pub speaker: &'a str,
    pub name: &'a str,
    pub text: &'a Text,
    /// Visible answers (index in the node, answer).
    pub choices: Vec<(usize, &'a Choice)>,
    /// This many characters of the text are already visible; answers only once everything is there.
    pub shown: usize,
}

fn tone_color(t: Option<Tone>) -> Color {
    match t {
        Some(Tone::Friendly) => TONE_FRIENDLY,
        Some(Tone::Curious) => TONE_CURIOUS,
        Some(Tone::Cheeky) => TONE_CHEEKY,
        None => Color::hex(0xc8bca8),
    }
}

/// Colour of the name tag per character (from the drafts).
pub fn name_color(speaker: &str) -> Color {
    match speaker {
        "oma" => Color::hex(0xa77be0),
        "klonk" => Color::hex(0xa8744a),
        "lotte" => Color::hex(0x5aaee8),
        "tueftel" => Color::hex(0x3fc1b0),
        "pip" => Color::hex(0xf28c3a),
        _ => Color::hex(0xe8a53a),
    }
}

/// Conversation box at the bottom (E-222): portrait, name, text, answers with tone. Returns
/// the number of the clicked answer (order in `choices`).
pub fn dialog(
    ui: &mut Ui<'_>,
    art: &CreatureArt,
    lang: &Lang,
    code: &str,
    v: &DialogView<'_>,
    screen: Vec2,
    keys: &crate::bindings::Bindings,
) -> Option<usize> {
    let s = ui.s;
    let w = (screen.x - 48.0 * s).min(920.0 * s);
    let text_w = w - 170.0 * s;
    let lines = wrap(ui, &with_keys(v.text.get(code), keys, lang), 13.0, text_w);
    #[allow(clippy::cast_precision_loss)]
    let h =
        (54.0 + lines.len() as f32 * 19.0 + v.choices.len() as f32 * 30.0 + 26.0).max(170.0) * s;
    let card = Rect::new((screen.x - w) / 2.0, screen.y - h - 20.0 * s, w, h);
    ui.card(card);

    // portrait of the character in a circle, the name tag below it
    let pc = Vec2::new(card.min.x + 76.0 * s, card.min.y + 70.0 * s);
    ui.batch.fill_circle(pc, 52.0 * s, ui::CARD_EDGE);
    ui.batch
        .fill_circle(pc, 50.0 * s, Color::rgba(0.95, 0.76, 0.31, 0.16));
    let ground = pc + Vec2::new(-4.0 * s, 46.0 * s);
    if !art.draw_character(ui.batch, v.speaker, ground, 1, 1.75 * s) {
        art.draw_loot_icon(ui.batch, "gleam_drops", pc, 2.5 * s);
    }
    let tag_w = ui.text_width(v.name, 11.0) + 24.0 * s;
    let tag = Rect::new(pc.x - tag_w / 2.0, pc.y + 56.0 * s, tag_w, 22.0 * s);
    ui.batch.fill_rounded_rect(
        tag.min - Vec2::new(1.5 * s, 1.5 * s),
        tag.max + Vec2::new(1.5 * s, 1.5 * s),
        12.0 * s,
        ui::OUTLINE,
    );
    ui.batch
        .fill_rounded_rect(tag.min, tag.max, 11.0 * s, name_color(v.speaker));
    ui.label(
        v.name,
        tag.center(),
        11.0,
        Color::rgb(1.0, 1.0, 1.0),
        Align::Center,
    );

    // text
    let x = card.min.x + 150.0 * s;
    let mut y = revealed_lines(ui, &lines, v.shown, Vec2::new(x, card.min.y + 34.0 * s));
    let complete = v.shown >= with_keys(v.text.get(code), keys, lang).chars().count();

    // answers
    let mut chosen = None;
    y += 8.0 * s;
    let visible = if complete { v.choices.len() } else { 0 };
    for (k, (_, c)) in v.choices.iter().enumerate().take(visible) {
        let r = Rect::new(x - 8.0 * s, y - 13.0 * s, text_w + 16.0 * s, 26.0 * s);
        if ui.hovered(r) {
            ui.batch
                .fill_rounded_rect(r.min, r.max, 13.0 * s, Color::rgba(0.95, 0.76, 0.31, 0.35));
        }
        if ui.click(&format!("dlg{k}"), r) {
            chosen = Some(k);
        }
        let dot = Vec2::new(x + 6.0 * s, y);
        ui.batch.fill_circle(dot, 6.5 * s, ui::OUTLINE);
        ui.batch.fill_circle(dot, 5.0 * s, tone_color(c.tone));
        ui.label(
            &(k + 1).to_string(),
            Vec2::new(x + 20.0 * s, y),
            11.0,
            ui::TEXT_DIM,
            Align::Left,
        );
        ui.label(
            &with_keys(c.text.get(code), keys, lang),
            Vec2::new(x + 34.0 * s, y),
            12.0,
            ui::TEXT,
            Align::Left,
        );
        if let Some(t) = c.tone {
            let key = match t {
                Tone::Friendly => "adventure.tone_friendly",
                Tone::Curious => "adventure.tone_curious",
                Tone::Cheeky => "adventure.tone_cheeky",
            };
            ui.label(
                lang.t(key),
                Vec2::new(r.max.x - 10.0 * s, y),
                10.0,
                ui::TEXT_DIM,
                Align::Right,
            );
        }
        y += 30.0 * s;
    }
    let hint = if v.choices.is_empty() {
        lang.t("adventure.dialog_hint_continue")
    } else {
        lang.t("adventure.dialog_hint_choose")
    };
    ui.label(
        hint,
        Vec2::new(card.max.x - 18.0 * s, card.max.y - 12.0 * s),
        9.0,
        ui::TEXT_DIM,
        Align::Right,
    );
    chosen
}

/// Speech bubble with tail, bottom centre at `p` (shouts, hints).
pub fn bubble(ui: &mut Ui<'_>, text: &str, p: Vec2) {
    let s = ui.s;
    let w = ui.text_width(text, 12.0) + 22.0 * s;
    let r = Rect::new(p.x - w / 2.0, p.y - 32.0 * s, w, 24.0 * s);
    let edge = 1.8 * s;
    ui.batch.fill_rounded_rect(
        r.min - Vec2::new(edge, edge),
        r.max + Vec2::new(edge, edge),
        12.0 * s + edge,
        ui::OUTLINE,
    );
    // tail
    let tip = [
        Vec2::new(p.x - 6.0 * s, r.max.y - 1.0 * s),
        Vec2::new(p.x, p.y - 1.0 * s),
        Vec2::new(p.x + 5.0 * s, r.max.y - 1.0 * s),
    ];
    ui.batch.stroke_polyline(&tip, 3.6 * s, ui::OUTLINE);
    ui.batch
        .fill_rounded_rect(r.min, r.max, 12.0 * s, ui::FIELD);
    ui.batch.stroke_polyline(&tip, 1.2 * s, ui::FIELD);
    ui.label(text, r.center(), 12.0, ui::TEXT, Align::Center);
}

/// Hint for the action key: key box “E” and text on a dark surface.
pub fn prompt(ui: &mut Ui<'_>, key: &str, text: &str, p: Vec2) {
    let s = ui.s;
    let w = ui.text_width(text, 12.0) + 44.0 * s;
    let r = Rect::new(p.x - w / 2.0, p.y - 30.0 * s, w, 26.0 * s);
    ui.batch.fill_rounded_rect(r.min, r.max, 13.0 * s, PANEL);
    let k = Rect::new(r.min.x + 6.0 * s, r.min.y + 4.0 * s, 18.0 * s, 18.0 * s);
    ui.batch.fill_rounded_rect(k.min, k.max, 5.0 * s, ui::FIELD);
    ui.label(key, k.center(), 11.0, ui::TEXT, Align::Center);
    ui.label(
        text,
        Vec2::new(k.max.x + 8.0 * s, r.center().y),
        12.0,
        WHITE,
        Align::Left,
    );
}

/// Name tag above a character (when stepping close, in addition to the hint).
pub fn name_tag(ui: &mut Ui<'_>, name: &str, speaker: &str, p: Vec2) {
    let s = ui.s;
    let w = ui.text_width(name, 11.0) + 22.0 * s;
    let r = Rect::new(p.x - w / 2.0, p.y - 22.0 * s, w, 20.0 * s);
    let edge = 1.5 * s;
    ui.batch.fill_rounded_rect(
        r.min - Vec2::new(edge, edge),
        r.max + Vec2::new(edge, edge),
        10.0 * s + edge,
        ui::OUTLINE,
    );
    ui.batch
        .fill_rounded_rect(r.min, r.max, 10.0 * s, name_color(speaker));
    ui.label(
        name,
        r.center(),
        11.0,
        Color::rgb(1.0, 1.0, 1.0),
        Align::Center,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::Language;
    use elora_adventure::{Conversation, Location};
    use elora_render::{Font, ShapeBatch};

    #[test]
    fn key_placeholders_show_the_bound_key() {
        let lang = Lang::new(Language::De);
        let keys = crate::bindings::Bindings::default();
        let interact = keys
            .trigger(crate::bindings::GameAction::Interact)
            .label(&lang);
        assert_eq!(
            with_keys("Drück {taste:interact}!", &keys, &lang),
            format!("Drück {interact}!")
        );
        assert_eq!(
            with_keys("{taste:gibtsnicht} a", &keys, &lang),
            "{taste:gibtsnicht} a"
        );
        assert_eq!(with_keys("offen {taste:", &keys, &lang), "offen {taste:");
    }

    /// Visual check: `cargo test -p elora-client --bin elora adventure_hud_sheet -- --ignored`,
    /// then `cargo xtask svg-preview target/abenteuer-hud.svg target/abenteuer-hud.png 1280`.
    #[test]
    #[ignore = "only writes a file for visual inspection"]
    fn victory_sheet() {
        let font = Font::new(include_bytes!("../../../assets/fonts/Inter-Regular.ttf")).unwrap();
        let lang = Lang::new(Language::De);
        let screen = Vec2::new(1280.0, 720.0);
        let mut batch = ShapeBatch::default();
        batch.fill_rect_vgradient(
            Vec2::ZERO,
            screen,
            Color::hex(0x8fbcdf),
            Color::hex(0xf6e2bf),
        );
        batch.fill_rect(Vec2::new(0.0, 560.0), screen, Color::hex(0xe0bf7c));
        let input = crate::ui::UiInput::default();
        let mut state = crate::ui::UiState::default();
        let mut ui = Ui {
            batch: &mut batch,
            font: &font,
            input: &input,
            state: &mut state,
            s: 1.0,
        };
        ui.begin(0.016);
        let gold = Color::hex(0xe0b85a);
        let view = VictoryView {
            chapter: 3,
            area: "Glutsandwüste",
            line: "Die Sandschlange döst friedlich in der Wärme – und die Glutquelle leuchtet golden wie Honig.",
            honor: "Wüstentänzerin",
            color: gold,
            springs: [
                Some(Color::hex(0xef7fb0)),
                Some(Color::hex(0x6cbf4a)),
                Some(gold),
                None,
                None,
            ],
            stats: "Stufe 17 · Spielzeit 0 h 40 min",
            time: 2.0,
        };
        victory(&mut ui, &lang, &view, screen);
        ui.end();
        let svg = batch.debug_svg(Vec2::ZERO, screen, Color::hex(0xa9cde8));
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/sieg.svg"),
            svg,
        )
        .unwrap();
    }

    #[test]
    #[ignore = "only writes a file for visual inspection"]
    fn adventure_hud_sheet() {
        let font = Font::new(include_bytes!("../../../assets/fonts/Inter-Regular.ttf")).unwrap();
        let lang = Lang::new(Language::De);
        let art = CreatureArt::load();
        let c = Content::builtin();
        let mut save = SaveGame::new(&c, Location::default());
        save.add_xp(&c, 70);
        save.add_item(&c, "gleam_drops", 128).unwrap();
        let (mut conv, _) = Conversation::start(&c, &mut save, "oma").unwrap();
        let screen = Vec2::new(1280.0, 720.0);
        let mut batch = ShapeBatch::default();
        batch.fill_rect_vgradient(
            Vec2::ZERO,
            screen,
            Color::hex(0xa9cde8),
            Color::hex(0xe8f1f7),
        );
        batch.fill_rect(Vec2::new(0.0, 560.0), screen, Color::hex(0x8fbf7a));
        let input = crate::ui::UiInput::default();
        let mut state = crate::ui::UiState::default();
        let mut ui = Ui {
            batch: &mut batch,
            font: &font,
            input: &input,
            state: &mut state,
            s: 1.0,
        };
        ui.begin(0.016);
        // start a conversation, pick an answer: quest is running (display top right)
        let choices: Vec<_> = conv.choices(&c, &save);
        let (d, node) = conv.current(&c).unwrap();
        let view = DialogView {
            speaker: d.speaker_of(node),
            name: "Oma Pfütze",
            text: &node.text,
            choices: choices.iter().map(|&i| (i, &node.choice[i])).collect(),
            shown: usize::MAX,
        };
        dialog(
            &mut ui,
            &art,
            &lang,
            "de",
            &view,
            screen,
            &crate::bindings::Bindings::default(),
        );
        conv.choose(&c, &mut save, 0);
        status(&mut ui, &art, &lang, "de", &c, &save, screen);
        heat_bar(&mut ui, 0.7, false, 1.0);
        cold_bar(&mut ui, 0.4, true, 1.0);
        bubble(&mut ui, "Hallo, Elora!", Vec2::new(300.0, 200.0));
        prompt(&mut ui, "E", "Sprechen", Vec2::new(600.0, 200.0));
        ui.end();
        let svg = batch.debug_svg(Vec2::ZERO, screen, Color::hex(0xa9cde8));
        std::fs::write(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../target/adventure-hud.svg"
            ),
            svg,
        )
        .unwrap();
    }
}

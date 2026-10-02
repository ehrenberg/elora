//! Anzeigen des Abenteuers (A1.7, E-222, E-225): Stufe mit Erfahrungsring, Glanztropfen,
//! aktuelle Aufgabe, Gesprächsfeld mit Bild und Auswahl, Sprechblasen, Hinweise.
//!
//! Alles in Bildschirm-Pixeln über [`crate::ui`]; Stil wie die Entwürfe
//! (`docs/release-2/design/abenteuer-ui.png`).

// Layout-Code: `s` (Skalierung), `w`/`h`/`x`/`y` wie in menu.rs
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

/// Dunkle HUD-Fläche wie die Leiste unten (hud.rs).
const PANEL: Color = Color::rgba(0.118, 0.165, 0.212, 0.6);
const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
const WHITE_DIM: Color = Color::rgba(1.0, 1.0, 1.0, 0.75);
const GOLD: Color = Color::hex(0xf2c14e);
const XP: Color = Color::hex(0x7fd99a);
const TONE_FRIENDLY: Color = Color::hex(0xf2c14e);
const TONE_CURIOUS: Color = Color::hex(0x5aaee8);
const TONE_CHEEKY: Color = Color::hex(0xe8685a);

/// Breite der Leiste unten (hud.rs), für das Stufen-Abzeichen links daneben.
const BAR_WIDTH: f32 = 300.0;
const BAR_HEIGHT: f32 = 54.0;
const BAR_MARGIN: f32 = 14.0;

/// Kreisbogen als Linienzug (Winkel im Bogenmaß, 0 = rechts, im Uhrzeigersinn).
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

/// Stufe mit Erfahrungsring links an der Leiste, Glanztropfen oben links, Aufgabe oben rechts.
#[allow(clippy::too_many_lines)] // drei kleine Anzeigen an einem Ort
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
    // Stufe
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

    // Glanztropfen
    let pill = Rect::new(14.0 * s, 14.0 * s, 104.0 * s, 32.0 * s);
    ui.batch
        .fill_rounded_rect(pill.min, pill.max, 16.0 * s, PANEL);
    art.draw_loot_icon(
        ui.batch,
        "glanztropfen",
        pill.min + Vec2::new(19.0 * s, 17.0 * s),
        1.1 * s,
    );
    ui.label(
        &save.glanztropfen.to_string(),
        Vec2::new(pill.min.x + 38.0 * s, pill.center().y),
        14.0,
        WHITE,
        Align::Left,
    );

    // Aufgabe: Hauptaufgabe zuerst, sonst die erste aktive
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

/// Zeilenumbruch an Wortgrenzen für die Breite `max` (Pixel).
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

/// Was das Gesprächsfeld zeigt.
pub struct DialogView<'a> {
    /// Figur (Grafik und Farbe des Namensschilds); `elora` für Elora.
    pub speaker: &'a str,
    pub name: &'a str,
    pub text: &'a Text,
    /// Sichtbare Antworten (Index im Knoten, Antwort).
    pub choices: Vec<(usize, &'a Choice)>,
}

fn tone_color(t: Option<Tone>) -> Color {
    match t {
        Some(Tone::Freundlich) => TONE_FRIENDLY,
        Some(Tone::Neugierig) => TONE_CURIOUS,
        Some(Tone::Frech) => TONE_CHEEKY,
        None => Color::hex(0xc8bca8),
    }
}

/// Farbe des Namensschilds je Figur (aus den Entwürfen).
fn name_color(speaker: &str) -> Color {
    match speaker {
        "oma" => Color::hex(0xa77be0),
        "klonk" => Color::hex(0xa8744a),
        "lotte" => Color::hex(0x5aaee8),
        "tueftel" => Color::hex(0x3fc1b0),
        "pip" => Color::hex(0xf28c3a),
        _ => Color::hex(0xe8a53a),
    }
}

/// Gesprächsfeld unten (E-222): Bild, Name, Text, Antworten mit Ton. Liefert die Nummer
/// der angeklickten Antwort (Reihenfolge in `choices`).
pub fn dialog(
    ui: &mut Ui<'_>,
    art: &CreatureArt,
    lang: &Lang,
    code: &str,
    v: &DialogView<'_>,
    screen: Vec2,
) -> Option<usize> {
    let s = ui.s;
    let w = (screen.x - 48.0 * s).min(920.0 * s);
    let text_w = w - 170.0 * s;
    let lines = wrap(ui, v.text.get(code), 13.0, text_w);
    #[allow(clippy::cast_precision_loss)]
    let h =
        (54.0 + lines.len() as f32 * 19.0 + v.choices.len() as f32 * 30.0 + 26.0).max(170.0) * s;
    let card = Rect::new((screen.x - w) / 2.0, screen.y - h - 20.0 * s, w, h);
    ui.card(card);

    // Bild der Figur im Kreis, darunter das Namensschild
    let pc = Vec2::new(card.min.x + 76.0 * s, card.min.y + 70.0 * s);
    ui.batch.fill_circle(pc, 52.0 * s, ui::CARD_EDGE);
    ui.batch
        .fill_circle(pc, 50.0 * s, Color::rgba(0.95, 0.76, 0.31, 0.16));
    let ground = pc + Vec2::new(-4.0 * s, 46.0 * s);
    if !art.draw_character(ui.batch, v.speaker, ground, 1, 1.75 * s) {
        art.draw_loot_icon(ui.batch, "glanztropfen", pc, 2.5 * s);
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

    // Text
    let x = card.min.x + 150.0 * s;
    let mut y = card.min.y + 34.0 * s;
    for line in &lines {
        ui.label(line, Vec2::new(x, y), 13.0, ui::TEXT, Align::Left);
        y += 19.0 * s;
    }

    // Antworten
    let mut chosen = None;
    y += 8.0 * s;
    for (k, (_, c)) in v.choices.iter().enumerate() {
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
            c.text.get(code),
            Vec2::new(x + 34.0 * s, y),
            12.0,
            ui::TEXT,
            Align::Left,
        );
        if let Some(t) = c.tone {
            let key = match t {
                Tone::Freundlich => "adventure.tone_friendly",
                Tone::Neugierig => "adventure.tone_curious",
                Tone::Frech => "adventure.tone_cheeky",
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

/// Sprechblase mit Zipfel, unten mittig an `p` (Zurufe, Hinweise).
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
    // Zipfel
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

/// Hinweis zur Aktionstaste: Tastenkästchen „E“ und Text auf dunkler Fläche.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::Language;
    use elora_adventure::{Conversation, Location};
    use elora_render::{Font, ShapeBatch};

    /// Sichtprüfung: `cargo test -p elora-client --bin elora adventure_hud_sheet -- --ignored`,
    /// dann `cargo xtask svg-preview target/abenteuer-hud.svg target/abenteuer-hud.png 1280`.
    #[test]
    #[ignore = "erzeugt nur eine Datei zur Sichtprüfung"]
    fn adventure_hud_sheet() {
        let font = Font::new(include_bytes!("../../../assets/fonts/Inter-Regular.ttf")).unwrap();
        let lang = Lang::new(Language::De);
        let art = CreatureArt::load();
        let c = Content::builtin();
        let mut save = SaveGame::new(&c, Location::default());
        save.add_xp(&c, 70);
        save.add_item(&c, "glanztropfen", 128).unwrap();
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
        // Gespräch beginnen, eine Antwort wählen: Aufgabe läuft (Anzeige oben rechts)
        let choices: Vec<_> = conv.choices(&c, &save);
        let (d, node) = conv.current(&c).unwrap();
        let view = DialogView {
            speaker: d.speaker_of(node),
            name: "Oma Pfütze",
            text: &node.text,
            choices: choices.iter().map(|&i| (i, &node.choice[i])).collect(),
        };
        dialog(&mut ui, &art, &lang, "de", &view, screen);
        conv.choose(&c, &mut save, 0);
        status(&mut ui, &art, &lang, "de", &c, &save, screen);
        bubble(&mut ui, "Hallo, Elora!", Vec2::new(300.0, 200.0));
        prompt(&mut ui, "E", "Sprechen", Vec2::new(600.0, 200.0));
        ui.end();
        let svg = batch.debug_svg(Vec2::ZERO, screen, Color::hex(0xa9cde8));
        std::fs::write(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../target/abenteuer-hud.svg"
            ),
            svg,
        )
        .unwrap();
    }
}

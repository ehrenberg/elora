//! UI-Toolkit der Spiel-UI (M7.1, E-031) im Stil „hell & weich“ (E-125).
//!
//! Sofortmodus: Jede Seite ruft pro Frame die Widgets auf; sie zeichnen sich in
//! einen [`ShapeBatch`] (Bildschirm-Pixel) und melden Klicks und Änderungen direkt
//! zurück. Der Zustand zwischen Frames (Fokus, gedrücktes Widget, Scroll-Positionen)
//! liegt in [`UiState`]; Widgets werden über eine Kennung (`id`) wiedererkannt.
//!
//! Maße gelten für 720 px Fensterhöhe und werden mit [`Ui::s`] skaliert.

// Wird mit dem Hauptmenü (M7.3) eingebunden; bis dahin nur von Tests benutzt.
#![allow(dead_code)]

use std::collections::HashMap;

use elora_render::{Align, Color, Font, ShapeBatch, lerp_color, shade};
use elora_sim::Vec2;

// ── Thema (E-125) ────────────────────────────────────────────────────────

pub const CARD: Color = Color::hex(0xfffaf0);
pub const CARD_EDGE: Color = Color::hex(0xe7dcc8);
pub const SHADOW: Color = Color::rgba(0.118, 0.165, 0.212, 0.12);
pub const TEXT: Color = Color::hex(0x3b3024);
pub const TEXT_DIM: Color = Color::hex(0x8a7a66);
pub const OUTLINE: Color = Color::hex(0x2b2b2b);
pub const FIELD: Color = Color::hex(0xffffff);
pub const HIGHLIGHT: Color = Color::rgba(0.949, 0.757, 0.306, 0.35);
pub const GREEN: Color = Color::hex(0x6cbf4a);
pub const BLUE: Color = Color::hex(0x5aaee8);
pub const VIOLET: Color = Color::hex(0xa77be0);
pub const ORANGE: Color = Color::hex(0xf28c3a);
pub const GRAY: Color = Color::hex(0x9aa4ae);
pub const SAND: Color = Color::hex(0xe0c89a);
pub const LOGO: Color = Color::hex(0xe8a53a);
const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);

/// Lesbare Schriftfarbe auf `bg`: dunkel auf hellen Farben (z. B. Sand), sonst weiß.
pub fn on_color(bg: Color) -> Color {
    let [r, g, b, _] = bg.0;
    let luminance = 0.299 * r + 0.587 * g + 0.114 * b;
    if luminance > 0.7 { TEXT } else { WHITE }
}

/// Achsenparalleles Rechteck in Bildschirm-Pixeln.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            min: Vec2::new(x, y),
            max: Vec2::new(x + w, y + h),
        }
    }

    pub fn w(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn h(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.min.x && p.x < self.max.x && p.y >= self.min.y && p.y < self.max.y
    }

    /// Nach innen verkleinert (negativ = vergrößert).
    #[must_use]
    pub fn shrink(&self, d: f32) -> Self {
        Self {
            min: self.min + Vec2::new(d, d),
            max: self.max - Vec2::new(d, d),
        }
    }

    #[must_use]
    pub fn offset(&self, d: Vec2) -> Self {
        Self {
            min: self.min + d,
            max: self.max + d,
        }
    }
}

/// Tasten, die Widgets auswerten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiKey {
    Enter,
    Escape,
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    Tab,
}

/// Eingaben eines Frames (vom Client aus den Fenster-Ereignissen gesammelt).
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)] // Zustände der linken Maustaste, wie vom Fenster gemeldet
pub struct UiInput {
    pub mouse: Vec2,
    /// Linke Maustaste gehalten / in diesem Frame gedrückt / losgelassen.
    pub down: bool,
    pub pressed: bool,
    pub released: bool,
    /// Zweiter Klick kurz nach dem ersten an fast gleicher Stelle.
    pub double_click: bool,
    /// Mausrad in Rasten (positiv = nach oben).
    pub scroll: f32,
    /// Getippte Zeichen.
    pub text: String,
    pub keys: Vec<UiKey>,
}

impl UiInput {
    /// Zustand für den nächsten Frame: Einmal-Ereignisse löschen, Halten behalten.
    pub fn next_frame(&mut self) {
        self.pressed = false;
        self.released = false;
        self.double_click = false;
        self.scroll = 0.0;
        self.text.clear();
        self.keys.clear();
    }

    pub fn key(&self, k: UiKey) -> bool {
        self.keys.contains(&k)
    }
}

/// Zustand zwischen Frames.
#[derive(Debug, Default)]
pub struct UiState {
    /// Widget mit Tastaturfokus (Textfeld).
    pub focus: Option<String>,
    /// Widget, auf dem die Maus gedrückt wurde.
    active: Option<String>,
    /// Cursor im fokussierten Textfeld (Zeichen).
    cursor: usize,
    /// Scroll-Position je Liste (Zeilen).
    scroll: HashMap<String, f32>,
    /// Hat in diesem Frame ein Widget den Mausdruck angenommen?
    claimed: bool,
    /// Laufzeit (blinkender Cursor).
    pub time: f32,
    /// Klänge der Oberfläche seit dem letzten Abholen (E-285).
    pub sounds: Vec<elora_audio::Sound>,
}

/// Zeichen- und Eingabekontext eines Frames.
pub struct Ui<'a> {
    pub batch: &'a mut ShapeBatch,
    pub font: &'a Font,
    pub input: &'a UiInput,
    pub state: &'a mut UiState,
    /// Skalierung (1 = 720 px Höhe).
    pub s: f32,
}

/// Rückmeldung eines Textfelds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldEvent {
    None,
    Changed,
    /// Enter gedrückt.
    Submitted,
}

impl Ui<'_> {
    /// Zu Beginn jeder Seite aufrufen.
    pub fn begin(&mut self, dt: f32) {
        self.state.claimed = false;
        self.state.time += dt;
    }

    /// Am Ende jeder Seite: Klick ins Leere nimmt den Fokus weg.
    pub fn end(&mut self) {
        if self.input.pressed && !self.state.claimed {
            self.state.focus = None;
        }
        if self.input.released {
            self.state.active = None;
        }
    }

    pub fn hovered(&self, r: Rect) -> bool {
        r.contains(self.input.mouse)
    }

    /// Gemeinsame Klick-Logik: `true`, wenn auf `r` gedrückt und dort losgelassen wurde.
    pub fn click(&mut self, id: &str, r: Rect) -> bool {
        let hot = self.hovered(r);
        if hot && self.input.pressed {
            self.state.active = Some(id.to_owned());
            self.state.claimed = true;
        }
        let was_active = self.state.active.as_deref() == Some(id);
        let clicked = was_active && hot && self.input.released;
        if clicked {
            self.state.sounds.push(elora_audio::Sound::UiClick);
        }
        clicked
    }

    fn pressing(&self, id: &str) -> bool {
        self.input.down && self.state.active.as_deref() == Some(id)
    }

    // ── Zeichnen ────────────────────────────────────────────────────────

    /// Karte: cremefarbene Fläche mit Kante und weichem Schatten.
    pub fn card(&mut self, r: Rect) {
        let s = self.s;
        self.batch.fill_rounded_rect(
            r.min + Vec2::new(3.0, 5.0) * s,
            r.max + Vec2::new(3.0, 5.0) * s,
            18.0 * s,
            SHADOW,
        );
        self.batch.fill_rounded_rect(
            r.min - Vec2::new(1.5, 1.5) * s,
            r.max + Vec2::new(1.5, 1.5) * s,
            19.0 * s,
            CARD_EDGE,
        );
        self.batch.fill_rounded_rect(r.min, r.max, 18.0 * s, CARD);
    }

    /// Text; `pos.y` ist die Mitte der Zeile.
    pub fn label(&mut self, text: &str, pos: Vec2, size: f32, color: Color, align: Align) {
        self.font
            .draw_centered(self.batch, text, pos, size * self.s, color, align);
    }

    pub fn text_width(&self, text: &str, size: f32) -> f32 {
        self.font.width(text, size * self.s)
    }

    /// Pille: Schatten, Umriss, Füllung; gedrückt rutscht sie auf den Schatten.
    fn pill(&mut self, r: Rect, color: Color, pressed: bool) {
        let s = self.s;
        let radius = r.h() / 2.0;
        let drop = Vec2::new(0.0, 3.0 * s);
        let body = if pressed { r.offset(drop * 0.7) } else { r };
        self.batch.fill_rounded_rect(
            r.min + drop,
            r.max + drop,
            radius,
            Color::rgba(color.0[0], color.0[1], color.0[2], 0.45),
        );
        let edge = body.shrink(-1.5 * s);
        self.batch
            .fill_rounded_rect(edge.min, edge.max, radius + 1.5 * s, OUTLINE);
        self.batch
            .fill_rounded_rect(body.min, body.max, radius, color);
    }

    /// Pillen-Knopf; `true` bei Klick.
    pub fn button(&mut self, id: &str, r: Rect, label: &str, color: Color) -> bool {
        let clicked = self.click(id, r);
        let hot = self.hovered(r);
        let pressed = self.pressing(id);
        let c = if hot { shade(color, 0.1) } else { color };
        self.pill(r, c, pressed);
        let y = if pressed { 2.0 * self.s } else { 0.0 };
        let size = (r.h() / self.s * 0.46).clamp(10.0, 18.0);
        self.label(
            label,
            r.center() + Vec2::new(0.0, y),
            size,
            on_color(color),
            Align::Center,
        );
        clicked
    }

    /// Auswahl als Chips, die in Zeilen umbrechen (`max_w`): der gewählte als farbige Pille,
    /// die anderen umrandet. Liefert den angeklickten Index und die belegte Höhe.
    #[allow(clippy::too_many_arguments)]
    pub fn chips(
        &mut self,
        id: &str,
        origin: Vec2,
        max_w: f32,
        height: f32,
        items: &[&str],
        selected: usize,
        color: Color,
    ) -> (Option<usize>, f32) {
        let s = self.s;
        let size = (height / s * 0.46).clamp(9.0, 18.0);
        let gap = 6.0 * s;
        let mut hit = None;
        let rects = self.chip_layout(origin, max_w, height, items);
        for (i, (item, r)) in items.iter().zip(&rects).enumerate() {
            let chip_id = format!("{id}#{i}");
            let radius = height / 2.0;
            if i == selected {
                self.pill(*r, color, false);
                self.label(item, r.center(), size, WHITE, Align::Center);
                let _ = self.click(&chip_id, *r);
            } else {
                // Umrandung, innen Kartenfarbe (hell beim Darüberfahren)
                self.batch
                    .fill_rounded_rect(r.min, r.max, radius, Color::hex(0xcdb894));
                let inner = r.shrink(1.5 * s);
                let fill = if self.hovered(*r) { HIGHLIGHT } else { CARD };
                self.batch
                    .fill_rounded_rect(inner.min, inner.max, radius - 1.5 * s, fill);
                self.label(item, r.center(), size, TEXT, Align::Center);
                if self.click(&chip_id, *r) {
                    hit = Some(i);
                }
            }
        }
        let bottom = rects.iter().map(|r| r.max.y).fold(origin.y, f32::max);
        (hit, bottom - origin.y + gap)
    }

    /// Lage der Chips von [`Self::chips`] (für die Höhe vor dem Zeichnen).
    #[allow(clippy::many_single_char_names)]
    pub fn chip_layout(&self, origin: Vec2, max_w: f32, height: f32, items: &[&str]) -> Vec<Rect> {
        let s = self.s;
        let size = (height / s * 0.46).clamp(9.0, 18.0);
        let gap = 6.0 * s;
        let (mut x, mut y) = (origin.x, origin.y);
        items
            .iter()
            .map(|item| {
                let w = self.text_width(item, size) + 24.0 * s;
                if x > origin.x && x + w > origin.x + max_w {
                    x = origin.x;
                    y += height + gap;
                }
                let r = Rect::new(x, y, w, height);
                x += w + gap;
                r
            })
            .collect()
    }

    /// Reiter als Knopf-Reihe: der gewählte als farbige Pille, die anderen als Text.
    /// Liefert den angeklickten Index. `colors` wird zyklisch verwendet.
    pub fn tabs(
        &mut self,
        id: &str,
        origin: Vec2,
        height: f32,
        items: &[&str],
        selected: usize,
        colors: &[Color],
    ) -> Option<usize> {
        let s = self.s;
        let size = (height / s * 0.46).clamp(9.0, 18.0);
        let mut x = origin.x;
        let mut hit = None;
        for (i, item) in items.iter().enumerate() {
            let w = self.text_width(item, size) + 24.0 * s;
            let r = Rect::new(x, origin.y, w, height);
            let tab_id = format!("{id}#{i}");
            if i == selected {
                let color = colors
                    .get(i % colors.len().max(1))
                    .copied()
                    .unwrap_or(ORANGE);
                self.pill(r, color, false);
                self.label(item, r.center(), size, WHITE, Align::Center);
                let _ = self.click(&tab_id, r);
            } else {
                if self.hovered(r) {
                    self.batch
                        .fill_rounded_rect(r.min, r.max, height / 2.0, HIGHLIGHT);
                }
                self.label(item, r.center(), size, TEXT, Align::Center);
                if self.click(&tab_id, r) {
                    hit = Some(i);
                }
            }
            x += w + 6.0 * s;
        }
        hit
    }

    /// Senkrechte Reiter (Seitenleiste der Einstellungen).
    pub fn side_tabs(
        &mut self,
        id: &str,
        r: Rect,
        items: &[&str],
        selected: usize,
        color: Color,
    ) -> Option<usize> {
        let s = self.s;
        let row = 32.0 * s;
        let mut hit = None;
        for (i, item) in items.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let rr = Rect::new(r.min.x, r.min.y + i as f32 * row, r.w(), row - 6.0 * s);
            let tab_id = format!("{id}#{i}");
            if i == selected {
                self.pill(rr, color, false);
                self.label(item, rr.center(), 12.0, WHITE, Align::Center);
                let _ = self.click(&tab_id, rr);
            } else {
                if self.hovered(rr) {
                    self.batch
                        .fill_rounded_rect(rr.min, rr.max, rr.h() / 2.0, HIGHLIGHT);
                }
                self.label(item, rr.center(), 12.0, TEXT, Align::Center);
                if self.click(&tab_id, rr) {
                    hit = Some(i);
                }
            }
        }
        hit
    }

    /// Schalter mit Beschriftung rechts; `true` bei Änderung.
    pub fn toggle(&mut self, id: &str, pos: Vec2, label: &str, value: &mut bool) -> bool {
        let s = self.s;
        let track = Rect::new(pos.x, pos.y, 38.0 * s, 20.0 * s);
        let w = self.text_width(label, 12.0) + 12.0 * s;
        let hit_r = Rect::new(pos.x, pos.y, track.w() + w, track.h());
        let changed = self.click(id, hit_r);
        if changed {
            *value = !*value;
        }
        let c = if *value { GREEN } else { SAND };
        self.pill(track, c, false);
        let knob_x = if *value {
            track.max.x - track.h() / 2.0
        } else {
            track.min.x + track.h() / 2.0
        };
        self.batch
            .fill_circle(Vec2::new(knob_x, track.center().y), 7.0 * s, WHITE);
        self.label(
            label,
            Vec2::new(track.max.x + 10.0 * s, track.center().y),
            12.0,
            TEXT,
            Align::Left,
        );
        changed
    }

    /// Schieberegler; `true` bei Änderung.
    pub fn slider(&mut self, id: &str, r: Rect, value: &mut f32, min: f32, max: f32) -> bool {
        let s = self.s;
        let _ = self.click(id, r);
        let mut changed = false;
        if self.pressing(id) {
            let t = ((self.input.mouse.x - r.min.x) / r.w()).clamp(0.0, 1.0);
            let v = min + (max - min) * t;
            if (v - *value).abs() > f32::EPSILON {
                *value = v;
                changed = true;
            }
        }
        let t = ((*value - min) / (max - min)).clamp(0.0, 1.0);
        let mid = r.center().y;
        let bar = Rect::new(r.min.x, mid - 4.0 * s, r.w(), 8.0 * s);
        self.batch
            .fill_rounded_rect(bar.min, bar.max, 4.0 * s, SAND);
        let filled = Rect::new(bar.min.x, bar.min.y, (bar.w() * t).max(bar.h()), bar.h());
        self.batch
            .fill_rounded_rect(filled.min, filled.max, 4.0 * s, ORANGE);
        let knob = Vec2::new(r.min.x + r.w() * t, mid);
        self.batch.fill_circle(knob, 10.0 * s, OUTLINE);
        self.batch.fill_circle(knob, 8.5 * s, WHITE);
        changed
    }

    /// Einzeiliges Textfeld. Klick setzt den Fokus; Tippen, Rücktaste, Entf,
    /// Pfeile, Pos1/Ende bearbeiten; Enter meldet [`FieldEvent::Submitted`].
    pub fn text_field(
        &mut self,
        id: &str,
        r: Rect,
        text: &mut String,
        max_chars: usize,
        placeholder: &str,
    ) -> FieldEvent {
        let s = self.s;
        if self.click(id, r) || (self.hovered(r) && self.input.pressed) {
            if self.state.focus.as_deref() != Some(id) {
                self.state.focus = Some(id.to_owned());
                self.state.cursor = text.chars().count();
            }
            self.state.claimed = true;
        }
        let focused = self.state.focus.as_deref() == Some(id);
        let mut event = FieldEvent::None;
        if focused {
            let len = text.chars().count();
            let mut cur = self.state.cursor.min(len);
            let byte = |t: &str, c: usize| t.char_indices().nth(c).map_or(t.len(), |(b, _)| b);
            for c in self.input.text.chars().filter(|c| !c.is_control()) {
                if text.chars().count() < max_chars {
                    text.insert(byte(text, cur), c);
                    cur += 1;
                    event = FieldEvent::Changed;
                }
            }
            for k in &self.input.keys {
                match k {
                    UiKey::Backspace if cur > 0 => {
                        let b = byte(text, cur - 1);
                        text.remove(b);
                        cur -= 1;
                        event = FieldEvent::Changed;
                    }
                    UiKey::Delete if cur < text.chars().count() => {
                        let b = byte(text, cur);
                        text.remove(b);
                        event = FieldEvent::Changed;
                    }
                    UiKey::Left => cur = cur.saturating_sub(1),
                    UiKey::Right => cur = (cur + 1).min(text.chars().count()),
                    UiKey::Home => cur = 0,
                    UiKey::End => cur = text.chars().count(),
                    UiKey::Enter => event = FieldEvent::Submitted,
                    UiKey::Escape => self.state.focus = None,
                    _ => {}
                }
            }
            self.state.cursor = cur;
        }

        let edge = r.shrink(-1.5 * s);
        let edge_color = if focused { ORANGE } else { CARD_EDGE };
        self.batch
            .fill_rounded_rect(edge.min, edge.max, r.h() / 2.0 + 1.5 * s, edge_color);
        self.batch
            .fill_rounded_rect(r.min, r.max, r.h() / 2.0, FIELD);
        let x = r.min.x + 12.0 * s;
        let size = 12.0;
        if text.is_empty() && !focused {
            self.label(
                placeholder,
                Vec2::new(x, r.center().y),
                size,
                TEXT_DIM,
                Align::Left,
            );
        } else {
            self.label(text, Vec2::new(x, r.center().y), size, TEXT, Align::Left);
        }
        if focused && (self.state.time * 2.0).fract() < 0.6 {
            let before: String = text.chars().take(self.state.cursor).collect();
            let cx = x + self.text_width(&before, size) + 1.0 * s;
            let half = 8.0 * s;
            self.batch.fill_rect(
                Vec2::new(cx, r.center().y - half),
                Vec2::new(cx + 1.5 * s, r.center().y + half),
                TEXT,
            );
        }
        event
    }

    /// Reihe von Farbfeldern (Palette); `true` bei Änderung.
    pub fn swatches(&mut self, id: &str, pos: Vec2, colors: &[Color], selected: &mut u8) -> bool {
        let s = self.s;
        let size = 16.0 * s;
        let gap = 3.0 * s;
        let mut changed = false;
        for (i, c) in colors.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let r = Rect::new(pos.x + i as f32 * (size + gap), pos.y, size, size);
            let sel = usize::from(*selected) == i;
            let edge = r.shrink(if sel { -2.5 * s } else { -s });
            self.batch.fill_rounded_rect(
                edge.min,
                edge.max,
                5.0 * s,
                if sel { TEXT } else { OUTLINE },
            );
            self.batch.fill_rounded_rect(r.min, r.max, 4.0 * s, *c);
            if self.click(&format!("{id}#{i}"), r) && !sel {
                *selected = u8::try_from(i).unwrap_or(0);
                changed = true;
            }
        }
        changed
    }

    /// Liste mit Spalten; Zeilen anklickbar, Mausrad scrollt. Liefert `(Index, Doppelklick)`.
    /// `columns`: (x-Versatz in px bei Skalierung 1, Überschrift); `rows`: Zellen je Zeile.
    pub fn list(
        &mut self,
        id: &str,
        r: Rect,
        columns: &[(f32, &str)],
        rows: &[Vec<String>],
        selected: Option<usize>,
    ) -> Option<(usize, bool)> {
        let s = self.s;
        let head = 22.0 * s;
        let row_h = 24.0 * s;
        for (x, title) in columns {
            self.label(
                title,
                Vec2::new(r.min.x + x * s, r.min.y + head / 2.0),
                10.0,
                TEXT_DIM,
                Align::Left,
            );
        }
        let body = Rect::new(r.min.x, r.min.y + head, r.w(), r.h() - head);
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let visible = (body.h() / row_h).floor().max(1.0) as usize;
        let max_scroll = rows.len().saturating_sub(visible);
        let scroll = self.state.scroll.entry(id.to_owned()).or_insert(0.0);
        if body.contains(self.input.mouse) {
            *scroll -= self.input.scroll;
        }
        #[allow(clippy::cast_precision_loss)]
        let clamped = scroll.clamp(0.0, max_scroll as f32);
        *scroll = clamped;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let first = clamped.round() as usize;

        let mut hit = None;
        for (n, row) in rows.iter().enumerate().skip(first).take(visible) {
            #[allow(clippy::cast_precision_loss)]
            let y = body.min.y + (n - first) as f32 * row_h;
            let rr = Rect::new(body.min.x, y, body.w(), row_h - 3.0 * s);
            if selected == Some(n) {
                self.batch
                    .fill_rounded_rect(rr.min, rr.max, rr.h() / 2.0, HIGHLIGHT);
            } else if self.hovered(rr) {
                self.batch.fill_rounded_rect(
                    rr.min,
                    rr.max,
                    rr.h() / 2.0,
                    lerp_color(CARD, HIGHLIGHT, 0.4),
                );
            }
            for ((x, _), cell) in columns.iter().zip(row) {
                self.label(
                    cell,
                    Vec2::new(r.min.x + x * s, rr.center().y),
                    12.0,
                    TEXT,
                    Align::Left,
                );
            }
            if self.click(&format!("{id}#{n}"), rr) {
                hit = Some((n, false));
            }
            if self.input.double_click && self.hovered(rr) {
                hit = Some((n, true));
            }
        }
        if rows.len() > visible {
            // Laufleiste rechts
            #[allow(clippy::cast_precision_loss)]
            let frac = visible as f32 / rows.len() as f32;
            #[allow(clippy::cast_precision_loss)]
            let pos = if max_scroll == 0 {
                0.0
            } else {
                clamped / max_scroll as f32
            };
            let track_h = body.h();
            let thumb_h = (track_h * frac).max(20.0 * s);
            let y = body.min.y + (track_h - thumb_h) * pos;
            self.batch.fill_rounded_rect(
                Vec2::new(body.max.x - 5.0 * s, y),
                Vec2::new(body.max.x - 1.0 * s, y + thumb_h),
                2.0 * s,
                SAND,
            );
        }
        hit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INTER: &[u8] = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");

    struct Harness {
        batch: ShapeBatch,
        font: Font,
        state: UiState,
        input: UiInput,
    }

    impl Harness {
        fn new() -> Self {
            Self {
                batch: ShapeBatch::default(),
                font: Font::new(INTER).unwrap(),
                state: UiState::default(),
                input: UiInput::default(),
            }
        }

        /// Ein Frame mit den aktuellen Eingaben.
        fn frame<R>(&mut self, f: impl FnOnce(&mut Ui<'_>) -> R) -> R {
            self.batch.clear();
            let mut ui = Ui {
                batch: &mut self.batch,
                font: &self.font,
                input: &self.input,
                state: &mut self.state,
                s: 1.0,
            };
            ui.begin(0.016);
            let r = f(&mut ui);
            ui.end();
            self.input.next_frame();
            r
        }

        fn press(&mut self, p: Vec2) {
            self.input.mouse = p;
            self.input.down = true;
            self.input.pressed = true;
        }

        fn release(&mut self) {
            self.input.down = false;
            self.input.released = true;
        }
    }

    const BTN: Rect = Rect {
        min: Vec2::new(10.0, 10.0),
        max: Vec2::new(110.0, 40.0),
    };

    #[test]
    fn button_clicks_on_press_and_release_inside() {
        let mut h = Harness::new();
        h.press(Vec2::new(50.0, 20.0));
        assert!(
            !h.frame(|ui| ui.button("b", BTN, "Los", GREEN)),
            "erst beim Loslassen"
        );
        h.release();
        assert!(h.frame(|ui| ui.button("b", BTN, "Los", GREEN)));
        // Drücken innen, Loslassen außen: kein Klick
        h.press(Vec2::new(50.0, 20.0));
        h.frame(|ui| ui.button("b", BTN, "Los", GREEN));
        h.input.mouse = Vec2::new(300.0, 300.0);
        h.release();
        assert!(!h.frame(|ui| ui.button("b", BTN, "Los", GREEN)));
    }

    #[test]
    fn toggle_and_tabs() {
        let mut h = Harness::new();
        let mut on = false;
        h.press(Vec2::new(15.0, 15.0));
        h.frame(|ui| ui.toggle("t", Vec2::new(10.0, 10.0), "Vollbild", &mut on));
        h.release();
        assert!(h.frame(|ui| ui.toggle("t", Vec2::new(10.0, 10.0), "Vollbild", &mut on)));
        assert!(on);

        let items = ["Spielen", "Training", "Beenden"];
        let mut hit = None;
        for _ in 0..2 {
            // Mitte des zweiten Reiters liegt sicher hinter dem ersten
            let x = 10.0 + h.font.width("Spielen", 11.5) + 24.0 + 6.0 + 20.0;
            h.press(Vec2::new(x, 20.0));
            h.frame(|ui| ui.tabs("tabs", Vec2::new(10.0, 10.0), 26.0, &items, 0, &[GREEN]));
            h.release();
            hit = h.frame(|ui| ui.tabs("tabs", Vec2::new(10.0, 10.0), 26.0, &items, 0, &[GREEN]));
        }
        assert_eq!(hit, Some(1));
    }

    #[test]
    fn slider_follows_mouse_while_held() {
        let mut h = Harness::new();
        let r = Rect::new(0.0, 0.0, 200.0, 20.0);
        let mut v = 0.0;
        h.press(Vec2::new(50.0, 10.0));
        assert!(h.frame(|ui| ui.slider("s", r, &mut v, 0.0, 1.0)));
        assert!((v - 0.25).abs() < 1e-3);
        h.input.mouse = Vec2::new(500.0, 10.0); // über das Ende hinaus
        h.frame(|ui| ui.slider("s", r, &mut v, 0.0, 1.0));
        assert!((v - 1.0).abs() < 1e-6);
        h.release();
        h.frame(|ui| ui.slider("s", r, &mut v, 0.0, 1.0));
        h.input.mouse = Vec2::new(0.0, 10.0);
        h.frame(|ui| ui.slider("s", r, &mut v, 0.0, 1.0));
        assert!(
            (v - 1.0).abs() < 1e-6,
            "ohne gedrückte Taste keine Änderung"
        );
    }

    #[test]
    fn text_field_edit_and_focus() {
        let mut h = Harness::new();
        let r = Rect::new(0.0, 0.0, 200.0, 26.0);
        let mut name = String::from("Elo");
        h.press(Vec2::new(20.0, 10.0));
        h.frame(|ui| ui.text_field("n", r, &mut name, 16, ""));
        h.release();
        h.input.text.push_str("ra!");
        assert_eq!(
            h.frame(|ui| ui.text_field("n", r, &mut name, 16, "")),
            FieldEvent::Changed
        );
        assert_eq!(name, "Elora!");
        h.input.keys = vec![UiKey::Backspace, UiKey::Home, UiKey::Delete];
        h.frame(|ui| ui.text_field("n", r, &mut name, 16, ""));
        assert_eq!(name, "lora");
        h.input.text.push('Ü');
        h.frame(|ui| ui.text_field("n", r, &mut name, 16, ""));
        assert_eq!(name, "Ülora", "Cursor nach Pos1 am Anfang, Umlaute");
        h.input.keys = vec![UiKey::Enter];
        assert_eq!(
            h.frame(|ui| ui.text_field("n", r, &mut name, 16, "")),
            FieldEvent::Submitted
        );
        // Klick daneben nimmt den Fokus
        h.press(Vec2::new(500.0, 500.0));
        h.frame(|ui| ui.text_field("n", r, &mut name, 16, ""));
        h.release();
        h.input.text.push('x');
        h.frame(|ui| ui.text_field("n", r, &mut name, 16, ""));
        assert_eq!(name, "Ülora");
        // Höchstlänge
        let mut short = String::new();
        h.press(Vec2::new(20.0, 10.0));
        h.frame(|ui| ui.text_field("k", r, &mut short, 3, ""));
        h.release();
        h.input.text.push_str("abcdef");
        h.frame(|ui| ui.text_field("k", r, &mut short, 3, ""));
        assert_eq!(short, "abc");
    }

    #[test]
    fn list_select_scroll_and_double_click() {
        let mut h = Harness::new();
        let r = Rect::new(0.0, 0.0, 300.0, 22.0 + 24.0 * 3.0);
        let rows: Vec<Vec<String>> = (0..10).map(|i| vec![format!("Server {i}")]).collect();
        let cols = [(8.0, "Name")];
        // Zeile 1 anklicken (Kopf 22 px, Zeilen 24 px)
        h.press(Vec2::new(50.0, 22.0 + 24.0 + 10.0));
        h.frame(|ui| ui.list("l", r, &cols, &rows, None));
        h.release();
        assert_eq!(
            h.frame(|ui| ui.list("l", r, &cols, &rows, None)),
            Some((1, false))
        );
        // zweimal nach unten scrollen → erste sichtbare Zeile 2
        h.input.mouse = Vec2::new(50.0, 22.0 + 10.0);
        h.input.scroll = -2.0;
        h.frame(|ui| ui.list("l", r, &cols, &rows, None));
        h.input.double_click = true;
        assert_eq!(
            h.frame(|ui| ui.list("l", r, &cols, &rows, None)),
            Some((2, true))
        );
        // Scrollen über das Ende hinaus wird begrenzt
        h.input.scroll = -50.0;
        h.frame(|ui| ui.list("l", r, &cols, &rows, None));
        assert!((h.state.scroll["l"] - 7.0).abs() < 1e-6);
    }

    /// Galerie zur Sichtprüfung: `cargo test -p elora-client --bin elora ui_gallery -- --ignored`,
    /// danach `cargo xtask svg-preview target/ui-gallery.svg target/ui-gallery.png 1280`.
    #[test]
    #[ignore = "erzeugt nur eine Datei zur Sichtprüfung"]
    #[allow(clippy::too_many_lines)] // Beispielseite
    fn ui_gallery() {
        let mut h = Harness::new();
        h.input.mouse = Vec2::new(222.0, 172.0); // über „Training“ (Hover)
        let mut name = String::from("Elora");
        let mut on = true;
        let mut off = false;
        let mut vol = 0.7;
        let mut sel = 7u8;
        let palette: Vec<Color> = crate::skins::BODY.iter().map(|(_, c)| *c).collect();
        h.state.focus = Some("name".into());
        h.state.cursor = 5;
        h.frame(|ui| {
            ui.card(Rect::new(40.0, 40.0, 1200.0, 640.0));
            ui.label("Elora", Vec2::new(70.0, 80.0), 26.0, LOGO, Align::Left);
            ui.tabs(
                "top",
                Vec2::new(190.0, 66.0),
                28.0,
                &[
                    "Spielen",
                    "Training",
                    "Server erstellen",
                    "Einstellungen",
                    "Beenden",
                ],
                0,
                &[GREEN],
            );
            for (i, (t, c)) in [
                ("Spielen", GREEN),
                ("Training", BLUE),
                ("Server erstellen", VIOLET),
                ("Einstellungen", ORANGE),
                ("Beenden", GRAY),
            ]
            .iter()
            .enumerate()
            {
                #[allow(clippy::cast_precision_loss)]
                let y = 140.0 + i as f32 * 44.0;
                ui.button(&format!("b{i}"), Rect::new(70.0, y, 220.0, 34.0), t, *c);
            }
            ui.side_tabs(
                "side",
                Rect::new(340.0, 140.0, 150.0, 200.0),
                &["Spieler", "Steuerung", "Grafik", "Ton", "Sprache"],
                0,
                ORANGE,
            );
            ui.label("Name", Vec2::new(540.0, 150.0), 11.0, TEXT_DIM, Align::Left);
            ui.text_field(
                "name",
                Rect::new(540.0, 162.0, 260.0, 30.0),
                &mut name,
                16,
                "Name",
            );
            ui.text_field(
                "addr",
                Rect::new(540.0, 204.0, 260.0, 30.0),
                &mut String::new(),
                64,
                "Adresse, z. B. 127.0.0.1:8303",
            );
            ui.label(
                "Körper",
                Vec2::new(540.0, 256.0),
                11.0,
                TEXT_DIM,
                Align::Left,
            );
            ui.swatches("body", Vec2::new(540.0, 268.0), &palette, &mut sel);
            ui.toggle("vsync", Vec2::new(540.0, 310.0), "VSync", &mut on);
            ui.toggle("full", Vec2::new(540.0, 342.0), "Vollbild", &mut off);
            ui.label(
                "Lautstärke",
                Vec2::new(540.0, 384.0),
                11.0,
                TEXT_DIM,
                Align::Left,
            );
            ui.slider(
                "vol",
                Rect::new(540.0, 394.0, 260.0, 24.0),
                &mut vol,
                0.0,
                1.0,
            );
            let rows: Vec<Vec<String>> = [
                ("Eloras Wiese", "ctf-test", "CTF", "6/8", "24 ms"),
                ("Tropfen-Arena", "sandbox", "DM", "3/8", "41 ms"),
                ("Nachtschicht", "sandbox", "iDM", "8/8", "63 ms"),
                ("Team Blau", "ctf-test", "TDM", "2/12", "88 ms"),
                ("LAN-Party", "sandbox", "LMS", "0/8", "12 ms"),
                ("Sechs", "x", "DM", "1/8", "99 ms"),
            ]
            .iter()
            .map(|r| vec![r.0.into(), r.1.into(), r.2.into(), r.3.into(), r.4.into()])
            .collect();
            ui.card(Rect::new(840.0, 140.0, 370.0, 170.0));
            ui.list(
                "servers",
                Rect::new(852.0, 150.0, 346.0, 150.0),
                &[
                    (8.0, "Name"),
                    (130.0, "Karte"),
                    (205.0, "Modus"),
                    (255.0, "Spieler"),
                    (305.0, "Ping"),
                ],
                &rows,
                Some(1),
            );
        });
        let svg = h.batch.debug_svg(
            Vec2::default(),
            Vec2::new(1280.0, 720.0),
            Color::hex(0xd8e3ec),
        );
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/ui-gallery.svg"),
            svg,
        )
        .unwrap();
    }

    #[test]
    fn swatch_selection() {
        let mut h = Harness::new();
        let colors = [GREEN, BLUE, VIOLET];
        let mut sel = 0u8;
        h.press(Vec2::new(10.0 + 2.0 * 19.0 + 5.0, 15.0));
        h.frame(|ui| ui.swatches("c", Vec2::new(10.0, 10.0), &colors, &mut sel));
        h.release();
        assert!(h.frame(|ui| ui.swatches("c", Vec2::new(10.0, 10.0), &colors, &mut sel)));
        assert_eq!(sel, 2);
    }
}

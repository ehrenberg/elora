//! HUD (M5.8, E-102 design B) as a custom game UI (E-031), drawn with the
//! vector renderer in screen pixels:
//!
//! - **bottom center** a bar with health and armor bars and the weapon selection
//!   (ammo below the active weapon),
//! - **top center** mode, phase/timer and score.
//!
//! All sizes apply to a 720 pixel window height and are scaled with the height.

use elora_game::{Mode, Phase};
use elora_protocol::GameView;
use elora_render::{Align, Color, Font, ShapeBatch};
use elora_sim::{Character, TICKS_PER_SECOND, Team, Vec2, Weapon};

use crate::items::ItemArt;
use crate::lang::Lang;

const PANEL: Color = Color::rgba(0.118, 0.165, 0.212, 0.6);
const EMPTY: Color = Color::rgba(1.0, 1.0, 1.0, 0.2);
const SELECTED: Color = Color::rgba(1.0, 1.0, 1.0, 0.25);
const TEXT: Color = Color::rgb(1.0, 1.0, 1.0);
const TEXT_DIM: Color = Color::rgba(1.0, 1.0, 1.0, 0.7);
const HEALTH: Color = Color::hex(0xe05a7a);
const ARMOR: Color = Color::hex(0xe0b85a);
const SUDDEN_DEATH: Color = Color::hex(0xff7850);

/// What the HUD should display.
#[derive(Debug, Clone, Copy)]
pub struct HudInfo<'a> {
    /// Own figure, `None` = dead or spectator.
    pub character: Option<&'a Character>,
    pub max_health: i32,
    pub view: Option<&'a GameView>,
    pub tick: u64,
    pub local: Option<usize>,
    pub lang: &'a Lang,
    /// UI scale from the settings (E-120).
    pub ui_scale: f32,
}

#[derive(Debug)]
pub struct Hud {
    font: Font,
}

impl Hud {
    /// # Panics
    /// If the embedded font is faulty (covered by tests).
    pub fn new() -> Self {
        Self {
            font: Font::new(include_bytes!("../../../assets/fonts/Inter-Regular.ttf"))
                .expect("Inter-Regular.ttf lesbar"),
        }
    }

    /// Font of the game UI (also for the game displays).
    pub fn font(&self) -> &Font {
        &self.font
    }

    /// Draws the HUD for an area of `screen` pixels.
    pub fn draw(&self, batch: &mut ShapeBatch, items: &ItemArt, screen: Vec2, info: &HudInfo<'_>) {
        let s = scale(screen, info.ui_scale);
        if let Some(ch) = info.character {
            self.bar(batch, items, screen, s, ch, info.max_health);
        } else {
            let pos = Vec2::new(screen.x / 2.0, screen.y - 40.0 * s);
            let text = info.lang.t("hud.dead");
            let w = self.font.width(text, 16.0 * s) + 28.0 * s;
            batch.fill_rounded_rect(
                pos - Vec2::new(w / 2.0, 18.0 * s),
                pos + Vec2::new(w / 2.0, 18.0 * s),
                12.0 * s,
                PANEL,
            );
            self.font
                .draw_centered(batch, text, pos, 16.0 * s, TEXT, Align::Center);
        }
        if let Some(view) = info.view {
            self.status(batch, screen, s, view, info);
        }
    }

    /// Bar at the bottom center: bars on the left, weapons on the right.
    fn bar(
        &self,
        batch: &mut ShapeBatch,
        items: &ItemArt,
        screen: Vec2,
        s: f32,
        ch: &Character,
        max: i32,
    ) {
        let size = Vec2::new(300.0, 54.0) * s;
        let min = Vec2::new((screen.x - size.x) / 2.0, screen.y - size.y - 14.0 * s);
        batch.fill_rounded_rect(min, min + size, 16.0 * s, PANEL);

        #[allow(clippy::cast_precision_loss)]
        let frac = |v: i32| (v.max(0) as f32 / max.max(1) as f32).min(1.0);
        let bar = |batch: &mut ShapeBatch, y: f32, h: f32, value: i32, color: Color| {
            let a = min + Vec2::new(16.0 * s, y * s);
            let full = Vec2::new(124.0 * s, h * s);
            batch.fill_rounded_rect(a, a + full, h * s / 2.0, EMPTY);
            if value > 0 {
                // at least as wide as high so that the round ends fit
                let part = Vec2::new((full.x * frac(value)).max(full.y), full.y);
                batch.fill_rounded_rect(a, a + part, h * s / 2.0, color);
            }
        };
        bar(batch, 14.0, 11.0, ch.health, HEALTH);
        bar(batch, 31.0, 8.0, ch.armor, ARMOR);

        for (i, &w) in Weapon::ALL.iter().enumerate() {
            let slot = ch.arsenal.slot(w);
            #[allow(clippy::cast_precision_loss)]
            let x = min.x + (160.0 + i as f32 * 44.0) * s;
            let center = Vec2::new(x + 20.0 * s, min.y + 22.0 * s);
            let active = w == ch.arsenal.active;
            if active {
                batch.fill_rounded_rect(
                    Vec2::new(x, min.y + 6.0 * s),
                    Vec2::new(x + 40.0 * s, min.y + 48.0 * s),
                    10.0 * s,
                    SELECTED,
                );
            }
            let alpha = if slot.got { 1.0 } else { 0.3 };
            items.draw_icon(batch, center, w, 0.95 * s, alpha);
            if active && let Some(ammo) = slot.ammo {
                self.font.draw(
                    batch,
                    &ammo.to_string(),
                    Vec2::new(center.x, min.y + 45.0 * s),
                    11.0 * s,
                    TEXT,
                    Align::Center,
                );
            }
        }
    }

    /// Top center: "DM · 3:24", below it the score or team standing.
    fn status(
        &self,
        batch: &mut ShapeBatch,
        screen: Vec2,
        s: f32,
        view: &GameView,
        info: &HudInfo<'_>,
    ) {
        let lang = info.lang;
        let title = format!("{} · {}", view.title(), phase_text(view, info.tick, lang));
        let mut lines: Vec<(String, f32, Color)> = vec![(title, 16.0, TEXT)];
        if view.mode.teams() {
            // team standing as a separate line, drawn in color below
            lines.push((String::new(), 20.0, TEXT));
        } else if let Some(me) = info.local {
            let mine = view.stats.get(&me).map_or(0, |st| st.score);
            let top = view.stats.values().map(|st| st.score).max().unwrap_or(0);
            lines.push((
                lang.f("hud.score_line", &[("mine", &mine), ("top", &top)]),
                14.0,
                TEXT_DIM,
            ));
        }
        if view.score_limit > 0 {
            let unit = lang.t(if view.mode == Mode::Ctf {
                "hud.unit_captures"
            } else {
                "hud.unit_points"
            });
            lines.push((
                lang.f("hud.goal", &[("n", &view.score_limit), ("unit", &unit)]),
                12.0,
                TEXT_DIM,
            ));
        }
        if view.sudden_death {
            lines.push((lang.t("hud.sudden_death").to_owned(), 14.0, SUDDEN_DEATH));
        }

        let red = lang.f("hud.red", &[("n", &view.team_score[0])]);
        let blue = lang.f("hud.blue", &[("n", &view.team_score[1])]);
        let team_width = self.font.width(&format!("{red} : {blue}"), 20.0 * s);
        let mut width = lines
            .iter()
            .map(|(t, size, _)| self.font.width(t, size * s))
            .fold(0.0, f32::max);
        if view.mode.teams() {
            width = width.max(team_width);
        }
        let width = width + 32.0 * s;
        let height: f32 = lines
            .iter()
            .map(|(_, size, _)| size * 1.45 * s)
            .sum::<f32>()
            + 12.0 * s;
        let top = Vec2::new((screen.x - width) / 2.0, 10.0 * s);
        batch.fill_rounded_rect(top, top + Vec2::new(width, height), 14.0 * s, PANEL);

        let mut y = top.y + 6.0 * s;
        let cx = screen.x / 2.0;
        for (text, size, color) in &lines {
            let h = size * 1.45 * s;
            let mid = Vec2::new(cx, y + h / 2.0);
            if text.is_empty() {
                // team standing: "Red 3 : 1 Blue" in team colors
                let colon = self.font.width(" : ", size * s);
                self.font
                    .draw_centered(batch, ":", mid, size * s, TEXT, Align::Center);
                self.font.draw_centered(
                    batch,
                    &red,
                    mid - Vec2::new(colon / 2.0, 0.0),
                    size * s,
                    crate::draw::team_color(Team::Red),
                    Align::Right,
                );
                self.font.draw_centered(
                    batch,
                    &blue,
                    mid + Vec2::new(colon / 2.0, 0.0),
                    size * s,
                    crate::draw::team_color(Team::Blue),
                    Align::Left,
                );
            } else {
                self.font
                    .draw_centered(batch, text, mid, size * s, *color, Align::Center);
            }
            y += h;
        }
    }
}

/// Scale of the game UI: 1 at a 720 pixel window height, times UI scale (E-120).
pub fn scale(screen: Vec2, ui_scale: f32) -> f32 {
    (screen.y / 720.0).clamp(0.6, 3.0) * ui_scale
}

fn secs_left(until: u64, tick: u64) -> u64 {
    until
        .saturating_sub(tick)
        .div_ceil(u64::from(TICKS_PER_SECOND))
}

pub fn clock(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// Phase or timer as text.
fn phase_text(view: &GameView, tick: u64, lang: &Lang) -> String {
    match view.phase {
        Phase::Warmup { until: Some(t) } => lang.f("hud.warmup", &[("s", &secs_left(t, tick))]),
        Phase::Warmup { until: None } => lang.t("hud.waiting").to_owned(),
        Phase::Countdown { until } => lang.f("hud.countdown", &[("s", &secs_left(until, tick))]),
        Phase::Running => {
            let elapsed = tick.saturating_sub(view.match_start_tick) / u64::from(TICKS_PER_SECOND);
            if view.time_limit > 0 {
                clock((u64::from(view.time_limit) * 60).saturating_sub(elapsed))
            } else {
                clock(elapsed)
            }
        }
        Phase::RoundOver { .. } => lang.t("hud.round_over").to_owned(),
        Phase::MatchOver { .. } => lang.t("hud.match_over").to_owned(),
    }
}

/// Crosshair color by health (E-102): white → yellow → red, smoothly.
pub fn crosshair_color(health: i32, max: i32) -> Color {
    const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    const YELLOW: Color = Color::hex(0xffd24a);
    const RED: Color = Color::hex(0xff4a4a);
    #[allow(clippy::cast_precision_loss)]
    let f = (health.max(0) as f32 / max.max(1) as f32).clamp(0.0, 1.0);
    if f >= 0.5 {
        elora_render::lerp_color(YELLOW, WHITE, (f - 0.5) * 2.0)
    } else {
        elora_render::lerp_color(RED, YELLOW, f * 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crosshair_goes_white_yellow_red() {
        let close = |a: Color, b: Color| a.0.iter().zip(b.0).all(|(x, y)| (x - y).abs() < 1e-3);
        assert!(close(crosshair_color(10, 10), Color::rgb(1.0, 1.0, 1.0)));
        assert!(close(crosshair_color(5, 10), Color::hex(0xffd24a)));
        assert!(close(crosshair_color(0, 10), Color::hex(0xff4a4a)));
        // smooth in between
        let mid = crosshair_color(7, 10);
        assert!(mid.0[2] > 0.29 && mid.0[2] < 1.0);
    }

    /// Visual inspection: `cargo test -p elora-client --bin elora hud_sheet -- --ignored`,
    /// then `cargo xtask svg-preview target/hud-dm.svg target/hud-dm.png 1280` (likewise `hud-ctf`).
    #[test]
    #[ignore = "erzeugt nur Dateien zur Sichtprüfung"]
    fn hud_sheet() {
        use elora_game::Stats;
        let hud = Hud::new();
        let items = ItemArt::load();
        let mut ch = Character::spawn(Vec2::default(), 10);
        ch.health = 7;
        ch.armor = 4;
        ch.arsenal.give(Weapon::Grenade, 6, 10);
        ch.arsenal.active = Weapon::Grenade;
        let stats: std::collections::BTreeMap<usize, Stats> = [
            (
                0,
                Stats {
                    score: 12,
                    ..Stats::default()
                },
            ),
            (
                1,
                Stats {
                    score: 15,
                    ..Stats::default()
                },
            ),
        ]
        .into_iter()
        .collect();
        for (name, mode, limit) in [("dm", Mode::Dm, 20), ("ctf", Mode::Ctf, 3)] {
            let view = GameView {
                mode,
                instagib: false,
                phase: Phase::Running,
                team_score: [2, 1],
                sudden_death: false,
                score_limit: limit,
                time_limit: 5,
                match_start_tick: 0,
                friendly_fire: false,
                stats: stats.clone(),
            };
            let mut batch = ShapeBatch::default();
            hud.draw(
                &mut batch,
                &items,
                Vec2::new(1280.0, 720.0),
                &HudInfo {
                    character: Some(&ch),
                    max_health: 10,
                    view: Some(&view),
                    tick: 96 * 50,
                    local: Some(0),
                    lang: &Lang::new(crate::lang::Language::De),
                    ui_scale: 1.0,
                },
            );
            let svg = batch.debug_svg(
                Vec2::default(),
                Vec2::new(1280.0, 720.0),
                Color::hex(0x98bfdf),
            );
            std::fs::write(
                format!("{}/../../target/hud-{name}.svg", env!("CARGO_MANIFEST_DIR")),
                svg,
            )
            .unwrap();
        }
    }

    #[test]
    fn hud_draws_without_view() {
        let hud = Hud::new();
        let items = ItemArt::load();
        let ch = Character::spawn(Vec2::default(), 10);
        let mut batch = ShapeBatch::default();
        hud.draw(
            &mut batch,
            &items,
            Vec2::new(1280.0, 720.0),
            &HudInfo {
                character: Some(&ch),
                max_health: 10,
                view: None,
                tick: 0,
                local: None,
                lang: &Lang::new(crate::lang::Language::De),
                ui_scale: 1.0,
            },
        );
        assert!(batch.triangle_count() > 100);
    }
}

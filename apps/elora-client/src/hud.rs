//! HUD (M5.8, E-102 Entwurf B) als eigene Spiel-UI (E-031), gezeichnet mit dem
//! Vektor-Renderer in Bildschirm-Pixeln:
//!
//! - **unten mittig** eine Leiste mit Leben- und Rüstungsbalken und der Waffenwahl
//!   (Munition unter der aktiven Waffe),
//! - **oben mittig** Modus, Phase/Timer und Punkte.
//!
//! Alle Maße gelten für 720 Pixel Fensterhöhe und werden mit der Höhe skaliert.

use elora_game::{Mode, Phase};
use elora_protocol::GameView;
use elora_render::{Align, Color, Font, ShapeBatch};
use elora_sim::{Character, TICKS_PER_SECOND, Team, Vec2, Weapon};

use crate::items::ItemArt;

const PANEL: Color = Color::rgba(0.118, 0.165, 0.212, 0.6);
const EMPTY: Color = Color::rgba(1.0, 1.0, 1.0, 0.2);
const SELECTED: Color = Color::rgba(1.0, 1.0, 1.0, 0.25);
const TEXT: Color = Color::rgb(1.0, 1.0, 1.0);
const TEXT_DIM: Color = Color::rgba(1.0, 1.0, 1.0, 0.7);
const HEALTH: Color = Color::hex(0xe05a7a);
const ARMOR: Color = Color::hex(0xe0b85a);
const SUDDEN_DEATH: Color = Color::hex(0xff7850);

/// Was das HUD anzeigen soll.
#[derive(Debug, Clone, Copy)]
pub struct HudInfo<'a> {
    /// Eigene Figur, `None` = tot oder Zuschauer.
    pub character: Option<&'a Character>,
    pub max_health: i32,
    pub view: Option<&'a GameView>,
    pub tick: u64,
    pub local: Option<usize>,
}

#[derive(Debug)]
pub struct Hud {
    font: Font,
}

impl Hud {
    /// # Panics
    /// Wenn die eingebettete Schrift fehlerhaft ist (wird von Tests abgedeckt).
    pub fn new() -> Self {
        Self {
            font: Font::new(include_bytes!("../../../assets/fonts/Inter-Regular.ttf"))
                .expect("Inter-Regular.ttf lesbar"),
        }
    }

    /// Zeichnet das HUD für eine Fläche von `screen` Pixeln.
    pub fn draw(&self, batch: &mut ShapeBatch, items: &ItemArt, screen: Vec2, info: &HudInfo<'_>) {
        let s = (screen.y / 720.0).clamp(0.6, 3.0);
        if let Some(ch) = info.character {
            self.bar(batch, items, screen, s, ch, info.max_health);
        } else {
            let pos = Vec2::new(screen.x / 2.0, screen.y - 40.0 * s);
            let text = "Tot – Feuertaste zum Respawn (sonst automatisch nach 3 s)";
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

    /// Leiste unten mittig: Balken links, Waffen rechts.
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
                // mindestens so breit wie hoch, damit die runden Enden passen
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

    /// Oben mittig: „DM · 3:24“, darunter Punkte bzw. Teamstand.
    fn status(
        &self,
        batch: &mut ShapeBatch,
        screen: Vec2,
        s: f32,
        view: &GameView,
        info: &HudInfo<'_>,
    ) {
        let title = format!("{} · {}", view.title(), phase_text(view, info.tick));
        let mut lines: Vec<(String, f32, Color)> = vec![(title, 16.0, TEXT)];
        if view.mode.teams() {
            // Teamstand als eigene Zeile, farbig gezeichnet unten
            lines.push((String::new(), 20.0, TEXT));
        } else if let Some(me) = info.local {
            let mine = view.stats.get(&me).map_or(0, |st| st.score);
            let top = view.stats.values().map(|st| st.score).max().unwrap_or(0);
            lines.push((format!("Punkte {mine} · Bester {top}"), 14.0, TEXT_DIM));
        }
        if view.score_limit > 0 {
            let unit = if view.mode == Mode::Ctf {
                "Eroberungen"
            } else {
                "Punkte"
            };
            lines.push((format!("Ziel: {} {unit}", view.score_limit), 12.0, TEXT_DIM));
        }
        if view.sudden_death {
            lines.push(("SUDDEN DEATH".into(), 14.0, SUDDEN_DEATH));
        }

        let red = format!("Rot {}", view.team_score[0]);
        let blue = format!("{} Blau", view.team_score[1]);
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
                // Teamstand: „Rot 3 : 1 Blau“ in Teamfarben
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

fn secs_left(until: u64, tick: u64) -> u64 {
    until
        .saturating_sub(tick)
        .div_ceil(u64::from(TICKS_PER_SECOND))
}

pub fn clock(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// Phase bzw. Timer als Text.
fn phase_text(view: &GameView, tick: u64) -> String {
    match view.phase {
        Phase::Warmup { until: Some(t) } => format!("Aufwärmen · {} s", secs_left(t, tick)),
        Phase::Warmup { until: None } => "Warte auf Spieler …".into(),
        Phase::Countdown { until } => format!("Start in {}", secs_left(until, tick)),
        Phase::Running => {
            let elapsed = tick.saturating_sub(view.match_start_tick) / u64::from(TICKS_PER_SECOND);
            if view.time_limit > 0 {
                clock((u64::from(view.time_limit) * 60).saturating_sub(elapsed))
            } else {
                clock(elapsed)
            }
        }
        Phase::RoundOver { .. } => "Runde vorbei".into(),
        Phase::MatchOver { .. } => "Match vorbei".into(),
    }
}

/// Farbe des Fadenkreuzes nach dem Leben (E-102): Weiß → Gelb → Rot, fließend.
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
        // dazwischen fließend
        let mid = crosshair_color(7, 10);
        assert!(mid.0[2] > 0.29 && mid.0[2] < 1.0);
    }

    /// Sichtprüfung: `cargo test -p elora-client --bin elora hud_sheet -- --ignored`,
    /// danach `cargo xtask svg-preview target/hud-dm.svg target/hud-dm.png 1280` (ebenso `hud-ctf`).
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
            },
        );
        assert!(batch.triangle_count() > 100);
    }
}

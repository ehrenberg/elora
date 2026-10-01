//! Spielanzeigen als eigene Spiel-UI (M5.8, E-031) im Stil des HUDs (E-102):
//! Abstimmung, Killfeed, Chat mit Eingabe und Scoreboard. Gezeichnet in
//! Bildschirm-Pixeln mit dem Vektor-Renderer; Maße für 720 px Höhe, skaliert mit `s`.

// Layout-Code: `s` (Skalierung), `g` (Anzeige-Daten), `x`/`y`/`w`/`h` sind hier lesbarer als lange Namen
#![allow(clippy::many_single_char_names)]

use std::collections::{BTreeMap, VecDeque};
use std::time::{Duration, Instant};

use elora_client::online::ChatLine;
use elora_protocol::{GameView, VoteInfo};
use elora_render::{Align, Color, Font, ShapeBatch};
use elora_sim::{DeathCause, Team, Vec2, Weapon};

use crate::draw::team_color;
use crate::items::ItemArt;
use crate::lang::Lang;

/// Wie lange Chat-Zeilen ohne offenes Chat-Fenster sichtbar sind.
const FADE: Duration = Duration::from_secs(10);
const KILLFEED_TIME: Duration = Duration::from_secs(6);

const PANEL: Color = Color::rgba(0.118, 0.165, 0.212, 0.6);
const TEXT: Color = Color::rgb(1.0, 1.0, 1.0);
const TEXT_DIM: Color = Color::rgba(1.0, 1.0, 1.0, 0.7);
const SERVER: Color = Color::hex(0xffdc78);
const OWN: Color = Color::hex(0xf2c14e);
const ROW: Color = Color::rgba(1.0, 1.0, 1.0, 0.06);

#[derive(Debug, Clone)]
pub struct KillEntry {
    pub killer: Option<(String, Team)>,
    pub victim: (String, Team),
    pub cause: DeathCause,
    pub at: Instant,
}

/// Killfeed aus Tod-Ereignissen ergänzen.
pub fn record_kills(
    feed: &mut VecDeque<KillEntry>,
    events: &[elora_sim::Event],
    names: &BTreeMap<usize, String>,
    teams: &BTreeMap<usize, Team>,
    now: Instant,
    lang: &Lang,
) {
    for e in events {
        if let elora_sim::Event::Death {
            player,
            killer,
            cause,
            ..
        } = *e
            && cause != DeathCause::Game
        {
            let who = |i: usize| {
                let name = names
                    .get(&i)
                    .cloned()
                    .unwrap_or_else(|| lang.f("game.slot", &[("n", &i)]));
                (name, teams.get(&i).copied().unwrap_or_default())
            };
            feed.push_back(KillEntry {
                killer: killer.filter(|k| *k != player).map(who),
                victim: who(player),
                cause,
                at: now,
            });
            while feed.len() > 20 {
                feed.pop_front();
            }
        }
    }
}

/// Offenes Chat-Eingabefeld.
#[derive(Debug, Default)]
pub struct ChatInput {
    pub open: bool,
    pub team: bool,
    pub text: String,
}

pub struct GameUi<'a> {
    pub view: Option<&'a GameView>,
    pub names: &'a BTreeMap<usize, String>,
    pub teams: &'a BTreeMap<usize, Team>,
    pub local: Option<usize>,
    pub chat: &'a [ChatLine],
    pub input: &'a ChatInput,
    pub scoreboard: bool,
    pub vote: Option<&'a VoteInfo>,
    pub killfeed: &'a VecDeque<KillEntry>,
    /// Karte wird geladen: Name, empfangene und gesamte Bytes (M6.5).
    pub loading: Option<(&'a str, usize, usize)>,
    /// Laufzeit in s (blinkender Cursor).
    pub time: f32,
    pub lang: &'a Lang,
}

/// Namensfarbe: Teamfarbe in Team-Modi, sonst weiß.
fn name_color(team: Team) -> Color {
    match team {
        Team::Red | Team::Blue => team_color(team),
        _ => TEXT,
    }
}

/// Kürzt `text` mit „…“, bis er in `max` Pixel passt.
fn fit(font: &Font, text: &str, size: f32, max: f32) -> String {
    if font.width(text, size) <= max {
        return text.to_owned();
    }
    let mut out: String = text.to_owned();
    while !out.is_empty() && font.width(&format!("{out}…"), size) > max {
        out.pop();
    }
    format!("{out}…")
}

pub fn draw(
    batch: &mut ShapeBatch,
    font: &Font,
    items: &ItemArt,
    screen: Vec2,
    s: f32,
    g: &GameUi<'_>,
) {
    if let Some(v) = g.vote {
        vote(batch, font, screen, s, v, g.lang);
    }
    if let Some((map, received, size)) = g.loading {
        loading(batch, font, screen, s, map, received, size, g.lang);
    }
    killfeed(batch, font, items, screen, s, g);
    chat(batch, font, screen, s, g);
    if g.scoreboard
        && let Some(view) = g.view
    {
        scoreboard(batch, font, screen, s, g, view);
    }
}

/// Ladeanzeige der Karte in der Bildmitte mit Fortschrittsbalken.
#[allow(clippy::too_many_arguments)]
fn loading(
    batch: &mut ShapeBatch,
    font: &Font,
    screen: Vec2,
    s: f32,
    map: &str,
    received: usize,
    size: usize,
    lang: &Lang,
) {
    #[allow(clippy::cast_precision_loss)]
    let frac = if size == 0 {
        1.0
    } else {
        (received as f32 / size as f32).clamp(0.0, 1.0)
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let percent = (frac * 100.0).round() as u32;
    let text = lang.f("game.loading_map", &[("map", &map), ("percent", &percent)]);
    let w = font.width(&text, 16.0 * s).max(260.0 * s) + 40.0 * s;
    let top = Vec2::new((screen.x - w) / 2.0, screen.y / 2.0 - 40.0 * s);
    batch.fill_rounded_rect(top, top + Vec2::new(w, 70.0 * s), 14.0 * s, PANEL);
    font.draw_centered(
        batch,
        &text,
        Vec2::new(screen.x / 2.0, top.y + 28.0 * s),
        16.0 * s,
        TEXT,
        Align::Center,
    );
    let bar = top + Vec2::new(20.0 * s, 44.0 * s);
    let bw = w - 40.0 * s;
    batch.fill_rounded_rect(bar, bar + Vec2::new(bw, 8.0 * s), 4.0 * s, ROW);
    batch.fill_rounded_rect(bar, bar + Vec2::new(bw * frac, 8.0 * s), 4.0 * s, OWN);
}

/// Abstimmung unter der Statusanzeige.
fn vote(batch: &mut ShapeBatch, font: &Font, screen: Vec2, s: f32, v: &VoteInfo, lang: &Lang) {
    let title = lang.f(
        "game.vote_title",
        &[("text", &lang.vote_subject(&v.subject))],
    );
    let info = lang.f(
        "game.vote_info",
        &[
            ("yes", &v.yes),
            ("no", &v.no),
            ("voters", &v.voters),
            ("secs", &v.seconds_left),
        ],
    );
    let w = font
        .width(&title, 15.0 * s)
        .max(font.width(&info, 12.0 * s))
        + 32.0 * s;
    let top = Vec2::new((screen.x - w) / 2.0, 108.0 * s);
    batch.fill_rounded_rect(top, top + Vec2::new(w, 50.0 * s), 14.0 * s, PANEL);
    let cx = screen.x / 2.0;
    font.draw_centered(
        batch,
        &title,
        Vec2::new(cx, top.y + 17.0 * s),
        15.0 * s,
        TEXT,
        Align::Center,
    );
    font.draw_centered(
        batch,
        &info,
        Vec2::new(cx, top.y + 36.0 * s),
        12.0 * s,
        TEXT_DIM,
        Align::Center,
    );
}

fn cause_weapon(cause: DeathCause) -> Option<Weapon> {
    match cause {
        DeathCause::Weapon(w) => Some(w),
        _ => None,
    }
}

/// Killfeed oben rechts: „Täter [Waffe] Opfer“.
fn killfeed(
    batch: &mut ShapeBatch,
    font: &Font,
    items: &ItemArt,
    screen: Vec2,
    s: f32,
    g: &GameUi<'_>,
) {
    let now = Instant::now();
    let entries: Vec<&KillEntry> = g
        .killfeed
        .iter()
        .filter(|k| now - k.at < KILLFEED_TIME)
        .collect();
    let size = 14.0 * s;
    let row = 28.0 * s;
    let icon = 44.0 * s;
    let right = screen.x - 12.0 * s;
    let first = entries.len().saturating_sub(6);
    for (n, k) in entries[first..].iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let y = 12.0 * s + n as f32 * (row + 4.0 * s);
        let cause_text = match k.cause {
            DeathCause::World => Some(g.lang.t("game.death_zone")),
            DeathCause::Suicide => Some(g.lang.t("game.suicide")),
            _ => None,
        };
        let middle = cause_text.map_or(icon, |t| font.width(t, 12.0 * s) + 12.0 * s);
        let killer_w = k
            .killer
            .as_ref()
            .map_or(0.0, |(n, _)| font.width(n, size) + 8.0 * s);
        let victim_w = font.width(&k.victim.0, size);
        let w = killer_w + middle + victim_w + 36.0 * s;
        let left = right - w;
        batch.fill_rounded_rect(
            Vec2::new(left, y),
            Vec2::new(right, y + row),
            10.0 * s,
            PANEL,
        );
        let mid_y = y + row / 2.0;
        let mut x = left + 12.0 * s;
        if let Some((name, team)) = &k.killer {
            font.draw_centered(
                batch,
                name,
                Vec2::new(x, mid_y),
                size,
                name_color(*team),
                Align::Left,
            );
            x += killer_w;
        }
        match (cause_weapon(k.cause), cause_text) {
            (Some(w), _) => {
                items.draw_icon(batch, Vec2::new(x + icon / 2.0, mid_y), w, 0.8 * s, 1.0);
            }
            (None, Some(t)) => {
                font.draw_centered(
                    batch,
                    t,
                    Vec2::new(x + middle / 2.0, mid_y),
                    12.0 * s,
                    TEXT_DIM,
                    Align::Center,
                );
            }
            (None, None) => {}
        }
        x += middle + 4.0 * s;
        font.draw_centered(
            batch,
            &k.victim.0,
            Vec2::new(x, mid_y),
            size,
            name_color(k.victim.1),
            Align::Left,
        );
    }
}

/// Chat unten links; offen mit Eingabezeile und längerem Verlauf.
fn chat(batch: &mut ShapeBatch, font: &Font, screen: Vec2, s: f32, g: &GameUi<'_>) {
    let now = Instant::now();
    let open = g.input.open;
    let lines: Vec<&ChatLine> = g
        .chat
        .iter()
        .filter(|c| open || now - c.at < FADE)
        .collect();
    if lines.is_empty() && !open {
        return;
    }
    let size = 14.0 * s;
    let line_h = 20.0 * s;
    let width = 460.0 * s;
    let shown: Vec<&&ChatLine> = lines.iter().rev().take(if open { 14 } else { 8 }).collect();
    let rows = shown.len() + usize::from(open);
    #[allow(clippy::cast_precision_loss)]
    let height = rows as f32 * line_h + 16.0 * s;
    // Unterkante über der HUD-Leiste
    let bottom = screen.y - 90.0 * s;
    let top = Vec2::new(14.0 * s, bottom - height);
    batch.fill_rounded_rect(top, top + Vec2::new(width, height), 12.0 * s, PANEL);
    let x = top.x + 12.0 * s;
    let mut y = bottom - 8.0 * s - line_h / 2.0;
    if open {
        let label = g.lang.t(if g.input.team {
            "game.chat_team"
        } else {
            "game.chat_all"
        });
        let lw = font.width(label, size);
        font.draw_centered(batch, label, Vec2::new(x, y), size, TEXT_DIM, Align::Left);
        let text = fit(font, &g.input.text, size, width - lw - 36.0 * s);
        font.draw_centered(batch, &text, Vec2::new(x + lw, y), size, TEXT, Align::Left);
        if (g.time * 2.0).fract() < 0.6 {
            let cx = x + lw + font.width(&text, size) + 2.0 * s;
            batch.fill_rect(
                Vec2::new(cx, y - 8.0 * s),
                Vec2::new(cx + 1.5 * s, y + 8.0 * s),
                TEXT,
            );
        }
        y -= line_h;
    }
    for c in shown {
        let (text, color) = match &c.from {
            Some(from) => (
                format!("{}{from}: {}", if c.team { "[Team] " } else { "" }, c.text),
                TEXT,
            ),
            None => (
                format!(
                    "*** {}",
                    c.message
                        .as_ref()
                        .map_or_else(|| c.text.clone(), |m| g.lang.message(m))
                ),
                SERVER,
            ),
        };
        let text = fit(font, &text, size, width - 24.0 * s);
        font.draw_centered(batch, &text, Vec2::new(x, y), size, color, Align::Left);
        y -= line_h;
    }
}

type Column = (Option<Team>, String, Vec<(usize, elora_game::Stats)>);

/// Spalten des Scoreboards: je Team (bzw. alle Spieler) und Zuschauer, sortiert nach Punkten.
fn columns(g: &GameUi<'_>, view: &GameView) -> Vec<Column> {
    let groups: Vec<(Option<Team>, &str)> = if view.mode.teams() {
        vec![
            (Some(Team::Red), "game.team_red"),
            (Some(Team::Blue), "game.team_blue"),
            (Some(Team::Spectator), "game.spectators"),
        ]
    } else {
        vec![
            (None, "game.players"),
            (Some(Team::Spectator), "game.spectators"),
        ]
    };
    groups
        .into_iter()
        .filter_map(|(team, title)| {
            let mut rows: Vec<(usize, elora_game::Stats)> = view
                .stats
                .iter()
                .filter(|(i, _)| {
                    let t = g.teams.get(i).copied().unwrap_or_default();
                    match team {
                        Some(Team::Spectator) => t == Team::Spectator,
                        Some(tt) => t == tt,
                        None => t != Team::Spectator,
                    }
                })
                .map(|(i, st)| (*i, *st))
                .collect();
            if rows.is_empty() && team == Some(Team::Spectator) {
                return None;
            }
            rows.sort_by_key(|r| std::cmp::Reverse(r.1.score));
            Some((team, g.lang.t(title).to_owned(), rows))
        })
        .collect()
}

/// Scoreboard in der Mitte.
fn scoreboard(
    batch: &mut ShapeBatch,
    font: &Font,
    screen: Vec2,
    s: f32,
    g: &GameUi<'_>,
    view: &GameView,
) {
    let columns = columns(g, view);
    let col_w = 330.0 * s;
    let row_h = 24.0 * s;
    let max_rows = columns.iter().map(|c| c.2.len()).max().unwrap_or(0);
    #[allow(clippy::cast_precision_loss)]
    let (w, h) = (
        columns.len() as f32 * col_w + 24.0 * s,
        (max_rows as f32 + 2.0) * row_h + 70.0 * s,
    );
    let top = Vec2::new((screen.x - w) / 2.0, (screen.y - h) / 2.0);
    batch.fill_rounded_rect(
        top,
        top + Vec2::new(w, h),
        16.0 * s,
        Color::rgba(0.118, 0.165, 0.212, 0.8),
    );
    font.draw_centered(
        batch,
        &g.lang
            .f("game.scoreboard_title", &[("mode", &view.title())]),
        Vec2::new(screen.x / 2.0, top.y + 24.0 * s),
        20.0 * s,
        TEXT,
        Align::Center,
    );
    for (ci, column) in columns.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let x0 = top.x + 12.0 * s + ci as f32 * col_w;
        scoreboard_column(
            batch,
            font,
            g,
            view,
            Vec2::new(x0, top.y + 58.0 * s),
            s,
            column,
        );
    }
}

/// Eine Spalte des Scoreboards ab `origin` (Kopfzeile, Überschriften, Zeilen).
fn scoreboard_column(
    batch: &mut ShapeBatch,
    font: &Font,
    g: &GameUi<'_>,
    view: &GameView,
    origin: Vec2,
    s: f32,
    (team, title, rows): &Column,
) {
    let (x0, mut y) = (origin.x, origin.y);
    let col_w = 330.0 * s;
    let row_h = 24.0 * s;
    let size = 14.0 * s;
    let head = match team.and_then(Team::index) {
        Some(t) => format!("{title} · {}", view.team_score[t]),
        None => title.clone(),
    };
    let head_color = team.map_or(TEXT, name_color);
    font.draw_centered(
        batch,
        &head,
        Vec2::new(x0 + 8.0 * s, y),
        16.0 * s,
        head_color,
        Align::Left,
    );
    y += row_h;
    let cols = [x0 + 190.0 * s, x0 + 250.0 * s, x0 + 305.0 * s];
    font.draw_centered(
        batch,
        g.lang.t("game.col_name"),
        Vec2::new(x0 + 8.0 * s, y),
        11.0 * s,
        TEXT_DIM,
        Align::Left,
    );
    for (label, cx) in ["game.col_score", "game.col_kills", "game.col_deaths"]
        .map(|k| g.lang.t(k))
        .iter()
        .zip(cols)
    {
        font.draw_centered(
            batch,
            label,
            Vec2::new(cx, y),
            11.0 * s,
            TEXT_DIM,
            Align::Right,
        );
    }
    for (n, (i, st)) in rows.iter().enumerate() {
        y += row_h;
        if n % 2 == 0 {
            batch.fill_rounded_rect(
                Vec2::new(x0, y - row_h / 2.0),
                Vec2::new(x0 + col_w - 12.0 * s, y + row_h / 2.0),
                6.0 * s,
                ROW,
            );
        }
        let name = g
            .names
            .get(i)
            .cloned()
            .unwrap_or_else(|| g.lang.f("game.slot", &[("n", i)]));
        let color = if Some(*i) == g.local { OWN } else { TEXT };
        let name = fit(font, &name, size, 170.0 * s);
        font.draw_centered(
            batch,
            &name,
            Vec2::new(x0 + 8.0 * s, y),
            size,
            color,
            Align::Left,
        );
        for (v, cx) in [
            st.score.to_string(),
            st.kills.to_string(),
            st.deaths.to_string(),
        ]
        .iter()
        .zip(cols)
        {
            font.draw_centered(batch, v, Vec2::new(cx, y), size, color, Align::Right);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sichtprüfung: `cargo test -p elora-client --bin elora ui_sheet -- --ignored`,
    /// danach `cargo xtask svg-preview target/ui.svg target/ui.png 1280`.
    #[test]
    #[ignore = "erzeugt nur eine Datei zur Sichtprüfung"]
    #[allow(clippy::too_many_lines)] // Beispieldaten
    fn ui_sheet() {
        use elora_game::{Mode, Phase, Stats};
        let font = Font::new(include_bytes!("../../../assets/fonts/Inter-Regular.ttf")).unwrap();
        let items = ItemArt::load();
        let now = Instant::now();
        let names: BTreeMap<usize, String> =
            [(0, "Elora"), (1, "Nimbus"), (2, "Pip"), (3, "Dummy 3")]
                .into_iter()
                .map(|(i, n)| (i, n.to_owned()))
                .collect();
        let teams: BTreeMap<usize, Team> = [
            (0, Team::Red),
            (1, Team::Blue),
            (2, Team::Red),
            (3, Team::Blue),
        ]
        .into_iter()
        .collect();
        let stats: BTreeMap<usize, Stats> =
            [(0, 7, 5, 2), (1, 4, 3, 4), (2, 2, 1, 3), (3, 0, 0, 6)]
                .into_iter()
                .map(|(i, score, kills, deaths)| {
                    (
                        i,
                        Stats {
                            score,
                            kills,
                            deaths,
                        },
                    )
                })
                .collect();
        let view = GameView {
            mode: Mode::Ctf,
            instagib: false,
            phase: Phase::Running,
            team_score: [2, 1],
            sudden_death: false,
            score_limit: 3,
            time_limit: 5,
            match_start_tick: 0,
            friendly_fire: false,
            stats,
        };
        let kill = |k: Option<usize>, v: usize, cause| KillEntry {
            killer: k.map(|k| (names[&k].clone(), teams[&k])),
            victim: (names[&v].clone(), teams[&v]),
            cause,
            at: now,
        };
        let killfeed: VecDeque<KillEntry> = [
            kill(Some(0), 1, DeathCause::Weapon(Weapon::Grenade)),
            kill(Some(1), 2, DeathCause::Weapon(Weapon::Laser)),
            kill(None, 3, DeathCause::World),
            kill(Some(2), 3, DeathCause::Weapon(Weapon::Hammer)),
        ]
        .into_iter()
        .collect();
        let line = |from: Option<&str>, team, text: &str| ChatLine {
            from: from.map(str::to_owned),
            team,
            text: text.to_owned(),
            message: None,
            at: now,
        };
        let chat = vec![
            line(None, false, "Nimbus ist beigetreten"),
            line(Some("Nimbus"), false, "hi!"),
            line(Some("Elora"), true, "ich hole die Flagge"),
            line(None, false, "Rot hat die Flagge erobert"),
        ];
        let input = ChatInput {
            open: true,
            team: false,
            text: "gg".into(),
        };
        let vote = VoteInfo {
            subject: elora_protocol::VoteSubject::Map("ctf-test".into()),
            yes: 2,
            no: 1,
            voters: 4,
            seconds_left: 17,
        };
        let mut batch = ShapeBatch::default();
        draw(
            &mut batch,
            &font,
            &items,
            Vec2::new(1280.0, 720.0),
            1.0,
            &GameUi {
                view: Some(&view),
                names: &names,
                teams: &teams,
                local: Some(0),
                chat: &chat,
                input: &input,
                scoreboard: true,
                vote: Some(&vote),
                killfeed: &killfeed,
                loading: None,
                time: 0.0,
                lang: &Lang::new(crate::lang::Language::De),
            },
        );
        let svg = batch.debug_svg(
            Vec2::default(),
            Vec2::new(1280.0, 720.0),
            Color::hex(0x98bfdf),
        );
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/ui.svg"),
            svg,
        )
        .unwrap();
    }

    #[test]
    fn fit_shortens_long_text() {
        let font = Font::new(include_bytes!("../../../assets/fonts/Inter-Regular.ttf")).unwrap();
        assert_eq!(fit(&font, "kurz", 14.0, 200.0), "kurz");
        let long = fit(&font, &"sehr lange Nachricht ".repeat(10), 14.0, 120.0);
        assert!(long.ends_with('…'));
        assert!(font.width(&long, 14.0) <= 120.0);
    }
}

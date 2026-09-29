//! Spielanzeigen (M4.9, Platzhalter bis M5): Statusleiste, Abstimmung, Killfeed,
//! Chat und Scoreboard.

use std::collections::{BTreeMap, VecDeque};
use std::time::{Duration, Instant};

use elora_client::online::ChatLine;
use elora_protocol::{GameView, VoteInfo};
use elora_sim::{DeathCause, Team, Weapon};

/// Wie lange Chat-Zeilen und Killfeed-Einträge ohne offenes Chat-Fenster sichtbar sind.
const FADE: Duration = Duration::from_secs(10);
const KILLFEED_TIME: Duration = Duration::from_secs(6);

#[derive(Debug, Clone)]
pub struct KillEntry {
    pub killer: Option<String>,
    pub victim: String,
    pub cause: DeathCause,
    pub at: Instant,
}

/// Killfeed aus Tod-Ereignissen ergänzen.
pub fn record_kills(
    feed: &mut VecDeque<KillEntry>,
    events: &[elora_sim::Event],
    names: &BTreeMap<usize, String>,
    now: Instant,
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
            let name = |i: usize| {
                names
                    .get(&i)
                    .cloned()
                    .unwrap_or_else(|| format!("Slot {i}"))
            };
            feed.push_back(KillEntry {
                killer: killer.map(name),
                victim: name(player),
                cause,
                at: now,
            });
            while feed.len() > 20 {
                feed.pop_front();
            }
        }
    }
}

/// Aktion aus der Anzeige.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    SendChat { team: bool, text: String },
    CloseChat,
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
    pub chat: Vec<ChatLine>,
    pub input: &'a mut ChatInput,
    pub scoreboard: bool,
    pub vote: Option<&'a VoteInfo>,
    pub killfeed: &'a VecDeque<KillEntry>,
}

fn team_color(t: Team) -> egui::Color32 {
    match t {
        Team::Red => egui::Color32::from_rgb(230, 100, 90),
        Team::Blue => egui::Color32::from_rgb(100, 150, 235),
        _ => egui::Color32::from_rgb(220, 220, 220),
    }
}

fn panel_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(egui::Color32::from_black_alpha(150))
        .corner_radius(6.0)
        .inner_margin(8.0)
}

pub fn draw(ui: &mut egui::Ui, g: &mut GameUi<'_>) -> Option<UiAction> {
    let ctx = ui.ctx().clone();
    // Modus, Timer und Punkte zeichnet das HUD (hud.rs)
    if let Some(v) = g.vote {
        vote(&ctx, v);
    }
    killfeed(&ctx, g);
    let action = chat(&ctx, g);
    if g.scoreboard
        && let Some(view) = g.view
    {
        scoreboard(&ctx, g, view);
    }
    action
}

fn vote(ctx: &egui::Context, v: &VoteInfo) {
    egui::Area::new("vote".into())
        .anchor(egui::Align2::CENTER_TOP, [0.0, 110.0])
        .interactable(false)
        .show(ctx, |ui| {
            panel_frame().show(ui, |ui| {
                ui.label(
                    egui::RichText::new(format!("Abstimmung: {}", v.description))
                        .color(egui::Color32::WHITE)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "Ja {} · Nein {} · {} Spieler · noch {} s  —  F3 Ja, F4 Nein",
                        v.yes, v.no, v.voters, v.seconds_left
                    ))
                    .color(egui::Color32::LIGHT_GRAY),
                );
            });
        });
}

fn weapon_name(cause: DeathCause) -> &'static str {
    match cause {
        DeathCause::Weapon(Weapon::Hammer) => "Hammer",
        DeathCause::Weapon(Weapon::Grenade) => "Granate",
        DeathCause::Weapon(Weapon::Laser) => "Laser",
        DeathCause::World => "Todeszone",
        DeathCause::Suicide => "kill",
        DeathCause::Game => "",
    }
}

fn killfeed(ctx: &egui::Context, g: &GameUi<'_>) {
    let now = Instant::now();
    let entries: Vec<&KillEntry> = g
        .killfeed
        .iter()
        .filter(|k| now - k.at < KILLFEED_TIME)
        .collect();
    if entries.is_empty() {
        return;
    }
    egui::Area::new("killfeed".into())
        .anchor(egui::Align2::RIGHT_TOP, [-360.0, 10.0])
        .interactable(false)
        .show(ctx, |ui| {
            panel_frame().show(ui, |ui| {
                for k in entries.iter().rev().take(6).rev() {
                    let text = match &k.killer {
                        Some(killer) if *killer != k.victim => {
                            format!("{killer}  [{}]  {}", weapon_name(k.cause), k.victim)
                        }
                        _ => format!("{}  [{}]", k.victim, weapon_name(k.cause)),
                    };
                    ui.label(egui::RichText::new(text).color(egui::Color32::WHITE));
                }
            });
        });
}

fn chat(ctx: &egui::Context, g: &mut GameUi<'_>) -> Option<UiAction> {
    let now = Instant::now();
    let open = g.input.open;
    let lines: Vec<&ChatLine> = g
        .chat
        .iter()
        .filter(|c| open || now - c.at < FADE)
        .collect();
    if lines.is_empty() && !open {
        return None;
    }
    let mut action = None;
    egui::Area::new("chat".into())
        .anchor(egui::Align2::LEFT_BOTTOM, [16.0, -16.0])
        .interactable(open)
        .show(ctx, |ui| {
            panel_frame().show(ui, |ui| {
                ui.set_max_width(520.0);
                for c in lines.iter().rev().take(if open { 14 } else { 8 }).rev() {
                    let text = match &c.from {
                        Some(from) => {
                            format!("{}{from}: {}", if c.team { "[Team] " } else { "" }, c.text)
                        }
                        None => format!("*** {}", c.text),
                    };
                    let color = if c.from.is_none() {
                        egui::Color32::from_rgb(255, 220, 120)
                    } else {
                        egui::Color32::WHITE
                    };
                    ui.label(egui::RichText::new(text).color(color));
                }
                if open {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(if g.input.team { "Team:" } else { "Alle:" })
                                .color(egui::Color32::LIGHT_GRAY),
                        );
                        let edit = ui.add(
                            egui::TextEdit::singleline(&mut g.input.text)
                                .desired_width(420.0)
                                .char_limit(elora_protocol::msg::MAX_CHAT),
                        );
                        edit.request_focus();
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let text = std::mem::take(&mut g.input.text);
                            action = Some(if text.trim().is_empty() {
                                UiAction::CloseChat
                            } else {
                                UiAction::SendChat {
                                    team: g.input.team,
                                    text,
                                }
                            });
                        } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            g.input.text.clear();
                            action = Some(UiAction::CloseChat);
                        }
                    });
                }
            });
        });
    action
}

fn scoreboard(ctx: &egui::Context, g: &GameUi<'_>, view: &GameView) {
    egui::Area::new("scoreboard".into())
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .interactable(false)
        .show(ctx, |ui| {
            panel_frame().inner_margin(14.0).show(ui, |ui| {
                ui.heading(
                    egui::RichText::new(format!("{} – Punkte", view.title()))
                        .color(egui::Color32::WHITE),
                );
                let groups: Vec<(Option<Team>, &str)> = if view.mode.teams() {
                    vec![
                        (Some(Team::Red), "Team Rot"),
                        (Some(Team::Blue), "Team Blau"),
                        (Some(Team::Spectator), "Zuschauer"),
                    ]
                } else {
                    vec![(None, "Spieler"), (Some(Team::Spectator), "Zuschauer")]
                };
                ui.horizontal_top(|ui| {
                    for (team, title) in groups {
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
                            .map(|(i, s)| (*i, *s))
                            .collect();
                        if rows.is_empty() && team == Some(Team::Spectator) {
                            continue;
                        }
                        rows.sort_by_key(|r| std::cmp::Reverse(r.1.score));
                        ui.vertical(|ui| {
                            let color = team.map_or(egui::Color32::WHITE, team_color);
                            let head = match team.and_then(Team::index) {
                                Some(t) => format!("{title} · {}", view.team_score[t]),
                                None => title.to_owned(),
                            };
                            ui.label(egui::RichText::new(head).color(color).strong().size(16.0));
                            egui::Grid::new(title)
                                .striped(true)
                                .num_columns(4)
                                .show(ui, |ui| {
                                    for h in ["Name", "Punkte", "Kills", "Tode"] {
                                        ui.label(
                                            egui::RichText::new(h).color(egui::Color32::LIGHT_GRAY),
                                        );
                                    }
                                    ui.end_row();
                                    for (i, s) in rows {
                                        let name = g
                                            .names
                                            .get(&i)
                                            .cloned()
                                            .unwrap_or_else(|| format!("Slot {i}"));
                                        let mut text =
                                            egui::RichText::new(name).color(egui::Color32::WHITE);
                                        if Some(i) == g.local {
                                            text = text
                                                .strong()
                                                .color(egui::Color32::from_rgb(242, 193, 78));
                                        }
                                        ui.label(text);
                                        for v in [
                                            s.score.to_string(),
                                            s.kills.to_string(),
                                            s.deaths.to_string(),
                                        ] {
                                            ui.label(
                                                egui::RichText::new(v).color(egui::Color32::WHITE),
                                            );
                                        }
                                        ui.end_row();
                                    }
                                });
                        });
                        ui.add_space(24.0);
                    }
                });
            });
        });
}

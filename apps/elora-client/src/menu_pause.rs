//! In-game menu (M7.9): pause with team choice, votes, settings – replaces
//! the game parts of the debug panel (E-031: egui only for developers now).

// Layout code: `s` (scale), `x`/`y`/`w`/`h` are more readable here than long names
#![allow(clippy::many_single_char_names)]

use std::collections::BTreeMap;

use elora_game::Mode;
use elora_protocol::{VoteInfo, VoteKind};
use elora_render::Align;
use elora_sim::{Team, Vec2};

use crate::menu::{MODES, MenuAction, MenuCtx};
use crate::ui::{BLUE, GRAY, GREEN, ORANGE, Rect, SAND, TEXT, TEXT_DIM, Ui, VIOLET};

/// Game state shown by the pause menu.
pub struct PauseCtx<'a> {
    pub online: bool,
    pub team_mode: bool,
    /// Own team.
    pub team: Team,
    pub names: &'a BTreeMap<usize, String>,
    pub local: Option<usize>,
    pub vote: Option<&'a VoteInfo>,
    /// "Map · mode" for orientation.
    pub server_line: String,
    /// Adventure running (A1.6): hint about saving instead of training.
    pub adventure: bool,
}

/// State of the pause menu between frames.
#[derive(Debug, Default)]
pub struct PauseState {
    /// Settings pages instead of pause content.
    pub settings_open: bool,
    /// 0 map, 1 mode, 2 kick, 3 spectator.
    vote_kind: usize,
    vote_map: String,
    vote_mode: Option<Mode>,
    vote_instagib: bool,
    vote_target: Option<usize>,
}

/// Content of the pause card `card`.
pub fn content(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    p: &PauseCtx<'_>,
    state: &mut PauseState,
    card: Rect,
) -> Option<MenuAction> {
    let s = cx.s;
    let lang = cx.lang;
    let mut action = None;
    ui.label(
        lang.t("menu.paused"),
        Vec2::new(card.min.x + 24.0 * s, card.min.y + 28.0 * s),
        20.0,
        TEXT,
        Align::Left,
    );
    ui.label(
        &p.server_line,
        Vec2::new(card.min.x + 24.0 * s, card.min.y + 52.0 * s),
        11.0,
        TEXT_DIM,
        Align::Left,
    );

    // left: continue, settings, main menu, quit
    let items = [
        ("menu.resume", GREEN, Some(MenuAction::Resume)),
        ("menu.settings", ORANGE, None),
        ("menu.to_menu", SAND, Some(MenuAction::ToMenu)),
        ("menu.quit", GRAY, Some(MenuAction::Quit)),
    ];
    for (i, (key, color, act)) in items.into_iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let r = Rect::new(
            card.min.x + 24.0 * s,
            card.min.y + (78.0 + i as f32 * 46.0) * s,
            200.0 * s,
            34.0 * s,
        );
        if ui.button(&format!("pause_{key}"), r, lang.t(key), color) {
            match act {
                Some(a) => action = Some(a),
                None => state.settings_open = true,
            }
        }
    }

    // right: game
    let right = Rect::new(
        card.min.x + 250.0 * s,
        card.min.y + 78.0 * s,
        card.max.x - card.min.x - 274.0 * s,
        card.h() - 100.0 * s,
    );
    if p.adventure {
        ui.label(
            lang.t("adventure.pause_warning"),
            Vec2::new(right.min.x, right.min.y + 8.0 * s),
            12.0,
            TEXT_DIM,
            Align::Left,
        );
        return action;
    }
    if !p.online {
        ui.label(
            lang.t("pause.training"),
            Vec2::new(right.min.x, right.min.y + 8.0 * s),
            13.0,
            TEXT,
            Align::Left,
        );
        let r = Rect::new(right.min.x, right.min.y + 24.0 * s, 160.0 * s, 30.0 * s);
        if ui.button("pause_respawn", r, lang.t("pause.respawn"), BLUE) {
            action = Some(MenuAction::Respawn);
        }
        return action;
    }
    action = team_row(ui, cx, p, right.min).or(action);
    action = vote_box(
        ui,
        cx,
        p,
        state,
        Rect::new(
            right.min.x,
            right.min.y + 84.0 * s,
            right.w(),
            right.h() - 84.0 * s,
        ),
    )
    .or(action);
    action
}

/// Choose a team or play/spectate, suicide.
fn team_row(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    p: &PauseCtx<'_>,
    origin: Vec2,
) -> Option<MenuAction> {
    let s = cx.s;
    let lang = cx.lang;
    let mut action = None;
    ui.label(
        lang.t("pause.team"),
        origin + Vec2::new(0.0, 8.0 * s),
        13.0,
        TEXT,
        Align::Left,
    );
    let options: Vec<(&str, Team, elora_render::Color)> = if p.team_mode {
        vec![
            ("pause.red", Team::Red, crate::draw::RED),
            ("pause.blue", Team::Blue, crate::draw::BLUE),
            ("pause.spectate", Team::Spectator, GRAY),
        ]
    } else {
        vec![
            ("pause.join", Team::None, GREEN),
            ("pause.spectate", Team::Spectator, GRAY),
        ]
    };
    let mut x = origin.x;
    for (key, team, color) in options {
        let w = ui.text_width(lang.t(key), 11.0) + 32.0 * s;
        let r = Rect::new(x, origin.y + 22.0 * s, w, 28.0 * s);
        let current = p.team == team;
        let c = if current { color } else { SAND };
        if ui.button(&format!("team_{key}"), r, lang.t(key), c) && !current {
            action = Some(MenuAction::SetTeam(team));
        }
        x += w + 8.0 * s;
    }
    let kw = ui.text_width(lang.t("pause.kill"), 11.0) + 32.0 * s;
    if ui.button(
        "pause_kill",
        Rect::new(x + 12.0 * s, origin.y + 22.0 * s, kw, 28.0 * s),
        lang.t("pause.kill"),
        GRAY,
    ) {
        action = Some(MenuAction::Kill);
    }
    action
}

/// Running vote (yes/no) or start a new one.
fn vote_box(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    p: &PauseCtx<'_>,
    state: &mut PauseState,
    r: Rect,
) -> Option<MenuAction> {
    let s = cx.s;
    let lang = cx.lang;
    let (x, mut y) = (r.min.x, r.min.y);
    if let Some(v) = p.vote {
        ui.label(
            &lang.f(
                "pause.vote_running",
                &[("text", &lang.vote_subject(&v.subject))],
            ),
            Vec2::new(x, y + 8.0 * s),
            12.0,
            TEXT,
            Align::Left,
        );
        let yes = Rect::new(x, y + 22.0 * s, 110.0 * s, 28.0 * s);
        let no = Rect::new(x + 120.0 * s, y + 22.0 * s, 110.0 * s, 28.0 * s);
        if ui.button("vote_yes", yes, lang.t("pause.yes"), GREEN) {
            return Some(MenuAction::Vote(true));
        }
        if ui.button("vote_no", no, lang.t("pause.no"), crate::draw::RED) {
            return Some(MenuAction::Vote(false));
        }
        return None;
    }
    ui.label(
        lang.t("pause.vote"),
        Vec2::new(x, y + 8.0 * s),
        13.0,
        TEXT,
        Align::Left,
    );
    y += 22.0 * s;
    let kinds = [
        lang.t("pause.vote_map"),
        lang.t("pause.vote_mode"),
        lang.t("pause.vote_kick"),
        lang.t("pause.vote_spec"),
    ];
    if let Some(i) = ui.tabs(
        "vote_kind",
        Vec2::new(x, y),
        24.0 * s,
        &kinds,
        state.vote_kind,
        &[VIOLET],
    ) {
        state.vote_kind = i;
    }
    y += 36.0 * s;
    let (kind, after) = vote_choice(ui, cx, p, state, Vec2::new(x, y), r.w());
    y = after;
    let start = Rect::new(x, y, 180.0 * s, 30.0 * s);
    let color = if kind.is_some() { GREEN } else { SAND };
    if ui.button("vote_start", start, lang.t("pause.vote_start"), color) {
        return kind.map(MenuAction::CallVote);
    }
    None
}

/// Choice per vote kind from `origin`; returns the vote (if complete)
/// and the y position below it.
fn vote_choice(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    p: &PauseCtx<'_>,
    state: &mut PauseState,
    origin: Vec2,
    width: f32,
) -> (Option<VoteKind>, f32) {
    let s = cx.s;
    let lang = cx.lang;
    let (x, mut y) = (origin.x, origin.y);
    let r = Rect::new(x, y, width, 0.0);
    let kind = match state.vote_kind {
        0 => {
            ui.text_field(
                "vote_map",
                Rect::new(x, y, r.w() * 0.7, 28.0 * s),
                &mut state.vote_map,
                32,
                lang.t("pause.vote_map_hint"),
            );
            y += 40.0 * s;
            let m = state.vote_map.trim();
            (!m.is_empty()).then(|| VoteKind::Map(m.to_owned()))
        }
        1 => {
            let names: Vec<&str> = MODES.iter().map(|m| m.name()).collect();
            let current = state
                .vote_mode
                .and_then(|m| MODES.iter().position(|x| *x == m))
                .unwrap_or(0);
            if let Some(i) = ui.tabs(
                "vote_mode",
                Vec2::new(x, y),
                24.0 * s,
                &names,
                current,
                &[BLUE],
            ) {
                state.vote_mode = Some(MODES[i]);
            }
            ui.toggle(
                "vote_instagib",
                Vec2::new(x, y + 34.0 * s),
                lang.t("menu.instagib"),
                &mut state.vote_instagib,
            );
            y += 68.0 * s;
            Some(VoteKind::Mode {
                mode: state.vote_mode.unwrap_or(MODES[0]),
                instagib: state.vote_instagib,
            })
        }
        k => {
            let others: Vec<(usize, &String)> = p
                .names
                .iter()
                .filter(|(i, _)| Some(**i) != p.local)
                .map(|(i, n)| (*i, n))
                .collect();
            if others.is_empty() {
                ui.label(
                    lang.t("pause.no_players"),
                    Vec2::new(x, y + 10.0 * s),
                    11.0,
                    TEXT_DIM,
                    Align::Left,
                );
                y += 30.0 * s;
                None
            } else {
                let rows: Vec<Vec<String>> =
                    others.iter().map(|(_, n)| vec![(*n).clone()]).collect();
                let list = Rect::new(x, y, r.w() * 0.7, 100.0 * s);
                let sel = state
                    .vote_target
                    .and_then(|t| others.iter().position(|(i, _)| *i == t));
                if let Some((row, _)) = ui.list(
                    "vote_target",
                    list,
                    &[(8.0, lang.t("game.col_name"))],
                    &rows,
                    sel,
                ) {
                    state.vote_target = Some(others[row].0);
                }
                y += 108.0 * s;
                state
                    .vote_target
                    .filter(|t| others.iter().any(|(i, _)| i == t))
                    .map(|t| {
                        let slot = u32::try_from(t).unwrap_or(0);
                        if k == 2 {
                            VoteKind::Kick(slot)
                        } else {
                            VoteKind::Spectate(slot)
                        }
                    })
            }
        }
    };
    (kind, y)
}

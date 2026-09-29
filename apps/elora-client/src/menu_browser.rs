//! Server-Browser auf der Seite „Spielen“ (M7.7): Reiter Internet / LAN / Favoriten,
//! Sortieren, Filter, Liste, Details mit Spielerliste, Direkt-Verbinden.

// Layout-Code: `s` (Skalierung), `x`/`y`/`w`/`h` sind hier lesbarer als lange Namen
#![allow(clippy::many_single_char_names)]

use std::time::Instant;

use elora_render::Align;
use elora_sim::{Team, Vec2};

use crate::browser::{Browser, SortBy, State, Tab};
use crate::menu::{MenuAction, MenuCtx};
use crate::ui::{BLUE, FieldEvent, GREEN, ORANGE, Rect, SAND, TEXT, TEXT_DIM, Ui};

/// Was der Browser-Teil der Seite bearbeitet.
pub struct BrowserEdit<'a> {
    pub browser: &'a mut Browser,
    /// Zuletzt geladener Reiter (ein Wechsel lädt neu).
    pub loaded: &'a mut Option<Tab>,
    pub address: &'a mut String,
    pub master_url: &'a mut String,
    /// Eingabe der Master-Adresse, bevor sie übernommen wird.
    pub master_input: &'a mut String,
}

const TABS: [Tab; 3] = [Tab::Internet, Tab::Lan, Tab::Favorites];
const SORTS: [SortBy; 3] = [SortBy::Ping, SortBy::Players, SortBy::Name];

/// Karte mit dem Browser in `r`.
pub fn card(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    r: Rect,
    e: &mut BrowserEdit<'_>,
) -> Option<MenuAction> {
    let s = cx.s;
    let lang = cx.lang;
    let now = Instant::now();
    ui.card(r);
    let x = r.min.x + 16.0 * s;
    let mut y = r.min.y + 14.0 * s;
    let mut action = None;

    // Reiter und Aktualisieren
    let names = [
        lang.t("browser.internet"),
        lang.t("browser.lan"),
        lang.t("browser.favorites"),
    ];
    let current = TABS.iter().position(|t| *t == e.browser.tab).unwrap_or(0);
    if let Some(i) = ui.tabs(
        "browser_tabs",
        Vec2::new(x, y),
        26.0 * s,
        &names,
        current,
        &[BLUE],
    ) {
        e.browser.tab = TABS[i];
    }
    let refresh = lang.t("browser.refresh");
    let rw = ui.text_width(refresh, 11.0) + 30.0 * s;
    let refresh_clicked = ui.button(
        "browser_refresh",
        Rect::new(r.max.x - 16.0 * s - rw, y, rw, 26.0 * s),
        refresh,
        SAND,
    );
    if refresh_clicked || *e.loaded != Some(e.browser.tab) {
        *e.loaded = Some(e.browser.tab);
        e.browser.refresh(e.master_url, cx.favorites, now);
    }
    y += 38.0 * s;

    toolbar(ui, cx, Vec2::new(x, y), r.max.x, e);
    y += 34.0 * s;

    // Liste bzw. Master-Adresse eintragen
    let list = Rect::new(x, y, r.w() - 32.0 * s, 210.0 * s);
    if e.browser.tab == Tab::Internet && e.master_url.trim().is_empty() {
        action = master_prompt(ui, cx, list, e).or(action);
    } else {
        action = server_list(ui, cx, list, e).or(action);
    }
    y += list.h() + 10.0 * s;

    // Details
    let details = Rect::new(x, y, list.w(), r.max.y - y - 60.0 * s);
    action = details_box(ui, cx, details, e).or(action);

    // Direkt verbinden
    let y = r.max.y - 46.0 * s;
    ui.label(
        lang.t("browser.direct"),
        Vec2::new(x, y + 15.0 * s),
        11.0,
        TEXT_DIM,
        Align::Left,
    );
    let lw = ui.text_width(lang.t("browser.direct"), 11.0) + 12.0 * s;
    let field = Rect::new(x + lw, y, list.w() - lw - 130.0 * s, 30.0 * s);
    let submitted = ui.text_field("address", field, e.address, 64, lang.t("menu.address_hint"))
        == FieldEvent::Submitted;
    let btn = Rect::new(field.max.x + 10.0 * s, y, 120.0 * s, 30.0 * s);
    if (ui.button("connect", btn, lang.t("menu.connect"), GREEN) || submitted)
        && !e.address.trim().is_empty()
    {
        action = Some(MenuAction::Connect(e.address.trim().to_owned()));
    }
    action
}

/// Serverliste; Klick wählt aus, Doppelklick verbindet.
fn server_list(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    list: Rect,
    e: &mut BrowserEdit<'_>,
) -> Option<MenuAction> {
    let s = cx.s;
    let lang = cx.lang;
    let visible = e.browser.visible();
    if visible.is_empty() {
        let text = if e.browser.busy() {
            lang.t("browser.searching")
        } else if !e.browser.status.is_empty() {
            e.browser.status.as_str()
        } else {
            lang.t("browser.empty")
        };
        ui.label(
            text,
            list.min + Vec2::new(8.0, 34.0) * s,
            12.0,
            TEXT_DIM,
            Align::Left,
        );
        return None;
    }
    let rows: Vec<Vec<String>> = visible
        .iter()
        .map(|entry| match &entry.state {
            State::Online { info, ping } => {
                let mut name = info.name.clone();
                if !info.compatible() {
                    name = format!("{name} ({})", lang.t("browser.other_version"));
                }
                vec![
                    name,
                    info.map.clone(),
                    info.mode.clone(),
                    format!("{}/{}", info.clients, info.max_clients),
                    format!("{} ms", ping.as_millis()),
                ]
            }
            State::Querying => vec![
                entry.addr.to_string(),
                "…".into(),
                String::new(),
                String::new(),
                String::new(),
            ],
            State::Unreachable => vec![
                entry.addr.to_string(),
                lang.t("browser.unreachable").to_owned(),
                String::new(),
                String::new(),
                String::new(),
            ],
        })
        .collect();
    let addrs: Vec<_> = visible.iter().map(|v| v.addr).collect();
    let w = list.w() / s;
    let cols = [
        (8.0, lang.t("browser.col_name")),
        (w * 0.44, lang.t("browser.col_map")),
        (w * 0.64, lang.t("browser.col_mode")),
        (w * 0.76, lang.t("browser.col_players")),
        (w * 0.88, lang.t("browser.col_ping")),
    ];
    let selected = e
        .browser
        .selected
        .and_then(|sel| addrs.iter().position(|a| *a == sel));
    let hit = ui.list("servers", list, &cols, &rows, selected)?;
    let addr = addrs[hit.0];
    e.browser.selected = Some(addr);
    e.address.clone_from(&addr.to_string());
    let connectable =
        matches!(&e.browser.entry(addr)?.state, State::Online { info, .. } if info.compatible());
    (hit.1 && connectable).then(|| MenuAction::Connect(addr.to_string()))
}

/// Details zum gewählten Server: Spielerliste, Verbinden, Favorit.
fn details_box(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    r: Rect,
    e: &mut BrowserEdit<'_>,
) -> Option<MenuAction> {
    let s = cx.s;
    let lang = cx.lang;
    let addr = e.browser.selected?;
    let entry = e.browser.entry(addr)?.clone();
    ui.batch
        .fill_rounded_rect(r.min, r.max, 12.0 * s, crate::ui::HIGHLIGHT);
    let x = r.min.x + 12.0 * s;
    let mut action = None;
    let address = addr.to_string();
    let fav = cx.favorites.contains(&address);
    let fav_label = lang.t(if fav {
        "menu.remove_favorite"
    } else {
        "menu.add_favorite"
    });
    let fw = ui.text_width(fav_label, 11.0) + 30.0 * s;
    if ui.button(
        "fav",
        Rect::new(r.max.x - 12.0 * s - fw, r.min.y + 10.0 * s, fw, 24.0 * s),
        fav_label,
        SAND,
    ) {
        action = Some(MenuAction::ToggleFavorite(address.clone()));
    }
    let State::Online { info, .. } = &entry.state else {
        ui.label(
            &address,
            Vec2::new(x, r.min.y + 22.0 * s),
            12.0,
            TEXT,
            Align::Left,
        );
        return action;
    };
    ui.label(
        &lang.f("browser.players", &[("name", &info.name)]),
        Vec2::new(x, r.min.y + 22.0 * s),
        13.0,
        TEXT,
        Align::Left,
    );
    ui.label(
        &address,
        Vec2::new(x, r.min.y + 40.0 * s),
        10.0,
        TEXT_DIM,
        Align::Left,
    );
    if info.compatible() {
        let cw = 120.0 * s;
        if ui.button(
            "connect_selected",
            Rect::new(
                r.max.x - 24.0 * s - fw - cw,
                r.min.y + 10.0 * s,
                cw,
                24.0 * s,
            ),
            lang.t("menu.connect"),
            GREEN,
        ) {
            action = Some(MenuAction::Connect(address.clone()));
        }
    }
    let players = &info.players;
    if players.is_empty() {
        ui.label(
            lang.t("browser.no_players"),
            Vec2::new(x, r.min.y + 62.0 * s),
            11.0,
            TEXT_DIM,
            Align::Left,
        );
        return action;
    }
    player_grid(ui, cx, r, players);
    action
}

/// Sortierung und Filter der Liste.
fn toolbar(ui: &mut Ui<'_>, cx: &MenuCtx<'_>, origin: Vec2, right: f32, e: &mut BrowserEdit<'_>) {
    let s = cx.s;
    let lang = cx.lang;
    let (x, y) = (origin.x, origin.y);
    let r = Rect::new(0.0, 0.0, right, 0.0);
    let sorts = [
        lang.t("browser.sort_ping"),
        lang.t("browser.sort_players"),
        lang.t("browser.sort_name"),
    ];
    let sort = SORTS.iter().position(|v| *v == e.browser.sort).unwrap_or(0);
    if let Some(i) = ui.tabs(
        "browser_sort",
        Vec2::new(x, y),
        22.0 * s,
        &sorts,
        sort,
        &[ORANGE],
    ) {
        e.browser.sort = SORTS[i];
    }
    let fx = r.max.x - 330.0 * s;
    ui.toggle(
        "hide_empty",
        Vec2::new(fx, y + 1.0 * s),
        lang.t("browser.hide_empty"),
        &mut e.browser.filter.hide_empty,
    );
    ui.toggle(
        "hide_full",
        Vec2::new(fx + 165.0 * s, y + 1.0 * s),
        lang.t("browser.hide_full"),
        &mut e.browser.filter.hide_full,
    );
}

/// Noch keine Master-Adresse: Feld zum Eintragen.
fn master_prompt(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    list: Rect,
    e: &mut BrowserEdit<'_>,
) -> Option<MenuAction> {
    let s = cx.s;
    let lang = cx.lang;
    let (x, y) = (list.min.x, list.min.y);
    let mut action = None;
    ui.label(
        lang.t("browser.no_master"),
        Vec2::new(x, y + 20.0 * s),
        12.0,
        TEXT,
        Align::Left,
    );
    let field = Rect::new(x, y + 36.0 * s, list.w() - 130.0 * s, 30.0 * s);
    let submitted = ui.text_field(
        "master_url",
        field,
        e.master_input,
        200,
        lang.t("browser.master_url"),
    ) == FieldEvent::Submitted;
    let apply = Rect::new(field.max.x + 10.0 * s, field.min.y, 110.0 * s, 30.0 * s);
    if (ui.button("master_apply", apply, lang.t("browser.apply"), GREEN) || submitted)
        && !e.master_input.trim().is_empty()
    {
        e.master_url.clone_from(&e.master_input.trim().to_owned());
        *e.loaded = None; // neu laden
        action = Some(MenuAction::SettingsChanged);
    }
    action
}

/// Spieler des gewählten Servers in Spalten, so viele wie Platz ist.
fn player_grid(ui: &mut Ui<'_>, cx: &MenuCtx<'_>, r: Rect, players: &[elora_protocol::InfoPlayer]) {
    let s = cx.s;
    let lang = cx.lang;
    let x = r.min.x + 12.0 * s;
    let row = 18.0 * s;
    let top = r.min.y + 58.0 * s;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let per_col = ((r.max.y - top - 6.0 * s) / row).floor().max(1.0) as usize;
    let col_w = 190.0 * s;
    for (i, p) in players.iter().enumerate() {
        let (col, line) = (i / per_col, i % per_col);
        #[allow(clippy::cast_precision_loss)]
        let px = x + col as f32 * col_w;
        if px + col_w > r.max.x {
            break;
        }
        #[allow(clippy::cast_precision_loss)]
        let py = top + line as f32 * row + row / 2.0;
        let color = match p.team {
            Team::Red | Team::Blue => crate::draw::team_color(p.team),
            _ => TEXT,
        };
        let name = if p.dummy {
            format!("{} ({})", p.name, lang.t("browser.dummy"))
        } else {
            p.name.clone()
        };
        ui.label(&name, Vec2::new(px, py), 11.0, color, Align::Left);
        ui.label(
            &p.score.to_string(),
            Vec2::new(px + col_w - 24.0 * s, py),
            11.0,
            TEXT_DIM,
            Align::Right,
        );
    }
}

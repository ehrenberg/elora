//! Main menu and pause menu (M7.3) in the "bar at the top, light & soft" style (E-125).
//!
//! The menu draws itself via [`crate::ui`] in screen pixels and reports
//! [`MenuAction`]s; the app executes them (start training, connect, host …).

// Layout code: `s` (scale), `w`/`h`/`x`/`y` are more readable here than long names
#![allow(clippy::many_single_char_names)]

use elora_game::Mode;
use elora_protocol::Skin;
use elora_render::{Align, Color, Font, ShapeBatch};
use elora_sim::{Team, Vec2};

use crate::figure::FigureArt;
use crate::lang::Lang;
use crate::menu_browser::BrowserEdit;
use crate::menu_settings::SettingsEdit;
use crate::ui::{
    self, BLUE, GRAY, GREEN, LOGO, ORANGE, Rect, TEXT, TEXT_DIM, Ui, UiInput, UiState, VIOLET,
};

/// Pages of the main menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Play,
    /// Adventure with save slots (A1.6).
    Adventure,
    Create,
    Settings,
}

/// What the app should do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    Training,
    /// Open the map editor (M6.6).
    Editor,
    Connect(String),
    /// Start a server with the values from [`CreateForm`] and connect.
    Host,
    ToggleFavorite(String),
    /// Settings were changed outside the settings pages (save).
    SettingsChanged,
    /// In-game pause menu (M7.9).
    SetTeam(Team),
    Kill,
    CallVote(elora_protocol::VoteKind),
    Vote(bool),
    Respawn,
    Resume,
    ToMenu,
    Quit,
    /// Adventure: new on slot, continue, delete (A1.6).
    AdventureNew(usize),
    AdventureContinue(usize),
    AdventureDelete(usize),
    /// Watch the intro video again (pause menu of the adventure, E-355).
    WatchIntro,
}

/// "Create server" form (E-122).
#[derive(Debug, Clone)]
pub struct CreateForm {
    pub name: String,
    /// Index into the map list.
    pub map: usize,
    pub mode: Mode,
    pub instagib: bool,
    /// Show on the internet (register with the master, E-170); off = private round.
    pub public: bool,
    pub max_clients: f32,
}

impl Default for CreateForm {
    fn default() -> Self {
        Self {
            name: "Eloras Server".into(),
            map: 0,
            mode: Mode::Dm,
            instagib: false,
            public: false,
            max_clients: 8.0,
        }
    }
}

pub const MODES: [Mode; 5] = [Mode::Dm, Mode::Tdm, Mode::Ctf, Mode::Lms, Mode::Lts];

/// Data the menu needs per frame.
pub struct MenuCtx<'a> {
    pub font: &'a Font,
    pub lang: &'a Lang,
    pub art: &'a FigureArt,
    /// Map decoration and figures for the background (E-292).
    pub map_art: &'a crate::map_art::MapArt,
    pub creatures: &'a crate::creatures::CreatureArt,
    pub last_server: Option<&'a str>,
    pub favorites: &'a [String],
    /// Map names for "Create server".
    pub maps: &'a [String],
    /// Status line (connection, hosting, errors).
    pub status: &'a str,
    /// Save slots (only filled on the "Adventure" page).
    pub slots: &'a [crate::app_adventure::SlotView],
    pub screen: Vec2,
    pub s: f32,
    pub dt: f32,
}

#[derive(Debug, Default)]
pub struct Menu {
    pub page: Page,
    pub settings_tab: usize,
    pub ui: UiState,
    pub input: UiInput,
    pub address: String,
    /// Server browser (M7.7) and last loaded tab.
    pub browser: crate::browser::Browser,
    pub browser_loaded: Option<crate::browser::Tab>,
    pub master_input: String,
    pub create: CreateForm,
    /// In-game pause menu open.
    pub paused: bool,
    pub pause: crate::menu_pause::PauseState,
    /// "Quit" chosen; the app quits on the next pass.
    pub quit: bool,
    /// Settings changed, saving waits for the mouse to be released.
    pub save_pending: bool,
    /// Last click (time, place) for double-click detection.
    pub last_click: Option<(std::time::Instant, Vec2)>,
    /// Slot whose deletion is about to be confirmed.
    pub confirm_delete: Option<usize>,
}

/// Color of the selected tab (editor, training and quit are actions and never selected).
const TAB_COLORS: [Color; 7] = [GREEN, LOGO, BLUE, VIOLET, GRAY, ORANGE, GRAY];

impl Menu {
    /// Draw the main menu; returns an action and whether settings were changed.
    pub fn draw_main(
        &mut self,
        batch: &mut ShapeBatch,
        cx: &MenuCtx<'_>,
        edit: &mut SettingsEdit<'_>,
    ) -> (Option<MenuAction>, bool) {
        background(batch, cx, *edit.skin, self.ui.time);
        let mut ui = Ui {
            batch,
            font: cx.font,
            input: &self.input,
            state: &mut self.ui,
            s: cx.s,
        };
        ui.begin(cx.dt);
        let mut action = top_bar(&mut ui, cx, &mut self.page);
        let s = cx.s;
        let content = Rect::new(
            18.0 * s,
            64.0 * s,
            cx.screen.x - 36.0 * s,
            cx.screen.y - 82.0 * s,
        );
        let mut changed = false;
        let page_action = match self.page {
            Page::Play => {
                self.browser.poll(std::time::Instant::now());
                let mut be = BrowserEdit {
                    browser: &mut self.browser,
                    loaded: &mut self.browser_loaded,
                    address: &mut self.address,
                    master_url: edit.master_url,
                    master_input: &mut self.master_input,
                };
                play_page(&mut ui, cx, content, edit.name, &mut be)
            }
            Page::Adventure => {
                crate::menu_adventure::page(&mut ui, cx, content, &mut self.confirm_delete)
            }
            Page::Create => create_page(&mut ui, cx, content, &mut self.create),
            Page::Settings => {
                changed = settings_page(&mut ui, cx, content, &mut self.settings_tab, edit);
                None
            }
        };
        action = action.or(page_action);
        if !cx.status.is_empty() {
            ui.label(
                cx.status,
                Vec2::new(cx.screen.x / 2.0, cx.screen.y - 10.0 * s),
                11.0,
                TEXT_DIM,
                Align::Center,
            );
        }
        ui.end();
        self.input.next_frame();
        (action, changed)
    }

    /// Pause menu over the game (M7.9); returns an action and whether settings were changed.
    pub fn draw_pause(
        &mut self,
        batch: &mut ShapeBatch,
        cx: &MenuCtx<'_>,
        p: &crate::menu_pause::PauseCtx<'_>,
        edit: &mut SettingsEdit<'_>,
    ) -> (Option<MenuAction>, bool) {
        let s = cx.s;
        batch.fill_rect(
            Vec2::default(),
            cx.screen,
            Color::rgba(0.118, 0.165, 0.212, 0.35),
        );
        let mut ui = Ui {
            batch,
            font: cx.font,
            input: &self.input,
            state: &mut self.ui,
            s,
        };
        ui.begin(cx.dt);
        let mut action = None;
        let mut changed = false;
        if self.pause.settings_open {
            let w = 760.0 * s;
            let area = Rect::new(
                (cx.screen.x - w) / 2.0,
                50.0 * s,
                w,
                cx.screen.y - 100.0 * s,
            );
            changed = settings_page(&mut ui, cx, area, &mut self.settings_tab, edit);
            let back = Rect::new(
                area.min.x + 8.0 * s,
                area.min.y + 224.0 * s,
                170.0 * s,
                32.0 * s,
            );
            if ui.button("pause_back", back, cx.lang.t("pause.back"), GRAY) {
                self.pause.settings_open = false;
            }
        } else {
            let w = 660.0 * s;
            let h = 360.0 * s;
            let card = Rect::new((cx.screen.x - w) / 2.0, (cx.screen.y - h) / 2.0, w, h);
            ui.card(card);
            action = crate::menu_pause::content(&mut ui, cx, p, &mut self.pause, card);
        }
        ui.end();
        self.input.next_frame();
        (action, changed)
    }
}

/// Background image (E-113, E-291, E-292): Tauwinkel at the time of day, two Eloras in front.
fn background(batch: &mut ShapeBatch, cx: &MenuCtx<'_>, skin: Skin, time: f32) {
    let (w, h, s) = (cx.screen.x, cx.screen.y, cx.s);
    crate::menu_scene::draw(
        batch,
        cx.map_art,
        cx.creatures,
        cx.screen,
        s,
        time,
        crate::menu_scene::local_hour(),
    );
    let tint = crate::skins::tint(skin, Team::None, false, crate::draw::team_color);
    cx.art.draw_pose(
        batch,
        Vec2::new(w * 0.8, h - 40.0 * s),
        190.0 * s,
        1.0,
        &tint,
    );
    let friend = Skin {
        body: 7,
        feet: 6,
        eyes: 1,
    };
    let tint = crate::skins::tint(friend, Team::None, false, crate::draw::team_color);
    cx.art.draw_pose(
        batch,
        Vec2::new(w * 0.68, h - 26.0 * s),
        100.0 * s,
        -1.0,
        &tint,
    );
}

/// Bar at the top: logo and tabs; training and quit are immediate actions.
fn top_bar(ui: &mut Ui<'_>, cx: &MenuCtx<'_>, page: &mut Page) -> Option<MenuAction> {
    let s = cx.s;
    let bar = Rect::new(0.0, 0.0, cx.screen.x, 48.0 * s);
    ui.batch.fill_rect(
        bar.min + Vec2::new(0.0, 4.0 * s),
        bar.max + Vec2::new(0.0, 4.0 * s),
        ui::SHADOW,
    );
    ui.batch
        .fill_rect(bar.min, bar.max + Vec2::new(0.0, 1.5 * s), ui::CARD_EDGE);
    ui.batch.fill_rect(bar.min, bar.max, ui::CARD);
    ui.label(
        "Elora",
        Vec2::new(20.0 * s, bar.center().y),
        22.0,
        LOGO,
        Align::Left,
    );
    let lang = cx.lang;
    let items = [
        lang.t("menu.play"),
        lang.t("adventure.tab"),
        lang.t("menu.training"),
        lang.t("menu.create"),
        lang.t("menu.editor"),
        lang.t("menu.settings"),
        lang.t("menu.quit"),
    ];
    let selected = match page {
        Page::Play => 0,
        Page::Adventure => 1,
        Page::Create => 3,
        Page::Settings => 5,
    };
    match ui.tabs(
        "top",
        Vec2::new(110.0 * s, 10.0 * s),
        28.0 * s,
        &items,
        selected,
        &[TAB_COLORS[selected]],
    ) {
        Some(0) => *page = Page::Play,
        Some(1) => *page = Page::Adventure,
        Some(2) => return Some(MenuAction::Training),
        Some(3) => *page = Page::Create,
        Some(4) => return Some(MenuAction::Editor),
        Some(5) => *page = Page::Settings,
        Some(6) => return Some(MenuAction::Quit),
        _ => {}
    }
    None
}

/// "Play": greeting and "Quick play" on the left, server browser on the right (M7.7).
fn play_page(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    area: Rect,
    name: &str,
    browser: &mut BrowserEdit<'_>,
) -> Option<MenuAction> {
    let s = cx.s;
    let lang = cx.lang;
    ui.label(
        lang.t("menu.welcome"),
        Vec2::new(area.min.x + 12.0 * s, area.min.y + 30.0 * s),
        15.0,
        TEXT,
        Align::Left,
    );
    ui.label(
        name,
        Vec2::new(area.min.x + 12.0 * s, area.min.y + 62.0 * s),
        30.0,
        TEXT,
        Align::Left,
    );
    let quick = quick_card(ui, cx, area);
    let card = Rect::new(
        area.min.x + 330.0 * s,
        area.min.y + 8.0 * s,
        area.w() - 340.0 * s,
        area.h() - 16.0 * s,
    );
    crate::menu_browser::card(ui, cx, card, browser).or(quick)
}

/// "Quick play" card: last server with "Go!".
fn quick_card(ui: &mut Ui<'_>, cx: &MenuCtx<'_>, area: Rect) -> Option<MenuAction> {
    let s = cx.s;
    let lang = cx.lang;
    let mut action = None;
    let quick = Rect::new(
        area.min.x + 8.0 * s,
        area.min.y + 96.0 * s,
        300.0 * s,
        110.0 * s,
    );
    ui.card(quick);
    ui.label(
        lang.t("menu.quick_play"),
        quick.min + Vec2::new(18.0, 24.0) * s,
        15.0,
        TEXT,
        Align::Left,
    );
    match cx.last_server {
        Some(addr) => {
            ui.label(
                addr,
                quick.min + Vec2::new(18.0, 48.0) * s,
                11.0,
                TEXT_DIM,
                Align::Left,
            );
            let r = Rect::new(
                quick.min.x + 18.0 * s,
                quick.min.y + 64.0 * s,
                100.0 * s,
                30.0 * s,
            );
            if ui.button("quick", r, lang.t("menu.go"), GREEN) {
                action = Some(MenuAction::Connect(addr.to_owned()));
            }
        }
        None => {
            ui.label(
                lang.t("menu.quick_none"),
                quick.min + Vec2::new(18.0, 56.0) * s,
                11.0,
                TEXT_DIM,
                Align::Left,
            );
        }
    }

    action
}

/// "Create server" (E-122): name, map, mode, instagib, player count.
#[allow(clippy::too_many_lines)]
fn create_page(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    area: Rect,
    form: &mut CreateForm,
) -> Option<MenuAction> {
    let s = cx.s;
    let lang = cx.lang;
    // the map list wraps into rows; the card grows with it
    let maps: Vec<&str> = cx.maps.iter().map(String::as_str).collect();
    let card_w = (area.w() - 16.0 * s).min(620.0 * s);
    let list_w = card_w - 40.0 * s;
    let probe = ui.chip_layout(Vec2::ZERO, list_w, 26.0 * s, &maps);
    let list_h = probe.iter().map(|r| r.max.y).fold(26.0 * s, f32::max);
    let card = Rect::new(
        area.min.x + 8.0 * s,
        area.min.y + 8.0 * s,
        card_w,
        334.0 * s + list_h,
    );
    ui.card(card);
    let x = card.min.x + 20.0 * s;
    let mut y = card.min.y + 24.0 * s;
    ui.label(
        lang.t("menu.server_name"),
        Vec2::new(x, y),
        11.0,
        TEXT_DIM,
        Align::Left,
    );
    ui.text_field(
        "srv_name",
        Rect::new(x, y + 12.0 * s, 300.0 * s, 30.0 * s),
        &mut form.name,
        32,
        "",
    );
    y += 64.0 * s;
    ui.label(
        lang.t("menu.map"),
        Vec2::new(x, y),
        11.0,
        TEXT_DIM,
        Align::Left,
    );
    let (hit, used) = ui.chips(
        "map",
        Vec2::new(x, y + 12.0 * s),
        list_w,
        26.0 * s,
        &maps,
        form.map,
        BLUE,
    );
    if let Some(i) = hit {
        form.map = i;
    }
    y += 12.0 * s + used + 16.0 * s;
    ui.label(
        lang.t("menu.mode"),
        Vec2::new(x, y),
        11.0,
        TEXT_DIM,
        Align::Left,
    );
    let names: Vec<&str> = MODES.iter().map(|m| m.name()).collect();
    let current = MODES.iter().position(|m| *m == form.mode).unwrap_or(0);
    if let Some(i) = ui.tabs(
        "mode",
        Vec2::new(x, y + 12.0 * s),
        26.0 * s,
        &names,
        current,
        &[VIOLET],
    ) {
        form.mode = MODES[i];
    }
    y += 60.0 * s;
    ui.toggle(
        "instagib",
        Vec2::new(x, y),
        lang.t("menu.instagib"),
        &mut form.instagib,
    );
    y += 40.0 * s;
    ui.toggle(
        "public",
        Vec2::new(x, y),
        lang.t("menu.public"),
        &mut form.public,
    );
    y += 40.0 * s;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = form.max_clients.round() as u32;
    ui.label(
        &lang.f("menu.max_players", &[("n", &n)]),
        Vec2::new(x, y),
        11.0,
        TEXT_DIM,
        Align::Left,
    );
    ui.slider(
        "max",
        Rect::new(x, y + 10.0 * s, 300.0 * s, 24.0 * s),
        &mut form.max_clients,
        2.0,
        16.0,
    );
    y += 50.0 * s;
    let btn = Rect::new(x, y, 200.0 * s, 34.0 * s);
    ui.button("host", btn, lang.t("menu.start_server"), GREEN)
        .then_some(MenuAction::Host)
}

/// "Settings": sidebar on the left, page on the right ([`crate::menu_settings`]).
fn settings_page(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    area: Rect,
    tab: &mut usize,
    edit: &mut SettingsEdit<'_>,
) -> bool {
    let s = cx.s;
    let lang = cx.lang;
    let side = Rect::new(
        area.min.x + 8.0 * s,
        area.min.y + 8.0 * s,
        170.0 * s,
        236.0 * s,
    );
    ui.card(side);
    let items = [
        lang.t("menu.settings_player"),
        lang.t("menu.settings_controls"),
        lang.t("menu.settings_graphics"),
        lang.t("menu.settings_audio"),
        lang.t("menu.settings_language"),
        lang.t("menu.settings_about"),
    ];
    if let Some(i) = ui.side_tabs("settings_tabs", side.shrink(12.0 * s), &items, *tab, ORANGE) {
        *tab = i;
    }
    let page = Rect::new(side.max.x + 16.0 * s, side.min.y, 560.0 * s, 400.0 * s);
    ui.card(page);
    ui.label(
        items[*tab],
        page.min + Vec2::new(20.0, 26.0) * s,
        16.0,
        TEXT,
        Align::Left,
    );
    crate::menu_settings::page(ui, cx, page, *tab, edit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::{Lang, Language};

    /// Visual inspection: `cargo test -p elora-client --bin elora menu_sheet -- --ignored`,
    /// then per page `cargo xtask svg-preview target/menu-<seite>.svg target/menu-<seite>.png 1280`.
    #[test]
    #[ignore = "only writes files for visual inspection"]
    #[allow(clippy::too_many_lines)] // sample data for all pages
    fn menu_sheet() {
        let font = Font::new(include_bytes!("../../../assets/fonts/Inter-Regular.ttf")).unwrap();
        let lang = Lang::new(Language::De);
        let art = FigureArt::load();
        let map_art = crate::map_art::MapArt::load();
        let creatures = crate::creatures::CreatureArt::load();
        let favorites = vec!["127.0.0.1:8303".to_owned(), "192.168.0.20:8303".to_owned()];
        let maps: Vec<String> = [
            "ctf-nacht",
            "ctf-test",
            "ctf-wald",
            "dm-wiese",
            "dm-winter",
            "dm-wueste",
            "faehigkeiten-test",
            "look-test",
            "sandbox",
            "tiles-test",
            "training",
        ]
        .map(str::to_owned)
        .to_vec();
        for (name, page) in [
            ("spielen", Page::Play),
            ("abenteuer", Page::Adventure),
            ("erstellen", Page::Create),
            ("einstellungen", Page::Settings),
            ("grafik", Page::Settings),
            ("ueber", Page::Settings),
            ("steuerung", Page::Settings),
            ("pause", Page::Play),
        ] {
            let mut browser = crate::browser::Browser::default();
            browser.tab = crate::browser::Tab::Favorites;
            let info = |name: &str,
                        map: &str,
                        mode: &str,
                        clients: u32,
                        players: Vec<elora_protocol::InfoPlayer>| {
                elora_protocol::ServerInfo {
                    version: elora_protocol::PROTOCOL_VERSION,
                    name: name.into(),
                    map: map.into(),
                    mode: mode.into(),
                    clients,
                    max_clients: 8,
                    players,
                }
            };
            let p = |name: &str, score: i32, team: Team, dummy: bool| elora_protocol::InfoPlayer {
                name: name.into(),
                score,
                team,
                dummy,
            };
            let entries = [
                (
                    "127.0.0.1:8303",
                    info(
                        "Eloras Wiese",
                        "ctf-test",
                        "CTF",
                        5,
                        vec![
                            p("Nimbus", 12, Team::Red, false),
                            p("Pip", 7, Team::Blue, false),
                            p("Tropf", 4, Team::Red, false),
                            p("Kiesel", 3, Team::Blue, false),
                            p("Moos", 1, Team::Red, false),
                            p("Dummy 6", 0, Team::Blue, true),
                        ],
                    ),
                    24,
                ),
                (
                    "192.168.0.20:8303",
                    info(
                        "Tropfen-Arena",
                        "sandbox",
                        "DM",
                        3,
                        vec![p("Elora", 9, Team::None, false)],
                    ),
                    41,
                ),
                (
                    "10.0.0.5:8304",
                    info("Nachtschicht", "sandbox", "iDM", 8, vec![]),
                    63,
                ),
            ];
            for (a, i, ping) in entries {
                browser.insert(crate::browser::Entry {
                    addr: a.parse().unwrap(),
                    state: crate::browser::State::Online {
                        info: i,
                        ping: std::time::Duration::from_millis(ping),
                    },
                });
            }
            browser.insert(crate::browser::Entry {
                addr: "10.0.0.9:8303".parse().unwrap(),
                state: crate::browser::State::Unreachable,
            });
            browser.selected = Some("127.0.0.1:8303".parse().unwrap());
            let mut menu = Menu {
                page,
                browser,
                browser_loaded: Some(crate::browser::Tab::Favorites),
                settings_tab: match name {
                    "grafik" => 2,
                    "ueber" => 5,
                    "steuerung" => 1,
                    _ => 0,
                },
                address: "127.0.0.1:8303".into(),
                ..Menu::default()
            };
            let cx = MenuCtx {
                font: &font,
                lang: &lang,
                art: &art,
                map_art: &map_art,
                creatures: &creatures,
                last_server: Some("127.0.0.1:8303"),
                favorites: &favorites,
                maps: &maps,
                status: "",
                slots: &[
                    crate::app_adventure::SlotView::Saved {
                        level: 4,
                        map: "wiese-1".into(),
                        play_secs: 4520,
                        gleam: 128,
                    },
                    crate::app_adventure::SlotView::Empty,
                    crate::app_adventure::SlotView::Damaged("checksum mismatch".into()),
                ],
                screen: Vec2::new(1280.0, 720.0),
                s: 1.0,
                dt: 0.016,
            };
            let mut batch = ShapeBatch::default();
            let mut player = "Elora".to_owned();
            let mut skin = Skin::default();
            let mut graphics = crate::settings::GraphicsSettings::default();
            let mut audio = elora_audio::AudioSettings::default();
            let mut effects = crate::effects::EffectSettings::default();
            let mut sens = 100.0;
            let mut language = Language::De;
            let mut bindings = crate::bindings::Bindings::default();
            let mut capture = None;
            let mut master = String::new();
            let mut edit = SettingsEdit {
                name: &mut player,
                skin: &mut skin,
                graphics: &mut graphics,
                audio: &mut audio,
                effects: &mut effects,
                sensitivity: &mut sens,
                auto_switch: &mut crate::settings::AutoSwitch::New,
                language: &mut language,
                bindings: &mut bindings,
                capture: &mut capture,
                master_url: &mut master,
                audio_device: true,
            };
            if name == "pause" {
                batch.fill_rect(Vec2::default(), cx.screen, Color::hex(0x8fb8d9));
                let names: std::collections::BTreeMap<usize, String> = [
                    (0, "Elora".to_owned()),
                    (1, "Nimbus".to_owned()),
                    (2, "Pip".to_owned()),
                ]
                .into_iter()
                .collect();
                let p = crate::menu_pause::PauseCtx {
                    online: true,
                    team_mode: true,
                    team: Team::Red,
                    names: &names,
                    local: Some(0),
                    vote: None,
                    server_line: "127.0.0.1:8303 · CTF".into(),
                    adventure: false,
                };
                menu.draw_pause(&mut batch, &cx, &p, &mut edit);
            } else {
                menu.draw_main(&mut batch, &cx, &mut edit);
            }
            let svg = batch.debug_svg(Vec2::default(), cx.screen, Color::hex(0x8fb8d9));
            std::fs::write(
                format!(
                    "{}/../../target/menu-{name}.svg",
                    env!("CARGO_MANIFEST_DIR")
                ),
                svg,
            )
            .unwrap();
        }
    }
}

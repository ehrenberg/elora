//! Hauptmenü und Pause-Menü (M7.3) im Stil „Leiste oben, hell & weich“ (E-125).
//!
//! Das Menü zeichnet sich über [`crate::ui`] in Bildschirm-Pixeln und meldet
//! [`MenuAction`]s; die App führt sie aus (Training starten, verbinden, hosten …).

// Layout-Code: `s` (Skalierung), `w`/`h`/`x`/`y` sind hier lesbarer als lange Namen
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

/// Seiten des Hauptmenüs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Play,
    Create,
    Settings,
}

/// Was die App tun soll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    Training,
    Connect(String),
    /// Server mit den Werten aus [`CreateForm`] starten und verbinden.
    Host,
    ToggleFavorite(String),
    /// Einstellungen wurden außerhalb der Einstellungsseiten geändert (speichern).
    SettingsChanged,
    Resume,
    ToMenu,
    Quit,
}

/// Formular „Server erstellen“ (E-122).
#[derive(Debug, Clone)]
pub struct CreateForm {
    pub name: String,
    /// Index in der Kartenliste.
    pub map: usize,
    pub mode: Mode,
    pub instagib: bool,
    pub max_clients: f32,
}

impl Default for CreateForm {
    fn default() -> Self {
        Self {
            name: "Eloras Server".into(),
            map: 0,
            mode: Mode::Dm,
            instagib: false,
            max_clients: 8.0,
        }
    }
}

pub const MODES: [Mode; 5] = [Mode::Dm, Mode::Tdm, Mode::Ctf, Mode::Lms, Mode::Lts];

/// Daten, die das Menü pro Frame braucht.
pub struct MenuCtx<'a> {
    pub font: &'a Font,
    pub lang: &'a Lang,
    pub art: &'a FigureArt,
    pub last_server: Option<&'a str>,
    pub favorites: &'a [String],
    /// Kartennamen für „Server erstellen“.
    pub maps: &'a [String],
    /// Statuszeile (Verbindung, Hosting, Fehler).
    pub status: &'a str,
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
    /// Server-Browser (M7.7) und zuletzt geladener Reiter.
    pub browser: crate::browser::Browser,
    pub browser_loaded: Option<crate::browser::Tab>,
    pub master_input: String,
    pub create: CreateForm,
    /// Pause-Menü im Spiel offen.
    pub paused: bool,
    /// „Beenden“ gewählt; die App beendet sich beim nächsten Durchlauf.
    pub quit: bool,
    /// Einstellungen geändert, Speichern wartet auf das Loslassen der Maus.
    pub save_pending: bool,
    /// Letzter Klick (Zeit, Ort) für die Doppelklick-Erkennung.
    pub last_click: Option<(std::time::Instant, Vec2)>,
}

const TAB_COLORS: [Color; 5] = [GREEN, BLUE, VIOLET, ORANGE, GRAY];

impl Menu {
    /// Hauptmenü zeichnen; liefert eine Aktion und ob Einstellungen geändert wurden.
    pub fn draw_main(
        &mut self,
        batch: &mut ShapeBatch,
        cx: &MenuCtx<'_>,
        edit: &mut SettingsEdit<'_>,
    ) -> (Option<MenuAction>, bool) {
        background(batch, cx, *edit.skin);
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

    /// Pause-Menü über dem Spiel.
    pub fn draw_pause(&mut self, batch: &mut ShapeBatch, cx: &MenuCtx<'_>) -> Option<MenuAction> {
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
        let w = 260.0 * s;
        let h = 220.0 * s;
        let card = Rect::new((cx.screen.x - w) / 2.0, (cx.screen.y - h) / 2.0, w, h);
        ui.card(card);
        let lang = cx.lang;
        ui.label(
            lang.t("menu.paused"),
            Vec2::new(card.center().x, card.min.y + 30.0 * s),
            20.0,
            TEXT,
            Align::Center,
        );
        let mut action = None;
        let items = [
            ("menu.resume", GREEN, MenuAction::Resume),
            ("menu.to_menu", ORANGE, MenuAction::ToMenu),
            ("menu.quit", GRAY, MenuAction::Quit),
        ];
        for (i, (key, color, act)) in items.into_iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let r = Rect::new(
                card.min.x + 30.0 * s,
                card.min.y + (62.0 + i as f32 * 48.0) * s,
                w - 60.0 * s,
                34.0 * s,
            );
            if ui.button(key, r, lang.t(key), color) {
                action = Some(act);
            }
        }
        ui.end();
        self.input.next_frame();
        action
    }
}

/// Ruhiges Hintergrundbild (E-113): Himmel, Wolken, Hügel, zwei Eloras.
fn background(batch: &mut ShapeBatch, cx: &MenuCtx<'_>, skin: Skin) {
    let (w, h, s) = (cx.screen.x, cx.screen.y, cx.s);
    batch.fill_rect_vgradient(
        Vec2::default(),
        cx.screen,
        Color::hex(0xa9cde8),
        Color::hex(0xe8f1f7),
    );
    let cloud = Color::rgba(1.0, 1.0, 1.0, 0.8);
    for (x, y, r) in [(0.55, 0.16, 1.0), (0.82, 0.12, 1.3), (0.2, 0.3, 0.8)] {
        let c = Vec2::new(w * x, h * y);
        let r = 26.0 * s * r;
        batch.fill_circle(c, r, cloud);
        batch.fill_circle(c + Vec2::new(r * 0.9, r * 0.25), r * 0.75, cloud);
        batch.fill_circle(c - Vec2::new(r * 0.9, -r * 0.3), r * 0.7, cloud);
    }
    batch.fill_circle(
        Vec2::new(w * 0.25, h + 240.0 * s),
        420.0 * s,
        Color::hex(0x8fbf7a),
    );
    batch.fill_circle(
        Vec2::new(w * 0.85, h + 260.0 * s),
        440.0 * s,
        Color::hex(0x7aae6a),
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

/// Leiste oben: Logo und Reiter; Training und Beenden sind sofortige Aktionen.
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
        lang.t("menu.training"),
        lang.t("menu.create"),
        lang.t("menu.settings"),
        lang.t("menu.quit"),
    ];
    let selected = match page {
        Page::Play => 0,
        Page::Create => 2,
        Page::Settings => 3,
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
        Some(1) => return Some(MenuAction::Training),
        Some(2) => *page = Page::Create,
        Some(3) => *page = Page::Settings,
        Some(4) => return Some(MenuAction::Quit),
        _ => {}
    }
    None
}

/// „Spielen“: Begrüßung und „Schnell spielen“ links, Server-Browser rechts (M7.7).
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

/// Karte „Schnell spielen“: letzter Server mit „Los!“.
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

/// „Server erstellen“ (E-122): Name, Karte, Modus, Instagib, Spielerzahl.
fn create_page(
    ui: &mut Ui<'_>,
    cx: &MenuCtx<'_>,
    area: Rect,
    form: &mut CreateForm,
) -> Option<MenuAction> {
    let s = cx.s;
    let lang = cx.lang;
    let card = Rect::new(
        area.min.x + 8.0 * s,
        area.min.y + 8.0 * s,
        520.0 * s,
        360.0 * s,
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
    let maps: Vec<&str> = cx.maps.iter().map(String::as_str).collect();
    if let Some(i) = ui.tabs(
        "map",
        Vec2::new(x, y + 12.0 * s),
        26.0 * s,
        &maps,
        form.map,
        &[BLUE],
    ) {
        form.map = i;
    }
    y += 60.0 * s;
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

/// „Einstellungen“: Seitenleiste links, Seite rechts ([`crate::menu_settings`]).
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
        200.0 * s,
    );
    ui.card(side);
    let items = [
        lang.t("menu.settings_player"),
        lang.t("menu.settings_controls"),
        lang.t("menu.settings_graphics"),
        lang.t("menu.settings_audio"),
        lang.t("menu.settings_language"),
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

    /// Sichtprüfung: `cargo test -p elora-client --bin elora menu_sheet -- --ignored`,
    /// danach je Seite `cargo xtask svg-preview target/menu-<seite>.svg target/menu-<seite>.png 1280`.
    #[test]
    #[ignore = "erzeugt nur Dateien zur Sichtprüfung"]
    #[allow(clippy::too_many_lines)] // Beispieldaten für alle Seiten
    fn menu_sheet() {
        let font = Font::new(include_bytes!("../../../assets/fonts/Inter-Regular.ttf")).unwrap();
        let lang = Lang::new(Language::De);
        let art = FigureArt::load();
        let favorites = vec!["127.0.0.1:8303".to_owned(), "192.168.0.20:8303".to_owned()];
        let maps = vec!["ctf-test".to_owned(), "sandbox".to_owned()];
        for (name, page) in [
            ("spielen", Page::Play),
            ("erstellen", Page::Create),
            ("einstellungen", Page::Settings),
            ("grafik", Page::Settings),
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
                last_server: Some("127.0.0.1:8303"),
                favorites: &favorites,
                maps: &maps,
                status: "",
                screen: Vec2::new(1280.0, 720.0),
                s: 1.0,
                dt: 0.016,
            };
            let mut batch = ShapeBatch::default();
            if name == "pause" {
                batch.fill_rect(Vec2::default(), cx.screen, Color::hex(0x8fb8d9));
                menu.draw_pause(&mut batch, &cx);
            } else {
                let mut name = "Elora".to_owned();
                let mut skin = Skin::default();
                let mut graphics = crate::settings::GraphicsSettings::default();
                let mut audio = elora_audio::AudioSettings::default();
                let mut effects = crate::effects::EffectSettings::default();
                let mut sens = 100.0;
                let mut language = Language::De;
                let mut bindings = crate::bindings::Bindings::default();
                let mut capture = None;
                let mut edit = SettingsEdit {
                    name: &mut name,
                    skin: &mut skin,
                    graphics: &mut graphics,
                    audio: &mut audio,
                    effects: &mut effects,
                    sensitivity: &mut sens,
                    language: &mut language,
                    bindings: &mut bindings,
                    capture: &mut capture,
                    master_url: &mut String::new(),
                    audio_device: true,
                };
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

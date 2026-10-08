// Windows: no console window next to the game in release builds
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

//! Elora client: local sandbox (M1/M2) or online with a server (M3).
//!
//! Usage: `elora [karte.emap] [--mode dm|tdm|ctf|lms|lts] [--instagib] [--connect adresse:port]`
//! (default map: `maps/training.emap`)

mod adventure_hud;
mod adventure_menu;
mod app_adventure;
mod app_debug;
mod app_editor;
mod app_frame;
mod app_input;
mod app_intro;
mod app_look;
mod app_menu;
mod app_net;
mod app_sound;
mod app_state;
mod app_window;
mod asset_files;
mod bindings;
mod browser;
mod connection;
mod controls;
mod creatures;
mod debug_ui;
mod draw;
mod editor;
mod effects;
mod emotes;
mod figure;
mod game_ui;
mod gui;
mod hosting;
mod hud;
mod intro;
mod items;
mod lang;
mod map_art;
mod map_view;
mod menu;
mod menu_adventure;
mod menu_browser;
mod menu_pause;
mod menu_scene;
mod menu_settings;
mod menu_setup;
mod sandbox;
mod settings;
mod skins;
mod sound;
mod startup;
mod tuning_file;
mod ui;
mod weather;

use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context as _;
use elora_client::online::OnlineClient;
use elora_render::{Renderer, ShapeBatch, ViewSettings};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use bindings::GameAction;
use connection::{Connection, KnownServers};
use controls::Controls;
use debug_ui::NetUi;
use gui::Gui;
use sandbox::Sandbox;
use settings::Settings;
use tuning_file::TuningFile;

const DEFAULT_MAP: &str = "maps/training.emap";

/// Command line argument is a map file.
fn is_map_path(arg: &str) -> bool {
    Path::new(arg)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(elora_map::EXTENSION))
}

fn main() {
    startup::init_logging();
    startup::install_panic_hook();
    if let Err(e) = run() {
        startup::report_fatal(&format!("{e:#}"));
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let connect = args
        .iter()
        .position(|a| a == "--connect")
        .and_then(|i| args.get(i + 1).cloned());
    let map_path = args
        .iter()
        .find(|a| is_map_path(a))
        .map_or_else(|| PathBuf::from(DEFAULT_MAP), PathBuf::from);
    if !elora_server::paths::data_found() {
        let searched: Vec<String> = elora_server::paths::candidates()
            .iter()
            .map(|p| format!("  {}", p.display()))
            .collect();
        anyhow::bail!(
            "Spieldaten nicht gefunden (Ordner „maps“). Bitte das ZIP vollständig entpacken und \
             elora.exe aus dem entpackten Ordner starten.\n\
             Game data not found (folder “maps”). Please extract the whole ZIP and start \
             elora.exe from the extracted folder.\n\nGesucht in / searched in:\n{}",
            searched.join("\n")
        );
    }
    let map_path = elora_server::paths::resolve(&map_path);
    settings::migrate_user_maps();
    let file = TuningFile::load(&app_state::tuning_path())?;
    let settings = Settings::load(&settings::settings_path()).unwrap_or_else(|e| {
        tracing::warn!("{e:#} – Standard-Einstellungen");
        Settings::default()
    });
    let mut sandbox = Sandbox::load(&map_path, file.physics.clone())?;
    if let Some(i) = args.iter().position(|a| a == "--mode") {
        let name = args
            .get(i + 1)
            .context("--mode expects dm, tdm, ctf, lms or lts")?;
        let mode =
            elora_game::Mode::parse(name).with_context(|| format!("unknown mode `{name}`"))?;
        let instagib = args.iter().any(|a| a == "--instagib");
        sandbox.set_mode(Some(elora_game::RulesConfig {
            mode,
            instagib,
            warmup_secs: 0,
            ..elora_game::RulesConfig::default()
        }));
    }
    tracing::info!(map = %map_path.display(), "map loaded");

    let event_loop = EventLoop::new().context("could not create the event loop")?;
    event_loop.set_control_flow(ControlFlow::Poll);
    // map, mode or address on the command line: straight into the game (development)
    let direct = connect.is_some() || args.iter().any(|a| is_map_path(a) || a == "--mode");
    let mut app = App::new(sandbox, &file, settings);
    if direct {
        app.screen = Screen::Game;
    }
    if let Some(address) = connect {
        app.net.address = address;
        app.connect();
    }
    // development: straight into the adventure in slot 1–3 (continue or new)
    if let Some(i) = args.iter().position(|a| a == "--adventure") {
        let slot: usize = args
            .get(i + 1)
            .and_then(|n| n.parse().ok())
            .filter(|n| (1..=3).contains(n))
            .context("--adventure expects a slot 1, 2 or 3")?;
        let new = matches!(
            app_adventure::slot_views()[slot - 1],
            app_adventure::SlotView::Empty
        );
        app.start_adventure(slot - 1, new);
    }
    event_loop.run_app(&mut app)?;
    app.error.map_or(Ok(()), Err)
}

/// Window and graphics only exist after `resumed`.
struct Gfx {
    window: Arc<Window>,
    renderer: Renderer,
    gui: Gui,
}

/// Running online game.
struct Online {
    client: OnlineClient,
    conn: Connection,
}

/// What is currently visible (M7.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    /// Main menu (start screen, E-113).
    Menu,
    /// Training (sandbox) or online game; the pause menu lies on top.
    Game,
    /// Map editor (M6.6).
    Editor,
}

/// Game information of a frame for HUD, displays and panel.
struct FrameInfo {
    names: BTreeMap<usize, String>,
    teams: BTreeMap<usize, elora_sim::Team>,
    view: Option<elora_protocol::GameView>,
    local: Option<usize>,
    tick: u64,
    vote: Option<elora_protocol::VoteInfo>,
    chat: Vec<elora_client::online::ChatLine>,
    /// Map is loading: name, received, total (M6.5).
    loading: Option<(String, usize, usize)>,
}

#[allow(clippy::struct_excessive_bools)] // independent states of the application
struct App {
    gfx: Option<Gfx>,
    sandbox: Sandbox,
    online: Option<Online>,
    known: KnownServers,
    net: NetUi,
    controls: Controls,
    view: ViewSettings,
    batch: ShapeBatch,
    /// HUD in screen pixels (M5.8).
    hud_batch: ShapeBatch,
    hud: hud::Hud,
    emotes: emotes::Emotes,
    sounds: sound::Sounds,
    /// Time of the newest chat line for which a sound was already played.
    chat_heard: Option<Instant>,
    /// Sounds from the client itself (babble sounds, windows), played in the next frame.
    ui_cues: Vec<elora_audio::Cue>,
    /// Weapons of the own character in the last frame (switch on pickup, E-287).
    owned_weapons: [bool; 3],
    /// Own character was in the colourful rush in the last frame (sound at the start).
    was_dazed: bool,
    /// Saturation of the world (E-328): follows the freed springs, glides smoothly after them.
    saturation: f32,
    /// Weather in the rendering (R2-W1).
    weather: weather::WeatherView,
    effects: effects::Effects,
    figures: figure::Figures,
    figure_art: figure::FigureArt,
    item_art: items::ItemArt,
    creature_art: creatures::CreatureArt,
    map_view: map_view::MapView,
    /// Empty skin table for the sandbox.
    no_skins: BTreeMap<usize, elora_protocol::Skin>,
    last_frame: Instant,
    fps: f32,
    /// Last frame times in ms (display in the debug panel, E-288).
    frame_times: VecDeque<f32>,
    cursor_grabbed: bool,
    show_panel: bool,
    status: String,
    error: Option<anyhow::Error>,
    chat_input: game_ui::ChatInput,
    scoreboard: bool,
    killfeed: VecDeque<game_ui::KillEntry>,
    /// Player settings (`settings.toml`, E-116) and language (E-114).
    settings: Settings,
    lang: lang::Lang,
    screen: Screen,
    editor: Option<editor::Editor>,
    /// Test game from the editor: training map before it (M6.9).
    editor_test: Option<elora_map::Map>,
    /// Map area of the editor from the last frame.
    editor_area: editor::panel::AreaInfo,
    menu: menu::Menu,
    /// Map names for “Create server”.
    maps: Vec<String>,
    /// Action currently being rebound (next key counts).
    bind_capture: Option<GameAction>,
    /// Running adventure (A1.6).
    adventure: Option<app_adventure::AdventureMode>,
    /// Intro video before a new adventure (E-355); the game waits while it runs.
    intro: Option<intro::Intro>,
}

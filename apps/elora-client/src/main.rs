// Windows: im Release kein Konsolenfenster neben dem Spiel
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

//! Elora-Client: lokale Sandbox (M1/M2) oder online mit einem Server (M3).
//!
//! Aufruf: `elora [karte.emap] [--mode dm|tdm|ctf|lms|lts] [--instagib] [--connect adresse:port]`
//! (Standardkarte: `maps/training.emap`)

mod adventure_hud;
mod adventure_menu;
mod app_adventure;
mod app_editor;
mod app_menu;
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
mod sandbox;
mod settings;
mod skins;
mod sound;
mod tuning_file;
mod ui;

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use elora_client::online::{OnlineClient, Status};
use elora_client::scene::Scene;
use elora_net::{ClientEvent, DisconnectReason};
use elora_render::{Camera, Renderer, ShapeBatch, ViewSettings};
use elora_sim::{Collision, Event, Tuning, Vec2};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use bindings::{GameAction, Trigger};
use connection::{Connection, KNOWN_SERVERS_FILE, KnownServers};
use controls::Controls;
use debug_ui::{Action, KeyWarning, NetUi, OnlineView};
use gui::Gui;
use sandbox::Sandbox;
use settings::Settings;
use tuning_file::{TUNING_FILE, TuningFile};

/// Musik, solange ein Hüter wach ist (`assets/music/boss.ogg`).
const BOSS_MUSIC: &str = "boss";
const DEFAULT_MAP: &str = "maps/training.emap";

/// Tuning- und Server-Schlüssel-Dateien im Einstellungsordner (M8.2).
fn tuning_path() -> PathBuf {
    settings::config_file(TUNING_FILE)
}

fn known_servers_path() -> PathBuf {
    settings::config_file(KNOWN_SERVERS_FILE)
}

/// Kommandozeilen-Argument ist eine Kartendatei.
fn is_map_path(arg: &str) -> bool {
    Path::new(arg)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(elora_map::EXTENSION))
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,wgpu_core=warn,wgpu_hal=warn".into()),
        )
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let connect = args
        .iter()
        .position(|a| a == "--connect")
        .and_then(|i| args.get(i + 1).cloned());
    let map_path = args
        .iter()
        .find(|a| is_map_path(a))
        .map_or_else(|| PathBuf::from(DEFAULT_MAP), PathBuf::from);
    let map_path = elora_server::paths::resolve(&map_path);
    let file = TuningFile::load(&tuning_path())?;
    let settings = Settings::load(&settings::settings_path()).unwrap_or_else(|e| {
        tracing::warn!("{e:#} – Standard-Einstellungen");
        Settings::default()
    });
    let mut sandbox = Sandbox::load(&map_path, file.physics.clone())?;
    if let Some(i) = args.iter().position(|a| a == "--mode") {
        let name = args
            .get(i + 1)
            .context("--mode erwartet dm, tdm, ctf, lms oder lts")?;
        let mode =
            elora_game::Mode::parse(name).with_context(|| format!("unbekannter Modus `{name}`"))?;
        let instagib = args.iter().any(|a| a == "--instagib");
        sandbox.set_mode(Some(elora_game::RulesConfig {
            mode,
            instagib,
            warmup_secs: 0,
            ..elora_game::RulesConfig::default()
        }));
    }
    tracing::info!(map = %map_path.display(), "Karte geladen");

    let event_loop = EventLoop::new().context("Event-Loop konnte nicht erstellt werden")?;
    event_loop.set_control_flow(ControlFlow::Poll);
    // Karte, Modus oder Adresse auf der Kommandozeile: direkt ins Spiel (Entwicklung)
    let direct = connect.is_some() || args.iter().any(|a| is_map_path(a) || a == "--mode");
    let mut app = App::new(sandbox, &file, settings);
    if direct {
        app.screen = Screen::Game;
    }
    if let Some(address) = connect {
        app.net.address = address;
        app.connect();
    }
    // Entwicklung: direkt ins Abenteuer auf Platz 1–3 (fortsetzen oder neu)
    if let Some(i) = args.iter().position(|a| a == "--abenteuer") {
        let slot: usize = args
            .get(i + 1)
            .and_then(|n| n.parse().ok())
            .filter(|n| (1..=3).contains(n))
            .context("--abenteuer erwartet einen Platz 1, 2 oder 3")?;
        let new = matches!(
            app_adventure::slot_views()[slot - 1],
            app_adventure::SlotView::Empty
        );
        app.start_adventure(slot - 1, new);
    }
    event_loop.run_app(&mut app)?;
    app.error.map_or(Ok(()), Err)
}

/// Fenster und Grafik existieren erst nach `resumed`.
struct Gfx {
    window: Arc<Window>,
    renderer: Renderer,
    gui: Gui,
}

/// Laufendes Online-Spiel.
struct Online {
    client: OnlineClient,
    conn: Connection,
}

/// Was gerade zu sehen ist (M7.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    /// Hauptmenü (Startbildschirm, E-113).
    Menu,
    /// Training (Sandbox) oder Online-Spiel; das Pause-Menü liegt darüber.
    Game,
    /// Karten-Editor (M6.6).
    Editor,
}

/// Spielinformationen eines Frames für HUD, Anzeigen und Panel.
struct FrameInfo {
    names: std::collections::BTreeMap<usize, String>,
    teams: std::collections::BTreeMap<usize, elora_sim::Team>,
    view: Option<elora_protocol::GameView>,
    local: Option<usize>,
    tick: u64,
    vote: Option<elora_protocol::VoteInfo>,
    chat: Vec<elora_client::online::ChatLine>,
    /// Karte wird geladen: Name, empfangen, gesamt (M6.5).
    loading: Option<(String, usize, usize)>,
}

#[allow(clippy::struct_excessive_bools)] // unabhängige Zustände der Anwendung
struct App {
    gfx: Option<Gfx>,
    sandbox: Sandbox,
    online: Option<Online>,
    known: KnownServers,
    net: NetUi,
    controls: Controls,
    view: ViewSettings,
    batch: ShapeBatch,
    /// HUD in Bildschirm-Pixeln (M5.8).
    hud_batch: ShapeBatch,
    hud: hud::Hud,
    emotes: emotes::Emotes,
    sounds: sound::Sounds,
    /// Zeitpunkt der neuesten Chat-Zeile, für die schon ein Sound kam.
    chat_heard: Option<Instant>,
    /// Klänge aus dem Client selbst (Plapperlaute, Fenster), abgespielt im nächsten Frame.
    ui_cues: Vec<elora_audio::Cue>,
    /// Waffen der eigenen Figur im letzten Frame (Wechsel beim Aufheben, E-287).
    owned_weapons: [bool; 3],
    /// Eigene Figur war im letzten Frame im bunten Rausch (Klang beim Beginn).
    was_dazed: bool,
    effects: effects::Effects,
    figures: figure::Figures,
    figure_art: figure::FigureArt,
    item_art: items::ItemArt,
    creature_art: creatures::CreatureArt,
    map_view: map_view::MapView,
    /// Leere Skin-Tabelle für die Sandbox.
    no_skins: std::collections::BTreeMap<usize, elora_protocol::Skin>,
    last_frame: Instant,
    fps: f32,
    /// Letzte Bildzeiten in ms (Anzeige im Debug-Panel, E-288).
    frame_times: std::collections::VecDeque<f32>,
    cursor_grabbed: bool,
    show_panel: bool,
    status: String,
    error: Option<anyhow::Error>,
    chat_input: game_ui::ChatInput,
    scoreboard: bool,
    killfeed: VecDeque<game_ui::KillEntry>,
    /// Spieler-Einstellungen (`settings.toml`, E-116) und Sprache (E-114).
    settings: Settings,
    lang: lang::Lang,
    screen: Screen,
    editor: Option<editor::Editor>,
    /// Testspiel aus dem Editor: Trainingskarte davor (M6.9).
    editor_test: Option<elora_map::Map>,
    /// Kartenfläche des Editors aus dem letzten Frame.
    editor_area: editor::panel::AreaInfo,
    menu: menu::Menu,
    /// Kartennamen für „Server erstellen“.
    maps: Vec<String>,
    /// Aktion, die gerade neu belegt wird (nächste Taste zählt).
    bind_capture: Option<GameAction>,
    /// Laufendes Abenteuer (A1.6).
    adventure: Option<app_adventure::AdventureMode>,
}

impl App {
    fn new(sandbox: Sandbox, file: &TuningFile, settings: Settings) -> Self {
        let mut controls = Controls::default();
        controls.sensitivity = settings.input.mouse_sensitivity;
        controls.auto_switch = settings.input.auto_switch;
        let mut net = NetUi::default();
        net.name.clone_from(&settings.player.name);
        net.skin = settings.player.skin();
        if let Some(r) = &sandbox.rules {
            net.sandbox_mode = Some(r.cfg.mode);
            net.sandbox_instagib = r.cfg.instagib;
        }
        Self {
            gfx: None,
            sandbox,
            online: None,
            known: KnownServers::load(&known_servers_path()),
            net,
            controls,
            view: file.view.into(),
            batch: ShapeBatch::default(),
            hud_batch: ShapeBatch::default(),
            hud: hud::Hud::new(),
            emotes: emotes::Emotes::new(),
            sounds: sound::Sounds::new(settings.audio),
            chat_heard: None,
            ui_cues: Vec::new(),
            owned_weapons: [false; 3],
            was_dazed: false,
            effects: effects::Effects::with_settings(settings.effects),
            figures: figure::Figures::default(),
            figure_art: figure::FigureArt::load(),
            item_art: items::ItemArt::load(),
            creature_art: creatures::CreatureArt::load(),
            map_view: map_view::MapView::default(),
            no_skins: std::collections::BTreeMap::new(),
            last_frame: Instant::now(),
            fps: 0.0,
            frame_times: std::collections::VecDeque::new(),
            cursor_grabbed: false,
            show_panel: false,
            status: String::new(),
            error: None,
            chat_input: game_ui::ChatInput::default(),
            scoreboard: false,
            killfeed: VecDeque::new(),
            lang: lang::Lang::new(settings.language),
            settings,
            screen: Screen::Menu,
            editor: None,
            editor_test: None,
            editor_area: editor::panel::AreaInfo::default(),
            menu: menu::Menu::default(),
            maps: app_menu::map_names(),
            bind_capture: None,
            adventure: None,
        }
    }

    /// Aktuelle Werte aus Spiel und Panel in die Einstellungen übernehmen und speichern.
    fn save_settings(&mut self) {
        let s = &mut self.settings;
        s.player.name.clone_from(&self.net.name);
        s.player.set_skin(self.net.skin);
        s.input.mouse_sensitivity = self.controls.sensitivity;
        s.input.auto_switch = self.controls.auto_switch;
        s.effects = self.effects.settings;
        s.audio = self.sounds.settings;
        let path = settings::settings_path();
        match s.save(&path) {
            Ok(()) => tracing::info!("Einstellungen gespeichert: {}", path.display()),
            Err(e) => tracing::warn!("{e:#}"),
        }
    }

    fn tuning_file(&self) -> TuningFile {
        TuningFile {
            physics: self.sandbox.world.tuning.clone(),
            view: self.view.into(),
        }
    }

    fn connect(&mut self) {
        let address = self.net.address.trim().to_owned();
        let expected = address
            .parse()
            .ok()
            .and_then(|a| self.known.get(a))
            .or_else(|| {
                std::net::ToSocketAddrs::to_socket_addrs(&address)
                    .ok()?
                    .next()
                    .and_then(|a| self.known.get(a))
            });
        match Connection::open(&address, expected, self.net.conditions()) {
            Ok(conn) => {
                self.status = format!("Verbinde mit {} …", conn.server);
                let client = OnlineClient::new(&self.net.name, self.net.skin, Instant::now())
                    .with_store(Box::new(elora_client::map_store::DiskStore {
                        maps_dirs: hosting::map_dirs(),
                        download_dir: settings::data_dir()
                            .map_or_else(|| PathBuf::from("downloads"), |d| d.join("downloads")),
                    }));
                self.online = Some(Online { client, conn });
                self.sandbox.stop_recording("Online");
            }
            Err(e) => self.status = format!("Verbindung fehlgeschlagen: {e:#}"),
        }
    }

    fn apply(&mut self, action: Action) {
        let path = &tuning_path();
        match action {
            Action::Save => {
                self.status = match self.tuning_file().save(path) {
                    Ok(()) => format!("Gespeichert in {TUNING_FILE}"),
                    Err(e) => format!("Fehler: {e:#}"),
                };
            }
            Action::SwitchMap(path) => {
                self.status = match self.sandbox.switch_map(&path) {
                    Ok(()) => format!("Karte {} geladen", path.display()),
                    Err(e) => format!("Fehler: {e:#}"),
                };
            }
            Action::Load => {
                self.status = match TuningFile::load(path) {
                    Ok(f) => {
                        self.sandbox.world.tuning = f.physics;
                        self.view = f.view.into();
                        format!("Geladen aus {TUNING_FILE}")
                    }
                    Err(e) => format!("Fehler: {e:#}"),
                };
            }
            Action::Respawn => {
                self.status = self.sandbox.stop_recording("Respawn").unwrap_or_default();
                self.sandbox.spawn_now();
            }
            Action::Connect => self.connect(),
            Action::Disconnect => {
                self.online = None;
                self.status = "Getrennt – zurück in der Sandbox".into();
            }
            Action::HostStart => match self.net.hosting.start() {
                Ok(address) => {
                    self.online = None;
                    self.net.address = address;
                    self.net.show_host = false;
                    self.connect();
                }
                Err(e) => self.net.hosting.status = format!("Fehler: {e:#}"),
            },
            Action::HostStop => self.net.hosting.stop(),
            Action::TrustNewKey => {
                if let Some(w) = self.net.key_warning.take() {
                    let got: Vec<u8> = (0..w.got.len())
                        .step_by(2)
                        .filter_map(|i| u8::from_str_radix(&w.got[i..i + 2], 16).ok())
                        .collect();
                    if let Err(e) = self.known.trust(&known_servers_path(), w.server, &got) {
                        self.status = format!("Fehler: {e:#}");
                    }
                    self.connect();
                }
            }
            Action::SetTeam(team) => match &mut self.online {
                Some(o) => o.client.set_team(team),
                None => self.sandbox.set_team(team),
            },
            Action::Kill => match &mut self.online {
                Some(o) => o.client.kill(),
                None => self.sandbox.kill(),
            },
            Action::CallVote(kind) => {
                if let Some(o) = &mut self.online {
                    o.client.call_vote(kind);
                }
            }
            Action::Vote(yes) => {
                if let Some(o) = &mut self.online {
                    o.client.vote(yes);
                }
            }
            Action::SandboxMode(cfg) => {
                self.status = cfg.as_ref().map_or_else(
                    || "Freies Spiel".into(),
                    |c| format!("Modus: {}", c.title()),
                );
                self.sandbox.set_mode(cfg);
            }
            Action::ApplyConditions => {
                if let Some(o) = &self.online {
                    o.conn.set_conditions(self.net.conditions());
                }
            }
        }
    }

    fn set_cursor_grab(&mut self, grab: bool) {
        let Some(gfx) = &self.gfx else { return };
        let window = &gfx.window;
        if grab {
            let ok = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
                .is_ok();
            if !ok {
                tracing::warn!("Mauszeiger konnte nicht gefangen werden");
            }
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            self.controls.release_all();
        }
        window.set_cursor_visible(!grab);
        self.cursor_grabbed = grab;
    }

    /// Netzwerk-Ereignisse verarbeiten und Online-Client vorantreiben.
    fn update_online(&mut self, now: Instant) {
        let Some(o) = &mut self.online else { return };
        let mut ended = None;
        for (event, at) in o.conn.events() {
            match event {
                ClientEvent::Connected { server_key } => {
                    if self.known.get(o.conn.server).is_none()
                        && let Err(e) =
                            self.known
                                .trust(&known_servers_path(), o.conn.server, &server_key)
                    {
                        tracing::warn!("Server-Schlüssel nicht gespeichert: {e:#}");
                    }
                    o.client.on_connected();
                    self.status = format!("Verbunden mit {}", o.conn.server);
                }
                ClientEvent::Message { data, .. } => o.client.on_message(&data, at),
                ClientEvent::Disconnected(DisconnectReason::KeyMismatch { expected, got }) => {
                    self.net.key_warning = Some(KeyWarning {
                        server: o.conn.server,
                        expected: elora_net::hex(&expected),
                        got: elora_net::hex(&got),
                    });
                    ended = Some(self.lang.t("reason.key_changed").to_owned());
                }
                ClientEvent::Disconnected(reason) => {
                    ended = Some(match reason {
                        DisconnectReason::Timeout => self.lang.t("reason.timeout").to_owned(),
                        DisconnectReason::Remote(r) | DisconnectReason::Rejected(r) => {
                            self.lang.reason(&r)
                        }
                        other => format!("{other:?}"),
                    });
                }
            }
        }
        if let Status::Disconnected(reason) = &o.client.status {
            ended = ended.or_else(|| Some(self.lang.reason(reason)));
        }
        if let Some(reason) = ended {
            self.status = format!("Getrennt: {reason}");
            self.online = None;
            return;
        }
        let controls = &mut self.controls;
        o.client.update(now, || controls.player_input());
        for (data, reliable) in o.client.take_outgoing() {
            o.conn.send(data, reliable);
        }
    }

    /// Simulation bzw. Online-Client vorantreiben; liefert, was zu zeichnen ist.
    fn advance(
        &mut self,
        now: Instant,
        elapsed: Duration,
    ) -> Option<(Scene, Collision, Tuning, Vec<elora_sim::Event>)> {
        Some(if self.online.is_some() {
            self.update_online(now);
            let o = self.online.as_mut()?;
            match (
                o.client.scene(now),
                o.client.map.as_ref(),
                o.client.tuning(),
            ) {
                (Some(scene), Some(map), Some(t)) => {
                    (scene, map.collision(), t.clone(), o.client.take_events())
                }
                _ => (
                    Scene::default(),
                    Collision::new(1, 1, vec![elora_sim::Tile::Air]),
                    Tuning::default(),
                    Vec::new(),
                ),
            }
        } else if self.adventure.is_some() {
            self.advance_adventure(elapsed);
            let mut scene = self.sandbox.scene();
            self.adventure_scene(&mut scene);
            (
                scene,
                self.sandbox.world.collision.clone(),
                self.sandbox.world.tuning.clone(),
                self.sandbox.take_events(),
            )
        } else {
            self.sandbox.poll_reload();
            if self
                .sandbox
                .recording
                .as_ref()
                .is_some_and(|r| r.tuning != self.sandbox.world.tuning)
                && let Some(msg) = self.sandbox.stop_recording("Tuning geändert")
            {
                self.status = msg;
            }
            self.sandbox.advance(elapsed, &mut self.controls);
            (
                self.sandbox.scene(),
                self.sandbox.world.collision.clone(),
                self.sandbox.world.tuning.clone(),
                self.sandbox.take_events(),
            )
        })
    }

    /// Namen, Teams, Spielzustand, eigener Slot, Tick, Abstimmung und Chat des Frames.
    fn game_info(&self, now: Instant) -> FrameInfo {
        match &self.online {
            Some(o) => FrameInfo {
                names: o.client.names.clone(),
                teams: o.client.teams(),
                view: o.client.game(),
                local: o.client.slot,
                tick: o.client.server_tick(now).unwrap_or(0),
                vote: o.client.vote.clone(),
                chat: o.client.chat.iter().cloned().collect(),
                loading: match &o.client.status {
                    Status::Loading {
                        map,
                        received,
                        size,
                    } => Some((map.clone(), *received, *size)),
                    _ => None,
                },
            },
            None => FrameInfo {
                names: self.sandbox.names(),
                teams: self
                    .sandbox
                    .world
                    .players
                    .iter()
                    .enumerate()
                    .filter_map(|(i, p)| Some((i, p.as_ref()?.team)))
                    .collect(),
                view: self.sandbox.game(),
                local: Some(self.sandbox.player),
                tick: self.sandbox.world.tick,
                vote: None,
                chat: self.sandbox.notices.iter().cloned().collect(),
                loading: None,
            },
        }
    }

    /// Zeit seit dem letzten Frame; aktualisiert die FPS-Anzeige.
    fn frame_time(&mut self, now: Instant) -> Duration {
        let elapsed = now - self.last_frame;
        self.last_frame = now;
        if elapsed.as_secs_f32() > 0.0 {
            self.fps = self.fps * 0.95 + (1.0 / elapsed.as_secs_f32()) * 0.05;
        }
        if self.frame_times.len() >= 240 {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(elapsed.as_secs_f32() * 1000.0);
        elapsed
    }

    /// Effekte und Figuren-Animationen fortschreiben, eigenen Skin abgleichen.
    fn update_looks(&mut self, dt: f32, scene: &Scene, collision: &Collision, events: &[Event]) {
        self.figures.update(dt, scene, collision, events);
        self.emotes.update(dt);
        if let Some(o) = &mut self.online {
            for (slot, emote) in o.client.take_emotes() {
                self.emotes.show(slot, emote);
            }
        }
        let skins = self
            .online
            .as_ref()
            .map_or(&self.no_skins, |o| &o.client.skins);
        let own = self.net.skin;
        self.effects
            .update(dt, events, scene, self.figures.landings(), |c| {
                let skin = if c.local {
                    own
                } else {
                    skins.get(&c.slot).copied().unwrap_or_default()
                };
                skins::tint(skin, c.team, c.dummy, draw::team_color).colors[figure::KEY_BODY]
            });
        // Leuchtpilze in der Nähe der Kamera lassen Sternchen aufsteigen (R2-M2.2)
        if self.online.is_none() {
            let near: Vec<elora_sim::Vec2> = self
                .sandbox
                .map
                .decor_front
                .iter()
                .chain(&self.sandbox.map.decor_back)
                .filter(|d| d.art == elora_map::Art::Builtin("leuchtpilze".into()))
                .map(|d| d.pos)
                .filter(|p| p.distance(scene.camera) < 1100.0)
                .collect();
            self.effects.glow_spores(dt, &near);
        }
        if let Some(o) = &mut self.online {
            // schickt nur bei Änderung eine Nachricht
            o.client.set_skin(self.net.skin);
        }
        self.auto_switch(scene, events);
        self.play_sounds(scene, events);
    }

    /// Zur aufgenommenen Waffe wechseln (E-287); `owned_weapons` merkt den Stand vor dem Frame.
    fn auto_switch(&mut self, scene: &Scene, events: &[Event]) {
        let Some(me) = scene.local() else {
            self.owned_weapons = [false; 3];
            return;
        };
        for e in events {
            if let Event::Pickup {
                player,
                kind: elora_sim::PickupKind::Weapon(w),
                ..
            } = *e
                && player == me.slot
                && self
                    .controls
                    .auto_switch
                    .wants(self.owned_weapons[w.index()])
            {
                self.controls.want_weapon(w);
            }
        }
        self.owned_weapons = elora_sim::Weapon::ALL.map(|w| me.ch.arsenal.has(w));
    }

    /// Sounds des Frames: Ereignisse, Figuren, Landungen, neue Emotes und Chat-Zeilen.
    fn play_sounds(&mut self, scene: &Scene, events: &[Event]) {
        // Musik des Gebiets im Abenteuer (E-285), sonst still
        let track = self.adventure.as_ref().and_then(|a| {
            let s = &a.session;
            // wacher Hüter: Kampfmusik (R2-M2.1)
            let world = &self.sandbox.world;
            let boss = world.creatures.iter().any(|c| {
                world.creature_kinds.get(c.kind).is_some_and(|k| k.boss)
                    && c.mode != elora_sim::creature::diver::SLEEP
            });
            let area = s.content.area_of(&s.map_name)?;
            if boss {
                return Some(
                    area.boss_music
                        .clone()
                        .unwrap_or_else(|| BOSS_MUSIC.to_owned()),
                );
            }
            let party = s.save.flag(elora_adventure::session::PARTY) != 0;
            party
                .then(|| area.party_music.clone())
                .flatten()
                .or_else(|| area.music.clone())
        });
        self.sounds.music(track.as_deref());
        let mut extra: Vec<elora_audio::Cue> = self
            .emotes
            .take_new()
            .into_iter()
            .filter_map(|slot| scene.chars.iter().find(|c| c.slot == slot))
            .map(|c| elora_audio::Cue::at(elora_audio::Sound::Emote, c.pos()))
            .collect();
        let newest_chat = match &self.online {
            Some(o) => o.client.chat.back().map(|c| c.at),
            None => self.sandbox.notices.back().map(|c| c.at),
        };
        if newest_chat.is_some() && newest_chat != self.chat_heard {
            // erste Zeile nach dem Start nicht vertonen (z. B. Begrüßung beim Verbinden)
            if self.chat_heard.is_some() {
                extra.push(elora_audio::Cue::global(elora_audio::Sound::Chat));
            }
            self.chat_heard = newest_chat;
        }
        let dazed = scene.local().is_some_and(|c| c.ch.core.dazed > 0);
        if dazed && !self.was_dazed {
            extra.push(elora_audio::Cue::global(elora_audio::Sound::Daze));
        }
        self.was_dazed = dazed;
        extra.extend(self.menu.ui.sounds.drain(..).map(elora_audio::Cue::global));
        extra.append(&mut self.ui_cues);
        self.sounds
            .update(scene, events, self.figures.landings(), &extra, scene.camera);
    }

    /// Welt, Figuren und Effekte des Frames in `self.batch` sammeln.
    fn build_batch(&mut self, scene: &Scene, tuning: &Tuning, camera: &Camera, tick: u64) {
        self.batch.clear();
        let map = match &self.online {
            Some(o) => o.client.map.as_ref(),
            None => Some(&self.sandbox.map),
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let look_time = map_view::LookTime {
            local_ms: (f64::from(self.figures.time()) * 1000.0) as i64,
            server_ms: (tick * 1000 / u64::from(elora_sim::TICKS_PER_SECOND)) as i64,
            // nur im eigenen Spiel (Abenteuer, Training); online ohne Hüter
            hook_wilt: self
                .online
                .is_none()
                .then_some(self.sandbox.world.collision.hook_wilt)
                .flatten(),
        };
        draw::scene(
            &mut self.batch,
            scene,
            map_view::MapLayer {
                map,
                view: &mut self.map_view,
                time: look_time,
            },
            tuning,
            camera,
            self.controls.mouse_pos,
            &draw::Looks {
                effects: &self.effects,
                figures: &self.figures,
                art: &self.figure_art,
                items: &self.item_art,
                creatures: &self.creature_art,
                emotes: &self.emotes,
                skins: self
                    .online
                    .as_ref()
                    .map_or(&self.no_skins, |o| &o.client.skins),
                own_skin: self.net.skin,
            },
        );
    }

    /// HUD des Frames in `self.hud_batch` sammeln; liefert die Bildschirmgröße.
    fn build_hud(&mut self, scene: &Scene, tuning: &Tuning, info: &FrameInfo) -> Vec2 {
        let size = self.gfx.as_ref().map_or((1, 1), |g| g.renderer.size());
        #[allow(clippy::cast_precision_loss)]
        let screen = Vec2::new(size.0 as f32, size.1 as f32);
        self.hud_batch.clear();
        self.hud.draw(
            &mut self.hud_batch,
            &self.item_art,
            screen,
            &hud::HudInfo {
                character: scene.local().map(|c| &c.ch),
                max_health: tuning.max_health,
                view: info.view.as_ref(),
                tick: info.tick,
                local: info.local,
                lang: &self.lang,
                ui_scale: self.settings.graphics.ui_scale(),
            },
        );
        game_ui::draw(
            &mut self.hud_batch,
            self.hud.font(),
            &self.item_art,
            screen,
            hud::scale(screen, self.settings.graphics.ui_scale()),
            &game_ui::GameUi {
                view: info.view.as_ref(),
                names: &info.names,
                teams: &info.teams,
                local: info.local,
                chat: &info.chat,
                input: &self.chat_input,
                scoreboard: self.scoreboard,
                vote: info.vote.as_ref(),
                killfeed: &self.killfeed,
                loading: info.loading.as_ref().map(|(m, r, s)| (m.as_str(), *r, *s)),
                time: self.figures.time(),
                lang: &self.lang,
            },
        );
        if self.emotes.wheel_open {
            let selected = emotes::selection(self.controls.mouse_pos);
            self.emotes.draw_wheel(
                &mut self.hud_batch,
                screen,
                hud::scale(screen, self.settings.graphics.ui_scale()),
                selected,
            );
        }
        screen
    }

    /// Emote zeigen: online über den Server, in der Sandbox direkt.
    fn send_emote(&mut self, emote: u8) {
        match &mut self.online {
            Some(o) => o.client.emote(emote),
            None => self.emotes.show(self.sandbox.player, emote),
        }
    }

    fn redraw(&mut self) {
        let now = Instant::now();
        let elapsed = self.frame_time(now);
        let dt = elapsed.as_secs_f32();
        if self.screen == Screen::Menu {
            self.redraw_menu(dt);
            return;
        }
        if self.screen == Screen::Editor {
            self.figures.advance_time(dt);
            self.redraw_editor();
            return;
        }

        // Szene aus Sandbox oder Online-Spiel
        let Some((scene, collision, tuning, events)) = self.advance(now, elapsed) else {
            return;
        };
        self.update_looks(dt, &scene, &collision, &events);
        let info = self.game_info(now);
        game_ui::record_kills(
            &mut self.killfeed,
            &events,
            &info.names,
            &info.teams,
            now,
            &self.lang,
        );

        let Some(aspect) = self.gfx.as_ref().map(|g| g.renderer.aspect()) else {
            return;
        };
        let center = self.adventure_camera(scene.camera, self.view.view_size(aspect), dt);
        // bunter Rausch: die Welt wabert leicht (E-311)
        let wobble = scene
            .local()
            .filter(|c| c.ch.core.dazed > 0)
            .map_or(Vec2::ZERO, |c| {
                #[allow(clippy::cast_precision_loss)]
                let t = info.tick as f32 / elora_sim::TICKS_PER_SECOND as f32;
                let fade = (c.ch.core.dazed as f32 / 25.0).min(1.0);
                Vec2::new((t * 2.3).sin() * 7.0, (t * 1.7).cos() * 4.0) * fade
            });
        let mut camera = Camera::new(
            center + self.effects.camera_offset() + wobble,
            &self.view,
            aspect,
        );
        // auf ganze Bildschirmpixel einrasten: feine Linien bleiben beim Scrollen ruhig (E-288)
        if let Some(gfx) = &self.gfx {
            #[allow(clippy::cast_precision_loss)]
            let px = gfx.renderer.size().0 as f32 / camera.size.x;
            if px > 0.0 {
                camera.center = Vec2::new(
                    (camera.center.x * px).round() / px,
                    (camera.center.y * px).round() / px,
                );
            }
        }
        self.build_batch(&scene, &tuning, &camera, info.tick);
        let screen = self.build_hud(&scene, &tuning, &info);
        let death_choice = if self.adventure.is_some() && !self.menu.paused {
            let s = hud::scale(screen, self.settings.graphics.ui_scale());
            self.draw_adventure_hud(&camera, screen, s, dt)
        } else {
            None
        };
        let pause_action = if self.menu.paused {
            self.draw_pause(dt, &info)
        } else {
            None
        };
        let Some(gfx) = &mut self.gfx else { return };

        let Some(mut frame) = gfx.renderer.begin_frame() else {
            return;
        };
        gfx.renderer
            .draw_shapes(&mut frame, &camera, &self.batch, draw::BACKGROUND);
        let screen_camera = Camera {
            center: screen * 0.5,
            size: screen,
        };
        gfx.renderer
            .draw_overlay(&mut frame, &screen_camera, &self.hud_batch);
        self.draw_debug_panel(&mut frame, &info);
        if let Some(gfx) = &mut self.gfx {
            gfx.renderer.end_frame(frame);
        }
        if let Some(a) = pause_action {
            self.apply_menu(a);
        }
        match death_choice {
            Some(true) => self.adventure_respawn(),
            Some(false) => self.leave_adventure(),
            None => {}
        }
    }

    /// Debug-Panel (egui, E-031) über dem Frame; führt seine Aktion aus.
    pub(crate) fn draw_debug_panel(&mut self, frame: &mut elora_render::Frame, info: &FrameInfo) {
        let mut action = None;
        let show_panel = self.show_panel;
        let online_view = self.online.as_ref().map(|o| OnlineView {
            client: &o.client,
            stats: o.conn.stats(),
            server: o.conn.server,
        });
        let is_online = online_view.is_some();
        let team_mode = info.view.as_ref().is_some_and(|v| v.mode.teams());
        let audio_device = self.sounds.has_device();
        let mut cx = debug_ui::Context {
            sandbox: if is_online {
                None
            } else {
                Some(&mut self.sandbox)
            },
            online: online_view,
            net: &mut self.net,
            view: &mut self.view,
            effects: &mut self.effects.settings,
            audio: debug_ui::AudioUi {
                settings: &mut self.sounds.settings,
                device: audio_device,
            },
            controls: &mut self.controls,
            fps: self.fps,
            frames: debug_ui::FrameStats::of(
                &self.frame_times,
                self.gfx
                    .as_ref()
                    .and_then(|g| g.window.current_monitor())
                    .and_then(|m| m.refresh_rate_millihertz()),
                self.settings.graphics.vsync,
            ),
            status: &self.status,
            cursor_grabbed: self.cursor_grabbed,
            names: &info.names,
            local: info.local,
            team_mode,
            vote_running: info.vote.is_some(),
        };
        let Some(gfx) = &mut self.gfx else { return };
        gfx.gui.draw(&gfx.window, &gfx.renderer, frame, |ui| {
            if show_panel || cx.net.key_warning.is_some() {
                action = action.take().or(debug_ui::panel(ui, &mut cx));
            }
        });
        if let Some(action) = action {
            self.apply(action);
        }
    }

    /// Tastatur bei offenem Chat: Enter sendet, Esc bricht ab, Rücktaste löscht.
    fn chat_key(&mut self, code: KeyCode, event: &KeyEvent) {
        if !event.state.is_pressed() {
            return;
        }
        match code {
            KeyCode::Enter | KeyCode::NumpadEnter => {
                let text = std::mem::take(&mut self.chat_input.text);
                if !text.trim().is_empty()
                    && let Some(o) = &mut self.online
                {
                    o.client.send_chat(self.chat_input.team, &text);
                }
                self.chat_input.open = false;
            }
            KeyCode::Escape => {
                self.chat_input.text.clear();
                self.chat_input.open = false;
            }
            KeyCode::Backspace => {
                self.chat_input.text.pop();
            }
            _ => {
                if let Some(text) = &event.text {
                    for c in text.chars().filter(|c| !c.is_control()) {
                        if self.chat_input.text.chars().count() < elora_protocol::msg::MAX_CHAT {
                            self.chat_input.text.push(c);
                        }
                    }
                }
            }
        }
    }

    fn key(&mut self, event: &KeyEvent) {
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        let pressed = event.state.is_pressed();
        if self.bind_capture.is_some() {
            self.capture_trigger(Trigger::Key(code), pressed);
            return;
        }
        if self.screen == Screen::Editor {
            return; // Tastatur gehört egui (Kürzel im Editor)
        }
        if self.adventure_halted() && !self.menu.paused {
            if pressed && !event.repeat {
                self.adventure_key(code);
            }
            return;
        }
        if self.menu_active() {
            if code == KeyCode::F1 && pressed && !event.repeat {
                self.show_panel = !self.show_panel;
            } else {
                self.menu_key(code, event);
            }
            return;
        }
        if self.chat_input.open {
            self.chat_key(code, event); // Tastatur gehört dem Chat-Feld
            return;
        }
        let unbound = self
            .settings
            .bindings
            .actions(Trigger::Key(code))
            .is_empty();
        let online = self.online.is_some();
        if pressed && !event.repeat {
            match code {
                KeyCode::Escape if self.testing_map() => return self.leave_editor_test(),
                KeyCode::Escape if self.testing_adventure() => return self.leave_adventure(),
                KeyCode::Escape => return self.toggle_pause(),
                KeyCode::F1 => {
                    // Panel offen: Maus frei zum Bedienen; zu: zurück ins Spiel
                    self.show_panel = !self.show_panel;
                    self.set_cursor_grab(!self.show_panel && !self.menu.paused);
                    return;
                }
                KeyCode::KeyR if unbound && !online && self.adventure.is_none() => {
                    return self.apply(Action::Respawn);
                }
                KeyCode::F5 if unbound && !online && self.adventure.is_none() => {
                    self.status = if self.sandbox.rules.is_some() {
                        "Aufzeichnung nur ohne Spielmodus (Golden-Tests = reine Simulation)".into()
                    } else {
                        self.sandbox.stop_recording("F5").unwrap_or_else(|| {
                            self.sandbox.start_recording();
                            "Aufzeichnung läuft … (F5 beendet)".into()
                        })
                    };
                    return;
                }
                _ => {}
            }
        }
        if !event.repeat {
            self.trigger(Trigger::Key(code), pressed);
        }
    }

    /// Alle Aktionen, die `t` auslöst (Tastenbelegung, M7.5).
    fn trigger(&mut self, t: Trigger, down: bool) {
        for action in self.settings.bindings.actions(t) {
            self.game_action(action, down);
        }
    }

    /// Eine belegte Aktion im Spiel.
    fn game_action(&mut self, action: GameAction, down: bool) {
        if self.controls.action(action, down) {
            return;
        }
        let online = self.online.is_some();
        match action {
            // im Abenteuer öffnet Tab das Abenteuer-Menü (E-263)
            GameAction::Scoreboard if self.adventure.is_some() => {
                if down {
                    self.toggle_adventure_menu();
                }
            }
            // Scoreboard solange gehalten (E-078)
            GameAction::Scoreboard => self.scoreboard = down,
            GameAction::QuickHeal if down => self.quick_heal(),
            // Emote-Rad solange gehalten (E-091); beim Loslassen wählen
            GameAction::Emote => {
                if down {
                    self.emotes.wheel_open = true;
                } else if std::mem::take(&mut self.emotes.wheel_open)
                    && let Some(e) = emotes::selection(self.controls.mouse_pos)
                {
                    self.send_emote(e);
                }
            }
            GameAction::Chat | GameAction::TeamChat if down && online => {
                self.controls.release_all();
                self.chat_input = game_ui::ChatInput {
                    open: true,
                    team: action == GameAction::TeamChat,
                    text: String::new(),
                };
            }
            GameAction::Kill if down && self.adventure.is_none() => self.apply(Action::Kill),
            GameAction::Interact if down => self.adventure_interact(),
            GameAction::VoteYes if down => self.apply(Action::Vote(true)),
            GameAction::VoteNo if down => self.apply(Action::Vote(false)),
            _ => {}
        }
    }

    /// Neu belegen: nächste Taste/Maustaste/Radrichtung übernehmen, Esc bricht ab.
    fn capture_trigger(&mut self, t: Trigger, pressed: bool) {
        if !pressed {
            return;
        }
        let Some(action) = self.bind_capture else {
            return;
        };
        if t == Trigger::Key(KeyCode::Escape) {
            self.bind_capture = None;
        } else if self.settings.bindings.set(action, t) {
            self.bind_capture = None;
            self.save_settings();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gfx.is_some() {
            return;
        }
        let graphics = self.settings.graphics;
        let attrs = Window::default_attributes()
            .with_title("Elora")
            .with_inner_size(LogicalSize::new(1280.0, 720.0))
            // maximiert starten; die Größe oben gilt beim Verkleinern
            .with_maximized(true)
            .with_fullscreen(
                graphics
                    .fullscreen
                    .then_some(winit::window::Fullscreen::Borderless(None)),
            );
        let result = (|| {
            let window = Arc::new(event_loop.create_window(attrs)?);
            let size = window.inner_size();
            let mut renderer =
                pollster::block_on(Renderer::new(window.clone(), size.width, size.height))?;
            renderer.set_vsync(graphics.vsync);
            renderer.set_msaa(graphics.msaa);
            let gui = Gui::new(&window, &renderer);
            anyhow::Ok(Gfx {
                window,
                renderer,
                gui,
            })
        })();
        match result {
            Ok(gfx) => {
                self.gfx = Some(gfx);
                self.last_frame = Instant::now();
                self.set_cursor_grab(self.screen == Screen::Game);
            }
            Err(e) => {
                self.error = Some(e.context("Grafik konnte nicht initialisiert werden"));
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        // Solange die Maus frei ist und egui sichtbar (Debug-Panel, Editor, Fenster),
        // bekommt egui die Eingaben zuerst – sonst schluckt es z. B. Tab und Klicks der Spiel-UI
        let egui_visible = self.show_panel
            || self.screen == Screen::Editor
            || self.net.show_host
            || self.net.key_warning.is_some();
        let to_gui = !self.cursor_grabbed && egui_visible;
        if to_gui
            && let Some(gfx) = &mut self.gfx
            && gfx.gui.on_window_event(&gfx.window, &event)
            && !matches!(
                event,
                WindowEvent::RedrawRequested | WindowEvent::Resized(_)
            )
        {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gfx) = &mut self.gfx {
                    gfx.renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::Focused(false) => {
                if self.screen == Screen::Game && !self.menu.paused {
                    self.toggle_pause();
                }
                self.set_cursor_grab(false);
            }
            WindowEvent::CursorMoved { position, .. } => {
                #[allow(clippy::cast_possible_truncation)]
                self.menu_cursor(elora_sim::Vec2::new(position.x as f32, position.y as f32));
            }
            WindowEvent::KeyboardInput { event, .. } => self.key(&event),
            WindowEvent::MouseInput { state, button, .. } => {
                if self.bind_capture.is_some() {
                    self.capture_trigger(Trigger::Mouse(button), state.is_pressed());
                } else if self.screen == Screen::Editor {
                    // Maus gehört egui
                } else if self.menu_active() {
                    self.menu_mouse_button(button, state);
                } else if self.cursor_grabbed {
                    self.trigger(Trigger::Mouse(button), state.is_pressed());
                } else if state.is_pressed()
                    && !self.gfx.as_ref().is_some_and(|g| g.gui.wants_pointer())
                {
                    self.set_cursor_grab(true);
                }
            }
            WindowEvent::MouseWheel { delta, .. } if self.bind_capture.is_some() => {
                let up = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y > 0.0,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y > 0.0,
                };
                let t = if up {
                    Trigger::WheelUp
                } else {
                    Trigger::WheelDown
                };
                self.capture_trigger(t, true);
            }
            WindowEvent::MouseWheel { delta, .. } if self.menu_active() => self.menu_wheel(delta),
            WindowEvent::MouseWheel { delta, .. } if self.cursor_grabbed => {
                let notches = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y.round() as i32,
                    winit::event::MouseScrollDelta::PixelDelta(p) => (p.y / 40.0).round() as i32,
                };
                let t = if notches > 0 {
                    Trigger::WheelUp
                } else {
                    Trigger::WheelDown
                };
                // je Raste Drücken + Loslassen
                for _ in 0..notches.unsigned_abs().min(8) {
                    self.trigger(t, true);
                    self.trigger(t, false);
                }
            }
            WindowEvent::RedrawRequested => self.redraw(),
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event
            && self.cursor_grabbed
        {
            self.controls.mouse_motion(dx, dy);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.menu.quit {
            event_loop.exit();
            return;
        }
        if let Some(gfx) = &self.gfx {
            gfx.window.request_redraw();
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        self.save_settings();
        // Verbindung sauber beenden; Server stoppt je nach Einstellung (Hosting::drop)
        self.online = None;
        std::thread::sleep(Duration::from_millis(20));
    }
}

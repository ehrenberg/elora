//! Elora-Client: lokale Sandbox (M1/M2) oder online mit einem Server (M3).
//!
//! Aufruf: `elora [karte.emap.toml] [--mode dm|tdm|ctf|lms|lts] [--instagib] [--connect adresse:port]`
//! (Standardkarte: `maps/sandbox.emap.toml`)

mod app_menu;
mod bindings;
mod connection;
mod controls;
mod debug_ui;
mod draw;
mod effects;
mod emotes;
mod figure;
mod game_ui;
mod gui;
mod hosting;
mod hud;
mod items;
mod lang;
mod menu;
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

const DEFAULT_MAP: &str = "maps/sandbox.emap.toml";

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
        .find(|a| a.ends_with(".emap.toml"))
        .map_or_else(|| PathBuf::from(DEFAULT_MAP), PathBuf::from);
    let file = TuningFile::load(Path::new(TUNING_FILE))?;
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
    let direct = connect.is_some()
        || args
            .iter()
            .any(|a| a.ends_with(".emap.toml") || a == "--mode");
    let mut app = App::new(sandbox, &file, settings);
    if direct {
        app.screen = Screen::Game;
        app.show_panel = true;
    }
    if let Some(address) = connect {
        app.net.address = address;
        app.connect();
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
}

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
    effects: effects::Effects,
    figures: figure::Figures,
    figure_art: figure::FigureArt,
    item_art: items::ItemArt,
    /// Leere Skin-Tabelle für die Sandbox.
    no_skins: std::collections::BTreeMap<usize, elora_protocol::Skin>,
    last_frame: Instant,
    fps: f32,
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
    menu: menu::Menu,
    /// Kartennamen für „Server erstellen“.
    maps: Vec<String>,
    /// Aktion, die gerade neu belegt wird (nächste Taste zählt).
    bind_capture: Option<GameAction>,
}

impl App {
    fn new(sandbox: Sandbox, file: &TuningFile, settings: Settings) -> Self {
        let mut controls = Controls::default();
        controls.sensitivity = settings.input.mouse_sensitivity;
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
            known: KnownServers::load(Path::new(KNOWN_SERVERS_FILE)),
            net,
            controls,
            view: file.view.into(),
            batch: ShapeBatch::default(),
            hud_batch: ShapeBatch::default(),
            hud: hud::Hud::new(),
            emotes: emotes::Emotes::new(),
            sounds: sound::Sounds::new(settings.audio),
            chat_heard: None,
            effects: effects::Effects::with_settings(settings.effects),
            figures: figure::Figures::default(),
            figure_art: figure::FigureArt::load(),
            item_art: items::ItemArt::load(),
            no_skins: std::collections::BTreeMap::new(),
            last_frame: Instant::now(),
            fps: 0.0,
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
            menu: menu::Menu::default(),
            maps: app_menu::map_names(),
            bind_capture: None,
        }
    }

    /// Aktuelle Werte aus Spiel und Panel in die Einstellungen übernehmen und speichern.
    fn save_settings(&mut self) {
        let s = &mut self.settings;
        s.player.name.clone_from(&self.net.name);
        s.player.set_skin(self.net.skin);
        s.input.mouse_sensitivity = self.controls.sensitivity;
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
                let client = OnlineClient::new(&self.net.name, self.net.skin, Instant::now());
                self.online = Some(Online { client, conn });
                self.sandbox.stop_recording("Online");
            }
            Err(e) => self.status = format!("Verbindung fehlgeschlagen: {e:#}"),
        }
    }

    fn apply(&mut self, action: Action) {
        let path = Path::new(TUNING_FILE);
        match action {
            Action::Save => {
                self.status = match self.tuning_file().save(path) {
                    Ok(()) => format!("Gespeichert in {TUNING_FILE}"),
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
                    if let Err(e) = self
                        .known
                        .trust(Path::new(KNOWN_SERVERS_FILE), w.server, &got)
                    {
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
                        && let Err(e) = self.known.trust(
                            Path::new(KNOWN_SERVERS_FILE),
                            o.conn.server,
                            &server_key,
                        )
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
                    ended = Some("Server-Schlüssel geändert – Verbindung abgebrochen".to_owned());
                }
                ClientEvent::Disconnected(reason) => {
                    ended = Some(match reason {
                        DisconnectReason::Timeout => "Zeitüberschreitung".to_owned(),
                        DisconnectReason::Remote(r) | DisconnectReason::Rejected(r) => r,
                        other => format!("{other:?}"),
                    });
                }
            }
        }
        if let Status::Disconnected(reason) = &o.client.status {
            ended = ended.or_else(|| Some(reason.clone()));
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
        if let Some(o) = &mut self.online {
            // schickt nur bei Änderung eine Nachricht
            o.client.set_skin(self.net.skin);
        }
        self.play_sounds(scene, events);
    }

    /// Sounds des Frames: Ereignisse, Figuren, Landungen, neue Emotes und Chat-Zeilen.
    fn play_sounds(&mut self, scene: &Scene, events: &[Event]) {
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
        self.sounds
            .update(scene, events, self.figures.landings(), &extra, scene.camera);
    }

    /// Welt, Figuren und Effekte des Frames in `self.batch` sammeln.
    fn build_batch(
        &mut self,
        scene: &Scene,
        collision: &Collision,
        tuning: &Tuning,
        camera: &Camera,
    ) {
        self.batch.clear();
        draw::scene(
            &mut self.batch,
            scene,
            collision,
            tuning,
            camera,
            self.controls.mouse_pos,
            &draw::Looks {
                effects: &self.effects,
                figures: &self.figures,
                art: &self.figure_art,
                items: &self.item_art,
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
        let camera = Camera::new(
            scene.camera + self.effects.camera_offset(),
            &self.view,
            aspect,
        );
        self.build_batch(&scene, &collision, &tuning, &camera);
        let screen = self.build_hud(&scene, &tuning, &info);
        let pause_action = if self.menu.paused {
            self.draw_pause(dt)
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
                KeyCode::Escape => return self.toggle_pause(),
                KeyCode::F1 => {
                    self.show_panel = !self.show_panel;
                    return;
                }
                KeyCode::KeyR if unbound && !online => return self.apply(Action::Respawn),
                KeyCode::F5 if unbound && !online => {
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
            // Scoreboard solange gehalten (E-078)
            GameAction::Scoreboard => self.scoreboard = down,
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
            GameAction::Kill if down => self.apply(Action::Kill),
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
        // Solange die Maus frei ist, bekommt egui (Debug-Panel) die Eingaben zuerst
        let to_gui = !self.cursor_grabbed;
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

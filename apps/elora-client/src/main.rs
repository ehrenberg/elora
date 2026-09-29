//! Elora-Client: lokale Sandbox (M1/M2) oder online mit einem Server (M3).
//!
//! Aufruf: `elora [karte.emap.toml] [--mode dm|tdm|ctf|lms|lts] [--instagib] [--connect adresse:port]`
//! (Standardkarte: `maps/sandbox.emap.toml`)

mod connection;
mod controls;
mod debug_ui;
mod draw;
mod effects;
mod figure;
mod game_ui;
mod gui;
mod hosting;
mod hud;
mod items;
mod sandbox;
mod settings;
mod skins;

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
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use connection::{Connection, KNOWN_SERVERS_FILE, KnownServers};
use controls::Controls;
use debug_ui::{Action, KeyWarning, NetUi, OnlineView};
use gui::Gui;
use sandbox::Sandbox;
use settings::{TUNING_FILE, TuningFile};

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
    let mut app = App::new(sandbox, &file);
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
}

impl App {
    fn new(sandbox: Sandbox, file: &TuningFile) -> Self {
        let mut controls = Controls::default();
        controls.sensitivity = file.input.mouse_sensitivity;
        let mut net = NetUi::default();
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
            effects: effects::Effects::with_settings(file.effects),
            figures: figure::Figures::default(),
            figure_art: figure::FigureArt::load(),
            item_art: items::ItemArt::load(),
            no_skins: std::collections::BTreeMap::new(),
            last_frame: Instant::now(),
            fps: 0.0,
            cursor_grabbed: false,
            show_panel: true,
            status: String::new(),
            error: None,
            chat_input: game_ui::ChatInput::default(),
            scoreboard: false,
            killfeed: VecDeque::new(),
        }
    }

    fn tuning_file(&self) -> TuningFile {
        TuningFile {
            physics: self.sandbox.world.tuning.clone(),
            view: self.view.into(),
            input: settings::InputFile {
                mouse_sensitivity: self.controls.sensitivity,
            },
            effects: self.effects.settings,
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
                        self.controls.sensitivity = f.input.mouse_sensitivity;
                        self.effects.settings = f.effects;
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

    /// Namen, Teams, Spielzustand, eigener Slot, Tick und Abstimmung.
    #[allow(clippy::type_complexity)]
    fn game_info(
        &self,
        now: Instant,
    ) -> (
        std::collections::BTreeMap<usize, String>,
        std::collections::BTreeMap<usize, elora_sim::Team>,
        Option<elora_protocol::GameView>,
        Option<usize>,
        u64,
        Option<elora_protocol::VoteInfo>,
    ) {
        match &self.online {
            Some(o) => (
                o.client.names.clone(),
                o.client.teams(),
                o.client.game(),
                o.client.slot,
                o.client.server_tick(now).unwrap_or(0),
                o.client.vote.clone(),
            ),
            None => (
                self.sandbox.names(),
                self.sandbox
                    .world
                    .players
                    .iter()
                    .enumerate()
                    .filter_map(|(i, p)| Some((i, p.as_ref()?.team)))
                    .collect(),
                self.sandbox.game(),
                Some(self.sandbox.player),
                self.sandbox.world.tick,
                None,
            ),
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
                skins: self
                    .online
                    .as_ref()
                    .map_or(&self.no_skins, |o| &o.client.skins),
                own_skin: self.net.skin,
            },
        );
    }

    /// HUD des Frames in `self.hud_batch` sammeln; liefert die Bildschirmgröße.
    fn build_hud(
        &mut self,
        scene: &Scene,
        tuning: &Tuning,
        view: Option<&elora_protocol::GameView>,
        tick: u64,
        local: Option<usize>,
    ) -> Vec2 {
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
                view,
                tick,
                local,
            },
        );
        screen
    }

    fn redraw(&mut self) {
        let now = Instant::now();
        let elapsed = self.frame_time(now);

        // Szene aus Sandbox oder Online-Spiel
        let Some((scene, collision, tuning, events)) = self.advance(now, elapsed) else {
            return;
        };
        self.update_looks(elapsed.as_secs_f32(), &scene, &collision, &events);
        let (names, teams, view, local_slot, tick, vote) = self.game_info(now);

        let Some(aspect) = self.gfx.as_ref().map(|g| g.renderer.aspect()) else {
            return;
        };
        let camera = Camera::new(
            scene.camera + self.effects.camera_offset(),
            &self.view,
            aspect,
        );
        self.build_batch(&scene, &collision, &tuning, &camera);
        let screen = self.build_hud(&scene, &tuning, view.as_ref(), tick, local_slot);
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

        // Spielinformationen für Anzeigen und Panel
        game_ui::record_kills(&mut self.killfeed, &events, &names, now);
        let chat: Vec<elora_client::online::ChatLine> = match &self.online {
            Some(o) => o.client.chat.iter().cloned().collect(),
            None => self.sandbox.notices.iter().cloned().collect(),
        };

        let mut action = None;
        let mut ui_action = None;
        let show_panel = self.show_panel;
        let online_view = self.online.as_ref().map(|o| OnlineView {
            client: &o.client,
            stats: o.conn.stats(),
            server: o.conn.server,
        });
        let is_online = online_view.is_some();
        let team_mode = view.as_ref().is_some_and(|v| v.mode.teams());
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
            controls: &mut self.controls,
            fps: self.fps,
            status: &self.status,
            cursor_grabbed: self.cursor_grabbed,
            names: &names,
            local: local_slot,
            team_mode,
            vote_running: vote.is_some(),
        };
        let mut game = game_ui::GameUi {
            view: view.as_ref(),
            names: &names,
            teams: &teams,
            local: local_slot,
            chat,
            input: &mut self.chat_input,
            scoreboard: self.scoreboard,
            vote: vote.as_ref(),
            killfeed: &self.killfeed,
        };
        gfx.gui.draw(&gfx.window, &gfx.renderer, &mut frame, |ui| {
            ui_action = ui_action.take().or(game_ui::draw(ui, &mut game));
            if show_panel || cx.net.key_warning.is_some() {
                action = action.take().or(debug_ui::panel(ui, &mut cx));
            }
        });
        gfx.renderer.end_frame(frame);
        if let Some(action) = action {
            self.apply(action);
        }
        match ui_action {
            Some(game_ui::UiAction::SendChat { team, text }) => {
                if let Some(o) = &mut self.online {
                    o.client.send_chat(team, &text);
                }
                self.chat_input.open = false;
            }
            Some(game_ui::UiAction::CloseChat) => self.chat_input.open = false,
            None => {}
        }
    }

    fn key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        // Scoreboard solange Tab gehalten wird (E-078)
        if code == KeyCode::Tab {
            self.scoreboard = event.state.is_pressed();
            return;
        }
        if self.chat_input.open {
            return; // Tastatur gehört dem Chat-Feld
        }
        if self.controls.key(code, event.state)
            || event.state != ElementState::Pressed
            || event.repeat
        {
            return;
        }
        let online = self.online.is_some();
        match code {
            KeyCode::Escape if self.cursor_grabbed => self.set_cursor_grab(false),
            KeyCode::Escape => event_loop.exit(),
            KeyCode::KeyR if !online => self.apply(Action::Respawn),
            KeyCode::F5 if !online && self.sandbox.rules.is_some() => {
                self.status =
                    "Aufzeichnung nur ohne Spielmodus (Golden-Tests = reine Simulation)".into();
            }
            KeyCode::F5 if !online => {
                self.status = self.sandbox.stop_recording("F5").unwrap_or_else(|| {
                    self.sandbox.start_recording();
                    "Aufzeichnung läuft … (F5 beendet)".into()
                });
            }
            KeyCode::F1 => self.show_panel = !self.show_panel,
            KeyCode::KeyT | KeyCode::KeyY if online => {
                self.controls.release_all();
                self.chat_input = game_ui::ChatInput {
                    open: true,
                    team: code == KeyCode::KeyY,
                    text: String::new(),
                };
            }
            KeyCode::KeyK => self.apply(Action::Kill),
            KeyCode::F3 => self.apply(Action::Vote(true)),
            KeyCode::F4 => self.apply(Action::Vote(false)),
            _ => {}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gfx.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("Elora")
            .with_inner_size(LogicalSize::new(1280.0, 720.0));
        let result = (|| {
            let window = Arc::new(event_loop.create_window(attrs)?);
            let size = window.inner_size();
            let renderer =
                pollster::block_on(Renderer::new(window.clone(), size.width, size.height))?;
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
                self.set_cursor_grab(true);
            }
            Err(e) => {
                self.error = Some(e.context("Grafik konnte nicht initialisiert werden"));
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        // Solange die Maus frei ist (oder der Chat offen), bekommt egui die Eingaben zuerst
        let to_gui = !self.cursor_grabbed || self.chat_input.open;
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
            WindowEvent::Focused(false) => self.set_cursor_grab(false),
            WindowEvent::KeyboardInput { event, .. } => self.key(event_loop, &event),
            WindowEvent::MouseInput { state, button, .. } => {
                if self.cursor_grabbed {
                    self.controls.mouse_button(button, state);
                } else if state.is_pressed()
                    && !self.gfx.as_ref().is_some_and(|g| g.gui.wants_pointer())
                {
                    self.set_cursor_grab(true);
                }
            }
            WindowEvent::MouseWheel { delta, .. } if self.cursor_grabbed => {
                let notches = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y.round() as i32,
                    winit::event::MouseScrollDelta::PixelDelta(p) => (p.y / 40.0).round() as i32,
                };
                self.controls.mouse_wheel(notches);
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

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(gfx) = &self.gfx {
            gfx.window.request_redraw();
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        // Verbindung sauber beenden; Server stoppt je nach Einstellung (Hosting::drop)
        self.online = None;
        std::thread::sleep(Duration::from_millis(20));
    }
}

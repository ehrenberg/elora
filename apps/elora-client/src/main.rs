//! Elora-Client. In M1 ist er die Physik-Sandbox (E-013).
//!
//! Aufruf: `elora [karte.emap.toml]` (Standard: `maps/sandbox.emap.toml`)

mod controls;
mod debug_ui;
mod draw;
mod gui;
mod sandbox;
mod settings;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context as _;
use elora_render::{Camera, Renderer, ShapeBatch, ViewSettings};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use controls::Controls;
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

    let map_path = std::env::args()
        .nth(1)
        .map_or_else(|| PathBuf::from(DEFAULT_MAP), PathBuf::from);
    let file = TuningFile::load(Path::new(TUNING_FILE))?;
    let sandbox = Sandbox::load(&map_path, file.physics.clone())?;
    tracing::info!(map = %map_path.display(), "Karte geladen");

    let event_loop = EventLoop::new().context("Event-Loop konnte nicht erstellt werden")?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(sandbox, &file);
    event_loop.run_app(&mut app)?;
    app.error.map_or(Ok(()), Err)
}

/// Fenster und Grafik existieren erst nach `resumed`.
struct Gfx {
    window: Arc<Window>,
    renderer: Renderer,
    gui: Gui,
}

struct App {
    gfx: Option<Gfx>,
    sandbox: Sandbox,
    controls: Controls,
    view: ViewSettings,
    batch: ShapeBatch,
    last_frame: Instant,
    fps: f32,
    cursor_grabbed: bool,
    show_panel: bool,
    status: String,
    error: Option<anyhow::Error>,
}

impl App {
    fn new(sandbox: Sandbox, file: &TuningFile) -> Self {
        let mut controls = Controls::default();
        controls.sensitivity = file.input.mouse_sensitivity;
        Self {
            gfx: None,
            sandbox,
            controls,
            view: file.view.into(),
            batch: ShapeBatch::default(),
            last_frame: Instant::now(),
            fps: 0.0,
            cursor_grabbed: false,
            show_panel: true,
            status: String::new(),
            error: None,
        }
    }

    fn tuning_file(&self) -> TuningFile {
        TuningFile {
            physics: self.sandbox.world.tuning.clone(),
            view: self.view.into(),
            input: settings::InputFile {
                mouse_sensitivity: self.controls.sensitivity,
            },
        }
    }

    fn apply(&mut self, action: debug_ui::Action) {
        let path = Path::new(TUNING_FILE);
        self.status = match action {
            debug_ui::Action::Save => match self.tuning_file().save(path) {
                Ok(()) => format!("Gespeichert in {TUNING_FILE}"),
                Err(e) => format!("Fehler: {e:#}"),
            },
            debug_ui::Action::Load => match TuningFile::load(path) {
                Ok(f) => {
                    self.sandbox.world.tuning = f.physics;
                    self.view = f.view.into();
                    self.controls.sensitivity = f.input.mouse_sensitivity;
                    format!("Geladen aus {TUNING_FILE}")
                }
                Err(e) => format!("Fehler: {e:#}"),
            },
            debug_ui::Action::Respawn => {
                let msg = self.sandbox.stop_recording("Respawn").unwrap_or_default();
                self.sandbox.respawn();
                msg
            }
        };
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

    fn redraw(&mut self) {
        let now = Instant::now();
        let elapsed = now - self.last_frame;
        self.last_frame = now;
        if elapsed.as_secs_f32() > 0.0 {
            self.fps = self.fps * 0.95 + (1.0 / elapsed.as_secs_f32()) * 0.05;
        }
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
        self.sandbox.advance(elapsed, &self.controls);

        let Some(gfx) = &mut self.gfx else { return };
        let camera = Camera::new(self.sandbox.render_pos(), &self.view, gfx.renderer.aspect());
        self.batch.clear();
        draw::sandbox(
            &mut self.batch,
            &self.sandbox,
            &camera,
            self.controls.mouse_pos,
        );

        let Some(mut frame) = gfx.renderer.begin_frame() else {
            return;
        };
        gfx.renderer
            .draw_shapes(&mut frame, &camera, &self.batch, draw::BACKGROUND);

        let mut action = None;
        if self.show_panel {
            let mut cx = debug_ui::Context {
                sandbox: &mut self.sandbox,
                view: &mut self.view,
                controls: &mut self.controls,
                fps: self.fps,
                status: &self.status,
                cursor_grabbed: self.cursor_grabbed,
            };
            gfx.gui.draw(&gfx.window, &gfx.renderer, &mut frame, |ui| {
                action = action.or(debug_ui::panel(ui, &mut cx));
            });
        }
        gfx.renderer.end_frame(frame);
        if let Some(action) = action {
            self.apply(action);
        }
    }

    fn key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        if self.controls.key(code, event.state)
            || event.state != ElementState::Pressed
            || event.repeat
        {
            return;
        }
        match code {
            KeyCode::Escape if self.cursor_grabbed => self.set_cursor_grab(false),
            KeyCode::Escape => event_loop.exit(),
            KeyCode::KeyR => {
                if let Some(msg) = self.sandbox.stop_recording("Respawn") {
                    self.status = msg;
                }
                self.sandbox.respawn();
            }
            KeyCode::F5 => {
                self.status = self.sandbox.stop_recording("F5").unwrap_or_else(|| {
                    self.sandbox.start_recording();
                    "Aufzeichnung läuft … (F5 beendet)".into()
                });
            }
            KeyCode::F1 => self.show_panel = !self.show_panel,
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
            .with_title("Elora – Sandbox")
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
        // Solange die Maus frei ist, bekommt das Panel die Eingaben zuerst
        if !self.cursor_grabbed
            && self.show_panel
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
}

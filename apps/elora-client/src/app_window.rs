//! Window events of winit: creating the window, routing events, shutting down.

use std::sync::Arc;
use std::time::{Duration, Instant};

use elora_render::Renderer;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::gui::Gui;
use crate::{App, Gfx, Screen};

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gfx.is_some() {
            return;
        }
        let graphics = self.settings.graphics;
        let attrs = Window::default_attributes()
            .with_title("Elora")
            .with_inner_size(LogicalSize::new(1280.0, 720.0))
            // start maximized; the size above applies when restoring down
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
                self.error = Some(e.context("could not initialise graphics"));
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        // As long as the mouse is free and egui visible (debug panel, editor, windows),
        // egui gets the input first – otherwise it swallows e.g. Tab and clicks of the game UI
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
            WindowEvent::MouseInput { state, button, .. } => self.mouse_button(button, state),
            WindowEvent::MouseWheel { delta, .. } => self.mouse_wheel(delta),
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
        // close the connection cleanly; the server stops depending on the setting (Hosting::drop)
        self.online = None;
        std::thread::sleep(Duration::from_millis(20));
    }
}

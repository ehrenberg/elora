//! Anbindung von Hauptmenü und Pause-Menü an die App (M7.3): Zustand Menü ↔ Spiel,
//! Eingaben für die Spiel-UI, Ausführen der Menü-Aktionen.

use std::time::Instant;

use elora_render::Camera;
use elora_sim::Vec2;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta};
use winit::keyboard::KeyCode;

use crate::lang::{Lang, Language};
use crate::menu::{MenuAction, MenuCtx};
use crate::menu_settings::SettingsEdit;
use crate::settings::GraphicsSettings;
use crate::ui::UiKey;
use crate::{Action, App, Screen, draw, hosting, hud};

/// Zweiter Klick innerhalb dieser Zeit und Entfernung gilt als Doppelklick.
const DOUBLE_CLICK_SECS: f32 = 0.4;
const DOUBLE_CLICK_PX: f32 = 6.0;

/// Kartennamen aus `maps/` (ohne `.emap.toml`).
pub fn map_names() -> Vec<String> {
    hosting::available_maps()
        .iter()
        .filter_map(|p| {
            let name = p.file_name()?.to_str()?;
            Some(name.trim_end_matches(".emap.toml").to_owned())
        })
        .collect()
}

impl App {
    /// Ist gerade eine Menü-Oberfläche (Haupt- oder Pause-Menü) aktiv?
    pub(crate) fn menu_active(&self) -> bool {
        self.screen == Screen::Menu || self.menu.paused
    }

    fn menu_ctx_parts(&self) -> (Vec2, f32) {
        let size = self.gfx.as_ref().map_or((1, 1), |g| g.renderer.size());
        #[allow(clippy::cast_precision_loss)]
        let screen = Vec2::new(size.0 as f32, size.1 as f32);
        (
            screen,
            hud::scale(screen, self.settings.graphics.ui_scale()),
        )
    }

    /// Hauptmenü zeichnen (ersetzt das Spielbild).
    pub(crate) fn redraw_menu(&mut self, dt: f32) {
        self.sounds.menu_music(true);
        let (screen, s) = self.menu_ctx_parts();
        self.hud_batch.clear();
        let graphics_before = self.settings.graphics;
        let language_before = self.settings.language;
        let audio_device = self.sounds.has_device();
        let cx = MenuCtx {
            font: self.hud.font(),
            lang: &self.lang,
            art: &self.figure_art,
            last_server: self.settings.last_server.as_deref(),
            favorites: &self.settings.favorites,
            maps: &self.maps,
            status: &self.status,
            screen,
            s,
            dt,
        };
        let mut edit = SettingsEdit {
            name: &mut self.net.name,
            skin: &mut self.net.skin,
            graphics: &mut self.settings.graphics,
            audio: &mut self.sounds.settings,
            effects: &mut self.effects.settings,
            sensitivity: &mut self.controls.sensitivity,
            language: &mut self.settings.language,
            bindings: &mut self.settings.bindings,
            capture: &mut self.bind_capture,
            audio_device,
        };
        let (action, changed) = self.menu.draw_main(&mut self.hud_batch, &cx, &mut edit);
        if changed {
            self.settings_changed(graphics_before, language_before);
        }
        let Some(gfx) = &mut self.gfx else { return };
        let Some(mut frame) = gfx.renderer.begin_frame() else {
            return;
        };
        let camera = Camera {
            center: screen * 0.5,
            size: screen,
        };
        gfx.renderer.draw_shapes(
            &mut frame,
            &camera,
            &elora_render::ShapeBatch::default(),
            draw::BACKGROUND,
        );
        gfx.renderer
            .draw_overlay(&mut frame, &camera, &self.hud_batch);
        let info = self.game_info(Instant::now());
        self.draw_debug_panel(&mut frame, &info);
        if let Some(gfx) = &mut self.gfx {
            gfx.renderer.end_frame(frame);
        }
        if let Some(a) = action {
            self.apply_menu(a);
        }
        if self.menu.save_pending && !self.menu.input.down {
            self.menu.save_pending = false;
            self.save_settings();
        }
    }

    /// Pause-Menü in den HUD-Batch zeichnen (über dem Spiel).
    pub(crate) fn draw_pause(&mut self, dt: f32) -> Option<MenuAction> {
        let (screen, s) = self.menu_ctx_parts();
        let cx = MenuCtx {
            font: self.hud.font(),
            lang: &self.lang,
            art: &self.figure_art,
            last_server: None,
            favorites: &[],
            maps: &[],
            status: "",
            screen,
            s,
            dt,
        };
        self.menu.draw_pause(&mut self.hud_batch, &cx)
    }

    /// Geänderte Einstellungen sofort anwenden und speichern.
    fn settings_changed(&mut self, graphics_before: GraphicsSettings, language_before: Language) {
        let g = self.settings.graphics;
        if self.settings.language != language_before {
            self.lang = Lang::new(self.settings.language);
        }
        if let Some(gfx) = &mut self.gfx {
            if g.fullscreen != graphics_before.fullscreen {
                gfx.window.set_fullscreen(
                    g.fullscreen
                        .then_some(winit::window::Fullscreen::Borderless(None)),
                );
            }
            if g.vsync != graphics_before.vsync {
                gfx.renderer.set_vsync(g.vsync);
            }
            if g.msaa != graphics_before.msaa {
                gfx.renderer.set_msaa(g.msaa);
            }
        }
        // Regler nicht bei jeder Bewegung speichern – erst beim Loslassen
        if self.menu.input.down {
            self.menu.save_pending = true;
        } else {
            self.save_settings();
        }
    }

    pub(crate) fn apply_menu(&mut self, action: MenuAction) {
        match action {
            MenuAction::Training => {
                self.online = None;
                self.enter_game();
                self.status.clear();
            }
            MenuAction::Connect(address) => {
                self.net.address.clone_from(&address);
                self.settings.last_server = Some(address);
                self.connect();
                self.enter_game();
            }
            MenuAction::Host => {
                let form = self.menu.create.clone();
                let config = &mut self.net.hosting.config;
                config.name = form.name;
                if let Some(map) = hosting::available_maps().get(form.map) {
                    config.map.clone_from(map);
                }
                config.rules.mode = form.mode;
                config.rules.instagib = form.instagib;
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                {
                    config.max_clients = form.max_clients.round() as usize;
                }
                self.apply(Action::HostStart);
                if self.online.is_some() {
                    self.settings.last_server = Some(self.net.address.clone());
                    self.enter_game();
                } else {
                    self.status.clone_from(&self.net.hosting.status);
                }
            }
            MenuAction::ToggleFavorite(address) => {
                self.settings.toggle_favorite(&address);
                self.save_settings();
            }
            MenuAction::Resume => {
                self.menu.paused = false;
                self.set_cursor_grab(true);
            }
            MenuAction::ToMenu => {
                self.online = None;
                self.menu.paused = false;
                self.screen = Screen::Menu;
                self.set_cursor_grab(false);
            }
            MenuAction::Quit => self.menu.quit = true,
        }
    }

    fn enter_game(&mut self) {
        self.screen = Screen::Game;
        self.menu.paused = false;
        self.sounds.menu_music(false);
        self.last_frame = Instant::now();
        self.set_cursor_grab(true);
    }

    /// Esc im Spiel öffnet die Pause, im Pause-Menü geht es weiter.
    pub(crate) fn toggle_pause(&mut self) {
        if self.menu.paused {
            self.apply_menu(MenuAction::Resume);
        } else {
            self.menu.paused = true;
            self.controls.release_all();
            self.set_cursor_grab(false);
        }
    }

    /// Mausbewegung (Pixel) für die Spiel-UI.
    pub(crate) fn menu_cursor(&mut self, pos: Vec2) {
        self.menu.input.mouse = pos;
    }

    pub(crate) fn menu_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        if button != MouseButton::Left {
            return;
        }
        let input = &mut self.menu.input;
        if state.is_pressed() {
            input.down = true;
            input.pressed = true;
            let now = Instant::now();
            if let Some((t, p)) = self.menu.last_click
                && (now - t).as_secs_f32() < DOUBLE_CLICK_SECS
                && (p - input.mouse).length() < DOUBLE_CLICK_PX
            {
                input.double_click = true;
                self.menu.last_click = None;
            } else {
                self.menu.last_click = Some((now, input.mouse));
            }
        } else {
            input.down = false;
            input.released = true;
        }
    }

    pub(crate) fn menu_wheel(&mut self, delta: MouseScrollDelta) {
        self.menu.input.scroll += match delta {
            MouseScrollDelta::LineDelta(_, y) => y,
            #[allow(clippy::cast_possible_truncation)]
            MouseScrollDelta::PixelDelta(p) => (p.y / 40.0) as f32,
        };
    }

    /// Tastatur in Menüs; Esc schließt die Pause bzw. führt zur ersten Seite zurück.
    pub(crate) fn menu_key(&mut self, code: KeyCode, event: &KeyEvent) {
        if !event.state.is_pressed() {
            return;
        }
        let key = match code {
            KeyCode::Enter | KeyCode::NumpadEnter => Some(UiKey::Enter),
            KeyCode::Escape => Some(UiKey::Escape),
            KeyCode::Backspace => Some(UiKey::Backspace),
            KeyCode::Delete => Some(UiKey::Delete),
            KeyCode::ArrowLeft => Some(UiKey::Left),
            KeyCode::ArrowRight => Some(UiKey::Right),
            KeyCode::Home => Some(UiKey::Home),
            KeyCode::End => Some(UiKey::End),
            KeyCode::Tab => Some(UiKey::Tab),
            _ => None,
        };
        if code == KeyCode::Escape && self.menu.ui.focus.is_none() {
            if self.menu.paused {
                self.toggle_pause();
            } else {
                self.menu.page = crate::menu::Page::Play;
            }
            return;
        }
        if let Some(k) = key {
            self.menu.input.keys.push(k);
        }
        if let Some(text) = &event.text
            && key.is_none()
        {
            self.menu.input.text.push_str(text);
        }
    }
}

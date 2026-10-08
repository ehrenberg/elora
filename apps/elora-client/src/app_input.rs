//! Input handling: keys, mouse and key bindings to game actions.

use elora_client::scene::Scene;
use elora_sim::Event;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::CursorGrabMode;

use crate::bindings::{GameAction, Trigger};
use crate::debug_ui::Action;
use crate::{App, Screen, emotes, game_ui};

impl App {
    /// A key: rebinding, intro, halted adventure, menus, chat or the game, in this order.
    pub(crate) fn key(&mut self, event: &KeyEvent) {
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        let pressed = event.state.is_pressed();
        if self.bind_capture.is_some() {
            self.capture_trigger(Trigger::Key(code), pressed);
            return;
        }
        if self.screen == Screen::Editor {
            return; // keyboard belongs to egui (shortcuts in the editor)
        }
        if self.intro.is_some() {
            if pressed && !event.repeat {
                self.intro_key(code);
            }
            return;
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
            self.chat_key(code, event); // keyboard belongs to the chat field
            return;
        }
        if pressed && !event.repeat && self.game_shortcut(code) {
            return;
        }
        if !event.repeat {
            self.trigger(Trigger::Key(code), pressed);
        }
    }

    /// Fixed keys in the game (Esc, F1, and R/F5 in the sandbox when unbound);
    /// `true` if the key was used.
    fn game_shortcut(&mut self, code: KeyCode) -> bool {
        let unbound = self
            .settings
            .bindings
            .actions(Trigger::Key(code))
            .is_empty();
        let online = self.online.is_some();
        match code {
            KeyCode::Escape if self.testing_map() => self.leave_editor_test(),
            KeyCode::Escape if self.testing_adventure() => self.leave_adventure(),
            KeyCode::Escape => self.toggle_pause(),
            KeyCode::F1 => {
                // panel open: mouse free for operating it; closed: back into the game
                self.show_panel = !self.show_panel;
                self.set_cursor_grab(!self.show_panel && !self.menu.paused);
            }
            KeyCode::KeyR if unbound && !online && self.adventure.is_none() => {
                self.apply(Action::Respawn);
            }
            KeyCode::F5 if unbound && !online && self.adventure.is_none() => {
                self.status = if self.sandbox.rules.is_some() {
                    "recording only without a game mode (golden tests = pure simulation)".into()
                } else {
                    self.sandbox.stop_recording("F5").unwrap_or_else(|| {
                        self.sandbox.start_recording();
                        "recording … (F5 stops)".into()
                    })
                };
            }
            _ => return false,
        }
        true
    }

    /// Keyboard with open chat: Enter sends, Esc cancels, Backspace deletes.
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

    /// All actions that `t` triggers (key bindings, M7.5).
    fn trigger(&mut self, t: Trigger, down: bool) {
        for action in self.settings.bindings.actions(t) {
            self.game_action(action, down);
        }
    }

    /// A mouse button: rebinding, skipping the intro, menus, the game or grabbing the mouse.
    pub(crate) fn mouse_button(&mut self, button: MouseButton, state: ElementState) {
        if self.bind_capture.is_some() {
            self.capture_trigger(Trigger::Mouse(button), state.is_pressed());
        } else if self.intro.is_some() {
            if state.is_pressed() {
                self.skip_intro();
            }
        } else if self.screen == Screen::Editor {
            // mouse belongs to egui
        } else if self.menu_active() {
            self.menu_mouse_button(button, state);
        } else if self.cursor_grabbed {
            self.trigger(Trigger::Mouse(button), state.is_pressed());
        } else if state.is_pressed() && !self.gfx.as_ref().is_some_and(|g| g.gui.wants_pointer()) {
            self.set_cursor_grab(true);
        }
    }

    /// The mouse wheel: rebinding, menus, or one press and release per notch in the game.
    pub(crate) fn mouse_wheel(&mut self, delta: MouseScrollDelta) {
        if self.bind_capture.is_some() {
            let up = match delta {
                MouseScrollDelta::LineDelta(_, y) => y > 0.0,
                MouseScrollDelta::PixelDelta(p) => p.y > 0.0,
            };
            let t = if up {
                Trigger::WheelUp
            } else {
                Trigger::WheelDown
            };
            self.capture_trigger(t, true);
        } else if self.menu_active() {
            self.menu_wheel(delta);
        } else if self.cursor_grabbed {
            let notches = match delta {
                MouseScrollDelta::LineDelta(_, y) => y.round() as i32,
                MouseScrollDelta::PixelDelta(p) => (p.y / 40.0).round() as i32,
            };
            let t = if notches > 0 {
                Trigger::WheelUp
            } else {
                Trigger::WheelDown
            };
            // per notch press + release
            for _ in 0..notches.unsigned_abs().min(8) {
                self.trigger(t, true);
                self.trigger(t, false);
            }
        }
    }

    /// A bound action in the game.
    fn game_action(&mut self, action: GameAction, down: bool) {
        if self.controls.action(action, down) {
            return;
        }
        let online = self.online.is_some();
        match action {
            // in the adventure Tab opens the adventure menu (E-263)
            GameAction::Scoreboard if self.adventure.is_some() => {
                if down {
                    self.toggle_adventure_menu();
                }
            }
            // scoreboard while held (E-078)
            GameAction::Scoreboard => self.scoreboard = down,
            GameAction::QuickHeal if down => self.quick_heal(),
            // emote wheel while held (E-091); select on release
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

    /// Rebind: take the next key/mouse button/wheel direction, Esc cancels.
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

    /// Show an emote: online via the server, in the sandbox directly.
    fn send_emote(&mut self, emote: u8) {
        match &mut self.online {
            Some(o) => o.client.emote(emote),
            None => self.emotes.show(self.sandbox.player, emote),
        }
    }

    /// Switch to the picked-up weapon (E-287); `owned_weapons` keeps the state before the frame.
    pub(crate) fn auto_switch(&mut self, scene: &Scene, events: &[Event]) {
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

    pub(crate) fn set_cursor_grab(&mut self, grab: bool) {
        let Some(gfx) = &self.gfx else { return };
        let window = &gfx.window;
        if grab {
            let ok = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
                .is_ok();
            if !ok {
                tracing::warn!("could not capture the mouse pointer");
            }
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            self.controls.release_all();
        }
        window.set_cursor_visible(!grab);
        self.cursor_grabbed = grab;
    }
}

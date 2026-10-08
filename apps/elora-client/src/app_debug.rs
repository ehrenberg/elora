//! Debug panel (egui) and its actions.

use crate::app_state::{known_servers_path, tuning_path};
use crate::debug_ui::{self, Action, OnlineView};
use crate::tuning_file::{TUNING_FILE, TuningFile};
use crate::{App, FrameInfo};

impl App {
    pub(crate) fn apply(&mut self, action: Action) {
        let path = &tuning_path();
        match action {
            Action::Save => {
                self.status = match self.tuning_file().save(path) {
                    Ok(()) => format!("saved to {TUNING_FILE}"),
                    Err(e) => format!("error: {e:#}"),
                };
            }
            Action::SwitchMap(path) => {
                self.status = match self.sandbox.switch_map(&path) {
                    Ok(()) => format!("map {} loaded", path.display()),
                    Err(e) => format!("error: {e:#}"),
                };
            }
            Action::Load => {
                self.status = match TuningFile::load(path) {
                    Ok(f) => {
                        self.sandbox.world.tuning = f.physics;
                        self.view = f.view.into();
                        format!("loaded from {TUNING_FILE}")
                    }
                    Err(e) => format!("error: {e:#}"),
                };
            }
            Action::Respawn => {
                self.status = self.sandbox.stop_recording("Respawn").unwrap_or_default();
                self.sandbox.spawn_now();
            }
            Action::Connect => self.connect(),
            Action::Disconnect => {
                self.online = None;
                self.status = "disconnected – back in the sandbox".into();
            }
            Action::HostStart => match self.net.hosting.start() {
                Ok(address) => {
                    self.online = None;
                    self.net.address = address;
                    self.net.show_host = false;
                    self.connect();
                }
                Err(e) => {
                    self.net.hosting.status =
                        self.lang.f("menu.host_failed", &[("e", &format!("{e:#}"))]);
                }
            },
            Action::HostStop => self.net.hosting.stop(),
            Action::TrustNewKey => {
                if let Some(w) = self.net.key_warning.take() {
                    let got: Vec<u8> = (0..w.got.len())
                        .step_by(2)
                        .filter_map(|i| u8::from_str_radix(&w.got[i..i + 2], 16).ok())
                        .collect();
                    if let Err(e) = self.known.trust(&known_servers_path(), w.server, &got) {
                        self.status = format!("error: {e:#}");
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
                self.status = cfg
                    .as_ref()
                    .map_or_else(|| "free play".into(), |c| format!("mode: {}", c.title()));
                self.sandbox.set_mode(cfg);
            }
            Action::ApplyConditions => {
                if let Some(o) = &self.online {
                    o.conn.set_conditions(self.net.conditions());
                }
            }
        }
    }

    /// Debug panel (egui, E-031) above the frame; executes its action.
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
        let weather_before = self.sandbox.map.weather;
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
        // weather switched in the debug panel: in the adventure also the gameplay effect (W1.7)
        if self.sandbox.map.weather != weather_before
            && let Some(a) = &mut self.adventure
        {
            a.session.map.weather = self.sandbox.map.weather;
        }
        if let Some(action) = action {
            self.apply(action);
        }
    }
}

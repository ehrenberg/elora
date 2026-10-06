//! Debug-Panel (M1.5, M2.8, M3.9): Tuning-Regler, Zustandsanzeige, Netzwerk.

use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::time::Duration;

use elora_client::online::OnlineClient;
use elora_game::{Mode, RulesConfig};
use elora_net::{Conditions, Stats};
use elora_protocol::VoteKind;
use elora_render::ViewSettings;
use elora_sim::{HookState, TICKS_PER_SECOND, TILE_SIZE, Team, Tuning};
use std::collections::BTreeMap;

use crate::controls::Controls;
use crate::hosting::{Hosting, available_maps};
use crate::sandbox::Sandbox;

/// Aktion, die das Panel ausgelöst hat.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    SetTeam(Team),
    Kill,
    CallVote(VoteKind),
    Vote(bool),
    /// Spielmodus der Sandbox (E-075); `None` = ohne Regeln.
    SandboxMode(Option<RulesConfig>),
    Save,
    Load,
    /// Andere Karte in der Sandbox laden.
    SwitchMap(std::path::PathBuf),
    Respawn,
    Connect,
    Disconnect,
    HostStart,
    HostStop,
    TrustNewKey,
    ApplyConditions,
}

/// Warnung bei geändertem Server-Schlüssel (E-062).
#[derive(Debug, Clone)]
pub struct KeyWarning {
    pub server: SocketAddr,
    pub expected: String,
    pub got: String,
}

/// Ton im Panel: Einstellungen und ob ein Audiogerät gefunden wurde.
#[derive(Debug)]
pub struct AudioUi<'a> {
    pub settings: &'a mut elora_audio::AudioSettings,
    pub device: bool,
}

/// Netzwerk-Einstellungen im Panel.
#[derive(Debug)]
pub struct NetUi {
    pub address: String,
    pub name: String,
    /// Eigener Skin (M5.4).
    pub skin: elora_protocol::Skin,
    /// Simulator (ausgehende Pakete dieses Clients).
    pub latency_ms: f32,
    pub jitter_ms: f32,
    pub loss_pct: f32,
    pub show_host: bool,
    pub hosting: Hosting,
    pub key_warning: Option<KeyWarning>,
    /// Abstimmung vorbereiten: 0 Karte, 1 Modus, 2 Kick, 3 Zuschauer.
    pub vote_kind: u8,
    pub vote_map: String,
    pub vote_mode: Mode,
    pub vote_instagib: bool,
    pub vote_target: Option<usize>,
    /// Modus der Sandbox (Auswahl im Panel).
    pub sandbox_mode: Option<Mode>,
    pub sandbox_instagib: bool,
    /// Gewählte Gegnerart zum Setzen in der Sandbox (A1.2).
    pub creature_kind: usize,
    /// Gewählte Karte zum Wechseln in der Sandbox.
    pub map_choice: Option<std::path::PathBuf>,
}

impl Default for NetUi {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:8303".into(),
            name: "Elora".into(),
            skin: elora_protocol::Skin::default(),
            latency_ms: 0.0,
            jitter_ms: 0.0,
            loss_pct: 0.0,
            show_host: false,
            hosting: Hosting::default(),
            key_warning: None,
            vote_kind: 0,
            vote_map: "sandbox".into(),
            vote_mode: Mode::Dm,
            vote_instagib: false,
            vote_target: None,
            sandbox_mode: None,
            sandbox_instagib: false,
            creature_kind: 0,
            map_choice: None,
        }
    }
}

impl NetUi {
    pub fn conditions(&self) -> Conditions {
        Conditions {
            latency: Duration::from_secs_f32(self.latency_ms.max(0.0) / 1000.0),
            jitter: Duration::from_secs_f32(self.jitter_ms.max(0.0) / 1000.0),
            loss: self.loss_pct.clamp(0.0, 100.0) / 100.0,
            duplicate: 0.0,
        }
    }
}

/// Angaben zum Online-Spiel.
pub struct OnlineView<'a> {
    pub client: &'a OnlineClient,
    pub stats: Option<Stats>,
    pub server: SocketAddr,
}

/// Zustand, den das Panel anzeigen und ändern darf.
pub struct Context<'a> {
    /// `None`, solange online gespielt wird.
    pub sandbox: Option<&'a mut Sandbox>,
    pub online: Option<OnlineView<'a>>,
    pub net: &'a mut NetUi,
    pub view: &'a mut ViewSettings,
    pub effects: &'a mut crate::effects::EffectSettings,
    pub audio: AudioUi<'a>,
    pub controls: &'a mut Controls,
    pub fps: f32,
    pub frames: FrameStats,
    pub status: &'a str,
    pub cursor_grabbed: bool,
    /// Namen und eigener Slot (für den Spiel-Abschnitt).
    pub names: &'a BTreeMap<usize, String>,
    pub local: Option<usize>,
    /// Modus mit Teams aktiv?
    pub team_mode: bool,
    pub vote_running: bool,
}

pub fn panel(ui: &mut egui::Ui, cx: &mut Context<'_>) -> Option<Action> {
    let mut action = None;
    egui::Panel::right("debug")
        .default_size(340.0)
        .resizable(true)
        .show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(if cx.online.is_some() {
                    "Elora – Online"
                } else {
                    "Elora – Sandbox"
                });
                if cx.cursor_grabbed {
                    ui.label("F1 zweimal: Maus wieder für das Panel freigeben");
                } else {
                    ui.label("Ins Spielfeld klicken, um weiterzuspielen; F1 schließt das Panel");
                }
                ui.separator();
                action = network(ui, cx);
                ui.separator();
                action = action.take().or(game_section(ui, cx));
                ui.separator();
                if let Some(sandbox) = cx.sandbox.as_deref_mut() {
                    action = action
                        .take()
                        .or(map_picker(ui, sandbox, &mut cx.net.map_choice));
                    state(ui, sandbox, cx.fps, &cx.frames);
                    ui.separator();
                    action = action.take().or(buttons(ui, cx.status));
                    ui.separator();
                    abilities(ui, sandbox);
                    creatures(ui, sandbox, &mut cx.net.creature_kind);
                    weather(ui, sandbox);
                    ui.separator();
                    tuning(ui, &mut sandbox.world.tuning);
                } else if !cx.status.is_empty() {
                    ui.small(cx.status);
                }
                egui::CollapsingHeader::new("Aussehen")
                    .default_open(false)
                    .show(ui, |ui| {
                        crate::skins::picker(ui, &mut cx.net.skin);
                        ui.small("In Team-Modi trägt der Körper die Teamfarbe.");
                    });
                egui::CollapsingHeader::new("Effekte")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.checkbox(&mut cx.effects.camera_shake, "Kamera-Wackeln");
                        ui.checkbox(&mut cx.effects.hit_marker, "Treffer-Marker");
                        ui.small("„Speichern“ legt die Schalter in tuning.toml ab.");
                    });
                egui::CollapsingHeader::new("Ton")
                    .default_open(false)
                    .show(ui, |ui| {
                        if !cx.audio.device {
                            ui.label("Kein Audiogerät gefunden – das Spiel bleibt stumm.");
                        }
                        ui.add(
                            egui::Slider::new(&mut cx.audio.settings.volume, 0.0..=1.0)
                                .text("Lautstärke"),
                        );
                        ui.checkbox(&mut cx.audio.settings.muted, "Stumm");
                        ui.small("„Speichern“ legt die Werte in tuning.toml ab.");
                    });
                view(ui, cx.view);
                egui::CollapsingHeader::new("Eingabe")
                    .default_open(false)
                    .show(ui, |ui| {
                        slider(
                            ui,
                            "Maus-Empfindlichkeit %",
                            &mut cx.controls.sensitivity,
                            10.0..=400.0,
                            100.0,
                        );
                    });
                ui.separator();
                help(ui);
            });
        });
    if cx.net.show_host {
        action = action.take().or(host_window(ui.ctx(), cx.net));
    }
    if let Some(w) = cx.net.key_warning.clone() {
        action = action.take().or(key_warning_window(ui.ctx(), cx.net, &w));
    }
    action
}

fn network(ui: &mut egui::Ui, cx: &mut Context<'_>) -> Option<Action> {
    let mut action = None;
    egui::CollapsingHeader::new("Netzwerk")
        .default_open(false)
        .show(ui, |ui| {
            if let Some(o) = &cx.online {
                online_state(ui, o);
                if ui.button("Trennen (zurück zur Sandbox)").clicked() {
                    action = Some(Action::Disconnect);
                }
                ui.label("Netzwerk-Simulator (ausgehend):");
                let mut changed = false;
                changed |= ui
                    .add(
                        egui::Slider::new(&mut cx.net.latency_ms, 0.0..=300.0)
                            .text("Verzögerung ms"),
                    )
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut cx.net.jitter_ms, 0.0..=100.0).text("Jitter ms"))
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut cx.net.loss_pct, 0.0..=50.0).text("Verlust %"))
                    .changed();
                if changed {
                    action = Some(Action::ApplyConditions);
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.add(egui::TextEdit::singleline(&mut cx.net.name).desired_width(120.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Server");
                    ui.add(egui::TextEdit::singleline(&mut cx.net.address).desired_width(150.0));
                    if ui.button("Verbinden").clicked() {
                        action = Some(Action::Connect);
                    }
                });
                if ui.button("Server einrichten …").clicked() {
                    cx.net.show_host = true;
                }
            }
            if !cx.net.hosting.status.is_empty() {
                ui.small(&cx.net.hosting.status);
            }
        });
    action
}

fn game_section(ui: &mut egui::Ui, cx: &mut Context<'_>) -> Option<Action> {
    let mut action = None;
    egui::CollapsingHeader::new("Spiel")
        .default_open(false)
        .show(ui, |ui| {
            let online = cx.online.is_some();
            if !online {
                action = sandbox_mode(ui, cx.net);
            }
            action = action.take().or(team_buttons(ui, cx.team_mode));
            if online {
                action = action.take().or(vote_ui(ui, cx));
            }
        });
    action
}

/// Spielmodus der Sandbox (E-075).
fn sandbox_mode(ui: &mut egui::Ui, net: &mut NetUi) -> Option<Action> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label("Modus");
        let label = net.sandbox_mode.map_or("aus (freies Spiel)", Mode::name);
        egui::ComboBox::from_id_salt("sbmode")
            .selected_text(label)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut net.sandbox_mode, None, "aus (freies Spiel)");
                for m in Mode::ALL {
                    ui.selectable_value(&mut net.sandbox_mode, Some(m), m.name());
                }
            });
        ui.checkbox(&mut net.sandbox_instagib, "Instagib");
        if ui.button("Starten").clicked() {
            let cfg = net.sandbox_mode.map(|mode| RulesConfig {
                mode,
                instagib: net.sandbox_instagib,
                warmup_secs: 0,
                ..RulesConfig::default()
            });
            action = Some(Action::SandboxMode(cfg));
        }
    });
    ui.small("CTF braucht eine Karte mit Flaggen, z. B. maps/ctf-test.emap");
    action
}

fn team_buttons(ui: &mut egui::Ui, team_mode: bool) -> Option<Action> {
    let mut action = None;
    ui.horizontal(|ui| {
        if team_mode {
            if ui.button("Rot").clicked() {
                action = Some(Action::SetTeam(Team::Red));
            }
            if ui.button("Blau").clicked() {
                action = Some(Action::SetTeam(Team::Blue));
            }
        } else if ui.button("Mitspielen").clicked() {
            action = Some(Action::SetTeam(Team::None));
        }
        if ui.button("Zuschauen").clicked() {
            action = Some(Action::SetTeam(Team::Spectator));
        }
        if ui.button("kill (K)").clicked() {
            action = Some(Action::Kill);
        }
    });
    action
}

/// Abstimmung starten und abstimmen (E-077).
fn vote_ui(ui: &mut egui::Ui, cx: &mut Context<'_>) -> Option<Action> {
    let mut action = None;
    ui.label("Abstimmung (E-077):");
    if cx.vote_running {
        ui.horizontal(|ui| {
            if ui.button("Ja (F3)").clicked() {
                action = Some(Action::Vote(true));
            }
            if ui.button("Nein (F4)").clicked() {
                action = Some(Action::Vote(false));
            }
        });
    }
    let net = &mut *cx.net;
    ui.horizontal(|ui| {
        for (k, label) in ["Karte", "Modus", "Kick", "Zuschauer"].iter().enumerate() {
            ui.selectable_value(&mut net.vote_kind, k as u8, *label);
        }
    });
    match net.vote_kind {
        0 => {
            ui.horizontal(|ui| {
                ui.label("Karte");
                ui.add(egui::TextEdit::singleline(&mut net.vote_map).desired_width(140.0));
            });
        }
        1 => {
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("vmode")
                    .selected_text(net.vote_mode.name())
                    .show_ui(ui, |ui| {
                        for m in Mode::ALL {
                            ui.selectable_value(&mut net.vote_mode, m, m.name());
                        }
                    });
                ui.checkbox(&mut net.vote_instagib, "Instagib");
            });
        }
        _ => {
            let label = net
                .vote_target
                .and_then(|t| cx.names.get(&t))
                .cloned()
                .unwrap_or_else(|| "Spieler wählen".into());
            egui::ComboBox::from_id_salt("vtarget")
                .selected_text(label)
                .show_ui(ui, |ui| {
                    for (i, n) in cx.names {
                        if Some(*i) != cx.local {
                            ui.selectable_value(&mut net.vote_target, Some(*i), n);
                        }
                    }
                });
        }
    }
    if ui
        .add_enabled(!cx.vote_running, egui::Button::new("Abstimmung starten"))
        .clicked()
    {
        let slot = |t: Option<usize>| t.and_then(|t| u32::try_from(t).ok());
        let kind = match net.vote_kind {
            0 => Some(VoteKind::Map(net.vote_map.trim().to_owned())),
            1 => Some(VoteKind::Mode {
                mode: net.vote_mode,
                instagib: net.vote_instagib,
            }),
            2 => slot(net.vote_target).map(VoteKind::Kick),
            _ => slot(net.vote_target).map(VoteKind::Spectate),
        };
        action = kind.map(Action::CallVote);
    }
    action
}

fn online_state(ui: &mut egui::Ui, o: &OnlineView<'_>) {
    let c = o.client;
    let info = c.info;
    egui::Grid::new("net")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            let mut row = |k: &str, v: String| {
                ui.label(k);
                ui.monospace(v);
                ui.end_row();
            };
            row("Server", o.server.to_string());
            row("Status", format!("{:?}", c.status));
            row("Karte / Slot", format!("{} / {:?}", c.map_name, c.slot));
            row(
                "Snapshots",
                if c.high_bandwidth {
                    "50 Hz".into()
                } else {
                    "25 Hz".into()
                },
            );
            if let Some(s) = o.stats {
                row("Ping", format!("{:.0} ms", s.rtt.as_secs_f64() * 1000.0));
                row("Verlust", format!("{:.1} %", s.loss * 100.0));
                row(
                    "Gesendet / Empfangen",
                    format!("{} / {} KiB", s.bytes_sent / 1024, s.bytes_received / 1024),
                );
            }
            row("Vorhersage", format!("{} Ticks", info.prediction_ticks));
            row("Vorlauf", format!("{:.0} ms", info.lead_ms));
            row(
                "Eingabe-Restzeit",
                format!("{} ms", info.input_time_left_ms),
            );
            row("Snapshot-Größe", format!("{} B", info.snapshot_bytes));
            row("Snapshot-Fehler", info.snapshot_errors.to_string());
            row("Korrektur", format!("{:.1} E", info.correction));
        });
}

fn host_window(ctx: &egui::Context, net: &mut NetUi) -> Option<Action> {
    let mut action = None;
    let mut open = true;
    egui::Window::new("Server einrichten")
        .open(&mut open)
        .resizable(false)
        .show(ctx, |ui| {
            let running = net.hosting.is_running();
            let c = &mut net.hosting.config;
            egui::Grid::new("host").num_columns(2).show(ui, |ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut c.name);
                ui.end_row();
                ui.label("Port");
                ui.add(egui::DragValue::new(&mut c.port).range(1024..=65535));
                ui.end_row();
                ui.label("Karte");
                egui::ComboBox::from_id_salt("map")
                    .selected_text(c.map.display().to_string())
                    .show_ui(ui, |ui| {
                        for m in available_maps() {
                            let label = m.display().to_string();
                            ui.selectable_value(&mut c.map, m, label);
                        }
                    });
                ui.end_row();
                ui.label("Max. Spieler");
                ui.add(egui::Slider::new(
                    &mut c.max_clients,
                    1..=elora_server::config::MAX_CLIENTS,
                ));
                ui.end_row();
                ui.label("Snapshots");
                ui.checkbox(&mut c.high_bandwidth, "50 Hz (nur LAN)");
                ui.end_row();
            });
            ui.checkbox(
                &mut net.hosting.keep_running,
                "Server beim Beenden des Clients weiterlaufen lassen",
            );
            ui.horizontal(|ui| {
                let label = if running {
                    "Neu starten und verbinden"
                } else {
                    "Starten und verbinden"
                };
                if ui.button(label).clicked() {
                    action = Some(Action::HostStart);
                }
                if running && ui.button("Server stoppen").clicked() {
                    action = Some(Action::HostStop);
                }
            });
            ui.small("Die Einstellungen werden in server.toml gespeichert.");
        });
    if !open {
        net.show_host = false;
    }
    action
}

fn key_warning_window(ctx: &egui::Context, net: &mut NetUi, w: &KeyWarning) -> Option<Action> {
    let mut action = None;
    egui::Window::new("Achtung: Server-Schlüssel geändert").collapsible(false).resizable(false).show(ctx, |ui| {
        ui.label(format!("Der Server {} meldet einen anderen Schlüssel als beim letzten Mal.", w.server));
        ui.label("Das kann ein neu eingerichteter Server sein – oder ein Angriff (jemand gibt sich als Server aus).");
        ui.monospace(format!("bekannt: {}", &w.expected[..w.expected.len().min(32)]));
        ui.monospace(format!("neu:     {}", &w.got[..w.got.len().min(32)]));
        ui.horizontal(|ui| {
            if ui.button("Neuem Schlüssel vertrauen und verbinden").clicked() {
                action = Some(Action::TrustNewKey);
            }
            if ui.button("Abbrechen").clicked() {
                net.key_warning = None;
            }
        });
    });
    action
}

/// Bildzeiten der letzten Frames (Verwischen eingrenzen, E-288).
#[derive(Debug, Clone, Copy, Default)]
pub struct FrameStats {
    pub min_ms: f32,
    pub avg_ms: f32,
    pub max_ms: f32,
    /// Bildwiederholrate des Bildschirms, falls bekannt.
    pub refresh_hz: Option<f32>,
    pub vsync: bool,
}

impl FrameStats {
    pub fn of(
        times: &std::collections::VecDeque<f32>,
        millihertz: Option<u32>,
        vsync: bool,
    ) -> Self {
        let n = times.len().max(1);
        #[allow(clippy::cast_precision_loss)]
        let avg_ms = times.iter().sum::<f32>() / n as f32;
        #[allow(clippy::cast_precision_loss)]
        Self {
            min_ms: times.iter().copied().fold(f32::INFINITY, f32::min),
            avg_ms,
            max_ms: times.iter().copied().fold(0.0, f32::max),
            refresh_hz: millihertz.map(|m| m as f32 / 1000.0),
            vsync,
        }
    }
}

fn state(ui: &mut egui::Ui, s: &Sandbox, fps: f32, frames: &FrameStats) {
    let tps = TICKS_PER_SECOND as f32;
    let tiles_per_s = |v: f32| v * tps / TILE_SIZE as f32;
    egui::Grid::new("state")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            let mut row = |k: &str, v: String| {
                ui.label(k);
                ui.monospace(v);
                ui.end_row();
            };
            row("FPS / Tick", format!("{:.0} / {}", fps, s.world.tick));
            row(
                "Bildzeit min/Ø/max",
                format!(
                    "{:.1} / {:.1} / {:.1} ms",
                    frames.min_ms, frames.avg_ms, frames.max_ms
                ),
            );
            row(
                "Bildschirm",
                format!(
                    "{} · VSync {}",
                    frames
                        .refresh_hz
                        .map_or_else(|| "? Hz".to_owned(), |h| format!("{h:.0} Hz")),
                    if frames.vsync { "an" } else { "aus" }
                ),
            );
            if let Some(ch) = s.character() {
                let c = &ch.core;
                let hook = match c.hook_state {
                    HookState::Idle => "bereit".to_string(),
                    HookState::Flying => "fliegt".to_string(),
                    HookState::Grabbed if c.hooked_player.is_some() => {
                        format!("hält Spieler ({})", c.hook_tick)
                    }
                    HookState::Grabbed => "hängt an Wand".to_string(),
                    HookState::Retracting(n) => format!("fährt ein ({n}/3)"),
                    HookState::Retracted => "eingefahren (Taste loslassen)".to_string(),
                };
                let grounded = if c.is_grounded(&s.world.collision) {
                    "ja"
                } else {
                    "nein"
                };
                let double = if c.jumped & 2 == 0 {
                    "verfügbar"
                } else {
                    "verbraucht"
                };
                row("Position", format!("{:.0}, {:.0}", c.pos.x, c.pos.y));
                row(
                    "Geschw. (E/Tick)",
                    format!("{:+.2}, {:+.2}", c.vel.x, c.vel.y),
                );
                row(
                    "Geschw. (Tiles/s)",
                    format!("{:+.1}, {:+.1}", tiles_per_s(c.vel.x), tiles_per_s(c.vel.y)),
                );
                row("Am Boden", grounded.into());
                row("Doppelsprung", double.into());
                row("Hook", hook);
                row("Leben / Rüstung", format!("{} / {}", ch.health, ch.armor));
                row(
                    "Waffe / Reload",
                    format!(
                        "{:?} / {} Ticks",
                        ch.arsenal.active, ch.arsenal.reload_timer
                    ),
                );
            } else {
                row("Elora", "tot".into());
            }
            let dummies = s
                .world
                .players
                .iter()
                .flatten()
                .filter(|p| p.is_dummy())
                .count();
            row("Dummies", dummies.to_string());
            row("Karte", s.map_path.display().to_string());
            row(
                "Aufzeichnung",
                s.recording
                    .as_ref()
                    .map_or_else(|| "aus (F5)".into(), |r| format!("● {} Ticks", r.len())),
            );
        });
    if let Some(err) = &s.reload_error {
        ui.colored_label(egui::Color32::LIGHT_RED, format!("Karte ungültig: {err}"));
    }
}

fn buttons(ui: &mut egui::Ui, status: &str) -> Option<Action> {
    let mut action = None;
    ui.horizontal(|ui| {
        if ui
            .button("Speichern")
            .on_hover_text("Werte in tuning.toml schreiben")
            .clicked()
        {
            action = Some(Action::Save);
        }
        if ui
            .button("Laden")
            .on_hover_text("tuning.toml neu einlesen")
            .clicked()
        {
            action = Some(Action::Load);
        }
        if ui.button("Respawn (R)").clicked() {
            action = Some(Action::Respawn);
        }
    });
    if !status.is_empty() {
        ui.small(status);
    }
    action
}

/// Fähigkeiten zum Ausprobieren (A1.1); im normalen Mehrspieler gibt es sie nicht (E-223).
fn abilities(ui: &mut egui::Ui, s: &mut Sandbox) {
    use elora_sim::{Abilities, Ability};
    egui::CollapsingHeader::new("Fähigkeiten (Abenteuer)")
        .default_open(false)
        .show(ui, |ui| {
            let mut a = s
                .world
                .player(s.player)
                .map_or(Abilities::NONE, |p| p.abilities);
            let before = a;
            for (ability, label) in [
                (
                    Ability::HookRuck,
                    "Hook-Ruck (Fähigkeit-Taste bei hängendem Hook)",
                ),
                (
                    Ability::Pull,
                    "Heranhooken (Hook zieht Gegner, Dinge, Zugschalter)",
                ),
                (Ability::Stomp, "Stampfen (Runter in der Luft)"),
                (Ability::Grip, "Eisgriff (gegen Kletterwand laufen)"),
                (
                    Ability::Glide,
                    "Gleiten (Springen halten nach Doppelsprung)",
                ),
            ] {
                let mut on = a.has(ability);
                ui.checkbox(&mut on, label);
                a.set(ability, on);
            }
            ui.horizontal(|ui| {
                if ui.button("Alle").clicked() {
                    a = Abilities::ALL;
                }
                if ui.button("Keine").clicked() {
                    a = Abilities::NONE;
                }
            });
            if a != before {
                s.world.set_abilities(s.player, a);
            }
        });
}

/// Karte wechseln: alle mitgelieferten und eigenen Karten (`hosting::available_maps`).
fn map_picker(
    ui: &mut egui::Ui,
    s: &Sandbox,
    choice: &mut Option<std::path::PathBuf>,
) -> Option<Action> {
    let mut action = None;
    egui::CollapsingHeader::new("Karte")
        .default_open(false)
        .show(ui, |ui| {
            let maps = crate::hosting::available_maps();
            let name = |p: &std::path::Path| {
                p.file_stem()
                    .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
            };
            let current = choice.clone().unwrap_or_else(|| s.map_path.clone());
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("map_choice")
                    .selected_text(name(&current))
                    .show_ui(ui, |ui| {
                        for m in &maps {
                            if ui.selectable_label(*m == current, name(m)).clicked() {
                                *choice = Some(m.clone());
                            }
                        }
                    });
                if ui.button("Laden").clicked() {
                    action = Some(Action::SwitchMap(current.clone()));
                    *choice = None;
                }
            });
            ui.small(format!("Aktuell: {}", s.map_path.display()));
        });
    action
}

/// Wetter der Karte zum Ausprobieren (R2-W1): Art, Stärke, Wind – wirkt sofort.
fn weather(ui: &mut egui::Ui, s: &mut Sandbox) {
    use elora_map::WeatherKind;
    egui::CollapsingHeader::new("Wetter")
        .default_open(false)
        .show(ui, |ui| {
            let w = &mut s.map.weather;
            egui::ComboBox::from_id_salt("debug_weather")
                .selected_text(w.kind.key())
                .show_ui(ui, |ui| {
                    for k in WeatherKind::ALL {
                        ui.selectable_value(&mut w.kind, k, k.key());
                    }
                });
            if !w.is_clear() && w.intensity <= 0.0 {
                w.intensity = 0.7;
            }
            ui.add(egui::Slider::new(&mut w.intensity, 0.0..=1.0).text("Stärke"));
            ui.add(egui::Slider::new(&mut w.wind, -1.0..=1.0).text("Wind"));
        });
}

/// Gegner zum Ausprobieren (A1.2): Abenteuer-Regeln, Gegner setzen und entfernen.
fn creatures(ui: &mut egui::Ui, s: &mut Sandbox, kind: &mut usize) {
    egui::CollapsingHeader::new("Gegner (Abenteuer)")
        .default_open(false)
        .show(ui, |ui| {
            ui.checkbox(
                &mut s.world.adventure,
                "Abenteuer-Regeln (Schutz nach Treffer, kein Eigenschaden)",
            );
            let names: Vec<String> = s
                .world
                .creature_kinds
                .iter()
                .map(|k| k.name.clone())
                .collect();
            if names.is_empty() {
                return;
            }
            *kind = (*kind).min(names.len() - 1);
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("creature_kind")
                    .selected_text(&names[*kind])
                    .show_ui(ui, |ui| {
                        for (i, n) in names.iter().enumerate() {
                            ui.selectable_value(kind, i, n);
                        }
                    });
                if ui.button("Setzen").clicked()
                    && let Some(c) = s.world.character(s.player)
                {
                    // vor Elora, in Blickrichtung
                    let a = c.core.angle as f32 / 256.0;
                    let side = if a.cos() < 0.0 { -1.0 } else { 1.0 };
                    let pos = c.core.pos + elora_sim::Vec2::new(side * 160.0, -40.0);
                    s.world.add_creature(*kind, pos);
                }
                if ui.button("Alle entfernen").clicked() {
                    s.world.creatures.clear();
                    s.world.creature_shots.clear();
                    s.world.loot.clear();
                }
            });
            ui.small(format!(
                "{} Gegner, {} Beute liegt herum",
                s.world.creatures.len(),
                s.world.loot.len()
            ));
        });
}

fn tuning(ui: &mut egui::Ui, t: &mut Tuning) {
    let d = Tuning::default();
    if ui.button("Alle Werte auf Standard").clicked() {
        *t = d.clone();
    }
    section(ui, "Boden (T-02 bis T-05)", false, |ui| ground(ui, t, &d));
    section(ui, "Luft (T-06 bis T-10)", false, |ui| air(ui, t, &d));
    section(ui, "Hook (T-12 bis T-17)", false, |ui| hook(ui, t, &d));
    section(ui, "Tile-Arten (T-31 bis T-35)", false, |ui| {
        tiles(ui, t, &d);
    });
    section(ui, "Fähigkeiten & Gegner (A-01 bis A-15)", false, |ui| {
        ability_values(ui, t, &d);
    });
    section(ui, "Waffen (T-18 bis T-27)", false, |ui| {
        weapons(ui, t, &d);
        weapons_grenade(ui, t, &d);
        weapons_laser(ui, t, &d);
    });
    section(ui, "Leben & Pickups (T-28 bis T-30)", false, |ui| {
        life(ui, t, &d);
    });
    section(ui, "Velocity Ramp (T-11)", false, |ui| {
        slider(
            ui,
            "Start",
            &mut t.velramp_start,
            0.0..=5000.0,
            d.velramp_start,
        );
        slider(
            ui,
            "Bereich",
            &mut t.velramp_range,
            1.0..=10000.0,
            d.velramp_range,
        );
        slider(
            ui,
            "Krümmung",
            &mut t.velramp_curvature,
            1.0..=5.0,
            d.velramp_curvature,
        );
    });
}

fn weapons(ui: &mut egui::Ui, t: &mut Tuning, d: &Tuning) {
    ui.label("Hammer");
    int(ui, "Schaden", &mut t.hammer_damage, 0..=20, d.hammer_damage);
    int(
        ui,
        "Verzögerung ms",
        &mut t.hammer_fire_delay,
        20..=2000,
        d.hammer_fire_delay,
    );
    slider(
        ui,
        "Knockback",
        &mut t.hammer_knockback,
        0.0..=40.0,
        d.hammer_knockback,
    );
}

fn weapons_grenade(ui: &mut egui::Ui, t: &mut Tuning, d: &Tuning) {
    ui.label("Granate");
    int(
        ui,
        "Schaden",
        &mut t.grenade_damage,
        0..=20,
        d.grenade_damage,
    );
    int(
        ui,
        "Verzögerung ms",
        &mut t.grenade_fire_delay,
        20..=3000,
        d.grenade_fire_delay,
    );
    slider(
        ui,
        "Geschwindigkeit",
        &mut t.grenade_speed,
        100.0..=4000.0,
        d.grenade_speed,
    );
    slider(
        ui,
        "Krümmung",
        &mut t.grenade_curvature,
        0.0..=30.0,
        d.grenade_curvature,
    );
    slider(
        ui,
        "Lebensdauer s",
        &mut t.grenade_lifetime,
        0.1..=5.0,
        d.grenade_lifetime,
    );
    slider(
        ui,
        "Explosionsradius",
        &mut t.explosion_radius,
        20.0..=400.0,
        d.explosion_radius,
    );
    slider(
        ui,
        "Innenradius",
        &mut t.explosion_inner_radius,
        0.0..=200.0,
        d.explosion_inner_radius,
    );
    slider(
        ui,
        "Explosionskraft",
        &mut t.explosion_max_force,
        0.0..=40.0,
        d.explosion_max_force,
    );
}

fn weapons_laser(ui: &mut egui::Ui, t: &mut Tuning, d: &Tuning) {
    ui.label("Laser");
    int(ui, "Schaden", &mut t.laser_damage, 0..=20, d.laser_damage);
    int(
        ui,
        "Verzögerung ms",
        &mut t.laser_fire_delay,
        20..=3000,
        d.laser_fire_delay,
    );
    slider(
        ui,
        "Reichweite",
        &mut t.laser_reach,
        50.0..=3000.0,
        d.laser_reach,
    );
    int(
        ui,
        "Abpraller",
        &mut t.laser_bounce_num,
        0..=10,
        d.laser_bounce_num,
    );
    int(
        ui,
        "Abprall-Verz. ms",
        &mut t.laser_bounce_delay,
        0..=1000,
        d.laser_bounce_delay,
    );
    slider(
        ui,
        "Knockback",
        &mut t.laser_knockback,
        0.0..=20.0,
        d.laser_knockback,
    );
    int(ui, "Max. Munition", &mut t.max_ammo, 1..=99, d.max_ammo);
}

fn life(ui: &mut egui::Ui, t: &mut Tuning, d: &Tuning) {
    int(ui, "Max. Leben", &mut t.max_health, 1..=50, d.max_health);
    int(ui, "Max. Rüstung", &mut t.max_armor, 0..=50, d.max_armor);
    slider(
        ui,
        "Pickup-Respawn s",
        &mut t.pickup_respawn,
        0.0..=120.0,
        d.pickup_respawn,
    );
    slider(
        ui,
        "Respawn frühestens s",
        &mut t.respawn_delay,
        0.0..=10.0,
        d.respawn_delay,
    );
    slider(
        ui,
        "Auto-Respawn s",
        &mut t.auto_respawn,
        0.0..=30.0,
        d.auto_respawn,
    );
}

fn ground(ui: &mut egui::Ui, t: &mut Tuning, d: &Tuning) {
    slider(
        ui,
        "Laufgeschw.",
        &mut t.ground_control_speed,
        1.0..=30.0,
        d.ground_control_speed,
    );
    slider(
        ui,
        "Beschleunigung",
        &mut t.ground_control_accel,
        0.1..=10.0,
        d.ground_control_accel,
    );
    slider(
        ui,
        "Reibung",
        &mut t.ground_friction,
        0.0..=1.0,
        d.ground_friction,
    );
    slider(
        ui,
        "Sprungimpuls",
        &mut t.ground_jump_impulse,
        1.0..=30.0,
        d.ground_jump_impulse,
    );
}

fn tiles(ui: &mut egui::Ui, t: &mut Tuning, d: &Tuning) {
    slider(
        ui,
        "Reibung Eis",
        &mut t.ice_friction,
        0.0..=1.0,
        d.ice_friction,
    );
    slider(
        ui,
        "Beschl. Eis",
        &mut t.ice_accel,
        0.05..=10.0,
        d.ice_accel,
    );
    slider(
        ui,
        "Sprungfeld",
        &mut t.jump_pad_force,
        1.0..=40.0,
        d.jump_pad_force,
    );
    slider(
        ui,
        "Beschleuniger",
        &mut t.conveyor_speed,
        0.0..=15.0,
        d.conveyor_speed,
    );
}

fn ability_values(ui: &mut egui::Ui, t: &mut Tuning, d: &Tuning) {
    ui.label("Hook-Ruck");
    slider(ui, "Tempo", &mut t.ruck_speed, 1.0..=40.0, d.ruck_speed);
    int(
        ui,
        "Abklingzeit ms",
        &mut t.ruck_cooldown,
        0..=5000,
        d.ruck_cooldown,
    );
    ui.label("Stampfen");
    slider(ui, "Tempo", &mut t.stomp_speed, 1.0..=60.0, d.stomp_speed);
    slider(
        ui,
        "Stoßwelle",
        &mut t.stomp_radius,
        0.0..=256.0,
        d.stomp_radius,
    );
    ui.label("Eisgriff");
    int(ui, "Haftdauer ms", &mut t.grip_time, 0..=5000, d.grip_time);
    slider(
        ui,
        "Rutschtempo",
        &mut t.grip_slide_speed,
        0.0..=10.0,
        d.grip_slide_speed,
    );
    slider(
        ui,
        "Wandsprung seitlich",
        &mut t.wall_jump_x,
        0.0..=30.0,
        d.wall_jump_x,
    );
    slider(
        ui,
        "Wandsprung hoch",
        &mut t.wall_jump_y,
        0.0..=30.0,
        d.wall_jump_y,
    );
    ui.label("Gegner (A1.2)");
    slider(
        ui,
        "Heranhooken Zug",
        &mut t.pull_accel,
        0.0..=10.0,
        d.pull_accel,
    );
    int(
        ui,
        "Schutz nach Treffer ms",
        &mut t.hit_invulnerable,
        0..=5000,
        d.hit_invulnerable,
    );
    slider(
        ui,
        "Rückstoß Berührung",
        &mut t.hit_knockback,
        0.0..=30.0,
        d.hit_knockback,
    );
    int(
        ui,
        "Stampf-Schaden",
        &mut t.stomp_damage,
        0..=20,
        d.stomp_damage,
    );
    int(
        ui,
        "Betäubung ms",
        &mut t.stomp_stun,
        0..=5000,
        d.stomp_stun,
    );
    slider(
        ui,
        "Beute-Magnet",
        &mut t.loot_magnet,
        0.0..=400.0,
        d.loot_magnet,
    );
    ui.label("Gleiten");
    slider(
        ui,
        "max. Fallen",
        &mut t.glide_fall_speed,
        0.1..=20.0,
        d.glide_fall_speed,
    );
    slider(
        ui,
        "Luftsteuerung",
        &mut t.glide_control_speed,
        0.5..=20.0,
        d.glide_control_speed,
    );
}

fn air(ui: &mut egui::Ui, t: &mut Tuning, d: &Tuning) {
    slider(
        ui,
        "Doppelsprung",
        &mut t.air_jump_impulse,
        0.0..=30.0,
        d.air_jump_impulse,
    );
    slider(
        ui,
        "Luftgeschw.",
        &mut t.air_control_speed,
        0.5..=20.0,
        d.air_control_speed,
    );
    slider(
        ui,
        "Luftbeschl.",
        &mut t.air_control_accel,
        0.0..=10.0,
        d.air_control_accel,
    );
    slider(
        ui,
        "Luftreibung",
        &mut t.air_friction,
        0.5..=1.0,
        d.air_friction,
    );
    slider(ui, "Gravitation", &mut t.gravity, 0.05..=2.0, d.gravity);
}

fn hook(ui: &mut egui::Ui, t: &mut Tuning, d: &Tuning) {
    slider(
        ui,
        "Länge",
        &mut t.hook_length,
        50.0..=1000.0,
        d.hook_length,
    );
    slider(
        ui,
        "Schussgeschw.",
        &mut t.hook_fire_speed,
        10.0..=200.0,
        d.hook_fire_speed,
    );
    slider(
        ui,
        "Zug-Beschl.",
        &mut t.hook_drag_accel,
        0.1..=10.0,
        d.hook_drag_accel,
    );
    slider(
        ui,
        "Zug-Max.",
        &mut t.hook_drag_speed,
        1.0..=40.0,
        d.hook_drag_speed,
    );
    ui.horizontal(|ui| {
        ui.add(egui::Slider::new(&mut t.player_hook_ticks, 1..=250).text("Spieler halten (Ticks)"));
        reset(ui, &mut t.player_hook_ticks, d.player_hook_ticks);
    });
    slider(
        ui,
        "Spieler-Zugkraft",
        &mut t.player_hook_force,
        0.0..=5.0,
        d.player_hook_force,
    );
}

fn view(ui: &mut egui::Ui, v: &mut ViewSettings) {
    let d = ViewSettings::default();
    section(ui, "Sichtbereich (E-045)", false, |ui| {
        let mut area_k = v.area / 1000.0;
        slider(
            ui,
            "Fläche (Tsd. E²)",
            &mut area_k,
            300.0..=4000.0,
            d.area / 1000.0,
        );
        v.area = area_k * 1000.0;
        slider(
            ui,
            "Max. Breite",
            &mut v.max_width,
            500.0..=4000.0,
            d.max_width,
        );
        slider(
            ui,
            "Max. Höhe",
            &mut v.max_height,
            300.0..=3000.0,
            d.max_height,
        );
    });
}

fn help(ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("Steuerung")
        .default_open(false)
        .show(ui, |ui| {
            ui.label("A / D – laufen");
            ui.label("Leertaste – springen / Doppelsprung");
            ui.label("Rechte Maustaste – Hook (halten)");
            ui.label("R – Respawn · F1 – Panel ein/aus");
            ui.label("Linke Maustaste – schießen · 1/2/3 oder Mausrad – Waffe");
            ui.label("F5 – Aufzeichnung starten/beenden (→ Golden-Test)");
            ui.label("T – Chat · Y – Team-Chat · Tab – Punkte · K – kill");
            ui.label("F3 / F4 – Ja / Nein bei Abstimmungen");
            ui.label("Esc – Maus freigeben · erneut Esc – beenden");
            ui.label("Karte speichern → wird automatisch neu geladen");
        });
}

fn section(ui: &mut egui::Ui, title: &str, open: bool, add: impl FnOnce(&mut egui::Ui)) {
    egui::CollapsingHeader::new(title)
        .default_open(open)
        .show(ui, add);
}

fn slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: RangeInclusive<f32>,
    default: f32,
) {
    ui.horizontal(|ui| {
        ui.add(egui::Slider::new(value, range).text(label).max_decimals(3));
        reset(ui, value, default);
    });
}

fn int<T>(ui: &mut egui::Ui, label: &str, value: &mut T, range: RangeInclusive<T>, default: T)
where
    T: egui::emath::Numeric + std::fmt::Display,
{
    ui.horizontal(|ui| {
        ui.add(egui::Slider::new(value, range).text(label));
        reset(ui, value, default);
    });
}

/// Knopf „Standardwert“; nur aktiv, wenn der Wert abweicht.
fn reset<T: PartialEq + Copy + std::fmt::Display>(ui: &mut egui::Ui, value: &mut T, default: T) {
    let changed = *value != default;
    if ui
        .add_enabled(changed, egui::Button::new("↺").small())
        .on_hover_text(format!("Standard: {default}"))
        .clicked()
    {
        *value = default;
    }
}

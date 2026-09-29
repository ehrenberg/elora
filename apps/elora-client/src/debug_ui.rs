//! Debug-Panel (M1.5, M2.8, M3.9): Tuning-Regler, Zustandsanzeige, Netzwerk.

use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::time::Duration;

use elora_client::online::OnlineClient;
use elora_net::{Conditions, Stats};
use elora_render::ViewSettings;
use elora_sim::{HookState, TICKS_PER_SECOND, TILE_SIZE, Tuning};

use crate::controls::Controls;
use crate::hosting::{Hosting, available_maps};
use crate::sandbox::Sandbox;

/// Aktion, die das Panel ausgelöst hat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Save,
    Load,
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

/// Netzwerk-Einstellungen im Panel.
#[derive(Debug)]
pub struct NetUi {
    pub address: String,
    pub name: String,
    /// Simulator (ausgehende Pakete dieses Clients).
    pub latency_ms: f32,
    pub jitter_ms: f32,
    pub loss_pct: f32,
    pub show_host: bool,
    pub hosting: Hosting,
    pub key_warning: Option<KeyWarning>,
}

impl Default for NetUi {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:8303".into(),
            name: "Elora".into(),
            latency_ms: 0.0,
            jitter_ms: 0.0,
            loss_pct: 0.0,
            show_host: false,
            hosting: Hosting::default(),
            key_warning: None,
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
    pub controls: &'a mut Controls,
    pub fps: f32,
    pub status: &'a str,
    pub cursor_grabbed: bool,
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
                    ui.label("Esc: Maus freigeben, um das Panel zu bedienen");
                } else {
                    ui.label("Ins Spielfeld klicken, um weiterzuspielen");
                }
                ui.separator();
                action = network(ui, cx);
                ui.separator();
                if let Some(sandbox) = cx.sandbox.as_deref_mut() {
                    state(ui, sandbox, cx.fps);
                    ui.separator();
                    action = action.or(buttons(ui, cx.status));
                    ui.separator();
                    tuning(ui, &mut sandbox.world.tuning);
                } else if !cx.status.is_empty() {
                    ui.small(cx.status);
                }
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
        action = action.or(host_window(ui.ctx(), cx.net));
    }
    if let Some(w) = cx.net.key_warning.clone() {
        action = action.or(key_warning_window(ui.ctx(), cx.net, &w));
    }
    action
}

fn network(ui: &mut egui::Ui, cx: &mut Context<'_>) -> Option<Action> {
    let mut action = None;
    egui::CollapsingHeader::new("Netzwerk")
        .default_open(true)
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

fn state(ui: &mut egui::Ui, s: &Sandbox, fps: f32) {
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

fn tuning(ui: &mut egui::Ui, t: &mut Tuning) {
    let d = Tuning::default();
    if ui.button("Alle Werte auf Standard").clicked() {
        *t = d.clone();
    }
    section(ui, "Boden (T-02 bis T-05)", true, |ui| ground(ui, t, &d));
    section(ui, "Luft (T-06 bis T-10)", true, |ui| air(ui, t, &d));
    section(ui, "Hook (T-12 bis T-17)", true, |ui| hook(ui, t, &d));
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
        .default_open(true)
        .show(ui, |ui| {
            ui.label("A / D – laufen");
            ui.label("Leertaste – springen / Doppelsprung");
            ui.label("Rechte Maustaste – Hook (halten)");
            ui.label("R – Respawn · F1 – Panel ein/aus");
            ui.label("Linke Maustaste – schießen · 1/2/3 oder Mausrad – Waffe");
            ui.label("F5 – Aufzeichnung starten/beenden (→ Golden-Test)");
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

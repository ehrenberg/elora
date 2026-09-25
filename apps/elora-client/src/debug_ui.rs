//! Debug-Panel der Sandbox (M1.5): Tuning-Regler, Zustandsanzeige, Speichern.

use std::ops::RangeInclusive;

use elora_render::ViewSettings;
use elora_sim::{HookState, TICKS_PER_SECOND, TILE_SIZE, Tuning};

use crate::controls::Controls;
use crate::sandbox::Sandbox;

/// Aktion, die das Panel ausgelöst hat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Save,
    Load,
    Respawn,
}

/// Zustand, den das Panel anzeigen und ändern darf.
pub struct Context<'a> {
    pub sandbox: &'a mut Sandbox,
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
                ui.heading("Elora – Sandbox");
                if cx.cursor_grabbed {
                    ui.label("Esc: Maus freigeben, um das Panel zu bedienen");
                } else {
                    ui.label("Ins Spielfeld klicken, um weiterzuspielen");
                }
                ui.separator();
                state(ui, cx);
                ui.separator();
                action = buttons(ui, cx.status);
                ui.separator();
                tuning(ui, &mut cx.sandbox.world.tuning);
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
    action
}

fn state(ui: &mut egui::Ui, cx: &Context<'_>) {
    let s = &*cx.sandbox;
    let c = s.character();
    let tps = TICKS_PER_SECOND as f32;
    let tiles_per_s = |v: f32| v * tps / TILE_SIZE as f32;
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
    egui::Grid::new("state")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            let mut row = |k: &str, v: String| {
                ui.label(k);
                ui.monospace(v);
                ui.end_row();
            };
            row("FPS / Tick", format!("{:.0} / {}", cx.fps, s.world.tick));
            row("Position", format!("{:.0}, {:.0}", c.pos.x, c.pos.y));
            row(
                "Geschw. (E/Tick)",
                format!("{:+.2}, {:+.2}", c.vel.x, c.vel.y),
            );
            row(
                "Geschw. (Tiles/s)",
                format!("{:+.1}, {:+.1}", tiles_per_s(c.vel.x), tiles_per_s(c.vel.y)),
            );
            row(
                "Am Boden",
                if c.is_grounded(&s.world.collision) {
                    "ja"
                } else {
                    "nein"
                }
                .into(),
            );
            row(
                "Doppelsprung",
                if c.jumped & 2 == 0 {
                    "verfügbar"
                } else {
                    "verbraucht"
                }
                .into(),
            );
            row("Hook", hook);
            row("Karte", s.map_path.display().to_string());
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

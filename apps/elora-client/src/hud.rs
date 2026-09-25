//! Platzhalter-HUD (M2.8): Leben, Rüstung, Waffen und Munition. Das finale HUD
//! entsteht in M5 als eigene Spiel-UI (E-031).

use elora_sim::{Character, Weapon};

use crate::draw::weapon_color;

fn color(c: elora_render::Color) -> egui::Color32 {
    #[allow(clippy::cast_sign_loss)] // Farbwerte liegen in 0..=1
    let [r, g, b, _] = c.0.map(|v| (v.clamp(0.0, 1.0) * 255.0) as u8);
    egui::Color32::from_rgb(r, g, b)
}

/// Zeichnet das HUD oben links. `max` = maximale Lebenspunkte/Rüstung.
pub fn hud(ui: &mut egui::Ui, character: Option<&Character>, max: i32) {
    egui::Area::new("hud".into())
        .fixed_pos(egui::pos2(16.0, 12.0))
        .interactable(false)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_black_alpha(140))
                .corner_radius(6.0)
                .inner_margin(10.0)
                .show(ui, |ui| match character {
                    Some(ch) => alive(ui, ch, max),
                    None => {
                        ui.label(
                            egui::RichText::new(
                                "Tot – Feuertaste zum Respawn (sonst automatisch nach 3 s)",
                            )
                            .color(egui::Color32::WHITE),
                        );
                    }
                });
        });
}

fn alive(ui: &mut egui::Ui, ch: &Character, max: i32) {
    let pips = |ui: &mut egui::Ui, value: i32, fill: egui::Color32| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 3.0;
            for k in 0..max {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                let c = if k < value {
                    fill
                } else {
                    egui::Color32::from_white_alpha(30)
                };
                ui.painter().rect_filled(rect, 3.0, c);
            }
        });
    };
    pips(ui, ch.health, egui::Color32::from_rgb(224, 90, 122));
    pips(ui, ch.armor, egui::Color32::from_rgb(224, 184, 90));
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        for (n, w) in Weapon::ALL.iter().enumerate() {
            let slot = ch.arsenal.slot(*w);
            let name = match w {
                Weapon::Hammer => "Hammer",
                Weapon::Grenade => "Granate",
                Weapon::Laser => "Laser",
            };
            let ammo = match (slot.got, slot.ammo) {
                (false, _) => "–".to_string(),
                (true, None) => "∞".to_string(),
                (true, Some(a)) => a.to_string(),
            };
            let mut text = egui::RichText::new(format!("{} {name} {ammo}", n + 1));
            text = if !slot.got {
                text.color(egui::Color32::from_white_alpha(60))
            } else if *w == ch.arsenal.active {
                text.color(color(weapon_color(*w))).strong()
            } else {
                text.color(egui::Color32::from_white_alpha(180))
            };
            ui.label(text);
        }
    });
}

//! Skins (M5.4): feste Palette je Teil (E-095, E-096, E-098), Teamfarbe für den
//! Körper in Team-Modi (E-099). Übertragen werden nur die Palettennummern.

use elora_protocol::Skin;
use elora_render::{Color, Tint};
use elora_sim::Team;

use crate::figure::{KEY_BODY, KEY_EYES, KEY_FEET};

/// Körper und Füße (16), freigegeben mit `docs/design/elora-palette.png`.
pub const BODY: [(&str, Color); Skin::BODY_COLORS as usize] = [
    ("Sonne", Color::hex(0xf2c14e)),
    ("Orange", Color::hex(0xf28c3a)),
    ("Koralle", Color::hex(0xe8685a)),
    ("Rot", Color::hex(0xd94a4a)),
    ("Rosa", Color::hex(0xef7fb0)),
    ("Violett", Color::hex(0xa77be0)),
    ("Indigo", Color::hex(0x6a78e0)),
    ("Himmel", Color::hex(0x5aaee8)),
    ("Türkis", Color::hex(0x3fc1b0)),
    ("Mint", Color::hex(0x7fd99a)),
    ("Grün", Color::hex(0x6cbf4a)),
    ("Limette", Color::hex(0xb8d94a)),
    ("Sand", Color::hex(0xe0c89a)),
    ("Braun", Color::hex(0xa8744a)),
    ("Grau", Color::hex(0x9aa4ae)),
    ("Weiß", Color::hex(0xf0ece4)),
];

/// Augen (8).
pub const EYES: [(&str, Color); Skin::EYE_COLORS as usize] = [
    ("Schwarz", Color::hex(0x2b2b2b)),
    ("Nachtblau", Color::hex(0x2e3f86)),
    ("Tannengrün", Color::hex(0x2f6b4a)),
    ("Kastanie", Color::hex(0x6b3a2a)),
    ("Wein", Color::hex(0x7a2a4a)),
    ("Pflaume", Color::hex(0x4a2a7a)),
    ("Petrol", Color::hex(0x1f6470)),
    ("Schiefer", Color::hex(0x5a5a5a)),
];

/// Körperfarbe der Dummies (unabhängig vom Skin, zur Unterscheidung).
const DUMMY: Color = Color::hex(0xb59fd6);

fn body(i: u8) -> Color {
    BODY[usize::from(i) % BODY.len()].1
}

/// Farben einer Figur: Skin, Körper in Teamfarbe (E-099), Dummies eigene Körperfarbe.
pub fn tint(skin: Skin, team: Team, dummy: bool, team_color: impl Fn(Team) -> Color) -> Tint {
    let mut colors = vec![Color::hex(0x2b2b2b); 3];
    colors[KEY_EYES] = EYES[usize::from(skin.eyes) % EYES.len()].1;
    colors[KEY_FEET] = body(skin.feet);
    colors[KEY_BODY] = match team {
        Team::Red | Team::Blue => team_color(team),
        _ if dummy => DUMMY,
        _ => body(skin.body),
    };
    if dummy {
        colors[KEY_FEET] = elora_render::shade(colors[KEY_BODY], -0.12);
    }
    Tint::new(colors)
}

/// Auswahl im Panel; `true` bei Änderung.
pub fn picker(ui: &mut egui::Ui, skin: &mut Skin) -> bool {
    let mut changed = false;
    changed |= row(ui, "Körper", &BODY, &mut skin.body);
    changed |= row(ui, "Füße", &BODY, &mut skin.feet);
    changed |= row(ui, "Augen", &EYES, &mut skin.eyes);
    changed
}

fn row(ui: &mut egui::Ui, label: &str, palette: &[(&str, Color)], value: &mut u8) -> bool {
    let mut changed = false;
    ui.label(label);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
        for (i, (name, color)) in palette.iter().enumerate() {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)] // Farben 0..=1
            let [r, g, b, _] = color.0.map(|v| (v * 255.0).round() as u8);
            let selected = usize::from(*value) == i;
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::click());
            let stroke = if selected {
                egui::Stroke::new(2.5, egui::Color32::WHITE)
            } else {
                egui::Stroke::new(1.0, egui::Color32::from_gray(40))
            };
            ui.painter()
                .rect_filled(rect, 4.0, egui::Color32::from_rgb(r, g, b));
            ui.painter()
                .rect_stroke(rect, 4.0, stroke, egui::StrokeKind::Inside);
            if response.on_hover_text(*name).clicked() && !selected {
                *value = u8::try_from(i).unwrap_or(0);
                changed = true;
            }
        }
    });
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_overrides_body_only() {
        let skin = Skin {
            body: 7,
            feet: 3,
            eyes: 2,
        };
        let red = Color::hex(0xe0574f);
        let t = tint(skin, Team::Red, false, |_| red);
        assert_eq!(t.colors[KEY_BODY], red);
        assert_eq!(t.colors[KEY_FEET], BODY[3].1);
        assert_eq!(t.colors[KEY_EYES], EYES[2].1);
        let t = tint(skin, Team::None, false, |_| red);
        assert_eq!(t.colors[KEY_BODY], BODY[7].1);
    }
}

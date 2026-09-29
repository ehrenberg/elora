//! Pickups, Waffen und Flaggen (M5.5, E-101 Stil A) aus `assets/items/*.svg`.
//!
//! Die Assets sind in Welteinheiten gezeichnet. Waffen haben den Ursprung am Griff
//! und zeigen nach +x; Pickups und Flaggen den Ursprung in der Mitte.

use elora_render::{Affine, Color, Mesh, ShapeBatch, SvgAsset, Tint};
use elora_sim::{PickupKind, Vec2, Weapon};

/// Abstand des Griffs von der Figurenmitte in Zielrichtung.
const GRIP: f32 = 4.0;
/// Schweben der Pickups: Höhe (Einheiten) und Frequenz (Hz).
const BOB_HEIGHT: f32 = 2.5;
const BOB_HZ: f32 = 0.6;

fn load(data: &[u8], name: &str) -> SvgAsset {
    SvgAsset::load(data, 0.08).unwrap_or_else(|e| panic!("assets/items/{name}.svg: {e}"))
}

fn whole(data: &[u8], name: &str) -> Mesh {
    load(data, name)
        .part("")
        .cloned()
        .unwrap_or_else(|| panic!("assets/items/{name}.svg ist leer"))
}

#[derive(Debug)]
pub struct ItemArt {
    health: Mesh,
    armor: Mesh,
    hammer: Mesh,
    grenade: Mesh,
    laser: Mesh,
    flag_pole: Mesh,
    flag_cloth: Mesh,
}

impl ItemArt {
    /// # Panics
    /// Wenn ein eingebettetes Asset fehlerhaft ist (wird von Tests abgedeckt).
    pub fn load() -> Self {
        let flag = load(include_bytes!("../../../assets/items/flag.svg"), "flag");
        let part = |n: &str| {
            flag.part(n)
                .cloned()
                .unwrap_or_else(|| panic!("Teil `{n}` fehlt in flag.svg"))
        };
        Self {
            health: whole(include_bytes!("../../../assets/items/health.svg"), "health"),
            armor: whole(include_bytes!("../../../assets/items/armor.svg"), "armor"),
            hammer: whole(include_bytes!("../../../assets/items/hammer.svg"), "hammer"),
            grenade: whole(
                include_bytes!("../../../assets/items/grenade.svg"),
                "grenade",
            ),
            laser: whole(include_bytes!("../../../assets/items/laser.svg"), "laser"),
            flag_pole: part("pole"),
            flag_cloth: part("cloth"),
        }
    }

    fn weapon_mesh(&self, w: Weapon) -> &Mesh {
        match w {
            Weapon::Hammer => &self.hammer,
            Weapon::Grenade => &self.grenade,
            Weapon::Laser => &self.laser,
        }
    }

    /// Waffe in der Hand: zeigt in Zielrichtung; nach links gespiegelt, damit
    /// Glanzlichter oben bleiben.
    pub fn draw_weapon(&self, batch: &mut ShapeBatch, pos: Vec2, aim: Vec2, w: Weapon) {
        let flip = if aim.x < 0.0 { -1.0 } else { 1.0 };
        let t = Affine::translate(pos + aim * GRIP)
            .then(Affine::rotate(aim.y.atan2(aim.x)))
            .then(Affine::scale(1.0, flip));
        batch.draw_mesh(self.weapon_mesh(w), &t, &Tint::default());
    }

    /// Pickup, schwebend; `time` in Sekunden.
    pub fn draw_pickup(&self, batch: &mut ShapeBatch, pos: Vec2, kind: PickupKind, time: f32) {
        // Phase nach Position, damit nicht alle Pickups im Gleichtakt schweben
        let phase = time * BOB_HZ * std::f32::consts::TAU + pos.x * 0.05;
        let p = pos + Vec2::new(0.0, phase.sin() * BOB_HEIGHT);
        match kind {
            PickupKind::Health => {
                batch.draw_mesh(&self.health, &Affine::translate(p), &Tint::default());
            }
            PickupKind::Armor => {
                batch.draw_mesh(&self.armor, &Affine::translate(p), &Tint::default());
            }
            PickupKind::Weapon(w) => {
                // Waffe mittig über dem Pickup-Punkt
                let len = match w {
                    Weapon::Hammer => 34.0,
                    Weapon::Grenade => 38.0,
                    Weapon::Laser => 40.0,
                };
                let t = Affine::translate(p - Vec2::new(len / 2.0, 0.0));
                batch.draw_mesh(self.weapon_mesh(w), &t, &Tint::default());
            }
        }
    }

    /// Flagge mit wehendem Tuch; `time` in Sekunden.
    pub fn draw_flag(&self, batch: &mut ShapeBatch, pos: Vec2, color: Color, time: f32) {
        batch.draw_mesh(&self.flag_pole, &Affine::translate(pos), &Tint::default());
        // Tuch um die Befestigung (1.5, -26) strecken und scheren
        let hinge = Vec2::new(1.5, -26.0);
        let wave = (time * 2.2 * std::f32::consts::TAU).sin();
        let cloth = Affine::translate(pos + hinge)
            .then(Affine {
                x_axis: Vec2::new(1.0 + 0.06 * wave, 0.08 * wave),
                y_axis: Vec2::new(0.0, 1.0),
                offset: Vec2::default(),
            })
            .then(Affine::translate(-hinge));
        batch.draw_mesh(&self.flag_cloth, &cloth, &Tint::new(vec![color]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assets_load() {
        let art = ItemArt::load();
        for m in [
            &art.health,
            &art.armor,
            &art.hammer,
            &art.grenade,
            &art.laser,
            &art.flag_pole,
            &art.flag_cloth,
        ] {
            assert!(!m.is_empty());
        }
        // Waffen beginnen am Griff
        let (min, max) = art.laser.bounds().unwrap();
        assert!(min.x > -2.0 && max.x > 38.0);
    }
}

//! Pickups, weapons and flags (M5.5, E-101 style A) from `assets/items/*.svg`.
//!
//! The assets are drawn in world units. Weapons have their origin at the grip
//! and point towards +x; pickups and flags have their origin in the center.

use elora_render::{Affine, Color, Mesh, ShapeBatch, SvgAsset, Tint};
use elora_sim::{PickupKind, Vec2, Weapon};

/// Distance of the grip from the figure center in aim direction.
const GRIP: f32 = 4.0;
/// Hovering of the pickups: height (units) and frequency (Hz).
const BOB_HEIGHT: f32 = 2.5;
const BOB_HZ: f32 = 0.6;

fn load(data: &[u8], name: &str) -> SvgAsset {
    SvgAsset::load(data, 0.08).unwrap_or_else(|e| panic!("assets/items/{name}.svg: {e}"))
}

fn whole(data: &[u8], name: &str) -> Mesh {
    load(data, name)
        .part("")
        .cloned()
        .unwrap_or_else(|| panic!("assets/items/{name}.svg is empty"))
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
    /// If an embedded asset is faulty (covered by tests).
    pub fn load() -> Self {
        let flag = load(include_bytes!("../../../assets/items/flag.svg"), "flag");
        let part = |n: &str| {
            flag.part(n)
                .cloned()
                .unwrap_or_else(|| panic!("part `{n}` missing in flag.svg"))
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

    /// Weapon in hand: points in aim direction; mirrored to the left so that
    /// highlights stay on top.
    /// `swing`: additional rotation (rad) for the hammer swing.
    pub fn draw_weapon(&self, batch: &mut ShapeBatch, pos: Vec2, aim: Vec2, swing: f32, w: Weapon) {
        let flip = if aim.x < 0.0 { -1.0 } else { 1.0 };
        let angle = aim.y.atan2(aim.x) + swing;
        let dir = Vec2::new(angle.cos(), angle.sin());
        let t = Affine::translate(pos + dir * GRIP)
            .then(Affine::rotate(angle))
            .then(Affine::scale(1.0, flip));
        batch.draw_mesh(self.weapon_mesh(w), &t, &Tint::default());
    }

    /// Weapon as an icon, centered around `center` (HUD); `alpha` < 1 for weapons not owned.
    pub fn draw_icon(
        &self,
        batch: &mut ShapeBatch,
        center: Vec2,
        w: Weapon,
        scale: f32,
        alpha: f32,
    ) {
        let mesh = self.weapon_mesh(w);
        let Some((min, max)) = mesh.bounds() else {
            return;
        };
        let mid = (min + max) * 0.5;
        let t = Affine::translate(center)
            .then(Affine::scale(scale, scale))
            .then(Affine::translate(-mid));
        let tint = Tint {
            alpha: Some(alpha),
            ..Tint::default()
        };
        batch.draw_mesh(mesh, &t, &tint);
    }

    /// Pickup, hovering; `time` in seconds.
    pub fn draw_pickup(&self, batch: &mut ShapeBatch, pos: Vec2, kind: PickupKind, time: f32) {
        // phase by position so that not all pickups hover in sync
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
                // weapon centered above the pickup point
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

    /// Flag with waving cloth; `time` in seconds.
    pub fn draw_flag(&self, batch: &mut ShapeBatch, pos: Vec2, color: Color, time: f32) {
        batch.draw_mesh(&self.flag_pole, &Affine::translate(pos), &Tint::default());
        // stretch and shear the cloth around the attachment (1.5, -26)
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
        // weapons start at the grip
        let (min, max) = art.laser.bounds().unwrap();
        assert!(min.x > -2.0 && max.x > 38.0);
    }
}

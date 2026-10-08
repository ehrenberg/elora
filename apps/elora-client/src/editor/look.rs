//! Look in the editor (M6.8, E-133): place, move, rotate, scale decoration;
//! background layers with parallax; animations (envelopes); embedded SVGs (E-144).

use std::time::Instant;

use elora_map::look::{Curve, EnvKind, EnvPoint, EnvRef, Envelope};
use elora_map::{Art, Background, Decor, Image, Rgba, Sky};
use elora_sim::{TILE_SIZE, Vec2};

use super::Editor;

/// Where decoration belongs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DecorLayer {
    Back,
    #[default]
    Front,
    Background(usize),
}

/// A decoration object in a layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecorRef {
    pub layer: DecorLayer,
    pub index: usize,
}

/// Presets for sky and background layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Day,
    Night,
}

/// Camera center (y) at the very top of a map: about half a view height (675) below the top edge.
const HIGHEST_CAMERA: f32 = 340.0;
/// Camera at the very bottom: its center is about this high above the ground.
const LOWEST_CAMERA_ABOVE_GROUND: f32 = 300.0;
/// Tall maps: the layers move this far in the image when the camera goes up from the ground …
const TALL_LAYER_TRAVEL: f32 = 150.0;
/// … and their foot lies this far below the image center with the camera at the ground.
const TALL_LAYER_BELOW: f32 = 170.0;

/// Maximum values as in the map format.
const MAX_BACKGROUNDS: usize = 16;
const MAX_ENVELOPES: usize = 256;
const MAX_IMAGES: usize = 64;
const MAX_IMAGE_BYTES: usize = 512 << 10;

/// Is `p` (world) inside the rotated, scaled rectangle `bounds` of an object at `at`?
pub fn hit(d: &Decor, at: Vec2, bounds: (Vec2, Vec2), point: Vec2) -> bool {
    let rel = point - at;
    let (sin, cos) = (-d.rotation.to_radians()).sin_cos();
    let scale = d.scale.abs().max(0.01);
    let mut local = Vec2::new(
        (rel.x * cos - rel.y * sin) / scale,
        (rel.x * sin + rel.y * cos) / scale,
    );
    if d.flip_x {
        local.x = -local.x;
    }
    let (lo, hi) = bounds;
    // some leeway for narrow objects
    let margin = 4.0 / scale;
    local.x >= lo.x - margin
        && local.x <= hi.x + margin
        && local.y >= lo.y - margin
        && local.y <= hi.y + margin
}

/// Corners of the rotated rectangle (for marking the selection).
pub fn corners(d: &Decor, at: Vec2, bounds: (Vec2, Vec2)) -> [Vec2; 4] {
    let (lo, hi) = bounds;
    let (s, c) = d.rotation.to_radians().sin_cos();
    let flip = if d.flip_x { -1.0 } else { 1.0 };
    [
        Vec2::new(lo.x, lo.y),
        Vec2::new(hi.x, lo.y),
        Vec2::new(hi.x, hi.y),
        Vec2::new(lo.x, hi.y),
    ]
    .map(|v| {
        let v = Vec2::new(v.x * flip, v.y) * d.scale;
        at + Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
    })
}

impl Editor {
    pub fn layer_items(&self, layer: DecorLayer) -> Option<&Vec<Decor>> {
        match layer {
            DecorLayer::Back => Some(&self.map.decor_back),
            DecorLayer::Front => Some(&self.map.decor_front),
            DecorLayer::Background(i) => self.map.backgrounds.get(i).map(|b| &b.items),
        }
    }

    fn layer_items_mut(&mut self, layer: DecorLayer) -> Option<&mut Vec<Decor>> {
        match layer {
            DecorLayer::Back => Some(&mut self.map.decor_back),
            DecorLayer::Front => Some(&mut self.map.decor_front),
            DecorLayer::Background(i) => self.map.backgrounds.get_mut(i).map(|b| &mut b.items),
        }
    }

    pub fn decor(&self, r: DecorRef) -> Option<&Decor> {
        self.layer_items(r.layer)?.get(r.index)
    }

    /// Get decoration for editing; `kind` merges quick changes into one step.
    pub fn edit_decor(&mut self, r: DecorRef, kind: &str, now: Instant) -> Option<&mut Decor> {
        self.decor(r)?;
        self.begin_edit(kind, now);
        self.layer_items_mut(r.layer)?.get_mut(r.index)
    }

    /// Offset of a layer in the world at camera center `camera` (parallax).
    pub fn layer_shift(&self, layer: DecorLayer, camera: Vec2) -> Vec2 {
        match layer {
            DecorLayer::Background(i) => self.map.backgrounds.get(i).map_or(Vec2::ZERO, |b| {
                b.offset
                    + Vec2::new(
                        camera.x * (1.0 - b.parallax.x),
                        camera.y * (1.0 - b.parallax.y),
                    )
            }),
            _ => Vec2::ZERO,
        }
    }

    /// Position of an object in the world; for repeated layers the copy near `near`.
    pub fn decor_world_pos(&self, r: DecorRef, camera: Vec2, near: Vec2) -> Option<Vec2> {
        let d = self.decor(r)?;
        let mut at = d.pos + self.layer_shift(r.layer, camera);
        if let DecorLayer::Background(i) = r.layer
            && let Some(step) = self.map.backgrounds[i].repeat_x.filter(|s| *s > 1.0)
        {
            at.x += ((near.x - at.x) / step).round() * step;
        }
        Some(at)
    }

    /// Place a new object at a world point in the target layer and select it.
    pub fn add_decor(
        &mut self,
        layer: DecorLayer,
        art: Art,
        world: Vec2,
        camera: Vec2,
        now: Instant,
    ) -> Option<DecorRef> {
        let pos = world - self.layer_shift(layer, camera);
        self.layer_items(layer)?;
        self.begin_edit("decor-add", now);
        self.end_edit();
        let items = self.layer_items_mut(layer)?;
        items.push(Decor::new(art, pos));
        let r = DecorRef {
            layer,
            index: items.len() - 1,
        };
        self.selected_decor = Some(r);
        Some(r)
    }

    pub fn move_decor(&mut self, r: DecorRef, delta: Vec2, now: Instant) {
        if let Some(d) = self.edit_decor(r, "decor-move", now) {
            d.pos += delta;
        }
    }

    pub fn remove_decor(&mut self, r: DecorRef, now: Instant) {
        if self.decor(r).is_none() {
            return;
        }
        self.begin_edit("decor-remove", now);
        self.end_edit();
        if let Some(items) = self.layer_items_mut(r.layer) {
            items.remove(r.index);
        }
        self.selected_decor = None;
    }

    /// New, empty background layer (frontmost).
    pub fn add_background(&mut self, now: Instant) {
        if self.map.backgrounds.len() >= MAX_BACKGROUNDS {
            return;
        }
        self.begin_edit("bg-add", now);
        self.end_edit();
        let n = self.map.backgrounds.len() + 1;
        self.map.backgrounds.push(Background {
            name: format!("Ebene {n}"),
            parallax: Vec2::new(0.5, 0.5),
            offset: Vec2::ZERO,
            repeat_x: None,
            items: Vec::new(),
        });
        self.selected_bg = Some(self.map.backgrounds.len() - 1);
    }

    pub fn remove_background(&mut self, i: usize, now: Instant) {
        if i >= self.map.backgrounds.len() {
            return;
        }
        self.begin_edit("bg-remove", now);
        self.end_edit();
        self.map.backgrounds.remove(i);
        self.selected_bg = None;
        self.selected_decor = None;
        if let DecorLayer::Background(j) = self.decor_layer
            && j >= i
        {
            self.decor_layer = DecorLayer::Front;
        }
    }

    /// Move a layer one step forward (`forward`) or back.
    pub fn move_background(&mut self, i: usize, forward: bool, now: Instant) {
        let j = if forward { i + 1 } else { i.wrapping_sub(1) };
        if i >= self.map.backgrounds.len() || j >= self.map.backgrounds.len() {
            return;
        }
        self.begin_edit("bg-move", now);
        self.end_edit();
        self.map.backgrounds.swap(i, j);
        self.selected_bg = Some(j);
        self.selected_decor = None;
    }

    /// Sky and background layers from a preset (replaces the existing layers).
    pub fn apply_preset(&mut self, preset: Preset, now: Instant) {
        self.begin_edit("preset", now);
        self.end_edit();
        let ts = TILE_SIZE as f32;
        #[allow(clippy::cast_precision_loss)]
        let ground = (self.map.height as f32 - 2.0).max(1.0) * ts;
        // Layers only grow upwards: for the highest camera (about half a view height below
        // the top edge) they end at the ground; lower down their edge disappears behind the
        // playfield
        let cam_y = HIGHEST_CAMERA.min(ground);
        let deco = |name: &str, x: f32, y: f32, tint: Rgba| {
            let mut d = Decor::new(Art::Builtin(name.into()), Vec2::new(x, y));
            d.tint = tint;
            d
        };
        // Tall maps (adventure): the camera moves far up. So that the layers do not then
        // disappear below the image, they follow the camera more strongly vertically (smaller
        // parallax in y) and lie somewhat below the image center with the camera at the ground.
        let cam_low = ground - LOWEST_CAMERA_ABOVE_GROUND;
        let span = (cam_low - cam_y).max(1.0);
        let layer = |name: &str, p: f32, items: Vec<Decor>| {
            let tall_py = TALL_LAYER_TRAVEL / span;
            let (py, oy) = if tall_py < p {
                (tall_py, TALL_LAYER_BELOW + cam_low * tall_py)
            } else {
                (p, ground - cam_y * (1.0 - p))
            };
            Background {
                name: name.into(),
                parallax: Vec2::new(p, py),
                offset: Vec2::new(0.0, oy),
                repeat_x: Some(1024.0),
                items,
            }
        };
        let night = preset == Preset::Night;
        let dim = |c: u32| if night { Rgba::hex(c) } else { Rgba::WHITE };
        let mut layers = Vec::new();
        if night {
            let mut sky = layer(
                "Sterne",
                0.05,
                vec![deco("stars", 0.0, -200.0, Rgba::WHITE)],
            );
            sky.offset.y = 0.0;
            layers.push(sky);
            let mut moon = layer("Mond", 0.05, vec![deco("moon", 700.0, -150.0, Rgba::WHITE)]);
            moon.offset.y = 0.0;
            moon.repeat_x = None;
            layers.push(moon);
        } else {
            let drift = self.ensure_cloud_drift();
            let mut clouds = vec![
                deco("cloud-1", 120.0, -380.0, Rgba::WHITE),
                deco("cloud-2", 520.0, -430.0, Rgba::WHITE),
                deco("cloud-3", 860.0, -350.0, Rgba::WHITE),
            ];
            for c in &mut clouds {
                c.pos_env = Some(EnvRef {
                    index: drift,
                    offset_ms: 0,
                });
            }
            layers.push(layer("Wolken", 0.1, clouds));
        }
        layers.push(layer(
            "Berge",
            0.2,
            vec![deco("mountains", 0.0, 0.0, dim(0x5c6a9a))],
        ));
        layers.push(layer(
            "Ferne Hügel",
            0.35,
            vec![deco("hills-far", 0.0, 0.0, dim(0x56638f))],
        ));
        layers.push(layer(
            "Wald",
            0.5,
            vec![deco("forest", 0.0, 0.0, dim(0x4a5a80))],
        ));
        layers.push(layer(
            "Nahe Hügel",
            0.7,
            vec![deco("hills-near", 0.0, 0.0, dim(0x46557a))],
        ));
        self.map.backgrounds = layers;
        self.map.sky = if night {
            Sky {
                top: Rgba::hex(0x1f2a55),
                bottom: Rgba::hex(0x4a5a8c),
            }
        } else {
            Sky {
                top: Rgba::hex(0xa9cde8),
                bottom: Rgba::hex(0xe6f2f8),
            }
        };
        self.selected_bg = None;
        self.selected_decor = None;
        self.decor_layer = DecorLayer::Front;
    }

    /// Animation „Wolkenzug“ (bound to server time, 1024 units in 90 s) – existing or new.
    fn ensure_cloud_drift(&mut self) -> u16 {
        const NAME: &str = "Wolkenzug";
        if let Some(i) = self.map.envelopes.iter().position(|e| e.name == NAME) {
            return u16::try_from(i).unwrap_or(0);
        }
        self.map.envelopes.push(Envelope {
            name: NAME.into(),
            kind: EnvKind::Position,
            synced: true,
            points: vec![
                EnvPoint {
                    time_ms: 0,
                    value: [0.0; 4],
                    curve: Curve::Linear,
                },
                EnvPoint {
                    time_ms: 90_000,
                    value: [1024.0, 0.0, 0.0, 0.0],
                    curve: Curve::Linear,
                },
            ],
        });
        u16::try_from(self.map.envelopes.len() - 1).unwrap_or(0)
    }

    /// New animation with two points (1 s), selected.
    pub fn add_envelope(&mut self, kind: EnvKind, now: Instant) {
        if self.map.envelopes.len() >= MAX_ENVELOPES {
            return;
        }
        self.begin_edit("env-add", now);
        self.end_edit();
        let v = kind.neutral();
        let n = self.map.envelopes.len() + 1;
        self.map.envelopes.push(Envelope {
            name: format!("Animation {n}"),
            kind,
            synced: false,
            points: vec![
                EnvPoint {
                    time_ms: 0,
                    value: v,
                    curve: Curve::Smooth,
                },
                EnvPoint {
                    time_ms: 1000,
                    value: v,
                    curve: Curve::Smooth,
                },
            ],
        });
        self.selected_env = Some(self.map.envelopes.len() - 1);
    }

    /// Delete an animation; references to it are dropped, later ones move up.
    pub fn remove_envelope(&mut self, i: usize, now: Instant) {
        if i >= self.map.envelopes.len() {
            return;
        }
        self.begin_edit("env-remove", now);
        self.end_edit();
        self.map.envelopes.remove(i);
        let fix = |r: &mut Option<EnvRef>| {
            if let Some(e) = r {
                match usize::from(e.index).cmp(&i) {
                    std::cmp::Ordering::Equal => *r = None,
                    std::cmp::Ordering::Greater => e.index -= 1,
                    std::cmp::Ordering::Less => {}
                }
            }
        };
        for d in self.all_decor_mut() {
            fix(&mut d.pos_env);
            fix(&mut d.color_env);
        }
        self.selected_env = None;
    }

    fn all_decor_mut(&mut self) -> impl Iterator<Item = &mut Decor> {
        let m = &mut self.map;
        m.decor_back
            .iter_mut()
            .chain(m.decor_front.iter_mut())
            .chain(m.backgrounds.iter_mut().flat_map(|b| b.items.iter_mut()))
    }

    /// Sort points by time and pull equal times apart (the format requires
    /// strictly increasing times).
    pub fn normalize_envelope(&mut self, i: usize) {
        let Some(e) = self.map.envelopes.get_mut(i) else {
            return;
        };
        e.points.sort_by_key(|p| p.time_ms);
        for k in 1..e.points.len() {
            if e.points[k].time_ms <= e.points[k - 1].time_ms {
                e.points[k].time_ms = e.points[k - 1].time_ms + 1;
            }
        }
    }

    /// Embed an SVG (E-144): checked as when loading a foreign map.
    ///
    /// # Errors
    /// With too many or too large images or an invalid SVG: language key and value (E-352).
    pub fn embed_image(
        &mut self,
        name: &str,
        data: Vec<u8>,
        now: Instant,
    ) -> Result<u16, (&'static str, String)> {
        if self.map.images.len() >= MAX_IMAGES {
            return Err(("editor.embed_too_many", MAX_IMAGES.to_string()));
        }
        if data.len() > MAX_IMAGE_BYTES {
            return Err(("editor.embed_too_big", (MAX_IMAGE_BYTES >> 10).to_string()));
        }
        elora_render::SvgAsset::load_untrusted(&data, 0.1, 200_000)
            .map_err(|e| ("editor.embed_failed", e.to_string()))?;
        self.begin_edit("image-add", now);
        self.end_edit();
        self.map.images.push(Image {
            name: name.chars().take(32).collect(),
            svg: data,
        });
        Ok(u16::try_from(self.map.images.len() - 1).unwrap_or(0))
    }

    /// Remove an image along with all decoration using it; later images move up.
    pub fn remove_image(&mut self, i: usize, now: Instant) {
        if i >= self.map.images.len() {
            return;
        }
        self.begin_edit("image-remove", now);
        self.end_edit();
        self.map.images.remove(i);
        let uses = |d: &Decor| matches!(d.art, Art::Image(j) if usize::from(j) == i);
        self.map.decor_back.retain(|d| !uses(d));
        self.map.decor_front.retain(|d| !uses(d));
        for b in &mut self.map.backgrounds {
            b.items.retain(|d| !uses(d));
        }
        for d in self.all_decor_mut() {
            if let Art::Image(j) = &mut d.art
                && usize::from(*j) > i
            {
                *j -= 1;
            }
        }
        if matches!(self.decor_art, Art::Image(_)) {
            self.decor_art = Art::Builtin("bush-1".into());
        }
        self.selected_decor = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn editor() -> Editor {
        Editor::new(None, PathBuf::from("maps"))
    }

    fn roundtrip(e: &Editor) {
        let data = elora_map::encode(&e.map);
        assert_eq!(
            elora_map::decode_draft(&data).unwrap(),
            e.map,
            "map stays valid"
        );
    }

    #[test]
    fn decor_add_move_remove_in_layers() {
        let mut e = editor();
        let t = Instant::now();
        let r = e
            .add_decor(
                DecorLayer::Back,
                Art::Builtin("bush-1".into()),
                Vec2::new(100.0, 200.0),
                Vec2::ZERO,
                t,
            )
            .unwrap();
        assert_eq!(e.map.decor_back.len(), 1);
        e.move_decor(r, Vec2::new(10.0, 0.0), t);
        e.move_decor(r, Vec2::new(10.0, 0.0), t);
        assert_eq!(e.decor(r).unwrap().pos, Vec2::new(120.0, 200.0));
        e.undo();
        assert_eq!(
            e.decor(r).unwrap().pos,
            Vec2::new(100.0, 200.0),
            "moving is one step"
        );
        e.remove_decor(r, t);
        assert!(e.map.decor_back.is_empty());
        roundtrip(&e);
    }

    #[test]
    fn background_items_are_placed_in_layer_space() {
        let mut e = editor();
        let t = Instant::now();
        e.add_background(t);
        e.map.backgrounds[0].parallax = Vec2::new(0.5, 0.5);
        let cam = Vec2::new(400.0, 200.0);
        let r = e
            .add_decor(
                DecorLayer::Background(0),
                Art::Builtin("cloud-1".into()),
                Vec2::new(500.0, 100.0),
                cam,
                t,
            )
            .unwrap();
        // layer moves by camera × (1 − 0.5)
        assert_eq!(e.decor(r).unwrap().pos, Vec2::new(300.0, 0.0));
        assert_eq!(
            e.decor_world_pos(r, cam, Vec2::new(500.0, 100.0)),
            Some(Vec2::new(500.0, 100.0))
        );
        e.map.backgrounds[0].repeat_x = Some(1024.0);
        assert_eq!(
            e.decor_world_pos(r, cam, Vec2::new(1500.0, 100.0)),
            Some(Vec2::new(1524.0, 100.0)),
            "next repetition"
        );
    }

    /// Tall maps (R2-M2.2): the layers stay in the image, no matter how high the camera is.
    #[test]
    fn layers_stay_in_view_on_tall_maps() {
        let mut e = editor();
        e.map = elora_map::Map::new("hoch", 100, 90);
        e.apply_preset(Preset::Day, Instant::now());
        let ground = 88.0 * TILE_SIZE as f32;
        for bg in e.map.backgrounds.iter().filter(|b| b.name != "Wolken") {
            // foot of the layer relative to the camera center, camera at the very bottom and top
            for cam in [ground - LOWEST_CAMERA_ABOVE_GROUND, HIGHEST_CAMERA] {
                let foot = bg.offset.y + cam * (1.0 - bg.parallax.y) - cam;
                assert!(
                    (100.0..=400.0).contains(&foot),
                    "{}: {foot} at {cam}",
                    bg.name
                );
            }
        }
    }

    #[test]
    fn background_order_and_presets() {
        let mut e = editor();
        let t = Instant::now();
        e.add_background(t);
        e.add_background(t);
        e.map.backgrounds[0].name = "A".into();
        e.move_background(0, true, t);
        assert_eq!(e.map.backgrounds[1].name, "A");
        e.move_background(1, true, t);
        assert_eq!(e.map.backgrounds[1].name, "A", "frontmost stays in front");
        e.apply_preset(Preset::Day, t);
        assert_eq!(e.map.backgrounds.len(), 5);
        assert_eq!(e.map.envelopes.len(), 1, "Wolkenzug");
        e.apply_preset(Preset::Day, t);
        assert_eq!(e.map.envelopes.len(), 1, "not duplicated");
        e.apply_preset(Preset::Night, t);
        assert_eq!(e.map.backgrounds[0].name, "Sterne");
        roundtrip(&e);
        e.undo();
        assert_eq!(e.map.backgrounds[0].name, "Wolken");
    }

    #[test]
    fn removing_envelopes_fixes_references() {
        let mut e = editor();
        let t = Instant::now();
        e.add_envelope(EnvKind::Position, t);
        e.add_envelope(EnvKind::Color, t);
        e.add_envelope(EnvKind::Position, t);
        let r = e
            .add_decor(
                DecorLayer::Front,
                Art::Builtin("bush-1".into()),
                Vec2::ZERO,
                Vec2::ZERO,
                t,
            )
            .unwrap();
        let d = e.edit_decor(r, "x", t).unwrap();
        d.pos_env = Some(EnvRef {
            index: 2,
            offset_ms: 5,
        });
        d.color_env = Some(EnvRef {
            index: 1,
            offset_ms: 0,
        });
        e.remove_envelope(1, t);
        let d = e.decor(r).unwrap();
        assert_eq!(d.color_env, None);
        assert_eq!(d.pos_env.map(|x| x.index), Some(1));
        roundtrip(&e);
    }

    #[test]
    fn envelope_points_stay_strictly_increasing() {
        let mut e = editor();
        let t = Instant::now();
        e.add_envelope(EnvKind::Color, t);
        let env = &mut e.map.envelopes[0];
        env.points[1].time_ms = 0;
        env.points.push(env.points[0]);
        e.normalize_envelope(0);
        let times: Vec<u32> = e.map.envelopes[0]
            .points
            .iter()
            .map(|p| p.time_ms)
            .collect();
        assert_eq!(times, vec![0, 1, 2]);
        roundtrip(&e);
    }

    #[test]
    fn images_are_checked_and_removal_cleans_up() {
        let mut e = editor();
        let t = Instant::now();
        assert!(e.embed_image("kaputt", b"<svg".to_vec(), t).is_err());
        assert!(
            e.embed_image("riesig", vec![b' '; MAX_IMAGE_BYTES + 1], t)
                .is_err()
        );
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10" fill="red"/></svg>"#;
        let a = e.embed_image("a", svg.to_vec(), t).unwrap();
        let b = e.embed_image("b", svg.to_vec(), t).unwrap();
        e.add_decor(DecorLayer::Front, Art::Image(a), Vec2::ZERO, Vec2::ZERO, t);
        e.add_decor(DecorLayer::Front, Art::Image(b), Vec2::ZERO, Vec2::ZERO, t);
        e.remove_image(0, t);
        assert_eq!(e.map.images.len(), 1);
        assert_eq!(e.map.decor_front.len(), 1);
        assert_eq!(e.map.decor_front[0].art, Art::Image(0));
        roundtrip(&e);
    }

    #[test]
    fn hit_test_respects_rotation_and_scale() {
        let mut d = Decor::new(Art::Builtin("x".into()), Vec2::ZERO);
        let bounds = (Vec2::new(-10.0, -20.0), Vec2::new(10.0, 0.0));
        assert!(hit(&d, Vec2::ZERO, bounds, Vec2::new(0.0, -10.0)));
        assert!(!hit(&d, Vec2::ZERO, bounds, Vec2::new(30.0, -10.0)));
        d.rotation = 90.0;
        // rotated by 90° the object points right instead of up
        assert!(hit(&d, Vec2::ZERO, bounds, Vec2::new(10.0, 0.0)));
        d.rotation = 0.0;
        d.scale = 3.0;
        assert!(hit(&d, Vec2::ZERO, bounds, Vec2::new(25.0, -50.0)));
        let c = corners(&d, Vec2::ZERO, bounds);
        assert_eq!(c[0], Vec2::new(-30.0, -60.0));
    }
}

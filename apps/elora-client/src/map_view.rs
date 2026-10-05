//! Karte zeichnen (M6.4): Himmel, Hintergrund-Ebenen mit Parallax, Deko hinter und vor der
//! Spielfläche, Spielfläche aus zwischengespeicherten Stücken, Animationen (Envelopes).
//!
//! Reihenfolge: Himmel → Hintergründe (hinten nach vorn) → Deko hinten → Spielfläche →
//! *(Figuren, Items, Geschosse)* → Deko vorn.

use std::collections::HashMap;

use elora_map::look::EnvKind;
use elora_map::{Decor, Map};
use elora_render::{Camera, Mesh, ShapeBatch, SvgAsset};
use elora_sim::{BeltDir, Collision, TILE_SIZE, Tile, Vec2};

use crate::map_art::{self, Anim, MapArt, TileRange};

/// Kantenlänge eines zwischengespeicherten Stücks der Spielfläche in Tiles.
const CHUNK: i32 = 16;
/// Feinheit eingebetteter SVGs und Höchstzahl ihrer Ecken (Schutz vor überladenen Karten).
const IMAGE_TOLERANCE: f32 = 0.1;
const IMAGE_MAX_VERTICES: usize = 200_000;
/// Laufende Pfeile der Beschleuniger: Durchläufe je Sekunde.
const BELT_ARROW_HZ: f32 = 1.5;

/// Zeit für Animationen.
#[derive(Debug, Clone, Copy, Default)]
pub struct LookTime {
    /// Uhr des Clients (ms).
    pub local_ms: i64,
    /// Spielzeit des Servers (ms) – für an den Server gebundene Animationen.
    pub server_ms: i64,
    /// Welke Hook-Blüten aus der Welt ([`elora_sim::Collision::hook_wilt`]).
    pub hook_wilt: Option<bool>,
}

/// Was sich an der Karte ändern muss, damit neu aufgebaut wird.
#[derive(Debug, Clone, PartialEq)]
struct Key {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
    materials: Vec<String>,
    material_map: Vec<u8>,
    /// Name und Größe der eingebetteten Bilder (der Inhalt wird nicht jedes Bild verglichen).
    images: Vec<(String, usize)>,
}

impl Key {
    fn of(map: &Map) -> Self {
        Self {
            width: map.width,
            height: map.height,
            tiles: map.tiles.clone(),
            materials: map.materials.clone(),
            material_map: map.material_map.clone(),
            images: map
                .images
                .iter()
                .map(|i| (i.name.clone(), i.svg.len()))
                .collect(),
        }
    }

    fn matches(&self, map: &Map) -> bool {
        self.width == map.width
            && self.height == map.height
            && self.tiles == map.tiles
            && self.materials == map.materials
            && self.material_map == map.material_map
            && self.images.len() == map.images.len()
            && self
                .images
                .iter()
                .zip(&map.images)
                .all(|((n, l), i)| *n == i.name && *l == i.svg.len())
    }
}

#[derive(Debug)]
struct Cache {
    key: Key,
    col: Collision,
    /// Material je Tile (Index in [`MapArt::materials`]).
    materials: Vec<Option<usize>>,
    chunks: HashMap<(i32, i32), Mesh>,
    /// Eingebettete SVGs als Meshes; ungültige bleiben leer.
    images: Vec<Mesh>,
    belts: Vec<(Vec2, BeltDir)>,
    /// Hook-Blüten: Ecke und Spalte (je Frame gezeichnet, sie welken).
    hook_points: Vec<(Vec2, i32)>,
}

impl Cache {
    fn build(art: &MapArt, map: &Map) -> Self {
        let materials = map
            .tiles
            .iter()
            .enumerate()
            .map(|(i, &t)| {
                let chosen = map
                    .material_map
                    .get(i)
                    .and_then(|&m| usize::from(m).checked_sub(1))
                    .and_then(|m| map.materials.get(m))
                    .map(String::as_str);
                art.material(t, chosen)
            })
            .collect();
        let images = map
            .images
            .iter()
            .map(|img| {
                SvgAsset::load_untrusted(&img.svg, IMAGE_TOLERANCE, IMAGE_MAX_VERTICES).map_or_else(
                    |e| {
                        tracing::warn!(image = %img.name, "eingebettetes Bild übersprungen: {e}");
                        Mesh::default()
                    },
                    |a| merge(&a),
                )
            })
            .collect();
        let ts = TILE_SIZE as f32;
        let belts = map
            .tiles
            .iter()
            .enumerate()
            .filter_map(|(i, t)| match t {
                Tile::Conveyor(d) => {
                    let (x, y) = (i % map.width, i / map.width);
                    #[allow(clippy::cast_precision_loss)]
                    Some((Vec2::new(x as f32 * ts, y as f32 * ts), *d))
                }
                _ => None,
            })
            .collect();
        Self {
            key: Key::of(map),
            col: map.collision(),
            materials,
            chunks: HashMap::new(),
            images,
            belts,
            hook_points: map
                .tiles
                .iter()
                .enumerate()
                .filter(|(_, t)| **t == Tile::HookPoint)
                .map(|(i, _)| {
                    let (x, y) = (i % map.width, i / map.width);
                    #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
                    (Vec2::new(x as f32 * ts, y as f32 * ts), x as i32)
                })
                .collect(),
        }
    }

    fn chunk(&mut self, art: &MapArt, cx: i32, cy: i32) -> &Mesh {
        let Self {
            chunks,
            col,
            materials,
            ..
        } = self;
        chunks.entry((cx, cy)).or_insert_with(|| {
            let mut batch = ShapeBatch::default();
            batch.tolerance = 0.1;
            let w = col.width();
            let material = |x: i32, y: i32| {
                let (x, y) = (usize::try_from(x).ok()?, usize::try_from(y).ok()?);
                if x >= w {
                    return None;
                }
                materials.get(y * w + x).copied().flatten()
            };
            let range = TileRange {
                x0: cx * CHUNK,
                y0: cy * CHUNK,
                x1: cx * CHUNK + CHUNK - 1,
                y1: cy * CHUNK + CHUNK - 1,
            };
            map_art::draw_terrain(&mut batch, art, col, &material, range);
            batch.to_mesh()
        })
    }
}

/// Alle Teile eines SVG-Assets als ein Mesh.
fn merge(asset: &SvgAsset) -> Mesh {
    let mut batch = ShapeBatch::default();
    for (_, m) in &asset.parts {
        batch.draw_mesh(
            m,
            &elora_render::Affine::IDENTITY,
            &elora_render::Tint::default(),
        );
    }
    batch.to_mesh()
}

/// Welche Ebenen gezeichnet werden (der Editor blendet einzelne aus).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Layers {
    /// Himmel über den ganzen Ausschnitt (der Editor zeichnet ihn nur innerhalb der Karte).
    pub sky: bool,
    pub backgrounds: bool,
    pub decor_back: bool,
    pub terrain: bool,
    pub decor_front: bool,
}

impl Layers {
    pub const ALL: Self = Self {
        sky: true,
        backgrounds: true,
        decor_back: true,
        terrain: true,
        decor_front: true,
    };
}

/// Karte, Zwischenspeicher und Zeit für einen Frame.
#[derive(Debug)]
pub struct MapLayer<'a> {
    pub map: Option<&'a Map>,
    pub view: &'a mut MapView,
    pub time: LookTime,
}

/// Kartengrafik mit Zwischenspeicher.
#[derive(Debug)]
pub struct MapView {
    pub art: MapArt,
    cache: Option<Cache>,
}

impl Default for MapView {
    fn default() -> Self {
        Self {
            art: MapArt::load(),
            cache: None,
        }
    }
}

/// Wirkung der Animationen eines Deko-Objekts zur Zeit `time`.
fn anim(map: &Map, d: &Decor, time: LookTime) -> Anim {
    let mut a = Anim::default();
    let at = |r: elora_map::look::EnvRef, kind: EnvKind| {
        let e = map
            .envelopes
            .get(usize::from(r.index))
            .filter(|e| e.kind == kind)?;
        let base = if e.synced {
            time.server_ms
        } else {
            time.local_ms
        };
        Some(e.eval(base + i64::from(r.offset_ms)))
    };
    if let Some(v) = d.pos_env.and_then(|r| at(r, EnvKind::Position)) {
        a.offset = Vec2::new(v[0], v[1]);
        a.rotation = v[2];
    }
    if let Some(v) = d.color_env.and_then(|r| at(r, EnvKind::Color)) {
        a.color = v.map(|c| c.clamp(0.0, 1.0));
    }
    a
}

/// Sichtbarer Bereich (Welt) mit etwas Rand.
fn visible(camera: &Camera, margin: f32) -> (Vec2, Vec2) {
    let tl = camera.top_left() - Vec2::new(margin, margin);
    (tl, tl + camera.size + Vec2::new(2.0 * margin, 2.0 * margin))
}

impl MapView {
    /// Zwischenspeicher zur Karte passend machen (neu aufbauen, wenn sie sich geändert hat).
    fn sync(&mut self, map: &Map) -> &mut Cache {
        if !self.cache.as_ref().is_some_and(|c| c.key.matches(map)) {
            self.cache = Some(Cache::build(&self.art, map));
        }
        self.cache.as_mut().expect("eben gesetzt")
    }

    /// Alles hinter den Figuren: Himmel, Hintergründe, Deko hinten, Spielfläche.
    pub fn draw_back(
        &mut self,
        batch: &mut ShapeBatch,
        map: &Map,
        camera: &Camera,
        time: LookTime,
    ) {
        self.draw_back_layers(batch, map, camera, time, Layers::ALL);
    }

    /// Wie [`MapView::draw_back`], nur die eingeschalteten Ebenen.
    pub fn draw_back_layers(
        &mut self,
        batch: &mut ShapeBatch,
        map: &Map,
        camera: &Camera,
        time: LookTime,
        layers: Layers,
    ) {
        if layers.sky {
            let tl = camera.top_left();
            batch.fill_rect_vgradient(
                tl,
                tl + camera.size,
                map_art::rgba(map.sky.top),
                map_art::rgba(map.sky.bottom),
            );
        }
        self.sync(map);
        if layers.backgrounds {
            self.backgrounds(batch, map, camera, time);
        }
        if layers.decor_back {
            self.decor(batch, map, &map.decor_back, camera, time);
        }
        if layers.terrain {
            self.terrain(batch, camera, time);
        }
    }

    /// Umschließendes Rechteck der Grafik eines Deko-Objekts (lokal, ohne Lage und Drehung).
    pub fn decor_bounds(&mut self, map: &Map, d: &Decor) -> Option<(Vec2, Vec2)> {
        self.sync(map);
        let cache = self.cache.as_ref()?;
        map_art::decor_mesh(&self.art, &cache.images, d)?.bounds()
    }

    /// Deko vor den Figuren.
    pub fn draw_front(
        &mut self,
        batch: &mut ShapeBatch,
        map: &Map,
        camera: &Camera,
        time: LookTime,
    ) {
        self.sync(map);
        self.decor(batch, map, &map.decor_front, camera, time);
    }

    fn terrain(&mut self, batch: &mut ShapeBatch, camera: &Camera, time: LookTime) {
        let Some(cache) = self.cache.as_mut() else {
            return;
        };
        let span = CHUNK as f32 * TILE_SIZE as f32;
        let (min, max) = visible(camera, TILE_SIZE as f32);
        #[allow(clippy::cast_possible_truncation)]
        let (cx0, cy0, cx1, cy1) = (
            (min.x / span).floor().max(0.0) as i32,
            (min.y / span).floor().max(0.0) as i32,
            (max.x / span).floor() as i32,
            (max.y / span).floor() as i32,
        );
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        let (cw, ch) = (
            (cache.col.width() as i32 + CHUNK - 1) / CHUNK,
            (cache.col.height() as i32 + CHUNK - 1) / CHUNK,
        );
        for cy in cy0..=cy1.min(ch - 1) {
            for cx in cx0..=cx1.min(cw - 1) {
                let mesh = cache.chunk(&self.art, cx, cy);
                batch.draw_mesh(
                    mesh,
                    &elora_render::Affine::IDENTITY,
                    &elora_render::Tint::default(),
                );
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let phase = (time.local_ms as f32 / 1000.0 * BELT_ARROW_HZ).fract();
        for &(pos, dir) in &cache.belts {
            if pos.x + TILE_SIZE as f32 >= min.x
                && pos.x <= max.x
                && pos.y + TILE_SIZE as f32 >= min.y
                && pos.y <= max.y
            {
                map_art::draw_belt_arrow(batch, &self.art, pos, dir, phase);
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let secs = time.local_ms as f32 / 1000.0;
        for &(pos, tx) in &cache.hook_points {
            if pos.x + TILE_SIZE as f32 >= min.x
                && pos.x <= max.x
                && pos.y + TILE_SIZE as f32 >= min.y
                && pos.y <= max.y
            {
                let active = time
                    .hook_wilt
                    .is_none_or(|even| (tx.rem_euclid(2) == 0) != even);
                map_art::draw_hook_point(batch, &self.art, pos, active, secs);
            }
        }
    }

    fn decor(
        &self,
        batch: &mut ShapeBatch,
        map: &Map,
        items: &[Decor],
        camera: &Camera,
        time: LookTime,
    ) {
        let Some(cache) = self.cache.as_ref() else {
            return;
        };
        let (min, max) = visible(camera, 0.0);
        for d in items {
            let Some(mesh) = map_art::decor_mesh(&self.art, &cache.images, d) else {
                continue;
            };
            // Ausdehnung der Grafik (große Bäume ragen weit über ihren Fuß hinaus)
            let reach = mesh
                .bounds()
                .map_or(400.0, |(lo, hi)| lo.length().max(hi.length()))
                * d.scale.abs()
                + 64.0;
            if d.pos.x + reach < min.x
                || d.pos.x - reach > max.x
                || d.pos.y + reach < min.y
                || d.pos.y - reach > max.y
            {
                continue;
            }
            map_art::draw_decor(batch, mesh, d, d.pos, &anim(map, d, time));
        }
    }

    /// Hintergrund-Ebenen: Lage = Position + Versatz + Kamera × (1 − Parallax);
    /// wiederholte Ebenen werden waagerecht über den ganzen Ausschnitt gelegt.
    fn backgrounds(&self, batch: &mut ShapeBatch, map: &Map, camera: &Camera, time: LookTime) {
        let Some(cache) = self.cache.as_ref() else {
            return;
        };
        let (min, max) = visible(camera, 0.0);
        for bg in &map.backgrounds {
            let shift = bg.offset
                + Vec2::new(
                    camera.center.x * (1.0 - bg.parallax.x),
                    camera.center.y * (1.0 - bg.parallax.y),
                );
            for d in &bg.items {
                let Some(mesh) = map_art::decor_mesh(&self.art, &cache.images, d) else {
                    continue;
                };
                let Some((lo, hi)) = mesh.bounds() else {
                    continue;
                };
                let reach = (lo.length().max(hi.length())) * d.scale.abs() + 50.0;
                let base = d.pos + shift;
                if base.y + reach < min.y || base.y - reach > max.y {
                    continue;
                }
                let a = anim(map, d, time);
                match bg.repeat_x {
                    Some(step) if step > 1.0 => {
                        let k0 = ((min.x - reach - base.x) / step).floor();
                        let k1 = ((max.x + reach - base.x) / step).ceil();
                        let mut k = k0;
                        while k <= k1 {
                            map_art::draw_decor(
                                batch,
                                mesh,
                                d,
                                base + Vec2::new(k * step, 0.0),
                                &a,
                            );
                            k += 1.0;
                        }
                    }
                    _ => {
                        if base.x + reach >= min.x && base.x - reach <= max.x {
                            map_art::draw_decor(batch, mesh, d, base, &a);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_map::look::{Curve, EnvPoint, EnvRef, Envelope};
    use elora_map::{Art, Background, Rgba};

    /// Zeilen der Vorführkarte; `e`/`s`/`n` sind feste Tiles aus Erde, Sand, Schnee.
    const LOOK_ROWS: &[&str] = &[
        r"%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%",
        r"%..............................................................%",
        r"%..............................................................%",
        r"%..............................................................%",
        r"%..............................................................%",
        r"%..............................................................%",
        r"%.....................................a........................%",
        r"%...............h...................eeeeee.....................%",
        r"%.............eeeeee................eeeeee.....................%",
        r"%.............eeeeee..............................%%%%.........%",
        r"%.eee..........................................................%",
        r"%.eee.........................S................................%",
        r"%.eee...................========...............G...............%",
        r"%..........L................................======.............%",
        r"%.......======.................................................%",
        r"%..............................................................%",
        r"%.......................................................~~~~~~.%",
        r"%..............................................................%",
        r"%..S................h.......................................S..%",
        r"%eee!!eeeeeeeee/eeeeesssssss^^^^s>>>>>sssnnnn\nnnnnn<<<<<nnnnnn%",
        r"%eeeeeeeeeeeeeeeeeeeessssssssssssssssssssnnnnnnnnnnnnnnnnnnnnnn%",
        r"%eeeeeeeeeeeeeeeeeeeessssssssssssssssssssnnnnnnnnnnnnnnnnnnnnnn%",
        r"%eeeeeeeeeeeeeeeeeeeessssssssssssssssssssnnnnnnnnnnnnnnnnnnnnnn%",
        r"%eeeeeeeeeeeeeeeeeeeessssssssssssssssssssnnnnnnnnnnnnnnnnnnnnnn%",
    ];

    fn decor(name: &str, x: f32, y: f32) -> Decor {
        Decor::new(Art::Builtin(name.into()), Vec2::new(x, y))
    }

    fn env(name: &str, kind: EnvKind, synced: bool, points: &[(u32, [f32; 4], Curve)]) -> Envelope {
        Envelope {
            name: name.into(),
            kind,
            synced,
            points: points
                .iter()
                .map(|&(time_ms, value, curve)| EnvPoint {
                    time_ms,
                    value,
                    curve,
                })
                .collect(),
        }
    }

    /// Vorführkarte für den Kartenlook (M6.4): Materialien, Spezial-Tiles, Hintergründe mit
    /// Parallax, ziehende Wolken, Bäume im Wind, Deko vor und hinter der Spielfläche.
    #[allow(clippy::too_many_lines)]
    pub fn look_test_map() -> Map {
        let plain: Vec<String> = LOOK_ROWS
            .iter()
            .map(|r| r.replace(['e', 's', 'n'], "#"))
            .collect();
        let refs: Vec<&str> = plain.iter().map(String::as_str).collect();
        let mut m = Map::from_rows("Look-Test", &refs).expect("gültig");
        m.author = Some("Elora-Team".into());
        m.materials = vec!["earth".into(), "sand".into(), "snow".into()];
        m.material_map = LOOK_ROWS
            .iter()
            .flat_map(|r| r.chars())
            .map(|c| match c {
                'e' => 1,
                's' => 2,
                'n' => 3,
                _ => 0,
            })
            .collect();
        m.sky = elora_map::Sky {
            top: Rgba::hex(0xa9cde8),
            bottom: Rgba::hex(0xe6f2f8),
        };
        let wind = 0u16;
        let clouds = 1u16;
        let glow = 2u16;
        m.envelopes = vec![
            env(
                "Wind",
                EnvKind::Position,
                false,
                &[
                    (0, [0.0, 0.0, -2.0, 0.0], Curve::Smooth),
                    (1800, [0.0, 0.0, 2.0, 0.0], Curve::Smooth),
                    (3600, [0.0, 0.0, -2.0, 0.0], Curve::Smooth),
                ],
            ),
            env(
                "Wolkenzug",
                EnvKind::Position,
                true,
                &[
                    (0, [0.0; 4], Curve::Linear),
                    (90_000, [1024.0, 0.0, 0.0, 0.0], Curve::Linear),
                ],
            ),
            env(
                "Leuchten",
                EnvKind::Color,
                false,
                &[
                    (0, [1.0, 1.0, 1.0, 1.0], Curve::Smooth),
                    (1200, [1.0, 0.85, 0.85, 0.75], Curve::Smooth),
                    (2400, [1.0, 1.0, 1.0, 1.0], Curve::Smooth),
                ],
            ),
        ];
        // Kamera steht meist um y ≈ 450; Ebenen sollen dann knapp über dem Boden (y = 608) enden
        let layer = |name: &str, p: f32, repeat: f32, items: Vec<Decor>| Background {
            name: name.into(),
            parallax: Vec2::new(p, p),
            offset: Vec2::new(0.0, 608.0 - 450.0 * (1.0 - p)),
            repeat_x: Some(repeat),
            items,
        };
        let mut cloud_items = vec![
            decor("cloud-1", 120.0, -380.0),
            decor("cloud-2", 520.0, -430.0),
            decor("cloud-3", 860.0, -350.0),
        ];
        for c in &mut cloud_items {
            c.pos_env = Some(EnvRef {
                index: clouds,
                offset_ms: 0,
            });
        }
        m.backgrounds = vec![
            layer("Wolken", 0.1, 1024.0, cloud_items),
            layer("Berge", 0.2, 1024.0, vec![decor("mountains", 0.0, 0.0)]),
            layer(
                "Ferne Hügel",
                0.35,
                1024.0,
                vec![decor("hills-far", 0.0, 0.0)],
            ),
            layer("Wald", 0.5, 1024.0, vec![decor("forest", 0.0, 0.0)]),
            layer(
                "Nahe Hügel",
                0.7,
                1024.0,
                vec![decor("hills-near", 0.0, 0.0)],
            ),
        ];
        let floor = 19.0 * 32.0;
        let sway = |mut d: Decor, phase: i32| {
            d.pos_env = Some(EnvRef {
                index: wind,
                offset_ms: phase,
            });
            d
        };
        m.decor_back = vec![
            sway(decor("tree-round", 9.0 * 32.0, floor), 0),
            sway(decor("tree-round", 23.0 * 32.0, floor), 700),
            sway(decor("tree-pine", 47.0 * 32.0, floor), 300),
            sway(decor("tree-pine", 58.0 * 32.0, floor), 1200),
            decor("bush-2", 12.5 * 32.0, floor),
            decor("fence", 18.0 * 32.0, floor),
            decor("sign-arrow", 26.5 * 32.0, floor),
            decor("rock-2", 39.0 * 32.0, floor),
            decor("bush-1", 16.0 * 32.0, 8.0 * 32.0),
            decor("mushroom-brown", 38.0 * 32.0, 7.0 * 32.0),
            decor("sign-board", 61.0 * 32.0, floor),
        ];
        let mut glowing = decor("mushroom-red", 41.0 * 32.0, 7.0 * 32.0);
        glowing.color_env = Some(EnvRef {
            index: glow,
            offset_ms: 0,
        });
        m.decor_front = vec![
            decor("grass-1", 2.0 * 32.0, floor),
            decor("flower-pink", 7.0 * 32.0, floor),
            decor("grass-2", 13.0 * 32.0, floor),
            decor("flower-yellow", 19.0 * 32.0, floor),
            decor("grass-1", 22.0 * 32.0, floor),
            decor("flower-blue", 36.0 * 32.0, floor),
            decor("rock-1", 43.0 * 32.0, floor),
            decor("grass-2", 55.0 * 32.0, floor),
            decor("flower-pink", 37.0 * 32.0, 7.0 * 32.0),
            glowing,
        ];
        m
    }

    /// Schreibt `maps/look-test.emap`: `cargo test -p elora-client --bin elora write_look_test_map -- --ignored`.
    #[test]
    #[ignore = "erzeugt die Vorführkarte"]
    fn write_look_test_map() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../maps/look-test.emap");
        look_test_map().save(std::path::Path::new(path)).unwrap();
    }

    /// Ausschnitt der Vorführkarte wie im Spiel: `cargo test -p elora-client --bin elora look_sheet -- --ignored`,
    /// danach `cargo xtask svg-preview target/look.svg target/look.png 1400`.
    #[test]
    #[ignore = "erzeugt nur eine Datei zur Sichtprüfung"]
    fn look_sheet() {
        let map = look_test_map();
        let mut view = MapView::default();
        let mut batch = ShapeBatch::default();
        batch.tolerance = 0.1;
        let cam = Camera {
            center: Vec2::new(1024.0, 400.0),
            size: Vec2::new(2048.0, 800.0),
        };
        let t = LookTime {
            local_ms: 900,
            server_ms: 20_000,
            hook_wilt: None,
        };
        view.draw_back(&mut batch, &map, &cam, t);
        view.draw_front(&mut batch, &map, &cam, t);
        let tl = cam.top_left();
        let svg = batch.debug_svg(tl, tl + cam.size, elora_render::Color::hex(0x8fb8d9));
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/look.svg"),
            svg,
        )
        .unwrap();
    }

    fn camera(center: Vec2) -> Camera {
        Camera {
            center,
            size: Vec2::new(1200.0, 675.0),
        }
    }

    #[test]
    fn look_test_map_draws_and_caches() {
        let map = look_test_map();
        let data = elora_map::encode(&map);
        assert_eq!(elora_map::decode(&data).unwrap(), map);
        let mut view = MapView::default();
        let mut batch = ShapeBatch::default();
        let cam = camera(Vec2::new(800.0, 450.0));
        view.draw_back(&mut batch, &map, &cam, LookTime::default());
        view.draw_front(&mut batch, &map, &cam, LookTime::default());
        let built = view.cache.as_ref().unwrap().chunks.len();
        assert!(built > 0 && built <= 8, "nur sichtbare Stücke: {built}");
        assert!(batch.triangle_count() > 1000);
        // gleiche Karte → kein Neuaufbau; geänderte Kollision → neu
        view.draw_back(&mut batch, &map, &cam, LookTime::default());
        assert_eq!(view.cache.as_ref().unwrap().chunks.len(), built);
        let mut changed = map.clone();
        changed.tiles[100] = Tile::Solid;
        view.draw_back(&mut batch, &changed, &cam, LookTime::default());
        assert!(view.cache.as_ref().unwrap().key.matches(&changed));
    }

    #[test]
    fn envelopes_move_and_tint_decor() {
        let map = look_test_map();
        let tree = &map.decor_back[0];
        let a = anim(&map, tree, LookTime::default());
        let b = anim(
            &map,
            tree,
            LookTime {
                local_ms: 1800,
                server_ms: 0,
                hook_wilt: None,
            },
        );
        assert!((a.rotation + 2.0).abs() < 1e-4 && (b.rotation - 2.0).abs() < 1e-4);
        let cloud = &map.backgrounds[0].items[0];
        let c = anim(
            &map,
            cloud,
            LookTime {
                local_ms: 0,
                server_ms: 45_000,
                hook_wilt: None,
            },
        );
        assert!(
            (c.offset.x - 512.0).abs() < 1e-2,
            "Wolken an Server-Zeit gebunden"
        );
        let glow = map.decor_front.last().unwrap();
        let g = anim(
            &map,
            glow,
            LookTime {
                local_ms: 1200,
                server_ms: 0,
                hook_wilt: None,
            },
        );
        assert!((g.color[3] - 0.75).abs() < 1e-4);
    }

    #[test]
    fn broken_embedded_image_is_skipped() {
        let mut map = look_test_map();
        map.images.push(elora_map::Image {
            name: "kaputt".into(),
            svg: b"<svg".to_vec(),
        });
        map.decor_front
            .push(Decor::new(Art::Image(0), Vec2::new(100.0, 500.0)));
        let mut view = MapView::default();
        let mut batch = ShapeBatch::default();
        view.draw_front(
            &mut batch,
            &map,
            &camera(Vec2::new(200.0, 450.0)),
            LookTime::default(),
        );
        assert!(view.cache.as_ref().unwrap().images[0].is_empty());
    }

    /// Aufbauzeit und Größe der Spielfläche für eine große Karte:
    /// `cargo test --release -p elora-client --bin elora map_view_benchmark -- --ignored --nocapture`.
    #[test]
    #[ignore = "Messung"]
    fn map_view_benchmark() {
        let (w, h) = (400usize, 200usize);
        let mut rows = vec![String::new(); h];
        for (y, row) in rows.iter_mut().enumerate() {
            *row = (0..w)
                .map(|x| {
                    let n = (x * 7 + y * 13 + (x / 5) * (y / 3)) % 11;
                    if x == 5 && y == 5 {
                        'S'
                    } else if y == 0 || y == h - 1 || x == 0 || x == w - 1 || n < 4 {
                        '#'
                    } else if n == 4 {
                        '%'
                    } else {
                        '.'
                    }
                })
                .collect();
        }
        let refs: Vec<&str> = rows.iter().map(String::as_str).collect();
        let map = Map::from_rows("Messung", &refs).unwrap();
        let mut view = MapView::default();
        let start = std::time::Instant::now();
        view.sync(&map);
        let cache = view.cache.as_mut().unwrap();
        let mut triangles = 0;
        for cy in 0..=(200 / CHUNK) {
            for cx in 0..=(400 / CHUNK) {
                triangles += cache.chunk(&view.art, cx, cy).triangle_count();
            }
        }
        let build = start.elapsed();
        let mut batch = ShapeBatch::default();
        let cam = camera(Vec2::new(3000.0, 3000.0));
        let start = std::time::Instant::now();
        for _ in 0..100 {
            batch.clear();
            view.draw_back(&mut batch, &map, &cam, LookTime::default());
        }
        let frame = start.elapsed() / 100;
        println!(
            "{w}×{h} Tiles: Aufbau {build:?}, {triangles} Dreiecke gesamt; je Frame {frame:?}, {} Dreiecke sichtbar",
            batch.triangle_count()
        );
    }
}

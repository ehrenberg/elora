//! Kartengrafik im Stil A (M6.3, E-139): Materialien mit automatischen Kanten, Spezial-Tiles,
//! Deko und Hintergründe aus `assets/map/`.
//!
//! Regeln für die Spielfläche (Auto-Kanten):
//! - **Form:** Feste Tiles (fest, nicht hookbar, Eis) bilden zusammen mit Sprungfeldern und
//!   Beschleunigern eine Fläche. Außenecken, an denen beide Nachbarn frei sind, werden gerundet.
//! - **Kontur** nur an freien Kanten; zwischen Materialien keine Kontur.
//! - **Kappe** (Gras, Schnee …) an jeder freien Oberkante; an freien Seiten das passende Endstück.
//! - **Details** verstreut im Inneren, fest je Tile (gleiche Karte → gleiches Bild).

use std::collections::BTreeMap;

use elora_map::{Art, Decor, Rgba};
use elora_render::{Affine, Color, Mesh, Path, ShapeBatch, SvgAsset, Tint};
use elora_sim::{BeltDir, Collision, JumpDir, TILE_SIZE, Tile, Vec2};
use lyon::math::point;
use serde::Deserialize;

/// Kontur der Spielfläche (wie Figur und Items).
const OUTLINE: Color = Color::hex(0x2b2b2b);
const OUTLINE_WIDTH: f32 = 2.5;
/// Überlappung der Körper benachbarter Tiles gegen Haarlinien.
const SEAM: f32 = 0.4;
/// Feinheit der Tessellierung für Kartengrafik.
const TOLERANCE: f32 = 0.1;

macro_rules! assets {
    ($dir:literal: $($name:literal),* $(,)?) => {
        &[$(($name, include_bytes!(concat!("../../../assets/map/", $dir, "/", $name, ".svg")))),*]
    };
}

const MATERIAL_FILES: &[(&str, &[u8])] =
    assets!("materials": "earth", "sand", "snow", "stone", "ice", "climb", "crumble");
const MATERIALS_TOML: &str = include_str!("../../../assets/map/materials.toml");

/// Eingebaute Deko (Namen für [`Art::Builtin`]).
pub const DECOR_FILES: &[(&str, &[u8])] = assets!("decor":
    "bush-1", "bush-2", "flower-pink", "flower-yellow", "flower-blue", "grass-1", "grass-2",
    "rock-1", "rock-2", "mushroom-red", "mushroom-brown", "tree-round", "tree-pine", "fence",
    "sign-arrow", "sign-board",
    // Tauwinkel (A1.9, E-278); Blumen, Beete, Fahnen verblasst und farbig (E-277)
    "haus-elora", "haus-oma", "brunnen", "werkstatt", "schmiede", "laden", "baumhaus",
    "anschlagbrett", "wegweiser", "blumenkasten-blass", "blumenkasten-bunt", "beet-blass",
    "beet-bunt", "kraeuterbeet-blass", "kraeuterbeet-bunt", "fahne-blass", "fahne-bunt",
    // Requisiten und Dornen (Playtest A1.9, E-283)
    "dornen", "bank", "laterne", "faesser", "holzstapel", "karren", "waescheleine", "heuballen",
    "mauer",
);

/// Eingebaute Hintergrund-Grafik (ebenfalls über [`Art::Builtin`] benutzt).
pub const BACKGROUND_FILES: &[(&str, &[u8])] = assets!("backgrounds":
    "cloud-1", "cloud-2", "cloud-3", "hills-far", "hills-near", "mountains", "forest", "stars",
    "moon",
);

/// Breite der wiederholbaren Hintergrund-Streifen.
#[allow(dead_code)] // für Tests und den Editor (M6.8)
pub const STRIP_WIDTH: f32 = 1024.0;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialDef {
    body: String,
    radius: f32,
    details: f32,
    #[serde(rename = "for")]
    tiles: Vec<String>,
}

fn parse_hex(s: &str) -> Option<Color> {
    let v = u32::from_str_radix(s.strip_prefix('#')?, 16).ok()?;
    (s.len() == 7).then(|| Color::hex(v))
}

/// Ein Material der Spielfläche.
#[derive(Debug)]
pub struct Material {
    pub name: String,
    body: Color,
    radius: f32,
    detail_chance: f32,
    /// Tile-Arten, für die es wählbar ist.
    pub tiles: Vec<Tile>,
    /// Mitte, links frei, rechts frei, beide frei.
    caps: Option<[Mesh; 4]>,
    details: Vec<Mesh>,
}

/// Spezial-Tiles.
#[derive(Debug)]
struct Specials {
    spikes: Mesh,
    /// Mitte, links, rechts, einzeln.
    planks: [Mesh; 4],
    jump_up: Mesh,
    jump_diag: Mesh,
    belt: Mesh,
    belt_arrow: Mesh,
}

/// Alle eingebauten Kartengrafiken.
#[derive(Debug)]
pub struct MapArt {
    pub materials: Vec<Material>,
    specials: Specials,
    builtin: BTreeMap<&'static str, Mesh>,
}

fn svg(data: &[u8], file: &str) -> SvgAsset {
    SvgAsset::load(data, TOLERANCE).unwrap_or_else(|e| panic!("assets/map/{file}: {e}"))
}

fn part(asset: &SvgAsset, name: &str, file: &str) -> Mesh {
    asset
        .part(name)
        .cloned()
        .unwrap_or_else(|| panic!("Teil `{name}` fehlt in assets/map/{file}"))
}

fn tile_kind(name: &str) -> Option<Tile> {
    Some(match name {
        "solid" => Tile::Solid,
        "unhookable" => Tile::Unhookable,
        "ice" => Tile::Ice,
        "climb" => Tile::Climb,
        "crumble" => Tile::Crumble,
        _ => return None,
    })
}

impl MapArt {
    /// # Panics
    /// Wenn ein eingebettetes Asset fehlerhaft ist (wird von Tests abgedeckt).
    pub fn load() -> Self {
        let defs: BTreeMap<String, MaterialDef> =
            toml::from_str(MATERIALS_TOML).expect("assets/map/materials.toml");
        let materials = MATERIAL_FILES
            .iter()
            .map(|&(name, data)| {
                let file = format!("materials/{name}.svg");
                let def = defs
                    .get(name)
                    .unwrap_or_else(|| panic!("materials.toml: `{name}` fehlt"));
                let asset = svg(data, &file);
                let caps = asset.part("cap").is_some().then(|| {
                    ["cap", "cap-left", "cap-right", "cap-single"].map(|p| part(&asset, p, &file))
                });
                let details = (1..)
                    .map_while(|i| asset.part(&format!("detail-{i}")).cloned())
                    .collect();
                Material {
                    name: name.to_owned(),
                    body: parse_hex(&def.body)
                        .unwrap_or_else(|| panic!("materials.toml: Farbe von `{name}`")),
                    radius: def.radius,
                    detail_chance: def.details,
                    tiles: def
                        .tiles
                        .iter()
                        .map(|t| {
                            tile_kind(t).unwrap_or_else(|| panic!("materials.toml: Tile-Art `{t}`"))
                        })
                        .collect(),
                    caps,
                    details,
                }
            })
            .collect();
        let file = |n: &str| format!("tiles/{n}.svg");
        let death = svg(
            include_bytes!("../../../assets/map/tiles/death.svg"),
            "death",
        );
        let plank = svg(
            include_bytes!("../../../assets/map/tiles/platform.svg"),
            "platform",
        );
        let jump = svg(include_bytes!("../../../assets/map/tiles/jump.svg"), "jump");
        let belt = svg(
            include_bytes!("../../../assets/map/tiles/conveyor.svg"),
            "conveyor",
        );
        let specials = Specials {
            spikes: part(&death, "up", &file("death")),
            planks: ["mid", "left", "right", "single"].map(|p| part(&plank, p, &file("platform"))),
            jump_up: part(&jump, "up", &file("jump")),
            jump_diag: part(&jump, "diag", &file("jump")),
            belt: part(&belt, "base", &file("conveyor")),
            belt_arrow: part(&belt, "arrow", &file("conveyor")),
        };
        let builtin = DECOR_FILES
            .iter()
            .map(|f| (f, "decor"))
            .chain(BACKGROUND_FILES.iter().map(|f| (f, "backgrounds")))
            .map(|(&(name, data), dir)| {
                let file = format!("{dir}/{name}.svg");
                (name, part(&svg(data, &file), "", &file))
            })
            .collect();
        Self {
            materials,
            specials,
            builtin,
        }
    }

    /// Standard-Material einer Tile-Art.
    pub fn default_material(&self, tile: Tile) -> Option<usize> {
        self.materials.iter().position(|m| m.tiles.contains(&tile))
    }

    /// Material nach Name, wenn es für die Tile-Art wählbar ist; sonst der Standard.
    pub fn material(&self, tile: Tile, name: Option<&str>) -> Option<usize> {
        name.and_then(|n| {
            self.materials
                .iter()
                .position(|m| m.name == n && m.tiles.contains(&tile))
        })
        .or_else(|| self.default_material(tile))
    }

    /// Für feste Tiles wählbare Materialien, Standard zuerst (E-148).
    pub fn solid_material_names(&self) -> Vec<String> {
        self.materials
            .iter()
            .filter(|m| m.tiles.contains(&Tile::Solid))
            .map(|m| m.name.clone())
            .collect()
    }

    pub fn builtin(&self, name: &str) -> Option<&Mesh> {
        self.builtin.get(name)
    }
}

/// Tiles, die zur Fläche gehören (Form und Kontur).
fn ground(t: Tile) -> bool {
    matches!(
        t,
        Tile::Solid
            | Tile::Unhookable
            | Tile::Ice
            | Tile::JumpPad(_)
            | Tile::Conveyor(_)
            | Tile::Climb
            | Tile::Crumble
    )
}

/// Tiles, die mit einem Material gezeichnet werden.
fn has_material(t: Tile) -> bool {
    matches!(
        t,
        Tile::Solid | Tile::Unhookable | Tile::Ice | Tile::Climb | Tile::Crumble
    )
}

/// Fester Pseudo-Zufall je Tile (gleiches Bild bei jedem Laden).
fn hash(x: i32, y: i32) -> u32 {
    #[allow(clippy::cast_sign_loss)]
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^ (h >> 13)
}

/// Freie Seiten eines Tiles.
#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
struct Open {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl Open {
    fn of(col: &Collision, tx: i32, ty: i32) -> Self {
        let free = |dx, dy| !ground(col.tile(tx + dx, ty + dy));
        Self {
            up: free(0, -1),
            down: free(0, 1),
            left: free(-1, 0),
            right: free(1, 0),
        }
    }

    /// Gerundete Ecken: oben links, oben rechts, unten rechts, unten links.
    fn corners(self) -> [bool; 4] {
        [
            self.up && self.left,
            self.up && self.right,
            self.down && self.right,
            self.down && self.left,
        ]
    }
}

/// Körper eines Tiles: Rechteck mit gerundeten Außenecken, an geschlossenen Seiten leicht
/// vergrößert (keine Haarlinien zwischen Tiles).
fn body_path(min: Vec2, open: Open, r: f32) -> Path {
    let ts = TILE_SIZE as f32;
    let grow = |o: bool| if o { 0.0 } else { SEAM };
    let (x0, y0) = (min.x - grow(open.left), min.y - grow(open.up));
    let (x1, y1) = (min.x + ts + grow(open.right), min.y + ts + grow(open.down));
    let [tl, tr, br, bl] = open.corners().map(|c| if c { r } else { 0.0 });
    let mut b = Path::builder();
    b.begin(point(x0 + tl, y0));
    b.line_to(point(x1 - tr, y0));
    b.quadratic_bezier_to(point(x1, y0), point(x1, y0 + tr));
    b.line_to(point(x1, y1 - br));
    b.quadratic_bezier_to(point(x1, y1), point(x1 - br, y1));
    b.line_to(point(x0 + bl, y1));
    b.quadratic_bezier_to(point(x0, y1), point(x0, y1 - bl));
    b.line_to(point(x0, y0 + tl));
    b.quadratic_bezier_to(point(x0, y0), point(x0 + tl, y0));
    b.end(true);
    b.build()
}

/// Punkte einer Viertelrundung (quadratische Kurve von `a` über Ecke `c` nach `b`).
fn arc(a: Vec2, c: Vec2, b: Vec2) -> impl Iterator<Item = Vec2> {
    (0..=6).map(move |i| {
        let t = i as f32 / 6.0;
        a * ((1.0 - t) * (1.0 - t)) + c * (2.0 * t * (1.0 - t)) + b * (t * t)
    })
}

/// Kontur an den freien Seiten eines Tiles, als zusammenhängende Linienzüge.
fn outline(batch: &mut ShapeBatch, min: Vec2, open: Open, r: f32) {
    let ts = TILE_SIZE as f32;
    let max = min + Vec2::new(ts, ts);
    let [tl, tr, br, bl] = open.corners();
    let rad = |c: bool| if c { r } else { 0.0 };
    // Umlauf im Uhrzeigersinn ab oben links: (Seite frei?, Punkte)
    let corner = |round: bool, a: Vec2, c: Vec2, b: Vec2| -> (bool, Vec<Vec2>) {
        if round {
            (true, arc(a, c, b).collect())
        } else {
            (false, vec![c])
        }
    };
    let pieces = [
        corner(
            tl,
            Vec2::new(min.x, min.y + r),
            min,
            Vec2::new(min.x + r, min.y),
        ),
        (
            open.up,
            vec![
                Vec2::new(min.x + rad(tl), min.y),
                Vec2::new(max.x - rad(tr), min.y),
            ],
        ),
        corner(
            tr,
            Vec2::new(max.x - r, min.y),
            Vec2::new(max.x, min.y),
            Vec2::new(max.x, min.y + r),
        ),
        (
            open.right,
            vec![
                Vec2::new(max.x, min.y + rad(tr)),
                Vec2::new(max.x, max.y - rad(br)),
            ],
        ),
        corner(
            br,
            Vec2::new(max.x, max.y - r),
            max,
            Vec2::new(max.x - r, max.y),
        ),
        (
            open.down,
            vec![
                Vec2::new(max.x - rad(br), max.y),
                Vec2::new(min.x + rad(bl), max.y),
            ],
        ),
        corner(
            bl,
            Vec2::new(min.x + r, max.y),
            Vec2::new(min.x, max.y),
            Vec2::new(min.x, max.y - r),
        ),
        (
            open.left,
            vec![
                Vec2::new(min.x, max.y - rad(bl)),
                Vec2::new(min.x, min.y + rad(tl)),
            ],
        ),
    ];
    // Beginn an einer geschlossenen Stelle, damit Linienzüge nicht am Anfang zerschnitten werden
    let Some(start) = pieces.iter().position(|(o, _)| !o) else {
        // ringsum frei: geschlossener Umlauf
        let mut all: Vec<Vec2> = pieces.iter().flat_map(|(_, p)| p.clone()).collect();
        all.push(all[0]);
        batch.stroke_polyline(&all, OUTLINE_WIDTH, OUTLINE);
        return;
    };
    let mut run: Vec<Vec2> = Vec::new();
    for i in 1..=pieces.len() {
        let (o, pts) = &pieces[(start + i) % pieces.len()];
        if *o {
            run.extend(pts);
        } else {
            if run.len() >= 2 {
                batch.stroke_polyline(&run, OUTLINE_WIDTH, OUTLINE);
            }
            run.clear();
        }
    }
}

/// Mesh im Tile `min` zeichnen, um die Tile-Mitte gedreht (`angle`) und/oder gespiegelt.
fn tile_mesh(batch: &mut ShapeBatch, mesh: &Mesh, min: Vec2, angle: f32, flip: bool) {
    let half = TILE_SIZE as f32 / 2.0;
    let t = Affine::translate(min + Vec2::new(half, half))
        .then(Affine::rotate(angle))
        .then(Affine::scale(if flip { -1.0 } else { 1.0 }, 1.0))
        .then(Affine::translate(Vec2::new(-half, -half)));
    batch.draw_mesh(mesh, &t, &Tint::default());
}

/// Ausschnitt in Tiles (einschließlich).
#[derive(Debug, Clone, Copy)]
pub struct TileRange {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

impl TileRange {
    fn tiles(self) -> impl Iterator<Item = (i32, i32)> {
        (self.y0..=self.y1).flat_map(move |y| (self.x0..=self.x1).map(move |x| (x, y)))
    }
}

/// Spielfläche zeichnen. `material(x, y)` liefert den Index in [`MapArt::materials`].
pub fn draw_terrain(
    batch: &mut ShapeBatch,
    art: &MapArt,
    col: &Collision,
    material: &dyn Fn(i32, i32) -> Option<usize>,
    range: TileRange,
) {
    let ts = TILE_SIZE as f32;
    let cells: Vec<(Vec2, Open, &Material, u32)> = range
        .tiles()
        .filter(|&(x, y)| has_material(col.tile(x, y)))
        .filter_map(|(x, y)| {
            let m = &art.materials[material(x, y)?];
            Some((
                Vec2::new(x as f32 * ts, y as f32 * ts),
                Open::of(col, x, y),
                m,
                hash(x, y),
            ))
        })
        .collect();
    for &(min, open, m, _) in &cells {
        batch.fill_path(&body_path(min, open, m.radius), m.body);
    }
    for &(min, open, m, h) in &cells {
        #[allow(clippy::cast_precision_loss)]
        let roll = (h % 1000) as f32 / 1000.0;
        if !m.details.is_empty() && roll < m.detail_chance {
            let d = &m.details[(h / 1000) as usize % m.details.len()];
            tile_mesh(batch, d, min, 0.0, h & 1 == 1);
        }
        if open.up
            && let Some(caps) = &m.caps
        {
            let i = usize::from(open.left) + 2 * usize::from(open.right);
            tile_mesh(batch, &caps[i], min, 0.0, false);
        }
    }
    for &(min, open, m, _) in &cells {
        outline(batch, min, open, m.radius);
    }
    draw_specials(batch, art, col, range);
}

fn draw_specials(batch: &mut ShapeBatch, art: &MapArt, col: &Collision, range: TileRange) {
    use std::f32::consts::{FRAC_PI_2, PI};
    let ts = TILE_SIZE as f32;
    let s = &art.specials;
    for (x, y) in range.tiles() {
        let min = Vec2::new(x as f32 * ts, y as f32 * ts);
        match col.tile(x, y) {
            Tile::Death => {
                // Stacheln zeigen vom festen Untergrund weg
                let free = |dx, dy| {
                    let t = col.tile(x + dx, y + dy);
                    !ground(t) && t != Tile::Death
                };
                let angle = if free(0, -1) {
                    0.0
                } else if free(0, 1) {
                    PI
                } else if free(1, 0) {
                    FRAC_PI_2
                } else if free(-1, 0) {
                    -FRAC_PI_2
                } else {
                    0.0
                };
                tile_mesh(batch, &s.spikes, min, angle, false);
            }
            Tile::Platform => {
                let left = col.tile(x - 1, y) != Tile::Platform;
                let right = col.tile(x + 1, y) != Tile::Platform;
                let i = usize::from(left) + 2 * usize::from(right);
                tile_mesh(batch, &s.planks[i], min, 0.0, false);
            }
            Tile::JumpPad(JumpDir::Up) => tile_mesh(batch, &s.jump_up, min, 0.0, false),
            Tile::JumpPad(JumpDir::UpRight) => tile_mesh(batch, &s.jump_diag, min, 0.0, false),
            Tile::JumpPad(JumpDir::UpLeft) => tile_mesh(batch, &s.jump_diag, min, 0.0, true),
            // Pfeil läuft mit und wird je Frame gezeichnet (draw_belt_arrow)
            Tile::Conveyor(_) => tile_mesh(batch, &s.belt, min, 0.0, false),
            _ => {}
        }
    }
}

/// Laufender Pfeil eines Beschleunigers; `phase` 0..1 wandert in Laufrichtung und blendet an den Enden aus.
pub fn draw_belt_arrow(batch: &mut ShapeBatch, art: &MapArt, min: Vec2, dir: BeltDir, phase: f32) {
    let half = TILE_SIZE as f32 / 2.0;
    let shift = (phase - 0.5) * 8.0 * dir.sign();
    let t = Affine::translate(min + Vec2::new(half + shift, half))
        .then(Affine::scale(dir.sign(), 1.0))
        .then(Affine::translate(Vec2::new(-half, -half)));
    let tint = Tint {
        alpha: Some((1.0 - (phase - 0.5).abs() * 2.0).clamp(0.0, 1.0).sqrt()),
        ..Tint::default()
    };
    batch.draw_mesh(&art.specials.belt_arrow, &t, &tint);
}

/// Wirkung der Animationen auf ein Deko-Objekt.
#[derive(Debug, Clone, Copy)]
pub struct Anim {
    pub offset: Vec2,
    /// Grad
    pub rotation: f32,
    /// Farbe (multipliziert)
    pub color: [f32; 4],
}

impl Default for Anim {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            rotation: 0.0,
            color: [1.0; 4],
        }
    }
}

pub fn rgba(c: Rgba) -> Color {
    Color(c.0.map(|v| f32::from(v) / 255.0))
}

/// Grafik eines Deko-Objekts; eingebettete Bilder (`images`) sind vorab geladene Meshes der Karte.
pub fn decor_mesh<'a>(art: &'a MapArt, images: &'a [Mesh], d: &Decor) -> Option<&'a Mesh> {
    match &d.art {
        Art::Builtin(name) => art.builtin(name),
        Art::Image(i) => images.get(usize::from(*i)),
    }
}

/// Ein Deko-Objekt an `at` (Position samt Lage der Ebene) zeichnen.
pub fn draw_decor(batch: &mut ShapeBatch, mesh: &Mesh, d: &Decor, at: Vec2, anim: &Anim) {
    let flip = if d.flip_x { -1.0 } else { 1.0 };
    let t = Affine::translate(at + anim.offset)
        .then(Affine::rotate((d.rotation + anim.rotation).to_radians()))
        .then(Affine::scale(d.scale * flip, d.scale));
    let base = rgba(d.tint).0;
    let color: [f32; 4] = std::array::from_fn(|i| base[i] * anim.color[i]);
    let tint = Tint {
        multiply: color
            .iter()
            .any(|c| (c - 1.0).abs() > f32::EPSILON)
            .then_some(Color(color)),
        ..Tint::default()
    };
    batch.draw_mesh(mesh, &t, &tint);
}

#[cfg(test)]
mod tests {
    use super::*;
    use elora_map::Sky;

    fn put(batch: &mut ShapeBatch, art: &MapArt, d: &Decor) {
        let mesh = decor_mesh(art, &[], d).expect("eingebaut");
        draw_decor(batch, mesh, d, d.pos, &Anim::default());
    }
    use std::fmt::Write as _;

    #[test]
    fn all_assets_load() {
        let art = MapArt::load();
        assert_eq!(art.materials.len(), 7);
        for t in [
            Tile::Solid,
            Tile::Unhookable,
            Tile::Ice,
            Tile::Climb,
            Tile::Crumble,
        ] {
            assert!(art.default_material(t).is_some(), "{t:?} ohne Material");
        }
        assert_eq!(
            art.materials[art.material(Tile::Solid, Some("snow")).unwrap()].name,
            "snow"
        );
        // Stein ist für hookbare Wände nicht wählbar (Lesbarkeit hookbar/nicht hookbar)
        assert_eq!(
            art.materials[art.material(Tile::Solid, Some("stone")).unwrap()].name,
            "earth"
        );
        for (name, _) in DECOR_FILES.iter().chain(BACKGROUND_FILES) {
            assert!(!art.builtin(name).unwrap().is_empty(), "{name} leer");
        }
        for m in &art.materials {
            assert!(!m.details.is_empty(), "{} ohne Details", m.name);
        }
    }

    /// Zeichnet ein Raster aus Zeichen; Kleinbuchstaben wählen das Material fester Tiles
    /// (`e` Erde, `s` Sand, `n` Schnee), sonst wie die Aufzeichnungen.
    fn grid(batch: &mut ShapeBatch, art: &MapArt, rows: &[&str], w: usize, h: usize) {
        let mut tiles = vec![Tile::Air; w * h];
        let mut mats = vec![None; w * h];
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                let i = y * w + x;
                let (tile, mat) = match c {
                    'e' => (Tile::Solid, Some("earth")),
                    's' => (Tile::Solid, Some("sand")),
                    'n' => (Tile::Solid, Some("snow")),
                    c => (Tile::from_char(c).unwrap_or(Tile::Air), None),
                };
                tiles[i] = tile;
                mats[i] = art.material(tile, mat);
            }
        }
        let col = Collision::new(w, h, tiles.clone());
        let material = |x: i32, y: i32| {
            let (x, y) = (usize::try_from(x).ok()?, usize::try_from(y).ok()?);
            mats.get(y * w + x).copied().flatten()
        };
        #[allow(clippy::cast_possible_wrap)]
        let range = TileRange {
            x0: 0,
            y0: 0,
            x1: w as i32 - 1,
            y1: h as i32 - 1,
        };
        draw_terrain(batch, art, &col, &material, range);
    }

    fn strip(
        batch: &mut ShapeBatch,
        art: &MapArt,
        name: &str,
        y: f32,
        x0: f32,
        x1: f32,
        tint: Rgba,
    ) {
        let mut x = x0;
        while x < x1 {
            let mut d = Decor::new(Art::Builtin(name.into()), Vec2::new(x, y));
            d.tint = tint;
            put(batch, art, &d);
            x += STRIP_WIDTH;
        }
    }

    /// Hintergrund-Szene: Himmel, Berge, Hügel, Wald, Wolken bzw. Sterne und Mond.
    fn scene(batch: &mut ShapeBatch, art: &MapArt, top: f32, w: f32, night: bool) {
        let h = 420.0;
        let sky = if night {
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
        batch.fill_rect_vgradient(
            Vec2::new(0.0, top),
            Vec2::new(w, top + h),
            rgba(sky.top),
            rgba(sky.bottom),
        );
        let base = top + h;
        let dim = |c: u32| if night { Rgba::hex(c) } else { Rgba::WHITE };
        if night {
            strip(batch, art, "stars", top, 0.0, w, Rgba::WHITE);
            let moon = Decor::new(Art::Builtin("moon".into()), Vec2::new(w * 0.8, top + 90.0));
            put(batch, art, &moon);
        } else {
            for (name, x, y) in [
                ("cloud-1", 180.0, 70.0),
                ("cloud-2", 620.0, 50.0),
                ("cloud-3", 1050.0, 95.0),
            ] {
                let c = Decor::new(Art::Builtin(name.into()), Vec2::new(x, top + y));
                put(batch, art, &c);
            }
        }
        strip(batch, art, "mountains", base, -200.0, w, dim(0x5c6a9a));
        strip(batch, art, "hills-far", base, -400.0, w, dim(0x56638f));
        strip(batch, art, "forest", base, -100.0, w, dim(0x4a5a80));
        strip(batch, art, "hills-near", base, -600.0, w, dim(0x46557a));
    }

    /// Übersichtsblatt zur Sichtprüfung: `cargo test -p elora-client --bin elora map_art_sheet -- --ignored`,
    /// danach `cargo xtask svg-preview target/map-art.svg docs/archiv/release-1/design/elora-kartenteile.png 1400`.
    #[test]
    #[ignore = "erzeugt nur eine Datei zur Sichtprüfung"]
    #[allow(clippy::too_many_lines)]
    fn map_art_sheet() {
        let art = MapArt::load();
        let mut batch = ShapeBatch::default();
        batch.tolerance = TOLERANCE;
        let (w, h) = (48usize, 25usize);
        let width = w as f32 * 32.0;
        let mut labels = Vec::new();
        let mut label = |x: f32, y: f32, text: &str, size: u32| {
            labels.push(format!(
                r##"<text x="{x}" y="{y}" font-family="Inter" font-size="{size}" fill="#1e2a36">{text}</text>"##
            ));
        };

        // 1) Materialien und Spezial-Tiles (Tiles ab Zeile 2)
        batch.fill_rect(
            Vec2::ZERO,
            Vec2::new(width, h as f32 * 32.0),
            Color::hex(0xd8e3ec),
        );
        batch.fill_rect_vgradient(
            Vec2::new(0.0, 40.0),
            Vec2::new(width, 26.0 * 32.0),
            Color::hex(0xa9cde8),
            Color::hex(0xe6f2f8),
        );
        let rows = [
            "................................................",
            "................................................",
            "..eeee....ssss....nnnn....%%%%....~~~~..........",
            "..eeeeee..ssssss..nnnnnn..%%%%%%..~~~~~~....e...",
            "..ee..ee..ss..ss..nn..nn..%%..%%..~~..~~........",
            "..eeeeee..ssssss..nnnnnn..%%%%%%..~~~~~~...eee..",
            "...ee......ss......nn......%%......~~.......e...",
            "................................................",
            "..........................||||||....::::::......",
            "..........................||||||....::::::......",
            "..===.....=.......................e..........e..",
            "..................................e..........e..",
            "..ee^^^ee....!\\/.....<<<>>>.......e^^^^^^^^^^e..",
            "..eeeeeee..eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee.",
            "..eeeeeee..eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee.",
            "................................................",
            "................................................",
            "................................................",
            "................................................",
            "................................................",
            "................................................",
            "................................................",
            "................................................",
            "..eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee..",
        ];
        grid(&mut batch, &art, &rows, w, h);
        label(16.0, 30.0, "Kartenteile im Stil A (M6.3)", 26);
        for (x, t) in [
            (2, "Erde"),
            (10, "Sand"),
            (18, "Schnee"),
            (26, "Stein (nicht hookbar)"),
            (34, "Eis"),
        ] {
            label(x as f32 * 32.0, 2.0 * 32.0 - 8.0, t, 15);
        }
        label(43.0 * 32.0, 2.0 * 32.0 + 22.0, "Einzel-", 12);
        label(43.0 * 32.0, 2.0 * 32.0 + 36.0, "stücke", 12);
        for (x, t) in [
            (2, "Plattformen"),
            (13, "Sprungfelder"),
            (21, "Beschleuniger"),
            (35, "Tod (Grube)"),
        ] {
            label(x as f32 * 32.0, 10.0 * 32.0 - 8.0, t, 15);
        }
        label(4.0 * 32.0, 12.0 * 32.0 - 4.0, "Tod", 11);
        label(26.0 * 32.0, 8.0 * 32.0 - 6.0, "Kletterwand", 15);
        label(36.0 * 32.0, 8.0 * 32.0 - 6.0, "Bröckelboden", 15);

        // 2) Deko auf dem Boden (Zeile 23)
        let ground_y = 23.0 * 32.0;
        let names: Vec<&str> = DECOR_FILES.iter().map(|(n, _)| *n).collect();
        let step = (width - 160.0) / names.len() as f32;
        for (i, name) in names.iter().enumerate() {
            let x = 110.0 + i as f32 * step;
            let d = Decor::new(Art::Builtin((*name).into()), Vec2::new(x, ground_y));
            put(&mut batch, &art, &d);
            label(x - 28.0, ground_y + 52.0 + (i % 2) as f32 * 16.0, name, 12);
        }
        label(16.0, 16.0 * 32.0, "Deko", 18);

        // 3) Hintergründe Tag und Nacht
        let day_top = h as f32 * 32.0 + 40.0;
        scene(&mut batch, &art, day_top, width, false);
        let night_top = day_top + 460.0;
        scene(&mut batch, &art, night_top, width, true);
        label(
            16.0,
            day_top - 12.0,
            "Hintergrund Tag: Wolken, Berge, ferne Hügel, Wald, nahe Hügel",
            18,
        );
        label(
            16.0,
            night_top - 12.0,
            "Hintergrund Nacht: Sterne, Mond, dieselben Ebenen abgedunkelt",
            18,
        );

        let max = Vec2::new(width, night_top + 420.0);
        let mut svg = batch.debug_svg(Vec2::ZERO, max, Color::hex(0xd8e3ec));
        svg.truncate(svg.len() - "</svg>".len());
        for l in labels {
            let _ = write!(svg, "{l}");
        }
        svg.push_str("</svg>");
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/map-art.svg"),
            svg,
        )
        .unwrap();
    }
}

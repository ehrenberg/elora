//! SVG-Assets (M5.2, D-M5-02): SVG → gecachte [`Mesh`]es.
//!
//! Konventionen für Elora-Assets:
//!
//! - **Ursprung:** Die `viewBox` legt die lokalen Koordinaten fest, z. B.
//!   `viewBox="-60 -70 120 140"` für eine Figur mit Mittelpunkt (0, 0).
//! - **Teile:** Jede Gruppe der obersten Ebene mit `id` wird ein eigenes Mesh
//!   (z. B. `body`, `foot`, `eyes`), damit Teile einzeln animiert werden können.
//!   Alles andere landet im Teil `""`.
//! - **Farbschlüssel:** Eine `id` der Form `tint-<n>` (an der Form oder einer
//!   umschließenden Gruppe) ersetzt die Füllfarbe beim Zeichnen durch Farbe `n`
//!   der [`Tint`](crate::Tint). Mit `-l<prozent>` wird sie aufgehellt, mit
//!   `-d<prozent>` abgedunkelt, z. B. `tint-2-l45` (Bauchfleck, E-097).
//!   Mehrere Formen: `tint-2`, `tint-2-a`, `tint-2-l45-b` – alles nach dem
//!   Muster wird ignoriert. Die Farbe in der Datei bleibt die Vorschaufarbe.
//! - **Konturen** behalten ihre Farbe und werden immer mit runden Ecken gezeichnet.
//! - Nicht unterstützt: radiale Verläufe, Muster, Bilder, Text, Filter.

use elora_sim::Vec2;
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::FillRule;
use usvg::tiny_skia_path::{PathSegment, Transform};

use crate::Color;
use crate::mesh::{Mesh, MeshBuilder, Paint};

#[derive(Debug, thiserror::Error)]
pub enum SvgError {
    #[error("SVG nicht lesbar: {0}")]
    Parse(#[from] usvg::Error),
    #[error("SVG-Merkmal nicht unterstützt: {0}")]
    Unsupported(String),
}

/// Ein geladenes SVG, aufgeteilt nach Gruppen der obersten Ebene.
#[derive(Debug, Clone, Default)]
pub struct SvgAsset {
    pub parts: Vec<(String, Mesh)>,
}

impl SvgAsset {
    /// Lädt ein SVG; `tolerance` in lokalen Einheiten (siehe [`MeshBuilder::new`]).
    ///
    /// # Errors
    /// Wenn das SVG nicht lesbar ist oder nicht unterstützte Merkmale enthält.
    pub fn load(data: &[u8], tolerance: f32) -> Result<Self, SvgError> {
        let text = std::str::from_utf8(data)
            .map_err(|e| SvgError::Unsupported(format!("keine UTF-8-Datei: {e}")))?;
        let tree = usvg::Tree::from_str(text, &usvg::Options::default())?;
        let local = to_local(text, &tree);
        let mut asset = Self::default();
        let mut rest = MeshBuilder::new(tolerance);
        // usvg legt Hüllgruppen ohne id an (z. B. für die viewBox) – überspringen
        let mut top = tree.root();
        while let [usvg::Node::Group(g)] = top.children()
            && g.id().is_empty()
        {
            top = g;
        }
        for node in top.children() {
            match node {
                usvg::Node::Group(g) if !g.id().is_empty() => {
                    let mut b = MeshBuilder::new(tolerance);
                    add_group(&mut b, g, &local, Key::from_id(g.id()), 1.0)?;
                    asset.parts.push((g.id().to_owned(), b.build()));
                }
                other => add_node(&mut rest, other, &local, None, 1.0)?,
            }
        }
        let rest = rest.build();
        if !rest.is_empty() {
            asset.parts.push((String::new(), rest));
        }
        Ok(asset)
    }

    pub fn part(&self, name: &str) -> Option<&Mesh> {
        self.parts.iter().find(|(n, _)| n == name).map(|(_, m)| m)
    }
}

/// Farbschlüssel aus einer `id` wie `tint-2-l45`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Key {
    slot: u8,
    shade: f32,
}

impl Key {
    fn from_id(id: &str) -> Option<Self> {
        let mut parts = id.strip_prefix("tint-")?.split('-');
        let slot: u8 = parts.next()?.parse().ok().filter(|s| *s > 0)?;
        let shade = parts
            .next()
            .and_then(|p| {
                let (sign, num) = match p.split_at_checked(1)? {
                    ("l", n) => (1.0, n),
                    ("d", n) => (-1.0, n),
                    _ => return None,
                };
                num.parse::<f32>().ok().map(|v| sign * v / 100.0)
            })
            .unwrap_or(0.0);
        Some(Self { slot, shade })
    }
}

/// Transformation von usvg-Ausgabekoordinaten zurück in `viewBox`-Koordinaten.
fn to_local(text: &str, tree: &usvg::Tree) -> Transform {
    let view_box = usvg::roxmltree::Document::parse(text).ok().and_then(|doc| {
        let v: Vec<f32> = doc
            .root_element()
            .attribute("viewBox")?
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse().ok())
            .collect();
        (v.len() == 4 && v[2] > 0.0 && v[3] > 0.0).then(|| [v[0], v[1], v[2], v[3]])
    });
    let Some([x, y, w, h]) = view_box else {
        return Transform::identity();
    };
    let size = tree.size();
    let (sx, sy) = (w / size.width(), h / size.height());
    Transform::from_row(sx, 0.0, 0.0, sy, x, y)
}

fn add_group(
    b: &mut MeshBuilder,
    g: &usvg::Group,
    local: &Transform,
    key: Option<Key>,
    opacity: f32,
) -> Result<(), SvgError> {
    let opacity = opacity * g.opacity().get();
    for child in g.children() {
        add_node(b, child, local, key, opacity)?;
    }
    Ok(())
}

fn add_node(
    b: &mut MeshBuilder,
    node: &usvg::Node,
    local: &Transform,
    key: Option<Key>,
    opacity: f32,
) -> Result<(), SvgError> {
    match node {
        usvg::Node::Group(g) => add_group(b, g, local, Key::from_id(g.id()).or(key), opacity),
        usvg::Node::Path(p) => {
            if p.is_visible() {
                add_path(b, p, local, Key::from_id(p.id()).or(key), opacity)?;
            }
            Ok(())
        }
        usvg::Node::Image(_) => Err(SvgError::Unsupported("Bild".into())),
        // Ohne Schrift-Feature liefert usvg keinen Text
        usvg::Node::Text(_) => Err(SvgError::Unsupported("Text".into())),
    }
}

fn add_path(
    b: &mut MeshBuilder,
    p: &usvg::Path,
    local: &Transform,
    key: Option<Key>,
    opacity: f32,
) -> Result<(), SvgError> {
    let t = local.pre_concat(p.abs_transform());
    let path = convert_path(p.data(), &t);
    let fill = p
        .fill()
        .map(|f| -> Result<_, SvgError> {
            let paint = match key {
                Some(k) => Paint::Key {
                    slot: k.slot,
                    shade: k.shade,
                    alpha: f.opacity().get() * opacity,
                },
                None => paint(f.paint(), f.opacity().get() * opacity, &t)?,
            };
            let rule = match f.rule() {
                usvg::FillRule::NonZero => FillRule::NonZero,
                usvg::FillRule::EvenOdd => FillRule::EvenOdd,
            };
            Ok((paint, rule))
        })
        .transpose()?;
    let stroke = p
        .stroke()
        .map(|s| -> Result<_, SvgError> {
            // Breite mit der mittleren Skalierung der Transformation
            let scale = (t.sx * t.sy - t.kx * t.ky).abs().sqrt();
            Ok((
                s.width().get() * scale,
                paint(s.paint(), s.opacity().get() * opacity, &t)?,
            ))
        })
        .transpose()?;
    let fill_first = p.paint_order() == usvg::PaintOrder::FillAndStroke;
    if fill_first && let Some((paint, rule)) = fill {
        b.fill_path_with_rule(&path, paint, rule);
    }
    if let Some((width, paint)) = stroke {
        b.stroke_path(&path, width, paint);
    }
    if !fill_first && let Some((paint, rule)) = fill {
        b.fill_path_with_rule(&path, paint, rule);
    }
    Ok(())
}

fn color(c: usvg::Color, alpha: f32) -> Color {
    Color::rgba(
        f32::from(c.red) / 255.0,
        f32::from(c.green) / 255.0,
        f32::from(c.blue) / 255.0,
        alpha,
    )
}

fn paint(p: &usvg::Paint, alpha: f32, t: &Transform) -> Result<Paint, SvgError> {
    match p {
        usvg::Paint::Color(c) => Ok(Paint::Solid(color(*c, alpha))),
        usvg::Paint::LinearGradient(g) => {
            let (Some(first), Some(last)) = (g.stops().first(), g.stops().last()) else {
                return Err(SvgError::Unsupported("Verlauf ohne Farben".into()));
            };
            if g.stops().len() > 2 {
                return Err(SvgError::Unsupported(
                    "Verlauf mit mehr als zwei Farben".into(),
                ));
            }
            let gt = t.pre_concat(g.transform());
            let map = |x: f32, y: f32| {
                let mut pt = usvg::tiny_skia_path::Point::from_xy(x, y);
                gt.map_point(&mut pt);
                Vec2::new(pt.x, pt.y)
            };
            Ok(Paint::Linear {
                from: map(g.x1(), g.y1()),
                to: map(g.x2(), g.y2()),
                a: color(first.color(), first.opacity().get() * alpha),
                b: color(last.color(), last.opacity().get() * alpha),
            })
        }
        usvg::Paint::RadialGradient(_) => Err(SvgError::Unsupported("radialer Verlauf".into())),
        usvg::Paint::Pattern(_) => Err(SvgError::Unsupported("Muster".into())),
    }
}

fn convert_path(data: &usvg::tiny_skia_path::Path, t: &Transform) -> Path {
    let map = |mut p: usvg::tiny_skia_path::Point| {
        t.map_point(&mut p);
        point(p.x, p.y)
    };
    let mut b = Path::builder();
    let mut open = false;
    for seg in data.segments() {
        match seg {
            PathSegment::MoveTo(p) => {
                if open {
                    b.end(false);
                }
                b.begin(map(p));
                open = true;
            }
            PathSegment::LineTo(p) => {
                b.line_to(map(p));
            }
            PathSegment::QuadTo(c, p) => {
                b.quadratic_bezier_to(map(c), map(p));
            }
            PathSegment::CubicTo(c1, c2, p) => {
                b.cubic_bezier_to(map(c1), map(c2), map(p));
            }
            PathSegment::Close => {
                if open {
                    b.end(true);
                    open = false;
                }
            }
        }
    }
    if open {
        b.end(false);
    }
    b.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Affine, Tint};

    const FIGURE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="240" height="280" viewBox="-60 -70 120 140">
  <g id="body">
    <circle id="tint-2" cx="0" cy="0" r="50" fill="#f2c14e" stroke="#2b2b2b" stroke-width="5"/>
    <ellipse id="tint-2-l45" cx="0" cy="30" rx="30" ry="10" fill="#fbe3a1"/>
  </g>
  <g id="foot"><ellipse cx="0" cy="0" rx="14" ry="8" fill="#d9a43a"/></g>
  <rect x="-5" y="-5" width="10" height="10" fill="#ff0000"/>
</svg>"##;

    fn emitted(mesh: &Mesh, tint: &Tint) -> Vec<crate::shapes::Vertex> {
        let mut out = lyon::tessellation::VertexBuffers::new();
        mesh.emit(&Affine::IDENTITY, tint, &mut out);
        out.vertices
    }

    #[test]
    fn keys_from_ids() {
        assert_eq!(
            Key::from_id("tint-2"),
            Some(Key {
                slot: 2,
                shade: 0.0
            })
        );
        assert_eq!(
            Key::from_id("tint-2-l45-b"),
            Some(Key {
                slot: 2,
                shade: 0.45
            })
        );
        assert_eq!(Key::from_id("tint-3-d20").map(|k| k.slot), Some(3));
        assert!((Key::from_id("tint-3-d20").unwrap().shade + 0.2).abs() < 1e-6);
        assert_eq!(Key::from_id("tint-0"), None);
        assert_eq!(Key::from_id("body"), None);
        assert_eq!(Key::from_id("tint-a"), None);
    }

    #[test]
    fn parts_and_local_coordinates() {
        let asset = SvgAsset::load(FIGURE.as_bytes(), 0.25).expect("lesbar");
        let names: Vec<&str> = asset.parts.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["body", "foot", ""]);
        // viewBox-Koordinaten, nicht Pixel: Kreis r=50 + halbe Kontur um (0,0)
        let (min, max) = asset.part("body").unwrap().bounds().unwrap();
        assert!(
            (min.x + 52.5).abs() < 0.5 && (max.x - 52.5).abs() < 0.5,
            "{min:?} {max:?}"
        );
        let (min, max) = asset.part("foot").unwrap().bounds().unwrap();
        assert!((min.x + 14.0).abs() < 0.3 && (max.y - 8.0).abs() < 0.3);
    }

    #[test]
    fn tint_replaces_keyed_fill_only() {
        let asset = SvgAsset::load(FIGURE.as_bytes(), 0.25).expect("lesbar");
        let body = asset.part("body").unwrap();
        let blue = Color::rgb(0.0, 0.0, 1.0);
        let v = emitted(body, &Tint::new(vec![Color::rgb(1.0, 0.0, 0.0), blue]));
        let has = |c: [f32; 4]| {
            v.iter()
                .any(|x| x.color.iter().zip(c).all(|(a, b)| (a - b).abs() < 1e-3))
        };
        assert!(has([0.0, 0.0, 1.0, 1.0]), "Körper in Schlüsselfarbe");
        assert!(has([0.45, 0.45, 1.0, 1.0]), "Bauchfleck aufgehellt");
        let outline = f32::from(0x2b_u8) / 255.0;
        assert!(has([outline, outline, outline, 1.0]), "Kontur bleibt");
        assert!(
            !has([242.0 / 255.0, 193.0 / 255.0, 78.0 / 255.0, 1.0]),
            "Vorschaufarbe ersetzt"
        );
    }

    #[test]
    fn linear_gradient_and_unsupported() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">
  <defs><linearGradient id="g" x1="0" y1="0" x2="0" y2="10" gradientUnits="userSpaceOnUse">
    <stop offset="0" stop-color="#000000"/><stop offset="1" stop-color="#ffffff"/></linearGradient></defs>
  <rect width="10" height="10" fill="url(#g)"/></svg>"##;
        let asset = SvgAsset::load(svg.as_bytes(), 0.25).expect("lesbar");
        let v = emitted(asset.part("").unwrap(), &Tint::default());
        let top = v.iter().find(|x| x.pos[1] < 0.01).unwrap();
        let bottom = v.iter().find(|x| x.pos[1] > 9.99).unwrap();
        assert!(top.color[0] < 0.01 && bottom.color[0] > 0.99);

        let radial = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">
  <defs><radialGradient id="g"><stop offset="0" stop-color="#000"/><stop offset="1" stop-color="#fff"/></radialGradient></defs>
  <rect width="10" height="10" fill="url(#g)"/></svg>"##;
        assert!(matches!(
            SvgAsset::load(radial.as_bytes(), 0.25),
            Err(SvgError::Unsupported(_))
        ));
    }
}

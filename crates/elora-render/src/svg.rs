//! SVG assets (M5.2, D-M5-02): SVG → cached [`Mesh`]es.
//!
//! Conventions for Elora assets:
//!
//! - **Origin:** The `viewBox` defines the local coordinates, e.g.
//!   `viewBox="-60 -70 120 140"` for a figure centered on (0, 0).
//! - **Parts:** Every top-level group with an `id` becomes its own mesh
//!   (e.g. `body`, `foot`, `eyes`) so parts can be animated individually.
//!   Everything else ends up in the part `""`.
//! - **Color keys:** An `id` of the form `tint-<n>` (on the shape or an
//!   enclosing group) replaces the fill color at draw time with color `n`
//!   of the [`Tint`](crate::Tint). `-l<percent>` lightens it,
//!   `-d<percent>` darkens it, e.g. `tint-2-l45` (belly patch, E-097).
//!   Multiple shapes: `tint-2`, `tint-2-a`, `tint-2-l45-b` – anything after the
//!   pattern is ignored. The color in the file remains the preview color.
//! - **Strokes** keep their color and are always drawn with round joins –
//!   except for shapes without a fill: there the color key colors the stroke
//!   (e.g. eyes as lines `> <`).
//! - Not supported: radial gradients, patterns, images, text, filters.

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

/// A loaded SVG, split by top-level groups.
#[derive(Debug, Clone, Default)]
pub struct SvgAsset {
    pub parts: Vec<(String, Mesh)>,
}

impl SvgAsset {
    /// Loads an SVG; `tolerance` in local units (see [`MeshBuilder::new`]).
    ///
    /// # Errors
    /// If the SVG cannot be read or contains unsupported features.
    pub fn load(data: &[u8], tolerance: f32) -> Result<Self, SvgError> {
        Self::load_with(data, tolerance, &usvg::Options::default())
    }

    /// Loads a foreign SVG (e.g. from a downloaded map): references to files
    /// or embedded images are never resolved, and the geometry is limited.
    ///
    /// # Errors
    /// Like [`SvgAsset::load`], and also with more than `max_vertices` vertices.
    pub fn load_untrusted(
        data: &[u8],
        tolerance: f32,
        max_vertices: usize,
    ) -> Result<Self, SvgError> {
        let options = usvg::Options {
            resources_dir: None,
            image_href_resolver: usvg::ImageHrefResolver {
                resolve_data: Box::new(|_, _, _| None),
                resolve_string: Box::new(|_, _| None),
            },
            ..usvg::Options::default()
        };
        let asset = Self::load_with(data, tolerance, &options)?;
        let vertices: usize = asset.parts.iter().map(|(_, m)| m.vertex_count()).sum();
        if vertices > max_vertices {
            return Err(SvgError::Unsupported(format!(
                "zu aufwendig ({vertices} Ecken, erlaubt {max_vertices})"
            )));
        }
        Ok(asset)
    }

    fn load_with(data: &[u8], tolerance: f32, options: &usvg::Options) -> Result<Self, SvgError> {
        let text = std::str::from_utf8(data)
            .map_err(|e| SvgError::Unsupported(format!("keine UTF-8-Datei: {e}")))?;
        let tree = usvg::Tree::from_str(text, options)?;
        let local = to_local(text, &tree);
        let mut asset = Self::default();
        let mut rest = MeshBuilder::new(tolerance);
        // usvg creates wrapper groups without an id (e.g. for the viewBox) – skip them
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

/// Color key from an `id` like `tint-2-l45`.
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

/// Transformation from usvg output coordinates back to `viewBox` coordinates.
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
        // Without the font feature, usvg yields no text
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
            // Width with the average scale of the transformation
            let scale = (t.sx * t.sy - t.kx * t.ky).abs().sqrt();
            let alpha = s.opacity().get() * opacity;
            // Shapes without a fill: the color key colors the stroke
            let paint = match key {
                Some(k) if p.fill().is_none() => Paint::Key {
                    slot: k.slot,
                    shade: k.shade,
                    alpha,
                },
                _ => paint(s.paint(), alpha, &t)?,
            };
            Ok((s.width().get() * scale, paint))
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

    #[test]
    fn untrusted_svg_resolves_nothing_and_is_limited() {
        // References to files are not loaded: no content instead of file access
        let img = br#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 10 10"><image width="10" height="10" xlink:href="/etc/hostname"/><image width="10" height="10" href="file:///etc/hostname"/></svg>"#;
        let a = SvgAsset::load_untrusted(img, 0.1, 1000).unwrap();
        assert!(a.parts.iter().all(|(_, m)| m.is_empty()));
        let circle = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><circle cx="50" cy="50" r="50" fill="red"/></svg>"#;
        assert!(SvgAsset::load_untrusted(circle, 0.1, 100_000).is_ok());
        assert!(matches!(
            SvgAsset::load_untrusted(circle, 0.001, 50),
            Err(SvgError::Unsupported(_))
        ));
        assert!(SvgAsset::load_untrusted(b"\xff\xfe", 0.1, 100).is_err());
    }
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
        // viewBox coordinates, not pixels: circle r=50 + half the stroke around (0,0)
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
    fn key_tints_stroke_of_unfilled_shape() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">
  <path id="tint-1" d="M 1,1 L 9,9" fill="none" stroke="#2b2b2b" stroke-width="2"/></svg>"##;
        let asset = SvgAsset::load(svg.as_bytes(), 0.25).expect("lesbar");
        let v = emitted(
            asset.part("").unwrap(),
            &Tint::new(vec![Color::rgb(0.0, 1.0, 0.0)]),
        );
        let green = |c: [f32; 4]| {
            c.iter()
                .zip([0.0, 1.0, 0.0, 1.0])
                .all(|(a, b)| (a - b).abs() < 1e-6)
        };
        assert!(!v.is_empty() && v.iter().all(|x| green(x.color)));
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

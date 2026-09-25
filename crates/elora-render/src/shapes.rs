//! Vektorformen, zur Laufzeit per lyon tesselliert (E-033).

use elora_sim::Vec2;
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};

/// RGBA-Farbe, Komponenten 0..=1 (sRGB).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub [f32; 4]);

impl Color {
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self([r, g, b, 1.0])
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self([r, g, b, a])
    }

    /// Aus 0xRRGGBB.
    pub const fn hex(v: u32) -> Self {
        Self::rgb(
            ((v >> 16) & 0xff) as f32 / 255.0,
            ((v >> 8) & 0xff) as f32 / 255.0,
            (v & 0xff) as f32 / 255.0,
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
}

/// Sammelt Formen eines Frames in Weltkoordinaten.
pub struct ShapeBatch {
    pub(crate) geometry: VertexBuffers<Vertex, u32>,
    fill: FillTessellator,
    stroke: StrokeTessellator,
    /// Toleranz der Tessellierung in Welteinheiten (kleiner = feiner).
    pub tolerance: f32,
}

impl std::fmt::Debug for ShapeBatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShapeBatch")
            .field("vertices", &self.geometry.vertices.len())
            .field("indices", &self.geometry.indices.len())
            .finish_non_exhaustive()
    }
}

impl Default for ShapeBatch {
    fn default() -> Self {
        Self {
            geometry: VertexBuffers::new(),
            fill: FillTessellator::new(),
            stroke: StrokeTessellator::new(),
            tolerance: 0.25,
        }
    }
}

impl ShapeBatch {
    pub fn clear(&mut self) {
        self.geometry.vertices.clear();
        self.geometry.indices.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.geometry.indices.is_empty()
    }

    /// Achsenparalleles Rechteck ohne Tessellierung (für viele Tiles).
    ///
    /// # Panics
    /// Bei mehr als `u32::MAX` Vertices in einem Batch.
    pub fn fill_rect(&mut self, min: Vec2, max: Vec2, color: Color) {
        let base = u32::try_from(self.geometry.vertices.len()).expect("zu viele Vertices");
        let c = color.0;
        self.geometry.vertices.extend_from_slice(&[
            Vertex {
                pos: [min.x, min.y],
                color: c,
            },
            Vertex {
                pos: [max.x, min.y],
                color: c,
            },
            Vertex {
                pos: [max.x, max.y],
                color: c,
            },
            Vertex {
                pos: [min.x, max.y],
                color: c,
            },
        ]);
        self.geometry.indices.extend_from_slice(&[
            base,
            base + 1,
            base + 2,
            base,
            base + 2,
            base + 3,
        ]);
    }

    pub fn fill_circle(&mut self, center: Vec2, radius: f32, color: Color) {
        let options = FillOptions::tolerance(self.tolerance);
        let c = color.0;
        // Tessellierung einfacher Formen kann nicht fehlschlagen
        let _ = self.fill.tessellate_circle(
            point(center.x, center.y),
            radius,
            &options,
            &mut BuffersBuilder::new(&mut self.geometry, |v: FillVertex| Vertex {
                pos: v.position().to_array(),
                color: c,
            }),
        );
    }

    /// Geschlossenes, gefülltes Polygon.
    pub fn fill_polygon(&mut self, points: &[Vec2], color: Color) {
        let Some(path) = polyline(points, true) else {
            return;
        };
        let c = color.0;
        let _ = self.fill.tessellate_path(
            &path,
            &FillOptions::tolerance(self.tolerance),
            &mut BuffersBuilder::new(&mut self.geometry, |v: FillVertex| Vertex {
                pos: v.position().to_array(),
                color: c,
            }),
        );
    }

    /// Linienzug mit runden Enden.
    pub fn stroke_polyline(&mut self, points: &[Vec2], width: f32, color: Color) {
        let Some(path) = polyline(points, false) else {
            return;
        };
        let c = color.0;
        let options = StrokeOptions::tolerance(self.tolerance)
            .with_line_width(width)
            .with_line_cap(lyon::tessellation::LineCap::Round);
        let _ = self.stroke.tessellate_path(
            &path,
            &options,
            &mut BuffersBuilder::new(&mut self.geometry, |v: StrokeVertex| Vertex {
                pos: v.position().to_array(),
                color: c,
            }),
        );
    }

    pub fn stroke_line(&mut self, from: Vec2, to: Vec2, width: f32, color: Color) {
        self.stroke_polyline(&[from, to], width, color);
    }

    pub fn stroke_circle(&mut self, center: Vec2, radius: f32, width: f32, color: Color) {
        let c = color.0;
        let _ = self.stroke.tessellate_circle(
            point(center.x, center.y),
            radius,
            &StrokeOptions::tolerance(self.tolerance).with_line_width(width),
            &mut BuffersBuilder::new(&mut self.geometry, |v: StrokeVertex| Vertex {
                pos: v.position().to_array(),
                color: c,
            }),
        );
    }
}

fn polyline(points: &[Vec2], close: bool) -> Option<Path> {
    let (first, rest) = points.split_first()?;
    let mut b = Path::builder();
    b.begin(point(first.x, first.y));
    for p in rest {
        b.line_to(point(p.x, p.y));
    }
    b.end(close);
    Some(b.build())
}

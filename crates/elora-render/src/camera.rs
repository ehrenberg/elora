//! Camera and view area (E-044, E-045).

use elora_sim::Vec2;

/// Settings for the visible section of the world. Initial values as in the original
/// (`CalcScreenParams(1150*1000, 1500, 1050, …)`); adjustable as tuning controls (E-045).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewSettings {
    /// Visible area in world units².
    pub area: f32,
    /// Maximum visible width.
    pub max_width: f32,
    /// Maximum visible height.
    pub max_height: f32,
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self {
            area: 1150.0 * 1000.0,
            max_width: 1500.0,
            max_height: 1050.0,
        }
    }
}

impl ViewSettings {
    /// Width and height of the visible area for an aspect ratio (width / height).
    pub fn view_size(&self, aspect: f32) -> Vec2 {
        let f = self.area.sqrt() / aspect.sqrt();
        let (mut w, mut h) = (f * aspect, f);
        if w > self.max_width {
            w = self.max_width;
            h = w / aspect;
        }
        if h > self.max_height {
            h = self.max_height;
            w = h * aspect;
        }
        Vec2::new(w, h)
    }
}

/// Visible section of the world for one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// Centre in world coordinates.
    pub center: Vec2,
    /// Size of the section in world units.
    pub size: Vec2,
}

impl Camera {
    pub fn new(center: Vec2, settings: &ViewSettings, aspect: f32) -> Self {
        Self {
            center,
            size: settings.view_size(aspect),
        }
    }

    /// Zoom by `factor` (> 1 = closer, less visible). Presentation only.
    #[must_use]
    pub fn zoomed(self, factor: f32) -> Self {
        Self {
            size: self.size / factor.max(0.05),
            ..self
        }
    }

    /// Top left corner.
    pub fn top_left(&self) -> Vec2 {
        self.center - self.size * 0.5
    }

    /// Screen point (pixels) → world point.
    pub fn screen_to_world(&self, screen: Vec2, screen_size: Vec2) -> Vec2 {
        let rel = Vec2::new(screen.x / screen_size.x, screen.y / screen_size.y);
        self.top_left() + Vec2::new(rel.x * self.size.x, rel.y * self.size.y)
    }

    /// World units per screen pixel.
    pub fn units_per_pixel(&self, screen_size: Vec2) -> f32 {
        self.size.x / screen_size.x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_area_matches_original_for_4_3() {
        let s = ViewSettings::default().view_size(4.0 / 3.0);
        assert!((s.x * s.y - 1_150_000.0).abs() < 1.0);
    }

    #[test]
    fn zoom_keeps_center() {
        let c = Camera::new(Vec2::new(100.0, 50.0), &ViewSettings::default(), 16.0 / 9.0);
        let z = c.zoomed(2.0);
        assert_eq!(z.center, c.center);
        assert!((z.size.x * 2.0 - c.size.x).abs() < 1e-3);
    }

    #[test]
    fn view_is_limited_on_wide_screens() {
        let s = ViewSettings::default().view_size(21.0 / 9.0);
        assert!((s.x - 1500.0).abs() < 0.01);
        assert!(s.y < 1050.0);
    }
}

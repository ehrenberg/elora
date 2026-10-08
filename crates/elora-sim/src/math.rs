//! Minimal 2D vector with explicit float operations (E-021).
//!
//! Deliberately without an external math library: every operation is visible here,
//! so that server and client compute bit-identically.

use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[must_use]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    /// Unit vector. For the zero vector the zero vector is returned
    /// (the original would produce `NaN` here).
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 { Self::ZERO } else { self / len }
    }

    /// Angle in radians (`atan2`).
    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }

    /// Linear interpolation between `self` and `other`.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }

    /// Closest point on the segment `a`–`b` to `target`.
    pub fn closest_point_on_segment(start: Self, end: Self, target: Self) -> Self {
        let len = start.distance(end);
        if len == 0.0 {
            return start;
        }
        let dir = (end - start).normalize();
        let t = dir.dot(target - start) / len;
        start.lerp(end, t.clamp(0.0, 1.0))
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, o: Self) {
        *self = *self + o;
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y)
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, o: Self) {
        *self = *self - o;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s)
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, s: f32) {
        *self = *self * s;
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    fn div(self, s: f32) -> Self {
        Self::new(self.x / s, self.y / s)
    }
}

impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

/// Rounds half away from zero (like `round_to_int` in the original).
pub fn round_to_int(f: f32) -> i32 {
    if f > 0.0 {
        (f + 0.5) as i32
    } else {
        (f - 0.5) as i32
    }
}

/// Adds `modifier` to `current` without exceeding the bound `[min, max]`.
/// If `current` is already beyond the bound, the value stays unchanged.
pub fn saturated_add(min: f32, max: f32, current: f32, modifier: f32) -> f32 {
    if modifier < 0.0 {
        if current < min {
            return current;
        }
        (current + modifier).max(min)
    } else {
        if current > max {
            return current;
        }
        (current + modifier).min(max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_to_int_rounds_away_from_zero() {
        assert_eq!(round_to_int(0.5), 1);
        assert_eq!(round_to_int(0.49), 0);
        assert_eq!(round_to_int(-0.5), -1);
        assert_eq!(round_to_int(-1.49), -1);
    }

    #[test]
    fn saturated_add_respects_limits() {
        assert!((saturated_add(-10.0, 10.0, 9.0, 2.0) - 10.0).abs() < f32::EPSILON);
        // already above the limit: no further acceleration, but no slowing down either
        assert!((saturated_add(-10.0, 10.0, 15.0, 2.0) - 15.0).abs() < f32::EPSILON);
        assert!((saturated_add(-10.0, 10.0, -9.0, -2.0) + 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn normalize_zero_is_zero() {
        assert_eq!(Vec2::ZERO.normalize(), Vec2::ZERO);
    }

    #[test]
    fn closest_point_clamps_to_segment() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        assert_eq!(
            Vec2::closest_point_on_segment(a, b, Vec2::new(5.0, 3.0)),
            Vec2::new(5.0, 0.0)
        );
        assert_eq!(
            Vec2::closest_point_on_segment(a, b, Vec2::new(-5.0, 3.0)),
            a
        );
        assert_eq!(
            Vec2::closest_point_on_segment(a, b, Vec2::new(15.0, 3.0)),
            b
        );
    }
}

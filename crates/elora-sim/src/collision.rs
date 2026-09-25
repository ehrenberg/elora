//! Tile-Kollision (Referenz: Teeworlds `CCollision`, E-007).

use crate::math::{Vec2, round_to_int};

/// Kantenlänge eines Tiles in Welteinheiten.
pub const TILE_SIZE: i32 = 32;

/// Kollisionsart eines Tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tile {
    #[default]
    Air,
    Solid,
    /// Wand, an der der Hook nicht greift.
    Unhookable,
    /// Tötet bei Berührung.
    Death,
}

impl Tile {
    pub fn is_solid(self) -> bool {
        matches!(self, Self::Solid | Self::Unhookable)
    }
}

/// Kollisionsraster der Welt. Alles außerhalb gilt als [`Tile::Solid`] (E-024).
#[derive(Debug, Clone)]
pub struct Collision {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
}

impl Collision {
    /// Erzeugt ein Raster aus `tiles` (zeilenweise, oben links beginnend).
    ///
    /// # Panics
    /// Wenn `tiles.len() != width * height`.
    pub fn new(width: usize, height: usize, tiles: Vec<Tile>) -> Self {
        assert_eq!(
            tiles.len(),
            width * height,
            "Tile-Anzahl passt nicht zur Rastergröße"
        );
        Self {
            width,
            height,
            tiles,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Tile an Rasterkoordinate. Außerhalb: [`Tile::Solid`].
    pub fn tile(&self, tx: i32, ty: i32) -> Tile {
        let (Ok(x), Ok(y)) = (usize::try_from(tx), usize::try_from(ty)) else {
            return Tile::Solid;
        };
        if x >= self.width || y >= self.height {
            return Tile::Solid;
        }
        self.tiles[y * self.width + x]
    }

    /// Tile an einer Weltposition.
    pub fn tile_at(&self, pos: Vec2) -> Tile {
        let x = round_to_int(pos.x).div_euclid(TILE_SIZE);
        let y = round_to_int(pos.y).div_euclid(TILE_SIZE);
        self.tile(x, y)
    }

    /// Ist die Weltposition fest (Wand oder Unhookable)?
    pub fn is_solid(&self, pos: Vec2) -> bool {
        self.tile_at(pos).is_solid()
    }

    fn test_box_with(&self, pos: Vec2, size: Vec2, check: impl Fn(Tile) -> bool) -> bool {
        let h = size * 0.5;
        [
            Vec2::new(pos.x - h.x, pos.y - h.y),
            Vec2::new(pos.x + h.x, pos.y - h.y),
            Vec2::new(pos.x - h.x, pos.y + h.y),
            Vec2::new(pos.x + h.x, pos.y + h.y),
        ]
        .into_iter()
        .any(|p| check(self.tile_at(p)))
    }

    /// Berührt eine der vier Ecken der Box eine Wand?
    pub fn test_box(&self, pos: Vec2, size: Vec2) -> bool {
        self.test_box_with(pos, size, Tile::is_solid)
    }

    /// Bewegt eine Box um `vel` in Einzelschritten (max. 1 Einheit pro Schritt),
    /// damit auch schnelle Objekte nicht durch Wände tunneln.
    ///
    /// Gibt zurück, ob dabei ein Todes-Tile berührt wurde (Todes-Box ist 2/3 so groß).
    pub fn move_box(&self, pos: &mut Vec2, vel: &mut Vec2, size: Vec2, elasticity: f32) -> bool {
        let distance = vel.length();
        let mut death = false;
        if distance <= 0.00001 {
            return death;
        }

        let steps = distance as i32;
        let fraction = 1.0 / (steps + 1) as f32;
        let mut p = *pos;
        for _ in 0..=steps {
            let mut new = p + *vel * fraction;

            if self.test_box_with(new, size * (2.0 / 3.0), |t| t == Tile::Death) {
                death = true;
            }

            if self.test_box(new, size) {
                let mut hits = 0;
                if self.test_box(Vec2::new(p.x, new.y), size) {
                    new.y = p.y;
                    vel.y *= -elasticity;
                    hits += 1;
                }
                if self.test_box(Vec2::new(new.x, p.y), size) {
                    new.x = p.x;
                    vel.x *= -elasticity;
                    hits += 1;
                }
                // echter Eckfall: keiner der Einzeltests trifft
                if hits == 0 {
                    new = p;
                    *vel *= -elasticity;
                }
            }
            p = new;
        }
        *pos = p;
        death
    }

    /// Tastet die Strecke `from`–`to` ab. Liefert beim ersten festen Punkt
    /// dessen Position und Tile-Art.
    pub fn intersect_line(&self, from: Vec2, to: Vec2) -> Option<(Vec2, Tile)> {
        let end = from.distance(to) as i32 + 1;
        let inv = 1.0 / end as f32;
        (0..=end).find_map(|i| {
            let p = from.lerp(to, i as f32 * inv);
            let tile = self.tile_at(p);
            tile.is_solid().then_some((p, tile))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 5×5-Raster mit Wand-Rahmen.
    fn boxed() -> Collision {
        let mut tiles = vec![Tile::Air; 25];
        for i in 0..5 {
            tiles[i] = Tile::Solid;
            tiles[20 + i] = Tile::Solid;
            tiles[i * 5] = Tile::Solid;
            tiles[i * 5 + 4] = Tile::Solid;
        }
        Collision::new(5, 5, tiles)
    }

    #[test]
    fn outside_is_solid() {
        let c = Collision::new(1, 1, vec![Tile::Air]);
        assert_eq!(c.tile(0, 0), Tile::Air);
        assert_eq!(c.tile(-1, 0), Tile::Solid);
        assert_eq!(c.tile(1, 0), Tile::Solid);
        assert!(c.is_solid(Vec2::new(-1.0, 5.0)));
    }

    #[test]
    fn move_box_stops_at_wall_even_when_fast() {
        let c = boxed();
        let mut pos = Vec2::new(80.0, 80.0);
        let mut vel = Vec2::new(500.0, 0.0);
        c.move_box(&mut pos, &mut vel, Vec2::new(28.0, 28.0), 0.0);
        // rechte Wand beginnt bei x = 128, Box-Halbbreite 14
        assert!(pos.x <= 128.0 - 14.0, "durch die Wand getunnelt: {pos:?}");
        assert!(pos.x > 100.0);
        assert!(vel.x.abs() < f32::EPSILON);
    }

    #[test]
    fn intersect_line_hits_first_wall() {
        let c = boxed();
        let hit = c.intersect_line(Vec2::new(80.0, 80.0), Vec2::new(80.0, -200.0));
        let (p, tile) = hit.expect("Decke muss getroffen werden");
        assert_eq!(tile, Tile::Solid);
        assert!(p.y < 32.0 && p.y > 20.0);
    }
}

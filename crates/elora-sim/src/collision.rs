//! Tile collision (reference: Teeworlds `CCollision`, E-007).

use crate::math::{Vec2, round_to_int};

/// Edge length of a tile in world units.
pub const TILE_SIZE: i32 = 32;

/// Direction of a jump pad (T-34).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JumpDir {
    Up,
    UpLeft,
    UpRight,
}

impl JumpDir {
    /// Unit vector of the launch direction (y pointing down).
    pub fn vector(self) -> Vec2 {
        let d = std::f32::consts::FRAC_1_SQRT_2;
        match self {
            Self::Up => Vec2::new(0.0, -1.0),
            Self::UpLeft => Vec2::new(-d, -d),
            Self::UpRight => Vec2::new(d, -d),
        }
    }
}

/// Running direction of an accelerator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BeltDir {
    Left,
    Right,
}

impl BeltDir {
    pub fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }
}

/// Collision kind of a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tile {
    #[default]
    Air,
    Solid,
    /// Wall the hook does not grab.
    Unhookable,
    /// Kills on contact.
    Death,
    /// Walkable from above, passable from below and the sides; hook, grenades and
    /// laser fly through (T-36, E-140). With "down" you fall through (E-141).
    Platform,
    /// Solid but slippery (T-31, T-32).
    Ice,
    /// Solid; launches a character standing on it with T-33 in direction T-34.
    JumpPad(JumpDir),
    /// Solid; carries a character standing on it like a conveyor belt (T-35).
    Conveyor(BeltDir),
    /// Climbing wall: solid, not hookable; with ice grip you can cling to it (E-228).
    Climb,
    /// Crumbling floor: solid and hookable; breaks when stomped (E-230).
    Crumble,
    /// Hook point in the air (hook blossom, R2-M2.1): the hook grabs in the center, everything
    /// else flies and walks through. Can be temporarily wilted ([`Collision::hook_wilt`]).
    HookPoint,
    /// Quicksand (R2-M2.3, E-318): not solid; characters slowly sink in and walk
    /// slower, jumping frees them, sunk in deep: small damage and back to the edge.
    Quicksand,
    /// Thin ice (R2-M2.4, E-343): solid, not hookable, not slippery; breaks after standing
    /// briefly (A-36) or immediately when stomped, and grows back after a while (A-37).
    ThinIce,
    /// Ice water (R2-M2.4): not solid; whoever falls in takes small damage (A-38) and
    /// returns to the edge.
    IceWater,
}

/// Characters of the tile kinds in the text format and in recordings (E-024, M6.1).
const TILE_CHARS: [(char, Tile); 17] = [
    ('.', Tile::Air),
    ('#', Tile::Solid),
    ('%', Tile::Unhookable),
    ('^', Tile::Death),
    ('=', Tile::Platform),
    ('~', Tile::Ice),
    ('!', Tile::JumpPad(JumpDir::Up)),
    ('\\', Tile::JumpPad(JumpDir::UpLeft)),
    ('/', Tile::JumpPad(JumpDir::UpRight)),
    ('<', Tile::Conveyor(BeltDir::Left)),
    ('>', Tile::Conveyor(BeltDir::Right)),
    ('|', Tile::Climb),
    (':', Tile::Crumble),
    ('*', Tile::HookPoint),
    ('&', Tile::Quicksand),
    ('-', Tile::ThinIce),
    ('+', Tile::IceWater),
];

impl Tile {
    /// Character in the text format (`.` air, `#` wall, `%` unhookable, `^` death, `=` platform,
    /// `~` ice, `!` `\` `/` jump pad up/diagonal left/diagonal right, `<` `>` accelerator,
    /// `|` climbing wall, `:` crumbling floor, `*` hook point, `&` quicksand, `-` thin ice,
    /// `+` ice water).
    pub fn to_char(self) -> char {
        TILE_CHARS
            .iter()
            .find(|(_, t)| *t == self)
            .map_or('.', |(c, _)| *c)
    }

    pub fn from_char(c: char) -> Option<Self> {
        TILE_CHARS.iter().find(|(ch, _)| *ch == c).map(|(_, t)| *t)
    }

    /// Solid for characters, hook, grenades and laser (platforms do not count).
    pub fn is_solid(self) -> bool {
        matches!(
            self,
            Self::Solid
                | Self::Unhookable
                | Self::Ice
                | Self::JumpPad(_)
                | Self::Conveyor(_)
                | Self::Climb
                | Self::Crumble
                | Self::ThinIce
        )
    }

    /// Does the hook grab on this tile (solid tiles only)?
    pub fn is_hookable(self) -> bool {
        self.is_solid() && !matches!(self, Self::Unhookable | Self::Climb | Self::ThinIce)
    }
}

/// Hit of a line test.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineHit {
    /// First solid point.
    pub pos: Vec2,
    /// Last free point before it.
    pub before: Vec2,
    pub tile: Tile,
}

/// Collision grid of the world. Everything outside counts as [`Tile::Solid`] (E-024).
#[derive(Debug, Clone)]
pub struct Collision {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
    /// Wilted hook points (guardian enraged, E-298): `Some(gerade)` = hook points in columns with
    /// `tx % 2 == 0` (or odd) currently do not grab; `None` = all fresh.
    pub hook_wilt: Option<bool>,
    /// Weather of this tick (R2-W1, adventure only): wind with gusts (−1..1) and wetness of the
    /// ground (0..1); 0 in multiplayer.
    pub wind: f32,
    pub wet: f32,
}

impl Collision {
    /// Creates a grid from `tiles` (row by row, starting top left).
    ///
    /// # Panics
    /// If `tiles.len() != width * height`.
    pub fn new(width: usize, height: usize, tiles: Vec<Tile>) -> Self {
        assert_eq!(
            tiles.len(),
            width * height,
            "tile count does not match the grid size"
        );
        Self {
            width,
            height,
            tiles,
            hook_wilt: None,
            wind: 0.0,
            wet: 0.0,
        }
    }

    /// Does the hook point in column `tx` currently grab (not wilted)?
    pub fn hook_point_active(&self, tx: i32) -> bool {
        self.hook_wilt
            .is_none_or(|even| (tx.rem_euclid(2) == 0) != even)
    }

    /// Center of the first grabbing hook point on the segment `from`–`to`.
    pub fn intersect_hook_point(&self, from: Vec2, to: Vec2) -> Option<Vec2> {
        let end = from.distance(to) as i32 + 1;
        let inv = 1.0 / end as f32;
        for i in 0..=end {
            let p = from.lerp(to, i as f32 * inv);
            let (tx, ty) = (
                round_to_int(p.x).div_euclid(TILE_SIZE),
                round_to_int(p.y).div_euclid(TILE_SIZE),
            );
            if self.tile(tx, ty) == Tile::HookPoint && self.hook_point_active(tx) {
                let ts = TILE_SIZE as f32;
                return Some(Vec2::new(
                    tx as f32 * ts + ts / 2.0,
                    ty as f32 * ts + ts / 2.0,
                ));
            }
        }
        None
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Tile at a grid coordinate. Outside: [`Tile::Solid`].
    pub fn tile(&self, tx: i32, ty: i32) -> Tile {
        let (Ok(x), Ok(y)) = (usize::try_from(tx), usize::try_from(ty)) else {
            return Tile::Solid;
        };
        if x >= self.width || y >= self.height {
            return Tile::Solid;
        }
        self.tiles[y * self.width + x]
    }

    /// Sets a tile (e.g. broken crumbling floor); no effect outside the grid.
    pub fn set_tile(&mut self, tx: i32, ty: i32, tile: Tile) {
        let (Ok(x), Ok(y)) = (usize::try_from(tx), usize::try_from(ty)) else {
            return;
        };
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile;
        }
    }

    /// Tile at a world position.
    pub fn tile_at(&self, pos: Vec2) -> Tile {
        let x = round_to_int(pos.x).div_euclid(TILE_SIZE);
        let y = round_to_int(pos.y).div_euclid(TILE_SIZE);
        self.tile(x, y)
    }

    /// Is the world position solid (wall or unhookable)?
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

    /// Does one of the four corners of the box touch a wall?
    pub fn test_box(&self, pos: Vec2, size: Vec2) -> bool {
        self.test_box_with(pos, size, Tile::is_solid)
    }

    /// Does a box with bottom edge `prev → new` (downwards) land on a platform?
    /// It must have been above the platform row before (passable from below). Rows are
    /// found by rounding like [`Collision::test_box`], so the box rests at the same height
    /// as on solid ground and can walk on onto a solid tile next to the platform.
    fn lands_on_platform(&self, x: f32, half_w: f32, prev_bottom: f32, new_bottom: f32) -> bool {
        if new_bottom <= prev_bottom {
            return false;
        }
        let ty = round_to_int(new_bottom).div_euclid(TILE_SIZE);
        if round_to_int(prev_bottom).div_euclid(TILE_SIZE) >= ty {
            return false;
        }
        [x - half_w, x + half_w]
            .into_iter()
            .any(|px| self.tile(round_to_int(px).div_euclid(TILE_SIZE), ty) == Tile::Platform)
    }

    /// Moves a box by `vel` in single steps (max. 1 unit per step),
    /// so that even fast objects do not tunnel through walls.
    ///
    /// Returns whether a death tile was touched (the death box is 2/3 as large).
    pub fn move_box(&self, pos: &mut Vec2, vel: &mut Vec2, size: Vec2, elasticity: f32) -> bool {
        self.move_box_platforms(pos, vel, size, elasticity, false)
    }

    /// Like [`Collision::move_box`]; with `platforms` the box lands on platforms
    /// (characters that are not currently falling through).
    pub fn move_box_platforms(
        &self,
        pos: &mut Vec2,
        vel: &mut Vec2,
        size: Vec2,
        elasticity: f32,
        platforms: bool,
    ) -> bool {
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
                // real corner case: none of the single tests hits
                if hits == 0 {
                    new = p;
                    *vel *= -elasticity;
                }
            }
            if platforms
                && self.lands_on_platform(
                    new.x,
                    size.x * 0.5,
                    p.y + size.y * 0.5,
                    new.y + size.y * 0.5,
                )
            {
                new.y = p.y;
                vel.y *= -elasticity;
            }
            p = new;
        }
        *pos = p;
        death
    }

    /// Moves a point by `vel`; at walls the velocity is mirrored with
    /// `elasticity`. Returns the number of bounces.
    pub fn move_point(&self, pos: &mut Vec2, vel: &mut Vec2, elasticity: f32) -> u32 {
        let p = *pos;
        let v = *vel;
        if !self.is_solid(p + v) {
            *pos = p + v;
            return 0;
        }
        let mut bounces = 0;
        if self.is_solid(Vec2::new(p.x + v.x, p.y)) {
            vel.x *= -elasticity;
            bounces += 1;
        }
        if self.is_solid(Vec2::new(p.x, p.y + v.y)) {
            vel.y *= -elasticity;
            bounces += 1;
        }
        if bounces == 0 {
            *vel *= -elasticity;
        }
        bounces
    }

    /// Scans the segment `from`–`to`. Returns the position and tile kind
    /// of the first solid point.
    pub fn intersect_line(&self, from: Vec2, to: Vec2) -> Option<(Vec2, Tile)> {
        self.intersect_line_detail(from, to)
            .map(|h| (h.pos, h.tile))
    }

    /// Like [`Self::intersect_line`], additionally with the last free point before it.
    pub fn intersect_line_detail(&self, from: Vec2, to: Vec2) -> Option<LineHit> {
        let end = from.distance(to) as i32 + 1;
        let inv = 1.0 / end as f32;
        let mut last = from;
        for i in 0..=end {
            let p = from.lerp(to, i as f32 * inv);
            let tile = self.tile_at(p);
            if tile.is_solid() {
                return Some(LineHit {
                    pos: p,
                    before: last,
                    tile,
                });
            }
            last = p;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 5×5 grid with a wall frame.
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
        // right wall starts at x = 128, box half-width 14
        assert!(pos.x <= 128.0 - 14.0, "tunneled through the wall: {pos:?}");
        assert!(pos.x > 100.0);
        assert!(vel.x.abs() < f32::EPSILON);
    }

    #[test]
    fn intersect_line_hits_first_wall() {
        let c = boxed();
        let hit = c.intersect_line(Vec2::new(80.0, 80.0), Vec2::new(80.0, -200.0));
        let (p, tile) = hit.expect("ceiling must be hit");
        assert_eq!(tile, Tile::Solid);
        assert!(p.y < 32.0 && p.y > 20.0);
    }
}

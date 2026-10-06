//! Tile-Kollision (Referenz: Teeworlds `CCollision`, E-007).

use crate::math::{Vec2, round_to_int};

/// Kantenlänge eines Tiles in Welteinheiten.
pub const TILE_SIZE: i32 = 32;

/// Richtung eines Sprungfelds (T-34).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JumpDir {
    Up,
    UpLeft,
    UpRight,
}

impl JumpDir {
    /// Einheitsvektor der Wurfrichtung (y nach unten).
    pub fn vector(self) -> Vec2 {
        let d = std::f32::consts::FRAC_1_SQRT_2;
        match self {
            Self::Up => Vec2::new(0.0, -1.0),
            Self::UpLeft => Vec2::new(-d, -d),
            Self::UpRight => Vec2::new(d, -d),
        }
    }
}

/// Laufrichtung eines Beschleunigers.
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
    /// Von oben begehbar, von unten und seitlich durchlässig; Hook, Granaten und
    /// Laser fliegen hindurch (T-36, E-140). Mit „Runter“ fällt man hindurch (E-141).
    Platform,
    /// Fest, aber rutschig (T-31, T-32).
    Ice,
    /// Fest; wirft eine darauf stehende Figur mit T-33 in Richtung T-34.
    JumpPad(JumpDir),
    /// Fest; trägt eine darauf stehende Figur wie ein Laufband (T-35).
    Conveyor(BeltDir),
    /// Kletterwand: fest, nicht hookbar; mit Eisgriff kann man daran haften (E-228).
    Climb,
    /// Bröckelboden: fest und hookbar; bricht beim Stampfen (E-230).
    Crumble,
    /// Hookpunkt in der Luft (Hook-Blüte, R2-M2.1): der Hook greift in der Mitte, alles andere
    /// fliegt und läuft hindurch. Kann zeitweise welk sein ([`Collision::hook_wilt`]).
    HookPoint,
    /// Treibsand (R2-M2.3, E-318): nicht fest; Figuren sinken langsam ein und laufen
    /// langsamer, Springen befreit, tief eingesunken kleiner Schaden und zurück an den Rand.
    Quicksand,
    /// Dünnes Eis (R2-M2.4, E-343): fest, nicht hookbar, nicht rutschig; bricht nach kurzem
    /// Stehen (A-36) oder sofort beim Stampfen und wächst nach einer Weile nach (A-37).
    ThinIce,
    /// Eiswasser (R2-M2.4): nicht fest; wer hineinfällt, nimmt kleinen Schaden (A-38) und
    /// kommt zurück an den Rand.
    IceWater,
}

/// Zeichen der Tile-Arten im Textformat und in Aufzeichnungen (E-024, M6.1).
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
    /// Zeichen im Textformat (`.` Luft, `#` Wand, `%` unhookable, `^` Tod, `=` Plattform,
    /// `~` Eis, `!` `\` `/` Sprungfeld hoch/schräg links/schräg rechts, `<` `>` Beschleuniger,
    /// `|` Kletterwand, `:` Bröckelboden, `*` Hookpunkt, `&` Treibsand, `-` dünnes Eis,
    /// `+` Eiswasser).
    pub fn to_char(self) -> char {
        TILE_CHARS
            .iter()
            .find(|(_, t)| *t == self)
            .map_or('.', |(c, _)| *c)
    }

    pub fn from_char(c: char) -> Option<Self> {
        TILE_CHARS.iter().find(|(ch, _)| *ch == c).map(|(_, t)| *t)
    }

    /// Fest für Figuren, Hook, Granaten und Laser (Plattformen zählen nicht).
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

    /// Greift der Hook an diesem Tile (nur feste Tiles)?
    pub fn is_hookable(self) -> bool {
        self.is_solid() && !matches!(self, Self::Unhookable | Self::Climb | Self::ThinIce)
    }
}

/// Treffer eines Linien-Tests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineHit {
    /// Erster fester Punkt.
    pub pos: Vec2,
    /// Letzter freier Punkt davor.
    pub before: Vec2,
    pub tile: Tile,
}

/// Kollisionsraster der Welt. Alles außerhalb gilt als [`Tile::Solid`] (E-024).
#[derive(Debug, Clone)]
pub struct Collision {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
    /// Welke Hookpunkte (Hüter wütend, E-298): `Some(gerade)` = Hookpunkte in Spalten mit
    /// `tx % 2 == 0` (bzw. ungerade) greifen gerade nicht; `None` = alle frisch.
    pub hook_wilt: Option<bool>,
    /// Wetter dieses Ticks (R2-W1, nur Abenteuer): Wind mit Böen (−1..1) und Nässe des Bodens
    /// (0..1); im Mehrspieler 0.
    pub wind: f32,
    pub wet: f32,
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
            hook_wilt: None,
            wind: 0.0,
            wet: 0.0,
        }
    }

    /// Greift der Hookpunkt in Spalte `tx` gerade (nicht welk)?
    pub fn hook_point_active(&self, tx: i32) -> bool {
        self.hook_wilt
            .is_none_or(|even| (tx.rem_euclid(2) == 0) != even)
    }

    /// Mitte des ersten greifenden Hookpunkts auf der Strecke `from`–`to`.
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

    /// Setzt ein Tile (z. B. zerbrochener Bröckelboden); außerhalb des Rasters ohne Wirkung.
    pub fn set_tile(&mut self, tx: i32, ty: i32, tile: Tile) {
        let (Ok(x), Ok(y)) = (usize::try_from(tx), usize::try_from(ty)) else {
            return;
        };
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile;
        }
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

    /// Landet eine Box mit Unterkante `prev → new` (abwärts) auf einer Plattform?
    /// Sie muss vorher auf oder über der Oberkante gewesen sein (von unten durchlässig).
    fn lands_on_platform(&self, x: f32, half_w: f32, prev_bottom: f32, new_bottom: f32) -> bool {
        if new_bottom <= prev_bottom {
            return false;
        }
        let ty = round_to_int(new_bottom).div_euclid(TILE_SIZE);
        #[allow(clippy::cast_precision_loss)]
        let top = (ty * TILE_SIZE) as f32;
        if prev_bottom > top + 0.01 || new_bottom <= top {
            return false;
        }
        [x - half_w, x + half_w]
            .into_iter()
            .any(|px| self.tile(round_to_int(px).div_euclid(TILE_SIZE), ty) == Tile::Platform)
    }

    /// Bewegt eine Box um `vel` in Einzelschritten (max. 1 Einheit pro Schritt),
    /// damit auch schnelle Objekte nicht durch Wände tunneln.
    ///
    /// Gibt zurück, ob dabei ein Todes-Tile berührt wurde (Todes-Box ist 2/3 so groß).
    pub fn move_box(&self, pos: &mut Vec2, vel: &mut Vec2, size: Vec2, elasticity: f32) -> bool {
        self.move_box_platforms(pos, vel, size, elasticity, false)
    }

    /// Wie [`Collision::move_box`]; mit `platforms` landet die Box auf Plattformen
    /// (Figuren, die nicht gerade durchfallen).
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
                // echter Eckfall: keiner der Einzeltests trifft
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

    /// Bewegt einen Punkt um `vel`; an Wänden wird die Geschwindigkeit mit
    /// `elasticity` gespiegelt. Liefert die Anzahl der Abpraller.
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

    /// Tastet die Strecke `from`–`to` ab. Liefert beim ersten festen Punkt
    /// dessen Position und Tile-Art.
    pub fn intersect_line(&self, from: Vec2, to: Vec2) -> Option<(Vec2, Tile)> {
        self.intersect_line_detail(from, to)
            .map(|h| (h.pos, h.tile))
    }

    /// Wie [`Self::intersect_line`], zusätzlich mit dem letzten freien Punkt davor.
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

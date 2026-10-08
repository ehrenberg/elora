//! Creatures in the adventure (R2-M1, A1.2, E-232 to E-238): enemies with behavior and health,
//! their projectiles and loot lying around.
//!
//! The kinds come as data (`assets/adventure/creatures.toml`); the simulation only knows
//! the behavior patterns. Everything is deterministic like the rest of the world.

use crate::math::Vec2;

/// An enemy kind.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreatureKind {
    /// Key for maps, graphics and translation (e.g. `stachelkaefer`).
    pub name: String,
    /// Collision box (width, height) in units; origin in the center.
    pub size: [f32; 2],
    pub health: i32,
    /// Damage on contact (E-232).
    #[cfg_attr(feature = "serde", serde(default))]
    pub touch_damage: i32,
    /// Small: the pull hook pulls it towards Elora (E-233).
    #[cfg_attr(feature = "serde", serde(default))]
    pub small: bool,
    /// Boss or special enemy: stays defeated (E-235, evaluated by the adventure).
    #[cfg_attr(feature = "serde", serde(default))]
    pub boss: bool,
    /// Experience for defeating it (evaluated by the adventure).
    #[cfg_attr(feature = "serde", serde(default))]
    pub xp: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub loot: Vec<LootEntry>,
    /// Contact triggers the colorful rush (this many ms, E-311): Elora walks slower.
    #[cfg_attr(feature = "serde", serde(default))]
    pub daze_ms: u32,
    /// Contact freezes Elora for this many ms (frost ghost, D-M24-07).
    #[cfg_attr(feature = "serde", serde(default))]
    pub freeze_ms: u32,
    /// Shell (sand crab, E-317): hits from the side or from below bounce off, only hits from
    /// above (strike, grenade on top) and stomp take effect.
    #[cfg_attr(feature = "serde", serde(default))]
    pub armor: bool,
    pub behavior: Behavior,
}

impl CreatureKind {
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.size[0], self.size[1])
    }

    /// Radius for hit checks (half the larger side).
    pub fn radius(&self) -> f32 {
        self.size[0].max(self.size[1]) / 2.0
    }
}

/// Behavior pattern.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
pub enum Behavior {
    /// Walks back and forth, turns at walls and (if desired) at edges.
    Walker { speed: f32, turn_at_edges: bool },
    /// Waits on the ground and leaps at Elora as soon as she is in sight.
    Hopper {
        wait_ms: u32,
        jump_x: f32,
        jump_y: f32,
        sight: f32,
    },
    /// Stands still and shoots at Elora when she is in range and in sight.
    Turret {
        interval_ms: u32,
        range: f32,
        shot_speed: f32,
        shot_damage: i32,
        /// Throws in an arc (gravity) instead of shooting straight (squirrel pirate, R2-M2.2).
        #[cfg_attr(feature = "serde", serde(default))]
        lob: bool,
    },
    /// Hovers around the start point and chases Elora when in sight. With `hover` it stays
    /// this high above her and drops a spark every `drop_ms` that glows on the ground for
    /// `glow_ms` (spark moth, R2-M2.3).
    Flyer {
        speed: f32,
        sight: f32,
        #[cfg_attr(feature = "serde", serde(default))]
        hover: f32,
        #[cfg_attr(feature = "serde", serde(default))]
        drop_ms: u32,
        #[cfg_attr(feature = "serde", serde(default))]
        drop_damage: i32,
        #[cfg_attr(feature = "serde", serde(default))]
        glow_ms: u32,
    },
    /// Guardian from the air (R2-M2.1, E-298, E-299): circles above the start point and drops
    /// projectiles, takes aim at Elora and dives down; afterwards it lies dazed on the
    /// ground – **only then vulnerable** – and rises again. From `enrage_at` (share of
    /// health) faster and two dives in a row, from `summon_at` it calls helpers.
    Diver(Box<DiverDef>),
    /// Sits in the ground and shoots up when Elora is closer than `sight` (root snake,
    /// R2-M2.2); stays out for `out_ms` – **only then vulnerable and dangerous** – and
    /// afterwards waits hidden for at least `hide_ms`.
    /// When emerging it grows out of the ground for `rise_ms` (still harmless).
    Burrower {
        sight: f32,
        out_ms: u32,
        hide_ms: u32,
        #[cfg_attr(feature = "serde", serde(default))]
        rise_ms: u32,
    },
    /// Guardian on the ground (Root Warden, R2-M2.2, E-307): stands, thrusts roots out of the
    /// ground (with a warning), cores in the bark can be pulled loose with the hook in a tug of
    /// war – **only then vulnerable**. From `enrage_at` faster and with root walls, at the last
    /// core roots at two spots.
    Warden(Box<WardenDef>),
    /// Companion (mushroom child, E-308): follows Elora on the ground, jumps over steps, waits
    /// at gaps and hazards; invulnerable and harmless.
    Follower { speed: f32, jump: f32 },
    /// Guardian in the sand (Sand Serpent, R2-M2.3, E-316): moves towards Elora as a sand trail,
    /// the sand quakes, then it shoots out in an arc and lies dazed on the ground after landing –
    /// **vulnerable only in the arc and while dazed**. From `enrage_at` faster and quicksand in
    /// the basin, from `double_at` two arcs in a row. Stomp hits twice.
    Serpent(Box<SerpentDef>),
    /// Travels under the sand (only the sand trail is visible) towards Elora, announces itself
    /// for `warn_ms` and leaps at her in an arc (dune worm, R2-M2.3); after landing
    /// it dives in and rests for `rest_ms`. **Vulnerable and dangerous only mid-leap.**
    Leaper {
        sight: f32,
        speed: f32,
        warn_ms: u32,
        jump_x: f32,
        jump_y: f32,
        rest_ms: u32,
    },
    /// Icicle on the ceiling (R2-M2.4, E-343): hangs harmlessly, trembles for `warn_ms` as soon
    /// as Elora is below it (horizontally up to `sight`, vertically up to `reach`, clear line of
    /// sight), then falls and shatters on the ground or on Elora. **Dangerous only while falling.**
    Icicle {
        sight: f32,
        reach: f32,
        warn_ms: u32,
    },
    /// Snow chunk of an avalanche (R2-M2.4): rolls downhill with `speed` in its facing direction
    /// and bursts at a wall, on Elora or after `life_ms`.
    Roller { speed: f32, life_ms: u32 },
    /// Snowball seal (R2-M2.4): slides closer on its belly with `speed` until Elora is closer
    /// than `range`, rears up and throws a snowball in an arc every `interval_ms`.
    Seal {
        sight: f32,
        speed: f32,
        range: f32,
        interval_ms: u32,
        shot_speed: f32,
        shot_damage: i32,
    },
    /// Ice-spike bat (R2-M2.4): hangs asleep from the ceiling, swoops down on Elora with `speed`
    /// as soon as she is below it (like the icicle), and flutters back.
    Bat { sight: f32, reach: f32, speed: f32 },
    /// Frost ghost (R2-M2.4, D-M24-07): floats through walls towards Elora with `speed`; after
    /// a touch (freezing for the kind's `freeze_ms`) it retreats for `flee_ms`.
    Ghost {
        sight: f32,
        speed: f32,
        flee_ms: u32,
    },
    /// Guardian of Frostspitzen (Ice Queen Kristella, R2-M2.4, E-341): floats above the
    /// hall and sends frost waves across the floor (fresh frost hurts, whoever hangs on the
    /// wall or jumps stays unharmed); after `waves` waves she sinks down exhausted –
    /// **only then vulnerable**. From `enrage_at` waves from both sides and icicles, from
    /// `storm_at` a blizzard in the hall.
    Queen(Box<QueenDef>),
}

/// Values of the guardian of Frostspitzen ([`Behavior::Queen`]).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QueenDef {
    /// Wakes up as soon as Elora is this close.
    pub sight: f32,
    /// Width of the hall (waves run from edge to edge, center = start point).
    pub width: f32,
    /// Hovering between two waves.
    pub hover_ms: u32,
    /// Waves until exhaustion.
    pub waves: u32,
    /// Speed of the wave front (units/tick) and length of the fresh frost behind it.
    pub wave_speed: f32,
    pub fresh_len: f32,
    pub wave_damage: i32,
    /// Exhausted on the ground: vulnerable.
    pub stun_ms: u32,
    /// Enraged from this share of health: waves alternating from both sides, in between
    /// `icicles` icicles above Elora; 0 = never.
    #[cfg_attr(feature = "serde", serde(default))]
    pub enrage_at: f32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub icicles: u32,
    /// Blizzard in the hall from this share of health (the session sets the weather).
    #[cfg_attr(feature = "serde", serde(default))]
    pub storm_at: f32,
    /// After this many hits during one exhaustion she rises again immediately (0 = never).
    #[cfg_attr(feature = "serde", serde(default))]
    pub open_hits: u32,
}

/// Values of the guardian from the air ([`Behavior::Diver`]).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiverDef {
    /// Wakes up as soon as Elora is this close.
    pub sight: f32,
    /// Circle above the start point: half width and height, speed (units/tick).
    pub circle: [f32; 2],
    pub speed: f32,
    /// Circle this long before taking aim.
    pub circle_ms: u32,
    /// Warning before the dive (stands still and trembles).
    pub aim_ms: u32,
    pub dive_speed: f32,
    /// Dazed on the ground.
    pub stun_ms: u32,
    /// Projectiles while circling (fall downwards); 0 = none.
    #[cfg_attr(feature = "serde", serde(default))]
    pub drop_ms: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub drop_speed: f32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub drop_damage: i32,
    /// Enraged from this share of health (1.35× speed, double dive); 0 = never.
    #[cfg_attr(feature = "serde", serde(default))]
    pub enrage_at: f32,
    /// From this share of health it calls `summon` (kind), at most `summon_max` at once.
    #[cfg_attr(feature = "serde", serde(default))]
    pub summon_at: f32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub summon: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub summon_max: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub summon_ms: u32,
}

/// Values of the guardian on the ground ([`Behavior::Warden`]).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WardenDef {
    /// Wakes up as soon as Elora is this close.
    pub sight: f32,
    /// Interval of the root thrusts and warning time before them (ground quakes).
    pub attack_ms: u32,
    pub warn_ms: u32,
    /// Root thrust: width and height of the hit zone, damage.
    pub spike_width: f32,
    pub spike_height: f32,
    pub spike_damage: i32,
    /// Pull away from the warden this long until a core comes loose; it is then open this long.
    pub pull_ms: u32,
    pub open_ms: u32,
    /// Cores in the bark (grow back once all are pulled).
    pub cores: u32,
    /// Enraged from this share of health: faster, root walls; 0 = never.
    #[cfg_attr(feature = "serde", serde(default))]
    pub enrage_at: f32,
    /// Root wall: every `wall_every_ms`, stands for `wall_ms`, this many tiles high.
    #[cfg_attr(feature = "serde", serde(default))]
    pub wall_every_ms: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub wall_ms: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub wall_height: u32,
}

/// Values of the guardian in the sand ([`Behavior::Serpent`]).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SerpentDef {
    /// Wakes up as soon as Elora is this close.
    pub sight: f32,
    /// Speed of the sand trail (units/tick) and minimum time under the sand.
    pub speed: f32,
    pub trail_ms: u32,
    /// Sand quakes this long before it shoots out.
    pub warn_ms: u32,
    /// Leap: horizontal (towards the center of the basin) and upwards.
    pub jump_x: f32,
    pub jump_y: f32,
    /// Dazed on the ground after landing.
    pub stun_ms: u32,
    /// Enraged from this share of health: faster, quicksand in the basin; 0 = never.
    #[cfg_attr(feature = "serde", serde(default))]
    pub enrage_at: f32,
    /// Two arcs in a row from this share of health; 0 = never.
    #[cfg_attr(feature = "serde", serde(default))]
    pub double_at: f32,
    /// Quicksand: every `sand_every_ms` for `sand_ms`, this many tiles wide.
    #[cfg_attr(feature = "serde", serde(default))]
    pub sand_every_ms: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub sand_ms: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub sand_width: u32,
    /// After this many hits during one opening it dives immediately (0 = never).
    #[cfg_attr(feature = "serde", serde(default))]
    pub open_hits: u32,
    /// Landing flings sand: damage in the radius (units, 0 = off).
    #[cfg_attr(feature = "serde", serde(default))]
    pub land_damage: i32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub land_radius: f32,
}

/// State of the guardian in the sand (in [`Creature::mode`]).
pub mod serpent {
    pub const SLEEP: u8 = 0;
    /// Under the sand, only the trail is visible.
    pub const TRAIL: u8 = 1;
    /// Sand quakes at [`super::Creature::goal`].
    pub const WARN: u8 = 2;
    /// In an arc through the air: dangerous and vulnerable.
    pub const LEAP: u8 = 3;
    /// Dazed on the ground: vulnerable.
    pub const STUNNED: u8 = 4;
}

/// State of the guardian on the ground (in [`Creature::mode`]).
pub mod warden {
    pub const SLEEP: u8 = 0;
    pub const IDLE: u8 = 1;
    /// Ground quakes at [`super::Creature::goal`], the root thrust is about to come.
    pub const WARN: u8 = 2;
    /// A core is pulled: vulnerable.
    pub const OPEN: u8 = 3;
}

/// State of a guardian from the air (in [`Creature::mode`]).
pub mod diver {
    /// Waits until Elora comes.
    pub const SLEEP: u8 = 0;
    pub const CIRCLE: u8 = 1;
    pub const AIM: u8 = 2;
    pub const DIVE: u8 = 3;
    /// Dazed on the ground: vulnerable.
    pub const STUNNED: u8 = 4;
    /// Rises back to the circle.
    pub const RISE: u8 = 5;
}

/// State of the root snake (in [`Creature::mode`]).
pub mod burrow {
    pub const HIDDEN: u8 = 0;
    pub const OUT: u8 = 1;
    /// Currently growing out of the ground (harmless, not vulnerable).
    pub const RISING: u8 = 2;
}

/// State of the dune worm (in [`Creature::mode`]).
pub mod leaper {
    /// Under the sand: harmless, invulnerable.
    pub const UNDER: u8 = 0;
    /// Sand quakes, it is about to leap.
    pub const WARN: u8 = 1;
    /// Mid-leap: dangerous and vulnerable.
    pub const LEAP: u8 = 2;
}

/// State of an icicle (in [`Creature::mode`]).
pub mod icicle {
    pub const HANG: u8 = 0;
    /// Trembles: about to fall.
    pub const SHAKE: u8 = 1;
    /// Falling: dangerous.
    pub const FALL: u8 = 2;
}

/// State of the snowball seal (in [`Creature::mode`]).
pub mod seal {
    pub const SLIDE: u8 = 0;
    /// Reared up, throws.
    pub const THROW: u8 = 1;
}

/// State of the ice-spike bat (in [`Creature::mode`]).
pub mod bat {
    /// Sleeps on the ceiling: harmless.
    pub const HANG: u8 = 0;
    pub const DIVE: u8 = 1;
    pub const RETURN: u8 = 2;
}

/// State of the guardian of Frostspitzen (in [`Creature::mode`]); the wave front is stored in
/// [`Creature::goal`] (x = front, y = floor of the hall), the direction in `facing`.
pub mod queen {
    pub const SLEEP: u8 = 0;
    pub const HOVER: u8 = 1;
    /// Frost wave runs across the floor.
    pub const WAVE: u8 = 2;
    /// Sunk down exhausted: vulnerable.
    pub const TIRED: u8 = 3;
    /// Rises back up.
    pub const RISE: u8 = 4;
}

/// State of the frost ghost (in [`Creature::mode`]).
pub mod ghost {
    pub const CHASE: u8 = 0;
    /// Retreats after a touch: harmless.
    pub const FLEE: u8 = 1;
}

/// Entry of the loot table: `min`–`max` pieces with probability `chance`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LootEntry {
    pub item: String,
    pub min: u32,
    pub max: u32,
    #[cfg_attr(feature = "serde", serde(default = "one"))]
    pub chance: f32,
}

#[cfg(feature = "serde")]
fn one() -> f32 {
    1.0
}

/// A living enemy.
#[derive(Debug, Clone, PartialEq)]
pub struct Creature {
    /// Fixed for the lifetime of the world (hook, graphics, events).
    pub id: u32,
    /// Index into [`crate::World::creature_kinds`].
    pub kind: usize,
    pub pos: Vec2,
    pub vel: Vec2,
    pub home: Vec2,
    pub health: i32,
    /// Facing direction: -1 left, 1 right.
    pub facing: i8,
    /// Counter of the behavior (waiting, reloading).
    pub timer: u32,
    /// Stunned (ticks, stomp A-14).
    pub stun: u32,
    /// Tick of the last hit (health bar, E-238).
    pub hit_tick: Option<u64>,
    pub grounded: bool,
    /// State of multi-stage behaviors (guardians, [`diver`]).
    pub mode: u8,
    /// Target of the behavior (dive direction).
    pub goal: Vec2,
    /// Counter within the state (dives in a row, angle while circling in 1/1000;
    /// for the Root Warden: remaining cores).
    pub count: u32,
    /// Tug of war on the Root Warden (ticks) and timing of the root walls.
    pub tug: u32,
    pub wall_timer: u32,
    /// Hits since the last opening (Sand Serpent: dives after `open_hits`).
    pub hits: u32,
}

impl Creature {
    /// Can the enemy take damage right now? Guardians from the air only when dazed (E-299).
    pub fn vulnerable(&self, kind: &CreatureKind) -> bool {
        match kind.behavior {
            Behavior::Diver(_) => self.mode == diver::STUNNED,
            Behavior::Burrower { .. } => self.mode == burrow::OUT,
            Behavior::Warden(_) => self.mode == warden::OPEN,
            Behavior::Follower { .. } => false,
            Behavior::Leaper { .. } => self.mode == leaper::LEAP,
            Behavior::Serpent(_) => matches!(self.mode, serpent::LEAP | serpent::STUNNED),
            Behavior::Queen(_) => self.mode == queen::TIRED,
            _ => true,
        }
    }

    /// Does contact hurt right now? (Root snake only when out, companion never.)
    pub fn harmful(&self, kind: &CreatureKind) -> bool {
        match kind.behavior {
            Behavior::Diver(_) => self.mode != diver::STUNNED,
            Behavior::Burrower { .. } => self.mode == burrow::OUT,
            Behavior::Warden(_) => !matches!(self.mode, warden::OPEN | warden::SLEEP),
            Behavior::Follower { .. } => false,
            Behavior::Leaper { .. } => self.mode == leaper::LEAP,
            Behavior::Serpent(_) => self.mode == serpent::LEAP,
            Behavior::Icicle { .. } => self.mode == icicle::FALL,
            Behavior::Bat { .. } => self.mode != bat::HANG,
            Behavior::Ghost { .. } => self.mode == ghost::CHASE,
            Behavior::Queen(_) => !matches!(self.mode, queen::TIRED | queen::SLEEP),
            _ => true,
        }
    }

    /// Shatters on contact (icicle, snow chunk)?
    pub fn shatters(kind: &CreatureKind) -> bool {
        matches!(
            kind.behavior,
            Behavior::Icicle { .. } | Behavior::Roller { .. }
        )
    }

    /// Can the hook grab it? (Hidden snakes and companions cannot.)
    pub fn hookable(&self, kind: &CreatureKind) -> bool {
        match kind.behavior {
            Behavior::Burrower { .. } => self.mode == burrow::OUT,
            Behavior::Follower { .. }
            | Behavior::Icicle { .. }
            | Behavior::Roller { .. }
            | Behavior::Ghost { .. } => false,
            Behavior::Leaper { .. } => self.mode == leaper::LEAP,
            Behavior::Serpent(_) => matches!(self.mode, serpent::LEAP | serpent::STUNNED),
            _ => true,
        }
    }

    /// Does a hit from direction `src` bounce off the shell? Only hits from above take effect
    /// (E-317); without a direction (stomp, tools) always.
    pub fn armor_blocks(&self, kind: &CreatureKind, src: Option<Vec2>) -> bool {
        kind.armor && src.is_some_and(|p| p.y > self.pos.y - kind.size[1] / 2.0)
    }
}

/// Projectile of an enemy (e.g. pollen ball): flies straight.
#[derive(Debug, Clone, PartialEq)]
pub struct CreatureShot {
    pub owner: u32,
    pub pos: Vec2,
    pub vel: Vec2,
    pub damage: i32,
    pub ticks: u32,
    /// Gravity per tick (0 = flies straight; nuts in an arc).
    pub gravity: f32,
    /// Keeps glowing on the ground for this many ticks after landing (sparks, 0 = vanishes).
    pub glow: u32,
    /// Lies glowing on the ground.
    pub landed: bool,
}

/// Loot lying around (E-236).
#[derive(Debug, Clone, PartialEq)]
pub struct Loot {
    pub id: u32,
    pub item: String,
    pub count: u32,
    pub pos: Vec2,
    pub vel: Vec2,
    /// Age in ticks (can only be collected after a short time).
    pub age: u32,
}

/// Target for the hook (creatures from a character's point of view).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HookTarget {
    pub id: u32,
    pub pos: Vec2,
    pub radius: f32,
    pub small: bool,
    /// Firmly anchored (Root Warden): the hook holds, but does not pull Elora there.
    pub anchor: bool,
}

/// Fixed pseudo-random generator (splitmix64) – same input, same result.
pub(crate) fn rng(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Random number 0..1 from a seed.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn rng_f32(seed: u64) -> f32 {
    (rng(seed) >> 40) as f32 / (1u64 << 24) as f32
}

/// File of the enemy kinds (`assets/adventure/creatures.toml`).
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct KindsFile {
    creature: Vec<CreatureKind>,
}

/// Reads enemy kinds from TOML.
///
/// # Errors
/// On invalid TOML or missing fields.
#[cfg(feature = "serde")]
pub fn kinds_from_toml(src: &str) -> Result<Vec<CreatureKind>, String> {
    toml::from_str::<KindsFile>(src)
        .map(|f| f.creature)
        .map_err(|e| e.to_string())
}

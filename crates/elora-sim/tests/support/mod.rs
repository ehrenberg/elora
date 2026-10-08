//! Shared helpers for the integration tests of `elora-sim` (R2-RF, RF-26): arena builders,
//! a `CreatureKind` builder with defaults, spawning, inputs and running the world.
//!
//! Used with `mod support;`; not every test file needs every helper.

#![allow(dead_code)]

use elora_sim::{
    Abilities, Behavior, CharacterCore, Collision, Creature, CreatureKind, Event, LootEntry,
    PlayerInput, Tile, Tuning, Vec2, World,
};

/// Width of the standard arena in tiles.
pub const W: usize = 60;
/// Height of the standard arena in tiles.
pub const H: usize = 30;
/// Floor row of the arenas.
pub const FLOOR: usize = 28;

// ---------------------------------------------------------------- Worlds

/// Open arena `w`×`h` with floor in row [`FLOOR`], solid bottom row and side walls; `edit`
/// sets further tiles.
pub fn arena(w: usize, h: usize, edit: impl FnOnce(&mut Vec<Tile>)) -> World {
    let mut tiles = vec![Tile::Air; w * h];
    for x in 0..w {
        tiles[FLOOR * w + x] = Tile::Solid;
        tiles[(h - 1) * w + x] = Tile::Solid;
    }
    for y in 0..h {
        tiles[y * w] = Tile::Solid;
        tiles[y * w + w - 1] = Tile::Solid;
    }
    edit(&mut tiles);
    World::new(Tuning::default(), Collision::new(w, h, tiles))
}

/// Standard arena ([`W`]×[`H`], floor in row [`FLOOR`]); `edit` sets further tiles.
pub fn test_world(edit: impl FnOnce(&mut Vec<Tile>)) -> World {
    arena(W, H, edit)
}

/// Like [`arena`], with adventure rules on and the given creature kinds.
pub fn adventure_arena(
    w: usize,
    h: usize,
    kinds: Vec<CreatureKind>,
    edit: impl FnOnce(&mut Vec<Tile>),
) -> World {
    let mut world = arena(w, h, edit);
    world.adventure = true;
    world.creature_kinds = kinds;
    world
}

/// Sets tile (`x`, `y`) in the tiles of a standard arena.
pub fn set(tiles: &mut [Tile], x: usize, y: usize, t: Tile) {
    tiles[y * W + x] = t;
}

// ---------------------------------------------------------------- Positions

/// Centre above tile column `tx` for a box of height `h` resting on row `row` (bottom 1 unit
/// above the top edge, otherwise the rounded corner would already be in the floor).
pub fn resting(tx: usize, row: usize, h: f32) -> Vec2 {
    Vec2::new(tx as f32 * 32.0 + 16.0, row as f32 * 32.0 - h / 2.0 - 1.0)
}

/// Figure stands on row `row`, column `tx`.
pub fn standing(tx: usize, row: usize) -> Vec2 {
    resting(tx, row, 28.0)
}

/// Centre above the floor ([`FLOOR`]) of column `tx` for a box of height `h`.
pub fn on_floor(tx: usize, h: f32) -> Vec2 {
    resting(tx, FLOOR, h)
}

// ---------------------------------------------------------------- Players

/// Lets a player join with `abilities` and stand at `at`.
pub fn spawn(w: &mut World, at: Vec2, abilities: Abilities) -> usize {
    let i = w.join();
    w.set_abilities(i, abilities);
    w.spawn_character(i, at);
    i
}

/// Like [`spawn`], but without the spawn protection: hits count right away.
pub fn spawn_vulnerable(w: &mut World, at: Vec2, abilities: Abilities) -> usize {
    let i = spawn(w, at, abilities);
    w.character_mut(i).unwrap().invulnerable_until = 0;
    i
}

/// Movement state of player 0.
pub fn core(w: &World) -> &CharacterCore {
    w.core(0).expect("alive")
}

/// Position of player 0.
pub fn pos(w: &World) -> Vec2 {
    w.character(0).expect("alive").core.pos
}

/// Health of player 0 (0 when dead).
pub fn health(w: &World) -> i32 {
    w.character(0).map_or(0, |c| c.health)
}

// ---------------------------------------------------------------- Inputs

pub fn idle() -> PlayerInput {
    PlayerInput::default()
}

pub fn input(direction: i8, jump: bool, down: bool) -> PlayerInput {
    PlayerInput {
        direction,
        jump,
        down,
        ..PlayerInput::default()
    }
}

/// Aims at (x, y) relative.
pub fn aim(x: i32, y: i32) -> PlayerInput {
    PlayerInput {
        target_x: x,
        target_y: y,
        ..PlayerInput::default()
    }
}

/// Aims at (x, y) relative with fire counter `counter` (odd = pressed).
pub fn fire(x: i32, y: i32, counter: u8) -> PlayerInput {
    PlayerInput {
        fire: counter,
        ..aim(x, y)
    }
}

// ---------------------------------------------------------------- Running

/// Steps the world `ticks` times with input `i` for player 0; collects the events.
pub fn run(w: &mut World, i: PlayerInput, ticks: u32) -> Vec<Event> {
    let mut all = Vec::new();
    for _ in 0..ticks {
        w.step(&[i]);
        all.extend(w.events.iter().cloned());
    }
    all
}

// ---------------------------------------------------------------- Creatures

/// Living creature with `id`.
pub fn creature(w: &World, id: u32) -> &Creature {
    w.creatures.iter().find(|c| c.id == id).expect("alive")
}

/// Builder for a [`CreatureKind`] with test defaults: size 40×26, health 1, touch damage 2,
/// not small, no boss, no xp, no loot, no daze, no freeze, no armor.
#[derive(Debug, Clone)]
pub struct KindBuilder(CreatureKind);

/// Starts a [`KindBuilder`] for `name` with `behavior`.
pub fn kind(name: &str, behavior: Behavior) -> KindBuilder {
    KindBuilder(CreatureKind {
        name: name.into(),
        size: [40.0, 26.0],
        health: 1,
        touch_damage: 2,
        small: false,
        boss: false,
        xp: 0,
        loot: Vec::new(),
        daze_ms: 0,
        freeze_ms: 0,
        armor: false,
        behavior,
    })
}

impl KindBuilder {
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.0.size = [w, h];
        self
    }

    pub fn health(mut self, health: i32) -> Self {
        self.0.health = health;
        self
    }

    pub fn touch_damage(mut self, damage: i32) -> Self {
        self.0.touch_damage = damage;
        self
    }

    pub fn small(mut self) -> Self {
        self.0.small = true;
        self
    }

    pub fn boss(mut self, boss: bool) -> Self {
        self.0.boss = boss;
        self
    }

    pub fn xp(mut self, xp: u32) -> Self {
        self.0.xp = xp;
        self
    }

    /// Always drops exactly `count` of `item`.
    pub fn loot(mut self, item: &str, count: u32) -> Self {
        self.0.loot.push(LootEntry {
            item: item.into(),
            min: count,
            max: count,
            chance: 1.0,
        });
        self
    }

    pub fn freeze_ms(mut self, ms: u32) -> Self {
        self.0.freeze_ms = ms;
        self
    }

    pub fn build(self) -> CreatureKind {
        self.0
    }
}

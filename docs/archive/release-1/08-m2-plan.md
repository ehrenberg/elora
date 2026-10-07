# M2 – Local Combat: Implementation Plan

Status: **completed** (E-056) · accepted (E-050–E-055) · Basis: [`06-roadmap.md`](06-roadmap.md) M2, E-016, E-023, E-025

## Goal

In the sandbox, hammer, laser and grenade can be used against training dummies and against yourself (rocket jump). HP, armor, pickups, death and respawn work. Acceptance is passed when you confirm that the weapons feel accurate and punchy.

## Work Steps

| # | Step | Crate | Content | Check |
|---|---|---|---|---|
| M2.1 ✅ | Weapon core | `elora-sim` | Weapon tuning (T-18 to T-27), weapon state per character (ownership, ammo, reload timer, active weapon, switch queue), fire logic (click or auto fire), input extended with fire and weapon selection | Unit tests: fire rates, ammo, switching only after reload |
| M2.2 ✅ | Hammer | `elora-sim` | Hit area, line of sight, knockback (T-19), lockout after a hit | Tests: knockback direction, no hits through walls |
| M2.3 ✅ | Grenade & explosion | `elora-sim` | Projectile with analytic trajectory (T-24), collision with wall and player, explosion with radius, force and damage (T-25), self-damage (T-26) → rocket jump | Tests: trajectory, explosion falloff, rocket jump height |
| M2.4 ✅ | Laser | `elora-sim` | Hitscan with range (T-21), bounce with delay (T-22), hit on the first player | Tests: range, bounce, energy consumption |
| M2.5 ✅ | Life & death | `elora-sim` | HP and armor (T-28), damage distribution as in the original, death by damage or death tile, events (hit, death) | Tests: armor logic, self-damage |
| M2.6 ✅ | Pickups & respawn | `elora-sim` | Pickups from the map (heart, armor, laser, grenade) with respawn timer (T-29), respawn delay (T-30), spawn point selection, starting equipment hammer only (E-025) | Tests: pickup rules, timers |
| M2.7 ✅ | Training dummies | `elora-sim` + client | Targets for testing, no bots (E-035), see D-M2-04 | Manual |
| M2.8 ✅ | Rendering & HUD | Client | Placeholders for projectiles, laser beam, explosion, pickups, weapon in hand; HUD with HP, armor, ammo and active weapon; debug panel extended with the weapon values | Visual check |
| M2.9 ✅ | Determinism | `elora-sim` | Recording and golden tests extended with fire and weapon selection (format version 2) | Tests green |
| M2.10 ✅ | Acceptance | – | You play the sandbox | Your acceptance |

After each step: `cargo xtask check` green, then a commit.

## Technical Specifications (Proposal)

- **Everything that affects gameplay lives in `elora-sim`:** weapons, projectiles, laser, damage, pickups, death. Only then can the client use the same logic for prediction in M3. Game rules such as score, teams and rounds follow in M4 in `elora-game`.
- **Events** (shot, hit, explosion, death, pickup) are delivered by the simulation as a list per tick. They are the basis for effects and sounds (M5) and later for network events.
- **Values:** All weapon values go into `Tuning` (adjustable live in the panel, saved in `tuning.toml`). Fixed values from the original that are not in `../../handbook/tuning.md` stay as in the original (hammer radius, lockout after a hammer hit, pickup radius, 125 ms lockout without ammo). They are documented as constants.

## Decisions for M2

| # | Question | Original behavior (0.7) |
|---|---|---|
| D-M2-01 → E-051 | Weapon selection: keys | Mouse wheel forward/back, number keys 1–5 (hammer, pistol, shotgun, grenade, laser) |
| D-M2-02 → E-051 | Order and numbers of our 3 weapons | – (original: hammer, …, grenade, laser) |
| D-M2-03 → E-052 | Laser knockback | no knockback |
| D-M2-04 → E-053/E-054 | Type of training dummies | – |
| D-M2-05 → E-055 | Suicide key (`kill`) already in M2? | present (default: K) |

## Note on T-30 (Respawn)

In the original: respawn **no earlier than 0.5 s** after death, as soon as the fire key is pressed – without a click, automatically after **3 s** (`CPlayer::Tick`). Exactly this behavior is implemented; T-30 (0.5 s) is the minimum delay. Dummies respawn without a click, i.e. after 3 s at their map position.

## Implementation Notes

- **Controls:** Left mouse button shoots, 1/2/3 select hammer/grenade/laser (E-051), the mouse wheel cycles (up = previous, down = next weapon, as in the original). When dead: fire key = respawn (no earlier than 0.5 s), otherwise automatically after 3 s. R immediately puts Elora at the best spawn point.
- **Input counters:** As in the original, fire and mouse wheel are counters (`fire`, `next_weapon`, `prev_weapon`) so that very short clicks are not lost.
- **Tick order** as in the original: inputs/click shots → projectiles → laser → pickups → characters (forces, then weapons) → movement → respawn. The reload timer is already decremented once in the shot tick (as in the original).
- **Pickups** are collected by every character, including dummies (as in the original). Effective pickup radius 40.
- **Dummies** count as player slots (hookable, hittable) and respawn without a click after 3 s at their map position.
- **Recordings** now have format version 2 (fire, weapon selection, dummies, pickups). Old recordings (version 1) are converted automatically on load.
- **Golden tests:** `scripted` (movement), `scripted-combat` (weapons, pickups, rocket jumps, deaths) and your recording from the M1 acceptance. The world rework was checked against the old golden file: movement is bit-identical and unchanged.

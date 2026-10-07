# Analysis: Teeworlds

Status: 2026-09-25 · Sources: teeworlds.com, source code `github.com/teeworlds/teeworlds` (`src/game/tuning.h`, `datasrc/content.py`)

Purpose of this document: record **what** defines Teeworlds, so the clone reaches the same game feel. Decisions on how we implement things are in [`02-decisions.md`](02-decisions.md).

---

## 1. Overview

| Property | Teeworlds |
|---|---|
| Genre | 2D side-scroller shooter, multiplayer (arena) |
| License | Open source (code: zlib-like Teeworlds license; assets: CC-BY-SA) |
| Language / technology | C++, SDL2, OpenGL, custom UDP network protocol |
| Latest version | 0.7.5 (2020). Successor community: **DDNet** (based on the 0.6 protocol) |
| Players per server | typically 8–16 (max. 16 in vanilla) |
| Player character | “Tee” – round ball with eyes and feet, customizable skins/colors |

## 2. What makes the “feeling”

The game feel comes almost entirely from these points – they must be **exactly** right in the clone:

1. **Deterministic tick physics at 50 ticks/s.** All values (velocity, acceleration, gravity) are defined per tick. Rendering interpolates between ticks.
2. **The grappling hook.** Core mechanic: hang on walls, swing, pull opponents in. It shapes movement, tactics and the skill ceiling.
3. **Strong air control + double jump.** You can steer strongly in the air and have a second jump.
4. **Momentum.** Explosions (grenade, hammer) fling players; “rocket jumps” with grenades are possible.
5. **Fast time-to-kill, fast respawn.** 10 HP + 10 armor; rounds are short and hectic.
6. **Client-side prediction.** Your own movement feels instant despite ping.
7. **Precise tile collision system** with 32px tiles.
8. **Audio-visual feedback:** cartoon style, particles, screen feedback, emotes, hit sounds.

## 3. World & collision

- Map = grid of **tiles of 32×32 units**.
- Tile types in vanilla: **air**, **solid** (wall), **death** (kills), **unhookable** (wall the hook does not grab).
- Player hitbox: **28×28** (physical size `PhysSize = 28`), drawn larger (~64px).
- Per-tick movement with sub-stepping against tiles (`MoveBox`), so fast objects do not tunnel through walls.
- Entities layer: spawn points (neutral/red/blue), flag stands, pickups (heart, armor, weapons, ninja).

## 4. Movement physics (default tuning, units/tick at 50 TPS)

| Parameter | Value | Meaning |
|---|---|---|
| `GroundControlSpeed` | 10.0 | Max. running speed on the ground |
| `GroundControlAccel` | 100/50 = 2.0 | Acceleration on the ground |
| `GroundFriction` | 0.5 | Ground friction without input |
| `GroundJumpImpulse` | 13.2 | Jump impulse from the ground |
| `AirJumpImpulse` | 12.0 | Double jump impulse |
| `AirControlSpeed` | 250/50 = 5.0 | Max. air steering speed |
| `AirControlAccel` | 1.5 | Air acceleration |
| `AirFriction` | 0.95 | Air friction |
| `Gravity` | 0.5 | Gravity per tick |
| `VelrampStart` | 550 | above this speed, movement is damped |
| `VelrampRange` | 2000 | Range of the damping |
| `VelrampCurvature` | 1.4 | Curve shape of the damping |
| `PlayerCollision` | 1 | Players collide with each other |
| `PlayerHooking` | 1 | Players can hook each other |

Further details:
- **Double jump:** one air jump, reset on ground contact. Visible (feet).
- **Velocity ramp:** at very high speeds, effective movement is reduced non-linearly – prevents uncontrollable speeds.
- Values are **quantized** (positions/velocities rounded to integers or a fixed precision), so server and client compute deterministically the same.

## 5. Hook

| Parameter | Value |
|---|---|
| `HookLength` | 380 (max. range) |
| `HookFireSpeed` | 80 / tick |
| `HookDragAccel` | 3.0 |
| `HookDragSpeed` | 15.0 |

States: `Idle → Flying → Grabbed (an Wand oder Spieler) → Retracted`.
- The hook flies in the aim direction and grabs solid tiles (not unhookable ones) or players.
- Grabbed: the player is pulled toward the hook position (acceleration up to `HookDragSpeed`).
- Player hook: pulls **both** players toward each other (force split between both).
- The player hook has a time limit: `SERVER_TICK_SPEED + SERVER_TICK_SPEED/5` = 60 ticks = **1.2 s**, after which it releases.
- Force split on a player hook: the hooked player gets `Dir * Accel * 1.5`, the hooking player less.
- Releasing the hook key = hook retracts.
- **Range quirk:** In the tick in which the hook exceeds `HookLength`, it is clamped and goes to `Retract` – a wall on this last, clamped segment is **not** grabbed (players are). Effective wall range = last full flight step: `PHYS_SIZE·1,5 + n·HookFireSpeed ≤ HookLength` → original **362** (42 + 4·80) instead of 380. In addition, the visible hook position then stays at the last flight step.
- Hook pull on a wall: upward full, downward only 30 %; horizontally 95 % in the running direction, otherwise 75 %. No pull below a distance of 46 units.

## 6. Weapons

Every player always has **hammer** and **pistol**. Further weapons via pickup. Max. ammo 10.

| Weapon | Damage | Fire rate (ms) | Ammo | Special |
|---|---|---|---|---|
| Hammer | 3 | 125 | ∞ | Melee, strong knockback (flings opponents away) |
| Pistol (gun) | 1 | 125 | 10, regenerates every 500 ms | Projectile, speed 2200, curvature 1.25, lifetime 2 s |
| Shotgun | 1 per pellet | 500 | 10 | Multiple pellets with spread, speed 2750, SpeedDiff 0.8, lifetime 0.2 s, knockback |
| Grenade launcher | up to 6 (explosion) | 500 | 10 | Ballistic (curvature 7.0), speed 1000, explosion with radius damage and knockback (rocket jump) |
| Laser (rifle) | 5 | 800 | 10 | Hitscan, range 800, bounces off walls once (150 ms delay) |
| Ninja | 9 | 800 | – | Power-up (15 s), dash attack (200 ms, velocity 50), temporarily replaces other weapons |

- **Curvature** = strength of the trajectory bending due to gravity.
- **Explosion:** radius 135, inner radius 48 (full damage), `MaxForce` 12. Factor falls linearly from 1 (≤ 48) to 0 (135). Damage = `(int)(Faktor · MaxDamage)`, force = `Richtung · MaxForce · Faktor`.
- **Hammer knockback:** `(0, -1) + normalize(Dir + (0, -1.1)) · 10` – always flings slightly upward. Only hits with a clear line of sight.
- **Self-damage:** `max(1, Dmg / 2)` – for all weapons, before the armor calculation.
- **Pickups:** respawn 15 s (ninja 90 s, first spawn also after 90 s).
- **Fire logic (`CCharacter::FireWeapon`):** hammer and pistol fire only per click; shotgun, grenade and laser are **automatic** while the button is held. Without ammo: 125 ms lockout + “click” sound. Fire delay (`Firedelay`) as a reload timer in ticks. Weapon switching only once the reload timer has expired.
- **Hammer:** hit center = player + aim direction · 21 (0.75 · 28), radius 14 + 28 (target's body) = 42. Only with a clear line of sight. **After a hit**, lockout of **1/3 s** instead of 125 ms. Knockback `(0,−1) + normalize(Dir + (0,−1,1)) · 10`.
- **Projectiles (grenade):** position computed analytically from start point, direction (rounded to 0.01), speed and curvature: `y = y0 + v·t + Curvature/10000 · t²` (t in s · speed). Per tick a line test against walls and players (radius 6 + 28), **never the own shooter**. The grenade explodes on a wall, a player or when its lifetime expires.
- **Laser:** instant beam of length `LaserReach`; bounces off walls after `LaserBounceDelay` ms, every segment consumes range. Hits the first player along the path (never the shooter). **No knockback** (force 0).
- **Damage (`TakeDamage`):** force is always added (even with friendly fire). Self-damage `max(1, Dmg/2)`. With armor: for Dmg > 1, 1 point goes to HP, the rest to armor first, the excess to HP. HP ≤ 0 → death.
- **Death/respawn:** respawn no earlier than after 0.5 s; spawn point chosen by distance to other players.
- **Pickups:** picked up when a player is closer than 20 + 28 units. Heart/shield +1 (only if < 10). Weapon: full ammo (10), only if not owned or not full.
- **Starting equipment in the original:** hammer + pistol (10 shots), active weapon pistol.
- **Velocity ramp:** `1 / Curvature^((v - Start) / Range)`.
- Weapon switching via mouse wheel / number keys; short switch time.

## 7. Health, pickups, death

- **10 hearts (HP)**, **10 shields (armor)**. Armor absorbs damage before HP.
- Pickups: heart (+1 HP), shield (+1 armor), weapons (full ammo), ninja. Pickups respawn after a fixed time.
- Self-damage from your own grenade (reduced).
- Death → short respawn delay (~0.5 s), spawn at a random/farthest spawn point.
- Suicide (`kill`) possible.

## 8. Game modes (vanilla)

| Mode | Description |
|---|---|
| **DM** | Deathmatch, free for all |
| **TDM** | Team deathmatch, red vs. blue |
| **CTF** | Capture the flag – bring the enemy flag to your own base |
| **LMS** (0.7) | Last man standing – no respawn within the round, the last one wins |
| **LTS** (0.7) | Last team standing – like LMS with teams |

Community modes (not vanilla): **DDRace** (cooperative parkour, freeze tiles), **Instagib** (variants iDM/iTDM/iCTF: laser only, one hit kills; whether it is included in 0.7 vanilla is still to be checked), zCatch, Race.

**Rules in detail (source code `gamecontroller.cpp`, `gamemodes/*.cpp`, `entities/flag.cpp`, 0.7):**

- **Points:** kill +1; suicide/death tile −1; teamkill −1 (team modes only). TDM: team point ± same as player point.
- **Win condition:** `sv_scorelimit` (default **20**, 0 = off) or `sv_timelimit` (minutes, default 0 = off). On a tie at the limit: **sudden death** (next point decides).
- **Game states:** warmup (`sv_warmup`, default 0 s; “game warmup” unlimited while there are too few players: DM < 2, teams: one team empty), countdown (`sv_countdown`, default 0; survival modes always 3 s), running, paused, **round end 5 s**, **match end 10 s**, then the next match (teams swap: `sv_match_swap 1`; map rotation `sv_maprotation`, `sv_matches_per_map 1`).
- **Friendly fire:** `sv_teamdamage 0` – no damage to team members, **knockback still applies**; self-damage remains.
- **Respawn:** TDM at least **3 s** (`sv_respawn_delay_tdm`); after `kill` (suicide command) 3 s; otherwise 0.5 s (see §7).
- **Teams:** automatic balancing after `sv_teambalance_time` (1 min) with unequal teams; spectators possible.
- **CTF:** the flag (radius 14) is picked up by an opponent when they touch it (14 + 28) and have a clear line of sight. **Capture:** the carrier touches their own flag while it is at its stand → team +1 capture, carrier +5 points. If the carrier dies, the flag drops (gravity, bounces with 0.5; killer +1). The own team touching the dropped flag → back to the stand (+1). Without a touch, back after **30 s** or on a death tile. Taking it from the stand +1 point.
- **LMS/LTS:** no respawn during the round; starting equipment **+5 armor, shotgun, grenade (10), laser (5)**; the round ends when ≤ 1 player or 1 team is left; winner +1. Time limit: all survivors +1.
- **Chat:** general and team chat, spam protection.

## 9. Controls (default)

| Action | Key |
|---|---|
| Move | A / D |
| Jump / double jump | Space |
| Aim | Mouse (free 360° aim direction) |
| Fire | Left mouse button |
| Hook | Right mouse button (hold) |
| Switch weapon | Mouse wheel / 1–5 |
| Emote | E (wheel) / shortcut |
| Chat / team chat | T / Y |
| Scoreboard | Tab |
| Kill | K (configurable) |

**Camera and mouse (0.7, source code `camera.cpp`/`controls.cpp`):**
- **Static camera (default, `cl_dynamic_camera 0`):** camera center = exactly the (interpolated) player position.
- **Dynamic camera:** offset toward the mouse = `max(Mausdistanz − 300, 0) · 0,6`.
- **Mouse:** relative (cursor captured); raw mouse deltas × `inp_mousesens/100` are added **directly in world units**. The crosshair is limited to **400 units** around the player (static; dynamic 1000). The aim vector `TargetX/Y` = crosshair relative to the player (integer).
- **View area:** area 1150 × 1000 units², max. 1500 × 1050 (`CalcScreenParams`).

## 10. Network

- **Client-server**, authoritative server, custom protocol over **UDP** (default port 8303).
- The server simulates at 50 TPS and by default sends **snapshots** **every 2nd tick (25 Hz)**; `sv_high_bandwidth 1` (LAN only) sends every tick. Snapshots are **delta-compressed** against the last acknowledged snapshot, with CRC; max. 900 bytes per packet part, larger snapshots are split.
- **Packets:** max. 1400 bytes. Header with flags (control, resend, compression, connless), ack number (10-bit sequence) and number of chunks. Chunks are *vital* (reliable, resent until acknowledged) or *non-vital*. Compression: Huffman with a fixed frequency table, integers as variable length.
- **Connection (0.7):** token handshake against spoofed senders, then connect/accept, keepalive, timeout; close with reason.
- **Player count:** `sv_max_clients` default **8**, technical maximum 64.
- **Inputs:** each tick, the client sends its input with a target tick (`PredTick`). The server reports back how much time was left until processing (`INPUTTIMING`); the client adjusts its prediction time so inputs arrive just (10 ms margin) before their tick.
- **Prediction (0.7 `OnPredict`):** the client computes forward from the last snapshot to the prediction tick – **movement/hook only** (`CCharacterCore`), with its own buffered inputs. Other players are co-simulated without inputs (for collision/hook), but by default **rendered interpolated** (`cl_predict_players 0`). **Weapons and projectiles are not predicted** (`cl_predict_projectiles 0`): shots appear when the server confirms them.
- **No lag compensation:** the server evaluates hits with its current positions (no rewinding).
- **Interpolation:** other objects are interpolated between the last two snapshots.
- **Master server** for the server list, server browser in the client (M7).
- **Demos** via recorded snapshots (O-18).

## 11. Maps & editor

- Own map format (`.map`, datafile with groups/layers).
- Layer types: **game layer** (collision), **tile layer** (graphics, tilesets), **quad layer** (free polygons, animatable), **entities**.
- Parallax groups for backgrounds.
- Integrated **map editor** in the client.
- Well-known vanilla maps: `dm1`, `dm2`, `dm6`, `dm7`, `dm8`, `dm9`, `ctf1`–`ctf7`.

## 12. Graphics & audio

- Hand-drawn, rounded cartoon style, vivid colors.
- Tee skins built from parts (body, feet, eyes, hands; 0.7 additionally decorations/markings), colorable.
- Emotes above the head, eye expressions (pain, joy, …).
- Particles: smoke, hits, explosions, blood-like “splats” in skin color.
- Short, distinctive sounds for every weapon, hook, jump, hit, pickup, death.

## 13. Interface

- Main menu: server browser (internet/LAN/favorites), settings (player, Tee skin, controls, graphics, sound), demos, editor.
- HUD: hearts/shields, ammo, weapon display, timer, score, kill feed, chat, emotes.
- In-game console (local and remote console for admins).

## 14. Legal (important for a clone)

- The code license allows reuse under conditions (attribution, no misrepresentation as the original).
- Assets (graphics/sound) are under **CC-BY-SA 3.0** → use possible with attribution and the same license.
- The name “Teeworlds” should not be used as a product name.
- → How we handle this is an open decision (see decision document).

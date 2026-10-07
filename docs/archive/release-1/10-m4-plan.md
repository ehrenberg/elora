# M4 – Game Modes: Implementation Plan

Status: **completed** (E-092) · accepted (E-066–E-079) · Basis: [`06-roadmap.md`](06-roadmap.md) M4, E-014, E-026, E-055, analysis §8

## Goal

DM, TDM, CTF, LMS, LTS and Instagib are playable on the server – with teams, scoring, rounds, win conditions, scoreboard, killfeed and chat. Acceptance: each mode in one playtest round.

## Work Steps

| # | Step | Crate | Content | Check |
|---|---|---|---|---|
| M4.1 ✅ | Rules framework | `elora-game` (new) | Game states (warmup, countdown, running, round end, match end), scoring, score/time limit, sudden death, events (kill, round, win) | Unit tests with a world without network |
| M4.2 ✅ | Teams | `elora-sim` + `elora-game` | Team per player (red/blue/spectator), friendly fire (damage off, knockback stays), team spawn points, team balancing, team swap after a match | Tests |
| M4.3 ✅ | DM / TDM | `elora-game` | Scoring as in the original, TDM respawn delay | Tests |
| M4.4 ✅ | CTF | `elora-game` | Flags with physics (fall, bounce, 30 s return), pick up, return, capture | Tests |
| M4.5 ✅ | LMS / LTS | `elora-game` | no respawn during the round, starting equipment, round win | Tests |
| M4.6 ✅ | Instagib | `elora-game` | laser only, unlimited ammo, 1 hit = death, no pickups (E-026) | Tests |
| M4.7 ✅ | Server integration | `elora-server` | Mode and rules in `server.toml`/command line, game state and scores in the snapshot, map rotation | Integration test |
| M4.8 ✅ | Chat & commands | Protocol, server, client | Chat and team chat, `kill` (E-055), choose team/spectate, server console (D-M4-07) | Tests |
| M4.9 ✅ | Display | Client | Scoreboard (Tab), killfeed, chat window, round end/winner display, timer, flag indicator – as placeholders (final HUD in M5) | Visual check |
| M4.10 ✅ | Acceptance | – | Playtest of all modes | Your acceptance |

## Technical Specifications (Proposal)

- **New crate `elora-game`** (as planned in the architecture): rules on top of the simulation. `elora-sim` stays rule-free but gets what prediction needs: team membership (for friendly fire) and the flag physics as an entity.
- **Rules run only on the server** (and optionally in the sandbox, see D-M4-10). The client receives game state, scores, teams and flags via the snapshot and messages (chat, killfeed).
- **The sandbox stays runnable without rules**, so that tuning and recordings keep working unchanged.

## Decisions for M4

| # | Question | Original behavior (0.7) |
|---|---|---|
| D-M4-01 → E-066 | Win conditions (default per mode) | Score limit 20, time limit off, sudden death on a tie |
| D-M4-02 → E-067 | CTF scoring | Capture = team point; carrier +5, pick up/return/kill carrier +1 |
| D-M4-03 → E-068 | Warmup / countdown | Warmup 0 s (unlimited with too few players), countdown off, survival 3 s |
| D-M4-04 → E-069 | Friendly fire | off (knockback still applies) |
| D-M4-05 → E-070 | TDM respawn delay | 3 s |
| D-M4-06 → E-071 | LMS/LTS starting equipment | +5 armor, shotgun, grenade 10, laser 5 |
| D-M4-07 → E-072 | Console / admin (O-20) | local server console + remote console with password |
| D-M4-08 → E-073 | Team selection and spectators | Players choose a team or spectator; automatic balancing |
| D-M4-09 → E-074 | Map rotation and team swap | Rotation via list, team swap after every match |
| D-M4-10 → E-075 | Game modes in the sandbox too (against dummies)? | – |
| D-M4-11 → E-076 | Instagib: for which modes? | Community: iDM, iTDM, iCTF |
| D-M4-12 → E-078 | Chat keys | T all, Y team |

## Elaboration

- **Warmup (E-068):** While too few players are present (DM < 2, team modes: one team empty), the game stays in unlimited warmup as in the original. After a map change 10 s warmup, then a 3 s countdown; after match end (10 s) or round end (5 s) directly a 3 s countdown.
- **Friendly fire (E-069):** Default "on" as server setting `friendly_fire`; with "off" it behaves as in the original (no damage, knockback stays).
- **Votes (E-077):** Duration 25 s; passed immediately if more than half of the players vote yes, rejected immediately at at least half no, otherwise passed on expiry if there are more yes than no votes. Kick bans the address for 5 min. One vote at a time.
- **Instagib (E-076):** Spawn with laser only (unlimited ammo), one hit kills, no pickups.
- **Sandbox (E-075):** Mode selectable in the panel; recordings (F5) only without a mode, so that golden tests stay pure simulation.

## Implementation Notes (as of 2026-09-29)

- **Test map:** `maps/ctf-test.emap.toml` with team spawns, flags, pickups and dummies. Sandbox: `cargo run --bin elora -- maps/ctf-test.emap.toml --mode ctf`; server: `cargo run --bin elora-server -- --map maps/ctf-test.emap.toml --mode ctf`.
- **Dummies count as players** (teams, "enough players", scoring) – handy on test servers; real maps have no dummies.
- **Displays (placeholders until M5):** status bar (mode, timer, phase, team scores or own score, goal, sudden death), vote banner, killfeed, chat with notices, scoreboard (Tab), team colors (own character with a yellow ring), flags with stand markers.
- **Console and votes:** see README.
- **Not clicked through with the mouse:** chat input, vote dialog and team buttons in the window (the logic behind them is covered by integration tests).
- **Huffman table** was still trained on the pre-M4 snapshot format; it keeps working (uncompressed is sent if needed). Retrain with `cargo xtask train-huffman` once real game data is available.

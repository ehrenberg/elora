# Roadmap to Release 1

Status: **accepted** (E-037, 2026-09-25)

## Principles

- **Feel first, everything else second.** Every milestone ends with an **acceptance by the project owner**: it has to feel right, not just work.
- **Always playable.** After every milestone there is a running program.
- **The network comes early.** Prediction and snapshots change the architecture deeply. So we build it before much gameplay builds on it (M3 before the game modes).
- **Art and assets come late.** Up to M4, simple vector shapes are enough, so gameplay changes cannot invalidate finished graphics.

## Overview

```
M0 Setup ─► M1 Physics sandbox ─► M2 Local combat ─► M3 Network ─► M4 Game modes
                                                                         │
      M8 Release ◄─ M6 Maps & editor ◄─ M7 Menus & infrastructure ◄─ M5 Look & sound
```

**Order since E-111:** M7 before M6 (numbers stay).

| # | Milestone | Core result | Open decisions beforehand | Decision |
|---|---|---|---|---|
| M0 | Project setup | Workspace compiles, local checks run | O-41 | ✅ |
| M1 | Physics sandbox | Elora runs, jumps and hooks, the feel is accepted | – | ✅ |
| M2 | Local combat | Hammer, laser, grenade, damage, pickups, death and respawn | – | ✅ |
| M3 | Network | Dedicated server, multiple clients, prediction, playable on LAN | O-42 | ✅ |
| M4 | Game modes | DM, TDM, CTF, LMS, LTS, Instagib, scoreboard, chat | O-20 | ✅ |
| M5 | Look & sound | Vector renderer, Elora art, skins, particles, audio, HUD | O-39, O-40 | ✅ |
| M6 | Maps & editor | Release map format, integrated editor, first real maps | O-37, O-43 | ✅ |
| M7 | Menus & infrastructure | Main menu, settings, key bindings, server browser | O-17, O-18 | ✅ |
| M8 | Release | Packages for 3 operating systems, balancing, playtests, Release 1 | O-44 | ✅ |

---

## M0 – Project setup

- Local Git repository (E-034), `.gitignore`, `LICENSE` (GPL-3.0), `THIRD_PARTY_LICENSES`
- Cargo workspace following [`../../handbook/architecture.md`](../../handbook/architecture.md), initially only with the M1 crates
- `rust-toolchain.toml`, rustfmt, clippy lints, cargo-deny
- Local check script (fmt + clippy + test + deny) as a substitute for CI (O-41)

**Acceptance:** `cargo build` and `cargo xtask check` run without errors.

**Status 2026-09-25:** implemented, `cargo xtask check` green (fmt, clippy, nextest, deny). **Accepted** (E-042).

## M1 – Physics sandbox (E-013)

- `elora-sim`: tuning (E-023), quantization (E-021), tile collision, movement, jump and double jump, hook including its state machine, velocity ramp
- `elora-map`: text format (E-024), validation, hot reload
- `elora-render`: window (winit), wgpu, camera with mouse offset, simple shapes (circle for Elora, rectangles for tiles, line for the hook)
- Fixed tick loop at 50 TPS with render interpolation
- egui debug panel: all tuning values adjustable live, display of velocity and state
- Golden tests for determinism (input recording → expected state)
- Test map `maps/sandbox.emap.toml`

**Acceptance:** The project owner plays the sandbox and confirms the movement feel, possibly after re-tuning.

**Status 2026-09-25:** **Accepted** (E-049).

## M2 – Local combat

- Mouse aiming, weapon switching
- Hammer (knockback T-19), laser (hitscan with bounce), grenade (ballistic, explosion, rocket jump)
- HP and armor, self-damage, death, respawn, death tiles
- Pickups with respawn timer, starting equipment hammer only (E-025)
- **Training dummies:** stationary or scripted targets for testing. These are not bots (E-035).
- Simple HUD via egui (placeholder)

**Acceptance:** The weapons feel accurate and punchy, rocket jumps and hammer jumps work.

## M3 – Network (E-008, E-012)

- `elora-net`: UDP, connection setup, reliability layer, timeouts
- `elora-protocol`: input messages, snapshots, delta compression
- `elora-server`: dedicated server without graphics
- Client: prediction of the own character, interpolation of the others, correction on mismatches
- **Network simulator:** artificial ping, jitter and packet loss for testing
- Local hosting and direct connect via IP (no server browser yet)

**Acceptance:** 2 to 8 players on LAN. With a simulated 100 ms ping, the own movement feels exactly as it does offline.

## M4 – Game modes (E-014)

- `elora-game`: framework for game rules, plus DM, TDM, CTF, LMS, LTS and Instagib (E-026)
- Teams, spawn logic, flags, rounds, warmup, score and time limit
- Scoreboard, kill feed, chat and team chat
- Server configuration (file and command line), console (O-20)

**Acceptance:** Every mode is playable in a playtest round.

## M5 – Look & sound (E-029, E-030, E-032, E-033)

- Vector renderer: lyon tessellation, transformations, deformation (squash and stretch)
- Elora design, animations (running, jumping, hook, eyes and emotes)
- Skin system built from parts (E-029, O-39)
- Particles, explosions, hit feedback
- kira audio: all game sounds, volume by distance
- Final HUD (own game UI, E-031)

**Acceptance:** Style and feedback are accepted. Elora is clearly distinguishable from a Tee.

**Status 2026-09-30:** **Completed** (E-110), plan and implementation in [`11-m5-plan.md`](11-m5-plan.md).

## M6 – Maps & editor (E-028) – after M7 (E-111)

- Release map format (O-37): game layer, graphics layers, parallax, quads, animations
- Integrated editor (egui, E-031): layers, tiles, entities, test play directly from the editor
- Import of text maps (E-024) into the release format
- First release maps (O-43: how many and which modes)

**Acceptance:** The editor can be used to build a map from scratch and play it.

**Status 2026-10-01:** **Completed** (E-160), plan and implementation in [`13-m6-plan.md`](13-m6-plan.md).

## M7 – Menus & infrastructure – before M6 (E-111)

- Main menu, settings (graphics, audio, controls, player and skin), key bindings
- Server browser: LAN plus internet via a master server (O-17)
- Possibly demos and replays (O-18)

**Acceptance:** A new player finds a server without help and can play.

**Status 2026-10-01:** **Completed** (E-128), plan and implementation in [`12-m7-plan.md`](12-m7-plan.md).

## M8 – Release 1 (E-003)

- Packages and installers for Linux, Windows and macOS (O-44: distribution channels)
- Balancing rounds, public playtests, bug fixing
- License and credits page (GPL-3.0, CC-BY-SA 4.0, third parties)

**Acceptance:** Release 1 is published.

---

## New open items from this roadmap

- **O-42 Network targets:** maximum player count per server, snapshot rate, target bandwidth
- **O-43 Release maps:** number and modes for Release 1
- **O-44 Distribution:** Where will Release 1 be published (itch.io, Steam, Flathub, own website …)?

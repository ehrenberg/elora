# Principles

What remains binding from Release 1 – in brief. The reasoning is in the
[Release 1 decision log](../archive/release-1/02-decisions.md) (E-001 to E-173);
new decisions are in the [Release 2 log](../release-2/decisions.md) (from E-200).

## Collaboration

- **The project owner makes all decisions.** Claude proposes, gives reasons and asks; no assumptions.
- Findings and decisions are recorded in `docs/` (E-002).
- Work happens in **milestones** with plan → approval → implementation → **acceptance in a playtest** (E-036). It has to feel right, not just work.
- After every step `cargo xtask check` is green, then commit (Conventional Commits).
- Keys and secrets never go into the repository (`server_key.toml`, API keys only as environment variables, E-084).

## Game

| Area | Principle | Origin |
|---|---|---|
| Goal | Own game with the game feel of Teeworlds 0.7, own identity “Elora” | E-001, E-005, E-018 |
| Platforms | Desktop: Linux, Windows, macOS | E-004 |
| Code | completely new in Rust, own engine; Teeworlds serves only as a reference for values and behavior | E-007, E-009 |
| Simulation | deterministic, 50 ticks/s, `f32` with quantization per tick | E-021 |
| Tuning | own, justified deviations from the original; every value decided individually | E-015, E-022, [`tuning.md`](tuning.md) |
| Weapons | Hammer, grenade launcher, laser; spawn with hammer only | E-016, E-025 |
| Modes | DM, TDM, CTF, LMS, LTS, each also as Instagib | E-014, E-076 |
| Character | Elora, drop shape “Wirbel”, skins from colors only (eyes, body, feet) | E-085, E-094, E-095 |
| Bots | Bots provide inputs like players (so far only training dummies) | E-035, E-053 |

## Technology

| Area | Principle | Origin |
|---|---|---|
| Graphics | wgpu + winit, own 2D vector renderer (lyon), assets as SVG | E-011, E-030, E-033 |
| UI | own game UI “light & soft”; egui (dark) for debug panel and editor | E-031, E-150 |
| Language | German and English, texts in `assets/lang/` | E-114 |
| Sound | kira; sounds from CC0 sources (Kenney), music as Ogg Vorbis from free sources (CC0, if desired CC BY with credit in SOURCES.md and “About”) | E-032, E-108, E-109, E-285, E-289 |
| Network | own UDP protocol: token, Noise encryption, server keys like SSH, snapshots with delta; IPv4 and IPv6 | E-012, E-061, E-062, E-063 |
| Server list | HTTP/JSON master over HTTPS, `https://elora.bastianswelt.de`; dedicated servers register, hosted ones only on request | E-112, E-127, E-166, E-170 |
| Maps | binary format `.emap`, in-game editor, automatic download, custom maps in the user directory | E-129, E-136, E-146, E-152 |
| Map look | style A “soft & lively”; stone only for unhookable walls | E-139, E-148 |
| Licenses | code GPL-3.0, own assets CC-BY-SA 4.0, third-party assets only with a suitable license | E-020, E-027 |
| Tools | Rust version pinned, `cargo xtask check` (fmt, clippy, nextest, deny), GitHub Actions for checks and release packages | E-038, E-039, E-163 |
| Distribution | GitHub Releases; versions follow SemVer, Release 1 = 0.9.0 Beta | E-161, E-165 |

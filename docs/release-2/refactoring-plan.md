# Refactoring before the next release – plan

Status: **Decided, ready to start** (E-348, E-350) · Scope: whole workspace (`crates/`, `apps/`, `xtask/`, `docs/`, `assets/`)

## Goal

Before the next release, the code base gets a cleanup pass: **English throughout** (code, docs,
README) and **better structure** where the last milestones left large files, duplicated logic and
suppressed lints. Gameplay must not change; save games, maps and replays must keep working.

## Language policy (E-348, applies from now on)

| Area | Rule |
|---|---|
| Code identifiers | English (functions, types, variables, modules, test names) |
| Code comments and doc comments | English |
| Documentation (`docs/`, `DEVELOPMENT.md`, handbook) | English |
| `README.md` | English |
| Player-facing text | stays translated via `assets/lang/*.toml` and the `de`/`en` fields in content files |
| Conversation with the project owner | German (unchanged) |

New and changed code follows the rule immediately; existing code is converted in the tasks below.

## Current state (measured 2026-10-07)

- ~66,000 lines of Rust, 35 Markdown documents, all docs and nearly all comments in German.
- Identifiers are mostly English already; German remains in map generator modules and
  functions (`kapitel1`…`kapitel4`, `wueste_1`, `stampf`, `kletter`, `ruck_gate`), content ids
  and world flags (`besiegt.*`, `befreit.*`, `quellen_befreit`, `eisgriff.stark`), and the
  dialog/quest condition language (`merker`, `quest … schritt`, `gib`, `nimm`, `faehigkeit`).
- Largest files: `elora-sim/src/creature_world.rs` (~1,960 lines), `elora-client/src/main.rs`
  (~1,450), `elora-map/src/binary.rs`, `elora-adventure/src/session.rs`, `debug_ui.rs`,
  `adventure_menu.rs`, `world.rs`, `protocol/msg.rs`, `app_adventure.rs` (all > 1,100).
- 27 `#[allow(clippy::too_many_lines)]`, `clippy::` allows in 81 files.

## Tasks

### Phase 1 – language (mechanical, low risk)

| # | Task | Notes |
|---|---|---|
| RF-01 ✅ | Rewrite `README.md` in English | User-facing; keep the structure from the last rewrite |
| RF-02 ✅ | Translate `DEVELOPMENT.md` and `docs/handbook/*` to English; rename the folder to `docs/handbook/` | Fix all links |
| RF-03 ✅ | Translate all plans, decisions and the archive (`docs/release-2/`, `docs/archive/`, `docs/releases/`) | D-RF-03; rename `docs/archive/` to `docs/archive/` |
| RF-04 ✅ | Translate code comments and doc comments, crate by crate (`elora-sim` → `elora-map` → `elora-protocol` → `elora-net` → `elora-audio` → `elora-render` → `elora-game` → `elora-adventure` → apps → `xtask`) | One commit per crate; no code changes in the same commit |
| RF-05 | Rename German identifiers in code (map generators `kapitel*` → `chapter*`, `wueste_*` → `desert_*`, helpers like `stampf`/`kletter`, test names) | Pure renames, compiler-checked |
| RF-06 | Translate log, panic and error messages and `xtask` output | Player-facing messages stay in `assets/lang` |
| RF-07 | Commit messages in English from the start of the refactoring | Conventional Commits stay |

### Phase 2 – data and content language (needs decisions, save-game impact)

| # | Task | Notes |
|---|---|---|
| RF-10 | Rename content ids (items, creatures, characters, maps, quests, flags) to English | D-RF-01; one rename table drives content files, maps and the save migration |
| RF-11 | English condition language (`flag`, `quest … step`, `give`, `take`, `ability`) | D-RF-02; the parser accepts the German keywords for one release |
| RF-12 | Save-game migration (format version bump, rename table for flags, items and quest ids) and a test with old save files | `SaveGame` already has a format version |
| RF-13 | Rewrite content files and map files via the generators; editor keeps loading old maps | Map ids are referenced by exits and the world map |

### Phase 3 – structure and quality

| # | Task | Notes |
|---|---|---|
| RF-20 | Split `creature_world.rs` into one module per behavior (`walker`, `turret`, `diver`, `warden`, `serpent`, `queen`, …) behind a small trait or function table | Biggest file; behaviors already have separate `tick_*` functions |
| RF-21 | Split `elora-client/src/main.rs` (app state, frame loop, look/weather, sound glue) | Removes several `too_many_lines` allows |
| RF-22 | Split `session.rs` (heat/cold, avalanches, followers, doors, quests glue) into submodules | Heat and cold share one “temperature bar” implementation (RF-23) |
| RF-23 | Unify heat and cold bars: one `TemperatureBar` with data from `worldmap.toml` (fill/cool times, zone prefixes) instead of constants in code | Today: `HEAT_*`/`COLD_*` constants, two near-identical functions |
| RF-24 | Protocol: one list of non-networked events instead of two hand-kept `matches!` lists in `msg.rs` | New events had to be added twice |
| RF-25 | Generate asset lists (decor, creatures, characters, music test lists) from the asset folders in `build.rs` instead of hand-kept arrays | Several were missed or had to be updated in 3 places |
| RF-26 | Shared test support: one `test_world()`/`CreatureKind` builder for `elora-sim` tests instead of per-file copies | `freeze_ms` had to be added to five initializers |
| RF-27 | Map generators: move shared helpers (`edge_exit`, `zone`, `big`, `chimney`, `climb_vault`, `stomp_vault`, …) into one `editor/gen` module; document the mirroring of forest maps | Helpers were made `pub(super)` ad hoc |
| RF-28 | Reduce `#[allow(clippy::…)]`: remove those that no longer apply, replace casts with helpers (`tile_of`, `to_f32`) | Goal: no `too_many_lines` outside generators and tests |
| RF-29 | Review public API docs (`cargo doc` without warnings, `missing_docs` on library crates) | |

### Phase 3b – new: first-start setup screen (E-351)

| # | Task | Notes |
|---|---|---|
| RF-30 | Setup screen on the first start: one page with **name** (required, not empty), **language** (switches the screen immediately) and **Elora's look** (body, feet, eyes with a live preview); a “Let's go” button saves the settings and opens the main menu | Shown only when no settings file exists yet; no skipping; built in English and in the new structure |
| RF-31 | First-start detection and validation as testable functions (settings file missing → setup; name trimmed, length limit like the player settings) | Unit tests; existing installs never see the screen |

### Phase 4 – verification

| # | Task | Notes |
|---|---|---|
| RF-40 | Golden tests, replays and all shipped maps unchanged after each phase | `cargo xtask check` after every commit |
| RF-41 | Old save games from 0.9.x load | Keep sample saves as test fixtures |
| RF-42 | Short playtest of every chapter after Phase 3 | |

## Decisions (E-350)

| # | Question | Decision |
|---|---|---|
| D-RF-01 | Content ids (items, creatures, maps, quests, flags) in English? | **Yes**, with a save-game migration |
| D-RF-02 | Dialog/quest condition language in English? | **Yes**; the parser accepts the German keywords for one release, then they are removed |
| D-RF-03 | Translate the archive (`docs/archive/`, `docs/qa-*`) too? | **Yes** |
| D-RF-04 | When? | **Now**, after the chapter 4 acceptance (E-349) |

## Order

1. Phase 1 (RF-01 … RF-07) – can be done in small commits without risk.
2. Phase 2 (decided in E-350).
3. Phase 3 one task at a time, each with `cargo xtask check` and unchanged golden tests.
4. Phase 4 before tagging the release.

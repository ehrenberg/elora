# Architecture & code structure

Status: Release 1 (0.9.0) · Principles: [`principles.md`](principles.md)

## 1. Guiding principles

1. **The simulation is pure logic.** Physics, hook, weapons and game rules know nothing about windows, graphics or network. Server, client (prediction), sandbox and tests use the same code.
2. **Dependencies point in one direction:** programs → engine crates → simulation. The simulation depends on nothing related to platform or input/output.
3. **Determinism is tested.** The same inputs yield a bit-exact identical state; golden tests replay recorded inputs (`crates/elora-sim/tests/recordings/`).
4. **Logic without a window is testable.** Online client, editor tools, maps and network run in tests without graphics (in-memory network `MemNetwork`, virtual time).

## 2. Workspace

```
crates/
  elora-sim        simulation: world, collision (tile kinds), character, hook, weapons, pickups, dummies, abilities, enemies, loot, tuning, recordings
  elora-game       game rules: modes, scores, rounds, teams, warmup, sudden death
  elora-map        map model and format .emap (binary, zlib), look (materials, decoration, layers, envelopes, SVGs)
  elora-protocol   messages, snapshots with delta, Huffman, server info, translatable messages
  elora-net        UDP: token, Noise handshake, reliability, fragments, info query, IPv4/IPv6, network simulator
  elora-render     wgpu renderer: shapes, meshes, SVG assets, text, camera
  elora-audio      sounds: procedural generator, mapping to events, playback (kira)
  elora-adventure  adventure: content as data, save game, levels, skill tree, inventory, shops, upgrades, saving
apps/
  elora-client     the game (lib: online client, scene, map store; bin: window, menus, HUD, editor)
  elora-server     dedicated server (lib: game server, console, votes, paths; bin: program, master registration)
  elora-master     master server for the internet list (HTTP/JSON, UDP check)
xtask/             cargo xtask: check, package, map-dump, svg-preview, sound-preview/-import, train-huffman, net-stats
deploy/            operation: master/ (systemd, Docker, proxy), master-php/ (web hosting: master + project page)
tools/design/      Python generators for drafts, map graphics and release maps
assets/            SVGs (character, items, emotes, maps), sounds, fonts, languages; adventure/ with adventure content
maps/              shipped maps (.emap)
```

### Client (`apps/elora-client/src`)

| Area | Modules |
|---|---|
| Flow | `main.rs` (app, screens menu/game/editor, input), `app_menu.rs`, `app_editor.rs` |
| Local game | `sandbox.rs` (training, hot reload, recording, test play) |
| Online game | `online.rs` (lib: snapshots, prediction, interpolation, map download), `connection.rs`, `map_store.rs` (lib) |
| Rendering | `draw.rs`, `map_view.rs` (layers, parallax, cache), `map_art.rs` (auto edges), `figure.rs`, `items.rs`, `effects.rs`, `emotes.rs`, `skins.rs` |
| UI | `ui.rs` (own toolkit), `menu*.rs`, `hud.rs`, `game_ui.rs`, `debug_ui.rs` + `gui.rs` (egui), `lang.rs` |
| Editor | `editor/` – state and history (`mod.rs`), tools (`tools.rs`), look (`look.rs`), UI (`panel*.rs`), view (`view.rs`) |
| Other | `browser.rs`, `hosting.rs`, `settings.rs`, `bindings.rs`, `controls.rs`, `sound.rs`, `tuning_file.rs` |

## 3. Runtime

- **Tick model:** fixed 50 ticks/s. The client renders at any frame rate and interpolates between two states (accumulator).
- **Online:** The server sends snapshots (25 Hz, LAN 50 Hz) as deltas; the client predicts its own character including weapons and interpolates the others. Maps arrive in chunks on joining, verified via BLAKE2s (`MapInfo` → `MapRequest`/`MapChunk` → `MapReady` → `Welcome`).
- **Files:** Shipped data via `elora_server::paths::resolve` (working directory, `ELORA_DATA`, next to the program, `../share/elora`, `../Resources`); settings, tuning, server key in the settings folder; custom and downloaded maps under `~/.local/share/elora`.
- **Protocol version:** 6 (increase on changes; adjust the master PHP `config.php`).

## 4. Standards & tools

| Area | Standard |
|---|---|
| Language | Rust 2024, version in `rust-toolchain.toml` |
| Checks | `cargo xtask check`: rustfmt, clippy (pedantic, `-D warnings`), cargo-nextest, cargo-deny; the same in GitHub Actions on every push |
| Dependencies | centrally in `[workspace.dependencies]`, licenses and advisories via cargo-deny |
| Errors | `thiserror` in libraries, `anyhow` in programs; logging with `tracing` |
| Visual checks | ignored tests generate SVGs under `target/`, `cargo xtask svg-preview` rasterizes them |
| Release | tag `v<version>` → GitHub Actions builds packages and creates a draft release; notes in `docs/releases/` |
| Commits | Conventional Commits |

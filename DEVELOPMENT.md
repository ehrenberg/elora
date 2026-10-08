# Elora – for developers

Everything about source code, building, testing, running servers and contributing. For players,
the essentials are in the [README](README.md).

**Status:** 0.9.2 Beta – multiplayer from Release 1 and, as a preview, the adventure “The Silent
Springs” (prologue and chapters 1–3). Planning: [Release 2 roadmap](docs/release-2/roadmap.md),
changes: [release notes](docs/releases/).

## Contents

- [Setup on Arch Linux](#setup-on-arch-linux)
- [Running from source](#running-from-source)
- [Checks and tests](#checks-and-tests)
- [Server, console and master](#server-console-and-master)
- [Maps](#maps)
- [Replacing sounds](#replacing-sounds)
- [Packages and releases](#packages-and-releases)
- [Files and folders](#files-and-folders)
- [Project structure](#project-structure)
- [Workflow](#workflow)
- [Troubleshooting](#troubleshooting)

## Setup on Arch Linux

### 1. Packages

```sh
sudo pacman -S --needed git base-devel rustup cargo-deny cargo-nextest
rustup default stable
```

- `rustup` replaces the `rust` package (pacman asks – confirm with yes). rustup downloads the exact Rust version (see `rust-toolchain.toml`) automatically on the first `cargo` call in the project.
- `cargo xtask check` needs `cargo-deny` (license and security checks) and `cargo-nextest` (test runner).

### 2. Graphics (Vulkan) and windows

The renderer uses wgpu, on Linux Vulkan by default. You need the Vulkan loader and the driver for your graphics card:

```sh
sudo pacman -S --needed vulkan-icd-loader
sudo pacman -S --needed vulkan-radeon    # AMD
# or: vulkan-intel                       # Intel
# or: nvidia-utils                       # NVIDIA (proprietary)
```

Windows run under Wayland and X11 (usually already present on desktop installations, otherwise: `sudo pacman -S --needed wayland libxkbcommon libx11 libxcursor libxrandr libxi`).

Check: `vulkaninfo --summary` (package `vulkan-tools`) should show the graphics card.

### 3. Sound

Audio output goes through ALSA (`alsa-lib`, present on desktop installations; with PipeWire also `pipewire-alsa`):

```sh
sudo pacman -S --needed alsa-lib pipewire-alsa
```

Without an audio device the game still starts – muted, with a notice in the panel under *Ton* (sound).

### 4. Building

```sh
git clone https://github.com/ehrenberg/elora.git && cd elora
cargo build            # builds the game (elora), server (elora-server) and master (elora-master)
```

Run all commands from the project folder (maps live in `maps/`).

## Running from source

```sh
cargo run --bin elora                               # starts in the main menu
```

### Sandbox and shortcuts

With a map, `--mode`, `--connect` or `--abenteuer` on the command line you skip the menu and go straight into the game, with the debug panel (F1):

```sh
cargo run --bin elora -- maps/sandbox.emap          # default map
cargo run --bin elora -- maps/eigene.emap           # another map
cargo run --bin elora -- maps/ctf-test.emap --mode ctf   # game mode against dummies
cargo run --bin elora -- maps/look-test.emap         # map look: materials, backgrounds, decoration, animations
cargo run --bin elora -- maps/tiles-test.emap        # platform, ice, jump pad, conveyor (S = down)
cargo run --bin elora -- maps/faehigkeiten-test.emap # adventure abilities (F1 → enable „Fähigkeiten“)
cargo run --bin elora -- --abenteuer 1              # adventure in slot 1 (continue or new)
cargo run --bin elora -- --connect 127.0.0.1:8303   # straight to a server
```

Game modes in the sandbox: `--mode dm|tdm|ctf|lms|lts` (optionally `--instagib`) or in the panel under *Spiel → Modus* (game → mode). CTF needs a map with flags, e.g. `maps/ctf-test.emap`.

The sandbox is for tuning and testing: dummies, pickups, adventure enemies, live sliders for all values ([Tuning](docs/handbook/tuning.md)), hot reload of the map.

### Developer keys

| Key | Action |
|---|---|
| F1 | Debug panel on/off (also in the menu); frees the mouse |
| R | Respawn (sandbox without a mode only) |
| F5 | Start/stop recording (sandbox only, see determinism tests); in the editor: test play |

## Checks and tests

```sh
cargo xtask check          # everything: formatting, clippy, tests, licenses & advisories
cargo xtask fmt            # format code
cargo nextest run --workspace --all-features   # tests only
```

After every change `cargo xtask check` must be green, then commit. Every push is checked on GitHub with the same command.

**Determinism tests (golden files):** The simulation must stay bit-exact reproducible. In the sandbox `F5` records your own inputs; the file ends up in `crates/elora-sim/tests/recordings/` and becomes a regression test. (Re)generate golden files – only after a deliberate physics change:

```sh
ELORA_BLESS=1 cargo nextest run -p elora-sim --all-features
```

**Tools:**

```sh
cargo xtask net-stats         # measure message sizes for 8/16/64 players
cargo xtask train-huffman    # retrain the Huffman table (after changes to the snapshot format)
cargo xtask sound-preview    # all sounds as WAV into target/sounds/ (listening test; parameters in assets/sounds/sounds.toml)
cargo xtask svg-preview a.svg a.png 1200   # rasterize an SVG (drafts, assets)
cargo xtask map-dump maps/dm-wiese.emap    # view a map as text
```

## Server, console and master

### Starting a server

```sh
cargo run --bin elora-server
cargo run --bin elora-server -- --port 8303 --map maps/dm-wiese.emap --max-clients 16 --name "Mein Server"
cargo run --bin elora-server -- --config server.toml
```

| Option | Default | Meaning |
|---|---|---|
| `--port` | 8303 | UDP port |
| `--bind` | `::` | `::` = IPv4 and IPv6 (registers with the master over both), `0.0.0.0` = IPv4 only |
| `--map` | `maps/dm-wiese.emap` | Map (relative: first the working directory, then the installation's data folder) |
| `--max-clients` | 8 | Players, 1–64 |
| `--name` | Elora-Server | Display name |
| `--high-bandwidth` | off | Snapshots at 50 instead of 25 Hz (LAN only) |
| `--key-file` | `server_key.toml` | Persistent server key (generated on first start) |
| `--tuning` | – | Tuning file (section `[physics]` as in `tuning.toml`) |
| `--mode` | dm | Game mode: `dm`, `tdm`, `ctf`, `lms`, `lts` |
| `--instagib` | off | Laser only, one hit kills (for all modes) |
| `--score-limit` | 20 (CTF: 5 captures) | Points to win, 0 = off |
| `--time-limit` | 0 | Time limit in minutes, 0 = off |
| `--no-friendly-fire` | on | No damage to teammates (knockback remains) |
| `--no-votes` | on | Disable votes |
| `--master` | `https://elora.bastianswelt.de` | Register with the master server (internet list); a custom one replaces the default, can be given several times |
| `--no-master` | – | Do not register in the internet list |
| `--config` | – | All options from a TOML file; further options override it |

Settings available only in the config file: `rotation = ["sandbox", "ctf-test"]` (map rotation), `maps_dir`, and a section `[rules]` (`warmup_secs`, `countdown_secs`, `tdm_respawn_secs`, `team_balance_secs`, `match_swap`, `matches_per_map`).

**Hosting from the client:** main menu → *Create server*. The client writes `server.toml` and starts `elora-server` as a separate process (from source, run `cargo build` once beforehand). More options in the debug panel → *Netzwerk* → *Server einrichten …* (network → set up server).

### Console

Type commands into the server's terminal – `help` shows all of them: `status`, `mode ctf`, `instagib on`, `scorelimit 10`, `timelimit 5`, `friendlyfire off`, `restart`, `map <name>`, `maps`, `kick <slot>`, `ban <slot>`, `spectate <slot>`, `say <text>`, `vote cancel`, `quit`.

Quit: `quit` + Enter or Ctrl+C. Control log output: `RUST_LOG=debug cargo run --bin elora-server`.

### Firewall and security

For players from the LAN/internet the UDP port (default 8303) must be open, e.g. `sudo ufw allow 8303/udp` or `sudo firewall-cmd --add-port=8303/udp`.

The connection is encrypted (Noise protocol). The client remembers each server's key in `known_servers.toml` and warns when it changes (like SSH). `server_key.toml` is secret and does not belong in Git.

### Master server (internet list)

```sh
cargo run --bin elora-master -- --bind 0.0.0.0:8300                # HTTP on port 8300
cargo run --bin elora-master -- --bind 127.0.0.1:8300 --behind-proxy   # behind a reverse proxy
```

Game servers register with the default master (disable with `--no-master`, custom one with `--master <URL>` or `masters = ["…"]` in `server.toml`; servers hosted from the client only with “Show on the internet list”) every 20 s; the master lists them only once it reaches them itself via UDP, and removes them after 60 s without registration. Run a public master behind a reverse proxy with HTTPS (e.g. Caddy: `reverse_proxy 127.0.0.1:8300`) and set `--behind-proxy`. Interface: `GET /servers`, `POST /register` (JSON). Running with systemd or Docker: [`docs/handbook/master-operation.md`](docs/handbook/master-operation.md).

For web hosting without your own service there is a PHP version of the master and project page: [`deploy/master-php/`](deploy/master-php/LIESMICH.md) (runs at `https://elora.bastianswelt.de`). When the game's protocol version changes, `protocol_version` in `deploy/master-php/config.php` must follow.

## Maps

- **Format:** [`docs/handbook/map-format.md`](docs/handbook/map-format.md); view with `cargo xtask map-dump`.
- **Release maps** (M6.10): `dm-wiese` (64×36, 4–8 players), `dm-wueste` (96×48, 8–12), `dm-winter` (128×64, 12–16), `ctf-wald` (150×48, 8–12), `ctf-nacht` (190×64, 12–16). Layouts are built and checked in `tools/design/release_maps/` (`python3 tools/design/release_maps/export.py`); the files are written by `cargo test -p elora-client --bin elora write_release_maps -- --ignored`.
- **Adventure maps** (`maps/abenteuer/`) come from generators in `apps/elora-client/src/editor/` (`prologue.rs`, `chapter1.rs` to `chapter4.rs`) and are written with `write_prologue_maps`, `write_chapter1_maps` etc. (`cargo test -p elora-client --bin elora <name> -- --ignored`). A test checks that the shipped files are up to date.
- **Adventure content** (characters, dialogs, quests, enemies, items) lives as TOML in `assets/adventure/`: [`docs/handbook/adventure-content.md`](docs/handbook/adventure-content.md).
- **Graphics** are made with Python scripts in `tools/design/` (no AI services, E-295).

## Replacing sounds

Every sound has a fixed name (list: `cargo xtask sound-preview` or `crates/elora-audio/src/cues.rs`, e.g. `jump`, `hammer_fire`, `grenade_explode`). A file `assets/sounds/files/<name>.wav` replaces the sound of the same name.

1. Import a file – any format ffmpeg reads; optionally start and length in seconds:
   ```sh
   cargo xtask sound-import jump ~/Downloads/boing.ogg          # whole file
   cargo xtask sound-import jump ~/Downloads/boing.ogg 0.1 0.3  # from 0.1 s, 0.3 s long
   ```
   The import converts to mono/44.1 kHz/16 bit, removes leading silence, fades out the end and normalizes the peak to −1 dBFS. At most 2.5 s (test).
2. In-game volume: section `[gain]` in `assets/sounds/sounds.toml` (1 = unchanged).
3. Add source and license to `assets/SOURCES.md` – only CC0 or GPL-3.0-compatible licenses.
4. Listen: `cargo xtask sound-preview jump` (into `target/sounds/`) or in the game (`cargo run --bin elora`, embedded at build time).
5. `cargo xtask check`, then commit.

Back to the procedural sound: delete the file and add an entry with layers in `sounds.toml` (otherwise the test fails).

**Music** lives as Ogg Vorbis in `assets/music/` (per region via `music`, `boss_music`, `party_music` in `assets/adventure/worldmap.toml`); sources in `assets/SOURCES.md`, CC-BY tracks must also be credited on the “About Elora” page.

## Packages and releases

`cargo xtask package --archive` builds a release package for your own system under `dist/` (programs, maps, music, licenses, icon; on macOS also `Elora.app`).

Windows package on Linux (cross build with mingw-w64):

```bash
sudo pacman -S mingw-w64-gcc            # Arch; Debian/Ubuntu: apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo xtask package --archive --target x86_64-pc-windows-gnu   # → dist/elora-<version>-windows-x86_64.zip
```

Start `elora.exe` from the extracted folder: the game data (`maps/`, `assets/`) lies next to it. If Elora stops, `%APPDATA%\Elora\crash.txt` and `elora.log` tell why.

A release:

1. Set the version in `Cargo.toml`, write release notes `docs/releases/v<version>.md`.
2. Commit, set the tag `v<version>` and push.
3. GitHub Actions builds Linux (tar.gz, AppImage), Windows (ZIP) and macOS (DMG) and creates a draft release with the notes.
4. Review the draft on GitHub and publish it by hand; upload the project page (`deploy/master-php/`) if needed.

## Files and folders

The game looks for shipped data (`maps/`, music) first in the working directory, then in `ELORA_DATA`, next to the program, in `../share/elora` or `../Resources` (macOS).

| File | Purpose | in Git |
|---|---|---|
| `settings.toml` | Player settings: name, skin, language, graphics, sound, effects, mouse, controls, favorites – in the settings folder, saved on exit | no, local |
| `tuning.toml` | Developer tuning of the sandbox: physics and view range (panel → *Speichern*) | no, local |
| `server.toml` | Server configuration written by the client | no |
| `server_key.toml` | Secret server key – do not share | no |
| `known_servers.toml` | Known server keys of the client | no |
| `maps/*.emap` | Maps | yes |

Settings folder: Linux `~/.config/elora`, Windows `%APPDATA%\Elora`, macOS `~/Library/Application Support/Elora`. Custom and downloaded maps and the adventure save games live under `~/.local/share/elora` (Windows/macOS in the settings folder).

## Project structure

```
crates/elora-sim        deterministic simulation (physics, hook, weapons, abilities, enemies)
crates/elora-map        map format and building the world from a map
crates/elora-render     wgpu 2D renderer (lyon tessellation), camera, post-processing
crates/elora-protocol   messages, delta snapshots, Huffman
crates/elora-net        UDP transport: handshake, encryption, reliability, simulator
crates/elora-game       game rules: modes, scores, rounds, teams, flags
crates/elora-adventure  adventure: save game, quests, dialogs, inventory, shops, session
crates/elora-audio      sounds: procedural generator, mapping to events, playback, music
apps/elora-client       the game (menu, sandbox, online, adventure, editor)
apps/elora-server       dedicated server
apps/elora-master       master server for the internet list
xtask                   development commands (cargo xtask …)
maps/                   maps (release, test, adventure)
assets/                 graphics, sounds, music, fonts, languages, adventure content
tools/design/           Python scripts for drafts and game graphics
deploy/                 master as a service and as PHP with project page
docs/                   handbook, Release 2 planning, release notes, Release 1 archive
```

Details: [Architecture](docs/handbook/architecture.md) · Principles: [`docs/handbook/principles.md`](docs/handbook/principles.md) · Documentation overview: [`docs/README.md`](docs/README.md).

## Workflow

- Decisions are recorded with a number (`E-…`): Release 2 in [`docs/release-2/decisions.md`](docs/release-2/decisions.md), Release 1 in the [archive](docs/archive/release-1/).
- Milestones run as plan → approval → drafts → implementation → acceptance (e.g. [`docs/release-2/m2-3-plan.md`](docs/release-2/m2-3-plan.md)).
- Commits follow Conventional Commits (`feat(adventure): …`, `fix(client): …`), each with a green `cargo xtask check`.
- Third-party content only CC0 or GPL-3.0-compatible, always with an entry in `assets/SOURCES.md`.

## Troubleshooting

| Problem | Solution |
|---|---|
| „kein passender Grafikadapter“ (no suitable graphics adapter) / window does not start | Install the Vulkan driver (see above); if necessary force OpenGL: `WGPU_BACKEND=gl cargo run --bin elora` |
| `elora-server … nicht gefunden` (not found) when hosting from the client | Run `cargo build` once (builds all programs) |
| `Port … nicht verfügbar` (not available) | Another server is already running on that port → change `--port` |
| `cargo xtask check` reports missing commands | Install `cargo-deny` and `cargo-nextest` (see setup) |
| Test „… veraltet“ (map out of date) | Generator changed, map not rewritten → run the matching `write_…_maps` test with `--ignored` |

## License

- Code: [GPL-3.0](LICENSE)
- Own graphics & sounds: CC-BY-SA 4.0
- Fonts (Inter, JetBrains Mono): SIL Open Font License 1.1
- Third-party content: [`assets/SOURCES.md`](assets/SOURCES.md) · Libraries: [THIRD_PARTY_LICENSES](THIRD_PARTY_LICENSES)

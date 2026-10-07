# M8 – Release 1: Implementation Plan

Status: **accepted** (E-169), in progress · Basis: [`06-roadmap.md`](06-roadmap.md) M8, E-003, E-027, E-161 to E-164, O-44, O-47, O-48

## Goal

Release 1 is published: packages for Linux, Windows and macOS on GitHub (E-161), a running master server for the internet list (E-162), translated server messages (E-164), license and credits page, balancing and playtests completed.

**Acceptance (roadmap):** Release 1 is published.

## Status

- Game, server, master and editor run from the project folder (`cargo run`).
- Several files are looked up **relative to the working directory**: `maps/`, `tuning.toml`, `known_servers.toml`, `assets/music/menu.wav`, and when hosting `elora-server` in the build folder. An installed package cannot find them this way.
- Version `0.1.0`, license GPL-3.0 (code), CC-BY-SA 4.0 for own assets (E-027), `THIRD_PARTY_LICENSES` present, sources of third-party assets in `assets/SOURCES.md`.
- Repository on GitHub (`ehrenberg/elora`), no CI yet.

## Work Steps

| # | Step | Content | Check |
|---|---|---|---|
| M8.1 | Translated server messages (O-48, E-164) | The server sends message codes with values instead of German texts (join, leave, map change, votes, kick, mode, round …); the client translates (DE/EN); console/log stay readable; protocol version 6 | Tests (all codes in both languages, integration test) · **implemented** |
| M8.2 | Packageability | Find data next to the program (folder `data/` or `Resources` in the macOS bundle), working directory irrelevant; writable files (`known_servers.toml`, own tuning, `server_key.toml` when hosting) go to the user directory; find `elora-server` next to the client | Tests + start from a different folder · **implemented** (server default map is now `dm-wiese`) |
| M8.3 | Release builds (E-163) | GitHub Actions workflow: on version tag `v*`, build and attach to a GitHub release – Linux (AppImage + tar.gz), Windows (ZIP), macOS (.app in DMG, unsigned, with a note on opening it); plus `cargo xtask package` for local packages; check workflow (fmt, clippy, test, deny) on every push | Workflow run on GitHub, packages start · **implemented** (`cargo xtask package`, `.github/workflows/`; macOS for now Apple Silicon only, O-50), run on GitHub open |
| M8.4 | Master server operation (O-47, E-162) | Ready-made operation files for `elora-master`: systemd service and Dockerfile, guide (HTTPS via reverse proxy, firewall, updates); default address in the client and the server | Local trial run; then your operation · **implemented** (E-170; `deploy/master/`, [`../../handbook/master-operation.md`](../../handbook/master-operation.md)), your operation open |
| M8.5 | Licenses & credits | Page in the main menu: license (GPL-3.0, CC-BY-SA 4.0), contributors, third-party assets (from `assets/SOURCES.md`), libraries (from `THIRD_PARTY_LICENSES`); license files in every package | Visual check · license files in the package implemented; menu page **after 0.9.0** (E-173) |
| M8.6 | Balancing & playtests | Game rounds with several people (UAT): tuning, maps, modes; collect feedback, record changes as decisions | Your playtest · **after 0.9.0** (E-173) |
| M8.7 | Release candidate | Bug fixing, set version, changelog, release notes, trial release (pre-release on GitHub) | Your acceptance · version 0.9.0, notes `docs/releases/v0.9.0.md`, tag `v0.9.0` |
| M8.8 | Publication | Set tag → GitHub release public (only after your explicit OK) | Release 1 published |

## Decisions for M8

| # | Question | Options | Decision |
|---|---|---|---|
| D-M8-01 | Distribution (O-44) | GitHub Releases / itch.io / Flathub / Steam | E-161: **GitHub Releases first**; further channels as an option (O-49) |
| D-M8-02 | Master server (O-47) | you operate it / none / later | E-162: **you operate it**, I provide operation files and a guide |
| D-M8-03 | Builds | GitHub Actions / local | E-163: **GitHub Actions** on version tag |
| D-M8-04 | Server messages (O-48) | translatable before Release 1 / later | E-164: **before Release 1** |
| D-M8-05 | Version number of Release 1 | e.g. `1.0.0` / `0.9.0` (beta) | E-165: **0.9.0 Beta** |
| D-M8-06 | Master server address | your domain, e.g. `https://master.example.org` | E-166: **https://elora.bastianswelt.de** (tentative) |
| D-M8-07 | macOS signing | unsigned (note "right-click → Open") / signed and notarized (Apple developer account, $99/year) | E-167: **unsigned** |
| D-M8-08 | Playtest rounds | Number, participants, procedure | E-168: project owner asks acquaintances |

# M7 – Menus & Infrastructure: Implementation Plan

Status: **completed** (E-128) · accepted (E-112–E-127) · Basis: [`06-roadmap.md`](06-roadmap.md) M7, E-031, E-082, E-111, O-17, O-18, O-45

## Goal

A new player starts Elora, finds a server without help and can play. This adds a main menu, settings (player and skin, controls, graphics, sound), key bindings and a server browser. The debug panel (egui) remains a developer tool (E-031).

**Acceptance (roadmap):** A new player finds a server without help and can play.

## What the Original Does (Teeworlds 0.7, `menus*.cpp`, `serverbrowser.cpp`, `mastersrv`)

| Area | Original |
|---|---|
| **Start** | Client starts into the main menu; the background is a running map with a slow camera pan |
| **Main menu** | Play (server browser), demos, editor, settings, quit |
| **Server browser** | Tabs Internet / LAN / Favorites; list with name, game type, map, players/max, ping; filters (empty, full, password, game type, ping, country); detail view with player list; direct connect by address |
| **Master server** | Servers register regularly with the master via UDP (heartbeat with token); the client fetches the address list there and queries **each server itself** for info (connectionless info packets, which also yield the ping). DDNet later switched to an HTTP/JSON master |
| **LAN** | Broadcast of the info request to ports 8303–8310 on the local network |
| **Settings** | Player (name, clan, country), Tee (skin parts, colors), controls (key bindings, mouse sensitivity), graphics (resolution, fullscreen, VSync, FSAA, texture quality), sound (volumes, music), general |
| **In-game menu (Esc)** | Game (choose team, spectate, disconnect), server info, votes (call vote), settings |
| **Demos** | Client records snapshots (`.demo`), playback with timeline, pause, speed |
| **Music** | Menu only (E-082 adopts this) |

## Status in Elora

- The client starts directly into the **sandbox**; connecting, hosting (server as a separate process, `hosting.rs`), team, votes, skin and name go through the **debug panel** (F1) or `--connect`.
- **Name and skin are not saved**; `tuning.toml` holds tuning, view, mouse, effects, sound.
- **Network:** connection setup with token (request padded to 512 bytes), Noise encryption, TOFU server keys. **There is no connectionless info query yet** – the server browser needs one (ping, name, map, players).
- **Game UI:** vector text, panels, icons in HUD style (M5.8); there are **no input elements** yet (buttons, sliders, text fields, lists) – they are created here as a small custom UI toolkit.

## Work Steps (Proposal)

| # | Step | Crate | Content | Check |
|---|---|---|---|---|
| M7.0 ✅ | Designs | – | 2–3 menu designs as images (E-123) | Your choice |
| M7.1 ✅ | UI toolkit | Client (`ui/`) | Button, toggle, slider, text field, list with selection and scrolling, tabs, keyboard/mouse focus; in HUD style, scaled with the window height | Tests + visual check |
| M7.2 ✅ | Settings & language | Client | `settings.toml` in the user directory (E-116): player, skin, controls, graphics, sound, language, favorites; translation files German/English (E-114) | Tests |
| M7.3 ✅ | Main menu & flow | Client | Main menu over a calm image (E-113), states menu ↔ game ↔ in-game menu, menu music prepared (E-121), "Create server" (E-122), training (sandbox) | Visual check |
| M7.4 ✅ | Settings pages | Client | Player & skin (preview of the character), controls, graphics (E-120: window/fullscreen, VSync, MSAA, UI scaling), sound, language | Visual check |
| M7.5 ✅ | Key bindings | Client | Actions instead of fixed keys, one key per action (E-117), rebinding by key press, showing conflicts | Tests |
| M7.6 ✅ | Server info query | `elora-net`, `elora-protocol`, server | Connectionless info request with token against amplification attacks; response: name, map, mode, players (names, scores), max, version | Tests |
| M7.7 ✅ | Server browser | Client | Internet (master, E-112), LAN (broadcast), favorites, direct connect; list, sorting, filters, details with player list, ping | Integration test + visual check |
| M7.8 ✅ | Master server | `elora-master` (new) | HTTP/JSON (E-112): servers register regularly (address, port; the master checks reachability via info query), `GET` returns the list as JSON, servers that stop registering expire | Tests |
| M7.9 ✅ | In-game menu | Client | Esc: resume, team/spectate, start vote, settings, disconnect, quit – replaces the game parts of the debug panel | Visual check |
| M7.10 ✅ | Acceptance | – | new player in the game without help | Your acceptance |

## Decisions for M7

| # | Question | Options | Decision |
|---|---|---|---|
| D-M7-01 | Menu design | 2–3 designs as images to choose from (as for HUD and character) | E-123: 2–3 designs as images (main menu, browser, settings) |
| D-M7-02 | Start and background | Main menu with a running map in the background / calm still image / straight into training | E-113: main menu over a calm, drawn image |
| D-M7-03 | Internet server list (O-17) | own UDP master (like the original) / HTTP/JSON master (like DDNet) / for Release 1 only LAN + favorites + direct | E-112: HTTP/JSON master (like DDNet); operation open (O-47) |
| D-M7-04 | Settings file | a new `settings.toml` for everything (tuning stays in `tuning.toml`) / everything in one file | E-116: `settings.toml` in the user directory |
| D-M7-05 | Key bindings | one key per action / two (primary + secondary) / any number | E-117: one key per action |
| D-M7-06 | Graphics settings | Scope: window/fullscreen, VSync, MSAA, UI scaling, FPS limit … | E-120: window/fullscreen, VSync, MSAA, UI scaling |
| D-M7-07 | Server password | introduce (browser shows a lock) / not for Release 1 | E-118: no password for Release 1 |
| D-M7-08 | Demos (O-18) | in M7 / later (after Release 1) / not at all | E-115: later |
| D-M7-09 | Menu music (E-082) | you provide the music (E-109) – I build playback with loop and volume | E-121: prepare playback (`assets/music/`, loop, own volume) |
| D-M7-10 | UI language | German only / German + English switchable / English only | E-114: German + English, switchable |
| D-M7-11 | Remote console (O-45) | in M7 (in-game menu, password) / later | E-119: later |
| D-M7-12 | Hosting from the menu | "Start server" in the menu (existing `hosting.rs`) / debug panel only | E-122: "Create server" in the menu |

## Technical Specifications (Proposal)

- **Own UI toolkit in immediate mode** (like egui, but with the vector renderer and in the game style): each page redraws itself every frame and returns actions – fits the existing HUD drawing and needs no extra library.
- **Info query** as a separate, unencrypted packet type next to the handshake: small request with token → response only to the requesting address and no larger than necessary; rate limit per address. The server identity is still verified by the handshake (TOFU).
- **States in the client:** `Menü` · `Spiel (Sandbox/Online)` · `Ingame-Menü` as a clear state machine; keys go to the menu or the game depending on the state.

## New Dependencies (Proposal, License Check via cargo-deny)

- **User directory:** `directories` (MIT/Apache-2.0) for the location of `settings.toml` per operating system.
- **HTTP (E-112):** client and server as HTTP clients (e.g. `ureq`, MIT/Apache-2.0, no mandatory TLS on LAN; TLS via `rustls` for the public master); master as a small HTTP service (e.g. `tiny_http`, MIT/Apache-2.0). Final choice after license and dependency review.
- **Fullscreen/VSync/MSAA:** available (winit, wgpu).

## Implementation Status

- **M7.0 Designs:** `design/elora-menue.png` (A/B/C); chosen (E-125): layout of C in the colors and shapes of B → `design/elora-menue-gewaehlt.png`. Generator `tools/design/elora_menu.py`.
- **M7.1 UI toolkit:** `apps/elora-client/src/ui.rs`, immediate mode with an ID per widget; state (focus, pressed widget, scroll) in `UiState`, input per frame in `UiInput`. Widgets: card, label, pill button (shadow, outline, lighter on hover, slides down onto its shadow when pressed), tabs (horizontal and as a sidebar), toggle, slider, text field (focus, cursor, umlauts, Backspace/Del/arrows/Home/End, Enter, Esc, max. length), color swatches, list with columns, selection, double click, mouse wheel and scrollbar. Theme per E-125. Gallery `design/elora-ui-toolkit.png`.
- **M7.2 Settings & language:** `settings.rs` – `settings.toml` in the user directory (location determined without an extra library: `directories` would have pulled in MPL-2.0 via `option-ext`), saved via a temporary file, on exit; contains language, name, skin, graphics (fullscreen, VSync, MSAA, UI scaling 0.5–2), sound, effects, mouse, favorites, last server. `tuning.toml` (now `tuning_file.rs`) only holds physics and view. Renderer: `set_vsync`, `set_msaa` at runtime. `lang.rs` + `assets/lang/{de,en}.toml` (sections, placeholder `{name}`, fallback German → key; test: same keys and placeholders); initial language from the system language. HUD and game displays translated. **Not translated:** messages the server sends as finished text (join, votes, round end) → O-48.
- **M7.3 Main menu & flow:** `menu.rs` (pages, pause menu) and `app_menu.rs` (integration with the app). State `Screen::Menu` / `Screen::Game`, with the pause menu on top (Esc or focus loss; Resume, To main menu, Quit – M7.9 extends it). Starts in the main menu; with a map, `--mode` or `--connect` on the command line directly into the game (development, then also with the debug panel). Top bar: Play, Training (starts the sandbox), Create server, Settings, Quit. "Play": welcome, "Quick play" (last server), direct connect with favorites (click adopts, double click connects, save/remove). "Create server": name, map, mode, instagib, max. 2–16 players → starts `elora-server` and connects. Background: sky, clouds, hills, your own Elora in the chosen skin and a second one. Menu music from `assets/music/menu.wav` in a loop (volume `music_volume`), off in game. Debug panel with F1. Image `design/elora-menue-umgesetzt.png`.
- **M7.4 Settings pages:** `menu_settings.rs`. Player (name, body/feet/eyes from the palette, large preview of the character), controls (mouse sensitivity 10–400 %; key bindings follow in M7.5), graphics (fullscreen, VSync, antialiasing, UI scaling 50–200 % in 5 % steps, camera shake, hit marker), sound (master and music volume, mute, notice when there is no audio device), language (Deutsch/English, notice about server messages). Changes take effect immediately (fullscreen, VSync and MSAA at runtime, language reloaded) and are saved – for sliders only on release. Image `design/elora-einstellungen-umgesetzt.png`.
- **M7.5 Key bindings:** `bindings.rs` – 17 actions (movement, hook, fire, three weapons, next/previous weapon, chat, team chat, scoreboard, emote wheel, suicide, vote yes/no) on key, mouse button or mouse wheel, one binding per action (E-117); saved as readable names under `[bindings]` (invalid entries → default). Fixed: Esc (pause), F1 (debug panel); R and F5 in the sandbox only as long as they are not bound to an action. "Controls" page: list in two columns, click → adopt the next key/mouse button/wheel direction (Esc cancels), duplicate bindings in red, "Restore defaults". Buttons choose the text color by brightness (dark on sand). Image `design/elora-steuerung-umgesetzt.png`.
- **M7.6 Server info query:** `elora-net/src/info.rs` (`InfoProbe`) and new packet types in the endpoint: request a token (existing request padded to 512 bytes – no amplification), then `[8][Token][Nonce]` → `[9][Nonce][Info]` only with a valid, address-bound token, at most 20 responses per IP and second; ping = time between info request and response; timeout 2 s, token request repeated every 0.5 s. LAN search: token request to broadcast addresses (`UdpSocket::set_broadcast`), every responding server is queried. Content (`elora-protocol/src/info.rs`, `ServerInfo`): protocol version, name, map, mode (e.g. `iCTF`), connected humans, maximum, up to 32 players with score, team, dummy flag (fits into one datagram). The server refreshes the info every second. Tests: ping, timeout, search, wrong token, integration test with player and dummies.
- **M7.8 Master server:** `apps/elora-master` (library + binary). Service with `tiny_http`: `POST /register` (`{"port", "version"}`; IP from the connection or, with `--behind-proxy`, from `X-Forwarded-For`), `GET /servers` (`{"servers": ["IP:Port", …]}`), `GET /`. Admission only after a successful UDP info query with a matching protocol version (`InfoProbe`); expiry after 60 s, re-registration no earlier than after 5 s, at most 32 servers per IP and 8192 in total, requests up to 4 KiB. HTTPS client (`ureq` with rustls, E-127) for registering and fetching the list; game servers register with `--master`/`masters` every 20 s (own thread, logs only changes). Local trial run: registration → check → list. Default address of the public master open (O-47).
- **M7.7 Server browser:** `browser.rs` (state, queries over its own UDP socket with broadcast, list from the master in its own thread) and `menu_browser.rs` (UI on "Play"). Tabs Internet (master list; without a configured master, a field to enter one, saved as `master_url`), LAN (broadcast to 255.255.255.255 and 127.0.0.1, ports 8303–8310) and Favorites; switching or "Refresh" reloads. Sorting by ping, players, name; filters "hide empty/full". List: name, map, mode, players, ping; "…" while querying, "unreachable" after 2 s, "different version" without connecting. Details: player list in team colors with scores and dummy flag, connect, save/remove favorite; double click connects; direct connect below. Image `design/elora-browser-umgesetzt.png`.
- **M7.9 In-game menu:** `menu_pause.rs`. Esc (or focus loss) opens the pause menu: on the left Resume, Settings (all settings pages directly in game, "Back"), To main menu, Quit; on the right, when online, a server line (address · mode), team (Red/Blue/Spectate or Play/Spectate; current team colored), suicide, start vote (map by name, mode with instagib, kick, spectator – player selected from the list) or Yes/No during a running vote; in training, respawn. After suicide, vote and respawn the game continues directly. The debug panel (F1) remains a developer tool. Image `design/elora-pause-umgesetzt.png`.

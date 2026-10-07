# Decision log

All decisions are made by the project owner. Every decision is recorded here with its date and rationale. Open items stay open until they are decided – **no assumptions** are made.

## Decisions made

| # | Date | Topic | Decision | Rationale |
|---|---|---|---|---|
| E-001 | 2026-09-25 | Goal | Clone of Teeworlds with an identical game feel | Requirement of the project owner |
| E-002 | 2026-09-25 | Documentation | Findings in the `docs/` folder | Requirement of the project owner |
| E-003 | 2026-09-25 | Goal (O-01) | Publication (public release, own identity) | Decision of the project owner |
| E-004 | 2026-09-25 | Platform (O-02) | Desktop: Linux, Windows, macOS | Decision of the project owner |
| E-005 | 2026-09-25 | Reference version (O-03) | Teeworlds 0.7 | Decision of the project owner |
| E-006 | 2026-09-25 | Assets (O-05) | Own assets, own style | Decision of the project owner |
| E-007 | 2026-09-25 | Code origin (O-23) | Written entirely from scratch; the Teeworlds source code serves as a reference for values/algorithms | Decision of the project owner |
| E-008 | 2026-09-25 | Protocol (O-04) | Own network protocol, no compatibility with the original | Decision of the project owner |
| E-009 | 2026-09-25 | Language/engine (O-07) | Rust with an own engine (server and client share code) | Decision of the project owner |
| ~~E-010~~ | 2026-09-25 | License (O-22) | ~~Open source, permissive~~ → **replaced by E-020** | Decision of the project owner |
| E-011 | 2026-09-25 | Rendering (O-25) | wgpu + winit, own 2D renderer | Decision of the project owner |
| E-012 | 2026-09-25 | Network (O-09) | Own UDP protocol (snapshots, delta compression, own reliability layer) | Decision of the project owner |
| E-013 | 2026-09-25 | First milestone (O-14/O-21) | Local physics sandbox: one Tee, test map, running/jumping/hook – tune the feel without networking | Decision of the project owner |
| E-014 | 2026-09-25 | Game modes Release 1 (O-12) | DM, TDM, CTF, LMS, LTS, Instagib | Decision of the project owner |
| E-015 | 2026-09-25 | Physics values (O-30) | Do **not** adopt the original tuning 1:1, but deviate somewhat (extent/direction → O-32) | Decision of the project owner; own identity |
| E-016 | 2026-09-25 | Weapons Release 1 (O-13) | Hammer, laser, grenade | Decision of the project owner |
| E-017 | 2026-09-25 | Test map (O-31) | Simple text format (syntax → O-33) | Decision of the project owner |
| E-018 | 2026-09-25 | Project name (O-06) | **Elora** – also the name of the playable character; distinct from Teeworlds | Decision of the project owner |
| E-019 | 2026-09-25 | Code structure (O-29) | Split according to professional Rust standards as described in [`../../handbook/architecture.md`](../../handbook/architecture.md) – **confirmed** | Decision of the project owner |
| E-020 | 2026-09-25 | License (O-24) | **GPL-3.0** (copyleft), replaces E-010 | Decision of the project owner |
| E-021 | 2026-09-25 | Physics arithmetic (O-28) | `f32` with quantization per tick (like the original) | Decision of the project owner |
| E-022 | 2026-09-25 | Approach to physics values (O-32) | Claude proposes a deviation with rationale per value, the project owner decides individually → [`../../handbook/tuning.md`](../../handbook/tuning.md) | Decision of the project owner |

| E-023 | 2026-09-25 | Tuning (O-32) | All proposals T-01 to T-30 from [`../../handbook/tuning.md`](../../handbook/tuning.md) accepted | Decision of the project owner |
| E-024 | 2026-09-25 | Map text format (O-33) | Proposal from [`../../handbook/map-format.md`](../../handbook/map-format.md) accepted (TOML + ASCII grid, legend, extension `.emap.toml`) – **only for test and development maps** | Decision of the project owner; too simple for Release 1 (no graphics layers) · **replaced by E-146** |
| E-025 | 2026-09-25 | Starting equipment (O-35) | Elora spawns **with the hammer only**; laser and grenade exclusively via pickup | Decision of the project owner; pickups and map control become important |
| E-026 | 2026-09-25 | Instagib rules | Classic: laser only, infinite ammo, one hit kills, no pickups | Decision of the project owner |
| E-027 | 2026-09-25 | Asset license (O-36) | **CC-BY-SA 4.0** for own graphics/sounds | Decision of the project owner; copyleft matching GPL-3.0 |
| E-028 | 2026-09-25 | Release map format + editor (O-10/O-16) | **Own format + own editor integrated into the game** (like Teeworlds) | Decision of the project owner; maximum control |
| E-029 | 2026-09-25 | Character & skins (O-34/O-19) | Elora is the base character; **skin system built from parts** (body, eyes, decoration etc., like 0.7) including community skins | Decision of the project owner |
| E-030 | 2026-09-25 | Graphics style | **Flat/vector**: clear shapes, modern palette, resolution-independent | Decision of the project owner; distinct from Teeworlds |
| E-031 | 2026-09-25 | UI (O-27) | **egui** for editor, console, debug sliders; **own game UI** for main menu, server browser, HUD | Decision of the project owner |
| E-032 | 2026-09-25 | Audio (O-26) | **kira** | Decision of the project owner |
| E-033 | 2026-09-25 | Vector pipeline (O-38) | **Runtime tessellation** (e.g. lyon → triangles → wgpu), truly resolution-independent, dynamic deformation possible | Decision of the project owner |
| E-034 | 2026-09-25 | Hosting (O-11) | Initially **local Git only**, hosting later | Decision of the project owner |
| E-035 | 2026-09-25 | Bots (O-15) | **After Release 1**; the architecture provides for them (bots supply inputs like players) | Decision of the project owner |
| E-036 | 2026-09-25 | Milestones (O-21) | Claude proposes a roadmap to Release 1, the project owner decides per milestone → [`06-roadmap.md`](06-roadmap.md) | Decision of the project owner |
| E-037 | 2026-09-25 | Roadmap (O-21) | Milestones M0–M8 from [`06-roadmap.md`](06-roadmap.md) accepted | Decision of the project owner |
| E-038 | 2026-09-25 | Rust toolchain | Switch to **rustup**; version pinned in `rust-toolchain.toml` | Decision of the project owner; reproducible for all developers |
| E-039 | 2026-09-25 | Local checks (O-41) | **`cargo xtask`** (Rust program in the workspace, platform-independent) | Decision of the project owner |
| E-040 | 2026-09-25 | Additional tools | **cargo-deny** (licenses, advisories) and **cargo-nextest** (tests) | Decision of the project owner |
| E-041 | 2026-09-25 | Workspace location | Directly in the project folder (`Cargo.toml`, `crates/`, `apps/`, `docs/` in the root) | Decision of the project owner |
| E-042 | 2026-09-25 | M0 acceptance | M0 project setup accepted (commit `5ab9055`) | Decision of the project owner |
| E-043 | 2026-09-25 | M1 plan | Plan from [`07-m1-plan.md`](07-m1-plan.md) including technical specifications accepted | Decision of the project owner |
| E-044 | 2026-09-25 | Camera (D-01) | **Static** like the 0.7 default: camera exactly on Elora (corrected, see analysis §9) | Decision of the project owner |
| E-045 | 2026-09-25 | View area (D-02) | Start with the original (1.15 million units², max. 1500 × 1050), as a **tuning slider** – final value in the M1 acceptance | Decision of the project owner |
| E-046 | 2026-09-25 | Saving tuning (D-03) | The sandbox saves values in **`tuning.toml`**, loaded at startup; defaults stay in the code | Decision of the project owner |
| E-047 | 2026-09-25 | Fonts | Do **not** use the egui default fonts (`epaint_default_fonts`, OFL-1.1 + Ubuntu Font Licence); instead **Inter** (UI) and **JetBrains Mono** (monospace), both OFL-1.1, as assets in `assets/fonts/` | Decision of the project owner; no special licenses in crate dependencies |
| E-048 | 2026-09-25 | Advisories | “unmaintained” reports from cargo-deny only as a **warning** (`-W unmaintained`), security vulnerabilities remain errors. Trigger: `ttf-parser` (RUSTSEC-2026-0192, indirectly via egui) | Decision of the project owner |
| E-049 | 2026-09-25 | M1 acceptance | Physics sandbox accepted: movement feel “perfect”, initial tuning values (E-023) stay unchanged | Decision of the project owner |
| E-050 | 2026-09-25 | M2 plan | Implement the plan from [`08-m2-plan.md`](08-m2-plan.md) including technical specifications | Decision of the project owner |
| E-051 | 2026-09-25 | Weapon selection (D-M2-01/02) | Keys **1 hammer, 2 grenade, 3 laser**; the mouse wheel cycles in this order | Decision of the project owner |
| E-052 | 2026-09-25 | Laser knockback (D-M2-03) | **Yes, slight**: push in the shot direction, initial value **2** (tuning slider) – deviation from the original (0) | Decision of the project owner |
| E-053 | 2026-09-25 | Training dummies (D-M2-04) | Dummies **from the map** with **movement patterns**: standing, walking back and forth, jumping, walking + jumping | Decision of the project owner |
| E-054 | 2026-09-25 | Dummy map characters | One character per pattern: `D` stands, `W` walks, `J` jumps, `X` walks + jumps (extension of E-024) | Decision of the project owner |
| E-055 | 2026-09-25 | Kill key (D-M2-05) | Only in **M4** with the game rules | Decision of the project owner |
| E-056 | 2026-09-29 | M2 acceptance | Local combat approved (“let's keep going”); respawn behavior as in the original (no earlier than 0.5 s by click, otherwise 3 s) stays, as there was no objection | Decision of the project owner |
| E-057 | 2026-09-29 | Prediction (D-M3-03) | Own movement/hook **and own weapons** (shots, laser beam, hammer effect, knockback) are predicted; damage/death only on the server; other players interpolated | Decision of the project owner |
| E-058 | 2026-09-29 | Lag compensation (D-M3-04) | **None** – like the original | Decision of the project owner |
| E-059 | 2026-09-29 | Capacity (D-M3-01/02) | **Up to 64 players** per server; snapshots **25 Hz**, **50 Hz as a LAN option** | Decision of the project owner |
| E-060 | 2026-09-29 | Local hosting (D-M3-06) | Server as a **separate process**; **setup and configuration of the server from within the client** (dialog, starts the process) | Decision of the project owner |
| E-061 | 2026-09-29 | Protection (D-M3-05) | **Token handshake + encryption** | Decision of the project owner |
| E-062 | 2026-09-29 | Server trust | **Like SSH (TOFU):** the client remembers the server key on first connect and warns when it changes; implemented with the Noise protocol `XX` (crate `snow`, Apache-2.0/MIT) | Decision of the project owner |
| E-063 | 2026-09-29 | Compression (D-M3-07) | “Better method than the original, otherwise Huffman” → **field-wise delta with change mask + compact numbers, followed by static Huffman with a table trained on our own traffic** (rationale: see `09-m3-plan.md`) | Decision of the project owner, method worked out by Claude |
| E-064 | 2026-09-29 | M3 plan | Implement the plan from [`09-m3-plan.md`](09-m3-plan.md) with the answers above | Decision of the project owner |
| E-065 | 2026-09-29 | M3 acceptance | Network accepted (“everything great for now”) | Decision of the project owner |
| E-066 | 2026-09-29 | Win condition (D-M4-01) | **Like the original:** score limit 20, no time limit, sudden death on a tie (CTF see E-067) | Decision of the project owner |
| E-067 | 2026-09-29 | CTF scoring (D-M4-02) | Individual points like the original (carrier +5, pickup/return/killing the carrier +1, kills like DM); **team score = captures, default limit 5** (instead of the raw value 100/capture) | Decision of the project owner |
| E-068 | 2026-09-29 | Warmup (D-M4-03) | **10 s warmup after a map change** (points do not count), then a **3 s countdown** (world frozen); countdown also before every further match/round | Decision of the project owner |
| E-069 | 2026-09-29 | Friendly fire (D-M4-04) | **On:** damage and knockback to team members, teamkill −1 (server setting, can be turned off) | Decision of the project owner |
| E-070 | 2026-09-29 | TDM respawn (D-M4-05) | no earlier than after **3 s** (like the original) | Decision of the project owner (part of the answer to D-M4-04) |
| E-071 | 2026-09-29 | Starting equipment LMS/LTS (D-M4-06) | **Hammer only + pickups**, as in all modes (E-025) | Decision of the project owner |
| E-072 | 2026-09-29 | Console (D-M4-07, O-20) | **Server console** (terminal) **+ votes**; remote console later | Decision of the project owner |
| E-073 | 2026-09-29 | Team selection (D-M4-08) | **Like the original:** join the smaller team, switching and spectating possible, automatic balancing after 1 min | Decision of the project owner |
| E-074 | 2026-09-29 | Rotation (D-M4-09) | **Like the original:** map list, matches per map, team swap after every match | Decision of the project owner |
| E-075 | 2026-09-29 | Sandbox modes (D-M4-10) | Game modes **selectable in the sandbox**, dummies get teams | Decision of the project owner |
| E-076 | 2026-09-29 | Instagib (D-M4-11) | **Toggle for all modes** (iDM, iTDM, iCTF, iLMS, iLTS) | Decision of the project owner |
| E-077 | 2026-09-29 | Votes | On **map, mode, kick (5 min ban), spectator**; procedure like the original (25 s) | Decision of the project owner |
| E-078 | 2026-09-29 | Keys (D-M4-12) | **T** chat, **Y** team chat, **Tab** scoreboard, **F3/F4** yes/no, **K** kill; votes and team selection in the panel | Decision of the project owner |
| E-079 | 2026-09-29 | M4 plan | Implement the plan from [`10-m4-plan.md`](10-m4-plan.md) with the answers above | Decision of the project owner |
| E-080 | 2026-09-29 | Creating graphics (D-M5-01) | **Claude generates SVG** (with self-review on the rendered image) **+ an external vector image API** for complex motifs; approval via screenshot by the project owner | Decision of the project owner |
| E-081 | 2026-09-29 | Sounds (D-M5-05) | **Procedurally generated + CC0 mixed** (generated for UI/simple effects, CC0 e.g. for explosion/hit; source list) | Decision of the project owner |
| E-082 | 2026-09-29 | Music (D-M5-06) | **Menu only**, comes with M7 | Decision of the project owner |
| E-083 | 2026-09-29 | Order within M5 (D-M5-11) | **Look first, then sound** | Decision of the project owner |
| E-084 | 2026-09-29 | Image API | **Recraft** (SVG output); API key only as an environment variable, never in the repo; check the terms of use before first use | Decision of the project owner |
| E-085 | 2026-09-29 | Elora shape (D-M5-03) | **Drop shape** – pointed at the top, strong squash & stretch when jumping/landing | Decision of the project owner |
| E-086 | 2026-09-29 | Skins (D-M5-04, O-39) | **Colors only + a few parts** from a fixed selection (e.g. body color, pattern, eye shape), **no community skins** – replaces the community part of E-029 | Decision of the project owner |
| E-087 | 2026-09-29 | Display size | **Slightly smaller than the original:** visible body ≈ 36 units (hitbox 28) | Decision of the project owner |
| E-088 | 2026-09-29 | Extra effects (D-M5-07) | **Camera shake** (nearby explosions, own damage) and **hit markers** when hitting others – both can be turned off; exact form in the draft | Decision of the project owner |
| E-089 | 2026-09-29 | World look of text maps (D-M5-08) | **Plain:** single-color tiles with outline, gradient background; effort goes into M6 | Decision of the project owner |
| E-090 | 2026-09-29 | HUD (D-M5-09) | **Modern, at the crosshair:** health/armor/ammo as bars or rings at the crosshair or bottom center; exact form in the draft | Decision of the project owner |
| E-091 | 2026-09-29 | Emotes (D-M5-10) | **Emote wheel with 8 own emoticons** (hold E, the mouse selects) + automatic eye expressions | Decision of the project owner |
| E-092 | 2026-09-29 | M4 acceptance | Game modes “done for now” | Decision of the project owner |
| E-093 | 2026-09-29 | M5 plan | Implement the plan from [`11-m5-plan.md`](11-m5-plan.md) | Decision of the project owner |
| E-094 | 2026-09-29 | Elora draft (M5.3) | **Draft B “Swirl”:** drop with its tip tilted to the side, light belly patch, small eyes with a smile ([`design/elora-entwuerfe.png`](design/elora-entwuerfe.png)) | Decision of the project owner |
| E-095 | 2026-09-29 | Skin parts (O-46) | Colorable are **eyes, body, feet** – **colors only**, no shape variants per part | Decision of the project owner |
| E-096 | 2026-09-29 | Color choice (O-46) | **Fixed palette** per part, no free sliders; palette colors as a draft for approval | Decision of the project owner |
| E-097 | 2026-09-29 | Belly patch (O-46) | Not a separate part – **lighter shade of the body color** | Decision of the project owner |
| E-098 | 2026-09-29 | Skin palette | Draft [`design/elora-palette.png`](design/elora-palette.png) approved: 16 colors for body and feet, 8 for eyes | Decision of the project owner |
| E-099 | 2026-09-29 | Skins in team modes | Body in the **team color**, feet and eyes keep the player's colors | Decision of the project owner |
| E-100 | 2026-09-29 | Huffman table | The newly trained table (commit `3a550bc`) is kept | Decision of the project owner |
| E-101 | 2026-09-29 | Pickups, weapons, flags (M5.5) | **Style A “Round”** from [`design/elora-items.png`](design/elora-items.png) | Decision of the project owner |
| E-102 | 2026-09-29 | HUD (M5.8) | **Draft B “Bar at the bottom center”** from [`design/elora-hud.png`](design/elora-hud.png); in addition, the **crosshair is colored by health: white → yellow → red** (smoothly) | Decision of the project owner |
| E-103 | 2026-09-29 | Emotes (M5.9) | Proposal from [`design/elora-emotes.png`](design/elora-emotes.png) approved: heart, laugh, anger, sad, amazement, question, GG, sleep; wheel on key E, shown for approx. 2 s | Decision of the project owner |
| E-104 | 2026-09-29 | Eye expressions (M5.9) | Eyes **additionally react automatically**: squinting on damage, happy after a kill | Decision of the project owner |
| E-105 | 2026-09-29 | Look (M5) and approach to sound (M5.7) | Look accepted in the playtest (“great”); sound as proposed: crate `elora-audio` with kira, procedural generator (sfxr principle, parameters in `assets/sounds/`), WAV samples for approval, CC0 only where needed | Decision of the project owner |
| E-106 | 2026-09-30 | Listening test of procedural sounds | Keep: `spawn`, `death`, `weapon_switch`, `pickup_weapon`. All others sound “too much like computer sounds” → replace with CC0 sounds, the goal is atmosphere | Decision of the project owner |
| E-107 | 2026-09-30 | CC0 sounds | Source **Kenney** (base set) **+ Freesound CC0 only** (gaps; Freesound dropped → E-108); sound style **organic / soft** (plops, goo, wood, cloth, natural hits) | Decision of the project owner |
| E-108 | 2026-09-30 | Freesound | **Without Freesound** – Kenney packs only; sounds may be edited and combined. Listening test 2: the hammer should sound like a hammer swing, the grenade like an explosion, the laser more organic, the jump more fitting; the rest is fine | Decision of the project owner |
| E-109 | 2026-09-30 | Sounds (M5.7) | State “fine for now” – M5.7 completed. **From now on, the project owner sources new sounds himself** (procedure in the README “Replacing sounds”) | Decision of the project owner |
| E-110 | 2026-09-30 | M5 acceptance | **M5 completed** (look accepted in the playtest, sounds fine for now). Playtest recording `rec-1790721458` adopted as a golden regression test | Decision of the project owner |
| E-111 | 2026-09-30 | Order M6/M7 | **M7 (menus & infrastructure) before M6 (maps & editor)**; numbers stay, order M5 → M7 → M6 → M8 | Decision of the project owner |
| E-112 | 2026-09-30 | Internet server list (D-M7-03, O-17) | **HTTP/JSON master** like DDNet: servers register via HTTP, the client fetches the list as JSON and queries each server itself (ping, info) | Decision of the project owner |
| E-113 | 2026-09-30 | Start (D-M7-02) | **Main menu over a calm, drawn image** | Decision of the project owner |
| E-114 | 2026-09-30 | Language (D-M7-10) | **German + English**, switchable; texts in one translation file per language | Decision of the project owner |
| E-115 | 2026-09-30 | Demos (D-M7-08, O-18) | **Later** (after Release 1) | Decision of the project owner |
| E-116 | 2026-09-30 | Settings file (D-M7-04) | **`settings.toml` in the user directory** (Linux `~/.config/elora`, otherwise the usual location per system) for player, skin, controls, graphics, sound, language, favorites; `tuning.toml` remains developer tuning | Decision of the project owner |
| E-117 | 2026-09-30 | Key bindings (D-M7-05) | **One key per action** | Decision of the project owner |
| E-118 | 2026-09-30 | Server password (D-M7-07) | **No** for Release 1 | Decision of the project owner |
| E-119 | 2026-09-30 | Remote console (D-M7-11, O-45) | **Later** | Decision of the project owner |
| E-120 | 2026-09-30 | Graphics settings (D-M7-06) | **Window/fullscreen, VSync, anti-aliasing (MSAA), UI scaling** | Decision of the project owner |
| E-121 | 2026-09-30 | Menu music (D-M7-09) | **Prepare playback:** a file in `assets/music/` loops in the menu with its own volume; silent without a file. Music is supplied by the project owner (E-109) | Decision of the project owner |
| E-122 | 2026-09-30 | Hosting (D-M7-12) | **“Create server” in the menu** (name, map, mode, players), starts `elora-server` in the background and connects | Decision of the project owner |
| E-123 | 2026-09-30 | Menu design (D-M7-01) | **2–3 drafts as images** (main menu, server browser, settings page) before implementation | Decision of the project owner |
| E-124 | 2026-09-30 | M7 plan | Plan from [`12-m7-plan.md`](12-m7-plan.md) approved, implementation starts with M7.0 (drafts) | Decision of the project owner |
| E-125 | 2026-09-30 | Menu style (M7.0) | **Layout of C “Bar at the top”** (tab bar at the top, start page with “Quick play”, settings with a sidebar) **in the colors and shapes of B “Light & soft”** (cream-colored cards, soft shadow, colored pill buttons) | Decision of the project owner |
| E-126 | 2026-09-30 | M7.0–M7.4 | Menu, settings and language “look good” in testing – continue with M7.5 | Decision of the project owner |
| E-127 | 2026-09-30 | Master server connection (M7.8) | **HTTPS** for clients and servers (rustls, Mozilla root certificates); the master speaks HTTP and runs publicly behind a reverse proxy with a certificate. License **CDLA-Permissive-2.0** (webpki-roots) is allowed | Decision of the project owner |
| E-128 | 2026-10-01 | M7 acceptance | **M7 completed** (“looks good”) – continue with M6 (E-111) | Decision of the project owner |
| E-129 | 2026-10-01 | Release map format (D-M6-01, O-37) | **Binary** (compact, like the original) | Decision of the project owner |
| E-130 | 2026-10-01 | Map look (D-M6-02) | **Both:** vector tiles with automatic edges/corners per material for the playing area + freely placeable vector decoration | Decision of the project owner |
| E-131 | 2026-10-01 | Layers (D-M6-03) | **Simplified:** game layer + decoration layers in front/behind + background layers with parallax | Decision of the project owner |
| E-132 | 2026-10-01 | Animations (D-M6-04) | **Like the original:** freely editable curves (envelopes) for position, rotation, color | Decision of the project owner |
| E-133 | 2026-10-01 | Editor scope (D-M6-05) | **Everything:** basic tools, decoration and background layers, animation editor, test play | Decision of the project owner |
| E-134 | 2026-10-01 | Release maps (D-M6-06, O-43) | **3 DM** (small, medium, large; also for TDM/LMS) **+ 2 CTF** | Decision of the project owner |
| E-135 | 2026-10-01 | Map building | **Claude builds the maps according to the project owner's specifications**, acceptance in the playtest | Decision of the project owner |
| E-136 | 2026-10-01 | Map download (D-M6-07) | **Automatic:** the server sends missing maps compressed in parts, checksum, cache in the client | Decision of the project owner |
| E-137 | 2026-10-01 | New tile types (D-M6-08) | **Platform** (passable from below/the side), **ice** (slippery), **jump pad** (launches upward/diagonally), **booster** (conveyor belt) – values as a tuning proposal for approval | Decision of the project owner |
| E-138 | 2026-10-01 | M6 plan | Plan from [`13-m6-plan.md`](13-m6-plan.md) approved, starting with M6.0 (map look drafts) | Decision of the project owner |
| E-139 | 2026-10-01 | Map look (M6.0) | **Style A “Soft & lively”** from [`design/elora-kartenlook.png`](design/elora-kartenlook.png): soil with turf, rounded outer corners, outline, bushes/flowers, cloud and hill layers; special tiles are adapted to the style | Decision of the project owner |
| E-140 | 2026-10-01 | Tuning of new tiles (M6.1) | Proposal accepted: **T-31** ice friction 0.985 · **T-32** ice acceleration 0.35 · **T-33** jump pad force 20 · **T-34** directions up/diagonal left/diagonal right (45°) · **T-35** booster 4.0 units/tick · **T-36** platform passable from below/the side, hook/grenade/laser pass through | Decision of the project owner |
| E-141 | 2026-10-01 | Dropping through platforms | New bindable action **“Down”** (default S); the player input gets a field for it (**protocol version 3**) | Decision of the project owner |
| E-142 | 2026-10-01 | M6.1 accepted | New tile types in the playtest “feel perfect”; values T-31 to T-36 stay | Decision of the project owner |
| E-143 | 2026-10-01 | Map format compression (M6.2) | **Deflate/zlib** like the original (pure Rust, `miniz_oxide`) | Decision of the project owner |
| E-144 | 2026-10-01 | Own graphics in maps | **Embedded SVGs allowed** (own decoration); limits 64 images × 512 KiB, the client parses without external references | Decision of the project owner |
| E-145 | 2026-10-01 | File extension | **`.emap`** | Decision of the project owner |
| E-146 | 2026-10-01 | Text format | **Text maps converted, text format `.emap.toml` removed** (replaces E-024); maps are built with the editor, tests use `Map::from_rows`; protocol version 4 (map as binary data in `Welcome`) | Decision of the project owner |
| E-147 | 2026-10-01 | Map graphics (M6.3) | Materials soil/grass, sand, snow, stone (not hookable), ice; special tiles death, wooden platform, jump pad, booster; decoration bushes, flowers, grass, stones, mushrooms, trees, fence, signs; backgrounds clouds, hills, mountains, forest, night sky | Decision of the project owner |
| E-148 | 2026-10-01 | M6.3 accepted | Map graphics accepted after rework (stone, booster arrow, bushes, tree, forest); **stone only for unhookable walls**, hookable are soil, sand, snow | Decision of the project owner |
| E-149 | 2026-10-01 | M6.4 accepted | Map look in the game (parallax, decoration, animations, cache) accepted in the playtest | Decision of the project owner |
| E-150 | 2026-10-01 | Editor look (M6.6) | **egui dark** (default dark theme), stands apart from the game | Decision of the project owner |
| E-151 | 2026-10-01 | Editor layout | **Everything on the right:** one sidebar with tools, layers and properties; the map takes the rest | Decision of the project owner |
| E-152 | 2026-10-01 | Storage location of own maps | **User directory** (`~/.local/share/elora/maps`, Windows/macOS in the settings folder); training, hosting and download lookup find them there | Decision of the project owner |
| E-153 | 2026-10-01 | M6.5 and M6.6 accepted | Map download and editor foundation accepted (“looks great”) | Decision of the project owner |
| E-154 | 2026-10-01 | M6.7 accepted | Editor tools accepted (“looks good”) | Decision of the project owner |
| E-155 | 2026-10-01 | Themes of the release maps (M6.10) | **One theme per map:** DM small meadow (soil, day) · DM medium desert (sand) · DM large winter (snow, ice) · CTF 1 forest (day) · CTF 2 night (stone, stars) | Decision of the project owner |
| E-156 | 2026-10-01 | Size of the release maps | **Rather large** (more space than vanilla, for full servers) | Decision of the project owner |
| E-157 | 2026-10-01 | New tile types in release maps | **Targeted:** platforms as a building element, ice/jump pad/booster at one or two distinctive spots per map | Decision of the project owner |
| E-158 | 2026-10-01 | Layout of the CTF maps | **Mirror-symmetric** (red on the left, blue on the right) | Decision of the project owner |
| E-159 | 2026-10-01 | M6.10 accepted | Release maps `dm-wiese`, `dm-wueste`, `dm-winter`, `ctf-wald`, `ctf-nacht` accepted, names stay | Decision of the project owner |
| E-160 | 2026-10-01 | M6 completed (M6.11) | Acceptance met: the project owner built an own map from scratch with the editor and played it | Decision of the project owner |
| E-161 | 2026-10-01 | Distribution of Release 1 (O-44) | **GitHub Releases first**; itch.io, Flathub, Steam recorded as later options (O-49) | Decision of the project owner |
| E-162 | 2026-10-01 | Operating the master server (O-47) | **The project owner operates it**; Claude supplies operations files (systemd, Docker) and instructions | Decision of the project owner |
| E-163 | 2026-10-01 | Release builds | **Automatically via GitHub Actions** on a version tag: Linux (AppImage + tar.gz), Windows (ZIP), macOS (.app in a DMG) | Decision of the project owner |
| E-164 | 2026-10-01 | Translated server messages (O-48) | Implement **before Release 1** | Decision of the project owner |
| E-165 | 2026-10-01 | Version number of Release 1 (D-M8-05) | **0.9.0 Beta** | Decision of the project owner |
| E-166 | 2026-10-01 | Address of the master server (D-M8-06) | presumably **https://elora.bastianswelt.de** (default in client and server) | Decision of the project owner |
| E-167 | 2026-10-01 | macOS signing (D-M8-07) | **Unsigned** (note “right-click → Open”) | Decision of the project owner |
| E-168 | 2026-10-01 | Playtests (D-M8-08) | The project owner asks acquaintances for play sessions | Decision of the project owner |
| E-169 | 2026-10-01 | M8 plan | Plan from [`14-m8-plan.md`](14-m8-plan.md) accepted (open items answered), starting with M8.1 | Decision of the project owner |
| E-170 | 2026-10-01 | Registering with the master | **Dedicated servers yes** (can be turned off with `--no-master`), **servers hosted from the client no**, unless “Show on the internet” is checked | Decision of the project owner |
| E-171 | 2026-10-01 | Master on web hosting | `elora.bastianswelt.de` is **web hosting with PHP** → master additionally as a PHP script (`deploy/master-php/`), same interface | Decision of the project owner |
| E-172 | 2026-10-01 | Project page | One-pager on `elora.bastianswelt.de` (`deploy/master-php/index.php`): German, graphics from the project SVGs, live server status, link to github.com/ehrenberg/elora, no legal notice (Impressum) | Decision of the project owner |
| E-173 | 2026-10-01 | Release 0.9.0 Beta | The project owner releases 0.9.0 once the master works over IPv6; credits page in the menu (M8.5) and playtests (M8.6) follow after the beta | Decision of the project owner |

> **Note on E-020:** GPL-3.0 is compatible with the Teeworlds license (zlib-like) and with MIT/Apache-licensed Rust crates (wgpu, winit, …). Dependencies under GPL-2.0-only would not be compatible – `cargo-deny` checks this. The own assets (E-006) need their own license (→ O-36).

> **Note on E-007:** The Teeworlds license (zlib-like) allows adoption/adaptation, but requires that modified versions are marked as such and that the license notice is retained. If algorithms are ported 1:1 to Rust, the Teeworlds license notice should be carried in the project as a precaution (file `THIRD_PARTY_LICENSES`). Pure numeric values (tuning) are uncritical.

## Open decisions

### Foundations
- [x] ~~O-01 Goal/scope~~ → E-003
- [x] ~~O-02 Platform~~ → E-004
- [x] ~~O-03 Reference version~~ → E-005
- [x] ~~O-04 Compatibility~~ → E-008
- [x] ~~O-05 Assets~~ → E-006
- [x] ~~O-06 Project name~~ → E-018
- [x] ~~O-22 License~~ → E-010
- [x] ~~O-23 Code origin~~ → E-007
- [x] ~~O-24 Concrete license~~ → E-020
- [x] ~~O-36 License of the assets~~ → E-027

### Technology
- [x] ~~O-07 Programming language / engine~~ → E-009
- [x] ~~O-25 Graphics/window library~~ → E-011
- [x] ~~O-26 Audio library~~ → E-032
- [x] ~~O-27 UI solution~~ → E-031
- [x] ~~O-38 Vector pipeline~~ → E-033
- [x] ~~O-40 Vector source format~~ → SVG (E-080, E-084)
- [x] ~~O-09 Network transport~~ → E-012
- [x] ~~O-28 Physics arithmetic~~ → E-021
- [x] ~~O-29 Rust workspace structure~~ → E-019
- [x] ~~O-10 Release map format~~ → E-028
- [x] ~~O-37 Details of the release map format~~ → E-129, E-131, E-132
- [x] ~~O-11 Version control/hosting~~ → E-034 (repo structure → E-019)
- [x] ~~O-41 CI without hosting~~ → E-039

### Gameplay scope
- [x] ~~O-12 Game modes~~ → E-014
- [x] ~~O-13 Weapons~~ → E-016
- [x] ~~O-14 Multiplayer vs. sandbox first~~ → E-013
- [x] ~~O-30 Physics values~~ → E-015
- [x] ~~O-31 Sandbox test map~~ → E-017
- [x] ~~O-32 Physics values in detail~~ → E-023
- [x] ~~O-33 Syntax of the map text format~~ → E-024
- [x] ~~O-34 Elora as a character~~ → E-029
- [x] ~~O-39 Skin structure~~ → E-086 (details → O-46)
- [x] ~~O-46 Skin selection in detail~~ → E-095, E-096, E-097
- [x] ~~O-35 Starting equipment~~ → E-025
- [x] ~~O-15 Bots~~ → E-035
- [x] ~~O-16 Map editor~~ → E-028 (timing → milestones O-21)

### Features / infrastructure
- [x] ~~O-17 Server browser / master server~~ → E-112 (operating the master → O-47)
- [x] ~~O-18 Demos / replays~~ → E-115 (later)
- [x] ~~O-19 Skin system~~ → E-029
- [x] ~~O-20 Console~~ → E-072 (remote console later → O-45)
- [ ] **O-45 Remote console** (admin commands from the client with a password) – later (E-119)
- [x] ~~O-48 Translated server messages~~ → E-164 (implementation M8.1)
- [x] ~~O-47 Operating the master server~~ → E-162 (address open: D-M8-06)
- [x] ~~O-42 Network targets~~ → E-059
- [x] ~~O-43 Release maps~~ → E-134, E-135
- [x] ~~O-44 Distribution~~ → E-161
- [ ] **O-50 macOS on Intel** – release builds initially only for Apple Silicon; add Intel Macs later via cross-build or a dedicated runner
- [ ] **O-49 Further distribution channels** (itch.io, Flathub, Steam, own website) – after Release 1 (E-161)
- [x] ~~O-21 Milestones~~ → E-037

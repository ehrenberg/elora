# M5 – Look & Sound: Implementation Plan

Status: **completed** (E-110) · accepted (E-080–E-109) · Basis: [`06-roadmap.md`](06-roadmap.md) M5, E-006, E-027, E-029, E-030, E-031, E-032, E-033

## Goal

Elora gets her own look in a flat/vector style (E-030), with animations, skins, effects, sounds and a final HUD. Acceptance: style and feedback are right, and Elora is clearly distinguishable from a Tee.

## What Defines the Original (source code `render.cpp`, `players.cpp`, `content.py`)

| Area | Original (0.7) |
|---|---|
| **Size** | Character drawn at **64 units** (sprite), visible body ≈ 40–45, hitbox only 28 – the character looks bigger than it hits |
| **Structure** | Body, marking, decoration, hands, feet, eyes; outline and fill pass; shadow and upper outline |
| **Animations** | `idle`, `inair`, `walk` (feet), `hammer_swing`; body rotates slightly; eyes look in the aim direction |
| **Eyes/emotes** | normal, pain, happiness, surprise, anger, blink; 16 emoticons above the head (emote wheel) |
| **Skins (0.7)** | Parts body, marking, decoration, hands, feet, eyes – each part with its own color (HSL); community skins as part sets |
| **Effects** | Smoke trail behind grenades, explosion with smoke, hammer hit, blood in body color, air jump clouds, dust on landing |
| **Sounds** | 40 sounds: per weapon fire/hit, laser bounce, hook (loop, wall, player, no grip), jump, air jump, landing, pain short/long, death, spawn, pickups, no ammo, hit confirmation, chat, CTF (drop, return, pickup own/enemy, capture), menu |
| **Positional audio** | Volume by distance to the camera, stereo panning |
| **HUD** | Hearts and shields top left, ammo as icons, weapon icon; score and timer at the top |

## Work Steps

| # | Step | Crate | Content | Check |
|---|---|---|---|---|
| M5.1 ✅ | Vector renderer | `elora-render` | Transformations (translate, rotate, scale, shear), gradients, antialiasing (MSAA), shape cache (tessellated once, drawn often), camera zoom | Visual check, benchmark |
| M5.2 ✅ | Vector assets | `elora-render` + `assets/` | Loading the source format (D-M5-02), color keys for tinting | Tests |
| M5.3 | Elora character | Client | Character from parts (D-M5-03), eyes follow the aim, feet walk, squash & stretch (jump, landing, hook), weapon in hand, emotes | Visual check |
| M5.4 ✅ | Skins | Client + protocol | Skin parts and colors (D-M5-04), selection in the client, transmission to other players | Tests + visual check |
| M5.5 ✅ | World look | Client | Tiles with edges and corners instead of rectangles, background, pickups, flags, weapons as graphics (D-M5-08) | Visual check |
| M5.6 ✅ | Effects | Client | Particle system: smoke, explosions, hits, death, dust, jump clouds, laser beam (D-M5-07) | Visual check |
| M5.7 ✅ | Audio | `elora-audio` (new) | kira integration, sounds from events, volume by distance + stereo, volume slider (D-M5-05, D-M5-06) | Tests (event → sound mapping), listening test |
| M5.8 ✅ | Final HUD | Client | Own game UI (E-031) instead of egui placeholders: health, armor, ammo, weapons, timer, score, killfeed, chat, scoreboard (D-M5-09) | Visual check |
| M5.9 ✅ | Emotes | Client + protocol | Emote wheel and emoticons above the head, transmitted to everyone (D-M5-10) | Test + visual check |
| M5.10 ✅ | Acceptance | – | Style, feedback, sound | Your acceptance |

## Technical Specifications (Proposal)

- **Everything tessellated at runtime (E-033), but cached:** A shape is split into triangles once and then drawn via transformation (position, rotation, scale, shear) – this keeps 64 players with particles smooth.
- **Tinting via color keys:** Skin parts are drawn in grayscale or with placeholder colors; the client replaces them with the chosen colors (like HSL coloring in the original).
- **New crate `elora-audio`** with kira (MIT/Apache-2.0, E-032). Sounds are triggered only from the events of the simulation and the rules – the same events that already feed effects and the killfeed.
- **No influence on the simulation:** display size, animations and effects are purely visual; hitbox (28) and physics stay unchanged.

## Decisions for M5

| # | Question | Decision |
|---|---|---|
| D-M5-01 | Who creates graphics? | E-080: Claude generates SVG + vector image API for complex motifs, approval via screenshot |
| – | Image API | E-084: Recraft (SVG), key only as an environment variable |
| D-M5-02 | Source format | SVG |
| D-M5-03 | Elora's shape | E-085: drop shape with strong squash & stretch |
| – | Display size | E-087: visible body ≈ 36 (hitbox 28) |
| D-M5-04 | Skins | E-086: colors + a few parts only, no community skins; E-095: eyes, body, feet, colors only; E-096: fixed palette; E-097: belly patch derived from body color; E-098: palette approved; E-099: team color for the body only |
| – | Elora design | E-094: design B "Swirl" |
| D-M5-05 | Sounds | E-081: procedurally generated + CC0 mixed |
| D-M5-06 | Music | E-082: menu only (M7) |
| D-M5-07 | Extra effects | E-088: camera shake, hit marker (can be disabled) |
| D-M5-08 | Tiles/background | E-089: simple |
| D-M5-09 | HUD | E-090: modern, at the crosshair |
| D-M5-10 | Emotes | E-091: emote wheel with 8 custom emoticons |
| D-M5-11 | Order | E-083: look first, then sound |

## Adjusted Steps

- **M5.2 Vector assets:** load SVG (`usvg`, Apache-2.0/MIT) → lyon tessellation; color keys for tinting. Tool `cargo xtask svg-preview` rasterizes SVGs to PNG (for my self-check and your approval). Recraft integration as an `xtask` command, only with `RECRAFT_API_KEY` set; generated SVGs are cleaned up and recorded with their source in `assets/SOURCES.md`.
- **M5.3 Elora:** first **3 designs of the drop shape** as an image to choose from.
- **M5.4 Skins:** one color per part (eyes, body, feet) from a fixed palette (E-095/E-096); only three palette indices are transmitted. Palette as a draft for approval.
- **M5.5 World:** simple (E-089).
- **M5.7 Audio:** procedural generator (sfxr-like, parameters as files in `assets/sounds/`) for UI/simple effects; CC0 sounds with a source list in `assets/SOURCES.md`.
- **M5.8 HUD:** 2–3 designs at the crosshair to choose from.
- **M5.9 Emotes:** 8 custom emoticons, emote wheel on key E.

## Implementation Status

- **M5.1 Vector renderer:** `Mesh` (tessellated once, local coordinates) + `Affine` (translate, rotate, scale/shear) + `Paint` (color, linear gradient, color key with lightening) + `Tint` (skin colors, opacity). 4× MSAA with fallback to 1 if the format does not support it. `Camera::zoomed`. Benchmark `cargo run --release -p elora-render --example bench_meshes`: 64 characters (316 triangles each) + 2000 particles = 34 224 triangles, CPU build ≈ 0.09 ms/frame.
- **M5.2 Vector assets:** `SvgAsset::load` (usvg → lyon). Conventions: `viewBox` = local coordinates (origin freely chosen), top-level groups with `id` = individually animatable parts, `id="tint-<n>[-l<%>|-d<%>]"` = color key with lightening/darkening (the preview color stays in the file), strokes keep their color. Supported: colors, linear gradients with two colors, opacity, fill rules. Rejected with an error: radial gradients, patterns, images, text. The Recraft integration only follows once a motif needs it (then review of the terms of use, key `RECRAFT_API_KEY`).
- **M5.3 Elora:** design B "Swirl" chosen (E-094); design sheet `design/elora-entwuerfe.png`, generator `tools/design/elora_entwuerfe.py`. Asset `assets/elora/elora.svg` with parts `foot-back`, `foot-front`, `body`, `eyes`; origin = ground contact at the bottom edge of the hitbox, scale 0.36. Animation in the client (`figure.rs`): damped spring for squash & stretch (impulse on jump and landing) plus stretching in the air, tilt in the walking direction or toward the hook, walk cycle of the feet by distance, eyes follow the aim, blinking, mirroring by facing direction. Pose sheet `design/elora-posen.png` (generated with `cargo test -p elora-client --bin elora pose_sheet -- --ignored`). Weapon in hand with M5.5, eye expressions with M5.9.

- **M5.4 Skins:** `Skin { body, feet, eyes }` (palette indices) in the protocol (version 2): in `Join`, new message `SetSkin`, in `PlayerInfo` to everyone; invalid indices are rejected on decoding. Palette in the client (`skins.rs`), selection in the debug panel under "Appearance" (like the name, not yet saved – a profile follows with the menu in M7). Dummies keep their own body color. Integration test: the skin is transmitted to everyone on join and on change.

- **M5.5 World (part 1):** sky as a vertical gradient across the visible area, tiles in a single color with a 3-unit outline on every edge bordering a different tile type (E-089). Visual check via `cargo test -p elora-client --bin elora world_sheet -- --ignored`. Design sheet for pickups, weapons and flags: `design/elora-items.png` (style A round, style B angular) → style A chosen (E-101).

- **M5.5 World (part 2):** assets `assets/items/*.svg` (world units; weapons with origin at the grip, flag with parts `pole`/`cloth` and team color as `tint-1`). The weapon in hand points in the aim direction, mirrored to the left; pickups float (2.5 units, 0.6 Hz, phase by position); the flag cloth waves via shearing around its attachment. Overall image `design/elora-welt.png`. Hammer swing on a strike: winds back 1.4 rad up and backward and strikes toward the target in 0.14 s.

- **M5.6 Effects:** own particle system (`effects.rs`, one cached circle per particle). Explosion: flash, smoke across the explosion radius, sparks; hammer hit: spark star; laser bounce: cyan sparks; damage: drops in body color; death: splatter in body color with gravity; spawn/pickup: glitter; ground jump: dust; air jump: cloud ring; landing: dust by fall speed; grenade: smoke trail. Camera shake on nearby explosions (up to 500 units) and own damage; hit marker (X at the crosshair) on own hit. Both can be disabled in the panel under "Effects", saved in `tuning.toml` under `[effects]` (E-088). Snapshots `design/elora-effekte.png` (explosion, death, hammer, spawn after 0.03/0.12/0.3 s).

- **M5.8 HUD (part 1, E-102):** own game UI instead of egui for the HUD and status display. Renderer: second draw pass (`draw_overlay`) in screen pixels with its own buffers; vector text `Font` (ttf-parser → lyon, glyphs cached, Inter). Bottom center: health and armor bars, weapon selection with ammo under the active weapon, missing weapons faded; top center: mode · phase/timer, score or team standings in team colors, goal, sudden death. Scales with the window height (base 720 px). The crosshair changes color with health: white → yellow (half) → red (empty). Image `design/elora-hud-umgesetzt.png`. 
- **M5.8 HUD (part 2):** vote (below the status display), killfeed (top right, "killer [weapon icon] victim", names in team color), chat (bottom left above the bar, server notices in yellow, open with input line and blinking cursor, long lines truncated with "…") and scoreboard (center, columns per team, own row yellow) as own game UI. Chat input runs through the client's keyboard handling (Enter sends, Esc cancels, Backspace); egui remains only for the debug panel (E-031). Image `design/elora-anzeigen-umgesetzt.png`.

- **M5.9 Emotes (E-103, E-104):** protocol `ClientMsg::Emote(n)` / `ServerMsg::Emote { slot, emote }` (indices `0..8`, invalid ones rejected); the server distributes to everyone, at most one emote per second per player (spam protection). Assets `assets/emotes/` (bubble + 8 symbols; "GG" and "Zzz" as paths). Bubble above the head: pop-up 0.15 s, visible 2 s, fade-out 0.3 s. Wheel: hold E, the mouse direction selects (heart at the top, clockwise), center = nothing; shown directly in the sandbox. Eyes react automatically (`eyes-pain.svg`, `eyes-happy.svg`): pain 0.45 s on damage, happiness 1.2 s after a kill. Color keys now also tint the strokes of shapes without fill (eyes as lines). Image `design/elora-emotes-umgesetzt.png`, pose sheet with expressions `design/elora-posen.png`.

- **M5.7 Audio (E-105):** crate `elora-audio`.
  - `synth`: generator on the sfxr principle – waveforms square (zero-mean), saw, sine, triangle, noise; slide, vibrato, arpeggio, repeat; envelope with punch; low-/high-pass; layers with delay; soft limiting. Parameters in `assets/sounds/sounds.toml` (31 sounds), same parameters = same sound.
  - `cues`: event → sound mapping: weapons (fire/hit/explosion/bounce), no ammo, weapon switch, hook (fire, wall, character, no grip), jump, air jump, landing, pain short/long (from 3 damage), death, spawn, pickups, pickup respawned, hit confirmation, chat, emote, CTF (own flag taken, enemy flag grabbed, dropped, returned, captured).
  - Playback (feature `playback`, kira + cpal/ALSA): spatial up to 1400 units (linearly quieter), stereo by lateral distance (±0.8), master volume and mute in the panel under *Sound*, saved in `tuning.toml` under `[audio]`. Without an audio device the game runs muted.
  - Listening test: `cargo xtask sound-preview` writes `target/sounds/*.wav`.
  - Listening test 1 (E-106): only spawn, death, weapon switch and weapon pickup stay procedural. The other 27 are now **CC0 audio files** (E-107, Kenney, organic/soft style) in `assets/sounds/files/`, sources in `assets/SOURCES.md`. Import with `cargo xtask sound-import` (ffmpeg → mono/44.1 kHz/16 bit, own WAV reader, no additional decoder license); `build.rs` embeds all files, a file replaces the procedural sound of the same name; volumes in the `[gain]` section of `sounds.toml`.
  - Listening test 2 (E-108, without Freesound): explosion mixed from two Kenney explosions; hammer swing, laser and jump reshaped from Kenney recordings (pitch, filter, reversed swell). Three variants each, alternatives in `target/sounds/alternativen/` for comparison.

## Prerequisites from the Project Owner

- For Recraft: an API key (as environment variable `RECRAFT_API_KEY`), only once a motif needs the API.

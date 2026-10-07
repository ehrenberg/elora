# M6 – Maps & Editor: Implementation Plan

Status: **completed** (E-160) · Basis: [`06-roadmap.md`](06-roadmap.md) M6 (after M7, E-111), E-024, E-028, E-030, E-031, E-089, O-37, O-43

## Goal

A dedicated release map format with graphics and an editor integrated into the game (E-028): maps can be built from scratch, tested directly from the editor and played on servers. Plus the first real maps for Release 1.

**Acceptance (roadmap):** With the editor, a map can be built from scratch and played.

## What the Original Does (Teeworlds 0.7, `datafile.cpp`, `mapitems.h`, `editor/`)

| Area | Original |
|---|---|
| **File** | Binary "datafile" (`.map`): entries (items) + zlib-compressed data blocks; images and sounds can be embedded |
| **Groups** | Layer groups with **parallax** (x/y in %), offset, optional clipping |
| **Layers** | **Game layer** (collision: empty, solid, death, unhookable + entities as tile indices), **tile layer** (image made of 16×16 tiles, per tile index + flip/rotation), **quad layer** (free, textured quads with a color per corner), **sound layer** (0.7) |
| **Animation** | **Envelopes**: curves for position, rotation and color; quads and tile layer color are attached to them (e.g. drifting clouds, pulsing light) |
| **Automapper** | Rules automatically pick matching edge/corner tiles from the neighbors |
| **Editor** | In the client (key/menu), brush made of tiles, rectangle, fill, tile selection, managing layers and groups, editing quads, envelope editor, embedding images/sounds, test play via a local server |
| **Transfer** | The server sends the map in chunks to clients that do not have it (checksum + name); clients store it in the downloads folder |
| **Vanilla maps** | dm1, dm2, dm6, dm7, dm8, dm9, ctf1–ctf7 – small to medium arenas, clearly readable collision |

## Status in Elora

- **Text format** (E-024, `.emap.toml`): only collision and entities as an ASCII grid – for test and development maps. Two maps: `sandbox`, `ctf-test`. *(Replaced by `.emap` with M6.2, E-146.)*
- **World look** (E-089): tiles in a single color with an outline, sky as a gradient – simple, without graphics layers.
- **Transfer:** The server sends the map as text in the `Welcome` (up to 4 MB, one packet over the reliable channel).
- **Style:** Everything is vector (E-030), assets are SVG (M5.2) – character, items, emotes.
- **UI:** egui is intended for the editor (E-031); the game UI has its own toolkit (M7.1).

## Work Steps (after the decisions)

| # | Step | Crate | Content | Check |
|---|---|---|---|---|
| M6.0 | Designs | – | Map look as an image to choose from: materials with edges/corners, decoration, background layers (E-130) | Your choice |
| M6.1 | New tile types | `elora-sim` | Platform, ice, jump pad, booster (E-137) in collision and movement; tuning proposal T-31 ff. for approval; test map; golden tests of the old maps unchanged | Tests + your playtest · **completed** (E-140–E-142; `maps/tiles-test.emap.toml`) |
| M6.2 | Release format | `elora-map` | Binary data model (E-129): header with version, sections (game layer with tile types and directions, material layer, decoration, background layers with parallax, envelopes, metadata), compressed, checksum; import of the text maps | Tests (round trip, import, broken files) · **completed** (E-143–E-146; text format removed, `cargo xtask map-dump`) |
| M6.3 | Materials & decoration | `assets/` | SVG sets per material with edges/corners (auto-edge rules), decoration objects, backgrounds – following the design from M6.0 | Visual check · **completed** (E-147, E-148, [`design/elora-kartenteile.png`](design/elora-kartenteile.png)) |
| M6.4 | Map rendering | Client, `elora-render` | Drawing layers (parallax, decoration in front of/behind the play area), auto edges, playing envelopes, cached meshes | Visual check, benchmark · **implemented** (`maps/look-test.emap`; measurement 400×200 tiles: 0.3 ms per frame) · **completed** (E-149) |
| M6.5 | Transfer | Protocol, server, client | Map compressed in chunks, checksum, cache in the client (E-136) | Integration test · **implemented** (protocol version 5; tests: download in chunks, cache, no slot while loading, map change) · **completed** (E-153) |
| M6.6 | Editor foundation | Client (`editor/`, egui) | Editor from the main menu, camera, grid, layer list, new/load/save, undo/redo | Tests + visual check · **implemented** (E-150–E-152; plus a preliminary tile brush) · **completed** (E-153) |
| M6.7 | Tools | Client | Brush, rectangle, fill, eraser, tile types with direction, materials, entities (spawns, pickups, flags, dummies), copy selection | Visual check · **completed** (E-154) |
| M6.8 | Decoration, background, animation | Client | Place/rotate/scale decoration, background layers with parallax, envelope editor (curves for position, rotation, color) (E-133) | Visual check · **implemented** (plus day/night templates, own SVGs) · **completed** (E-160) |
| M6.9 | Test play | Client | from the editor directly into a training round and back | Visual check · **implemented** (F5 or button, Esc back, without saving) · **completed** (E-160) |
| M6.10 | Release maps | `maps/` | 3 DM + 2 CTF according to your specifications (E-134/E-135) | Your playtest · **implemented** (E-155–E-158: `dm-wiese`, `dm-wueste`, `dm-winter`, `ctf-wald`, `ctf-nacht`; construction kit with checks in `tools/design/release_maps/`) · **completed** (E-159) |
| M6.11 | Acceptance | – | Build a map from scratch and play it | Your acceptance · **completed** (E-160) |

## Decisions for M6

| # | Question | Options | Decision |
|---|---|---|---|
| D-M6-01 | Storage format (O-37) | **Text** (TOML, readable, Git-friendly, larger) / **binary** (compact, like the original) / **text, compressed for transfer** | E-129: **binary** |
| D-M6-02 | Graphics of the map parts | **Vector tiles with automatic edges/corners** (tile grid stays, look from SVG sets per material) / **free vector shapes** (polygons independent of the grid) / **both** (grid for collision, free shapes as decoration) | E-130: **both** – tiles with auto edges + free vector decoration |
| D-M6-03 | Layer model | like the original (groups with parallax, tile and quad layers) / simplified (game layer + decoration layers in front/behind + background layers with parallax) | E-131: **simplified** |
| D-M6-04 | Animations | yes (moving decoration, color changes – like envelopes) / no for Release 1 | E-132: **like the original** (envelopes) |
| D-M6-05 | Editor scope for Release 1 | Basic tools (brush, rectangle, fill, entities, layers, undo, test play) / plus decoration & background / plus animations | E-133: **everything** incl. animation editor |
| D-M6-06 | Release maps (O-43) | Number and modes (e.g. 3 DM + 2 CTF), who builds them (me with the editor following your specifications / you / together) | E-134/E-135: **3 DM + 2 CTF**, built by Claude following your specifications |
| D-M6-07 | Map download | The server sends missing maps automatically (like the original) / maps must be installed beforehand | E-136: **automatic** |
| D-M6-08 | New tile types | only the existing ones (solid, death, unhookable) / additional ones (e.g. platform passable from below) | E-137: **platform, ice, jump pad, booster** |

## Technical Specifications (Proposal)

- **Collision stays a grid** of 32-unit tiles (physics and netcode unchanged); the graphics are layered on top.
- **Backward compatible:** text maps (`.emap.toml`) stay readable and are converted into the new model on load; server and client understand both.
- **Checksum** (BLAKE2s, already available via `snow`) identifies maps during download and in the cache.
- **Editor in egui** (E-031) – toolbars and lists are quick to build there; the map itself is drawn by the vector renderer.

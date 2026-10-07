# Map format `.emap`

Status: **accepted** (E-129, E-143 to E-146) · replaces the former text format `.emap.toml` (E-024, removed with M6.2) · Code: `crates/elora-map`

## 1. Goals

1. **One format for everything:** test, development and release maps (E-146). Maps are built with the editor (from M6.6).
2. **Compact:** zlib-compressed (E-143), suitable for download from the server (E-136).
3. **Robust:** Any damaged or malicious file leads to an error, never to a crash. Fixed upper limits protect against oversized data.
4. **Extensible:** Sections with an identifier; unknown sections are skipped, the format version is in the header.
5. **Looks included:** materials, decoration, background layers with parallax, animations and embedded SVGs (E-130 to E-132, E-144).

## 2. File layout

```
Offset  Content
0       "EMAP"                       identifier (4 bytes)
4       format version (u16)         currently 1
6       zlib stream                  sequence of sections (at most 32 MiB unpacked)
```

Each **section**: `Identifier (4 bytes ASCII) | Length (u32) | Content`. Numbers are little endian, texts UTF-8 prefixed with their length (u32), floating-point numbers are `f32` and must be finite. Each section may occur at most once.

| Identifier | Required | Content |
|---|---|---|
| `INFO` | yes | Name, author (empty = none); at most 128 bytes each |
| `GAME` | yes | Width, height (1–1000 each), then 1 byte tile kind per tile (row by row, starting top left) |
| `ENTS` | yes | Count, per entity: kind (u8), column, row |
| `MATL` | no | Material names (at most 255), then 1 byte per tile: 0 = default of the tile kind, otherwise index + 1 |
| `SKY ` | no | Sky gradient top, bottom (RGBA); if missing, the previous sky applies |
| `WTHR` | no | Weather (R2-W1): kind (u8: 0 fair, 1 rain, 2 thunderstorm, 3 fog, 4 wind with leaves, 5 wind with blossoms, 6 sandstorm, 7 snow, 8 blizzard), strength (f32, 0–1), wind (f32, −1 to 1); if missing, the weather is fair. Only written if there is weather; older programs skip it |
| `BGRD` | no | Background layers (at most 16), from back to front: name, parallax (x, y), offset (x, y), horizontal repetition (0 = none), decoration list |
| `DECO` | no | Decoration list behind the playing field, decoration list in front of it |
| `ENVL` | no | Animations (at most 256): name, kind (0 movement, 1 color), bound to server time, points (at most 1024, time strictly increasing): time (ms), 4 values, curve |
| `IMGS` | no | Embedded SVGs (at most 64, at most 512 KiB each): name, data |
| `ADVN` | no | Adventure objects (A1.5, at most 4096): id, position, kind and its values (see below) |

**Decoration object:** graphic (0 = built-in + name, 1 = embedded SVG + index), position, scale, rotation (degrees), mirrored, tint (RGBA), movement animation and color animation (index u16 + offset in ms each; `0xFFFF` = none). At most 20 000 decoration objects in total.

**Animations** loop over the time of the last point. Movement: offset x, y (world units) and rotation (degrees). Color: r, g, b, a (0 to 1, multiplied).

### Encoding of the enumerations

| Code | Tile kind | | Code | Entity |
|---|---|---|---|---|
| 0 | Air | | 0 | Spawn (neutral) |
| 1 | Solid | | 1 / 2 | Spawn red / blue |
| 2 | Unhookable | | 3 / 4 | Flag stand red / blue |
| 3 | Death | | 5 / 6 | Heart / armor |
| 4 | Platform | | 7 / 8 | Laser / grenade launcher |
| 5 | Ice | | 9–12 | Dummy: stands, walks, jumps, walks + jumps |
| 6 / 7 / 8 | Jump pad up / diagonal left / diagonal right | | | |
| 9 / 10 | Conveyor left / right | | | |
| 11 | Climbing wall (E-228) | | | |
| 12 | Crumbling floor (E-230) | | | |

Animation curves: 0 step, 1 linear, 2 slow start, 3 fast start, 4 smooth (as in the original).

### Adventure objects (`ADVN`, E-252 to E-259)

Per object: id (unique, without `:`; key in the save game), position (f32 × 2), kind (u8) and its values. Characters and items are placed with their center at `pos`, areas and doors with their top left corner. A map with an entrance needs no multiplayer spawn.

| Code | Kind | Values |
|---|---|---|
| 0 | Enemy | Kind from `creatures.toml`, stays defeated (boss/special, E-235) |
| 1 | NPC | Character, dialog, facing direction, half walking distance (0 = stands, E-257) |
| 2 | Chest | Content (item, count; at most 64), lock condition (empty = open, E-255) |
| 3 | Switch | Flag, once only, trigger: action key / hammer / hook (E-256) |
| 4 | Door | Size in tiles (on the grid), condition to open (E-254) |
| 5 | Collectible | Item |
| 6 | Save point | – |
| 7 | Healing plant | Health (E-258) |
| 8 | Entrance | – (target of transitions) |
| 9 | Transition | Size, target map, target entrance, on walking in (otherwise action key, E-252) |
| 10 | Zone | Size (for “reach location” quests) |
| 11 | Camera | Size, kind: fix / limit (E-259) |

The map checks the structure (ids, position, sizes, grid); references to enemy kinds, characters, dialogs, items, conditions and target maps are checked by `elora-adventure` (`check::map_objects`, `check::map_links`).

## 3. Meaning of the tile kinds

| Tile | Meaning |
|---|---|
| Air | Empty |
| Solid | Wall, hook grabs |
| Unhookable | Wall, hook does **not** grab |
| Death | Kills on contact |
| Platform | Carries from above, passable from below/the sides; hook, grenade and laser fly through; with “down” you drop through (T-36, E-141) |
| Ice | Wall, slippery (T-31, T-32) |
| Jump pad | Throws a character standing on it upwards or diagonally (T-33, T-34); hook grabs |
| Conveyor | Carries a character standing on it like a treadmill (T-35); hook grabs |
| Climbing wall | Wall, hook does **not** grab; with ice grip Elora can cling to it and jump off (E-228) |
| Crumbling floor | Wall, hook grabs; breaks on a stomp and stays broken (E-230) |
| Hook blossom | Hook point in the air; everything else flies and walks through (R2-M2.1) |
| Quicksand | Not solid; characters sink in and walk more slowly, jumping frees them (E-318) |
| Thin ice | Carries, hook does **not** grab; breaks after standing on it briefly (A-36) or immediately on a stomp, and grows back (A-37) (R2-M2.4) |
| Ice water | Not solid; whoever falls in takes damage (A-38) and is returned to the edge (R2-M2.4) |

## 4. Rules

| Rule | Definition |
|---|---|
| Coordinates | Origin top left, x to the right, y downwards. 1 tile = 32 units. Entities sit in the tile center |
| Outside the map | counts as solid (nobody falls out of the world) |
| Validation | at least 1 spawn; flags only as a pair (exactly 1× red and 1× blue); entities within the grid; references to images and animations (of the matching kind) must exist |
| Supported modes | derived from the entities: neutral spawns → DM/LMS/Instagib, red + blue spawns → TDM/LTS, plus a flag pair → CTF |
| Checksum | BLAKE2s-256 over the file bytes; identifies the map during download and in the cache (M6.5) |
| Embedded SVGs | The map only checks count and size. The client parses them when drawing **without external references** (no files, no network addresses, M6.4) |

## 5. Tools

- **View:** `cargo xtask map-dump maps/<karte>.emap` prints header, checksum, modes, layers and the grid as characters.
- **Tests:** `Map::from_rows` builds maps from character grids (tiles as in the recordings: `. # % ^ = ~ ! \ / < >`; entities `S R B r b h a L G D W J X`). This is a helper in code, not a file format.
- **Hot reload:** The sandbox watches the map file and reloads it on save (e.g. from the editor).

## 6. Built-in graphics (style A, M6.3)

Location `assets/map/`, overview in [`../archive/release-1/design/elora-kartenteile.png`](../archive/release-1/design/elora-kartenteile.png). Generated once with `tools/../archive/release-1/design/elora_map_assets.py`, afterwards normal SVGs that can be edited by hand.

| Kind | Names | Note |
|---|---|---|
| Materials (`MATL`) | `earth`, `sand`, `snow` (solid) · `stone` (unhookable) · `ice` (ice) | Colors, rounding and detail share in `materials.toml`; one SVG per material with caps (`cap`, `cap-left`, `cap-right`, `cap-single`) and details (`detail-1` …). The first material of a tile kind is its default |
| Special tiles | `tiles/death`, `platform`, `jump`, `conveyor` | fixed assignment; spikes point away from the ground, jump pad left and conveyor left are mirrored |
| Decoration (`Art::Builtin`) | `bush-1`, `bush-2`, `flower-pink`, `flower-yellow`, `flower-blue`, `grass-1`, `grass-2`, `rock-1`, `rock-2`, `mushroom-red`, `mushroom-brown`, `tree-round`, `tree-pine`, `fence`, `sign-arrow`, `sign-board` | Origin at the bottom center |
| Background (`Art::Builtin`) | `cloud-1` … `cloud-3`, `moon` (origin center) · `hills-far`, `hills-near`, `mountains`, `forest` (origin bottom left) · `stars` (top left) | Strips 1024 wide and seamlessly repeatable; darkened at night via the tint |

**Auto edges:** Solid tiles form one surface together with jump pads and conveyors. Outer corners with two free neighbors are rounded, the outline lies only on free edges, caps on every free top edge (with an end piece on free sides), details scattered deterministically per tile.

## 7. In-game rendering (M6.4)

- **Order:** sky → background layers (back to front) → decoration behind → playing field → characters, items, projectiles → decoration in front → effects.
- **Parallax:** An object of a background layer is placed at `Position + Versatz + Kamera-Mitte × (1 − Parallax)` (position + offset + camera center × (1 − parallax)). Parallax 1 moves with the playing field, 0 stays fixed on screen. With repetition, the layer is tiled horizontally across the whole view.
- **Animations:** Movement acts as offset (x, y) and rotation, color is multiplied. Animations bound to server time are seen by all players in the same phase (e.g. drifting clouds), the others run on the client's clock.
- **Cache:** The playing field is tessellated once in chunks of 16 × 16 tiles and only rebuilt when the map changes; only visible chunks are built and drawn. The conveyor arrows animate every frame.
- **Embedded SVGs** are loaded by the client without resolving any external references and with at most 200 000 vertices per image; invalid images stay invisible.
- **Showcase map:** `maps/look-test.emap` (generated from `map_view.rs`, test `write_look_test_map`).

## 8. Transfer (M6.5, E-136)

1. Client → `Join`. The server answers with `MapInfo` (name, checksum, size) – the player is **not yet** in the world.
2. The client looks for the map with exactly this checksum: first among the downloads (`~/.local/share/elora/downloads/<name>-<prüfsumme>.emap`, on Windows/macOS in the settings folder), then in `maps/<name>.emap`.
3. If it is missing, the client requests it with `MapRequest` in chunks of 16 KiB (4 chunks in flight at once) and shows the progress. After the last chunk it checks size and checksum and stores the file.
4. Client → `MapReady`. Only now does it get a slot and `Welcome` (with the checksum for verification).
5. **Map change:** The server sends `MapInfo` to everyone again; everyone loads and rejoins.

Limits: maps up to 4 MiB (the server does not even load larger ones), each chunk at most twice per client; file names derived from server names are reduced to `A–Z a–z 0–9 - _`. Protocol version 5.

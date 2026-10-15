# R2-WM – World map – implementation plan (interim task)

Status: **planned** · Decisions E-370 to E-373 · Interrupts R2-M2.5 after M2.5.4 (M2.5.5 is
parked as work in progress) · Basis: E-264 (world map in the adventure menu)

## Goal

The world map shows the Tauland as a painted overview and **roughly where Elora is**: on the
path piece of her map section, at the place matching her position inside the map, plus a
detail window of the current section. It opens with **M** as its own full-screen map.

**Acceptance:** open the map with M in every area; Elora's marker moves along her section while
she walks; undiscovered sections lie under clouds; markers are right.

## Starting point

- The tab “Map” of the adventure menu (`adventure_menu.rs`, `world_map`) draws six area circles
  with dashed paths from Tauwinkel; “you are here” only marks the area.
- `worldmap.toml` has per area: position (0..1), colour, map prefix (`maps = "meadow-"`).
- The session remembers visited maps as flags `visited:<map>`.
- Quest steps have goals (`talk`, `reach`, `defeat`, `collect`, `bring`, `flag`, `manual`) but no
  place.

## Work steps

| # | Step | Content | Check |
|---|---|---|---|
| WM.0 | Drafts | Painted overview of the Tauland (Tauwinkel in the middle; meadows, forest, desert, frost peaks, gorge in their colours, paths, river), path pieces per section, clouds over undiscovered parts, icons (spring stone, quest star, spring grey/colourful, guardian arena, shop/character), look of the detail window, Elora marker; python script in `tools/design/`, no AI (E-295) | Your selection |
| WM.1 | Data and location | `worldmap.toml`: per area a list of **sections** `[[area.section]] map = "meadow-1", path = [[x, y], …]` (polyline 0..1 on the overview; arena as a point); optional `axis = "y"` for maps that run upwards. Check: every adventure map has a section. Session: `map_location()` = section + fraction along it (Elora's x in the map, or y with `axis`); visited sections; **visited spring stones** remembered (new flag `rested:<map>:<id>`) | Tests |
| WM.2 | Quest goal | Place of the current main quest step derived from the goal: `talk`/`bring` → map with that character, `defeat` → map with that enemy kind, `collect` → map with that collectible or chest, `reach` → map with that zone; optional `map = "…"` on a step overrides it; otherwise no star | Tests |
| WM.3 | Overview | Painted base, path pieces (visited: in the area colour, current one highlighted), clouds over unvisited sections and unknown areas (names only after the first visit), Elora marker moving along her section, markers per E-372 with a small legend | Visual check |
| WM.4 | Detail window | Current section drawn small from its tiles (solid ground, platforms, water/thorns tinted), Elora as a dot, exits as arrows with the target area, spring stones, characters/shops, quest star if the goal is on this map; cached per map | Visual check |
| WM.5 | Key M | New bindable action `map` (default M) in the settings; M opens the full-screen map (game pauses), M or Esc closes; the tab “Map” uses the same view; German and English texts | Tests + your playtest |
| WM.6 | Acceptance | Open the map in every area, walk, check markers and clouds | Your acceptance |

## Considerations

- **Placement of the sections:** written by hand in `worldmap.toml` (looks best); a fallback
  places missing sections automatically in a row from the area point, so new maps (gorge,
  M2.5.7) appear before they get their own path.
- **Fraction along the section:** by default Elora's x across the map width; maps that are
  climbed upwards (frost peaks) use `axis = "y"`. The path direction in the data decides which
  end is the map's left (or bottom).
- **Tauwinkel** is a town icon in the middle, not a path; its detail window works like any map.
- **Arenas** are points with a guardian icon (defeated: calm icon).
- **Characters and shops:** from the NPC objects of the maps; the overview shows them only for
  Tauwinkel (shops) and as a small icon per section, the detail window shows them all.
- **Clouds:** an area is “known” once one of its sections was visited; unvisited sections of a
  known area are drawn under thinner clouds, so the way ahead can be guessed.
- **No new save format:** spring stones visited are new flags; old saves show only the spring
  stones used again.
- **Controller/scale:** the full-screen map scales with the UI scale like the other menus.

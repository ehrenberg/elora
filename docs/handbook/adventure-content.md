# Writing adventure content

Status: R2-M1 (A1.9) · Decisions: E-217, E-218, E-246 to E-251

All adventure content consists of TOML files under `assets/adventure/`. Texts are always written in **both languages side by side** (`{ de = "…", en = "…" }`); if one is missing, the game reports file and location on loading (likewise unknown characters, items, quests, nodes and typos in conditions). `cargo xtask check` checks the shipped content.

| File | Content |
|---|---|
| `characters.toml` | Characters with name and picture (`elora` needs no entry) |
| `dialogs/<id>.toml` | One dialog per file (register new files in `crates/elora-adventure/src/data.rs` under `dialogs!`) |
| `quests.toml` | Quests with steps |
| `items.toml`, `skills.toml`, `upgrades.toml`, `shops.toml`, `progression.toml`, `creatures.toml` | Items, skill tree, weapon upgrades, shops, progression, enemies ([`progression.md`](../release-2/progression.md)) |
| `worldmap.toml` | Areas of the world map (maps, music, spring, chapter, weather) and the temperature bars |

## Dialogs

```toml
speaker = "oma"                       # default speaker

[[start]]                             # entry points: the first whose condition holds
if = "quest well active"
node = "memory"

[[start]]
node = "greeting"

[[node]]
id = "greeting"
text = { de = "Ach, Elora …", en = "Oh, Elora …" }
do = ["affection oma +1"]             # effects when reached (optional)
next = "far"                       # without choices: next node; without next = end

[[node.choice]]                       # choices (optional)
tone = "friendly"                   # freundlich | neugierig | frech (optional)
if = "level >= 2"                     # only visible if the condition holds
text = { de = "…", en = "…" }
next = "promise"                       # without next = end
do = ["quest well start"]

[[bark]]                              # short calls as a speech bubble (E-222)
if = "quest well active"
text = { de = "Pass auf dich auf!", en = "Take care!" }
```

A node can have `speaker = "elora"` or another character. Every node must be reachable.

**Keys in text (A1.9):** `{taste:<aktion>}` shows the bound key, e.g. `{taste:jump}`, `{taste:interact}`, `{taste:hook}`, `{taste:fire}`, `{taste:down}`, `{taste:quick_heal}`, `{taste:scoreboard}` (names as in the settings files for controls). Works in nodes, choices and barks.

**Appearance:** `show_if = "<Bedingung>"` in `characters.toml` shows a character only while the condition holds (e.g. the bumblebee only after the fight: `flag defeated.bumblebear`). Defeated guardians set the flag `defeated.<art>`.

**Companions (E-308):** `follower = "<gegnerart>"`, `follow_if = "<Bedingung>"` and `home_zone = "<zone>"` in `characters.toml`: while the condition holds, the enemy kind (behavior `follower`) follows Elora, even across map changes. When it reaches the zone, the flag `<id>.home` is set.

**Voice (E-286):** In `characters.toml`, `voice` sets the pitch of the babble sounds while the text appears (1 = medium, smaller = deeper, 0 = silent). Elora speaks at 1.25.

**Music (E-285, E-290):** In `worldmap.toml`, `music = "<name>"` selects the file `assets/music/<name>.ogg` per region (Ogg Vorbis, 44.1 kHz). Changing region crossfades; `menu.ogg` plays in the main menu. Source and license go into `assets/SOURCES.md`.

**Signs (E-273):** Signposts are characters with `fixed = true` in `characters.toml` (they do not turn towards Elora); every sign has its own dialog `dialogs/sign-<ort>.toml`.

## Conditions (`if`)

The German keywords of 0.9.x (`merker`, `quest … schritt`, `gib`, `faehigkeit` …) are still accepted for one release (D-RF-02, `assets/renames.toml`).

| Condition | Meaning |
|---|---|
| `level >= 3`, `gleam < 50` | Level, gleam drops; comparisons `= != < <= > >=` |
| `quest well new` / `active` / `done` / `failed` | State of a quest |
| `quest well step bridge` | Current step of a quest |
| `flag oma.cheeky`, `flag gate >= 2` | World state (without comparison: set) |
| `affection lotte >= 5` | A character's affection (−10 to 10) |
| `has amber 3`, `has healing_potion` | Item (without a number: at least one) |
| `ability glide` | hook-jerk, pull_hook, stomp, ice_grip, glide |
| `weapon hammer` | Elora owns the weapon: hammer, grenade, laser (E-354) |

Combine with ` and `, negate with a leading `not `: `not flag oma.cheeky and level >= 2`.

## Effects (`do`)

| Effect | Meaning |
|---|---|
| `quest well start` / `advance` / `finish` / `fail` | Start a quest, complete the current step, complete it entirely, fail it |
| `affection oma +1` | Change affection |
| `flag oma.cheeky = 1`, `flag gate +1` | Set or change world state |
| `give healing_potion 2`, `take amber 3` | Give or take an item (also `gleam_drops`) |
| `xp 50`, `points 1` | Experience, dewdrop points |
| `ability hook-jerk`, `weapon grenade` | Unlock a region ability or a weapon |
| `shop lotte`, `forge`, `skills` | Open a shop, the smithy, the skill tree |

## Quests

```toml
[[quest]]
id = "well"
kind = "main"                          # main | side
giver = "oma"
name = { de = "…", en = "…" }
desc = { de = "…", en = "…" }
reward = { xp = 50, gleam_drops = 30, items = [{ item = "healing_potion", count = 1 }], points = 0 }
fail_if = "flag well.zu_spaet"    # optional: fails as soon as this holds (E-250)
next = "blossom_spring"                 # optional: starts after completion

[[quest.step]]
id = "tueftel"
text = { de = "Bei Tüftel vorbeischauen", en = "Drop by Tüftel's workshop" }
goal = { type = "talk", who = "tueftel" }
```

| Goal (`goal.type`) | Fields | done when … |
|---|---|---|
| `talk` | `who` | a dialog with the character starts |
| `reach` | `map`, optional `zone` | Elora reaches the map or zone (zones come with A1.5) |
| `defeat` | `kind`, `count`, optional `map` | that many enemies of the kind are defeated |
| `collect` | `item`, `count` | Elora owns that many |
| `bring` | `item`, `count`, `to` | Elora brings them to the character (they are handed over) |
| `flag` | `flag`, optional `value` (1) | the flag has the value (switch, chest, door …) |
| `manual` | – | a dialog runs `quest <id> advance` |

The quest book shows completed steps and the current one, further ones as “?” (E-251).

## Temperature bars (`worldmap.toml`, E-320, E-342)

Areas with `hot = true` get the **heat bar**, areas with `cold = true` the **cold bar** (not both). Both bars work alike; their values are in `[temperature.heat]` and `[temperature.cold]`:

```toml
[temperature.cold]
fill_ms = 60_000              # in the open until full
faster_weather = ["blizzard"] # in these weathers …
faster = 2.0                  # … it fills this many times as fast
shelter_ms = 10_000           # in shelter until empty:
shelter_zones = []            #   roof (solid tile or platform up to 10 tiles above Elora),
shelter_weather = []          #   zones with these prefixes, these weathers,
shelter_arenas = true         #   the guardian arenas (`…-arena`)
relief_ms = 3_000             # in zones with these prefixes until empty
relief_zones = ["fire"]
recover = 0.5                 # full = Elora is slower until the bar is below this
```

The heat bar fills in 20 s, cools in 8 s in the shade (roof, zones `shade…`, sandstorm) and in 2.5 s at the oasis (zones `oasis…`). The cold bar fills in 60 s (twice as fast in a blizzard), warms in 10 s under a roof and in the guardian arenas and in 3 s at the fire (zones `fire…`). Equipment with `heat_pct` / `cold_pct` changes how fast a bar fills (negative = more slowly). Weathers are the keys of the weather lists (`sandstorm`, `blizzard` …); the check rejects unknown weathers, empty prefixes, times of 0 and `recover` outside (0, 1].

## Terrain with state (R2-M2.4)

- **Zones with a fixed name prefix:** `shade…` and `oasis…` cool the heat bar (E-320), `fire…` warms up the cold bar within a few seconds (E-342; roofs warm more slowly, in the guardian arenas it is paused). The prefixes and times come from `[temperature]` in `worldmap.toml` (see below). `avalanche…` is an **avalanche slope**: a stomp or a grenade in it, or a step into the zone `<hang>-step` (e.g. `avalanche-ridge` and `avalanche-ridge-step`), makes A-39 snowballs roll downhill from the higher end at an interval of A-40; afterwards the slope rests for A-41.
- **Icicles** are enemies of the kind `icicle` (behavior `icicle`): place them on the ceiling; they tremble as soon as Elora is below, fall and shatter. No experience, no loot.

## Adventure maps in the editor (A1.8)

Tool **9 “Adventure”** (E-268): choose the kind in the sidebar; a click places the object on the ground below the mouse (collectibles float in the tile center); door, transition, zone and camera are dragged out. Clicking an object selects it, dragging moves it (by whole tiles), right-click or Del deletes it; undo as usual. On the right are the values of the selected object, for NPCs the dialog preview and **“Test dialog”** (window with changes to flags, quests, affection and items, E-270). “Reload content” reads `assets/adventure` without a restart; the **check** shows missing enemy kinds, characters, dialogs, items, wrong conditions and transitions without a target.

- **Map name** = file name and target of transitions (e.g. `meadow-1`). It is saved to `<user folder>/maps/adventure/<name>.emap` (user directory); the game prefers this file over the shipped one of the same name (E-271).
- **F5** test-plays in the adventure with the **test setup** (level, abilities, weapons, flags such as `tor.dorf=1, oma.frech`; start at an entrance/spring stone or at the mouse). Nothing is saved; transitions load the other maps, including the one currently being edited even if unsaved. Esc returns to the editor (E-269).

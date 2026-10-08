# R2-M1 – Adventure Foundation: Implementation Plan

Status: **Decisions made, plan up for approval** · Basis: E-203 to E-224, [`world-book.md`](world-book.md)

## Goal

The technology for the single-player adventure – enemies, NPCs, dialogues with consequences, quests, levels and skills, equipment, save games, multiple maps with transitions – and the editor tools for it. Afterwards, areas can be built like maps.

**Acceptance:** The **prologue** is playable: Tauwinkel (base version) and the path into the Blütenwiesen up to the first section, with conversations, a quest, enemies, loot, a level-up and saving/loading (about 20–30 minutes).

## Starting point

- `elora-sim` simulates players (human, remote character, training dummy), projectiles, lasers, pickups and flags deterministically at 50 ticks/s. Dummies are already computer-controlled (`Controller::Dummy`) – that is where enemy behaviour plugs in.
- Maps (`.emap`) are divided into sections; unknown sections are skipped. A new section for adventure objects therefore breaks nothing.
- Game UI ("bright & soft"), language files, editor (egui) and sandbox are in place.

## Work steps

| # | Step | Content | Check |
|---|---|---|---|
| A1.0 ✅ | Drafts (E-225) | Adventure UI (dialogue box, HUD with level and quest, inventory, skill tree, merchant) and the first enemies and NPCs (spike beetle, pollen puffer, grass hopper; Oma Pfütze, Klonk, Lotte, Tüftel, Pip) as images to choose from | Your choice |
| A1.1 ✅ | Abilities in the simulation (E-231) | Hook jerk, pull hook, stomp, ice grip, glide as switchable abilities of the character with their own tuning values (A-01 ff.); without abilities nothing changes (golden tests) | Tests + playtest in the sandbox |
| A1.2 ✅ | Creatures (E-240) | new simulation elements for enemies: species with behaviour (patrol, jump, fly, shoot), health, hits from weapons and hook, contact damage, loot; deterministic | Tests + visual check |
| A1.3 ✅ | Adventure core | new crate `elora-adventure` (pure logic): save game (level, experience, skill tree, inventory, gleam drops, quests, world state, consequences of dialogues), rewards, merchants, upgrades | Tests |
| A1.4 ✅ | Dialogues & quests | data format for conversations with choices, conditions and consequences, and for quests with steps; texts translatable | Tests |
| A1.5 ✅ | Map extension | section for adventure objects in `.emap`: enemies, NPCs (with conversation), chests, switches, doors, collectibles, save points, transitions to other maps, trigger zones | Tests (round trip, broken data) |
| A1.6 ✅ | Adventure mode | main menu "Adventure" with save games; switching between maps (village ↔ area sections); death and restart; saving | Tests + visual check |
| A1.7 ✅ | UI | dialogue box with portrait and choices, speech bubbles for call-outs, look-ahead camera with camera zones, HUD (health, level/experience, gleam drops, current quest), adventure menu (inventory, skills, quests, map), merchant and smith | Visual check |
| A1.8 ✅ | Editor extension | "Adventure" tool: place enemies, NPCs, objects, zones (trigger, camera) and transitions; show each NPC's conversations and test and tune them directly; test play in adventure mode | Visual check |
| A1.9 ✅ | Prologue | Tauwinkel (base version), tutorial path, first section of the Blütenwiesen with conversations, quest, enemies, loot | Your playtest |
| A1.10 | Acceptance | play the prologue from start to finish, save, continue | Your acceptance |

## Technical decisions (proposal)

- **Abilities and creatures belong in the simulation**, not in the client: the later PvP mode "Spring Battle" (E-204) runs over the network and needs them there.
- **Adventure logic is its own crate** (`elora-adventure`): save game, quests, dialogues, rewards – testable without a window, like `elora-game` for the multiplayer rules.
- **Content as data**, not as code: enemy species, items, conversations and quests live in files under `assets/adventure/`; maps only refer to their names.
- **The core feel stays:** running, jumping, double jump and hook behave as in multiplayer (E-212, world book §6).

## Decisions on R2-M1

| # | Question | Options | Decision |
|---|---|---|---|
| D-A1-01 | Language of the adventure texts | German and English from the start / German only first, English later | **German + English** (E-217) |
| D-A1-02 | Where conversations and quests are maintained | in the editor (graphical) / as text files (readable, Git-friendly) / text files with preview and test in the editor | **Text files + editor test** (E-218) |
| D-A1-03 | Save games | number of slots; only at save points or anytime; automatically on map change | **3 slots, automatic + save points** (E-219) |
| D-A1-04 | Death in the adventure | back to the last save point without loss / with a small loss (e.g. gleam drops) / choice per difficulty | **Save point, small loss** (E-220) |
| D-A1-05 | Difficulty levels | one / several (e.g. easy, normal, hard) | **One level** (E-221) |
| D-A1-06 | Presentation of conversations | text box at the bottom with the character's portrait / speech bubbles above the characters / both | **Both** (E-222) |
| D-A1-07 | Abilities in normal multiplayer | only adventure and role-playing mode / also as a server option for other modes | **Only adventure + Spring Battle** (E-223) |
| D-A1-08 | Camera in the adventure | as in multiplayer (fixed on Elora) / slightly looking ahead and adapted to rooms | **Look-ahead + camera zones** (E-224) |

## A1.1 Abilities – mechanics (E-226 to E-230)

The abilities are switches on the character (`CharacterCore`); without them everything behaves as before (golden tests stay unchanged). Values as their own tuning numbers **A-01 ff.**, adjustable live in the sandbox; on/off per ability in the debug panel for trying them out.

| Ability | Trigger (proposal) | Effect | Tuning (start value) |
|---|---|---|---|
| **Hook jerk** | **"Ability"** key (new, default: Shift) while the hook is attached to a wall | strong jerk towards the hook point, then normal pull | A-01 jerk strength 16 units/tick · A-02 cooldown 0.8 s |
| **Pull hook** | hook hits an item, switch or small enemy | pulls it to Elora (like the player hook, reversed) | A-03 pull force · comes with the creatures in **A1.2** |
| **Stomp** | **Down** in the air (press, not hold) | thrusts straight down; on impact a shockwave: stuns/damages enemies, breaks **crumbling floor** (new tile, stays broken) | A-04 stomp speed 22 · A-05 shockwave radius 64 (2 tiles) |
| **Ice grip** | in the air, **run against a climbing tile** (solid, non-hookable wall) | Elora clings briefly and slides slowly; **jumping** pushes off the wall, double jump is kept | A-06 cling time 1.0 s · A-07 slide speed 1.0 · A-08 wall jump (9 sideways, 12 up) |
| **Glide** | **hold jump** while Elora is falling | fall speed capped, a bit more air control | A-09 max. fall 2.0 · A-10 air control 7.0 (normal 5.0) |

Technology: additional input "Ability" in `PlayerInput` (goes over the network only with Spring Battle, then protocol 7), new events (jerk, stomp, impact, wall grip, wall jump) for sound and effects, tiles `Kletterwand` and `Bröckelboden` in collision, map and editor.

**Status A1.1:** implemented except for pull hook (with A1.2). Try it: `cargo run --bin elora -- maps/faehigkeiten-test.emap`, enable in the debug panel (F1) under "Abilities (Adventure)"; values under "Abilities (A-01 to A-10)". "Ability" key = left Shift (rebindable in the settings). Test map: on the left a hall with a ceiling (hook jerk), crumbling bridge over a chamber (stomp), chimney and single wall made of climbing walls (ice grip), from the ledge over the spike pit to the platform (glide).

**Status A1.2:** Enemies as simulation elements (`elora-sim/src/creature*.rs`), species as data in `assets/adventure/creatures.toml` (spike beetle: walks, turns at edges · pollen puffer: stands, shoots pollen balls · grass hopper: jumps at Elora; plus the "flyer" pattern for later areas). Hits from hammer, grenade, laser and stomp (with stun); the hook grabs enemies and pulls Elora towards them, with pull hook small enemies come to Elora; contact damages with knockback and a protection time; loot pops out and flies to Elora from close by; health bars after hits. Try it in the sandbox: F1 → "Enemies (Adventure)" (enable adventure rules, choose a species, "Place"). Placing enemies in maps comes with A1.5/A1.8; the adventure counts experience and gleam drops from A1.3 on.

**Status A1.3:** new crate `elora-adventure` with content as data (`assets/adventure/`: `items.toml`, `skills.toml`, `upgrades.toml`, `shops.toml`, `progression.toml`, `creatures.toml`; texts in German and English), save game (`SaveGame`: levels and experience, skill tree, inventory, equipment, weapons with upgrades, abilities, world state, location, play time), rules for learning, buying, selling, upgrading, consuming, death and resting, and save games in three slots (compressed with checksum, E-245). The simulation's tuning is derived from this; special upgrades (range, stun, shockwave, shrapnel, piercing) are implemented in the simulation. Healing blossoms, second chance, dew potion and armour from equipment are implemented by the adventure mode (A1.6).

**Status A1.4:** Conversations (`assets/adventure/dialogs/*.toml`) with entry points by condition, nodes, answers with tone, conditions and consequences, and call-outs; quests (`quests.toml`) with the objectives talk, reach location, defeat, collect, deliver, flag and "manual", rewards and failure; affection per character (−10 to 10) with a discount at Lotte (from 5: 10 %, from 10: 20 %); characters in `characters.toml`. Everything is checked on load (references, conditions, consequences, both languages, unreachable nodes). Sample content: Oma Pfütze, Tüftel, quests "The Pale Well" and "The Glitter Stone in the Grass" (drafts, polish with A1.9). Guide: [`handbook/adventure-content.md`](../handbook/adventure-content.md).

**Status A1.5:** Section `ADVN` in the map format with twelve object types (enemy, NPC, chest, switch, door, collectible, save point, healing plant, entrance, transition, zone, camera), checked on load (ids, position, sizes, door on the grid) and against the content (`check::map_objects`, `check::map_links` for transitions between maps). Adventure maps only need an entrance instead of a multiplayer spawn. New binding: **E = action** everywhere, **emote wheel on Ctrl** (E-253; old settings are migrated). Description: [`handbook/map-format.md`](../handbook/map-format.md).

**Status A1.6:** Main menu tab **"Adventure"** with three slots (Continue, New adventure, Delete with confirmation; damaged slots are shown). The session (`elora-adventure/src/session.rs`) builds the world from map and save game and evaluates every tick: doors (open and stay open), chests, lever/hammer/hook switches, healing plants, collectibles, zones, transitions (by walking in or with E), spring stone (rest + save), call-outs, death with a choice (continue at the spring stone / main menu), second chance, dew potion, ammunition; saving on map change and at the spring stone, not on exit (hint in the pause menu). Conversations run in a plain text box (answers with 1–6 or mouse, continue with E/Space); NPCs and objects have simple placeholder graphics, hints appear in the message area – the proper UI comes with A1.7. Test maps `maps/abenteuer/tauwinkel.emap` and `wiese-1.emap`; start directly with `cargo run --bin elora -- --abenteuer 1`.

**Status A1.7:** Graphics for NPCs (Oma Pfütze, Klonk, Lotte, Tüftel, Pip) and objects (chest, spring stone – glows where Elora last rested –, switch, healing plant) from the drafts; HUD with level and experience ring, gleam drops and current quest; conversation box with portrait, name plate and answers with tone; speech bubbles with a tail; action key hint showing the bound key; camera with look-ahead in the running direction and camera zones (lock/limit, smooth transition); adventure menu with **Tab** (inventory with equipment, skill tree for learning, quests with completed and current step, world map), Lotte's shop (buy/sell, discount) and Klonk's smithy (upgrades with cost and stock), opened from conversations; **Q** drinks a healing potion. Items have simple icons per type. Previews: `cargo test -p elora-client --bin elora adventure_ -- --ignored`.

**Status A1.8:** Tool 9 "Adventure" in the editor with all twelve object types (place on the ground, drag out areas, select, move, delete, undo), properties per type with selection lists from the content, conversation preview and test window, checks (objects and transitions), reload content, test state and test play with F5 in the adventure; adventure maps under `maps/abenteuer/` in the user folder, which take precedence over the bundled ones. Guide: [`handbook/adventure-content.md`](../handbook/adventure-content.md).

**Status A1.9:** Prologue playable ([`prolog.md`](prolog.md)), playtest with polish accepted. Maps `maps/abenteuer/tauwinkel.emap` (450 × 50) and `wiese-1.emap` (300 × 60), built by `apps/elora-client/src/editor/prologue.rs` (rewrite: `cargo test -p elora-client --bin elora write_prologue_maps -- --ignored`, overview: `prologue_sheets`). Open map edges with natural ends (E-279), sprawling village with an upper village (E-280), Tüftel's yard (E-281), Blütenwiesen with heights, hook gorges and two paths (E-282), thorns cost 2 health and reset (E-283, A-22). Buildings, props and details (birds, cat, butterflies, smoke, waving flags) from `tools/design/tauwinkel_buildings.py` and `tauwinkel_props.py`; flower beds, flower boxes, flower pots and flags faded, with every freed spring a part turns colourful (E-277). Conversations of Pip, Oma, Tüftel, Klonk and Lotte, eight signposts with the bound keys (`{taste:…}`). Playthrough as a test: `crates/elora-adventure/tests/prolog.rs`.

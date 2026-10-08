# R2-M2.1 – Chapter 1: Blütenwiesen – Implementation Plan

Status: **Completed, provisionally accepted (E-305)** · Decisions E-297 to E-304 · Basis: [`world-book.md`](world-book.md) §4.1 and §5, [`prolog.md`](prolog.md), E-272 to E-295

## Goal

The first chapter is playable from start to finish: after the prologue on through the Blütenwiesen, beekeeper Wabe and her bees, the fight against the guardian **Bumblebear**, the **spring spark** for Tüftel and the new ability **hook jerk**, plus a village that gets back some colour.

**Acceptance:** Play through chapter 1 once (about 60–90 minutes with side quests), saving and continuing in between; afterwards "The Blossom Spring" is marked as completed in the quest book and the next chapter is announced.

## Starting point (from R2-M1)

- Enemies with the behaviours walker, hopper, shooter, flyer; species with `boss = true` stay defeated (E-235), but have **no behaviour of their own and no health bar in the HUD** yet.
- Abilities finished in the simulation (A1.1); unlocked via the consequence `faehigkeit hook-ruck`.
- Colour return in the village prepared: flag `quellen_befreit` turns `-blass` decoration partly colourful (E-277).
- Maps are created by a generator in `apps/elora-client/src/editor/prologue.rs` (terrain, objects, decoration), graphics via the Python scripts (E-295), music per area via `worldmap.toml` (E-290).
- `wiese-1` ends at the meadow edge with a spring stone; that is where "The Blossom Spring" begins (step "Deeper into the Blütenwiesen").

## Work steps

| # | Step | Content | Check |
|---|---|---|---|
| M2.1.0 ✅ | Drafts ([`design/chapter1-drafts.png`](design/chapter1-drafts.png)) | Beekeeper Wabe (character and conversation portrait), Bumblebear (rest, flight, dive, stunned, defeated), confused bee (phase 3 enemy), bee (collectible), honeycomb hat, garlands and festival lanterns, hook flower (hook point in the arena), spring spark (item), beehives and apiary as decoration, Blossom Spring (freed/withered) | Your choice |
| M2.1.1 ✅ | Guardian technology | new behaviour **boss** in the simulation with phases (patterns of steps: circle, take aim, dive, stunned on the ground), vulnerable only in certain phases; health bar with name at the top of the HUD; arena: entrance and exit close during the fight (door with condition, already exists), camera zone "Lock"; after the victory an event for quests and flags | Tests + sandbox (F1 → Enemies) |
| M2.1.2 ✅ | Hook points | **Hook flower**: single hook point in mid-air (map object; for the simulation a hookable tile without collision for characters), so Elora can stay up high in the arena | Tests + visual check |
| M2.1.3 ✅ | Content | character and conversation **Wabe**; side quest **"Wabe's Bees"** (five bees in the sections, some behind hook spots); main quest **"The Blossom Spring"** in steps up to the spring spark; new conversations for Oma, Tüftel, Pip, Klonk and Lotte after chapter 1; call-outs in the village change | Tests (content check, playthrough) |
| M2.1.4 ✅ | Maps | `wiese-2` (forests of giant flowers, apiary, stream, first hard hook course), `wiese-3` (caves under the roots, thorns, hidden bee, path up to the arena), `wiese-arena` (Blossom Spring, hook flowers, arena) – size and decoration like `wiese-1`; return paths to the earlier sections | Visual check + playthrough test |
| M2.1.5 ✅ | Spring spark and hook jerk | victory → spring spark; hand it in to Tüftel → ability **hook jerk** (consequence `faehigkeit hook-ruck`) with explanation via signpost text; first spot that only works with the hook jerk (returning to `wiese-1` pays off: hidden collectible) | Tests |
| M2.1.6 ✅ | Village after chapter 1 | flag `quellen_befreit = 1`: part of the flower beds, flower boxes and flags colourful; festive moment at the well (short conversation with everyone); world map shows the Blütenwiesen as freed | Visual check |
| M2.1.7 ✅ | Music and sounds | boss music (CC0 or CC BY, presented for listening, E-285); sounds for the bumblebee (buzzing, dive, hit), bee (collecting), spring spark (fanfare) from Kenney or other free sources | Your listening test |
| M2.1.8 ✅ | Acceptance | play through chapter 1, save, continue | Your acceptance |

**Status M2.1.1:** Behaviour `diver` (guardian from the air) in the simulation: sleeps until Elora arrives; circles and drops pollen, takes aim, dives down, lies dazed (only then vulnerable, otherwise hits bounce off with little stars, no contact damage) and rises again; from half health faster with two dives, from one third it calls confused bees (at most 3). Values in `creatures.toml` (`brummbaer`, `wirrbiene`), graphics with flight, dive and dazed poses, health bar with name at the top of the HUD, flag `besiegt.<art>` after the victory (for doors, conversations, quests), loot spring spark. The arena (entrance and exit) is created with the maps (M2.1.4): jumping down into the arena, the exit opens with `merker besiegt.brummbaer` (doors stay open, E-254). Try it: Training, F1 → "Enemies (Adventure)" → `brummbaer`.

**Status M2.1.2:** new tile **hook flower** (`Tile::HookPoint`, character `*`, in the editor among the tiles): the hook grabs its centre, characters, projectiles and lasers pass through. While a guardian from the air is enraged, half of the flowers wilt in turn (even/odd columns, 2.5 s each); a hook on a wilting flower lets go. Rendered fresh/wilted, swaying slightly. Try it: Training, three hook flowers above the thorn pit.

**Status M2.1.3:** Characters **Wabe** and **Bumblebear** (appears only after the fight, new field `show_if` in `characters.toml`), own images for bee, spring spark and honeycomb hat. Main quest "The Blossom Spring": `wiese-2` → Wabe → `wiese-3` → `wiese-arena` → calm the bumblebee → spring spark to Tüftel (hook jerk, `quellen_befreit = 1`, flag `fest`, yard course `hof.ruck`) → festival at Oma's; afterwards "Whispers in the Murmelwald" (announcement). Side quest "Wabe's Bees" (5 bees → honeycomb hat, healing blossoms heal more). Conversations: Wabe, Bumblebear (hints at the grey shadow, flag `duerrer.gesehen`), Oma (spring spark, festival, Murmelwald), Tüftel (hook jerk with `{taste:ability}`), Klonk (grenade launcher after chapter 1, E-243); new call-outs in the village. Tests in `tests/story.rs`.

**Status M2.1.4:** Maps from `apps/elora-client/src/editor/chapter1.rs` (rewrite: `write_chapter1_maps -- --ignored`, overview `chapter1_sheets`): `wiese-2` (300 × 60: apiary with Wabe and spring stone, giant flower forest with leaf walkways and a blossom crown, stream with stepping stones, hook course over a thorn hollow), `wiese-3` (220 × 70: shaft down into the cave under the roots, hanging roots over thorn pits, crumbling floor, spring stone, niche in the ceiling, ascent over walkways), `wiese-arena` (90 × 50: ledge with spring stone, arena with two rows of hook flowers, Bumblebear, gate after the victory, root path back to `wiese-1`). `wiese-1` now continues to the east. Decoration with state: `bluetenquelle-verdorrt` blooms after `befreit.bluetenquelle` (conversation with the bumblebee), festive decorations `…-fest` only with flag `fest`. Bees: 1 in the blossom crown, 2 and 3 in the cave, 4 on the high ledge above the hook course (becomes a hook jerk spot with M2.1.5), the fifth comes with M2.1.5.

**Status M2.1.5:** Victory → spring spark (loot) → conversation with the bumblebee (spring blooms) → Tüftel builds the **hook jerk**. **Jerk spot** as a building block (`ruck_gate` in `editor/prologue.rs`): stone shaft with a hook flower, next to it a stone tower 9 tiles above the flower, a passage below. Measured: without the jerk at most about 8.8 tiles above the flower, with the jerk about 10.8 – the test `ruck_gate_needs_the_hook_ruck` (elora-sim) searches many timings for hook, release, jumps and jerk and makes sure the tower is reachable only with the jerk. Used behind Tüftel's workshop (chest, sign "schild-ruck"), in `wiese-2` (bee 4 on the old stone tower) and in `wiese-1` (bee 5, return).

**Status M2.1.6:** After the hook jerk (`quellen_befreit = 1`) part of the flower beds, flower boxes, flower pots and flags turns colourful. **Festival** (flag `fest`, E-301): garlands with lanterns at the well square and in the upper village (decoration `…-fest`), festival music via the area's `party_music` (file comes with M2.1.7), call-outs from Oma, Tüftel, Klonk and Lotte; the festival ends as soon as Elora leaves Tauwinkel. World map: freed areas with a halo and "Spring freed" (field `spring` in `worldmap.toml`).

**Status M2.1.7:** Music: `boss.ogg` ("Urban Boss Battle", mintodog) while a guardian is awake, `fest.ogg` ("Minstrel Dance", randommind) during the festival in Tauwinkel (E-304). Sounds: event `CreatureAct` (awakening, dive, impact) with buzzing and whistling (procedural, `sounds.toml`) and a soft impact; bouncing hits chime; collectible, spring spark and completed quest with Kenney sounds (CC0, `assets/SOURCES.md`).

## Flow of chapter 1 (proposal)

| # | Location | What happens |
|---|---|---|
| 1 | `wiese-1`, meadow edge | Spring stone; quest "The Blossom Spring": deeper into the meadows |
| 2 | `wiese-2` | **Beekeeper Wabe** at her apiary: the bees have swarmed out because the bumblebee is raging; side quest "Wabe's Bees" (1 bee lies here) |
| 3 | `wiese-2` | Stream and giant flowers, hook course; enemies as before, plus more pollen puffers |
| 4 | `wiese-3` | Caves under the roots, thorns; spring stone before the ascent; 2 bees |
| 5 | `wiese-arena` | Blossom Spring withered, the **Bumblebear** circles above it; fight |
| 6 | `wiese-arena` | Victory: the bumblebee lands exhausted and was only confused (conversation); spring spark, the spring blooms |
| 7 | Tauwinkel | Tüftel builds the **hook jerk** and opens the new part of his yard; **small celebration** at the well (lanterns, garlands, festival music); Oma tells of the second spring (announcement Murmelwald) |
| 8 | free | bring the remaining bees (2 of them only with the hook jerk) to Wabe: **honeycomb hat** and experience |

## The fight against the Bumblebear (proposal)

- **Phase 1 – Circling:** The bumblebee flies circles high above the arena and drops pollen now and then (like pollen puffer balls). Elora stays up high with the hook on the **hook flowers**.
- **Phase 2 – Dive:** It takes aim at Elora (short warning: buzzing gets louder, shadow on the ground) and dives down. If Elora dodges, the bumblebee stays **briefly dazed on the ground** – now the hammer hits fully, other weapons at half.
- **Phase 2b – from half health:** faster, two dives in a row, the hook flowers wilt in turn.
- **Phase 3 – from one third health:** It calls **small, confused bees** that buzz around Elora and deal light damage (defeatable). After the fight they fly back to Wabe.
- **Hits only while it is dazed** – in the air they bounce off (little stars), so that dodging counts (E-299).
- Elora dies? Back to the spring stone before the arena (E-261), the bumblebee is back at full health.

## Decisions on R2-M2.1

| # | Question | Decision |
|---|---|---|
| D-M21-01 | Number of new sections | **Three:** `wiese-2`, `wiese-3`, `wiese-arena` (E-297) |
| D-M21-02 | Fight against the bumblebee | **Harder:** circling with pollen, dive, dazed; from half health faster with wilting hook flowers; third phase with summoned bees (E-298) |
| D-M21-03 | How the bumblebee is vulnerable | **Only when dazed on the ground** (E-299) |
| D-M21-04 | Reward for the bees | **Equipment:** honeycomb hat with a small bonus, plus experience (E-300) |
| D-M21-05 | Festival in the village after chapter 1 | **Small celebration:** conversation at the well, lanterns, garlands and festival music until Elora moves on (E-301) |
| D-M21-06 | Explaining the hook jerk | **Practice area in the yard:** Tüftel explains and opens a new part of his yard (E-302) |

## Technical decisions (proposal)

- **Boss behaviour in the simulation** (deterministic, like all enemies), phases and values as data in `creatures.toml`; the client only displays.
- **Hook flower** as a new "hook point" tile (hookable, passable for characters) instead of an object – then it also works in the multiplayer editor; rendered as a flower via the materials.
- **Maps still from generators** like Tauwinkel and `wiese-1`; collectibles, enemies and conversations as objects.
- The playthrough test `tests/prolog.rs` is extended with chapter 1 (conversations, quests, transitions, boss victory via events, save/continue).

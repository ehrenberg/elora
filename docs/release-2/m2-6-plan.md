# R2-M2.6 – Finale: The Old Well – implementation plan

Status: **approved** (E-380, spring trials E-381) – in progress · Basis:
[`world-book.md`](world-book.md) §2, §4.6 and §5, [`m2-5-plan.md`](m2-5-plan.md),
[`story-drafts.md`](story-drafts.md) (“the last boss fight is a reawakening”), E-210 to E-214

## Goal

The finale is playable from start to finish: after chapter 5 every spring sings again except the
old well in Tauwinkel. Elora follows the old well song, opens the well and climbs down into the
**Origin Cave** below the village. In its heart sits **the Withered One** – the forgotten guardian
of the sixth spring, the **Origin Spring**. There is no fight to defeat him: Elora uses every
ability she has learned to bring him the five colours back, and finally plays him the well song
that nobody sang for him. He becomes the guardian of the Origin Spring again, the well opens, the
village celebrates with all guardians, and Oma Pfütze finally remembers the whole song.
Afterwards the adventure continues as free play, and Born offers the **spring trials**: every
section of the adventure again, harder and timed, for bronze, silver and gold.

**Acceptance:** play through the finale once (about 30–45 minutes, world book §8), saving and
continuing in between; the ending, the credits and free play afterwards work; a few spring
trials played with their medals.

## What the story has prepared

- **Murmelwald runes** (chapter 2, Plumm): “Five songs the springs did sing, yet beneath them all
  lay a sixth, quiet song. The Origin Spring gave the others their dew. Its keeper was lonely, for
  no one heard its song. Where the oldest well stands, the sixth spring sleeps.” (flag
  `sixth_spring`)
- **Sirup** (chapter 3): a grey wanderer; **Kristella** (chapter 4): “He is lonely, not wicked. He is
  looking for something someone took from him.”
- **Luma** (chapter 5): he hummed an old **well song** as he left, towards Tauwinkel; flag
  `withered_one.well_song`.
- **Oma** (end of chapter 5): remembers only the first line – „Tief unten, wo das Wasser
  schläft, …“ – and that her mother sang it at the well. Quest `finale` (“The Old Well”) has one
  placeholder step.

## Core decisions

Approved on 2026-11-12: **E-380** – the proposals of D-M26-01 to D-M26-08 and D-M26-10;
**E-381** – D-M26-09 differently: the spring trials are **part of M2.6** (section below).

| # | Question | Proposal (approved unless noted) | Alternative |
|---|---|---|---|
| D-M26-01 | How does the well open? | **Gather the song:** each guardian remembers **one line** of the well song (Bumblebear, Root Warden, Sand Serpent, Kristella, Luma); with all five lines Oma sings the song at the well and the lid lifts by itself. A short journey back through all areas (world map, fast travel at spring stones if present) | **Klonk and Tüftel** open the lid with a tool – quicker (no journey), but the guardians play no part |
| D-M26-02 | Origin Cave | **Three maps:** `well-shaft` (the climb down, mixed abilities, dark → lighter), `origin-cave` (five **colour veins**, one per ability and spring), `origin-heart` (the finale) | One big map with all of it |
| D-M26-03 | Colour veins | In each vein a **colour drop** of one spring waits behind an ability spot: green – hook jerk, brown – pull hook, gold – stomp, ice blue – ice grip, violet – gliding; Elora collects them in any order | A fixed order (more like a tutorial, less freedom) |
| D-M26-04 | The finale (“reawakening”, world book §4.6) | **Five rounds, one colour each:** the Withered One curls up in the middle of the heart; in each round grey waves and shadows fill the cave (as in “Protect Luma”) while a colour drop appears in a place only one ability reaches; Elora carries it to him (it floats behind her, a hit makes her drop it); every colour he gets back changes the cave (light, plants, water). After the fifth: **stage 2** below | Bring the five drops collected in the veins straight to him (shorter, without danger) |
| D-M26-05 | Stage 2: the song | Five **note stones in the spring colours** appear around him; Elora plays the **well song** (a tune of five notes, one per line) – he opens his eyes, turns blue-white and the Origin Spring rises. No failure here, it is a calm end | The rounds alone end the finale |
| D-M26-06 | Failure | No game over: if the grey waves fill the heart, Elora wakes at the spring stone in front of it; colours already given back **stay** | Restart all rounds |
| D-M26-07 | His new form and name | Drafts in M2.6.0: a tall clear drop with a crown of five coloured streams; name: **Born** (old word for spring and well) | Keeps no name (“the guardian of the Origin Spring”) |
| D-M26-08 | Ending | Scripted ending scene (the intro technique): water and colour rise up the well, the five guardians come to the well square, Oma sings the whole song with everyone; victory screen with title of honour **„Quellenhüterin“**, then the **credits**; afterwards Elora stands at the well – free play | Victory screen and credits only |
| D-M26-09 | Free play afterwards | The village stays celebrating (Born at the open well, guardians visit now and then), all side quests stay open, the save shows “Finale done”; the **spring trials** (world book §8) become a **milestone of their own** after Release 2's story – **not chosen:** the owner took the alternative (E-381) | **Spring trials inside M2.6 – chosen** (E-381) |
| D-M26-10 | Reward of the Origin Spring | **No new ability**; Born gives a **dew crown** (hat: +1 health, all spring colours as a glow) and every guardian gives a small keepsake in the celebration | A sixth ability (larger, changes the base game, E-226) |

Texts (dialogues, the five lines of the well song, the ending) follow as a content draft
`m2-6-content.md` for approval once the core decisions are made (as for chapters 4 and 5,
E-346/E-377). **The well song itself is a text for the owner**: proposal in the content draft,
the owner decides.

## Starting point (from R2-M1 to R2-M2.5)

- All five abilities exist (hook jerk, pull hook, stomp, ice grip, gliding) with their map spots
  (`jerk_gate`, `pull_vault`, `stomp_vault`, `climb_vault`, `flap_pillar`, updrafts, islands).
- **Climax module** (`session/climax.rs`, M2.5.5): rounds, grey shadows from spawn zones, star
  lanterns, a song bar, failure back to the spring stone, the Withered One as a scripted figure
  with modes (drain, look, flee) – the base for the finale rounds.
- **Tunes** on note stones (`[[tune]]`, M2.5.6) – the base for the well song.
- **Companions** (`session/followers.rs`, E-308) – the base for a colour drop that floats behind
  Elora.
- Darkness and light (M2.5.1), weather, decor by world state (`…-withered` → `…-freed`,
  `springs_freed`), the victory screen per area (`victory`, `honor`), the intro with keyframes
  (`app_intro.rs`) and the credits page (E-209).
- Content: quest `finale` with a placeholder step, flags `sixth_spring`,
  `withered_one.well_song`, the old well decor in Tauwinkel (stays grey).

## Work steps

| # | Step | Content | Check |
|---|---|---|---|
| M2.6.0 | Drafts | Born (two variants), the Withered One curled up and in his colour stages, colour drops (five), the Origin Cave (well shaft, colour veins, heart before/after), the open well and the Origin Spring, dew crown, the ending scene (key pictures) | Your selection |
| M2.6.1 | Way to the well | Depending on D-M26-01: guardians remember their lines (dialogues after their chapter), Oma sings at the well, the lid opens (decor `well-sealed` → `well-open`), entrance to `well-shaft` | Tests |
| M2.6.2 | Colour drops | Collectible colour drops that **float behind Elora** while carried (companion), are dropped by a hit and fly back to their place; delivering one at a zone counts it | Tests + sandbox |
| M2.6.3 | Finale tech | `session/finale.rs` from the climax module: five rounds with grey waves and shadows, a drop per round at an ability spot, cave changes per colour (decor and light by flag), stage 2 with five note stones and the well song tune, failure back to the spring stone (colours stay) | Tests + sandbox |
| M2.6.4 | Content | Content draft `m2-6-content.md` (approval), then characters, dialogues, quest `finale` with its steps, the well song, rewards | Tests |
| M2.6.5 | Maps | `well-shaft`, `origin-cave`, `origin-heart` from a generator `editor/finale.rs`; the well square with the open well | Visual check + run-through test |
| M2.6.6 | Ending | Ending scene (keyframes), victory screen „Quellenhüterin“, credits, back to the well square in free play | Your look |
| M2.6.7 | Village after the finale | Born at the well, guardians visiting, calls, the celebration stays; save marked “finale done” | Tests |
| M2.6.8 | Music and sounds | Music for the cave and the finale, an ending theme (CC0, presented for listening); sounds for colour drops, grey waves, the opening well, Born | Your listening test |
| M2.6.9 | Trial tech | Trial mode in the session: start a section map in its harder version (extra enemies from map objects marked `trial`, enemy health and damage × factor), timer from entering to the exit, best time and medal in the save game, back to Born with the result; no healing changes (E-381) | Tests |
| M2.6.10 | Trials content | Born's trial menu (all section maps of the five areas plus `well-shaft` and `origin-cave`, only those Elora has visited), extra enemies per map, medal limits computed from path length and running speed (a tool that measures the main path per map) | Tests + sandbox |
| M2.6.11 | Acceptance | Play through the finale, save, continue; ending and free play; some trials with medals, limits adjusted | Your acceptance |

## Spring trials (E-381)

- **Where:** after the finale, Born at the open well in Tauwinkel offers the trials (a menu like
  the shops: list of trials with best time and medal).
- **Which:** one trial per **section map** of the five areas (Blütenwiesen to Sternschlucht, not
  the guardian arenas – no guardian fights) plus `well-shaft` and `origin-cave`; a trial is
  listed once Elora has been on that map.
- **Harder:** the same map with **more enemies** (extra spawn objects marked for trials) and
  **stronger enemies** (more health, more damage); healing stays as it is.
- **One run:** the harder version is timed from the entrance to the exit of the map; dying or
  leaving ends the run without a time.
- **Rating:** the **best time** is saved; **bronze, silver, gold** for fixed limits, computed from
  the main path length and Elora's running speed (values proposed per map, adjusted at the
  acceptance).
- **Elora** plays with her own level, equipment and skills; the trial does not change the save
  game except best times and medals.
- **Afterwards** Elora is back with Born, who shows the result. **Rewards:** medals only.

Open (to be asked when building M2.6.9/M2.6.10): exact factors for stronger enemies, how many
extra enemies, the medal formula, whether a trial uses its map's weather and darkness.

## Course of the finale (proposal)

1. **The song** (D-M26-01): Oma: “Every guardian heard the well song once, a long time ago.” Elora
   visits the five guardians; each remembers one line. Back at the well Oma sings all five lines,
   the grey lid cracks and lifts. Pip: “I'm coming too!” – Oma: “No.”
2. **`well-shaft` – down the well:** a deep shaft with ledges, roots and old stone steps; every
   ability once, getting lighter towards the bottom (glowing water veins).
3. **`origin-cave` – the colour veins:** a hall with five side passages in the spring colours; each
   holds a colour drop behind its ability spot. Grey patches everywhere; the Withered One's
   footprints lead to a closed stone door that opens once all five drops are found.
4. **`origin-heart` – the reawakening:** the Withered One, curled up and grey beside a dry basin.
   “Nobody came. Nobody sang.” Five rounds (D-M26-04), one colour each – with every colour he
   looks up a little more. Then the five note stones (D-M26-05): Elora plays the well song, he
   speaks his name, the Origin Spring rises.
5. **Ending** (D-M26-08): water rises through the well, colour spreads over Tauwinkel, the
   guardians arrive, Oma sings, credits; free play at the well.

## Technical specifications (proposal)

- **Finale session** `session/finale.rs` beside `climax.rs`: shared parts (shadow spawning,
  failure, the scripted figure) move into a small helper both use; phases `Round(n)`, `Song`,
  `Rising`; flags `finale.colour.<spring>` per returned colour, `finale.done`.
- **Colour drop:** creature kind `colour_drop` (no damage, follows Elora like a companion while
  carried); `ObjectKind::Collectible` with item `drop_<spring>` starts carrying; a hit (damage
  event on Elora) drops it; zone `drop-goal` delivers it.
- **Cave changes:** decor `…-grey` → `…-<spring>` by the flags (the mechanism of
  `adapt_decor`), light sources per colour in `assets/map/lights.toml`, darkness of the heart
  falling from 0.9 to 0.2 with each colour.
- **Ending scene:** keyframes like the intro (`app_intro.rs`), playable from the menu
  afterwards (“Ending”), skippable.
- **Save game:** flag `finale.done`; the world map shows the Origin Spring; free play needs no
  new state.

## Approved

- Core decisions (E-380, E-381), the steps and order, the course of the finale and the map names
  `well-shaft`, `origin-cave`, `origin-heart` (2026-11-12).

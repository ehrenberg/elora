# R2-M2.3 – Chapter 3: Glutsandwüste – Implementation Plan

Status: **Accepted** (E-327) · Decisions E-315 to E-327 · Basis: [`world-book.md`](world-book.md) §4.3 and §5, [`m2-2-plan.md`](m2-2-plan.md), E-243, E-296, E-314

## Goal

Chapter 3 is playable from start to finish: from Tauwinkel into the Glutsandwüste, caravan leader **Sirup** with rare goods, an **oasis** that needs water, a **buried ruin**, traces of a creature that drinks colour, the guardian **Sand Serpent** and the new ability **stomp**. Afterwards Klonk hands over the **laser** (E-243), and Tauwinkel gets even more colourful (`quellen_befreit = 3`).

**Acceptance:** Play through chapter 3 once (about 60–90 minutes with side quests), saving and continuing in between.

## Starting point (from R2-M1 to R2-M2.2)

- Enemy behaviours: walker, hopper, shooter (also in an arc), flyer, guardian from the air (`diver`), root snake (`burrower`), guardian on the ground (`warden`), companion (`follower`); contact with colourful rush.
- The ability **stomp** exists in the simulation (A1.1): breaks crumbling floor, the shockwave stuns and damages enemies.
- Building blocks: map generators (mirroring possible), jerk spot, pull vault, hook flowers, decoration with state, characters with `show_if`, companions, music and battle music per area.
- The material **sand** and the release theme "Desert" (sky, tint) already exist; booster tiles (`<` `>`) as conveyor belts.
- After chapter 2 "Tracks in the Sand" is running (announcement, manual step).

## Work steps

| # | Step | Content | Check |
|---|---|---|---|
| M2.3.0 ✅ | Drafts | Sirup and his caravan (camel or similar), oasis keeper (side quest character), sand crab, dune worm, spark moth, Sand Serpent (under the sand, surfacing, arc through the air, dazed, defeated), water skin, ruin stone tablet, spring (withered/freed), decoration: dunes, rock arches, ruins (pillars, gates, stairs), cacti, palms, oasis, tents | Your choice |
| M2.3.1 ✅ | New enemies | **Sand crab**: armoured, hits from the side bounce off, vulnerable from above (hammer from above, stomp, grenade above it). **Dune worm**: travels under the sand (sand trail), leaps out in an arc and dives back in. **Spark moth**: flies, drops sparks that glow briefly on the ground | Tests + sandbox |
| M2.3.2 ✅ | Quicksand and heat | new tile **quicksand**: Elora slowly sinks in and runs slower, jumping frees her, when sunk deep a little damage and back to the edge (E-318); **heat shimmer** as a screen effect and **heat bar** in the HUD: sun fills it, shade (zones/roofs) and oasis cool it down, full = slower (E-320) | Tests |
| M2.3.3 ✅ | Guardian technology | **Sand Serpent**: dives under the sand (only the sand trail is visible), shoots up at Elora's position, flies in an arc and dives back in; vulnerable only when surfaced; from half health faster, at the end two arcs in a row | Tests + sandbox |
| M2.3.4 ✅ | Content | Characters Sirup and oasis keeper; main quest "Tracks in the Sand" up to the spring spark; side quests **"Water for the Oasis"** and **"The Buried Ruin"**; traces of the grey wanderer (story); conversations in the village after chapter 3; Klonk hands over the laser | Tests |
| M2.3.5 ✅ | Maps | Path from Tauwinkel into the desert; `wueste-1` to `wueste-3` and `wueste-arena`: dunes, quicksand conveyor belts, ruins with crumbling floors (return with stomp), oasis, caravan camp, spike pits; jerk and pull spots | Visual check + playthrough test |
| M2.3.6 ✅ | Spring spark and stomp | Victory → spring spark → Tüftel builds the **stomp**; practice spot in Tüftel's yard; spots in chapters 1 and 2 that only work with the stomp | Tests |
| M2.3.7 ✅ | Village after chapter 3 | `quellen_befreit = 3`, festival, conversations and call-outs, world map | Visual check |
| M2.3.8 ✅ | Music and sounds | Music for the desert and the guardian (presented for listening), sounds for crab, worm, moth, quicksand, Sand Serpent | Your listening test |
| M2.3.9 ✅ | Acceptance | Play through chapter 3, save, continue | Your acceptance |

**Status M2.3.0–M2.3.7:** Drafts accepted (E-321). Enemies sand crab (armour, E-317), dune worm (`leaper`), spark moth (sparks glow on the ground). Quicksand tile `&` (E-318), heat bar and heat shimmer as a shader (E-320, E-321). Sand Serpent (`serpent`, E-316). Content according to [`m2-3-content.md`](m2-3-content.md) (E-322 to E-325). Maps `wueste-1` to `wueste-3`, `wueste-arena`, sunken lane at the east path (E-315); the arena's basin lies in the shade. Stomp plate in Tüftel's yard, stomp chambers in `wiese-2`, `wald-1`, `wald-3`. Village after chapter 3: `quellen_befreit = 3`, festival, Lotte gives away cactus fruit from Sirup, Klonk explains the laser upgrade with ember stone, new call-outs from Pip, Lotte and Tüftel; world map: Ember Spring freed (`spring = "glutquelle"`).

**Status M2.3.8:** Music (E-326): Glutsandwüste "Desert Loop" (iamoneabe, CC0), Sand Serpent "Hard Boss Battle 1" (MintoDog, CC0). Sounds per enemy species: sand splashes (worm, serpent), sand quakes before the leap, back into the sand (Kenney), sparks crackle, armour clacks (Kenney), the serpent hisses, squelching when stepping into quicksand (procedural).

## Flow of chapter 3 (proposal)

| # | Location | What happens |
|---|---|---|
| 1 | Tauwinkel | Oma sends Elora off; a path leads from the upper village over the east path out into the desert |
| 2 | `wueste-1` | Dune edge, first sand crabs and dune worms, quicksand conveyor belts |
| 3 | `wueste-2` | Caravan camp: **Sirup** with rare goods tells of the grey wanderer; nearby the **oasis**, almost dried up (side quest water) |
| 4 | `wueste-3` | Ruins of an ancient people: tablets with traces of a creature that drinks colour; buried chamber (side quest); spark moths; spring stone |
| 5 | `wueste-arena` | Ember Spring in a sand basin, the **Sand Serpent** sleeps beneath it; fight |
| 6 | `wueste-arena` | Victory: the serpent was confused and tired; spring spark; grey footprints lead north (Frostspitzen) |
| 7 | Tauwinkel | Tüftel builds the **stomp**; Klonk hands over the **laser**; festival; Oma announces the Frostspitzen |
| 8 | free | Water for the oasis, uncover the ruin (with stomp), return to chapters 1 and 2 |

## The fight against the Sand Serpent (proposal)

- **Phase 1:** Only a sand trail travels through the basin. The sand quakes under Elora (warning), then the serpent shoots up, flies in an arc and dives back in. While it is in the air and lies dazed on the ground shortly afterwards, it is vulnerable.
- **Phase 2 – from half health:** faster, parts of the basin turn into quicksand.
- **Phase 3 – last quarter:** two arcs in a row.
- Hammer and grenades hit; with stomp (once available, e.g. on a later return) double damage (E-316).

## Decisions on R2-M2.3

| # | Question | Decision |
|---|---|---|
| D-M23-01 | Path into the desert | **Branch at the east path:** behind the east path the way forks, to the south a sunken lane leads down into the desert (E-315) |
| D-M23-02 | Sand Serpent and stomp | **Hammer and grenades hit** the surfaced serpent; stomp (after the fight) later deals double damage (E-316) |
| D-M23-03 | Sand crab | **Vulnerable only from above** (strike from above, stomp, grenade on top), hits from the side bounce off (E-317) |
| D-M23-04 | Quicksand | **Sinks in slowly and slows down, jumping frees;** when sunk deep a little damage and back to the edge (E-318) |
| D-M23-05 | Oasis quest | **Water skin:** fill it at a spring in the ruins and water three withered spots of the oasis, each one blooms (E-319) |
| D-M23-06 | Heat | **Heat shimmer as a screen effect and a heat bar:** it fills up in the blazing sun and cools down in the shade and at the oasis; full = Elora gets slower (E-320) |

## Technical decisions (proposal)

- New behaviours and the guardian in the simulation (deterministic), values as data in `creatures.toml`.
- **Quicksand** as a new tile in collision, map and editor (like ice and boosters), graphics as a material.
- Sand crab armour: hits evaluate the direction (strike from above, shockwave), otherwise they bounce off as with the guardian.
- Maps from a generator `editor/chapter3.rs`; playthrough test extended with chapter 3.

# R2-M2.4 – Chapter 4: Frostspitzen – implementation plan

Status: **Accepted** (E-349) · Decisions E-340 to E-349 · Basis: [`world-book.md`](world-book.md) §4.4 and §5, [`m2-3-plan.md`](m2-3-plan.md), [`w1-plan.md`](w1-plan.md), E-228, E-243, E-320

## Goal

Chapter 4 is playable from start to finish: from Tauwinkel up into the Frostspitzen, mountain guide
**Flocke** in the abandoned mountain village, the new ability **ice grip** already in the middle of the chapter,
**cold** that Elora drives off at fireplaces, icicles, avalanches and thin ice, lost
climbers and ice crystals for Klonk, the guardian **Ice Queen Kristella** and her warning:
“He is lonely, not wicked.” Afterwards Tauwinkel becomes even more colourful (`quellen_befreit = 4`), and Oma
announces the Sternschlucht.

**Acceptance:** play through chapter 4 once (about 60–90 minutes with side quests), saving and
continuing in between.

## Core decisions

| # | Question | Decision |
|---|---|---|
| E-340 | Ice grip | **In the middle of the chapter:** Flocke gives Elora climbing claws (= ice grip) in the mountain village; from then on paths open up, and the guardian fight uses the walls. The spring spark after the victory **strengthens** the ice grip |
| E-341 | Fight against Kristella | **Frost waves and walls:** she hovers above an ice hall and freezes the floor in waves (freshly frozen floor hurts); Elora escapes onto climbing walls; after the frost breath Kristella is exhausted, sinks down and is vulnerable; later icicles fall |
| E-342 | Cold | **Cold bar** (counterpart to the heat, E-320): it fills outside, faster in a blizzard; Elora warms up at fireplaces and under roofs; full = Elora gets slower |
| E-343 | New terrain | **Icicles, avalanches, thin ice** (no jump pads) |

## Starting point (from R2-M1 to R2-W1)

- **Ice grip** exists in the simulation (A1.1, E-228): run against a **climbing wall tile** in the air,
  cling briefly (A-06: 1.0 s), jump off; in the skill tree “Firm grip” (cling time).
- Tiles **ice** (slippery), **climbing wall**, **crumbly floor** (breaks when stomping), quicksand.
- **Weather** (R2-W1): snow and blizzard with wind that pushes Elora in the air, wet
  ground; area `frostspitzen` in `worldmap.toml` (maps `frost-`, dull: blizzard, otherwise
  snow or fair).
- **Heat bar** (E-320) with zones `schatten…`/`oase…` and roof detection – template for the cold.
- Enemy building blocks: walker, hopper, shooter (also arcing), flyer (with drop), `diver`,
  `burrower`, `leaper`, `warden`, `serpent`; guardian tech with phases, warnings, exhaustion.
- Material **ice crystal** and upgrades at Klonk that need it (`upgrades.toml`); quest
  “The Call of the Frostspitzen” (teaser, step set by hand).

## Work steps

| # | Step | Content | Check |
|---|---|---|---|
| M2.4.0 ✅ | Drafts | Flocke, climbers (3), Kristella (hovering, frost breath, exhausted, calmed), snowball seal, ice bat, frost ghost, climbing claws, icicle, avalanche snowball, thin ice and ice water, fireplace, spring (frozen/freed); decoration: peaks, firs in snow, mountain huts, rope bridges, glacier | Your selection |
| M2.4.1 ✅ | Terrain in the simulation | **Icicles** (tremble, fall, shatter), **thin ice** (new tile: breaks after standing on it briefly or instantly when stomping, grows back), **ice water** (new tile: small damage, back to the edge), **avalanches** (zone: stomping or an explosion sets off rolling snow chunks) | Tests |
| M2.4.2 ✅ | Cold | Cold bar in the HUD, frost border as an image effect (post shader); fireplaces and huts warm; values as tuning | Tests |
| M2.4.3 ✅ | New enemies | **Snowball seal**: slides up on its belly, stops and throws snowballs in an arc. **Ice bat**: hangs asleep from the ceiling, swoops down when Elora is below, flutters back. **Frost ghost**: floats through walls, touch makes Elora freeze briefly | Tests + sandbox |
| M2.4.4 ✅ | Guardian tech | **Kristella** per E-341, phases see below | Tests + sandbox |
| M2.4.5 ✅ | Content | Characters Flocke and climbers; main quest up to the spring spark with climbing claws in the middle; side quests **“Lost Climbers”** and **“Ice Crystals for Klonk”**; tracks of the Withered One; village dialogues after chapter 4 | Tests |
| M2.4.6 ✅ | Maps | Mountain path out of Tauwinkel; `frost-1` to `frost-3` and `frost-arena` | Visual check + run-through test |
| M2.4.7 ✅ | Spring spark and village | Victory → spring spark → Tüftel strengthens the ice grip; climbing spots in chapters 1–3 (going back pays off); `quellen_befreit = 4`, celebration, world map; Oma announces the Sternschlucht | Tests + visual check |
| M2.4.8 ✅ | Music and sounds | Music for the Frostspitzen and Kristella (presented for listening), sounds for seal, bat, ghost, icicle, avalanche, breaking ice, fireplace, Kristella | Your listening test |
| M2.4.9 ✅ | Acceptance | Play through chapter 4, save, continue | Your acceptance |

**Status M2.4.0–M2.4.1:** Drafts accepted (E-345). Tiles **thin ice** (`-`) and **ice water** (`+`) in simulation, map format, editor and graphics; cracks as a warning, breaking and regrowing via the temporary tiles (never grow into a character). Enemy kinds **`eiszapfen`** (behaviour `icicle`) and **`schneebrocken`** (`roller`); **avalanches** as zones `lawine…` with trigger zone `…-tritt` in the session. Values A-36 to A-41. Sounds placeholders for now (M2.4.8).

**Status M2.4.2:** Cold bar in the HUD (snowflake, light blue to deep blue, pulses when full) for areas with `cold = true` (Frostspitzen): outside 60 s until full, in a blizzard 30 s, under roofs 10 s and in zones `feuer…` 3 s until empty, it warms up in guardian arenas; full slows down like the heat (A-27) until below half. Bonus `cold_pct` for equipment. Frost border as a post shader (frost flowers from the edges, from a third of the bar).

**Status M2.4.3:** New behaviours `seal` (snowball seal: slides up, straightens up within throwing range and throws in an arc every 1.5 s), `bat` (ice bat: sleeps upside down and harmless, swoops down on Elora as soon as she is below, flutters home and rests briefly), `ghost` (frost ghost: floats through walls, touch with `freeze_ms` freezes Elora for 0.6 s – only aiming works, she is stuck in an ice block –, then it backs off for 1.8 s; not hookable). Kinds `schneeballrobbe`, `fledermaus`, `frostgeist` with loot (ice crystal, rare). Try it: training, F1 → “Gegner (Abenteuer)”.

**Status M2.4.4:** Behaviour `queen` (kind `kristella`, 36 health): sleeps until Elora is close, hovers above the hall (1100 wide) and sends a frost wave across the floor every 2.6 s (front 6 units/tick, behind it 170 of fresh frost with 2 damage and a push upwards, ahead of it rime creeps as a warning) – safe on climbing walls, on ledges and while jumping. When calm, the wave comes from the side Elora is not on. After three waves she sinks down exhausted and is vulnerable for 3.4 s from landing. From half health: faster, waves alternating from both sides, with three icicles above Elora in between. In the last quarter she calls up a blizzard (the session sets the hall's weather: wind pushes Elora off the wall, visuals and sound); after the victory it dies down.

**Status M2.4.5:** Content per [`m2-4-content.md`](m2-4-content.md) (E-346): characters Flocke, Bolle, Kiesel, Wicke (outside and afterwards in the hut), Kristella after the fight, grey patch; main quest “The Call of the Frostspitzen” with climbing claws (ice grip) for Flocke's rope, then the teaser “The Song of the Stars”; side quests “Lost Climbers” (bobble hat) and “Clear Crystals for Klonk” (pendant of your choice); herbal tea (effect `warm`), fur boots, shop `flocke`; dialogues for Oma, Tüftel (flag `eisgriff.stark`), Klonk; five signs; area with spring, chapter 4, guardian and victory screen (“Summit Climber”).

**Status M2.4.6:** Maps from `editor/chapter4.rs`: **mountain path** in Tauwinkel (ice lid of crumbly floor at the top of the upper village, stomp only; below it the passage to the transition). **`frost-1` glacier foot** (fire at the entrance, ice surfaces, rock passage with three icicles, thin ice bridge over ice water with a crystal underneath on dry rock, Bolle's niche only via a climbing shaft, avalanche slope with a trigger spot). **`frost-2` mountain village** (cheese dairy with cellar: thin ice over ice water, two bats, rope chest; Flocke's hut with a fire and the returned climbers; climbing chimney behind the hut, 26 rows high; plateau with Kiesel's glacier crevasse). **`frost-3` summit ridge** (fixed blizzard, two avalanche slopes, valley with Wicke's ledge above a chimney, bats under a rock overhang, grey patch, three fireplaces, spring stone in front of the hall). **`frost-arena` ice hall** (ice floor exactly as wide as the frost waves, climbing walls and two climbing pillars, ceiling for the icicles, gate after the victory). Eight clear crystals; in the mountains only clouds and distant peaks in the background. Tests: chimney climbable with ice grip (inputs only), run-through of the chapter on the maps.

**Status M2.4.7:** Strengthened ice grip (flag `eisgriff.stark` from Tüftel): double cling time, pushing towards the wall pulls Elora up with A-42; applies right after the dialogue. Climbing spots for going back in `wiese-2`, `wald-1` and `wueste-2` (hanging chimney, open at the bottom, ledge with a chest 19 rows high, ice grip only; climbed in the test with inputs only). Village after chapter 4: `quellen_befreit = 4` and celebration (Tüftel), new calls from Pip, Lotte and Tüftel, herbal tea at Lotte's; the world map shows the Frost Spring freed.

**Status M2.4.8:** Music (E-347): Frostspitzen “Ice Village” (KarateStudios, CC0), Kristella “Dramatic Boss Encounter” (cynicmusic, CC0). Sounds: thin ice cracks and breaks, icicle clinks and shatters, avalanche rumbles at the start, snow chunks and snowballs crunch (Kenney, CC0); bat squeaks, Elora freezes with a clink, Kristella's frost breath (procedural); fireplaces crackle in the ambient track, louder the closer you are (“Fireplace Sound loop”, PagDev, CC0).

**Playtest 2026-10-07** (all chapters in 29 min): victory screen confetti across the whole width (the random number for x only reached the middle); weather particles wrap around the view instead of spawning anew at the top (leaves came in bursts while running and jumping); cat sits on Pip's tree house; no more shot when entering a map (the first input after joining is only the starting point of click detection); thin ice grows back after 5.5 s; icicle 3 damage; **Kristella clearly harder**: 52 health, after 4 hits during one exhaustion straight back up, exhaustion 3.0 s, waves faster (7) and more frequent (2.1 s), enraged from 60 % with four icicles, blizzard from 30 %.

## Course of chapter 4 (proposal)

| # | Place | What happens |
|---|---|---|
| 1 | Tauwinkel | Oma sends Elora off; at the top of the upper village a **mountain path** leads north, so far blocked by an ice block – it breaks with **stomp** |
| 2 | `frost-1` glacier foot | First slopes and ice surfaces, snowball seals, icicles in a rock passage, thin ice over an ice water pool; cold bar introduced with the first fireplace |
| 3 | `frost-2` abandoned mountain village | **Flocke** waits in a hut; Elora fetches her rope back from a frozen cellar (bats, thin ice) and receives the **climbing claws (ice grip)**; right after that a chimney of climbing walls leads out. First climber, Klonk's task (ice crystals) |
| 4 | `frost-3` summit ridge | Blizzard sections with wind, avalanche slopes, climbing wall shafts, frost ghosts; grey tracks and a frozen patch where someone “drank colour”; spring stone in front of the hall |
| 5 | `frost-arena` ice hall | Frost Spring beneath a sheet of ice, Kristella hovers above it; fight |
| 6 | `frost-arena` | Victory: Kristella calms down – “He is lonely, not wicked.” Spring spark; she points east, to the Sternschlucht |
| 7 | Tauwinkel | Tüftel strengthens the ice grip with the spring spark; celebration; Oma announces the Sternschlucht |
| 8 | free | Rescue climbers, ice crystals for Klonk, climbing spots in chapters 1–3 |

## The fight against Kristella (proposal per E-341)

- **Arena:** ice hall, about two screens wide; climbing walls on the left and right and on two pillars;
  ice floor; the cold bar rests during the fight.
- **Phase 1:** Kristella hovers above and sends a **frost wave** across the floor from one side:
  rime creeps ahead as a warning, then the floor freezes – whoever stands on it takes damage. Elora
  jumps onto a climbing wall and holds on (ice grip) or jumps over the wave. After three
  waves Kristella is **exhausted**, sinks down and is briefly vulnerable (hammer, grenades, laser).
- **Phase 2 – from half health:** waves from both sides one after another; between the waves
  **icicles** fall from the ceiling (with trembling as a warning).
- **Phase 3 – last quarter:** blizzard in the hall (wind pushes Elora off the wall),
  shorter pauses, one more wave.

## Open for approval

| # | Question | Proposal |
|---|---|---|
| D-M24-01 | Way into the Frostspitzen | **Mountain path at the top of the upper village** leading north, blocked by an ice block that Elora breaks with stomp (uses the ability from chapter 3) |
| D-M24-02 | Climbing claws | Flocke gives them after Elora has fetched her **rope from the frozen cellar**; the cellar still works without ice grip |
| D-M24-03 | Spring spark | **Ice grip strengthened:** double cling time, and Elora can **pull herself up** the wall a bit (up key) instead of only sliding down |
| D-M24-04 | Cold bar | Fills outside in about 60 s (blizzard about 30 s), a fireplace warms fully in about 3 s, a roof in about 10 s; full = speed × 0.7 like the heat; equipment with cold protection (hat, cloak) slows the filling. Rests in arenas |
| D-M24-05 | Ice water | Small damage (1) and back to the edge, like fully sunken quicksand (E-318); thin ice grows back after about 4 s |
| D-M24-06 | Avalanches | Only in marked zones; triggered by stomp, grenade or a step on a trigger spot; 6–8 rolling snow chunks (2 damage, push), shelter in niches or on a climbing wall above |
| D-M24-07 | Frost ghost | Floats slowly through walls towards Elora; touch freezes her for 0.6 s (like the rush, but rigid); vulnerable to everything, visibly brighter outside the rock |
| D-M24-08 | Climbers | **Three climbers**, one each in `frost-1` to `frost-3`, reachable only with ice grip (the first one after going back); reward from Flocke: cold protection hat |
| D-M24-09 | Ice crystals | **Eight crystals** hidden (under thin ice, behind avalanche slopes, in climbing shafts); Klonk builds an upgrade of your choice from them |
| D-M24-10 | Weather | Glacier foot and mountain village: snow (dull: blizzard); summit ridge: blizzard sections fixed in the map; ice hall: fair (D-W1-01) |

## Technical specifications (proposal)

- New tiles **thin ice** and **ice water** in collision, map format and editor (like quicksand).
- **Icicles** as a map object with state (hanging, trembling, falling, shattering) in the
  simulation; **avalanches** as zones `lawine…` with a trigger and rolling bodies (deterministic).
- **Cold** shares the heat tech: one temperature bar per area (`hot` / new
  `cold` in `worldmap.toml`), zones `feuer…` warm, roof detection like the shade.
- Enemies and Kristella as new behaviours in `elora-sim` (deterministic), values in
  `creatures.toml`; the frost ghost ignores collision.
- Maps from a generator `editor/chapter4.rs`; run-through test extended to chapter 4.
- Music and sounds from free sources (CC0), presented for listening (E-295).

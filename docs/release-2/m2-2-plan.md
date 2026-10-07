# R2-M2.2 – Chapter 2: Murmelwald – Implementation Plan

Status: **Completed, accepted (E-314)** · Decisions E-306 to E-313 · Basis: [`world-book.md`](world-book.md) §4.2 and §5, [`m2-1-plan.md`](m2-1-plan.md), E-296, E-305

## Goal

Chapter 2 is playable from start to finish: from Tauwinkel into the Murmelwald, the old owl **Plumm**, **memory runes** that tell of a sixth spring, the lost **mushroom child**, the guardian **Root Warden** and the new ability **pull hook**. Afterwards Tauwinkel gets a bit more colourful (`quellen_befreit = 2`).

**Acceptance:** Play through chapter 2 once (about 60–90 minutes with side quests), saving and continuing in between.

## Starting point (from R2-M1 and R2-M2.1)

- Enemy behaviours: walker, hopper, shooter, flyer, guardian from the air (`diver`); guardian health bar, flag `besiegt.<art>`, events for sounds.
- The ability **pull hook** (`Pull`) already exists in the simulation (A1.2): the hook pulls small enemies to Elora. For chapter 2, pulling **items and switches** is still missing.
- Building blocks from chapter 1: map generators, hook flowers, jerk spots, decoration with state (`-verdorrt`/`-befreit`, `-fest`), characters with `show_if`, music per area, battle music.
- After chapter 1 the quest "Whispers in the Murmelwald" is running (announcement, manual step).

## Work steps

| # | Step | Content | Check |
|---|---|---|---|
| M2.2.0 ✅ | Drafts ([`design/kapitel2-entwuerfe.png`](design/kapitel2-entwuerfe.png), E-312) | Owl Plumm, mushroom child, root snake, squirrel pirate (with nut), mushroom imp, Root Warden (rest, root attack, core open, defeated), memory rune, core, Forest Spring (withered/freed), forest decoration (tall trees, tree houses, rope bridges, glowing mushrooms, roots, moss) | Your choice |
| M2.2.1 ✅ | New enemies | **Root snake**: hidden in the ground, shoots up when Elora is close, retreats (vulnerable only when out). **Squirrel pirate**: sits on branches, throws nuts in an arc. **Mushroom imp**: walks, on contact a colourful rush (Elora rainbow-coloured and slower, E-311); **companion** for the mushroom child (follows Elora, waits at difficult spots, E-308) | Tests + sandbox |
| M2.2.2 ✅ | Extend pull hook | The hook pulls **items** (chest contents, cores, collectibles) and **hook switches** (levers that can only be flipped with the pull hook) to Elora; new switch type "pull switch" in map and editor | Tests |
| M2.2.3 ✅ | Guardian technology | **Root Warden**: large guardian on the ground, blocks paths with roots (temporary solid tiles), strikes with roots from the ground; **cores** in its bark: hooking one and pulling away (tug-of-war) exposes a core, then vulnerable | Tests + sandbox |
| M2.2.4 ✅ | Content | Characters Plumm and mushroom child; main quest "Whispers in the Murmelwald" up to the spring spark; side quests **"Memory Runes"** (runes tell of the sixth spring in fragments) and **"The Lost Mushroom Child"**; conversations in the village after chapter 2 | Tests |
| M2.2.5 ✅ | Maps | Path from Tauwinkel into the forest; `wald-1` to `wald-3` and `wald-arena`: lots of verticality, tree houses, rope bridges, dark caves under roots, shortcuts, return spots for later abilities; jerk and hook spots | Visual check + playthrough test |
| M2.2.6 ✅ | Spring spark and pull hook | Victory → spring spark → Tüftel builds the **pull hook**; practice spot in Tüftel's yard; spots in chapter 1 that only work with the pull hook (returning pays off) | Tests |
| M2.2.7 ✅ | Village after chapter 2 | `quellen_befreit = 2` (more colour), conversations and call-outs, world map; possibly a small festival as after chapter 1 | Visual check |
| M2.2.8 ✅ | Music and sounds | Music for the forest and the guardian (presented for listening), sounds for snake, nuts, mushroom imp, roots, cores | Your listening test |
| M2.2.9 ✅ | Acceptance | Play through chapter 2, save, continue | Your acceptance |

**Status M2.2.1:** New behaviours in the simulation: `burrower` (root snake: hidden, grows out of the ground within 0.7 s from a distance of a good 6 tiles, vulnerable only when out, dangerous and hookable), arcing throw for shooters (`lob`, squirrel pirate), contact with `daze_ms` (mushroom imp: colourful rush for 5.5 s, Elora runs at A-23 = × 0.55, shimmers in rainbow colours, the world wobbles slightly), `follower` (mushroom child: follows Elora, jumps over steps, waits at gaps and thorns, invulnerable, harmless). Species in `creatures.toml` (`wurzelschlange`, `eichhornpirat`, `pilzwicht`, `pilzkind`), graphics from the drafts (E-312), plus Plumm and the mushroom child as characters. Try it: Training, F1 → "Enemies (Adventure)".

**Status M2.2.2:** With the **pull hook**, the hook grabs collectibles (bees, runes, glitter stones …) from a distance, loot flies straight to Elora, and hook switches ("pull switch", in the editor among the switch's triggers) now only react to this ability – once per hook shot. The Root Warden's cores follow with the guardian (M2.2.3).

**Status M2.2.3:** Behaviour `warden` (Root Warden, E-307): sleeps until Elora arrives; the ground quakes at Elora's position (warning 0.9 s), then roots shoot up (2 damage, upward knockback). **Cores:** keep the hook on the guardian and run away from it (0.8 s tug-of-war) – a core comes loose, the guardian is open for 3.5 s and vulnerable only then; the hook does not pull Elora towards it while doing so. Three cores, which grow back once all have been pulled. From half health faster and **root walls** (column of stone, 5 tiles high, 3.5 s) between Elora and it; at the last core roots at two spots. Graphics with poses per state (asleep, awake, attack, core pulled), root strikes and quakes as effects, health bar and battle music as with the first guardian. Try it: Training, F1 → "Enemies (Adventure)" → `wurzelwaechter`.

**Status M2.2.4:** Main quest "Whispers in the Murmelwald": west slope → Plumm → root caves → Forest Spring → calm the Root Warden → spring spark to Tüftel (**pull hook**, `quellen_befreit = 2`, festival, pull switch in the yard `hof.zug`) → festival at Oma's; afterwards "Tracks in the Sand" (announcement of chapter 3). Side quests **"Memory Runes"** (5 runes, Plumm reads fragments aloud; all five together tell the story of the sixth spring beneath the oldest well, flag `sechste_quelle`; reward owl feather with a larger loot magnet and one dewdrop point, E-309) and **"The Lost Mushroom Child"** (Krümel follows Elora across map changes to the mushroom ring, where Mama Morel waits). **Companions** as data in `characters.toml` (`follower`, `follow_if`, `home_zone`). Characters Plumm, mushroom child (lost/at home), Mama Morel, Root Warden after the fight (spring blooms, flag `befreit.waldquelle`); signs west slope, forest, mushroom ring; conversations and call-outs in the village. Tests in `tests/story.rs` and `tests/session.rs`.

**Status M2.2.5:** Maps from `apps/elora-client/src/editor/kapitel2.rs` (rewrite: `write_kapitel2_maps -- --ignored`, overview `kapitel2_sheets`), from right (Tauwinkel) to left (deeper into the forest): **west slope** in Tauwinkel (earth steps and roots for hooking, the transition at the top); `wald-1` (220 × 60: mushroom ring with Mama Morel at the forest edge, gentle hills without gaps for the mushroom child, walkways in the treetops, rune 1 on a branch, rune 2 on a floating stone pillar – only with the pull hook); `wald-2` (200 × 90: tree house settlement with zigzag walkways, rope bridge, Plumm in the treetops, mushroom child on the ground, rune 3 high up); `wald-3` (200 × 70: root caves with thorns, crumbling floor, hanging roots, pull switch chamber with rune 5 – only with the pull hook, a jerk spot with rune 4 at the top); `wald-arena` (100 × 50: Root Warden on the Forest Spring, hook flowers for dodging, gate after the victory, root path back to Tauwinkel). Forest look from the release theme "Forest" with a shadier sky; decoration forest tree, tree house, rope bridge, glowing mushrooms, root arch, mushroom ring, Forest Spring.

**Status M2.2.6:** Victory → spring spark → Tüftel builds the **pull hook** (M2.2.4). Building block **pull vault** (`pull_vault` in `editor/prolog.rs`): stone hut with a gate and a chest, 10 tiles above it a root with a pull switch. Practice in Tüftel's yard (with sign "schild-zug"), return rewards in `wiese-1` and `wiese-2`; in the forest additionally rune 2 (stone pillar) and rune 5 (chamber in `wald-3`). The pull hook checks the entire path of the hook tip. Test: all pull switches with a real hook shot (`tests/prolog.rs`).

**Status M2.2.7:** After the pull hook: `quellen_befreit = 2` (more flower beds, flower boxes, flower pots and flags colourful), festival with garlands, lanterns and festival music until Elora leaves the village, Oma at the festival (sixth spring, grey wanderer, announcement Glutsandwüste), Lotte gives away mushroom soup (new: heals 8), Klonk talks about resin for the weapon upgrade, new call-outs from Pip and Lotte. World map: Murmelwald with a halo "Spring freed" (Tüftel also sets `befreit.<quelle>` as a safeguard, in case Elora does not talk to the guardian after the fight).

**Status M2.2.8:** Music (E-313): Murmelwald "Woodland Fantasy" (Matthew Pablo, CC BY 3.0), Root Warden "Bamboo Blitz" (Tsorthan Grove, CC0) – battle music now per area (`boss_music` in `worldmap.toml`). Sounds: the root snake creaks when surfacing and submerging, the ground rumbles before the root strike, wood cracks on the strike, a pop when a core comes loose (Kenney CC0 and procedural), shimmering glitter when the colourful rush begins.

## Flow of chapter 2 (proposal)

| # | Location | What happens |
|---|---|---|
| 1 | Tauwinkel | Oma sends Elora off; a new path leads out of the village into the forest |
| 2 | `wald-1` | Forest edge, first root snakes and mushroom imps; first memory rune |
| 3 | `wald-2` | Tree house settlement in the treetops: **owl Plumm** collects stories, the **mushroom child** is crying, it has got lost; squirrel pirates on the branches |
| 4 | `wald-3` | Root caves, the path is blocked by roots; spring stone before the arena; more runes |
| 5 | `wald-arena` | Forest Spring, the **Root Warden** sleeps right on top of it; fight |
| 6 | `wald-arena` | Victory: the guardian was tired and confused (like the bumblebee); spring spark; the runes form a sentence about the **sixth spring** |
| 7 | Tauwinkel | Tüftel builds the **pull hook**; festival; Oma announces the Glutsandwüste |
| 8 | free | Bring the mushroom child home, bring all runes to Plumm, return to chapter 1 |

## The fight against the Root Warden (proposal)

- **Phase 1:** It stands in the middle and strikes with roots from the ground (warning: the ground quakes, earth crumbles at the spot). Three **cores** glow in its bark. Elora hooks a core and pulls (hold the hook and run or jump away): the core comes loose, the guardian is briefly open.
- **Phase 2 – from half health:** It closes off parts of the arena with root walls; Elora has to dodge via hook spots.
- **Phase 3 – last core:** faster, roots at two spots at once.
- Hits only while a core is loose.

## Decisions on R2-M2.2

| # | Question | Decision |
|---|---|---|
| D-M22-01 | Path into the Murmelwald | **Path up the west slope** of Tauwinkel (steps, hook spots), the forest begins at the top (E-306) |
| D-M22-02 | Fight against the Root Warden | **As proposed** (E-307) |
| D-M22-03 | Bringing the mushroom child home | **It follows Elora** (new companion technology, waits at difficult spots) (E-308) |
| D-M22-04 | Reward for the runes | **A piece of equipment and one dewdrop point**, plus the story of the sixth spring (E-309) |
| D-M22-05 | Festival after chapter 2 | **As after chapter 1** (E-310) |
| D-M22-06 | Mushroom imp effect | **Colourful rush:** Elora shimmers in rainbow colours for a few seconds and runs slower, the world wobbles slightly (child-friendly, no reference to drugs in the game) (E-311) |

## Technical decisions (proposal)

- New behaviours and the guardian in the simulation (deterministic), values as data in `creatures.toml`.
- **Temporary root walls** as tiles that the simulation sets and removes again (like crumbling floor, but back to the original); event for graphics and sound.
- **Pulling items with the hook**: loot and collectibles as hookable targets; pull switch as an adventure object.
- Maps from a generator `editor/kapitel2.rs`; playthrough test extended with chapter 2.

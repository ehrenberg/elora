# R2-M2.5 – Chapter 5: Sternschlucht – implementation plan

Status: **in progress** (M2.5.0 to M2.5.5 done) · Decisions E-362 to E-369 · Basis: [`world-book.md`](world-book.md) §4.5 and §5, [`m2-4-plan.md`](m2-4-plan.md), E-228, E-354

## Goal

Chapter 5 is playable from start to finish: from the far east of the Blütenwiesen down into the
night-dark **Sternschlucht** with glowing mushrooms, floating star islands, updrafts and the ruins
of the vanished Star Spring. Elora meets **Luma the star weaver**, who weaves her a star cape
(= **gliding**) in the middle of the chapter. At the end the **Withered One** appears for the
first time: he drains the last colour of the Star Spring while Luma sings, and Elora protects
her until the song is finished. He flees towards Tauwinkel; a small star spark remains. Tüftel
strengthens gliding, Tauwinkel becomes almost fully colourful (`springs_freed = 5`) – only the old
well stays grey and points to the finale.

**Acceptance:** play through chapter 5 once (about 60–90 minutes with side quests), saving and
continuing in between.

## Core decisions

| # | Question | Decision |
|---|---|---|
| E-362 | Climax | **Protect Luma:** the Withered One drains the Star Spring in rounds; his grey shadows move towards Luma, Elora holds them off (light star lanterns, drive shadows away, keep the note stones lit) until Luma's song is finished. Then he flees towards Tauwinkel |
| E-363 | Gliding | **From Luma in the middle of the chapter** (star cape, like the climbing claws from Flocke); the second half and the climax use gliding; the star spark after the chapter **strengthens** gliding at Tüftel's |
| E-364 | Signature mechanics | **Darkness and light**, **floating star islands**, **star bridges from note stones**, **updrafts** |
| E-365 | Enemies | **Shadow moths, stone goblins, echoes** (world book) |
| E-366 | End of the chapter | **A spark is saved:** star spark → Tüftel strengthens gliding, `springs_freed = 5`; the village is colourful except the old well |
| E-367 | Way there | **East of the Blütenwiesen:** a gorge entrance at the far east of the meadows opens after chapter 4; the grey trail of the Withered One leads there |
| E-368 | Side quests | **“Star fragments”** and **“Luma's song”** (world book) |

## Starting point (from R2-M1 to R2-M2.4 and R2-RF)

- **Gliding exists in the simulation** (`Ability::Glide`, A1.1): hold jump in the air, fall
  slowly (`glide_fall_speed`) and steer (`glide_control_speed`); skill tree node “Wide glide”
  (shown as “???” until Elora has gliding, E-354).
- **Temporary tiles** (`temp_tiles`, crumbling floor, thin ice, root walls) – the base for star
  bridges that appear for a short time.
- **Zones** with prefixes (`shade…`, `oasis…`, `fire…`, `avalanche…`) and the temperature bar
  (RF-23) – template for updraft zones and light sources.
- Enemy building blocks (RF-20: one module per behaviour): walker, hopper, turret, flyer,
  diver, burrower, follower, leaper, serpent, warden, queen, icicle, roller, seal, bat, ghost.
- Weather and post shader (tint, fog, darkening) – the base for the night look; **there is no
  light system yet** and **no moving platforms**.
- Content: quest `sternschlucht` (placeholder step “soon”), flags `withered_one.*` from chapters
  1–4, area `sternschlucht` in `worldmap.toml`, Kristella sends Elora east.

## Work steps

| # | Step | Content | Check |
|---|---|---|---|
| M2.5.0 ✅ | Drafts | Luma (weaving, weak, singing, relieved), the Withered One (first appearance: grey, thin, sad – draining, fleeing), shadow moth, stone goblin (rock/awake/rolling), echo (Elora silhouette, translucent), star cape, star islands, note stones (dark/lit), star bridge, star lantern, updraft, Star Spring ruins (dark/one spark), gorge decoration: night rocks, crystal stars, glowing mushrooms, ruin arches | Your selection |
| M2.5.1 ✅ | Darkness and light | Night maps get a **darkness layer** (Sternschlucht `darkness = 0.9`, single maps may override up to 1.0 via `map_darkness`) in the renderer: a soft light circle around Elora plus light sources (glow mushrooms, star lanterns, lit note stones, star bridges); light strength as map/area setting; the simulation knows **light zones** so shadow enemies can flee from light. Setting “gentle” for players who find it too dark | Tests + visual check |
| M2.5.2 ✅ | Star islands and updrafts | **Moving platforms** in the simulation (deterministic): islands drift on a path, some sink slowly under Elora and rise again; they carry characters and creatures. **Updraft zones** lift gliding figures (and push non-gliding ones a little). Built as zones `island…` (+ `<id>-to`, `sink` in the id) and `updraft…`, values A-43 to A-49 | Tests + sandbox |
| M2.5.3 ✅ | Note stones and star bridges | Touching a note stone plays a note and lights a star bridge (visible light) for a few seconds; note stones in the right order are part of “Luma's song”. Built as zones `note-<pitch>…` and `<stone>-bridge…`; the bridges carry like fixed star islands while lit (no new tile kind); A-50; placeholder bell sound until M2.5.9 | Tests |
| M2.5.4 ✅ | New enemies | **Shadow moth**: flies in small swarms, avoids light, attacks in the dark. **Stone goblin**: looks like a rock until Elora is close, then rolls towards her; stomp or hammer stops it. **Echo**: appears from a dark mirror, repeats Elora's movements with a delay; touch hurts, light makes it fade | Tests + sandbox |
| M2.5.5 ✅ | Climax tech | **“Protect Luma”** (see below): rounds of grey shadows towards Luma, star lanterns to light, song progress, failure → back to the spring stone; the Withered One as a scripted figure (appears, drains, flees) | Tests + sandbox |
| M2.5.6 | Content | Character Luma; main quest from the gorge entrance up to the star spark with the star cape in the middle; side quests **“Star fragments”** and **“Luma's song”**; dialogues of the village after chapter 5; the finale is announced (the well) | Tests |
| M2.5.7 | Maps | Gorge entrance at the east end of the meadows (opens after chapter 4); `gorge-1` to `gorge-3` and `gorge-spring` (climax at the ruins) | Visual check + run-through test |
| M2.5.8 | Spark and village | Victory → star spark → Tüftel strengthens gliding; gliding spots in chapters 1–4 (going back pays off); `springs_freed = 5`, celebration, world map; the well stays grey | Tests + visual check |
| M2.5.9 | Music and sounds | Music for the Sternschlucht and the climax (CC0, presented for listening), sounds for moths, goblins, echoes, note stones, star bridges, updrafts, star cape, the Withered One | Your listening test |
| M2.5.10 | Acceptance | Play through chapter 5, save, continue | Your acceptance |

## Course of chapter 5 (proposal)

1. **Gorge entrance** (east end of `meadow-3`): the grey trail of the Withered One ends at a
   crack in the rocks; it opens after chapter 4. A signpost (E-354) says “closed” before.
2. **`gorge-1` – Descent at dusk:** first glowing mushrooms, it gets dark; stone goblins, a few
   shadow moths; first star islands (fixed, then drifting); note stones teach star bridges.
3. **`gorge-2` – Luma's hideout:** a hollow full of threads of light; Luma is weak, the Star
   Spring has almost faded. She weaves Elora the **star cape** (gliding). After that: updrafts,
   gliding across the gorge, sinking islands, echoes in a mirror cave.
4. **`gorge-3` – The dark heights:** long gliding routes between islands and updrafts, star
   fragments in places only reachable by gliding, the last note stones of Luma's song.
5. **`gorge-spring` – The ruins of the Star Spring:** Luma starts her song to keep the last
   light; the Withered One appears – **“Protect Luma”**. When the song is finished he flees with
   most of the colour towards Tauwinkel; a small **star spark** remains.
6. **Home:** Tüftel strengthens gliding, celebration in Tauwinkel, almost everything colourful –
   only the old well stays grey. Oma remembers a few lines of a forgotten song (teaser of the
   finale).

## “Protect Luma” (proposal per E-362)

- **Arena:** the ruins in the middle, Luma on the spring stone; around her three to four star
  lanterns on islands and pillars (some only reachable by gliding or updraft).
- **Rounds:** the Withered One stands at the edge and drains; grey shadows (slow, many) drift
  from the dark towards Luma. Lit star lanterns make a circle of light that dissolves shadows;
  lanterns go out after a while (or when a shadow reaches them) and Elora lights them again by
  touching them. Hammer and grenades push shadows back; the laser does not hurt them.
- **Song progress:** a bar above Luma fills while no shadow touches her; a shadow on Luma stops
  the song for a moment and dims the arena. Three verses (rounds) with more and faster shadows;
  between the verses the Withered One moves to another place and the islands drift differently
  (built in M2.5.5 without the island change: the islands keep their paths, the map in M2.5.7
  places lanterns so that the routes change with the Withered One's place).
- **Side quest bonus:** with Luma's song complete (all note stones found) Luma sings stronger:
  the bar fills faster.
- **End:** after the third verse the Withered One stops, looks at Elora – no fight – and flees
  towards Tauwinkel; Luma: “He is not taking the colour for himself. He is looking for something
  he lost.” Failure (Luma loses all light): back to the spring stone, no loss.

## Open for approval

- The steps and order above, the course of the chapter and the climax rules.
- Darkness: how dark the maps are by default (proposal: clearly night, Elora always sees about
  six tiles around her; the graphics setting “gentle” brightens it).
- Map names `gorge-1` … `gorge-3`, `gorge-spring` (English ids, E-358).

## Technical specifications (proposal)

- **Light:** render a light mask (soft circles) into an intermediate texture and multiply it
  over the world in the post pass; light sources come from decoration names/objects and from
  Elora; the simulation gets light zones as data (no render dependency) for enemy behaviour.
- **Moving platforms:** new entity kind in `elora-sim` with a path (points, speed) or a sink
  behaviour; collision against characters as one-way or solid boxes; deterministic (no float
  drift: positions from tick count); the protocol does not change for the adventure (not
  networked, like other adventure-only state).
- **Echo:** records Elora's positions in a ring buffer and replays them with a delay of about
  1.5 s.
- **Climax:** session logic in `session/` (like avalanches and temperature), the Withered One as
  a scripted NPC with states (appear, drain, move, flee).

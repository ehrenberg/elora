# R2-W1 – Weather – Implementation Plan

Status: **Accepted** (E-339) · Decisions E-329 to E-339 · Basis: O-209 (times of day stay open, E-332), E-320 (heat), E-328 (post shader, colour of the springs)

## Goal

Weather makes the areas more alive and helps tell the story: as long as a spring is silent, its area
is gloomy; after it is freed the sky clears, and the weather changes on entering. In the adventure it
has a light gameplay effect (wind, wetness, fog); in multiplayer it is pure atmosphere. Maps get
weather as a property in the editor.

**Acceptance:** see and hear every weather once (weather test map), play Tauwinkel and chapters 1–3
with weather, one multiplayer map with weather online.

## Starting point

| Building block | Status | Use |
|---|---|---|
| Post shader | heat shimmer, saturation (E-320, E-328) | colour mood, darkening, fog, lightning |
| Particles | dust, sparks, spores (`effects.rs`) | template for weather particles (own layer, because there are many) |
| Sky, backgrounds | gradient and tint per map | gloomy sky, denser clouds |
| Decoration animations | wind in trees, flags | stronger wind |
| Music | streamed per area (kira) | template for a second, continuous ambience track |
| Map format `.emap` | sections; unknown ones are skipped | new section `WTHR` stays compatible with 0.9.1 |
| Simulation | deterministic, the same in multiplayer | gameplay effect only when `world.adventure` (E-330) |

## Weather types (E-333)

| Weather | Visuals | Sound | Effect in the adventure (E-330) |
|---|---|---|---|
| **Clear** | as before | – | – |
| **Rain** (drizzle to heavy) | slanted drops following the wind, splashes on surfaces, greyer sky, image slightly darkened | rain | wet ground: braking a bit softer (one third of the ice effect) |
| **Thunderstorm** | like rain, darker; lightning briefly brightens the whole image, cloud glow | rain, thunder (delayed to match the lightning) | like rain, plus gusty wind; **lightning strikes** with a warning on the ground, small damage (E-336) |
| **Fog** | veil over the world, denser towards the bottom; distant backgrounds disappear | soft wind | visibility: the camera looks less far ahead, enemies emerge from the veil |
| **Wind** (with leaves or blossoms) | leaves or petals swirl, trees and flags sway more | wind | wind pushes Elora in the air and grenades (barely on the ground) |
| **Sandstorm** | horizontal sand veil, yellow-brown image, shorter visibility | storm, trickling sand | wind as above, stronger; shorter visibility like fog |
| **Snow** | calm flakes, light-grey sky | very quiet | ground somewhat slippery (like rain) |
| **Blizzard** | dense slanted flakes, short visibility | storm | strong wind, slippery ground, shorter visibility |

Intensity per weather 0–1 (drizzle to downpour); wind with direction and gusts. The desert heat
(E-320) stays: during a sandstorm the sun does not fill the heat bar (sand covers the sun).

## Weather per area (proposal, E-331)

| Area | while the spring is silent | afterwards, random on entering |
|---|---|---|
| Tauwinkel | drizzle (only rarely after the 1st spring) | clear (mostly), wind with blossoms, summer rain |
| Blütenwiesen | rain, now and then thunderstorms | clear, wind with blossoms, summer rain |
| Murmelwald | fog, rain | clear, morning fog, wind with leaves |
| Glutsandwüste | sandstorm, heat thunderstorm | clear (hot), rarely sandstorm |
| Frostspitzen (chapter 4) | blizzard | snow, clear |
| Guardian arenas | fixed weather per arena (e.g. thunderstorm over the Blossom Spring?) – or clear, so the fight stays readable | clear |

Weights and intensities are stored as data in `assets/adventure/worldmap.toml`; the weather is rolled
when entering a map and applies until the map changes. A map with its own weather from the editor
takes precedence over the area rules (e.g. caves: no rain).

## Multiplayer maps (E-329)

Weather is a map property (type, intensity, wind) and is transferred with the map; each client
renders it itself. No gameplay effect, no new protocol. Proposal for the release maps:
`dm-winter` snow, `ctf-nacht` light fog, `dm-wueste` wind with sand, the rest clear – to be decided
at approval.

## Work steps

| # | Step | Content | Check |
|---|---|---|---|
| W1.0 ✅ | Drafts | Mood images: the same scene in all eight weathers (Python script), plus particle shapes (drop, flake, leaf, blossom, sand grain) | Your choice |
| W1.1 ✅ | Data model | weather type (kind, intensity, wind, gusts) in `elora-map`; section `WTHR` in the map format; property in the editor with preview; weather test map | Tests (format round trip, old maps unchanged) |
| W1.2 ✅ | Visuals | weather particles in two layers (behind and in front of the play area), splashes and flakes on surfaces, lightning; post shader: colour mood, darkening, height fog, sand veil; gloomier sky; wind makes decoration sway more; setting "Weather: full, gentle, off" (E-335) | Visual check, frame rate in full rain |
| W1.3 ✅ | Adventure control | area rules in `worldmap.toml` (gloomy while the spring is silent, otherwise weighted random), roll on entering, smooth transitions; sandstorm covers the sun (heat) | Tests |
| W1.4 ✅ | Gameplay effect | in the simulation only with `world.adventure`: wind force on Elora in the air and on grenades, wet/snowy ground (softer friction), lightning with warning and damage in thunderstorms (E-336), camera visibility in fog/storm; values as tuning (A-29 to A-35) | Tests; golden tests unchanged |
| W1.5 ✅ | Sound | second ambience track with smooth crossfading; rain, wind, storm, sand, thunder from free sources (CC0, presented for listening, E-334, selection E-338) | Your listening test |
| W1.6 ✅ | Maps | Release maps: `dm-winter` snow (0.6), `ctf-nacht` light fog (0.35), `dm-wueste` wind with sand (0.3), the rest clear; guardian arenas stay clear (D-W1-01); no precipitation in caves and under roofs, weather sounds muffled | Visual check |
| W1.7 ✅ | Acceptance | Weather test map, chapters 1–3, one multiplayer round | Your acceptance |

## Acceptance checklist (W1.7)

Switching the weather: debug panel (F1) → "Weather" (type, intensity, wind); in the adventure this also
affects wind, wetness and lightning.

| # | Where | What to check |
|---|---|---|
| 1 | `elora maps/wetter-test.emap` | all nine weathers in turn: particles, colour mood, fog, lightning; no precipitation under the roof and in the pit; sound quieter under the roof |
| 2 | same map | setting Graphics → Weather "full / gentle / off"; frame rate in full rain and blizzard (debug panel) |
| 3 | Chapter 1 (Blütenwiesen) | weather on entering changes over time; thunderstorm: ground glows, lightning strikes and deals damage, thunder; dodging works |
| 4 | Chapter 2 (Murmelwald) | fog: camera looks less far ahead, game stays readable; leaf wind: Elora drifts noticeably but controllably when jumping over pits |
| 5 | Chapter 3 (Glutsandwüste) | sandstorm: heat bar does not fill up (shade), grenades drift with the wind; arena stays clear |
| 6 | Rain/snow in the adventure | wet ground: softer braking, not unfair at edges |
| 7 | Multiplayer | `dm-winter`, `ctf-nacht`, `dm-wueste` with a server: weather visible and audible, no gameplay effect |

## Technical decisions (proposal)

- **Particles as their own layer** (not in the general effects system): fixed upper limit (full about
  600, gentle 200), anchored to the camera view and with slight parallax so that rain does not
  "move along". Drawn as lines (rain, sand) or small shapes (flakes, leaves).
- **Shader:** a second parameter set for colour mood (tint, brightness), fog (colour,
  density, height gradient) and lightning; the same pass as heat shimmer and saturation.
- **Gameplay effect deterministic** in `elora-sim`: the world's weather is set by the session
  (like heat), gusts from a fixed pseudo-random sequence by tick – recordings stay identical.
- **Visibility** is presentation (camera and fog), not simulation.
- **Multiplayer:** only the map section; protocol version stays 6.
- **Performance:** particles as one mesh per frame; with "gentle" fewer particles and no lightning.

## Open for approval

| # | Question | Proposal |
|---|---|---|
| D-W1-01 | Weather in the guardian arenas | clear, so fights stay readable; at most light wind |
| D-W1-02 | Release maps with weather | `dm-winter` snow, `ctf-nacht` light fog, `dm-wueste` wind with sand |
| D-W1-03 | Strength of the gameplay effect | Wind: Elora in the air up to about 1/6 of the air control, grenades noticeable; wetness: one third of the ice effect |
| D-W1-04 | Lightning | **E-336: lightning can deal damage** – only in the adventure: the ground briefly glows at the impact point (warning), then the lightning strikes (small damage in a radius); in multiplayer only visuals and sound |

# Adventure intro video – plan

Status: **accepted** (E-357) · Decisions: E-355 (real video, skippable, exception to E-295), E-356 (AV1 via `rav1d`, `unsafe` only in `elora-video`)

## Goal

When a player starts a **new adventure**, a short intro video (about 60 s) tells how the springs
of the Tauland fall silent and Elora sets out. It can be skipped at any time with Esc, Space,
Enter or a click. Afterwards the game starts in Tauwinkel as today.

## How it works

- **The video has no text and no voice.** Narration appears as subtitles drawn by the game from
  `assets/lang` (de/en), so one video serves both languages and the font matches the game.
- **Sound:** the video file carries no audio track; the game plays the existing `tauwinkel` track and fades it into the map music.
- **Format:** AV1 in an IVF container (`assets/intro/intro.ivf`), 1280×720, 24 fps, about
  8–15 MB. Subtitle timing lives in `assets/intro/intro.toml`.
- **Who makes what:** the owner creates the video with an external AI tool from the storyboard
  below (the tool's terms must allow redistribution under the game's licences) and hands over
  any common video file. `cargo xtask intro-import <file>` converts it with the local ffmpeg.
- Without `intro.ivf` the game simply starts without an intro.

## Tasks

| # | Task | Notes |
|---|---|---|
| I-1 ✅ | New crate `crates/elora-video`: IVF reader, AV1 decoding with `rav1d`, YUV 4:2:0 → RGBA | The only crate with `#![allow(unsafe_code)]` (E-356); small, documented, unit tests with a tiny test clip (~50 KB) made by ffmpeg; `asm` feature off (no nasm needed for cross builds) unless decoding is too slow |
| I-2 ✅ | Renderer: full-screen textured quad, the texture updated per frame | First image texture in `elora-render` (everything else is meshes) |
| I-3 ✅ | Client: intro screen before the first map of a new adventure; skip with Esc/Space/Enter/click (not in the first 0.5 s); subtitles with fade; decoding on a background thread, a few frames buffered; music start and fade-out | Also “Watch intro” in the adventure menu (pause → adventure tab) |
| I-4 ✅ | `cargo xtask intro-import <video> [--crf N]`: ffmpeg → 1280×720, 24 fps, AV1 (SVT-AV1), no audio, writes `assets/intro/intro.ivf`, prints size and duration | Checks that ffmpeg with an AV1 encoder is installed |
| I-5 ✅ | Packaging: ship `assets/intro/`; `SOURCES.md` and the About page name the tool and the licence of the video (E-355) | |
| I-6 ✅ | Placeholder until the real video exists: a short clip from the existing chapter drafts (design tools) so I-3 can be tested and played | Replaced by the real video |
| I-7 ✅ | Tests: decoder (frame count, size, first pixel colours of the test clip), intro timing (skip, end, subtitle at time t), the game starts without `intro.ivf` | `cargo xtask check` green |

## Storyboard (for the video tool) – please review

Style for every shot: *flat 2D vector illustration, soft pastel colours, thick dark outlines,
rounded shapes, children's picture-book style, no text, no people, gentle slow camera movement,
16:9.* Characters are round drop creatures with big eyes, no arms (see `assets/elora/elora.svg` and the
chapter drafts in `docs/release-2/design/` as reference images).

| Shot | Time | Picture (prompt idea) | Subtitle de | Subtitle en |
|---|---|---|---|---|
| 1 | 0–9 s | Wide view over the Tauland at dawn: hills, a forest, a desert far away, snowy peaks; five springs sparkle in five colours (green, brown, gold, ice blue, violet) | Im Tauland singen fünf Quellen. Ihr Tau schenkt allem Farbe, Mut und Leben. | In the Tauland, five springs sing. Their dew gives everything colour, courage and life. |
| 2 | 9–18 s | The small village Tauwinkel in a hollow, colourful houses around a well with a wooden lid; drop creatures go about their day | In Tauwinkel, mitten im Tauland, leben die Tropfen – fröhlich und bunt. | In Tauwinkel, in the middle of the Tauland, the drops live – cheerful and colourful. |
| 3 | 18–30 s | One by one the springs stop sparkling; a grey shimmer creeps over the land; the colours of the village fade to grey | Doch eines Morgens verstummt die erste Quelle. Dann die nächste. Und die Farben verblassen. | But one morning the first spring falls silent. Then the next. And the colours fade. |
| 4 | 30–40 s | Inside a cosy house: an old grandmother drop with a knitted hat looks worried out of the window at the grey village | „Als ich so klein war wie du, sangen die Quellen noch Morgenlieder …“ | “When I was as small as you, the springs still sang morning songs …” |
| 5 | 40–50 s | A small, brave yellow drop (Elora) stands on the well square, looks up at the hills, a hook on a line in her hand | Elora will nicht warten, bis alles grau ist. | Elora does not want to wait until everything is grey. |
| 6 | 50–60 s | Elora swings with her hook over a stream towards the green hills; sunrise; the camera follows her and stops on the horizon | Sie macht sich auf, die Quellen wieder zum Singen zu bringen. | She sets out to make the springs sing again. |

The secret of the sixth spring and the Withered One are deliberately not shown.

## Making the video with Veo (two clips of 30 s)

Veo creates up to 30 s per clip here, so the intro is **two clips**: clip 1 = shots 1–3
(0–30 s), clip 2 = shots 4–6 (30–60 s). The subtitle times in `assets/intro/intro.toml`
already follow this split. Use **Frames to Video** with the start and end frame; add the
reference sheets as ingredients/reference images where the tool allows it.

Images (in `docs/release-2/design/intro/`, made from the game's assets with
`tools/design/intro_keyframes.py`):

| File | Use |
|---|---|
| `clip1-start.png` | first frame of clip 1: the colourful Tauland at dawn, five glowing springs, Tauwinkel |
| `clip1-end.png` | last frame of clip 1: the same view, grey, the springs dry |
| `clip2-start.png` | first frame of clip 2: the grey well square, Oma Pfütze and Elora |
| `clip2-end.png` | last frame of clip 2: Elora on a green hill at sunrise |
| `ref-elora.png`, `ref-oma.png`, `ref-tauwinkel.png` | reference images for characters and style |

### Prompt clip 1 (30 s)

```text
Animated children's picture-book film in flat 2D vector style: soft pastel colours, thick
dark outlines, rounded simple shapes, exactly the style of the start and end frames. 16:9,
30 seconds, slow and gentle camera, calm and slightly melancholic mood.

0–10 s: Dawn over a peaceful fantasy valley. The camera drifts slowly from left to right over
snowy mountains, a green forest and soft hills. Five magical springs glow and sparkle in five
colours – green, brown-gold, golden orange, ice blue and violet – tiny light particles rise
from them like a quiet song. Birds glide past.
10–20 s: The camera gently moves down to the small village in the hollow: cosy colourful
cottages, a smithy with smoke from the chimney, trees and an old wooden well with a lid in
the middle. Tiny round drop creatures without arms hop happily between the houses.
20–30 s: One after another the springs stop sparkling, their light fades out. A grey shimmer
slowly creeps over the hills, the forest and the village; colours drain away until everything
is grey and quiet, as in the end frame. The drop creatures stop and look up worried.

No text, no letters, no logos, no subtitles, no speech, no humans. Characters keep their
simple shapes: round drop bodies with big eyes and small feet, no arms, no hands, no mouths
talking. Keep the art style consistent from the first to the last frame.
```

### Prompt clip 2 (30 s)

```text
Animated children's picture-book film in flat 2D vector style: soft pastel colours, thick
dark outlines, rounded simple shapes, exactly the style of the start and end frames and the
reference images. 16:9, 30 seconds, gentle camera, the mood turns from sad to hopeful.

0–10 s: A grey, colourless village square with a stone cottage, an old wooden well and a
workshop. Next to the well stands an old grandmother drop in lilac with a knitted bobble hat,
round glasses, a red scarf and a walking stick. She looks sadly at the grey houses and sways
slightly as if remembering old songs. Beside her stands Elora, a small brave yellow drop with
big eyes and orange feet (see the reference image), listening.
10–20 s: Elora turns towards the hills, takes a deep breath and gives a determined little
hop. Close-up on her face: she is brave and ready. The grandmother nods softly.
20–30 s: Elora hops out of the village; a small grappling hook on a thin chain shoots from
her body, catches a branch and she swings in a big arc over a little stream towards the green
hills. The camera follows her. The sun rises, warm light and colour return to the hills and
flowers around her. She lands on a hilltop and looks into the distance, as in the end frame.

No text, no letters, no logos, no subtitles, no speech, no humans. Characters keep their
exact shapes and colours from the reference images: round drop bodies, big eyes, small feet,
no arms, no hands, no talking mouths. Keep the art style consistent throughout.
```

### Afterwards

- Download both clips (any format) and run
  `cargo xtask intro-import clip1.mp4 clip2.mp4` – it joins them in order, drops the sound
  and checks the result with the game's decoder.
- If the clips are not exactly 30 s, tell Claude the lengths; the subtitle times in
  `assets/intro/intro.toml` will be adjusted.
- Check Google's terms for the generated video (use in a GPL/CC-BY-SA game, visible
  watermark/SynthID) and note the tool in `assets/SOURCES.md`.

## Decided (E-357)

- Intro music: the existing `tauwinkel` track.
- The intro plays at every new adventure, always skippable.

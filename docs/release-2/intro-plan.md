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

## Making the video with Veo (four shots of 8 s)

Veo in Google AI Studio makes 8-second clips, so the intro is **four shots of 8 s (≈ 32 s)**,
joined by `cargo xtask intro-import`. Each shot starts from a **start image** (image to video),
so style and characters stay the same across the cuts. The subtitles in
`assets/intro/intro.toml` follow these four shots (one line per shot).

Images in `docs/release-2/design/intro/` (made from the game's assets with
`tools/design/intro_keyframes.py`):

| Shot | Start image | Content | Subtitle |
|---|---|---|---|
| 1 (0–8 s) | `shot1-start.png` | the colourful Tauland at dawn, five glowing springs | „Im Tauland singen fünf Quellen …“ |
| 2 (8–16 s) | `shot2-start.png` | the colourful well square, Oma Pfütze and Elora, drops hop around | „In Tauwinkel … fröhlich und bunt.“ |
| 3 (16–24 s) | last frame of shot 2 (see below) | the springs fall silent, the colour drains from the square | „Doch eines Morgens verstummt …“ |
| 4 (24–32 s) | `shot4-start.png` | grey square; Elora decides and swings off with her hook into the sunrise (`shot4-end.png` shows the goal) | „Elora will nicht warten …“ |

`ref-elora.png`, `ref-oma.png`, `ref-tauwinkel.png`: reference images, if the tool accepts them.

**Last frame of shot 2 as the start image of shot 3** (the square turns grey without a jump):

```sh
ffmpeg -sseof -0.05 -i shot2.mp4 -frames:v 1 shot3-start.png
```

Every prompt starts with the same **style block** so the four shots match:

```text
Animated children's picture-book film in flat 2D vector style: soft pastel colours, thick
dark outlines, rounded simple shapes, exactly the style of the start image. 16:9, 8 seconds,
slow gentle camera. No text, no letters, no logos, no subtitles, no speech, no humans.
Characters keep their exact shapes and colours: round drop bodies with big eyes and small
feet, no arms, no hands, no talking mouths.
```

**Shot 1** (start `shot1-start.png`): style block +

```text
Dawn over a peaceful fantasy valley. The camera drifts slowly from left to right over the
snowy mountains, the forest and the soft hills. The five magical springs glow and sparkle in
green, brown-gold, golden orange, ice blue and violet; tiny light particles rise from them
like a quiet song. A few birds glide past. Calm, warm, magical mood.
```

**Shot 2** (start `shot2-start.png`): style block +

```text
A cosy colourful village square with a thatched cottage, an old wooden well and a workshop.
The lilac grandmother drop with bobble hat, glasses, red scarf and walking stick smiles and
sways gently; the small yellow drop Elora hops happily beside the well. Two or three other
small colourful drop creatures hop past in the background. Butterflies, flowers in the
window box. Cheerful, sunny mood. The camera pushes in very slowly.
```

**Shot 3** (start: last frame of shot 2): style block +

```text
The same village square. Suddenly the light changes: a grey shimmer creeps in from the edges
of the image, and the colours slowly drain from the houses, the trees, the flowers and the
sky until everything is pale grey. The grandmother drop and Elora stop and look up worried.
The background drop creatures hide. The camera stays still. Quiet, sad mood.
```

**Shot 4** (start `shot4-start.png`): style block +

```text
The grey village square. Elora, the small yellow drop, turns towards the hills, takes a deep
breath and gives a determined little hop; the grandmother drop nods. Elora hops out of the
square, a small grappling hook on a thin chain shoots from her body, catches a branch and she
swings in a big arc towards the green hills while the sun rises and warm colour returns to
the land around her. The camera follows her. Hopeful, brave mood.
```

### Afterwards

- Download the four clips and run
  `cargo xtask intro-import shot1.mp4 shot2.mp4 shot3.mp4 shot4.mp4` – it joins them in
  order, drops the sound and checks the result with the game's decoder.
- If a shot is not exactly 8 s, tell Claude the lengths; the subtitle times are adjusted.
- Try two or three generations per shot and keep the best; the start image keeps the look.
- Check Google's terms for the generated video (use in a GPL/CC-BY-SA game, watermark,
  SynthID) and note the tool in `assets/SOURCES.md`.

## Decided (E-357)

- Intro music: the existing `tauwinkel` track.
- The intro plays at every new adventure, always skippable.

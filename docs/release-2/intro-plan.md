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
| I-1 | New crate `crates/elora-video`: IVF reader, AV1 decoding with `rav1d`, YUV 4:2:0 → RGBA | The only crate with `#![allow(unsafe_code)]` (E-356); small, documented, unit tests with a tiny test clip (~50 KB) made by ffmpeg; `asm` feature off (no nasm needed for cross builds) unless decoding is too slow |
| I-2 | Renderer: full-screen textured quad, the texture updated per frame | First image texture in `elora-render` (everything else is meshes) |
| I-3 | Client: intro screen before the first map of a new adventure; skip with Esc/Space/Enter/click (not in the first 0.5 s); subtitles with fade; decoding on a background thread, a few frames buffered; music start and fade-out | Also “Watch intro” in the adventure menu (pause → adventure tab) |
| I-4 | `cargo xtask intro-import <video> [--crf N]`: ffmpeg → 1280×720, 24 fps, AV1 (SVT-AV1), no audio, writes `assets/intro/intro.ivf`, prints size and duration | Checks that ffmpeg with an AV1 encoder is installed |
| I-5 | Packaging: ship `assets/intro/`; `SOURCES.md` and the About page name the tool and the licence of the video (E-355) | |
| I-6 | Placeholder until the real video exists: a short clip from the existing chapter drafts (design tools) so I-3 can be tested and played | Replaced by the real video |
| I-7 | Tests: decoder (frame count, size, first pixel colours of the test clip), intro timing (skip, end, subtitle at time t), the game starts without `intro.ivf` | `cargo xtask check` green |

## Storyboard (for the video tool) – please review

Style for every shot: *flat 2D vector illustration, soft pastel colours, thick dark outlines,
rounded shapes, children's picture-book style, no text, no people, gentle slow camera movement,
16:9.* Characters are round drop creatures with big eyes (see `assets/elora/elora.svg` and the
chapter drafts in `docs/release-2/design/` as reference images).

| Shot | Time | Picture (prompt idea) | Subtitle de | Subtitle en |
|---|---|---|---|---|
| 1 | 0–9 s | Wide view over the Tauland at dawn: hills, a forest, a desert far away, snowy peaks; five springs sparkle in five colours (green, brown, gold, ice blue, violet) | Im Tauland singen fünf Quellen. Ihr Tau schenkt allem Farbe, Mut und Leben. | In the Tauland, five springs sing. Their dew gives everything colour, courage and life. |
| 2 | 9–18 s | The small village Tauwinkel in a hollow, colourful houses around a well with a wooden lid; drop creatures go about their day | In Tauwinkel, mitten im Tauland, leben die Tropfen – fröhlich und bunt. | In Tauwinkel, in the middle of the Tauland, the drops live – cheerful and colourful. |
| 3 | 18–30 s | One by one the springs stop sparkling; a grey shimmer creeps over the land; the colours of the village fade to grey | Doch eines Morgens verstummt die erste Quelle. Dann die nächste. Und die Farben verblassen. | But one morning the first spring falls silent. Then the next. And the colours fade. |
| 4 | 30–40 s | Inside a cosy house: an old grandmother drop with a knitted hat looks worried out of the window at the grey village | „Als ich so klein war wie du, sangen die Quellen noch Morgenlieder …“ | “When I was as small as you, the springs still sang morning songs …” |
| 5 | 40–50 s | A small, brave turquoise drop (Elora) stands on the well square, looks up at the hills, a hook on a line in her hand | Elora will nicht warten, bis alles grau ist. | Elora does not want to wait until everything is grey. |
| 6 | 50–60 s | Elora swings with her hook over a stream towards the green hills; sunrise; the camera follows her and stops on the horizon | Sie macht sich auf, die Quellen wieder zum Singen zu bringen. | She sets out to make the springs sing again. |

The secret of the sixth spring and the Withered One are deliberately not shown.

## Decided (E-357)

- Intro music: the existing `tauwinkel` track.
- The intro plays at every new adventure, always skippable.

//! Intro video at the start of a new adventure (E-355, E-357, `docs/release-2/intro-plan.md`).
//!
//! The video (`assets/intro/intro.ivf`) is decoded on a background thread a few frames ahead;
//! the game shows the frame that is due, draws the subtitles from `assets/intro/intro.toml`
//! and plays the Tauwinkel music. Without the video file there is no intro.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError, sync_channel};
use std::time::Instant;

use elora_video::Frame;
use serde::Deserialize;

pub const VIDEO: &str = "assets/intro/intro.ivf";
pub const SUBTITLES: &str = "assets/intro/intro.toml";
/// Skipping only after this time, so the key that started the adventure does not skip it.
pub const SKIP_GUARD: f32 = 0.5;
/// Fade of a subtitle in and out (seconds).
const FADE: f32 = 0.4;
/// Frames decoded ahead.
const AHEAD: usize = 4;

/// Is there an intro video? Checked once.
pub fn available() -> bool {
    static FOUND: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *FOUND.get_or_init(|| elora_server::paths::resolve(Path::new(VIDEO)).is_file())
}

/// One subtitle: shown from `from` to `to` seconds, text `intro.<key>` from `assets/lang`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Line {
    pub from: f32,
    pub to: f32,
    pub key: String,
}

#[derive(Debug, Deserialize)]
struct SubtitleFile {
    line: Vec<Line>,
}

/// The subtitle at time `t` and its opacity (fade in and out).
pub fn subtitle_at(lines: &[Line], t: f32) -> Option<(&Line, f32)> {
    let line = lines.iter().find(|l| t >= l.from && t < l.to)?;
    let alpha = ((t - line.from) / FADE)
        .min((line.to - t) / FADE)
        .clamp(0.0, 1.0);
    Some((line, alpha))
}

/// Index of the frame that is due at `t` seconds.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn frame_due(t: f32, fps: f32) -> usize {
    (t.max(0.0) * fps).floor() as usize
}

/// Splits `text` into lines that are at most `max` wide (`width` measures a piece of text).
pub fn wrap(text: &str, max: f32, width: impl Fn(&str) -> f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if !line.is_empty() && width(&candidate) > max {
            lines.push(std::mem::replace(&mut line, word.to_owned()));
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// A running intro.
#[derive(Debug)]
pub struct Intro {
    frames: Receiver<Frame>,
    fps: f32,
    /// Number of frames taken from the decoder so far.
    shown: usize,
    started: Instant,
    /// The decoder has no more frames.
    decoded_all: bool,
    pub subtitles: Vec<Line>,
}

impl Intro {
    /// Starts the intro if the video exists and can be opened; otherwise `None`.
    pub fn start(video: &Path, subtitles: &Path, now: Instant) -> Option<Self> {
        let data = std::fs::read(video).ok()?;
        let fps = elora_video::Header::parse(&data)
            .inspect_err(|e| tracing::warn!("intro video {}: {e}", video.display()))
            .ok()?
            .fps();
        let lines = std::fs::read_to_string(subtitles)
            .ok()
            .and_then(|s| {
                toml::from_str::<SubtitleFile>(&s)
                    .inspect_err(|e| tracing::warn!("intro subtitles: {e}"))
                    .ok()
            })
            .map_or_else(Vec::new, |f| f.line);
        let (tx, rx) = sync_channel(AHEAD);
        let spawned = std::thread::Builder::new()
            .name("intro-video".into())
            .spawn(move || {
                // the decoder lives on this thread only (it is not `Send`)
                let mut decoder = match elora_video::Video::new(data) {
                    Ok(d) => d,
                    Err(e) => {
                        tracing::warn!("intro video: {e}");
                        return;
                    }
                };
                loop {
                    match decoder.next_frame() {
                        // stops when the intro is over (receiver dropped)
                        Ok(Some(f)) => {
                            if tx.send(f).is_err() {
                                return;
                            }
                        }
                        Ok(None) => return,
                        Err(e) => {
                            tracing::warn!("intro video: {e}");
                            return;
                        }
                    }
                }
            });
        if let Err(e) = spawned {
            tracing::warn!("intro video thread: {e}");
            return None;
        }
        Some(Self {
            frames: rx,
            fps,
            shown: 0,
            started: now,
            decoded_all: false,
            subtitles: lines,
        })
    }

    /// The default files next to the game data.
    pub fn start_default(now: Instant) -> Option<Self> {
        let path = |p: &str| -> PathBuf { elora_server::paths::resolve(Path::new(p)) };
        Self::start(&path(VIDEO), &path(SUBTITLES), now)
    }

    /// Seconds since the start.
    pub fn time(&self, now: Instant) -> f32 {
        now.saturating_duration_since(self.started).as_secs_f32()
    }

    /// The newest frame that is due now (`None`: keep showing the last one).
    pub fn update(&mut self, now: Instant) -> Option<Frame> {
        let due = frame_due(self.time(now), self.fps);
        let mut newest = None;
        while self.shown <= due && !self.decoded_all {
            match self.frames.try_recv() {
                Ok(f) => {
                    newest = Some(f);
                    self.shown += 1;
                }
                // decoder behind: show the newest frame we have
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => self.decoded_all = true,
            }
        }
        newest
    }

    /// The video is over: all frames shown and their time passed.
    pub fn finished(&self, now: Instant) -> bool {
        #[allow(clippy::cast_precision_loss)]
        let end = self.shown as f32 / self.fps;
        self.decoded_all && self.time(now) >= end
    }

    /// May the player skip now?
    pub fn can_skip(&self, now: Instant) -> bool {
        self.time(now) >= SKIP_GUARD
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines() -> Vec<Line> {
        toml::from_str::<SubtitleFile>(include_str!("../../../assets/intro/intro.toml"))
            .expect("intro.toml")
            .line
    }

    #[test]
    fn subtitles_fade_and_have_texts_in_both_languages() {
        let lines = lines();
        assert!(lines.len() >= 4);
        assert!(subtitle_at(&lines, 0.0).is_none());
        let (l, a) = subtitle_at(&lines, 0.7).expect("first line");
        assert_eq!(l.key, "lands");
        assert!(a > 0.0 && a < 1.0, "fading in: {a}");
        assert!((subtitle_at(&lines, 4.0).unwrap().1 - 1.0).abs() < 1e-6);
        for w in lines.windows(2) {
            assert!(w[0].to <= w[1].from, "lines overlap: {w:?}");
        }
        for lang in crate::lang::Language::ALL.map(crate::lang::Lang::new) {
            for l in &lines {
                let key = format!("intro.{}", l.key);
                assert_ne!(lang.t(&key), key, "text missing: {key}");
            }
        }
    }

    #[test]
    fn frame_timing() {
        assert_eq!(frame_due(0.0, 24.0), 0);
        assert_eq!(frame_due(1.0, 24.0), 24);
        assert_eq!(frame_due(-1.0, 24.0), 0);
    }

    #[test]
    fn wrapping_keeps_words() {
        let w = |s: &str| s.chars().count() as f32;
        assert_eq!(
            wrap("one two three four", 9.0, w),
            ["one two", "three", "four"]
        );
        assert_eq!(wrap("", 9.0, w), Vec::<String>::new());
        assert_eq!(wrap("longwordhere", 4.0, w), ["longwordhere"]);
    }

    #[test]
    fn plays_the_test_clip_to_the_end() {
        let dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../crates/elora-video/tests/data"
        );
        let start = Instant::now();
        let mut intro = Intro::start(
            &Path::new(dir).join("clip.ivf"),
            Path::new("does/not/exist.toml"),
            start,
        )
        .expect("clip opens");
        assert!(intro.subtitles.is_empty());
        assert!(!intro.can_skip(start));
        let mut got = 0;
        // 12 frames at 24 fps = 0.5 s; step in 10 ms until done
        for i in 0..400u64 {
            let now = start + std::time::Duration::from_millis(i * 10);
            if intro.update(now).is_some() {
                got += 1;
            }
            if intro.finished(now) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(got > 0, "frames shown");
        assert_eq!(intro.shown, 12);
        assert!(intro.finished(start + std::time::Duration::from_secs(5)));
        assert!(Intro::start(Path::new("no/video.ivf"), Path::new("x"), start).is_none());
    }
}

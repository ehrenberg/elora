//! Intro video at the start of a new adventure (E-355, E-357, `docs/release-2/intro-plan.md`).
//!
//! The video (`assets/intro/intro.ivf`) is decoded on a background thread a few frames ahead;
//! the game shows the frame that is due. The narration (`assets/intro/narration-<lang>.ogg`)
//! and the Tauwinkel music come from [`crate::sound::Sounds::intro`]. Without the video file
//! there is no intro.

use std::path::Path;
use std::sync::mpsc::{Receiver, TryRecvError, sync_channel};
use std::time::Instant;

use elora_video::Frame;

pub const VIDEO: &str = "assets/intro/intro.ivf";
/// Skipping only after this time, so the key that started the adventure does not skip it.
pub const SKIP_GUARD: f32 = 0.5;
/// Frames decoded ahead.
const AHEAD: usize = 4;

/// Is there an intro video? Checked once.
pub fn available() -> bool {
    static FOUND: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *FOUND.get_or_init(|| elora_server::paths::resolve(Path::new(VIDEO)).is_file())
}

/// Index of the frame that is due at `t` seconds.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn frame_due(t: f32, fps: f32) -> usize {
    (t.max(0.0) * fps).floor() as usize
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
}

impl Intro {
    /// Starts the intro if the video exists and can be opened; otherwise `None`.
    pub fn start(video: &Path, now: Instant) -> Option<Self> {
        let data = std::fs::read(video).ok()?;
        let fps = elora_video::Header::parse(&data)
            .inspect_err(|e| tracing::warn!("intro video {}: {e}", video.display()))
            .ok()?
            .fps();
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
        })
    }

    /// The default files next to the game data.
    pub fn start_default(now: Instant) -> Option<Self> {
        Self::start(&elora_server::paths::resolve(Path::new(VIDEO)), now)
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

    /// Narration for every language of the game.
    #[test]
    fn narration_exists_for_every_language() {
        for lang in crate::lang::Language::ALL {
            let code = match lang {
                crate::lang::Language::De => "de",
                crate::lang::Language::En => "en",
            };
            let path = format!(
                "{}/../../{}/narration-{code}.ogg",
                env!("CARGO_MANIFEST_DIR"),
                crate::sound::INTRO_DIR
            );
            assert!(Path::new(&path).is_file(), "{path}");
        }
    }

    #[test]
    fn frame_timing() {
        assert_eq!(frame_due(0.0, 24.0), 0);
        assert_eq!(frame_due(1.0, 24.0), 24);
        assert_eq!(frame_due(-1.0, 24.0), 0);
    }

    #[test]
    fn plays_the_test_clip_to_the_end() {
        let dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../crates/elora-video/tests/data"
        );
        let start = Instant::now();
        let mut intro = Intro::start(&Path::new(dir).join("clip.ivf"), start).expect("clip opens");
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
        assert!(Intro::start(Path::new("no/video.ivf"), start).is_none());
    }
}

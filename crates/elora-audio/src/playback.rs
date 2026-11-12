//! Playback with kira (feature `playback`).

use std::collections::BTreeMap;
use std::sync::Arc;

use elora_sim::Vec2;
use kira::sound::FromFileError;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::{AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Frame, Panning, Tween};

#[cfg(target_os = "linux")]
use crate::OUTPUT_BUFFER_FRAMES;
use crate::{AudioSettings, Bank, Cue, SAMPLE_RATE, Sound, spatial};

/// Playback with kira.
pub struct Audio {
    manager: Option<AudioManager<DefaultBackend>>,
    sounds: BTreeMap<Sound, StaticSoundData>,
    applied: Option<AudioSettings>,
    /// Running music (loop) and its volume.
    music: Option<(StreamingSoundHandle<FromFileError>, f32)>,
    /// Ambience track (weather, R2-W1): running loops per name with volume.
    ambience: BTreeMap<String, (StreamingSoundHandle<FromFileError>, f32)>,
    /// Spoken narration (intro, E-355): once, can be cut off.
    voice: Option<StreamingSoundHandle<FromFileError>>,
}

impl std::fmt::Debug for Audio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Audio")
            .field("device", &self.manager.is_some())
            .field("sounds", &self.sounds.len())
            .finish_non_exhaustive()
    }
}

fn to_data(samples: &[f32]) -> StaticSoundData {
    let frames: Arc<[Frame]> = samples.iter().map(|&s| Frame::from_mono(s)).collect();
    StaticSoundData {
        sample_rate: SAMPLE_RATE,
        frames,
        settings: StaticSoundSettings::default(),
        slice: None,
    }
}

/// Opens the audio output (E-375). Linux: the `PulseAudio` server's default sink (`PipeWire`
/// serves it too, without `pipewire-alsa`), then every ALSA output device in turn, the default
/// one first. The device is passed explicitly, so kira does not poll for the default device
/// (with `PulseAudio` that would open a new connection every 500 ms); the sound server moves
/// the stream when the user switches outputs. Other systems: kira's default device.
/// `None` when nothing opens; the game then stays silent.
fn open_output() -> Option<AudioManager<DefaultBackend>> {
    #[cfg(target_os = "linux")]
    {
        use kira::backend::cpal::cpal::traits::DeviceTrait;
        for (host, device) in linux_outputs() {
            let name = device.to_string();
            let config = device
                .default_output_config()
                .ok()
                .map(|c| stream_config(host, c.config()));
            let settings = AudioManagerSettings {
                backend_settings: kira::backend::cpal::CpalBackendSettings {
                    device: Some(device),
                    config,
                },
                ..AudioManagerSettings::default()
            };
            match AudioManager::<DefaultBackend>::new(settings) {
                Ok(m) => {
                    tracing::info!("Audio output: {host} `{name}`, buffer {config:?}");
                    return Some(m);
                }
                Err(e) => tracing::warn!("Audio output {host} `{name}` failed: {e}"),
            }
        }
        tracing::warn!("No audio device, game stays silent");
        None
    }
    #[cfg(not(target_os = "linux"))]
    match AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
        Ok(m) => Some(m),
        Err(e) => {
            tracing::warn!("No audio device, game stays silent: {e}");
            None
        }
    }
}

/// Stream configuration for an output of `host` (E-378). `PulseAudio` gets a fixed buffer of
/// [`OUTPUT_BUFFER_FRAMES`]: without one, cpal leaves the buffer to the server, which picks a
/// target length of 2 s, so every sound was heard about 2 s after its event. ALSA keeps its
/// default (≈ 35 ms through `pipewire-alsa`; a fixed period there fails with I/O errors).
#[cfg(target_os = "linux")]
fn stream_config(
    host: &str,
    default: kira::backend::cpal::cpal::StreamConfig,
) -> kira::backend::cpal::cpal::StreamConfig {
    use kira::backend::cpal::cpal::{BufferSize, StreamConfig};
    if host == PULSE_AUDIO {
        StreamConfig {
            buffer_size: BufferSize::Fixed(OUTPUT_BUFFER_FRAMES),
            ..default
        }
    } else {
        default
    }
}

/// Host name of cpal's `PulseAudio` host in [`linux_outputs`].
#[cfg(target_os = "linux")]
const PULSE_AUDIO: &str = "PulseAudio";

/// Output devices to try on Linux in this order: the `PulseAudio` default sink, then the
/// ALSA default device and all other ALSA output devices.
#[cfg(target_os = "linux")]
fn linux_outputs() -> Vec<(&'static str, kira::backend::cpal::cpal::Device)> {
    use kira::backend::cpal::cpal::{self, HostId, traits::HostTrait};
    let mut devices = Vec::new();
    if cpal::available_hosts().contains(&HostId::PulseAudio)
        && let Ok(host) = cpal::host_from_id(HostId::PulseAudio)
        && let Some(device) = host.default_output_device()
    {
        devices.push((PULSE_AUDIO, device));
    }
    if let Ok(host) = cpal::host_from_id(HostId::Alsa) {
        devices.extend(host.default_output_device().map(|d| ("ALSA", d)));
        if let Ok(all) = host.output_devices() {
            devices.extend(all.map(|d| ("ALSA", d)));
        }
    }
    devices
}

/// Amplitude 0..1 → decibels for kira.
fn decibels(amplitude: f32) -> Decibels {
    if amplitude <= 0.001 {
        Decibels::SILENCE
    } else {
        Decibels(20.0 * amplitude.log10())
    }
}

impl Audio {
    /// Creates all sounds of the bank and opens the audio device. Without a device
    /// the game stays muted (warning in the log).
    pub fn new(bank: &Bank) -> Self {
        let manager = open_output();
        let sounds = bank
            .sounds
            .iter()
            .map(|(s, src)| (*s, to_data(&src.samples())))
            .collect();
        for s in bank.missing() {
            tracing::warn!(
                "Sound `{}` missing (neither sounds.toml nor sound file)",
                s.name()
            );
        }
        Self {
            manager,
            sounds,
            applied: None,
            music: None,
            ambience: BTreeMap::new(),
            voice: None,
        }
    }

    /// Whether an audio device is open.
    pub fn has_device(&self) -> bool {
        self.manager.is_some()
    }

    /// Applies the master volume (only on change).
    pub fn apply(&mut self, settings: AudioSettings) {
        if self.applied == Some(settings) {
            return;
        }
        self.applied = Some(settings);
        if let Some(m) = &mut self.manager {
            let amp = if settings.muted {
                0.0
            } else {
                settings.volume.clamp(0.0, 1.0)
            };
            m.main_track().set_volume(decibels(amp), Tween::default());
        }
    }

    /// Starts music (Ogg Vorbis or WAV, decoded while playing) in a loop if none is
    /// running yet; `volume` 0..1 is smoothly adjusted on change.
    ///
    /// # Errors
    /// The file is not readable music (without an audio device: never).
    pub fn play_music(&mut self, data: &Arc<[u8]>, volume: f32) -> Result<(), String> {
        let tween = Tween {
            duration: std::time::Duration::from_millis(300),
            ..Tween::default()
        };
        if let Some((handle, v)) = &mut self.music {
            if (*v - volume).abs() > 0.001 {
                handle.set_volume(decibels(volume), tween);
                *v = volume;
            }
            return Ok(());
        }
        let Some(manager) = &mut self.manager else {
            return Ok(());
        };
        let sound = StreamingSoundData::from_cursor(std::io::Cursor::new(data.clone()))
            .map_err(|e| e.to_string())?
            .loop_region(..)
            .volume(Decibels::SILENCE);
        match manager.play(sound) {
            // fade in smoothly
            Ok(mut handle) => {
                handle.set_volume(
                    decibels(volume),
                    Tween {
                        duration: std::time::Duration::from_millis(1200),
                        ..Tween::default()
                    },
                );
                self.music = Some((handle, volume));
            }
            Err(e) => tracing::debug!("Music not played: {e}"),
        }
        Ok(())
    }

    /// Fades out and stops the music.
    pub fn stop_music(&mut self) {
        if let Some((mut handle, _)) = self.music.take() {
            handle.stop(Tween {
                duration: std::time::Duration::from_millis(600),
                ..Tween::default()
            });
        }
    }

    /// Adjusts the ambience loop `name` (second track next to the music, R2-W1) to
    /// `volume` 0..1 with a soft crossfade; at 0 it is faded out and stopped. Call every frame.
    ///
    /// # Errors
    /// The file is not a readable sound file (without an audio device: never).
    pub fn ambience(&mut self, name: &str, data: &Arc<[u8]>, volume: f32) -> Result<(), String> {
        let fade = |ms| Tween {
            duration: std::time::Duration::from_millis(ms),
            ..Tween::default()
        };
        if let Some((handle, v)) = self.ambience.get_mut(name) {
            if volume <= 0.001 {
                handle.stop(fade(1500));
                self.ambience.remove(name);
            } else if (*v - volume).abs() > 0.02 {
                handle.set_volume(decibels(volume), fade(800));
                *v = volume;
            }
            return Ok(());
        }
        if volume <= 0.001 {
            return Ok(());
        }
        let Some(manager) = &mut self.manager else {
            return Ok(());
        };
        let sound = StreamingSoundData::from_cursor(std::io::Cursor::new(data.clone()))
            .map_err(|e| e.to_string())?
            .loop_region(..)
            .volume(Decibels::SILENCE);
        match manager.play(sound) {
            Ok(mut handle) => {
                handle.set_volume(decibels(volume), fade(2000));
                self.ambience.insert(name.to_owned(), (handle, volume));
            }
            Err(e) => tracing::debug!("Ambience `{name}` not played: {e}"),
        }
        Ok(())
    }

    /// Fades out all ambience loops (menu, editor).
    pub fn stop_ambience(&mut self) {
        let names: Vec<String> = self.ambience.keys().cloned().collect();
        for n in names {
            if let Some((mut h, _)) = self.ambience.remove(&n) {
                h.stop(Tween {
                    duration: std::time::Duration::from_millis(800),
                    ..Tween::default()
                });
            }
        }
    }

    /// Plays spoken narration once (streamed); a running one is replaced.
    ///
    /// # Errors
    /// The file is not a readable sound file (without an audio device: never).
    pub fn play_voice(&mut self, data: &Arc<[u8]>, volume: f32) -> Result<(), String> {
        self.stop_voice();
        let Some(manager) = &mut self.manager else {
            return Ok(());
        };
        let sound = StreamingSoundData::from_cursor(std::io::Cursor::new(data.clone()))
            .map_err(|e| e.to_string())?
            .volume(decibels(volume));
        match manager.play(sound) {
            Ok(handle) => self.voice = Some(handle),
            Err(e) => tracing::debug!("Voice not played: {e}"),
        }
        Ok(())
    }

    /// Fades the narration out quickly (intro skipped).
    pub fn stop_voice(&mut self) {
        if let Some(mut handle) = self.voice.take() {
            handle.stop(Tween {
                duration: std::time::Duration::from_millis(250),
                ..Tween::default()
            });
        }
    }

    /// Plays a longer sound (thunder) once, streamed, with volume and panning.
    ///
    /// # Errors
    /// The file is not a readable sound file (without an audio device: never).
    pub fn play_once(&mut self, data: &Arc<[u8]>, volume: f32, pan: f32) -> Result<(), String> {
        let Some(manager) = &mut self.manager else {
            return Ok(());
        };
        let sound = StreamingSoundData::from_cursor(std::io::Cursor::new(data.clone()))
            .map_err(|e| e.to_string())?
            .volume(decibels(volume))
            .panning(Panning(pan));
        if let Err(e) = manager.play(sound) {
            tracing::debug!("Sound not played: {e}");
        }
        Ok(())
    }

    /// Plays `cue`; spatial sounds relative to `ear` (camera center).
    pub fn play(&mut self, cue: &Cue, ear: Vec2) {
        let Some(manager) = &mut self.manager else {
            return;
        };
        let Some(data) = self.sounds.get(&cue.sound) else {
            return;
        };
        let (gain, pan) = match cue.pos {
            Some(p) => match spatial(p, ear) {
                Some(v) => v,
                None => return,
            },
            None => (1.0, 0.0),
        };
        let sound = data
            .volume(decibels(gain))
            .panning(Panning(pan))
            .playback_rate(kira::PlaybackRate(f64::from(cue.pitch)));
        if let Err(e) = manager.play(sound) {
            tracing::debug!("Sound `{}` not played: {e}", cue.sound.name());
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod latency_tests {
    use kira::backend::cpal::cpal::{BufferSize, StreamConfig};

    use super::*;

    fn default_config() -> StreamConfig {
        StreamConfig {
            channels: 2,
            sample_rate: 48_000,
            buffer_size: BufferSize::Default,
        }
    }

    #[test]
    fn pulse_audio_output_requests_a_short_buffer() {
        // E-378: without a fixed size the server buffers 2 s, so sounds came late
        let config = stream_config(PULSE_AUDIO, default_config());
        let BufferSize::Fixed(frames) = config.buffer_size else {
            panic!("PulseAudio output without a fixed buffer: {config:?}");
        };
        // the server holds two callbacks' worth: keep that well under a noticeable 50 ms
        let latency_ms = 2.0 * f64::from(frames) / f64::from(config.sample_rate) * 1000.0;
        assert!(latency_ms <= 50.0, "buffer latency {latency_ms:.1} ms");
        assert_eq!((config.channels, config.sample_rate), (2, 48_000));
    }

    #[test]
    fn alsa_output_keeps_its_default_buffer() {
        let config = stream_config("ALSA", default_config());
        assert_eq!(config.buffer_size, BufferSize::Default);
    }
}

#[cfg(test)]
mod music_tests {
    use super::*;

    #[test]
    fn shipped_music_can_be_streamed() {
        // every shipped music and ambience file (the folders are packaged as they are)
        let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let mut count = 0;
        for folder in ["music", "ambience"] {
            for entry in std::fs::read_dir(assets.join(folder)).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().is_none_or(|e| e != "ogg") {
                    continue;
                }
                let data: Arc<[u8]> = std::fs::read(&path).unwrap().into();
                assert!(
                    StreamingSoundData::from_cursor(std::io::Cursor::new(data)).is_ok(),
                    "{}",
                    path.display()
                );
                count += 1;
            }
        }
        assert!(count >= 16, "music and ambience found: {count}");
        let junk: Arc<[u8]> = Arc::from(&b"not music"[..]);
        assert!(StreamingSoundData::from_cursor(std::io::Cursor::new(junk)).is_err());
    }
}

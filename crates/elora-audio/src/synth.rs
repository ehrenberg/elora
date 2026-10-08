//! Procedural sound generator following the sfxr principle (E-081, E-105).
//!
//! A sound consists of one or more [`Layer`]s that are mixed. Each layer is a
//! waveform with a pitch curve, a volume envelope and filters. All values are in
//! seconds, hertz or octaves – so they can be tuned by hand in
//! `assets/sounds/sounds.toml`. The same parameters always produce the same sound
//! (noise with a fixed seed).

use serde::{Deserialize, Serialize};

/// Sample rate of the generated sounds.
pub const SAMPLE_RATE: u32 = 44_100;
/// Longest allowed sound (protection against typos in the file).
const MAX_SECONDS: f32 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Wave {
    #[default]
    Square,
    Saw,
    Sine,
    Triangle,
    /// Noise whose "pitch" sets the frequency (as in sfxr).
    Noise,
}

/// One sound layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Layer {
    pub wave: Wave,
    /// Start frequency (Hz).
    pub freq: f32,
    /// Pitch change in octaves per second (negative = falling).
    pub slide: f32,
    /// Change of `slide` per second.
    pub slide_accel: f32,
    /// Lower frequency limit (Hz); below it the layer ends.
    pub min_freq: f32,
    /// Vibrato: depth (fraction of the frequency) and rate (Hz).
    pub vibrato: f32,
    pub vibrato_hz: f32,
    /// Arpeggio: after `arp_after` s the frequency is multiplied by `arp`.
    pub arp: f32,
    pub arp_after: f32,
    /// Repeat: reset the pitch every `repeat` s (0 = off).
    pub repeat: f32,
    /// Envelope (s): attack, sustain, decay; `punch` boosts the sustain (0..1).
    pub attack: f32,
    pub sustain: f32,
    pub punch: f32,
    pub decay: f32,
    /// Duty cycle of the square wave (0..1) and its change per second.
    pub duty: f32,
    pub duty_sweep: f32,
    /// Low-pass (Hz, 0 = off), change as a factor per second, resonance (0..1).
    pub lowpass: f32,
    pub lowpass_sweep: f32,
    pub resonance: f32,
    /// High-pass (Hz, 0 = off).
    pub highpass: f32,
    /// Volume of the layer (0..1).
    pub volume: f32,
    /// Delay of the onset (s).
    pub delay: f32,
    /// Seed of the noise.
    pub seed: u64,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            wave: Wave::Square,
            freq: 440.0,
            slide: 0.0,
            slide_accel: 0.0,
            min_freq: 20.0,
            vibrato: 0.0,
            vibrato_hz: 6.0,
            arp: 1.0,
            arp_after: 0.0,
            repeat: 0.0,
            attack: 0.0,
            sustain: 0.05,
            punch: 0.0,
            decay: 0.15,
            duty: 0.5,
            duty_sweep: 0.0,
            lowpass: 0.0,
            lowpass_sweep: 1.0,
            resonance: 0.0,
            highpass: 0.0,
            volume: 1.0,
            delay: 0.0,
            seed: 1,
        }
    }
}

impl Layer {
    /// Duration including delay (s).
    pub fn duration(&self) -> f32 {
        self.delay + self.attack + self.sustain + self.decay
    }

    /// Volume envelope at time `t` after the onset.
    fn envelope(&self, t: f32) -> f32 {
        if t < self.attack {
            t / self.attack.max(1e-6)
        } else if t < self.attack + self.sustain {
            let x = (t - self.attack) / self.sustain.max(1e-6);
            1.0 + self.punch * 2.0 * (1.0 - x)
        } else {
            let x = (t - self.attack - self.sustain) / self.decay.max(1e-6);
            (1.0 - x).max(0.0)
        }
    }

    /// Mixes the layer into `out` (mono, [`SAMPLE_RATE`]).
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn render_into(&self, out: &mut [f32]) {
        let dt = 1.0 / SAMPLE_RATE as f32;
        let start = (self.delay.max(0.0) * SAMPLE_RATE as f32) as usize;
        let len =
            ((self.attack + self.sustain + self.decay).max(0.0) * SAMPLE_RATE as f32) as usize;
        let mut rng = self.seed.max(1);
        let mut noise = [0.0_f32; 32];
        let refill = |rng: &mut u64, buf: &mut [f32; 32]| {
            for v in buf.iter_mut() {
                *rng ^= *rng << 13;
                *rng ^= *rng >> 7;
                *rng ^= *rng << 17;
                *v = (*rng >> 40) as f32 / (1u64 << 23) as f32 - 1.0;
            }
        };
        refill(&mut rng, &mut noise);

        let mut phase = 0.0_f32;
        let mut slide = self.slide;
        let mut octave = 0.0_f32;
        let mut duty = self.duty;
        let mut cutoff = self.lowpass;
        // Filter state: state-variable low-pass and one-pole high-pass
        let (mut low, mut band) = (0.0_f32, 0.0_f32);
        let (mut hp_prev_in, mut hp_prev_out) = (0.0_f32, 0.0_f32);

        for i in 0..len {
            let t = i as f32 * dt;
            let local = if self.repeat > 0.0 {
                t % self.repeat
            } else {
                t
            };
            if self.repeat > 0.0 && local < dt {
                octave = 0.0;
                slide = self.slide;
            }
            slide += self.slide_accel * dt;
            octave += slide * dt;
            let mut freq = self.freq * octave.exp2();
            if (self.arp - 1.0).abs() > f32::EPSILON
                && self.arp_after > 0.0
                && local >= self.arp_after
            {
                freq *= self.arp;
            }
            if self.vibrato > 0.0 {
                freq *= 1.0 + self.vibrato * (t * self.vibrato_hz * std::f32::consts::TAU).sin();
            }
            if freq < self.min_freq {
                break;
            }
            let prev_phase = phase;
            phase = (phase + freq * dt).fract();
            if self.wave == Wave::Noise && phase < prev_phase {
                refill(&mut rng, &mut noise);
            }
            duty = (duty + self.duty_sweep * dt).clamp(0.02, 0.98);
            let sample = match self.wave {
                // zero-mean: no DC offset at any duty cycle
                Wave::Square => {
                    if phase < duty {
                        1.0 - duty
                    } else {
                        -duty
                    }
                }
                Wave::Saw => 1.0 - phase * 2.0,
                Wave::Sine => (phase * std::f32::consts::TAU).sin(),
                Wave::Triangle => 1.0 - 4.0 * (phase - 0.5).abs(),
                Wave::Noise => noise[((phase * 32.0) as usize).min(31)],
            };

            let mut s = sample;
            if self.lowpass > 0.0 {
                cutoff = (cutoff * self.lowpass_sweep.powf(dt)).clamp(20.0, 20_000.0);
                let f = (2.0 * (std::f32::consts::PI * cutoff / SAMPLE_RATE as f32).sin()).min(1.0);
                let q = 1.0 - self.resonance.clamp(0.0, 0.95);
                low += f * band;
                let high = s - low - q * band;
                band += f * high;
                s = low;
            }
            if self.highpass > 0.0 {
                let rc = 1.0 / (std::f32::consts::TAU * self.highpass);
                let a = rc / (rc + dt);
                let y = a * (hp_prev_out + s - hp_prev_in);
                hp_prev_in = s;
                hp_prev_out = y;
                s = y;
            }
            if let Some(o) = out.get_mut(start + i) {
                *o += s * self.envelope(t) * self.volume;
            }
        }
    }
}

/// A sound made of layers with an overall volume.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SoundDef {
    pub volume: f32,
    #[serde(rename = "layer")]
    pub layers: Vec<Layer>,
}

impl Default for SoundDef {
    fn default() -> Self {
        Self {
            volume: 0.5,
            layers: Vec::new(),
        }
    }
}

impl SoundDef {
    pub fn duration(&self) -> f32 {
        self.layers
            .iter()
            .map(Layer::duration)
            .fold(0.0, f32::max)
            .min(MAX_SECONDS)
    }

    /// Produces the samples (mono, [`SAMPLE_RATE`], values in −1..1).
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn render(&self) -> Vec<f32> {
        let len = (self.duration() * SAMPLE_RATE as f32).ceil() as usize;
        let mut out = vec![0.0; len];
        for layer in &self.layers {
            layer.render_into(&mut out);
        }
        for s in &mut out {
            // soft clipping so that loud layers do not clip hard
            *s = (*s * self.volume).tanh();
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(wave: Wave) -> SoundDef {
        SoundDef {
            volume: 1.0,
            layers: vec![Layer {
                wave,
                freq: 441.0,
                sustain: 0.1,
                decay: 0.0,
                ..Layer::default()
            }],
        }
    }

    /// Zero crossings per second ≈ 2 × frequency.
    #[allow(clippy::cast_precision_loss)]
    fn crossings_per_second(s: &[f32]) -> f32 {
        let n = s
            .windows(2)
            .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
            .count();
        n as f32 / (s.len() as f32 / SAMPLE_RATE as f32)
    }

    #[test]
    fn waves_have_the_right_pitch() {
        for wave in [Wave::Square, Wave::Saw, Wave::Sine, Wave::Triangle] {
            let s = tone(wave).render();
            assert_eq!(s.len(), 4410);
            let c = crossings_per_second(&s);
            assert!((c - 882.0).abs() < 25.0, "{wave:?}: {c}");
            assert!(s.iter().all(|v| v.abs() <= 1.0));
        }
    }

    #[test]
    fn square_has_no_dc_offset() {
        let mut d = tone(Wave::Square);
        d.layers[0].duty = 0.2;
        let s = d.render();
        #[allow(clippy::cast_precision_loss)]
        let mean = s.iter().sum::<f32>() / s.len() as f32;
        // without compensation it would be −0.3; a small remainder comes from the soft
        // clipping (tanh) of the asymmetric wave
        assert!(mean.abs() < 0.05, "DC offset {mean}");
    }

    #[test]
    fn slide_lowers_pitch() {
        let mut d = tone(Wave::Sine);
        d.layers[0].slide = -3.0; // three octaves per second downwards
        let s = d.render();
        let (a, b) = s.split_at(s.len() / 2);
        assert!(crossings_per_second(b) < crossings_per_second(a) * 0.95);
    }

    #[test]
    fn envelope_and_delay() {
        let d = SoundDef {
            volume: 1.0,
            layers: vec![Layer {
                wave: Wave::Sine,
                delay: 0.05,
                attack: 0.01,
                sustain: 0.02,
                decay: 0.05,
                ..Layer::default()
            }],
        };
        assert!((d.duration() - 0.13).abs() < 1e-6);
        let s = d.render();
        let silent = SAMPLE_RATE as usize / 20; // 0.05 s
        assert!(
            s[..silent].iter().all(|v| v.abs() < f32::EPSILON),
            "silent before the onset"
        );
        assert!(s[silent..].iter().any(|v| v.abs() > 0.5));
        assert!(s.last().unwrap().abs() < 0.05, "fades out");
    }

    #[test]
    fn noise_is_deterministic_and_filters_work() {
        let mut d = tone(Wave::Noise);
        d.layers[0].freq = 2000.0;
        assert_eq!(d.render(), d.render());
        let loud: f32 = d.render().iter().map(|v| v * v).sum();
        d.layers[0].lowpass = 300.0;
        let dull: f32 = d.render().iter().map(|v| v * v).sum();
        assert!(dull < loud * 0.8, "low-pass removes energy");
    }
}

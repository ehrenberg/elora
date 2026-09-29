//! Prozeduraler Sound-Generator nach dem sfxr-Prinzip (E-081, E-105).
//!
//! Ein Sound besteht aus einer oder mehreren [`Layer`]n, die gemischt werden. Jede
//! Schicht ist eine Wellenform mit Tonhöhenverlauf, Lautstärke-Hüllkurve und Filtern.
//! Alle Werte sind in Sekunden, Hertz bzw. Oktaven – so lassen sie sich in
//! `assets/sounds/sounds.toml` von Hand einstellen. Gleiche Parameter ergeben
//! immer denselben Klang (Rauschen mit festem Startwert).

use serde::{Deserialize, Serialize};

/// Abtastrate der erzeugten Sounds.
pub const SAMPLE_RATE: u32 = 44_100;
/// Längster erlaubter Sound (Schutz vor Tippfehlern in der Datei).
const MAX_SECONDS: f32 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Wave {
    #[default]
    Square,
    Saw,
    Sine,
    Triangle,
    /// Rauschen, dessen „Tonhöhe“ die Frequenz bestimmt (wie in sfxr).
    Noise,
}

/// Eine Klangschicht.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Layer {
    pub wave: Wave,
    /// Startfrequenz (Hz).
    pub freq: f32,
    /// Tonhöhenänderung in Oktaven pro Sekunde (negativ = fällt).
    pub slide: f32,
    /// Änderung von `slide` pro Sekunde.
    pub slide_accel: f32,
    /// Untergrenze der Frequenz (Hz); darunter endet die Schicht.
    pub min_freq: f32,
    /// Vibrato: Tiefe (Anteil der Frequenz) und Tempo (Hz).
    pub vibrato: f32,
    pub vibrato_hz: f32,
    /// Arpeggio: nach `arp_after` s wird die Frequenz mit `arp` multipliziert.
    pub arp: f32,
    pub arp_after: f32,
    /// Wiederholung: Tonhöhe alle `repeat` s zurücksetzen (0 = aus).
    pub repeat: f32,
    /// Hüllkurve (s): Anstieg, Halten, Abklingen; `punch` hebt das Halten an (0..1).
    pub attack: f32,
    pub sustain: f32,
    pub punch: f32,
    pub decay: f32,
    /// Tastverhältnis der Rechteckwelle (0..1) und seine Änderung pro Sekunde.
    pub duty: f32,
    pub duty_sweep: f32,
    /// Tiefpass (Hz, 0 = aus), Änderung als Faktor pro Sekunde, Resonanz (0..1).
    pub lowpass: f32,
    pub lowpass_sweep: f32,
    pub resonance: f32,
    /// Hochpass (Hz, 0 = aus).
    pub highpass: f32,
    /// Lautstärke der Schicht (0..1).
    pub volume: f32,
    /// Verzögerung des Einsatzes (s).
    pub delay: f32,
    /// Startwert des Rauschens.
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
    /// Dauer inklusive Verzögerung (s).
    pub fn duration(&self) -> f32 {
        self.delay + self.attack + self.sustain + self.decay
    }

    /// Lautstärke-Hüllkurve zur Zeit `t` nach dem Einsatz.
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

    /// Schicht in `out` hineinmischen (Mono, [`SAMPLE_RATE`]).
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
        // Zustand der Filter: Zustandsvariablen-Tiefpass und einpoliger Hochpass
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
                // mittelwertfrei: bei jedem Tastverhältnis ohne Gleichanteil
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

/// Ein Sound aus Schichten mit Gesamtlautstärke.
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

    /// Erzeugt die Samples (Mono, [`SAMPLE_RATE`], Werte in −1..1).
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
            // weich begrenzen, damit laute Schichten nicht hart übersteuern
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

    /// Nulldurchgänge pro Sekunde ≈ 2 × Frequenz.
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
        // ohne Ausgleich wären es −0.3; ein kleiner Rest entsteht durch die weiche
        // Begrenzung (tanh) der unsymmetrischen Welle
        assert!(mean.abs() < 0.05, "Gleichanteil {mean}");
    }

    #[test]
    fn slide_lowers_pitch() {
        let mut d = tone(Wave::Sine);
        d.layers[0].slide = -3.0; // drei Oktaven pro Sekunde abwärts
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
            "vor dem Einsatz still"
        );
        assert!(s[silent..].iter().any(|v| v.abs() > 0.5));
        assert!(s.last().unwrap().abs() < 0.05, "klingt aus");
    }

    #[test]
    fn noise_is_deterministic_and_filters_work() {
        let mut d = tone(Wave::Noise);
        d.layers[0].freq = 2000.0;
        assert_eq!(d.render(), d.render());
        let loud: f32 = d.render().iter().map(|v| v * v).sum();
        d.layers[0].lowpass = 300.0;
        let dull: f32 = d.render().iter().map(|v| v * v).sum();
        assert!(dull < loud * 0.8, "Tiefpass nimmt Energie heraus");
    }
}

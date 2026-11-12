//! Measures the output latency of the audio hosts Elora uses (E-378): opens a silent
//! stream per host and buffer size and prints how far ahead of the sound card the
//! callback writes (cpal's `playback - callback`), i.e. the delay a new sound has.
//!
//! `cargo run -p elora-audio --example output_latency` (Linux, needs a sound server or ALSA).

#[cfg(target_os = "linux")]
fn main() {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::{BufferSize, HostId};

    let sizes = [
        BufferSize::Default,
        BufferSize::Fixed(elora_audio::OUTPUT_BUFFER_FRAMES),
    ];
    for id in [HostId::PulseAudio, HostId::Alsa] {
        let Ok(host) = cpal::host_from_id(id) else {
            println!("{id:?}: host not available");
            continue;
        };
        let Some(device) = host.default_output_device() else {
            println!("{id:?}: no default output device");
            continue;
        };
        let Ok(default) = device.default_output_config() else {
            println!("{id:?}: no default output config");
            continue;
        };
        for size in sizes {
            let mut config = default.config();
            config.buffer_size = size;
            let lat: Arc<Mutex<Vec<Duration>>> = Arc::default();
            let sink = lat.clone();
            let stream = device.build_output_stream::<f32, _, _>(
                config,
                move |data, info| {
                    data.fill(0.0);
                    let t = info.timestamp();
                    if let Ok(mut v) = sink.lock() {
                        v.push(t.playback.duration_since(t.callback));
                    }
                },
                |e| eprintln!("stream error: {e}"),
                None,
            );
            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    println!("{id:?} {size:?}: failed: {e}");
                    continue;
                }
            };
            if let Err(e) = stream.play() {
                println!("{id:?} {size:?}: play failed: {e}");
                continue;
            }
            std::thread::sleep(Duration::from_millis(1500));
            drop(stream);
            let mut v = lat.lock().map(|v| v.clone()).unwrap_or_default();
            // skip the start-up phase while the buffer fills
            let settled = v.split_off(v.len() / 2);
            let ms = |d: Duration| d.as_secs_f64() * 1000.0;
            let max = settled.iter().copied().max().unwrap_or_default();
            let mean = settled.iter().map(|d| ms(*d)).sum::<f64>() / settled.len().max(1) as f64;
            println!(
                "{id:?} {size:?}: {} callbacks, latency mean {mean:.1} ms, max {:.1} ms",
                v.len() + settled.len(),
                ms(max)
            );
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("Only measured on Linux (PulseAudio and ALSA hosts).");
}

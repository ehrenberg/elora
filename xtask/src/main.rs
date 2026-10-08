//! Development tasks for Elora (E-039).
//!
//! Usage: `cargo xtask <command>`

mod package;
mod traffic;

use std::fmt::Write as _;
use std::process::{Command, ExitCode};

const HELP: &str = "\
Usage: cargo xtask <command>

Commands:
  check          all checks: fmt, clippy, test, deny
  fmt            format the code
  train-huffman  generate the Huffman table from synthetic traffic (E-063)
  net-stats      measure message sizes for 8/16/64 players
  svg-preview <input.svg> <output.png> [width]
                 rasterise an SVG (check drafts, M5)
  sound-preview [name …]
                 sounds (procedural + files) as WAV to target/sounds/ (listening test, M5.7)
  sound-import <name> <input> [start_s] [length_s]
                 sound file via ffmpeg to assets/sounds/files/<name>.wav (add the source to assets/SOURCES.md!)
  intro-import <video> [<video> …] [--crf N]
                 videos via ffmpeg (AV1), joined in order, to assets/intro/intro.ivf:
                 1280×720, 24 fps, no sound
                 (E-355; name the tool and licence in assets/SOURCES.md!)
  package [--archive]
                 release package in dist/ (programs, maps, licenses; macOS: Elora.app),
                 with --archive as .tar.gz or .zip (M8.3)
  map-dump <map.emap>
                 print a map in readable form: header, checksum, grid, layers (M6.2)
  help           this help
";

fn main() -> ExitCode {
    let task = std::env::args().nth(1);
    let result = match task.as_deref() {
        Some("check") => check(),
        Some("fmt") => cargo(&["fmt", "--all"]),
        Some("train-huffman") => train_huffman(),
        Some("svg-preview") => {
            let args: Vec<String> = std::env::args().skip(2).collect();
            svg_preview(&args)
        }
        Some("sound-import") => {
            let args: Vec<String> = std::env::args().skip(2).collect();
            sound_import(&args)
        }
        Some("intro-import") => {
            let args: Vec<String> = std::env::args().skip(2).collect();
            intro_import(&args)
        }
        Some("sound-preview") => {
            let args: Vec<String> = std::env::args().skip(2).collect();
            sound_preview(&args)
        }
        Some("package") => {
            let args: Vec<String> = std::env::args().skip(2).collect();
            package::package(&args)
        }
        Some("map-dump") => std::env::args().nth(2).map_or_else(
            || Err("Usage: cargo xtask map-dump <map.emap>".to_owned()),
            |p| map_dump(std::path::Path::new(&p)),
        ),
        Some("net-stats") => {
            net_stats();
            Ok(())
        }
        Some("help") | None => {
            print!("{HELP}");
            Ok(())
        }
        Some(other) => Err(format!("Unknown command `{other}`\n\n{HELP}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("xtask: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn check() -> Result<(), String> {
    let steps: [(&str, &[&str]); 4] = [
        ("Formatting", &["fmt", "--all", "--check"]),
        (
            "Clippy",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
        ),
        (
            "Tests",
            &[
                "nextest",
                "run",
                "--workspace",
                "--all-features",
                "--no-tests=pass",
            ],
        ),
        (
            "Licenses & advisories",
            // "unmaintained" only as a warning, real security vulnerabilities stay errors (E-048)
            &["deny", "check", "-W", "unmaintained"],
        ),
    ];
    for (name, args) in steps {
        println!("==> {name}");
        cargo(args)?;
    }
    println!("==> All checks passed");
    Ok(())
}

fn cargo(args: &[&str]) -> Result<(), String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(&cargo)
        .args(args)
        .status()
        .map_err(|e| format!("`{cargo}` could not be started: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`cargo {}` failed", args.join(" ")))
    }
}

const TABLE_FILE: &str = "crates/elora-protocol/src/huffman_table.rs";

/// Counts byte frequencies over traffic with 8, 16 and 64 players.
fn train_huffman() -> Result<(), String> {
    let mut freq = [0u64; 256];
    for (players, seed) in [(8, 1), (16, 2), (64, 3)] {
        let t = traffic::generate(players, seed, 3000, 3);
        for msg in t.server.iter().chain(&t.client) {
            for &b in msg {
                freq[b as usize] += 1;
            }
        }
    }
    // scale to u32
    let max = *freq.iter().max().unwrap_or(&1);
    let scale = (max / u64::from(u32::MAX / 2)).max(1);
    let values: Vec<String> = freq
        .iter()
        .map(|f| (f / scale).max(1).to_string())
        .collect();
    let mut body = String::new();
    for row in values.chunks(12) {
        let _ = writeln!(body, "    {},", row.join(", "));
    }
    let text = format!(
        "//! Byte frequencies for the static Huffman code (E-063).\n\
         //!\n\
         //! Generated with `cargo xtask train-huffman` – do not edit by hand.\n\n\
         /// Frequencies of the byte values 0..=255.\n\
         pub const FREQUENCIES: [u32; 256] = [\n{body}];\n"
    );
    std::fs::write(TABLE_FILE, text).map_err(|e| format!("{TABLE_FILE}: {e}"))?;
    println!("Table written: {TABLE_FILE} – rebuild, then `cargo xtask net-stats`");
    Ok(())
}

/// Measures average message sizes raw and compressed (different seed than in training).
fn net_stats() {
    println!(
        "| Players | Snapshot like original (delta of all fields + Huffman) | Elora raw | Elora + Huffman | vs. original | Input raw | Input + Huffman | Server→client kB/s (25 Hz) |"
    );
    println!("|---|---|---|---|---|---|---|---|");
    for players in [8, 16, 64] {
        let t = traffic::generate(players, 99, 3000, 3);
        let avg = |msgs: &[Vec<u8>], packed: bool| {
            let total: usize = msgs
                .iter()
                .map(|m| {
                    if packed {
                        elora_protocol::msg::pack(m.clone()).len()
                    } else {
                        m.len()
                    }
                })
                .sum();
            total as f64 / msgs.len().max(1) as f64
        };
        let (sr, sp) = (avg(&t.server, false), avg(&t.server, true));
        let orig = avg(&t.original, true);
        let (cr, cp) = (avg(&t.client, false), avg(&t.client, true));
        // per client: one snapshot every 2 ticks, plus approx. 40 bytes header/encryption
        let kbps = (sp + 40.0) * 25.0 / 1000.0;
        println!(
            "| {players} | {orig:.0} B | {sr:.0} B | {sp:.0} B | {:+.0} % | {cr:.0} B | {cp:.0} B | {kbps:.1} |",
            (sp / orig - 1.0) * 100.0
        );
    }
}

/// Rasterises an SVG to PNG (resvg), width optional (height proportional).
/// Bank as in the game, but fresh from disk (changes audible without a rebuild).
fn load_bank() -> Result<elora_audio::Bank, String> {
    let src = std::fs::read_to_string("assets/sounds/sounds.toml")
        .map_err(|e| format!("assets/sounds/sounds.toml: {e}"))?;
    let mut bank = elora_audio::Bank::parse(&src).map_err(|e| e.to_string())?;
    let dir = std::path::Path::new(SOUND_FILES);
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("{SOUND_FILES}: {e}"))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "wav"))
        .collect();
    files.sort();
    for path in files {
        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_owned();
        let data = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        bank.add_file(&name, &data).map_err(|e| e.to_string())?;
    }
    Ok(bank)
}

const SOUND_FILES: &str = "assets/sounds/files";

/// Writes all (or the named) sounds as WAV to `target/sounds/`.
fn sound_preview(names: &[String]) -> Result<(), String> {
    let bank = load_bank()?;
    let dir = std::path::Path::new("target/sounds");
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut count = 0;
    for (sound, src) in &bank.sounds {
        if !names.is_empty() && !names.iter().any(|n| n == sound.name()) {
            continue;
        }
        let path = dir.join(format!("{}.wav", sound.name()));
        std::fs::write(&path, elora_audio::wav(&src.samples()))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let kind = if src.is_file() { "file" } else { "procedural" };
        println!("{} ({:.2} s, {kind})", path.display(), src.duration());
        count += 1;
    }
    for s in bank.missing() {
        println!("missing: {}", s.name());
    }
    if count == 0 {
        return Err("no matching sounds".into());
    }
    Ok(())
}

/// `sound-import <name> <input> [start_s] [length_s]`: sound file (any format that
/// ffmpeg reads) to `assets/sounds/files/<name>.wav` – mono, 44.1 kHz, 16 bit,
/// silence at the start removed, 15 ms fade-out at the end, peak at −1 dB.
/// Converts the intro video for the game (I-4, E-355): AV1 in IVF, 1280×720 with black bars
/// if needed, 24 fps, BT.709, no audio (the game plays the Tauwinkel music, E-357).
fn intro_import(args: &[String]) -> Result<(), String> {
    let usage = "Usage: cargo xtask intro-import <video> [<video> …] [--crf N]";
    let crf_at = args.iter().position(|a| a == "--crf");
    let crf = match crf_at {
        Some(i) => args.get(i + 1).ok_or(usage)?.clone(),
        None => "36".to_owned(),
    };
    let inputs: Vec<&String> = args
        .iter()
        .enumerate()
        .filter(|(i, _)| crf_at.is_none_or(|c| *i != c && *i != c + 1))
        .map(|(_, a)| a)
        .collect();
    if inputs.is_empty() {
        return Err(usage.into());
    }
    let encoders = Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
        .map_err(|e| format!("ffmpeg could not be started: {e}"))?;
    let list = String::from_utf8_lossy(&encoders.stdout);
    let encoder = ["libsvtav1", "libaom-av1"]
        .into_iter()
        .find(|e| list.contains(e))
        .ok_or("ffmpeg has no AV1 encoder (libsvtav1 or libaom-av1)")?;
    let out = std::path::Path::new("assets/intro/intro.ivf");
    std::fs::create_dir_all("assets/intro").map_err(|e| e.to_string())?;
    // every clip letterboxed to 1280×720 at 24 fps, then all joined in order
    let mut filter = String::new();
    for i in 0..inputs.len() {
        let _ = write!(
            filter,
            "[{i}:v]scale=1280:720:force_original_aspect_ratio=decrease,\
             pad=1280:720:(ow-iw)/2:(oh-ih)/2,fps=24,format=yuv420p,setsar=1[v{i}];"
        );
    }
    for i in 0..inputs.len() {
        let _ = write!(filter, "[v{i}]");
    }
    let _ = write!(filter, "concat=n={}:v=1:a=0[out]", inputs.len());
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-v", "error", "-y"]);
    for input in &inputs {
        cmd.args(["-i", input.as_str()]);
    }
    cmd.args(["-filter_complex", &filter, "-map", "[out]", "-an"]);
    cmd.args(["-c:v", encoder, "-crf", &crf]);
    if encoder == "libsvtav1" {
        cmd.args(["-preset", "5", "-g", "240"]);
    } else {
        cmd.args(["-cpu-used", "4", "-g", "240"]);
    }
    cmd.args([
        "-colorspace",
        "bt709",
        "-color_primaries",
        "bt709",
        "-color_trc",
        "bt709",
        "-color_range",
        "tv",
        "-f",
        "ivf",
    ]);
    cmd.arg(out);
    let status = cmd
        .status()
        .map_err(|e| format!("ffmpeg could not be started: {e}"))?;
    if !status.success() {
        return Err("ffmpeg failed".into());
    }
    let data = std::fs::read(out).map_err(|e| e.to_string())?;
    let header = elora_video::Header::parse(&data).map_err(|e| e.to_string())?;
    // decode once completely: the game must be able to play it
    let mut video = elora_video::Video::new(data.clone()).map_err(|e| e.to_string())?;
    let mut frames = 0u32;
    while video.next_frame().map_err(|e| e.to_string())?.is_some() {
        frames += 1;
    }
    #[allow(clippy::cast_precision_loss)]
    let mb = data.len() as f64 / 1_048_576.0;
    println!(
        "{}: {}×{}, {frames} frames, {:.1} s, {mb:.1} MB ({encoder}, crf {crf})",
        out.display(),
        header.width,
        header.height,
        f64::from(frames) / f64::from(header.fps()),
    );
    Ok(())
}

fn sound_import(args: &[String]) -> Result<(), String> {
    let usage = "Usage: cargo xtask sound-import <name> <input> [start_s] [length_s]";
    let (Some(name), Some(input)) = (args.first(), args.get(1)) else {
        return Err(usage.into());
    };
    if elora_audio::Sound::from_name(name).is_none() {
        return Err(format!(
            "unknown sound `{name}` (see crates/elora-audio/src/cues.rs)"
        ));
    }
    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.args(["-v", "error"]);
    if let Some(start) = args.get(2) {
        cmd.args(["-ss", start]);
    }
    cmd.args(["-i", input]);
    if let Some(len) = args.get(3) {
        cmd.args(["-t", len]);
    }
    cmd.args(["-ac", "1", "-ar", "44100", "-f", "f32le", "-"]);
    let out = cmd
        .output()
        .map_err(|e| format!("ffmpeg could not be started: {e}"))?;
    if !out.status.success() {
        return Err(format!("ffmpeg: {}", String::from_utf8_lossy(&out.stderr)));
    }
    let mut samples: Vec<f32> = out
        .stdout
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect();
    // remove silence at the start (sounds should start immediately); threshold relative
    // to the peak, so that quiet recordings and swelling sounds are kept
    let raw_peak = samples.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
    let first = samples
        .iter()
        .position(|s| s.abs() > raw_peak * 0.01)
        .unwrap_or(0);
    samples.drain(..first);
    let fade = elora_audio::SAMPLE_RATE as usize * 15 / 1000; // 15 ms
    let len = samples.len();
    for (i, s) in samples
        .iter_mut()
        .enumerate()
        .skip(len.saturating_sub(fade))
    {
        #[allow(clippy::cast_precision_loss)]
        let t = (len - i) as f32 / fade as f32;
        *s *= t;
    }
    let peak = samples.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
    if peak <= 0.0 {
        return Err("input is silent".into());
    }
    let target = 0.89; // −1 dBFS
    for s in &mut samples {
        *s *= target / peak;
    }
    let path = std::path::Path::new(SOUND_FILES).join(format!("{name}.wav"));
    std::fs::write(&path, elora_audio::wav(&samples))
        .map_err(|e| format!("{}: {e}", path.display()))?;
    #[allow(clippy::cast_precision_loss)]
    let secs = samples.len() as f32 / elora_audio::SAMPLE_RATE as f32;
    println!("{} ({secs:.2} s)", path.display());
    Ok(())
}

fn svg_preview(args: &[String]) -> Result<(), String> {
    let [input, output, rest @ ..] = args else {
        return Err("Usage: cargo xtask svg-preview <input.svg> <output.png> [width]".into());
    };
    let data = std::fs::read(input).map_err(|e| format!("{input}: {e}"))?;
    let mut opt = resvg::usvg::Options::default();
    // own font instead of system fonts (reproducible)
    if let Ok(font) = std::fs::read("assets/fonts/Inter-Regular.ttf") {
        opt.fontdb_mut().load_font_data(font);
    }
    opt.font_family = "Inter".into();
    let tree = resvg::usvg::Tree::from_data(&data, &opt).map_err(|e| format!("{input}: {e}"))?;
    let size = tree.size();
    let width: f32 = match rest.first() {
        Some(w) => w
            .parse()
            .map_err(|_| "width must be a number".to_string())?,
        None => size.width(),
    };
    let scale = width / size.width();
    #[allow(clippy::cast_sign_loss)]
    let (w, h) = (
        (size.width() * scale).ceil() as u32,
        (size.height() * scale).ceil() as u32,
    );
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).ok_or("invalid size")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap
        .save_png(output)
        .map_err(|e| format!("{output}: {e}"))?;
    println!("{output} ({w}×{h})");
    Ok(())
}

/// Prints a map in readable form (M6.2): replacement for the removed text format when
/// checking maps.
fn map_dump(path: &std::path::Path) -> Result<(), String> {
    let data = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let map = elora_map::decode(&data).map_err(|e| format!("{}: {e}", path.display()))?;
    let sum: String = elora_map::checksum(&data)
        .iter()
        .fold(String::new(), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        });
    println!("Name:       {}", map.name);
    println!("Author:     {}", map.author.as_deref().unwrap_or("–"));
    println!(
        "Size:       {} × {} tiles, {} bytes",
        map.width,
        map.height,
        data.len()
    );
    println!("Checksum:   {sum}");
    let modes = map.supported_modes();
    println!(
        "Modes:      DM/LMS {} · TDM/LTS {} · CTF {}",
        yes(modes.free_for_all),
        yes(modes.team),
        yes(modes.ctf)
    );
    println!("Materials:  {:?}", map.materials);
    println!(
        "Sky:        #{} → #{}",
        hex(map.sky.top.0),
        hex(map.sky.bottom.0)
    );
    for b in &map.backgrounds {
        println!(
            "Background \"{}\": parallax {:?}, {} objects",
            b.name,
            (b.parallax.x, b.parallax.y),
            b.items.len()
        );
    }
    println!(
        "Decor:      {} back, {} front",
        map.decor_back.len(),
        map.decor_front.len()
    );
    for o in &map.adventure.objects {
        println!(
            "Adventure:  {} \"{}\" at ({:.0}, {:.0})",
            o.kind.name(),
            o.id,
            o.pos.x,
            o.pos.y
        );
    }
    for e in &map.envelopes {
        println!(
            "Animation \"{}\": {:?}, {} points, {} ms",
            e.name,
            e.kind,
            e.points.len(),
            e.duration_ms()
        );
    }
    for i in &map.images {
        println!("Image \"{}\": {} bytes", i.name, i.svg.len());
    }
    println!();
    for row in map.to_rows() {
        println!("{row}");
    }
    Ok(())
}

fn yes(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

fn hex(c: [u8; 4]) -> String {
    format!("{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

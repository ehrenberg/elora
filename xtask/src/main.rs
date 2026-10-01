//! Entwicklungsaufgaben für Elora (E-039).
//!
//! Aufruf: `cargo xtask <befehl>`

mod traffic;

use std::fmt::Write as _;
use std::process::{Command, ExitCode};

const HELP: &str = "\
Verwendung: cargo xtask <befehl>

Befehle:
  check          Alle Prüfungen: fmt, clippy, test, deny
  fmt            Code formatieren
  train-huffman  Huffman-Tabelle aus synthetischem Verkehr erzeugen (E-063)
  net-stats      Nachrichtengrößen für 8/16/64 Spieler messen
  svg-preview <eingabe.svg> <ausgabe.png> [breite]
                 SVG rastern (Entwürfe prüfen, M5)
  sound-preview [name …]
                 Sounds (prozedural + Dateien) als WAV nach target/sounds/ (Hörprobe, M5.7)
  sound-import <name> <eingabe> [start_s] [länge_s]
                 Tondatei per ffmpeg nach assets/sounds/files/<name>.wav (Quelle in assets/SOURCES.md eintragen!)
  map-dump <karte.emap>
                 Karte lesbar ausgeben: Kopf, Prüfsumme, Raster, Ebenen (M6.2)
  help           Diese Hilfe
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
        Some("sound-preview") => {
            let args: Vec<String> = std::env::args().skip(2).collect();
            sound_preview(&args)
        }
        Some("map-dump") => std::env::args().nth(2).map_or_else(
            || Err("Verwendung: cargo xtask map-dump <karte.emap>".to_owned()),
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
        Some(other) => Err(format!("Unbekannter Befehl `{other}`\n\n{HELP}")),
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
        ("Formatierung", &["fmt", "--all", "--check"]),
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
            "Lizenzen & Advisories",
            // „unmaintained“ nur als Warnung, echte Sicherheitslücken bleiben Fehler (E-048)
            &["deny", "check", "-W", "unmaintained"],
        ),
    ];
    for (name, args) in steps {
        println!("==> {name}");
        cargo(args)?;
    }
    println!("==> Alle Prüfungen bestanden");
    Ok(())
}

fn cargo(args: &[&str]) -> Result<(), String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(&cargo)
        .args(args)
        .status()
        .map_err(|e| format!("`{cargo}` konnte nicht gestartet werden: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`cargo {}` fehlgeschlagen", args.join(" ")))
    }
}

const TABLE_FILE: &str = "crates/elora-protocol/src/huffman_table.rs";

/// Zählt Byte-Häufigkeiten über Verkehr mit 8, 16 und 64 Spielern.
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
    // auf u32 skalieren
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
        "//! Byte-Häufigkeiten für den statischen Huffman-Code (E-063).\n\
         //!\n\
         //! Erzeugt mit `cargo xtask train-huffman` – nicht von Hand ändern.\n\n\
         /// Häufigkeiten der Byte-Werte 0..=255.\n\
         pub const FREQUENCIES: [u32; 256] = [\n{body}];\n"
    );
    std::fs::write(TABLE_FILE, text).map_err(|e| format!("{TABLE_FILE}: {e}"))?;
    println!("Tabelle geschrieben: {TABLE_FILE} – neu bauen, dann `cargo xtask net-stats`");
    Ok(())
}

/// Misst durchschnittliche Nachrichtengrößen roh und komprimiert (anderer Seed als beim Training).
fn net_stats() {
    println!(
        "| Spieler | Snapshot wie Original (Delta aller Felder + Huffman) | Elora roh | Elora + Huffman | vs. Original | Eingabe roh | Eingabe + Huffman | Server→Client kB/s (25 Hz) |"
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
        // Pro Client: ein Snapshot alle 2 Ticks, zzgl. ca. 40 Byte Kopf/Verschlüsselung
        let kbps = (sp + 40.0) * 25.0 / 1000.0;
        println!(
            "| {players} | {orig:.0} B | {sr:.0} B | {sp:.0} B | {:+.0} % | {cr:.0} B | {cp:.0} B | {kbps:.1} |",
            (sp / orig - 1.0) * 100.0
        );
    }
}

/// Rastert ein SVG zu PNG (resvg), Breite optional (Höhe proportional).
/// Bank wie im Spiel, aber frisch von der Platte (Änderungen ohne Neubau hörbar).
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

/// Schreibt alle (oder die genannten) Sounds als WAV nach `target/sounds/`.
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
        let kind = if src.is_file() { "Datei" } else { "prozedural" };
        println!("{} ({:.2} s, {kind})", path.display(), src.duration());
        count += 1;
    }
    for s in bank.missing() {
        println!("fehlt: {}", s.name());
    }
    if count == 0 {
        return Err("keine passenden Sounds".into());
    }
    Ok(())
}

/// `sound-import <name> <eingabe> [start_s] [länge_s]`: Tondatei (jedes Format, das
/// ffmpeg liest) nach `assets/sounds/files/<name>.wav` – Mono, 44,1 kHz, 16 Bit,
/// Stille am Anfang entfernt, 15 ms Ausblenden am Ende, Spitze auf −1 dB.
fn sound_import(args: &[String]) -> Result<(), String> {
    let usage = "Verwendung: cargo xtask sound-import <name> <eingabe> [start_s] [länge_s]";
    let (Some(name), Some(input)) = (args.first(), args.get(1)) else {
        return Err(usage.into());
    };
    if elora_audio::Sound::from_name(name).is_none() {
        return Err(format!(
            "unbekannter Sound `{name}` (siehe crates/elora-audio/src/cues.rs)"
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
        .map_err(|e| format!("ffmpeg nicht startbar: {e}"))?;
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
    // Stille am Anfang entfernen (Sounds sollen sofort einsetzen); Schwelle relativ zur
    // Spitze, damit leise Aufnahmen und anschwellende Klänge erhalten bleiben
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
        return Err("Eingabe ist stumm".into());
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
        return Err(
            "Verwendung: cargo xtask svg-preview <eingabe.svg> <ausgabe.png> [breite]".into(),
        );
    };
    let data = std::fs::read(input).map_err(|e| format!("{input}: {e}"))?;
    let mut opt = resvg::usvg::Options::default();
    // eigene Schrift statt Systemschriften (reproduzierbar)
    if let Ok(font) = std::fs::read("assets/fonts/Inter-Regular.ttf") {
        opt.fontdb_mut().load_font_data(font);
    }
    opt.font_family = "Inter".into();
    let tree = resvg::usvg::Tree::from_data(&data, &opt).map_err(|e| format!("{input}: {e}"))?;
    let size = tree.size();
    let width: f32 = match rest.first() {
        Some(w) => w
            .parse()
            .map_err(|_| "Breite muss eine Zahl sein".to_string())?,
        None => size.width(),
    };
    let scale = width / size.width();
    #[allow(clippy::cast_sign_loss)]
    let (w, h) = (
        (size.width() * scale).ceil() as u32,
        (size.height() * scale).ceil() as u32,
    );
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).ok_or("ungültige Größe")?;
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

/// Karte lesbar ausgeben (M6.2): Ersatz für das entfernte Textformat beim Prüfen von Karten.
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
    println!("Autor:      {}", map.author.as_deref().unwrap_or("–"));
    println!(
        "Größe:      {} × {} Tiles, {} Bytes",
        map.width,
        map.height,
        data.len()
    );
    println!("Prüfsumme:  {sum}");
    let modes = map.supported_modes();
    println!(
        "Modi:       DM/LMS {} · TDM/LTS {} · CTF {}",
        yes(modes.free_for_all),
        yes(modes.team),
        yes(modes.ctf)
    );
    println!("Materialien: {:?}", map.materials);
    println!(
        "Himmel:     #{} → #{}",
        hex(map.sky.top.0),
        hex(map.sky.bottom.0)
    );
    for b in &map.backgrounds {
        println!(
            "Hintergrund „{}“: Parallax {:?}, {} Objekte",
            b.name,
            (b.parallax.x, b.parallax.y),
            b.items.len()
        );
    }
    println!(
        "Deko:       {} hinten, {} vorn",
        map.decor_back.len(),
        map.decor_front.len()
    );
    for e in &map.envelopes {
        println!(
            "Animation „{}“: {:?}, {} Punkte, {} ms",
            e.name,
            e.kind,
            e.points.len(),
            e.duration_ms()
        );
    }
    for i in &map.images {
        println!("Bild „{}“: {} Bytes", i.name, i.svg.len());
    }
    println!();
    for row in map.to_rows() {
        println!("{row}");
    }
    Ok(())
}

fn yes(b: bool) -> &'static str {
    if b { "ja" } else { "nein" }
}

fn hex(c: [u8; 4]) -> String {
    format!("{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

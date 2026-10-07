//! Release-Pakete bauen (M8.3, E-163): `cargo xtask package [--archive]`.
//!
//! Baut `elora`, `elora-server` und `elora-master` im Release-Profil und legt unter `dist/`
//! einen Paketordner an:
//!
//! ```text
//! elora-<version>-<system>-<arch>/
//!   elora, elora-server, elora-master   Programme
//!   maps/                               Release-Karten + Trainingskarte, abenteuer/ mit den Abenteuer-Karten
//!   assets/music/                       Musik (Menü, Gebiete)
//!   assets/ambience/                    Wetterklänge (Regen, Wind, Sand, Donner)
//!   LICENSE, THIRD_PARTY_LICENSES, SOURCES.md, LIESMICH.txt
//!   elora.png                           Programmsymbol (256 × 256)
//! ```
//!
//! Unter macOS entsteht zusätzlich `Elora.app` (Daten in `Contents/Resources`). `--archive`
//! packt den Ordner als `.tar.gz` (Linux, macOS) bzw. `.zip` (Windows). `AppImage` und DMG
//! baut der GitHub-Workflow aus diesem Ordner.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Karten, die ausgeliefert werden (Testkarten bleiben im Repository).
pub const SHIPPED_MAPS: [&str; 6] = [
    "dm-wiese",
    "dm-wueste",
    "dm-winter",
    "ctf-wald",
    "ctf-nacht",
    "sandbox",
];

const BINARIES: [&str; 3] = ["elora", "elora-server", "elora-master"];

fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

fn platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn copy_dir(from: &Path, to: &Path, filter: &dyn Fn(&Path) -> bool) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    for entry in std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() && filter(&path) {
            std::fs::copy(&path, to.join(entry.file_name()))
                .map_err(|e| format!("{}: {e}", path.display()))?;
        }
    }
    Ok(())
}

fn copy(from: &str, to: &Path) -> Result<(), String> {
    std::fs::copy(from, to).map_err(|e| format!("{from} → {}: {e}", to.display()))?;
    Ok(())
}

/// Programmsymbol aus der Figur (quadratisch, transparent).
fn render_icon(out: &Path) -> Result<(), String> {
    let data = std::fs::read("assets/elora/elora.svg").map_err(|e| format!("Figur: {e}"))?;
    let tree = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default())
        .map_err(|e| format!("Figur: {e}"))?;
    let size = tree.size();
    let edge = 256.0_f32;
    let scale = edge * 0.9 / size.width().max(size.height());
    let dx = (edge - size.width() * scale) / 2.0;
    let dy = (edge - size.height() * scale) / 2.0;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(256, 256).ok_or("ungültige Größe")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_row(scale, 0.0, 0.0, scale, dx, dy),
        &mut pixmap.as_mut(),
    );
    pixmap
        .save_png(out)
        .map_err(|e| format!("{}: {e}", out.display()))
}

const README: &str = "Elora {version}\n\
=====================\n\n\
Ein schnelles 2D-Multiplayer-Spiel mit Hook, Hammer, Granate und Laser.\n\n\
Starten:   elora             (Hauptmenü)\n\
Server:    elora-server --port 8303 --map maps/dm-wiese.emap --name \"Mein Server\"\n\
Master:    elora-master      (Server-Liste fürs Internet, siehe docs im Repository)\n\n\
Einstellungen und eigene Karten liegen im Benutzerverzeichnis\n\
(Linux ~/.config/elora und ~/.local/share/elora, Windows %APPDATA%\\Elora,\n\
macOS ~/Library/Application Support/Elora).\n\n\
Startet Elora nicht, stehen dort elora.log und crash.txt mit dem Grund.\n\
If Elora does not start, elora.log and crash.txt in that folder tell why.\n\n\
macOS: Das Programm ist nicht signiert. Beim ersten Start mit Rechtsklick →\n\
„Öffnen“ starten und bestätigen.\n\n\
Lizenzen: Code GPL-3.0 (LICENSE), eigene Grafiken und Sounds CC-BY-SA 4.0,\n\
fremde Assets siehe SOURCES.md, Bibliotheken siehe THIRD_PARTY_LICENSES.\n\
Quelltext: https://github.com/ehrenberg/elora\n";

/// Paketordner füllen (Programme, Daten, Lizenzen).
fn fill(dir: &Path, bin_dir: &Path, data_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(bin_dir).map_err(|e| e.to_string())?;
    for b in BINARIES {
        let name = format!("{b}{}", std::env::consts::EXE_SUFFIX);
        copy(&format!("target/release/{name}"), &bin_dir.join(&name))?;
    }
    copy_dir(Path::new("maps"), &data_dir.join("maps"), &|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| SHIPPED_MAPS.contains(&s))
    })?;
    // Abenteuer-Karten (E-262)
    copy_dir(
        Path::new("maps/abenteuer"),
        &data_dir.join("maps/abenteuer"),
        &|p| p.extension().is_some_and(|e| e == "emap"),
    )?;
    for dir in ["assets/music", "assets/ambience"] {
        if Path::new(dir).is_dir() {
            copy_dir(Path::new(dir), &data_dir.join(dir), &|_| true)?;
        }
    }
    for f in ["LICENSE", "THIRD_PARTY_LICENSES"] {
        copy(f, &dir.join(f))?;
    }
    copy("assets/SOURCES.md", &dir.join("SOURCES.md"))?;
    std::fs::write(
        dir.join("LIESMICH.txt"),
        README.replace("{version}", &version()),
    )
    .map_err(|e| e.to_string())?;
    render_icon(&dir.join("elora.png"))
}

/// macOS-Bundle `Elora.app` mit Programmen in `MacOS` und Daten in `Resources`.
fn mac_bundle(dist: &Path) -> Result<PathBuf, String> {
    let app = dist.join("Elora.app");
    let _ = std::fs::remove_dir_all(&app);
    let contents = app.join("Contents");
    fill(
        &contents.join("Resources"),
        &contents.join("MacOS"),
        &contents.join("Resources"),
    )?;
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>Elora</string>
  <key>CFBundleDisplayName</key><string>Elora</string>
  <key>CFBundleIdentifier</key><string>de.bastianswelt.elora</string>
  <key>CFBundleVersion</key><string>{v}</string>
  <key>CFBundleShortVersionString</key><string>{v}</string>
  <key>CFBundleExecutable</key><string>elora</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
</dict></plist>
"#,
        v = version()
    );
    std::fs::write(contents.join("Info.plist"), plist).map_err(|e| e.to_string())?;
    Ok(app)
}

fn run(cmd: &mut Command) -> Result<(), String> {
    let shown = format!("{cmd:?}");
    let status = cmd
        .status()
        .map_err(|e| format!("{shown} konnte nicht starten: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{shown} fehlgeschlagen"))
    }
}

/// `cargo xtask package [--archive]`
pub fn package(args: &[String]) -> Result<(), String> {
    let archive = args.iter().any(|a| a == "--archive");
    let mut build = vec!["build", "--release"];
    for b in BINARIES {
        build.extend(["--bin", b]);
    }
    super::cargo(&build)?;
    let dist = PathBuf::from("dist");
    let name = format!("elora-{}-{}", version(), platform());
    let dir = dist.join(&name);
    let _ = std::fs::remove_dir_all(&dir);
    fill(&dir, &dir, &dir)?;
    println!("Paket: {}", dir.display());
    if cfg!(target_os = "macos") {
        let app = mac_bundle(&dist)?;
        println!("Bundle: {}", app.display());
    }
    if archive {
        let file = if cfg!(windows) {
            format!("{name}.zip")
        } else {
            format!("{name}.tar.gz")
        };
        let flags = if cfg!(windows) { "-a -cf" } else { "-czf" };
        let mut tar = Command::new("tar");
        tar.current_dir(&dist);
        tar.args(flags.split(' ')).arg(&file).arg(&name);
        run(&mut tar)?;
        println!("Archiv: {}", dist.join(file).display());
    }
    Ok(())
}

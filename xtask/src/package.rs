//! Build release packages (M8.3, E-163): `cargo xtask package [--archive]`.
//!
//! Builds `elora`, `elora-server` and `elora-master` in the release profile and creates a
//! package folder under `dist/`:
//!
//! ```text
//! elora-<version>-<system>-<arch>/
//!   elora, elora-server, elora-master   programs
//!   maps/                               release + training maps, abenteuer/: adventure maps
//!   assets/music/                       music (menu, regions)
//!   assets/ambience/                    weather sounds (rain, wind, sand, thunder)
//!   LICENSE, THIRD_PARTY_LICENSES, SOURCES.md, LIESMICH.txt
//!   elora.png                           program icon (256 × 256)
//! ```
//!
//! On macOS, `Elora.app` is created as well (data in `Contents/Resources`). `--archive`
//! packs the folder as `.tar.gz` (Linux, macOS) or `.zip` (Windows). The GitHub workflow
//! builds `AppImage` and DMG from this folder.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Maps that are shipped (test maps stay in the repository).
pub const SHIPPED_MAPS: [&str; 7] = [
    // the client starts with the training map (0.9.1 shipped without it and did not start)
    "training",
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

/// What the package is built for: the host or a cross target (`--target <triple>`, e.g.
/// `x86_64-pc-windows-gnu` to build a Windows package on Linux with mingw-w64).
struct Target {
    triple: Option<String>,
}

impl Target {
    fn windows(&self) -> bool {
        self.triple
            .as_deref()
            .map_or(cfg!(windows), |t| t.contains("windows"))
    }

    fn exe_suffix(&self) -> &'static str {
        if self.windows() { ".exe" } else { "" }
    }

    /// Folder with the built programs.
    fn bin_source(&self) -> PathBuf {
        match &self.triple {
            Some(t) => PathBuf::from("target").join(t).join("release"),
            None => PathBuf::from("target/release"),
        }
    }

    /// `<os>-<arch>` for the package name.
    fn platform(&self) -> String {
        match &self.triple {
            None => format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            Some(t) => {
                let arch = t.split('-').next().unwrap_or("unknown");
                let os = if self.windows() {
                    "windows"
                } else if t.contains("darwin") {
                    "macos"
                } else {
                    "linux"
                };
                format!("{os}-{arch}")
            }
        }
    }
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

/// Program icon from the character (square, transparent).
fn render_icon(out: &Path) -> Result<(), String> {
    let data = std::fs::read("assets/elora/elora.svg").map_err(|e| format!("character: {e}"))?;
    let tree = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default())
        .map_err(|e| format!("character: {e}"))?;
    let size = tree.size();
    let edge = 256.0_f32;
    let scale = edge * 0.9 / size.width().max(size.height());
    let dx = (edge - size.width() * scale) / 2.0;
    let dy = (edge - size.height() * scale) / 2.0;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(256, 256).ok_or("invalid size")?;
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
A fast 2D multiplayer game with hook, hammer, grenade launcher and laser.\n\n\
Start:     elora             (main menu)\n\
Server:    elora-server --port 8303 --map maps/dm-wiese.emap --name \"My server\"\n\
Master:    elora-master      (server list for the internet, see the docs in the repository)\n\n\
Settings and your own maps live in your user folder\n\
(Linux ~/.config/elora and ~/.local/share/elora, Windows %APPDATA%\\Elora,\n\
macOS ~/Library/Application Support/Elora).\n\n\
If Elora does not start, elora.log and crash.txt in that folder tell why.\n\
Startet Elora nicht, stehen dort elora.log und crash.txt mit dem Grund.\n\n\
macOS: the app is not signed. On the first start, right click it, choose\n\
\"Open\" and confirm.\n\n\
Licences: code GPL-3.0 (LICENSE), own graphics and sounds CC-BY-SA 4.0,\n\
third-party assets see SOURCES.md, libraries see THIRD_PARTY_LICENSES.\n\
Source: https://github.com/ehrenberg/elora\n";

/// Fills the package folder (programs, data, licences).
fn fill(target: &Target, dir: &Path, bin_dir: &Path, data_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(bin_dir).map_err(|e| e.to_string())?;
    for b in BINARIES {
        let name = format!("{b}{}", target.exe_suffix());
        let from = target.bin_source().join(&name);
        copy(&from.to_string_lossy(), &bin_dir.join(&name))?;
    }
    copy_dir(Path::new("maps"), &data_dir.join("maps"), &|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| SHIPPED_MAPS.contains(&s))
    })?;
    // adventure maps (E-262)
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

/// macOS bundle `Elora.app` with programs in `MacOS` and data in `Resources`.
fn mac_bundle(target: &Target, dist: &Path) -> Result<PathBuf, String> {
    let app = dist.join("Elora.app");
    let _ = std::fs::remove_dir_all(&app);
    let contents = app.join("Contents");
    fill(
        target,
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
        .map_err(|e| format!("{shown} could not start: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{shown} failed"))
    }
}

/// `cargo xtask package [--archive] [--target <triple>]`
pub fn package(args: &[String]) -> Result<(), String> {
    let archive = args.iter().any(|a| a == "--archive");
    let target = Target {
        triple: args
            .iter()
            .position(|a| a == "--target")
            .and_then(|i| args.get(i + 1))
            .cloned(),
    };
    let mut build = vec!["build", "--release"];
    if let Some(t) = &target.triple {
        build.extend(["--target", t.as_str()]);
    }
    for b in BINARIES {
        build.extend(["--bin", b]);
    }
    super::cargo(&build)?;
    let dist = PathBuf::from("dist");
    let name = format!("elora-{}-{}", version(), target.platform());
    let dir = dist.join(&name);
    let _ = std::fs::remove_dir_all(&dir);
    fill(&target, &dir, &dir, &dir)?;
    println!("Package: {}", dir.display());
    if cfg!(target_os = "macos") && target.triple.is_none() {
        let app = mac_bundle(&target, &dist)?;
        println!("Bundle: {}", app.display());
    }
    if archive {
        let file = if target.windows() {
            format!("{name}.zip")
        } else {
            format!("{name}.tar.gz")
        };
        let flags = if target.windows() { "-a -cf" } else { "-czf" };
        // GNU tar cannot write ZIP files: cross-building for Windows needs bsdtar
        let program = if target.windows() && !cfg!(windows) {
            "bsdtar"
        } else {
            "tar"
        };
        let mut tar = Command::new(program);
        tar.current_dir(&dist);
        tar.args(flags.split(' ')).arg(&file).arg(&name);
        run(&mut tar)?;
        println!("Archive: {}", dist.join(file).display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SHIPPED_MAPS;

    /// The map the client starts with must be in every package.
    #[test]
    fn client_start_map_is_shipped() {
        let main = include_str!("../../apps/elora-client/src/main.rs");
        let line = main
            .lines()
            .find(|l| l.contains("const DEFAULT_MAP: &str"))
            .expect("DEFAULT_MAP in the client");
        let stem = line
            .split("maps/")
            .nth(1)
            .and_then(|r| r.split(".emap").next())
            .expect("DEFAULT_MAP = \"maps/<name>.emap\"");
        assert!(SHIPPED_MAPS.contains(&stem), "{stem} is not shipped");
    }
}

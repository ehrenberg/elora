//! Embeds all `assets/sounds/files/*.wav` (CC0 sounds, E-107). If a file exists for a
//! sound, it replaces the procedural sound from `sounds.toml`.

use std::fmt::Write as _;
use std::path::Path;

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/sounds/files");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut entries: Vec<(String, String)> = std::fs::read_dir(&dir)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "wav"))
                .filter_map(|p| {
                    let name = p.file_stem()?.to_str()?.to_owned();
                    let path = p.canonicalize().ok()?.to_str()?.to_owned();
                    Some((name, path))
                })
                .collect()
        })
        .unwrap_or_default();
    entries.sort();
    let mut out = String::from(
        "/// Embedded sound files: (sound name, WAV data).\npub static FILES: &[(&str, &[u8])] = &[\n",
    );
    for (name, path) in &entries {
        println!("cargo:rerun-if-changed={path}");
        let _ = writeln!(out, "    ({name:?}, include_bytes!({path:?})),");
    }
    out.push_str("];\n");
    let target = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("files.rs");
    std::fs::write(target, out).expect("files.rs writable");
}

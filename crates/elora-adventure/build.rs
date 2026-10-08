//! Embeds all `assets/adventure/dialogs/*.toml` (RF-25): every file is a dialog with its file
//! name as id, so a new dialog only needs its file. The list lands in `$OUT_DIR/dialogs.rs`.

use std::fmt::Write as _;
use std::path::Path;

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/adventure/dialogs");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut entries: Vec<(String, String)> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .filter_map(|p| {
            let id = p.file_stem()?.to_str()?.to_owned();
            let path = p.canonicalize().ok()?.to_str()?.to_owned();
            Some((id, path))
        })
        .collect();
    entries.sort();
    let mut out = String::from(
        "/// Bundled dialogs (`assets/adventure/dialogs/<id>.toml`): (id, TOML text).\n\
         const DIALOG_FILES: &[(&str, &str)] = &[\n",
    );
    for (id, path) in &entries {
        println!("cargo:rerun-if-changed={path}");
        let _ = writeln!(out, "    ({id:?}, include_str!({path:?})),");
    }
    out.push_str("];\n");
    let target = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("dialogs.rs");
    std::fs::write(target, out).expect("dialogs.rs writable");
}

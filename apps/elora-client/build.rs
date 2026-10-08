//! Generates the lists of embedded SVG graphics from the asset folders (RF-25): every
//! `*.svg` in a folder below is embedded under its file name without extension, so a new
//! graphic only needs its file. The lists land in `$OUT_DIR/asset_files.rs`
//! (see `src/asset_files.rs`).

use std::fmt::Write as _;
use std::path::Path;

/// Folder below `assets/`, name of the generated constant and its doc comment.
const LISTS: [(&str, &str, &str); 6] = [
    (
        "map/decor",
        "DECOR_FILES",
        "Built-in decoration (`assets/map/decor/`, names for `Art::Builtin`).",
    ),
    (
        "map/backgrounds",
        "BACKGROUND_FILES",
        "Built-in background graphics (`assets/map/backgrounds/`, also used via `Art::Builtin`).",
    ),
    (
        "adventure/creatures",
        "CREATURE_FILES",
        "Enemies (`assets/adventure/creatures/`), named like their creature kind.",
    ),
    (
        "adventure/characters",
        "CHARACTER_FILES",
        "NPC figures (`assets/adventure/characters/`), named like their character id.",
    ),
    (
        "adventure/items",
        "ITEM_FILES",
        "Item graphics (`assets/adventure/items/`), named like their item id.",
    ),
    (
        "adventure/objects",
        "OBJECT_FILES",
        "Objects with two states (`assets/adventure/objects/`).",
    ),
];

fn main() {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let mut out = String::new();
    for (folder, name, doc) in LISTS {
        let dir = assets.join(folder);
        println!("cargo:rerun-if-changed={}", dir.display());
        let mut entries: Vec<(String, String)> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "svg"))
            .filter_map(|p| {
                let stem = p.file_stem()?.to_str()?.to_owned();
                let path = p.canonicalize().ok()?.to_str()?.to_owned();
                Some((stem, path))
            })
            .collect();
        entries.sort();
        let _ = writeln!(out, "/// {doc}\npub const {name}: &[(&str, &[u8])] = &[");
        for (stem, path) in &entries {
            println!("cargo:rerun-if-changed={path}");
            let _ = writeln!(out, "    ({stem:?}, include_bytes!({path:?})),");
        }
        out.push_str("];\n");
    }
    let target = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("asset_files.rs");
    std::fs::write(target, out).expect("asset_files.rs writable");
}

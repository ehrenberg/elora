//! Icon and version information in `elora-server.exe` (E-376): unsigned programs without
//! them are flagged more often by Windows Defender. Only for Windows targets.

use std::path::Path;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let icon = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/icon/elora.ico");
    println!("cargo:rerun-if-changed={}", icon.display());
    let mut res = winresource::WindowsResource::new();
    res.set_icon(icon.to_str().expect("icon path is UTF-8"))
        .set("ProductName", "Elora")
        .set("FileDescription", "Elora server")
        .set("CompanyName", "Elora project")
        .set("LegalCopyright", "GPL-3.0-only");
    res.compile().expect("Windows resource");
}

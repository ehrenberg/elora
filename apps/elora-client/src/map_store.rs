//! Karten für den Online-Client (M6.5, E-136): erst im Zwischenspeicher und in `maps/`
//! suchen, sonst vom Server laden und ablegen.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use elora_protocol::MapChecksum;

/// Woher der Client Karten nimmt und wohin er heruntergeladene legt.
pub trait MapStore: std::fmt::Debug + Send {
    /// Kartendatei mit genau dieser Prüfsumme, falls vorhanden.
    fn find(&mut self, name: &str, checksum: &MapChecksum) -> Option<Vec<u8>>;
    /// Heruntergeladene (bereits geprüfte) Karte ablegen.
    fn store(&mut self, name: &str, checksum: &MapChecksum, data: &[u8]);
}

/// Nur im Speicher (Tests, oder wenn kein Benutzerverzeichnis bekannt ist).
#[derive(Debug, Default)]
pub struct MemoryStore {
    pub maps: HashMap<MapChecksum, Vec<u8>>,
}

impl MapStore for MemoryStore {
    fn find(&mut self, _name: &str, checksum: &MapChecksum) -> Option<Vec<u8>> {
        self.maps.get(checksum).cloned()
    }

    fn store(&mut self, _name: &str, checksum: &MapChecksum, data: &[u8]) {
        self.maps.insert(*checksum, data.to_vec());
    }
}

/// Auf der Platte: Karten aus `maps_dirs` (mitgelieferte und eigene, E-152) und heruntergeladene
/// in `download_dir` (Dateiname `<name>-<prüfsumme>.emap`, so liegen verschiedene Stände nebeneinander).
#[derive(Debug, Clone)]
pub struct DiskStore {
    pub maps_dirs: Vec<PathBuf>,
    pub download_dir: PathBuf,
}

/// Kartenname vom Server als sicherer Dateiname (keine Pfade, keine Sonderzeichen).
pub fn safe_name(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(64)
        .collect();
    if s.is_empty() { "karte".into() } else { s }
}

fn hex(checksum: &MapChecksum) -> String {
    use std::fmt::Write as _;
    checksum.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

fn read_matching(path: &Path, checksum: &MapChecksum) -> Option<Vec<u8>> {
    let data = std::fs::read(path).ok()?;
    (elora_map::checksum(&data) == *checksum).then_some(data)
}

impl DiskStore {
    fn download_path(&self, name: &str, checksum: &MapChecksum) -> PathBuf {
        self.download_dir
            .join(format!("{}-{}.emap", safe_name(name), &hex(checksum)[..16]))
    }
}

impl MapStore for DiskStore {
    fn find(&mut self, name: &str, checksum: &MapChecksum) -> Option<Vec<u8>> {
        let file = format!("{}.emap", safe_name(name));
        read_matching(&self.download_path(name, checksum), checksum).or_else(|| {
            self.maps_dirs
                .iter()
                .find_map(|d| read_matching(&d.join(&file), checksum))
        })
    }

    fn store(&mut self, name: &str, checksum: &MapChecksum, data: &[u8]) {
        let path = self.download_path(name, checksum);
        let tmp = path.with_extension("part");
        let result = std::fs::create_dir_all(&self.download_dir)
            .and_then(|()| std::fs::write(&tmp, data))
            .and_then(|()| std::fs::rename(&tmp, &path));
        match result {
            Ok(()) => tracing::info!(path = %path.display(), "Karte gespeichert"),
            Err(e) => tracing::warn!(path = %path.display(), "Karte nicht gespeichert: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_cannot_escape_the_folder() {
        assert_eq!(safe_name("../../etc/passwd"), "______etc_passwd");
        assert_eq!(safe_name("ctf-test"), "ctf-test");
        assert_eq!(safe_name(""), "karte");
        assert_eq!(safe_name("ä/b"), "__b");
        assert_eq!(safe_name(&"x".repeat(200)).len(), 64);
    }

    #[test]
    fn disk_store_finds_only_matching_checksums() {
        let dir = std::env::temp_dir().join(format!("elora-store-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut store = DiskStore {
            maps_dirs: vec![dir.join("maps")],
            download_dir: dir.join("downloads"),
        };
        let data = b"EMAP-Testdaten".to_vec();
        let sum = elora_map::checksum(&data);
        assert!(store.find("arena", &sum).is_none());
        store.store("arena", &sum, &data);
        assert_eq!(store.find("arena", &sum), Some(data.clone()));
        // eigene Karte in maps/ mit gleichem Inhalt wird ebenfalls gefunden, andere nicht
        std::fs::create_dir_all(&store.maps_dirs[0]).unwrap();
        std::fs::write(store.maps_dirs[0].join("eigen.emap"), &data).unwrap();
        assert!(store.find("eigen", &sum).is_some());
        assert!(store.find("eigen", &[0; 32]).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

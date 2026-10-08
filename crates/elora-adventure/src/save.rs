//! Save games (E-219, E-245): three slots, packed with checksum – no readable files.
//!
//! Layout: `ESAV`, format version (u16, little endian), BLAKE2s-256 of the packed content,
//! zlib-packed content. A damaged slot is reported and never silently overwritten (P-32).

use std::path::{Path, PathBuf};

use blake2::Digest as _;

use crate::state::SaveGame;

const MAGIC: &[u8; 4] = b"ESAV";
const VERSION: u16 = 1;
/// Upper limit of the unpacked content (protection against tampered files).
const MAX_PAYLOAD: usize = 4 * 1024 * 1024;
/// Number of slots (E-219).
pub const SLOTS: usize = 3;

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("kein Spielstand von Elora")]
    NotASave,
    #[error("Formatversion {0} wird nicht unterstützt")]
    Version(u16),
    #[error("Prüfsumme stimmt nicht – der Spielstand ist beschädigt")]
    Checksum,
    #[error("Inhalt nicht lesbar: {0}")]
    Content(String),
    #[error("Platz {0} ist beschädigt und wird nicht überschrieben")]
    Damaged(usize),
    #[error("Platz {0} gibt es nicht")]
    NoSlot(usize),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Packs a save game.
///
/// # Panics
/// Never for valid save games (serialization cannot fail).
pub fn encode(game: &SaveGame) -> Vec<u8> {
    let text = toml::to_string(game).expect("Spielstand serialisierbar");
    let packed = miniz_oxide::deflate::compress_to_vec_zlib(text.as_bytes(), 9);
    let mut out = Vec::with_capacity(packed.len() + 38);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&blake2::Blake2s256::digest(&packed));
    out.extend_from_slice(&packed);
    out
}

/// Unpacks and checks a save game.
///
/// # Errors
/// Wrong magic, unknown version, wrong checksum, invalid content.
pub fn decode(data: &[u8]) -> Result<SaveGame, SaveError> {
    if data.len() < 38 || &data[..4] != MAGIC {
        return Err(SaveError::NotASave);
    }
    let version = u16::from_le_bytes([data[4], data[5]]);
    if version != VERSION {
        return Err(SaveError::Version(version));
    }
    let (sum, packed) = data[6..].split_at(32);
    if blake2::Blake2s256::digest(packed).as_slice() != sum {
        return Err(SaveError::Checksum);
    }
    let raw = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(packed, MAX_PAYLOAD)
        .map_err(|e| SaveError::Content(format!("{e:?}")))?;
    let text = std::str::from_utf8(&raw).map_err(|e| SaveError::Content(e.to_string()))?;
    toml::from_str(text).map_err(|e| SaveError::Content(e.to_string()))
}

/// State of a slot.
#[derive(Debug)]
pub enum SlotState {
    Empty,
    Ok(Box<SaveGame>),
    Damaged(SaveError),
}

/// The save game slots in a directory (`<Benutzerverzeichnis>/saves`).
#[derive(Debug, Clone)]
pub struct Slots {
    dir: PathBuf,
}

impl Slots {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn path(&self, slot: usize) -> PathBuf {
        self.dir.join(format!("platz-{}.esav", slot + 1))
    }

    /// # Errors
    /// If the slot does not exist.
    pub fn state(&self, slot: usize) -> Result<SlotState, SaveError> {
        if slot >= SLOTS {
            return Err(SaveError::NoSlot(slot));
        }
        Ok(match std::fs::read(self.path(slot)) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => SlotState::Empty,
            Err(e) => SlotState::Damaged(e.into()),
            Ok(data) => match decode(&data) {
                Ok(g) => SlotState::Ok(Box::new(g)),
                Err(e) => SlotState::Damaged(e),
            },
        })
    }

    /// All slots.
    pub fn list(&self) -> Vec<SlotState> {
        (0..SLOTS)
            .map(|s| self.state(s).unwrap_or(SlotState::Empty))
            .collect()
    }

    /// # Errors
    /// Slot missing, empty or damaged.
    pub fn load(&self, slot: usize) -> Result<Option<SaveGame>, SaveError> {
        match self.state(slot)? {
            SlotState::Empty => Ok(None),
            SlotState::Ok(g) => Ok(Some(*g)),
            SlotState::Damaged(e) => Err(e),
        }
    }

    /// Saves via a temporary file (no half save game on a crash). A damaged slot is only
    /// replaced with `overwrite_damaged`.
    ///
    /// # Errors
    /// Damaged slot or write error.
    pub fn save(
        &self,
        slot: usize,
        game: &SaveGame,
        overwrite_damaged: bool,
    ) -> Result<(), SaveError> {
        if let SlotState::Damaged(_) = self.state(slot)?
            && !overwrite_damaged
        {
            return Err(SaveError::Damaged(slot + 1));
        }
        std::fs::create_dir_all(&self.dir)?;
        let path = self.path(slot);
        let tmp = path.with_extension("esav.tmp");
        std::fs::write(&tmp, encode(game))?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    /// # Errors
    /// Delete error (an empty slot is not an error).
    pub fn delete(&self, slot: usize) -> Result<(), SaveError> {
        match std::fs::remove_file(self.path(slot)) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
            _ => Ok(()),
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Content;
    use crate::state::Location;

    fn sample() -> SaveGame {
        let c = Content::builtin();
        let mut g = SaveGame::new(
            &c,
            Location {
                map: "tauwinkel".into(),
                spawn: "brunnen".into(),
            },
        );
        g.add_xp(&c, 300);
        g.add_item(&c, "glanztropfen", 77).unwrap();
        g.add_item(&c, "bernstein", 2).unwrap();
        g.set_flag("tuer.wiese-1.tor", 1);
        g.broken.entry("wiese-1".into()).or_default().insert((4, 9));
        g.defeated.insert("wiese-3:hummel".into());
        g.give_weapon(elora_sim::Weapon::Grenade);
        g
    }

    #[test]
    fn roundtrip_and_not_readable() {
        let g = sample();
        let data = encode(&g);
        assert_eq!(decode(&data).unwrap(), g);
        assert!(
            !data.windows(9).any(|w| w == b"tauwinkel"),
            "Inhalt ist gepackt (E-245)"
        );
    }

    #[test]
    fn damage_is_detected() {
        let mut data = encode(&sample());
        let n = data.len();
        data[n - 3] ^= 0x55;
        assert!(matches!(decode(&data), Err(SaveError::Checksum)));
        assert!(matches!(decode(b"nichts"), Err(SaveError::NotASave)));
        let mut v2 = encode(&sample());
        v2[4] = 9;
        assert!(matches!(decode(&v2), Err(SaveError::Version(9))));
    }

    #[test]
    fn slots_save_load_and_protect_damaged() {
        let dir = std::env::temp_dir().join(format!("elora-saves-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let slots = Slots::new(&dir);
        assert!(matches!(slots.state(0).unwrap(), SlotState::Empty));
        let g = sample();
        slots.save(1, &g, false).unwrap();
        assert_eq!(slots.load(1).unwrap(), Some(g.clone()));
        std::fs::write(slots.path(2), b"kaputt").unwrap();
        assert!(matches!(slots.list()[2], SlotState::Damaged(_)));
        assert!(matches!(
            slots.save(2, &g, false),
            Err(SaveError::Damaged(3))
        ));
        slots.save(2, &g, true).unwrap();
        slots.delete(2).unwrap();
        assert!(matches!(slots.state(2).unwrap(), SlotState::Empty));
        assert!(slots.state(3).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

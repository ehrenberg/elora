//! IVF container: 32-byte file header (`DKIF`), then frames of `size (u32) | pts (u64) | data`.
//! All numbers little endian.

/// What the IVF file header says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub header_len: u16,
    pub fourcc: [u8; 4],
    pub width: u16,
    pub height: u16,
    /// Time base: `rate / scale` frames per second.
    pub rate: u32,
    pub scale: u32,
    pub frames: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IvfError {
    #[error("not an IVF file")]
    NotIvf,
    #[error("codec `{0}` is not AV1")]
    NotAv1(String),
    #[error("frame at byte {0} is cut off")]
    Truncated(usize),
}

fn u16_at(d: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([d[at], d[at + 1]])
}

fn u32_at(d: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([d[at], d[at + 1], d[at + 2], d[at + 3]])
}

impl Header {
    /// # Errors
    /// Missing `DKIF` signature, too short or not AV1.
    pub fn parse(d: &[u8]) -> Result<Self, IvfError> {
        if d.len() < 32 || &d[0..4] != b"DKIF" {
            return Err(IvfError::NotIvf);
        }
        let h = Self {
            header_len: u16_at(d, 6).max(32),
            fourcc: [d[8], d[9], d[10], d[11]],
            width: u16_at(d, 12),
            height: u16_at(d, 14),
            rate: u32_at(d, 16),
            scale: u32_at(d, 20),
            frames: u32_at(d, 24),
        };
        if &h.fourcc != b"AV01" {
            return Err(IvfError::NotAv1(
                String::from_utf8_lossy(&h.fourcc).into_owned(),
            ));
        }
        Ok(h)
    }

    /// Frames per second (24 if the time base is missing).
    pub fn fps(&self) -> f32 {
        if self.rate == 0 || self.scale == 0 {
            24.0
        } else {
            self.rate as f32 / self.scale as f32
        }
    }
}

/// The frame starting at `offset`: its data and the offset of the next frame; `None` at the end.
pub(crate) fn frame_at(d: &[u8], offset: usize) -> Result<Option<(&[u8], usize)>, IvfError> {
    if offset >= d.len() {
        return Ok(None);
    }
    if offset + 12 > d.len() {
        return Err(IvfError::Truncated(offset));
    }
    let size = u32_at(d, offset) as usize;
    let start = offset + 12;
    let end = start.checked_add(size).filter(|&e| e <= d.len());
    let end = end.ok_or(IvfError::Truncated(offset))?;
    Ok(Some((&d[start..end], end)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_other_files() {
        assert_eq!(Header::parse(b"hello"), Err(IvfError::NotIvf));
        let mut vp9 = include_bytes!("../tests/data/clip.ivf").to_vec();
        vp9[8..12].copy_from_slice(b"VP90");
        assert_eq!(Header::parse(&vp9), Err(IvfError::NotAv1("VP90".into())));
    }

    #[test]
    fn truncated_frame_is_an_error() {
        let d = include_bytes!("../tests/data/clip.ivf");
        let cut = &d[..d.len() - 5];
        let mut at = 32;
        let err = loop {
            match frame_at(cut, at) {
                Ok(Some((_, next))) => at = next,
                Ok(None) => panic!("the cut-off frame must be noticed"),
                Err(e) => break e,
            }
        };
        assert!(matches!(err, IvfError::Truncated(_)));
    }
}

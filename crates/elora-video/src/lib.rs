//! AV1 video playback for the adventure intro (E-355, E-356).
//!
//! [`Video`] reads an IVF file (the simple container ffmpeg writes with `-f ivf`) and decodes
//! its AV1 frames one after another into RGBA images. Decoding uses `rav1d`, whose API is the
//! C API of dav1d; the small wrapper in [`av1`] is the **only** place in the workspace where
//! `unsafe` is allowed (E-356).

mod av1;
mod ivf;
mod yuv;

pub use ivf::{Header, IvfError};

/// One decoded image: `width × height` pixels, RGBA, row after row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Something went wrong while reading or decoding the video.
#[derive(Debug, thiserror::Error)]
pub enum VideoError {
    #[error(transparent)]
    Ivf(#[from] IvfError),
    #[error("AV1 decoder: {0}")]
    Decoder(String),
    #[error("unsupported picture: {0}")]
    Unsupported(String),
}

/// A video in memory, decoded frame by frame.
#[derive(Debug)]
pub struct Video {
    header: Header,
    data: Vec<u8>,
    /// Position of the next IVF frame in `data`.
    offset: usize,
    decoder: av1::Decoder,
    /// Decoded frames that are ready but not taken yet.
    ready: std::collections::VecDeque<Frame>,
    flushed: bool,
}

impl Video {
    /// Opens an IVF file with AV1 frames.
    ///
    /// # Errors
    /// Not an IVF file, not AV1, or the decoder cannot start.
    pub fn new(data: Vec<u8>) -> Result<Self, VideoError> {
        let header = Header::parse(&data)?;
        Ok(Self {
            offset: usize::from(header.header_len),
            header,
            data,
            decoder: av1::Decoder::new()?,
            ready: std::collections::VecDeque::new(),
            flushed: false,
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Frames per second from the IVF time base.
    pub fn fps(&self) -> f32 {
        self.header.fps()
    }

    /// The next frame, `Ok(None)` at the end of the video.
    ///
    /// # Errors
    /// Damaged frame data or an unsupported picture format.
    pub fn next_frame(&mut self) -> Result<Option<Frame>, VideoError> {
        loop {
            if let Some(f) = self.ready.pop_front() {
                return Ok(Some(f));
            }
            if self.flushed {
                return Ok(None);
            }
            if let Some((packet, next)) = ivf::frame_at(&self.data, self.offset)? {
                self.offset = next;
                let packet = packet.to_vec();
                self.ready.extend(self.decoder.decode(&packet)?);
            } else {
                self.flushed = true;
                self.ready.extend(self.decoder.drain()?);
            }
        }
    }
}

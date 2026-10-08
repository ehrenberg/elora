//! Thin wrapper around the C API of `rav1d` (E-356). This module is the only place in the
//! workspace that uses `unsafe`; every block states why it is sound.
#![allow(unsafe_code)]

use std::ptr::NonNull;

use rav1d::include::dav1d::data::Dav1dData;
use rav1d::include::dav1d::dav1d::{Dav1dContext, Dav1dSettings};
use rav1d::include::dav1d::headers::DAV1D_PIXEL_LAYOUT_I420;
use rav1d::include::dav1d::picture::Dav1dPicture;
use rav1d::src::lib::{
    dav1d_close, dav1d_data_create, dav1d_data_unref, dav1d_default_settings, dav1d_get_picture,
    dav1d_open, dav1d_picture_unref, dav1d_send_data,
};

use crate::yuv::{Plane, to_rgba};
use crate::{Frame, VideoError};

/// `DAV1D_ERR(EAGAIN)`: the decoder wants the caller to take pictures / send more data first.
const AGAIN: i32 = -libc::EAGAIN;

/// An open dav1d decoder context; closed on drop.
pub(crate) struct Decoder {
    ctx: Option<Dav1dContext>,
}

impl std::fmt::Debug for Decoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Decoder")
    }
}

fn check(what: &str, r: i32) -> Result<(), VideoError> {
    if r < 0 {
        Err(VideoError::Decoder(format!("{what} failed ({r})")))
    } else {
        Ok(())
    }
}

impl Decoder {
    pub(crate) fn new() -> Result<Self, VideoError> {
        let mut init = std::mem::MaybeUninit::<Dav1dSettings>::uninit();
        // SAFETY: `dav1d_default_settings` writes a complete `Dav1dSettings` to the pointer,
        // which points to memory of the right size and alignment; afterwards it is initialised.
        let mut settings = unsafe {
            dav1d_default_settings(NonNull::new_unchecked(init.as_mut_ptr()));
            init.assume_init()
        };
        settings.n_threads = 0; // as many threads as the machine has
        settings.max_frame_delay = 1; // low latency: one picture out per packet
        let mut ctx: Option<Dav1dContext> = None;
        // SAFETY: both pointers point to live, exclusively borrowed values; on success dav1d
        // writes the new context into `ctx`.
        let r = unsafe {
            dav1d_open(
                Some(NonNull::from(&mut ctx)),
                Some(NonNull::from(&mut settings)),
            )
        };
        check("dav1d_open", r.0)?;
        Ok(Self { ctx })
    }

    /// Sends one AV1 packet and returns every picture that is ready.
    pub(crate) fn decode(&mut self, packet: &[u8]) -> Result<Vec<Frame>, VideoError> {
        let mut data = Dav1dData::default();
        // SAFETY: `data` is a live default `Dav1dData`; dav1d allocates `packet.len()` bytes
        // that it owns and returns a pointer to them (null on failure).
        let buf = unsafe { dav1d_data_create(Some(NonNull::from(&mut data)), packet.len()) };
        if buf.is_null() {
            return Err(VideoError::Decoder("dav1d_data_create failed".into()));
        }
        // SAFETY: `buf` points to `packet.len()` writable bytes owned by `data`, which do not
        // overlap `packet`.
        unsafe { std::ptr::copy_nonoverlapping(packet.as_ptr(), buf, packet.len()) };
        let mut frames = Vec::new();
        loop {
            // SAFETY: `ctx` comes from `dav1d_open` and is not closed; `data` is live. dav1d
            // consumes the data it accepts and leaves the rest in `data`.
            let r = unsafe { dav1d_send_data(self.ctx, Some(NonNull::from(&mut data))) };
            if r.0 == AGAIN {
                // output full: take pictures, then send the rest of the packet
                self.collect(&mut frames)?;
                continue;
            }
            if r.0 < 0 {
                // SAFETY: `data` is a valid `Dav1dData`; unref frees what is left of it.
                unsafe { dav1d_data_unref(Some(NonNull::from(&mut data))) };
                return Err(VideoError::Decoder(format!(
                    "dav1d_send_data failed ({})",
                    r.0
                )));
            }
            if data.sz == 0 {
                break;
            }
        }
        self.collect(&mut frames)?;
        Ok(frames)
    }

    /// Takes the pictures that are still in the decoder at the end of the video.
    pub(crate) fn drain(&mut self) -> Result<Vec<Frame>, VideoError> {
        let mut frames = Vec::new();
        // without new data dav1d returns the delayed pictures until EAGAIN
        self.collect(&mut frames)?;
        Ok(frames)
    }

    fn collect(&mut self, out: &mut Vec<Frame>) -> Result<(), VideoError> {
        loop {
            let mut pic = Dav1dPicture::default();
            // SAFETY: `ctx` is open, `pic` is a live default picture that dav1d fills.
            let r = unsafe { dav1d_get_picture(self.ctx, Some(NonNull::from(&mut pic))) };
            if r.0 == AGAIN {
                return Ok(());
            }
            check("dav1d_get_picture", r.0)?;
            let frame = convert(&pic);
            // SAFETY: `pic` was filled by `dav1d_get_picture` and is released exactly once.
            unsafe { dav1d_picture_unref(Some(NonNull::from(&mut pic))) };
            out.push(frame?);
        }
    }
}

/// Copies a decoded 8-bit 4:2:0 picture into RGBA.
fn convert(pic: &Dav1dPicture) -> Result<Frame, VideoError> {
    if pic.p.bpc != 8 || pic.p.layout != DAV1D_PIXEL_LAYOUT_I420 {
        return Err(VideoError::Unsupported(format!(
            "{} bit, layout {}",
            pic.p.bpc, pic.p.layout
        )));
    }
    let (w, h) = (
        usize::try_from(pic.p.w).unwrap_or(0),
        usize::try_from(pic.p.h).unwrap_or(0),
    );
    let (ys, cs) = (
        usize::try_from(pic.stride[0]).unwrap_or(0),
        usize::try_from(pic.stride[1]).unwrap_or(0),
    );
    let [Some(luma), Some(cb), Some(cr)] = pic.data else {
        return Err(VideoError::Unsupported("picture without planes".into()));
    };
    if w == 0 || h == 0 || ys < w || cs < w.div_ceil(2) {
        return Err(VideoError::Unsupported(format!("size {w}×{h}")));
    }
    let ch = h.div_ceil(2);
    // SAFETY: dav1d guarantees that each plane holds `stride × rows` bytes (luma: `h` rows,
    // chroma: `ceil(h / 2)` rows) and keeps them alive until `dav1d_picture_unref`, which the
    // caller only calls after this function returned. The slices do not outlive `pic`.
    let (yp, up, vp) = unsafe {
        (
            std::slice::from_raw_parts(luma.as_ptr().cast::<u8>(), ys * h),
            std::slice::from_raw_parts(cb.as_ptr().cast::<u8>(), cs * ch),
            std::slice::from_raw_parts(cr.as_ptr().cast::<u8>(), cs * ch),
        )
    };
    let rgba = to_rgba(
        w,
        h,
        Plane {
            data: yp,
            stride: ys,
        },
        Plane {
            data: up,
            stride: cs,
        },
        Plane {
            data: vp,
            stride: cs,
        },
    );
    Ok(Frame {
        width: u32::try_from(w).unwrap_or(0),
        height: u32::try_from(h).unwrap_or(0),
        rgba,
    })
}

impl Drop for Decoder {
    fn drop(&mut self) {
        // SAFETY: `ctx` comes from `dav1d_open` and is closed only here; dav1d sets it to None.
        unsafe { dav1d_close(Some(NonNull::from(&mut self.ctx))) };
    }
}

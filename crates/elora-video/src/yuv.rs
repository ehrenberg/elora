//! YUV 4:2:0 (8 bit, limited range, BT.709) to RGBA.

/// One plane of a picture: `stride` bytes per row.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Plane<'a> {
    pub data: &'a [u8],
    pub stride: usize,
}

/// Converts a 4:2:0 picture with chroma planes of half width and height.
pub(crate) fn to_rgba(
    width: usize,
    height: usize,
    y: Plane<'_>,
    u: Plane<'_>,
    v: Plane<'_>,
) -> Vec<u8> {
    let mut out = vec![0u8; width * height * 4];
    for row in 0..height {
        let yr = &y.data[row * y.stride..row * y.stride + width];
        let cr = row / 2;
        let ur = &u.data[cr * u.stride..];
        let vr = &v.data[cr * v.stride..];
        let line = &mut out[row * width * 4..(row + 1) * width * 4];
        for (col, px) in line.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            // fixed point (×1024) of the BT.709 limited-range matrix
            let luma = (i32::from(yr[col]) - 16) * 1192;
            let cb = i32::from(ur[col / 2]) - 128;
            let crv = i32::from(vr[col / 2]) - 128;
            px[0] = channel(luma + 1836 * crv);
            px[1] = channel(luma - 218 * cb - 546 * crv);
            px[2] = channel(luma + 2163 * cb);
            px[3] = 255;
        }
    }
    out
}

#[allow(clippy::cast_sign_loss)]
fn channel(v: i32) -> u8 {
    ((v + 512) >> 10).clamp(0, 255) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grey_white_and_red() {
        let p = |d: &'static [u8]| Plane { data: d, stride: 2 };
        // 2×2 picture: one chroma sample
        let grey = to_rgba(2, 2, p(&[126, 126, 126, 126]), p(&[128]), p(&[128]));
        assert!(
            grey.chunks(4).all(|c| c == [128, 128, 128, 255]),
            "{grey:?}"
        );
        let white = to_rgba(2, 2, p(&[235, 235, 235, 235]), p(&[128]), p(&[128]));
        assert!(white.chunks(4).all(|c| c == [255, 255, 255, 255]));
        // BT.709 red: Y 63, Cb 102, Cr 240
        let red = to_rgba(2, 2, p(&[63, 63, 63, 63]), p(&[102]), p(&[240]));
        assert!(red[0] > 250 && red[1] < 5 && red[2] < 5, "{red:?}");
    }
}

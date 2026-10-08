//! Decodes the test clip (64×48, 24 fps, 12 frames, made with ffmpeg + libaom).

use elora_video::Video;

const CLIP: &[u8] = include_bytes!("data/clip.ivf");
/// First frame as ffmpeg decodes it (RGB, BT.709 limited range).
const FRAME0: &[u8] = include_bytes!("data/frame0.rgb");

#[test]
fn header_and_all_frames() {
    let mut v = Video::new(CLIP.to_vec()).expect("open");
    assert_eq!((v.header().width, v.header().height), (64, 48));
    assert!((v.fps() - 24.0).abs() < 1e-3);
    let mut n = 0;
    while let Some(f) = v.next_frame().expect("decode") {
        assert_eq!((f.width, f.height, f.rgba.len()), (64, 48, 64 * 48 * 4));
        n += 1;
    }
    assert_eq!(n, 12);
    assert!(v.next_frame().expect("end").is_none(), "stays at the end");
}

#[test]
fn colours_match_ffmpeg() {
    let mut v = Video::new(CLIP.to_vec()).unwrap();
    let f = v.next_frame().unwrap().unwrap();
    let diff: u64 = f
        .rgba
        .chunks(4)
        .zip(FRAME0.chunks(3))
        .map(|(a, b)| (0..3).map(|i| u64::from(a[i].abs_diff(b[i]))).sum::<u64>())
        .sum();
    let mean = diff as f64 / (64.0 * 48.0 * 3.0);
    assert!(mean < 3.0, "mean difference per channel: {mean}");
}

#[test]
fn garbage_is_an_error_not_a_crash() {
    assert!(Video::new(b"not a video".to_vec()).is_err());
    let mut broken = CLIP.to_vec();
    for b in broken.iter_mut().skip(60).step_by(7) {
        *b ^= 0x5a;
    }
    if let Ok(mut v) = Video::new(broken) {
        for _ in 0..20 {
            if !matches!(v.next_frame(), Ok(Some(_))) {
                break;
            }
        }
    }
}

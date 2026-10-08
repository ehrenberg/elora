//! Decoding speed: `cargo run --release -p elora-video --example bench -- <file.ivf>`.

fn main() {
    let path = std::env::args().nth(1).expect("usage: bench <file.ivf>");
    let data = std::fs::read(&path).expect("read");
    let mut v = elora_video::Video::new(data).expect("open");
    let start = std::time::Instant::now();
    let mut n = 0u32;
    while v.next_frame().expect("decode").is_some() {
        n += 1;
    }
    let secs = start.elapsed().as_secs_f64();
    println!("{n} frames in {secs:.2} s = {:.0} fps", f64::from(n) / secs);
}

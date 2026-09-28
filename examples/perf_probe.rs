//! Frame-time probe: how long does the busiest frame take to render?
//!
//! `cargo run --release --example float_probe`
//!
//! The player targets 24 fps (41.7 ms per frame), so the worst-case render must
//! stay far below that for the animation to be smooth.

use std::time::Instant;

use world_execute_me_rust::Film;

fn main() {
    let film = Film::load().expect("assets");

    // Glitch-heavy windows are the expensive ones: the late sections apply
    // escalating corruption on top of the scene.
    let windows: [(&str, f64, f64); 6] = [
        ("boot", 0.0, 16.0),
        ("title", 16.0, 29.709),
        ("devotion", 29.709, 74.0),
        ("organic", 74.0, 110.9),
        ("isolation", 110.9, 177.246),
        ("love", 177.246, 211.9),
    ];

    for (name, lo, hi) in windows {
        let mut worst = 0.0f64;
        let mut total = 0.0f64;
        let mut n = 0u32;
        let mut t = lo;
        while t < hi {
            let begin = Instant::now();
            let _ = film.render(t, 120, 40, false, 0.0, false, false);
            let dt = begin.elapsed().as_secs_f64();
            worst = worst.max(dt);
            total += dt;
            n += 1;
            t += 0.25;
        }
        println!(
            "{name:<10} frames={n:<5} mean={:>7.3} ms   worst={:>7.3} ms",
            total / n as f64 * 1000.0,
            worst * 1000.0
        );
    }

    // Full-timeline sweep at 24 fps: the real playback load.
    let begin = Instant::now();
    let mut frames = 0u32;
    let mut t = 0.0;
    while t < 211.9 {
        let _ = film.render(t, 120, 40, false, 0.0, false, false);
        frames += 1;
        t += 1.0 / 24.0;
    }
    let elapsed = begin.elapsed().as_secs_f64();
    println!(
        "\nfull song @24fps: {frames} frames in {elapsed:.2} s (render only) -> {:.0}x realtime headroom",
        211.9 / elapsed
    );
}


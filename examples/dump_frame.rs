//! Dump a frame's plain grid to stdout.
//!
//! `cargo run --release --example dump_frame -- <t> [w] [h] [mode]`
//!
//! mode `render` (default) renders the full frame; mode `source` renders the
//! exact pre-transition source frame that the title takeover scatters, which is
//! `render(15.799, ...)` regardless of the requested time.

use world_execute_me_rust::Film;

fn main() {
    let mut a = std::env::args().skip(1);
    let t: f64 = a.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let w: i64 = a.next().and_then(|s| s.parse().ok()).unwrap_or(120);
    let h: i64 = a.next().and_then(|s| s.parse().ok()).unwrap_or(40);
    let mode = a.next().unwrap_or_else(|| "render".to_string());
    let film = Film::load().expect("assets");
    let t = if mode == "source" { 15.799 } else { t };
    print!("{}", film.render(t, w, h, false, 0.0, false, false).plain());
}

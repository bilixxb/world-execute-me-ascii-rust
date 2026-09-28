//! Scene helpers, the glitch engine, and 3D projection — port of `scenes.py`
//! lines 1–127.

use crate::canvas::Canvas;
use crate::core::{self, hash16, pyround, Style, BRIGHT, DARK, DIM, GREEN, NORMAL, RED, WHITE};
use crate::TAU;

pub use crate::core::mix;

/// Python `clamp(x)` — the overwhelmingly common single-argument form, clamping
/// to `0..1`.
#[inline]
pub fn clamp(x: f64) -> f64 {
    core::unit(x)
}

/// Python `clamp(x, a, b)` — the explicit three-argument form.
#[inline]
pub fn clamp3(x: f64, a: f64, b: f64) -> f64 {
    core::clamp(x, a, b)
}

/// Python's `min(b, max(a, x))` over integers.
#[inline]
pub fn imin(a: i64, b: i64) -> i64 {
    if b < a {
        b
    } else {
        a
    }
}

#[inline]
pub fn imax(a: i64, b: i64) -> i64 {
    if a < b {
        b
    } else {
        a
    }
}

/// Float `min`/`max`, re-exported under names that cannot shadow Rust's
/// generic integer versions.
pub use crate::core::{fmax, fmin};

/// Python `clear(c, x, y, w, h)`.
pub fn clear(c: &Canvas, x: f64, y: f64, w: f64, h: f64) {
    let blank = " ".repeat(core::fmax(0.0, w as i64 as f64) as usize);
    for yy in (y as i64)..((y + h) as i64) {
        c.put(x, yy as f64, &blank, DARK);
    }
}

/// Python `hash16(...) % m < t` where `t` is a **float** threshold.
///
/// The comparison must happen in float space. A threshold below 1 (which
/// happens constantly: `buildup * 50` at `buildup = 3.6e-15`) still matches
/// every cell whose residue is 0; casting the threshold to an integer would
/// turn ~50 matching cells into zero.
#[inline]
pub fn hash_below(hash: u32, m: u32, threshold: f64) -> bool {
    let residue = f64::from(hash % m);
    residue < threshold
}

/// Python `glitch_intensity(t)`. Each departure accumulates damage; later
/// execution never resets it.
pub fn glitch_intensity(t: f64) -> f64 {
    if t < 60.0 {
        return 0.05;
    }
    if t < 110.0 {
        return core::mix(0.05, 0.25, (t - 60.0) / 50.0);
    }
    if t < 110.9 {
        return core::mix(0.25, 0.6, (t - 110.0) / 37.0);
    }
    let anchors: [(f64, f64); 10] = [
        (110.9, 0.259),
        (112.22, 0.34),
        (113.1, 0.43),
        (114.18, 0.51),
        (114.92, 0.59),
        (115.78, 0.66),
        (117.274, 0.73),
        (125.708, 0.81),
        (147.66, 0.92),
        (177.246, 1.0),
    ];
    for (i, w) in anchors.windows(2).enumerate() {
        let (a, low) = anchors[i];
        let (b, high) = w[1];
        if t < b {
            return core::mix(low, high, core::unit((t - a) / (b - a)));
        }
    }
    1.0
}

/// Port of `apply_glitch`.
pub fn apply_glitch(c: &Canvas, t: f64, top: i64, bt: i64, intensity: Option<f64>) {
    let intensity = intensity.unwrap_or_else(|| glitch_intensity(t));
    if intensity < 0.01 {
        return;
    }
    let frame = (t * 24.0) as i64;

    // Horizontal line displacement
    if hash_below(hash16(frame), 100, intensity * 30.0) {
        for i in 0..(intensity * 5.0) as i64 {
            let row = top + (hash16(frame + i * 7) % (bt - top + 1) as u32) as i64;
            let shift = (((hash16(frame + i * 13) % 20) as i64 - 10) as f64 * intensity) as i64;
            if shift != 0 {
                c.rotate_row(row, shift);
            }
        }
    }

    // Color corruption
    if hash_below(hash16(frame + 100), 100, intensity * 40.0) {
        for i in 0..(intensity * 15.0) as i64 {
            let x = (hash16(frame + i * 19) % (c.w - 4).max(1) as u32) as i64 + 2;
            let y = top + (hash16(frame + i * 23) % (bt - top + 1) as u32) as i64;
            let cell = c.get(x, y);
            c.set(
                x,
                y,
                cell.ch,
                if intensity > 0.7 {
                    RED
                } else if intensity > 0.4 {
                    WHITE
                } else {
                    BRIGHT
                },
            );
        }
    }

    // Scanline interference
    if intensity > 0.3 {
        let scan_rows: Vec<i64> = (0..(intensity * 3.0) as i64)
            .map(|i| top + ((t * 17.0) as i64 + i * 31) % (bt - top + 1))
            .collect();
        for row in scan_rows {
            if row < 0 || row >= c.nrows() {
                continue;
            }
            for x in 2..(c.w - 2) {
                if hash_below(hash16(x + frame), 100, intensity * 60.0) {
                    let ch = c.get(x, row).ch;
                    let new = match ch {
                        Some(' ') | None => Some('='),
                        other => other,
                    };
                    c.set(x, row, new, WHITE);
                }
            }
        }
    }

    // Block corruption (late-stage)
    if intensity > 0.6 {
        for i in 0..((intensity - 0.6) * 20.0) as i64 {
            let x = (hash16(frame + i * 31) % (c.w - 10).max(1) as u32) as i64 + 2;
            let y = top + (hash16(frame + i * 37) % (bt - top - 3).max(1) as u32) as i64;
            let w = 3 + (hash16(i * 41) % 8) as i64;
            let h = 2 + (hash16(i * 43) % 4) as i64;
            if hash_below(hash16(frame + i), 100, (intensity - 0.6) * 100.0) {
                let chars: Vec<char> = "█▓▒░#@%$".chars().collect();
                for dy in 0..h {
                    for dx in 0..w {
                        let ch = chars[(hash16(dx + dy * 3) as usize) % chars.len()];
                        let style = if hash16(i) % 3 == 0 { RED } else { WHITE };
                        if y + dy < c.nrows() && x + dx < c.ncols() {
                            c.set(x + dx, y + dy, Some(ch), style);
                        }
                    }
                }
            }
        }
    }

    // Persistent corruption after YOU leaves, beyond occasional flash frames.
    if t >= 110.9 {
        let tick = (t * 12.0) as i64;
        for band in 0..1 + (intensity * 5.0) as i64 {
            let row = top + (hash16(tick * 7 + band * 41) % (bt - top + 1) as u32) as i64;
            let start = 2 + (hash16(tick + band * 131) % core::fmax(1.0, (c.w - 18) as f64) as u32) as i64;
            let length = 3 + (intensity * 14.0) as i64;
            for dx in 0..length {
                let x = start + dx;
                if x < c.w - 2 {
                    let ch = ['0', '1', '/', ':', '#', '_'][(hash16(tick + dx + band) % 6) as usize];
                    c.put(x as f64, row as f64, &ch.to_string(), if dx % 4 != 0 { NORMAL } else { BRIGHT });
                }
            }
        }
        // Low-luminance duplicated rows produce readable CRT signal ghosts.
        if intensity > 0.4 {
            let row = top + (hash16(tick * 17) % (bt - top).max(1) as u32) as i64;
            let source: Vec<crate::canvas::Cell> = {
                let cells = c.cells.borrow();
                if row < 0 || row as usize >= cells.len() {
                    Vec::new()
                } else {
                    cells[row as usize].clone()
                }
            };
            let shift = 2 + (intensity * 5.0) as i64;
            for x in 2..(c.w - shift - 2) {
                let ch = source.get(x as usize).and_then(|cell| cell.ch);
                if let Some(ch) = ch {
                    if !ch.is_whitespace() && hash16(x + tick) % 3 == 0 {
                        c.put((x + shift) as f64, (row + 1) as f64, &ch.to_string(), GREEN);
                    }
                }
            }
        }
    }
}

/// The full-width scene area, without sidebars. Python `simple_area`.
pub fn simple_area(c: &Canvas, top: i64, bt: i64) -> (i64, i64, i64, i64) {
    (2, top, c.w - 3, bt)
}

/// Python `rot(x, y, z, t)` — the shared time-driven 3D rotation.
#[inline]
pub fn rot(x: f64, y: f64, z: f64, t: f64) -> (f64, f64, f64) {
    let (a, b) = (t * 0.37, t * 0.23);
    let (x, z) = (
        x * a.cos() + z * a.sin(),
        z * a.cos() - x * a.sin(),
    );
    let (y, z) = (
        y * b.cos() - z * b.sin(),
        y * b.sin() + z * b.cos(),
    );
    (x, y, z)
}

/// Python `point(x, y, z, area, t=0, rotate=True)`.
pub fn point(x: f64, y: f64, z: f64, area: (i64, i64, i64, i64), t: f64, rotate: bool) -> (f64, f64, f64) {
    let (x, y, z) = if rotate { rot(x, y, z, t) } else { (x, y, z) };
    let (l, top, r, bt) = area;
    let p = 3.7 / (3.7 + z);
    (
        (l + r) as f64 / 2.0 + x * (r - l) as f64 * 0.34 * p,
        (top + bt) as f64 / 2.0 + y * (bt - top) as f64 * 0.34 * p,
        z,
    )
}

/// Port of `projected`.
pub fn projected(
    c: &Canvas,
    vertices: &[(f64, f64, f64)],
    edges: &[(usize, usize)],
    area: (i64, i64, i64, i64),
    t: f64,
    style: Style,
    reveal: f64,
) {
    let ps: Vec<(f64, f64, f64)> = vertices.iter().map(|&(x, y, z)| point(x, y, z, area, t, true)).collect();
    let count = (edges.len() as f64 * core::unit(reveal)) as usize;
    for &(a, b) in edges.iter().take(count) {
        let (x, y, z) = ps[a];
        let (xx, yy, zz) = ps[b];
        c.line(
            x,
            y,
            xx,
            yy,
            if (z + zz) < 0.0 { ':' } else { '.' },
            if (z + zz) < 0.0 { style } else { GREEN },
        );
    }
    let limit = vertices.len() as f64 * reveal;
    for (i, &(x, y, z)) in ps.iter().enumerate() {
        if (i as f64) < limit {
            c.put(
                x,
                y,
                if z > 0.0 { "+" } else { "@" },
                if z > 0.0 { NORMAL } else { BRIGHT },
            );
        }
    }
}

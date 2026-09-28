//! Shared palette + numeric helpers.
//!
//! Ported 1:1 from `scenes.py` / `player.py`. Style indices must match the
//! Python constants exactly, because `Canvas::ansi` emits `STYLES[style]`.

pub type Style = u8;

/// `player.py`: DIM, NORMAL, BRIGHT, WHITE, RED
pub const DIM: Style = 0;
pub const NORMAL: Style = 1;
pub const BRIGHT: Style = 2;
pub const WHITE: Style = 3;
pub const RED: Style = 4;
/// `scenes.py`: G, K (extra dim greens used by glitch/CRT effects)
pub const GREEN: Style = 5;
pub const DARK: Style = 6;

/// `player.py` STYLES table, indexed by style id.
pub const STYLES: [&str; 7] = [
    "\x1b[0;38;5;137m",
    "\x1b[0;38;5;215m",
    "\x1b[1;38;5;221m",
    "\x1b[1;38;5;230m",
    "\x1b[1;38;5;203m",
    "\x1b[0;38;5;94m",
    "\x1b[0;38;5;58m",
];

/// Python `a + (b - a) * u`.
#[inline]
pub fn mix(a: f64, b: f64, u: f64) -> f64 {
    a + (b - a) * u
}

/// Python `clamp(x, a=0, b=1)` — note the argument order and that the default
/// upper bound is 1.
#[inline]
pub fn clamp(x: f64, a: f64, b: f64) -> f64 {
    fmin(b, fmax(a, x))
}

/// Python `clamp(x)` — the single-argument form, clamped to `0..1`.
#[inline]
pub fn unit(x: f64) -> f64 {
    clamp(x, 0.0, 1.0)
}

/// `min`/`max` for floats.
///
/// Deliberately *not* named `min`/`max`: scene code needs Rust's generic
/// `std::cmp::min`/`max` for integer comparisons, and a glob-imported
/// `core::min` would shadow them and break `<` parsing.
#[inline]
pub fn fmin(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}

#[inline]
pub fn fmax(a: f64, b: f64) -> f64 {
    if a < b {
        b
    } else {
        a
    }
}

/// Port of `hash16`. All arithmetic wraps at 32 bits, matching the Python
/// `& 0xFFFFFFFF` masks. Returns a value in `0..=65535`.
#[inline]
pub fn hash16(i: i64) -> u32 {
    let mut v = (i as i32).wrapping_add(0x9E37_79B9u32 as i32) as u32;
    v = (v ^ (v >> 16)).wrapping_mul(0x7FEB_352D);
    v = (v ^ (v >> 15)).wrapping_mul(0x846C_A68B);
    (v ^ (v >> 16)) & 0xFFFF
}

/// Python `Canvas.line`'s step count: `max(1, int(max(|dx|,|dy|) * 1.5))`.
#[inline]
pub fn line_steps(dx: f64, dy: f64) -> i64 {
    let m = fmax(dx.abs(), dy.abs()) * 1.5;
    let steps = m as i64;
    if steps < 1 {
        1
    } else {
        steps
    }
}

/// Python `round()`: round-half-to-even on the exact binary value.
///
/// Rust's `f64::round` rounds half away from zero, which differs for exact
/// `.5` values — and those occur constantly in coordinate math here.
///
/// The tie is resolved by flooring to the integer grid and picking whichever of
/// the two neighbours is even. `x.round()` cannot be used as the starting point
/// because it is wrong for negative ties (`round(-2.5)` must be `-2`, not `-3`).
#[inline]
pub fn pyround(x: f64) -> f64 {
    let frac = x - x.floor();
    if frac == 0.5 {
        let lo = x.floor();
        if lo % 2.0 == 0.0 {
            lo
        } else {
            lo + 1.0
        }
    } else {
        x.round()
    }
}

/// Python `int(f)` truncation towards zero.
#[inline]
pub fn pyint(x: f64) -> i64 {
    x as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash16_matches_python_reference() {
        // Values produced by the Python implementation (see tools/oracle.py).
        assert_eq!(hash16(0), 58706);
        assert_eq!(hash16(1), 22068);
        assert_eq!(hash16(7), 43995);
        assert_eq!(hash16(-1), 35180);
        assert_eq!(hash16(1000), 58606);
        assert_eq!(hash16(123456789), 53349);
        // Python masks the (possibly negative) input to 32 bits first, which is
        // equivalent to `as i32` for values within and beyond the range.
        assert_eq!(hash16(1 << 31), 59727);
        assert_eq!(hash16(-(1 << 31)), 59727);
        assert_eq!(hash16(1 << 40), 58706);
        assert_eq!(hash16(-(1 << 40)), 58706);
    }

    #[test]
    fn pyround_is_banker_rounding() {
        assert_eq!(pyround(0.5), 0.0);
        assert_eq!(pyround(1.5), 2.0);
        assert_eq!(pyround(2.5), 2.0);
        assert_eq!(pyround(-0.5), -0.0);
        assert_eq!(pyround(-1.5), -2.0);
        assert_eq!(pyround(-2.5), -2.0);
        assert_eq!(pyround(-3.5), -4.0);
        assert_eq!(pyround(2.4), 2.0);
        assert_eq!(pyround(-2.6), -3.0);
    }
}

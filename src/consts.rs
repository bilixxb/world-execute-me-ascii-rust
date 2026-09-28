//! Project-wide numeric constants shared by the renderer.

/// Python `math.tau`.
pub const TAU: f64 = std::f64::consts::TAU;

/// The font's column advance, re-exported for scene code that builds rows of
/// bitmap characters by hand.
pub use crate::font::ADVANCE as FONT_ADVANCE;

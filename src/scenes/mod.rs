//! Lyric-driven choreography — Rust port of `world.execute-me-ascii/scenes.py`.
//!
//! # Layout of this module
//!
//! The original is a single 3073-line file. It is split here by time section so
//! that independent parts can be ported and reviewed separately:
//!
//! | module         | Python range | content                                    |
//! | -------------- | ------------ | ------------------------------------------ |
//! | `common`       | 1–127        | helpers, glitch engine, 3D projection      |
//! | `boot`         | 130–433      | boot sequence + simulation (t < 29.709)    |
//! | `devotion`     | 434–1274     | points → satisfaction (29.709 ≤ t < 74.045)|
//! | `organic`      | 1275–1600    | execution, heart, legacy organic (74–88.6) |
//! | `identity`     | 1601–2030    | identity, clock, role switch, vibration    |
//! | `isolation`    | 2029–2600    | isolation → execution orb                  |
//! | `love`         | 2604–2774    | love equation, trapped loop, outro         |
//! | `dispatch`     | 2777–3073    | title takeover, `draw_scene`, `phosphor`   |
//!
//! # Porting notes
//!
//! * Everything is free functions taking `&Canvas`, mirroring Python's `c`.
//!   Mutations go through `Canvas`'s interior mutability, so no `&mut` plumbing.
//! * `round()` **must** be [`crate::core::pyround`] (banker's rounding), never
//!   `f64::round`.
//! * `int()` truncates towards zero — use `as i64` on the float.
//! * `clamp(x, a, b)` is [`crate::core::clamp`]; the 1-arg form is
//!   [`crate::core::unit`].
//! * Python's negative-slice row rotation is [`Canvas::rotate_row`].
//! * `c.cells[y][x] = (ch, style)` becomes
//!   `c.set(x, y, Some(ch), style)`; reads use `c.get(x, y).ch`.
//! * Every scene function must produce byte-identical output to the Python
//!   original — verified against `tools/goldens.json` in `tests/golden.rs`.

pub mod boot;
pub mod common;
pub mod devotion;
pub mod dispatch;
pub mod identity;
pub mod isolation;
pub mod love;
pub mod organic;

pub use boot::*;
pub use common::*;
pub use devotion::*;
pub use dispatch::*;
pub use identity::*;
pub use isolation::*;
pub use love::*;
pub use organic::*;

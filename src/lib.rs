//! world.execute(me); — bilingual terminal MV.
//!
//! A Rust port of `world.execute-me-ascii`. The library exposes the renderer so
//! that its output can be diffed against the original Python byte-for-byte;
//! `main.rs` drives it interactively.

pub mod assets;
pub mod audio;
pub mod canvas;
pub mod consts;
pub mod core;
pub mod film;
pub mod font;
pub mod player;
pub mod scenes;
pub mod text;

pub use canvas::Canvas;
pub use consts::TAU;
pub use film::{Config, Film, LyricLine, Spectrum, CHAPTERS, SPECTRUM_BANDS};

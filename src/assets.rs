//! Compile-time bundled assets.
//!
//! The original loads these from its own directory (or unpacks them from the
//! single-file bundle). Embedding keeps one self-contained binary, matching the
//! bundled distribution's behaviour.

pub const LYRICS: &str = include_str!("../assets/lyrics.json");
pub const SPECTRUM: &str = include_str!("../assets/spectrum.json");
pub const CONFIG: &str = include_str!("../assets/config.json");
/// The bundled original song (Mili — world.execute(me);), as shipped inside
/// `world-execute-mv.pyz`.
pub const SONG: &[u8] = include_bytes!("../assets/song.mp3");

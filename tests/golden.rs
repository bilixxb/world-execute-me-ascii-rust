//! Differential test against the original Python renderer.
//!
//! `tools/goldens.json` is produced by `tools/gen_goldens.py`, which renders
//! each frame with the **unmodified** `world.execute-me-ascii/player.py` in a
//! fresh process and records the exact ANSI byte stream. This test renders the
//! same frames with the Rust port and compares byte-for-byte.
//!
//! Regenerate with:
//! ```text
//! python tools/gen_goldens.py
//! ```

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Deserialize;
use world_execute_me_rust::Film;

#[derive(Deserialize)]
struct Golden {
    t: f64,
    w: i64,
    h: i64,
    #[serde(default)]
    paused: bool,
    #[serde(default)]
    help: bool,
    #[serde(default)]
    ready: bool,
    #[serde(default)]
    offset: f64,
    ansi: String,
}

fn goldens_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tools/goldens.json")
}

fn load() -> BTreeMap<String, Golden> {
    let raw = std::fs::read_to_string(goldens_path())
        .expect("tools/goldens.json missing — run `python tools/gen_goldens.py`");
    serde_json::from_str(&raw).expect("tools/goldens.json is malformed")
}

fn first_difference(a: &str, b: &str) -> String {
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    let mut i = 0;
    while i < ab.len() && i < bb.len() && ab[i] == bb[i] {
        i += 1;
    }
    // Back up to a character boundary before slicing, so multi-byte glyphs in
    // the context window do not panic.
    let lo = (0..=i.saturating_sub(80))
        .rev()
        .find(|&k| a.is_char_boundary(k))
        .unwrap_or(0);
    let hi_a = (i + 80).min(a.len());
    let hi_a = (0..=hi_a)
        .rev()
        .find(|&k| a.is_char_boundary(k))
        .unwrap_or(0);
    let hi_b = (i + 80).min(b.len());
    let hi_b = (0..=hi_b)
        .rev()
        .find(|&k| b.is_char_boundary(k))
        .unwrap_or(0);
    format!(
        "first difference at byte {i} (expected len {}, got len {})\n  expected: {:?}\n  actual:   {:?}",
        ab.len(),
        bb.len(),
        &a[lo..hi_a],
        &b[lo..hi_b],
    )
}

fn render_case(film: &Film, g: &Golden) -> String {
    film.render(g.t, g.w, g.h, g.paused, g.offset, g.help, g.ready)
        .ansi()
}

#[test]
fn matches_python_renderer_byte_for_byte() {
    let film = Film::load().expect("assets failed to load");
    let goldens = load();
    let mut failures: Vec<String> = Vec::new();

    for (id, g) in &goldens {
        let got = render_case(&film, g);
        if got != g.ansi {
            failures.push(format!(
                "{id} (t={}, w={}, h={}, paused={}, help={}, ready={}, offset={})\n{}",
                g.t,
                g.w,
                g.h,
                g.paused,
                g.help,
                g.ready,
                g.offset,
                first_difference(&g.ansi, &got),
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} golden frames diverged from the Python original:\n\n{}",
        failures.len(),
        goldens.len(),
        failures.join("\n\n"),
    );
}

/// The renderer must be deterministic: the same frame rendered twice (and after
/// rendering other frames, which exercise the internal caches) must be equal.
#[test]
fn rendering_is_deterministic_across_cache_state() {
    let film = Film::load().expect("assets failed to load");
    let goldens = load();
    let ids: Vec<&String> = goldens.keys().collect();
    let mut mismatch = Vec::new();

    for id in ids.iter().step_by(37) {
        let g = &goldens[*id];
        let first = render_case(&film, g);
        // Render unrelated frames in between to perturb any internal caches.
        for other in goldens.values().take(5) {
            let _ = render_case(&film, other);
        }
        let again = render_case(&film, g);
        if first != again {
            mismatch.push((*id).clone());
        }
    }

    assert!(
        mismatch.is_empty(),
        "rendering is not deterministic for frames: {mismatch:?}"
    );
}

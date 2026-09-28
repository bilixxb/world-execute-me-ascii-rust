//! Diagnostic: report per-row differences between the Rust renderer and the
//! Python goldens, in plain-character space (ANSI stripped).
//!
//! `cargo run --release --example diff_goldens -- [max_frames] [only_id]`

use std::collections::BTreeMap;

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

/// Strip ANSI, splitting rows on `\x1b[{n};1H` cursor addressing.
fn strip_ansi(s: &str, h: i64) -> Vec<Vec<char>> {
    let mut cur = 0usize;
    let mut rows: Vec<Vec<char>> = vec![Vec::new(); h.max(0) as usize];
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c != '\x1b' {
            if cur < rows.len() {
                rows[cur].push(c);
            }
            continue;
        }
        if it.peek() != Some(&'[') {
            continue;
        }
        it.next();
        let mut params = String::new();
        let mut final_byte = ' ';
        for d in it.by_ref() {
            if ('\x40'..='\x7e').contains(&d) {
                final_byte = d;
                break;
            }
            params.push(d);
        }
        if final_byte == 'H' {
            let n: usize = params
                .split(';')
                .next()
                .and_then(|p| p.parse().ok())
                .unwrap_or(1);
            cur = n.saturating_sub(1);
        }
    }
    rows
}

fn window(chars: &[char], at: usize) -> String {
    let lo = at.saturating_sub(30);
    let hi = (at + 50).min(chars.len());
    chars[lo..hi].iter().collect()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let max: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(4);
    let only: Option<String> = args.next();

    let film = Film::load().expect("assets");
    let raw = std::fs::read_to_string("tools/goldens.json").expect("goldens");
    let goldens: BTreeMap<String, Golden> = serde_json::from_str(&raw).expect("parse");

    let mut shown = 0;
    let mut total = 0;
    for (id, g) in &goldens {
        if let Some(want) = &only {
            if !id.contains(want.as_str()) {
                continue;
            }
        }
        let got = film
            .render(g.t, g.w, g.h, g.paused, g.offset, g.help, g.ready)
            .ansi();
        if got == g.ansi {
            continue;
        }
        total += 1;
        if shown >= max {
            continue;
        }
        shown += 1;
        let want = strip_ansi(&g.ansi, g.h);
        let have = strip_ansi(&got, g.h);
        println!("=== {id} t={} w={} h={} ===", g.t, g.w, g.h);
        let mut bad_rows = 0;
        for y in 0..want.len().max(have.len()) {
            let empty = Vec::new();
            let a = want.get(y).unwrap_or(&empty);
            let b = have.get(y).unwrap_or(&empty);
            if a == b {
                continue;
            }
            bad_rows += 1;
            if bad_rows <= 4 {
                let mut first = 0;
                while first < a.len() && first < b.len() && a[first] == b[first] {
                    first += 1;
                }
                println!("  row {y} differs at col {first} (len {} vs {})", a.len(), b.len());
                println!("    PY: {:?}", window(a, first));
                println!("    RS: {:?}", window(b, first));
            }
        }
        println!("  differing rows: {bad_rows}");
    }
    println!("total diverging frames: {total} / {}", goldens.len());
}

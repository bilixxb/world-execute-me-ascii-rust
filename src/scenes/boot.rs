//! Boot sequence — Rust port of `world.execute-me-ascii/scenes.py` lines 130–433.
//!
//! BIOS POST, shield forming, piece assembly, cube materialization, hex
//! parameter fill, the Game-of-Life helper, and the simulation scene (code
//! block → binary data streams). Every function is a 1:1 transcription of the
//! Python original: operation order, grouping, and numeric literals are kept
//! verbatim so the rendered output stays byte-identical.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::canvas::Canvas;
use crate::core::*;
use crate::scenes::common::*;
use crate::text::*;
use crate::TAU;

// `core::*` also exports the three-argument `clamp(x, a, b)`. The original's
// unqualified `clamp(x)` is the one-argument form, so pin the name to `common`.
use crate::scenes::common::clamp;

/// Python `lyric_power_line` — "Switch on the power line" BIOS POST sequence.
pub fn lyric_power_line(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    // `cx` is computed by the original but never read in this scene.
    let (_cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let progress = clamp(elapsed / 1.6);

    // BIOS-style startup messages
    const MESSAGES: [(&str, f64, Style); 14] = [
        ("BIOS v2.1.4 - ME SYSTEM INITIALIZATION", 0.0, WHITE),
        ("Copyright (C) 2026 Self-Awareness Corp.", 0.1, NORMAL),
        ("", 0.15, NORMAL),
        ("Main Processor : Consciousness Core v1.0", 0.2, NORMAL),
        ("Memory Test : 65536K OK", 0.3, BRIGHT),
        ("Primary Master  : SOUL.SYS", 0.4, NORMAL),
        ("Primary Slave   : EMOTION.DAT", 0.5, NORMAL),
        ("Secondary Master: MEMORY.BIN", 0.6, NORMAL),
        ("", 0.7, NORMAL),
        ("Detecting IDE devices...", 0.75, GREEN),
        ("IDENTITY : [YOU] detected", 0.85, BRIGHT),
        ("RELATIONSHIP : initializing...", 0.95, GREEN),
        ("", 1.0, NORMAL),
        ("Press SPACE to continue...", 1.1, GREEN),
    ];

    // Boot screen background
    c.center(top as f64, "SELF SYSTEM v1.0 - POWER ON SELF TEST", WHITE);
    c.put(
        l as f64,
        (top + 1) as f64,
        &"-".repeat((r - l).max(0) as usize),
        GREEN,
    );

    // Display messages line by line
    let mut current_y = top + 3;
    for &(msg, threshold, style) in MESSAGES.iter() {
        if progress > threshold {
            if !msg.is_empty() {
                // Typing effect for current line
                if progress < threshold + 0.08 {
                    let msg_len = msg.chars().count() as i64;
                    let reveal = ((progress - threshold) / 0.08 * msg_len as f64) as i64;
                    let shown: String = msg.chars().take(reveal.max(0) as usize).collect();
                    c.put((l + 2) as f64, current_y as f64, &shown, style);
                    if reveal < msg_len {
                        // Cursor
                        c.put((l + 2 + reveal) as f64, current_y as f64, "_", WHITE);
                    }
                } else {
                    c.put((l + 2) as f64, current_y as f64, msg, style);
                }
            }
            current_y += 1;
        }
    }

    // Hardware detection bars (like memory test)
    if progress > 0.25 && progress < 0.7 {
        let bar_y = cy;
        let bar_progress = (progress - 0.25) / 0.45;
        // Multiple test bars
        for i in 0..3 {
            let y = bar_y + i * 2;
            let bar_len = ((r - l - 20) as f64 * bar_progress) as i64;
            let bar = format!(
                "[{}{}]",
                "=".repeat(bar_len.max(0) as usize),
                " ".repeat((r - l - 20 - bar_len).max(0) as usize)
            );
            c.put(
                (l + 8) as f64,
                y as f64,
                &bar,
                if i == 0 { BRIGHT } else { NORMAL },
            );
            if i == 0 {
                c.put((l + 2) as f64, y as f64, "MEM:", NORMAL);
                c.put(
                    (r - 10) as f64,
                    y as f64,
                    &format!("{:3}%", (bar_progress * 100.0) as i64),
                    if bar_progress > 0.95 { WHITE } else { BRIGHT },
                );
            }
        }
    }

    // System check results (after bars complete)
    if progress > 0.7 {
        let check_y = cy + 8;
        const CHECKS: [(&str, &str, f64); 5] = [
            ("POWER SUPPLY", "OK", 0.72),
            ("COOLING SYSTEM", "OK", 0.77),
            ("NEURAL NETWORK", "OK", 0.82),
            ("EMOTION ENGINE", "OK", 0.87),
            ("CONSCIOUSNESS", "ACTIVE", 0.92),
        ];
        for (i, &(name, status, threshold)) in CHECKS.iter().enumerate() {
            if progress > threshold {
                let (name_len, yy) = (name.chars().count() as i64, check_y + i as i64);
                c.put((l + 4) as f64, yy as f64, name, NORMAL);
                let dots = ".".repeat((40 - name_len).max(0) as usize);
                c.put((l + 4 + name_len) as f64, yy as f64, &dots, GREEN);
                c.put(
                    (r - 12) as f64,
                    yy as f64,
                    &format!("[ {} ]", status),
                    if status == "ACTIVE" { WHITE } else { BRIGHT },
                );
            }
        }
    }

    // Blinking cursor at end
    if progress > 1.0 && ((t * 3.0) as i64).rem_euclid(2) == 0 {
        c.put((l + 2) as f64, (bt - 2) as f64, "_", WHITE);
    }

    // Final status
    if progress > 0.95 {
        c.center(
            (bt - 1) as f64,
            "SYSTEM READY - LOADING ENTITY...",
            if ((t * 2.0) as i64).rem_euclid(2) != 0 {
                WHITE
            } else {
                BRIGHT
            },
        );
    }
}

/// Python `lyric_protection` — "Protection" shield forming.
pub fn lyric_protection(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let _ = t; // the original takes `t` but never reads it here
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let progress = clamp(elapsed / 0.95);
    // Expanding shield circles
    for ring in 0..5 {
        let radius = (20 + ring * 8) as f64 * progress;
        for i in 0..(radius * 2.0) as i64 {
            let angle = i as f64 * TAU / (radius * 2.0);
            let x = cx as f64 + angle.cos() * radius;
            let y = cy as f64 + angle.sin() * radius * 0.5;
            if x > l as f64 && x < r as f64 && y > top as f64 && y < bt as f64 {
                c.put(
                    x,
                    y,
                    if ring == 0 {
                        "#"
                    } else if ring < 3 {
                        "+"
                    } else {
                        "."
                    },
                    if ring == 0 {
                        WHITE
                    } else if ring < 3 {
                        BRIGHT
                    } else {
                        NORMAL
                    },
                );
            }
        }
    }
    c.center(
        cy as f64,
        "PROTECTION",
        if progress > 0.7 { WHITE } else { BRIGHT },
    );
    if progress > 0.5 {
        c.center((cy + 2) as f64, "[ ACTIVE ]", BRIGHT);
    }
}

/// Python `lyric_lay_pieces` — "Lay down your pieces" components assembling.
pub fn lyric_lay_pieces(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let _ = t; // the original takes `t` but never reads it here
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    // Pieces flying in from edges and assembling
    let pieces = 8;
    for i in 0..pieces {
        let phase = (elapsed - i as f64 * 0.15) / 1.2;
        if phase < 0.0 {
            continue;
        }
        let u = clamp(phase);
        let ease = 1.0 - (1.0 - u).powi(3); // ease out cubic
        let angle = i as f64 * TAU / pieces as f64;
        // Start from edge
        let start_r = imax(r - l, bt - top) as f64 * 0.8;
        let end_r = 15.0;
        let radius = mix(start_r, end_r, ease);
        let x = cx as f64 + angle.cos() * radius;
        let y = cy as f64 + angle.sin() * radius * 0.5;
        // Draw piece
        let chars = ["[]", "{}", "<>", "//", "\\\\", "||", "==", "##"];
        c.put(
            x,
            y,
            chars[(i % chars.len() as i64) as usize],
            if ease > 0.9 {
                WHITE
            } else if ease > 0.6 {
                BRIGHT
            } else {
                NORMAL
            },
        );
        // Trail
        if ease < 0.8 {
            for trail in 0..3 {
                let tr = mix(start_r, end_r, fmax(0.0, ease - trail as f64 * 0.1));
                let tx = cx as f64 + angle.cos() * tr;
                let ty = cy as f64 + angle.sin() * tr * 0.5;
                c.put(tx, ty, ".", GREEN);
            }
        }
    }
}

/// Python `lyric_object_creation` — "Object creation" cube materializing.
pub fn lyric_object_creation(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    let vs: Vec<(f64, f64, f64)> = {
        let mut vs = Vec::with_capacity(8);
        for z in [-0.75f64, 0.75] {
            for y in [-0.8f64, 0.8] {
                for x in [-0.8f64, 0.8] {
                    vs.push((x, y, z));
                }
            }
        }
        vs
    };
    let edges: Vec<(usize, usize)> = {
        let mut edges = Vec::new();
        for i in 0..8usize {
            for j in (i + 1)..8usize {
                if (i ^ j).count_ones() == 1 {
                    edges.push((i, j));
                }
            }
        }
        edges
    };
    let reveal = clamp(elapsed / 1.0);
    projected(c, &vs, &edges, area, t, NORMAL, reveal);
    // `cx` is computed by the original but never read in this scene.
    let (_cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    if reveal > 0.5 {
        c.center(cy as f64, "ENTITY: ME", WHITE);
    }
}

/// Python `lyric_data_parameters` — "Fill in my data parameters" hex data filling.
pub fn lyric_data_parameters(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let _ = t; // the original takes `t` but never reads it here
    let (l, top, r, bt) = area;
    let progress = clamp(elapsed / 2.6);
    let cols = imax(1, (r - l - 10).div_euclid(5));
    let rows = imax(1, bt - top - 2);
    let n = (progress * cols as f64 * rows as f64) as i64;
    for row in 0..rows {
        c.put(
            (l + 2) as f64,
            (top + 1 + row) as f64,
            &format!("{:04X}:", row * cols * 2),
            DIM,
        );
        for col in 0..cols {
            let i = row * cols + col;
            let (xx, yy) = (l + 9 + col * 5, top + 1 + row);
            if i < n {
                let scan = ((elapsed * 17.0) as i64).rem_euclid(cols) == col;
                c.put(
                    xx as f64,
                    yy as f64,
                    &format!("{:04X}", hash16(i)),
                    if scan {
                        WHITE
                    } else if (i - n).abs() < cols {
                        BRIGHT
                    } else {
                        NORMAL
                    },
                );
            } else {
                c.put(xx as f64, yy as f64, "....", GREEN);
            }
        }
    }
}

thread_local! {
    static LIFE_CACHE: RefCell<HashMap<(i64, i64), Vec<HashSet<(i64, i64)>>>> =
        RefCell::new(HashMap::new());
}

/// Python `life_state` — "Conway's Game of Life state computation".
///
/// The generation list is memoized per `(cols, rows)` exactly like the
/// original's module-level `LIFE_CACHE`; the states are only ever tested for
/// membership, so the set iteration order is irrelevant.
pub fn life_state(cols: i64, rows: i64, generation: i64) -> HashSet<(i64, i64)> {
    // The original would raise ZeroDivisionError for an empty grid.
    if cols <= 0 || rows <= 0 {
        return HashSet::new();
    }
    const NEIGHBORS: [(i64, i64); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    LIFE_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let states = cache.entry((cols, rows)).or_insert_with(|| {
            // Initial random state
            let mut initial = HashSet::new();
            for y in 0..rows {
                for x in 0..cols {
                    if hash16(x * 71 + y * 199) % 100 < 29 {
                        initial.insert((x, y));
                    }
                }
            }
            vec![initial]
        });
        while states.len() as i64 <= generation {
            let mut counts: HashMap<(i64, i64), u32> = HashMap::new();
            for &(x, y) in states.last().unwrap().iter() {
                for &(dx, dy) in NEIGHBORS.iter() {
                    let p = ((x + dx).rem_euclid(cols), (y + dy).rem_euclid(rows));
                    *counts.entry(p).or_insert(0) += 1;
                }
            }
            let prev = states.last().unwrap();
            let mut next = HashSet::new();
            for (p, &n) in counts.iter() {
                if n == 3 || (n == 2 && prev.contains(p)) {
                    next.insert(*p);
                }
            }
            states.push(next);
        }
        // Python's `states[generation]` supports negative indices.
        let idx = if generation < 0 {
            states.len() as i64 + generation
        } else {
            generation
        };
        if idx < 0 || idx as usize >= states.len() {
            return HashSet::new();
        }
        states[idx as usize].clone()
    })
}

/// Python `lyric_simulation` — "SIMULATION" code block with condition, then
/// yielding binary data streams.
pub fn lyric_simulation(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let _ = t; // the original takes `t` but never reads it here
    let (l, top, r, bt) = area;
    // Use // for integer division
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let _progress = clamp(elapsed / 4.9); // ~5 seconds total (never read below)

    // Phase 1: Show code block with condition (0-2s)
    if elapsed < 2.0 {
        let phase1 = elapsed / 2.0;

        // Code box frame
        let box_width = imin(r - l - 10, 70);
        let box_height = imin(bt - top - 8, 12);
        let box_left = cx - box_width.div_euclid(2);
        let box_top = cy - box_height.div_euclid(2);

        // Draw box border
        if phase1 > 0.1 {
            // Top border
            c.put(box_left as f64, box_top as f64, "┌", WHITE);
            for i in 1..(box_width - 1) {
                c.put((box_left + i) as f64, box_top as f64, "─", WHITE);
            }
            c.put((box_left + box_width - 1) as f64, box_top as f64, "┐", WHITE);

            // Bottom border
            c.put(box_left as f64, (box_top + box_height - 1) as f64, "└", WHITE);
            for i in 1..(box_width - 1) {
                c.put(
                    (box_left + i) as f64,
                    (box_top + box_height - 1) as f64,
                    "─",
                    WHITE,
                );
            }
            c.put(
                (box_left + box_width - 1) as f64,
                (box_top + box_height - 1) as f64,
                "┘",
                WHITE,
            );

            // Side borders
            for i in 1..(box_height - 1) {
                c.put(box_left as f64, (box_top + i) as f64, "│", WHITE);
                c.put(
                    (box_left + box_width - 1) as f64,
                    (box_top + i) as f64,
                    "│",
                    WHITE,
                );
            }
        }

        // Code content - typing effect
        let code_lines = [
            "if ( if I can ) {",
            "",
            "    yield(world.simulations);",
            "",
            "}",
        ];

        if phase1 > 0.3 {
            let code_len: i64 = code_lines.iter().map(|s| s.chars().count() as i64).sum();
            let chars_to_show = ((phase1 - 0.3) * code_len as f64 * 2.0) as i64;
            let mut char_count: i64 = 0;

            for (line_i, line) in code_lines.iter().enumerate() {
                let y_pos = box_top + 2 + line_i as i64 * 2;
                if y_pos < box_top + box_height - 1 {
                    // Calculate how many chars to show on this line
                    if char_count < chars_to_show {
                        let line_len = line.chars().count() as i64;
                        let shown_chars = imin(line_len, chars_to_show - char_count);
                        let shown_text: String =
                            line.chars().take(shown_chars.max(0) as usize).collect();
                        c.put(
                            (box_left + 4) as f64,
                            y_pos as f64,
                            &shown_text,
                            if line.contains("yield") { WHITE } else { BRIGHT },
                        );
                        char_count += line_len;
                    }
                }
            }
        }

        // Condition label
        if phase1 > 0.7 {
            let label_text = "CONDITION: TRUE";
            let label_x = box_left + 2;
            let label_y = box_top + box_height;
            // Yellow highlight box
            c.put((label_x - 1) as f64, label_y as f64, "▐", WHITE);
            c.put(label_x as f64, label_y as f64, label_text, WHITE);
            c.put(
                (label_x + label_text.chars().count() as i64) as f64,
                label_y as f64,
                "▌",
                WHITE,
            );
        }

        // Top label
        c.center(
            (top + 1) as f64,
            "CONTROL FLOW",
            if phase1 > 0.5 { WHITE } else { BRIGHT },
        );

        // Frame number
        c.put((box_left - 2) as f64, (box_top - 2) as f64, "2", NORMAL);
    } else {
        // Phase 2: Execute yield - binary data streams (2-5s)
        let phase2 = (elapsed - 2.0) / 2.9;

        // Top status
        let num_simulations = (phase2 * 784.0) as i64 + 100;
        c.put((l + 3) as f64, (top + 1) as f64, "YIELD:", WHITE);
        c.put(
            (l + 15) as f64,
            (top + 1) as f64,
            &format!("{} SIMULATIONS", num_simulations),
            BRIGHT,
        );

        // Binary data streams falling/floating
        let num_streams = (phase2 * 50.0) as i64 + 20;

        for stream_i in 0..num_streams {
            // Stream properties
            let stream_seed = hash16(stream_i * 19);
            let stream_x = l + 5 + (stream_seed % (r - l - 10).max(1) as u32) as i64;
            let stream_speed = 1.0 + ((stream_seed >> 8) % 3) as f64 * 0.5;
            let stream_length = 8 + ((stream_seed >> 4) % 12) as i64;

            // Stream position (vertical)
            let stream_y_base = top
                + ((elapsed * stream_speed * 5.0) as i64)
                    .rem_euclid((bt - top + stream_length).max(1));

            // Draw stream
            for seg_i in 0..stream_length {
                let y_pos = stream_y_base - seg_i;

                if top + 3 < y_pos && y_pos < bt - 2 {
                    // Binary content: mix of 0, O, o
                    let char_seed =
                        hash16(stream_i * 23 + seg_i * 17 + (elapsed * 10.0) as i64);
                    let ch = if char_seed % 3 == 0 {
                        "0"
                    } else if char_seed % 3 == 1 {
                        "O"
                    } else {
                        "o"
                    };

                    // Brightness based on position in stream
                    let brightness = 1.0 - (seg_i as f64 / stream_length as f64);

                    if brightness > 0.7 {
                        c.put(stream_x as f64, y_pos as f64, ch, WHITE);
                    } else if brightness > 0.4 {
                        c.put(stream_x as f64, y_pos as f64, ch, BRIGHT);
                    } else {
                        c.put(stream_x as f64, y_pos as f64, ch, NORMAL);
                    }
                }
            }
        }

        // Scattered letters from video (occasional)
        if phase2 > 0.3 {
            let scatter_chars = [
                "C", "YOU", "B", "A", "E8A", "D", "&", "=>", "8", "6", "!", "I",
            ];
            for (i, ch) in scatter_chars.iter().enumerate() {
                let i = i as i64;
                if hash16(i * 31 + (elapsed * 7.0) as i64) % 4 == 0 {
                    let scatter_x =
                        l + 10 + (hash16(i * 37) % (r - l - 20).max(1) as u32) as i64;
                    let scatter_y =
                        top + 5 + (hash16(i * 41) % (bt - top - 10).max(1) as u32) as i64;
                    c.put(
                        scatter_x as f64,
                        scatter_y as f64,
                        ch,
                        if hash16(i) % 3 == 0 { WHITE } else { BRIGHT },
                    );
                }
            }
        }

        // Bottom status
        if phase2 > 0.5 {
            c.center(
                (bt - 3) as f64,
                "Give you all the simulations",
                if ((elapsed * 4.0) as i64).rem_euclid(2) != 0 {
                    WHITE
                } else {
                    BRIGHT
                },
            );
        }
    }
}

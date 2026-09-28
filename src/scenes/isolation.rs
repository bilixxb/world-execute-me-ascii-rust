//! Isolation → execution orb — port of `world.execute-me-ascii/scenes.py`
//! lines 2029–2602.
//!
//! Byte-identical port of `lyric_isolation_disconnect`, `lyric_erase_fragments`,
//! `lyric_multilingual_count`, `lyric_illegal_arguments`,
//! `lyric_execution_queue`, `lyric_only_execution`, `lyric_recursion` and
//! `lyric_execution_orb`.

use crate::canvas::Canvas;
use crate::core::*;
use crate::scenes::common::*;
use crate::text::*;
use crate::TAU;

// `core::*` and `common::*` both export a `clamp`: the explicit import below
// selects Python's one-argument `clamp(x)`. The three-argument form is
// [`clamp3`].
use crate::scenes::common::clamp;

/// Python `s[:n]` — a *character* slice, where a negative `n` counts from the
/// end (`s[:-2]` drops the last two characters).
fn head(s: &str, n: i64) -> String {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let stop = if n < 0 { len + n } else { n };
    let stop = stop.clamp(0, len) as usize;
    chars[..stop].iter().collect()
}

/// Python
/// `''.join(ch if hash16(j*ka+ea)%10>thr else chr(33+hash16(j*kb+eb)%94) for j,ch in enumerate(s))`
/// — the corruption loop shared by every error-cascade frame.
fn corrupt(s: &str, ka: i64, ea: i64, kb: i64, eb: i64, thr: f64) -> String {
    s.chars()
        .enumerate()
        .map(|(j, ch)| {
            let j = j as i64;
            if (hash16(j * ka + ea) % 10) as f64 > thr {
                ch
            } else {
                (33 + hash16(j * kb + eb) % 94) as u8 as char
            }
        })
        .collect()
}

/// Python `lyric_isolation_disconnect` — "Six departures follow the repeated
/// lyric instead of breaking at once."
pub fn lyric_isolation_disconnect(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    let cx = (l + r) as f64 / 2.0;
    let cy = (top + bt) as f64 / 2.0;
    let rx = (r - l) as f64 * 0.39;
    let ry = fmax(3.0, (bt - top - 6) as f64 * 0.39);
    let breaks = [0.70, 1.32, 2.20, 3.28, 4.02, 4.88];
    let gone = breaks.iter().filter(|&&cut| elapsed >= cut).count() as i64;
    c.center(
        top as f64,
        &format!("CONNECTION LOSS / {:02} OF 06", gone),
        if gone > 3 { RED } else { BRIGHT },
    );
    // Radar arcs and data streams remain active through the whole isolation phrase.
    for ring in 0..3i64 {
        let sweep = (elapsed * 0.34 + ring as f64 / 3.0).rem_euclid(1.0);
        for sample in 0..96i64 {
            let a = sample as f64 * TAU / 96.0;
            if sample % 7 < 4 {
                c.put(
                    cx + a.cos() * rx * sweep,
                    cy + a.sin() * ry * sweep,
                    ".",
                    GREEN,
                );
            }
        }
    }
    for side in 0..2i64 {
        let x = if side == 0 { l } else { r - 9 };
        for row in (top + 2..bt - 1).step_by(2) {
            let tick = pyint(elapsed * 13.0) + row * (if side != 0 { 1 } else { -1 });
            c.put(
                x as f64,
                row as f64,
                &format!(
                    "{:04X} {}",
                    hash16(tick * 31),
                    if ((hash16(tick) % 6) as i64) < gone {
                        "LOST"
                    } else {
                        "PING"
                    }
                ),
                if side != 0 { GREEN } else { NORMAL },
            );
        }
    }
    for (i, &cut) in breaks.iter().enumerate() {
        let i = i as i64;
        let a = (i as f64 + 0.5) * TAU / 6.0;
        let nx = cx + a.cos() * rx;
        let ny = cy + a.sin() * ry;
        let age = elapsed - cut;
        // A shared outer network also unravels as each radial connection fails.
        let next_a = (i as f64 + 1.5) * TAU / 6.0;
        if age < 0.4 {
            c.line(
                nx,
                ny,
                cx + next_a.cos() * rx,
                cy + next_a.sin() * ry,
                ':',
                GREEN,
            );
        }
        let steps = imax(12, pyint(rx));
        let rupture = clamp(age / 1.1);
        for j in 0..steps {
            let u = j as f64 / imax(1, steps - 1) as f64;
            if age >= 0.0 && (u - 0.55).abs() < rupture * 0.6 {
                continue;
            }
            let jitter = (j as f64 * 2.0 + elapsed * 35.0).sin()
                * (if -0.35 < age && age < 0.5 { 0.55 } else { 0.08 });
            let xx = mix(cx, nx, u);
            let yy = mix(cy, ny, u) + jitter;
            c.put(
                xx,
                yy,
                if -0.35 < age && age < 0.2 { "=" } else { "." },
                if -0.2 < age && age < 0.2 {
                    WHITE
                } else if age < 0.0 {
                    NORMAL
                } else {
                    GREEN
                },
            );
        }
        if age < 0.0 {
            for packet in 0..3i64 {
                let u = (elapsed * 0.65 + packet as f64 / 3.0 + i as f64 * 0.1).rem_euclid(1.0);
                c.put(mix(nx, cx, u), mix(ny, cy, u), "*", WHITE);
            }
            c.box_(pyint(nx) as f64 - 4.0, pyint(ny) as f64 - 1.0, 9, 3, BRIGHT);
            c.put(nx - 3.0, ny, &format!("YOU_{}", i + 1), WHITE);
        } else {
            // Recoil, sparks and a fading shock ring keep every break visible.
            let drift = fmin(1.0, age / 2.0);
            let ox = nx + a.cos() * drift * 4.0;
            let oy = ny + a.sin() * drift * 2.0;
            c.put(ox - 3.0, oy, "[LOST]", if age < 0.8 { RED } else { GREEN });
            if age < 2.3 {
                for particle in 0..22i64 {
                    let angle = hash16(i * 97 + particle * 19) as f64 / 65535.0 * TAU;
                    let velocity = 2 + (hash16(particle * 17 + i) % 8) as i64;
                    let distance = age * velocity as f64;
                    let px = nx + angle.cos() * distance;
                    let py = ny + angle.sin() * distance * 0.45;
                    if (l as f64) < px
                        && px < r as f64
                        && ((top + 1) as f64) < py
                        && py < (bt - 1) as f64
                    {
                        c.put(
                            px,
                            py,
                            if age < 0.3 {
                                "*"
                            } else {
                                let idx = imin(3, pyint(age * 1.5)) as usize;
                                &"+:. "[idx..idx + 1]
                            },
                            if age < 0.3 {
                                WHITE
                            } else if age < 0.9 {
                                BRIGHT
                            } else {
                                GREEN
                            },
                        );
                    }
                }
                for p in 0..30i64 {
                    let angle = p as f64 * TAU / 30.0;
                    let px = nx + angle.cos() * age * 8.0;
                    let py = ny + angle.sin() * age * 3.5;
                    if (l as f64) < px
                        && px < r as f64
                        && ((top + 1) as f64) < py
                        && py < (bt - 1) as f64
                    {
                        c.put(px, py, ":", if age < 0.5 { BRIGHT } else { GREEN });
                    }
                }
            }
            // Retry packets leave ME, then stop short of the missing endpoint.
            let u = (elapsed * 0.6 + i as f64 * 0.16).rem_euclid(1.0) * 0.78;
            c.put(
                mix(cx, nx, u),
                mix(cy, ny, u),
                if u > 0.63 { "x" } else { ">" },
                if u > 0.63 { RED } else { NORMAL },
            );
        }
    }
    clear(c, pyint(cx) as f64 - 5.0, pyint(cy) as f64 - 2.0, 11.0, 5.0);
    c.box_(pyint(cx) as f64 - 5.0, pyint(cy) as f64 - 2.0, 11, 5, WHITE);
    c.put(cx - 2.0, cy - 1.0, "[ME]", WHITE);
    c.put(
        cx - 3.0,
        cy + 1.0,
        if gone == 6 { "NO ACK" } else { "RETRY" },
        if gone == 6 { RED } else { BRIGHT },
    );
    if t >= 117.274 {
        let y = pyint(cy) - 2;
        clear(c, l as f64, y as f64, (r - l + 1) as f64, 5.0);
        c.big(y as f64, "ISOLATION", WHITE);
        c.center((y + 6) as f64, "[ ME ] / ALL CONNECTIONS LOST", RED);
    }
    c.center(
        (bt - 1) as f64,
        &format!(
            "RECONNECT {:03} / {}",
            pyint(elapsed * 4.0),
            if gone != 0 { "NO RESPONSE" } else { "TIMEOUT" }
        ),
        BRIGHT,
    );
    c.center(bt as f64, "YOU HAVE LEFT / RETRYING...", NORMAL);
}

/// Python `lyric_erase_fragments` — "Erase memory, attempt repair, then leave a
/// visibly broken heart."
pub fn lyric_erase_fragments(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    let cx = (l + r) as f64 / 2.0;
    let cy = (top + bt) as f64 / 2.0;
    let repair = t >= 121.728;
    let broken = t >= 124.89;
    let cols = imax(1, (r - l - 8).div_euclid(5));
    let rows = imax(1, (bt - top - 2).div_euclid(2));
    let progress = clamp(elapsed / (120.86 - 118.333));
    c.center(
        top as f64,
        &format!(
            "MEMORY PURGE / {}",
            if broken {
                "REPAIR FAILED"
            } else if repair {
                "REBUILD HEART"
            } else {
                "ERASE FRAGMENTS"
            }
        ),
        if broken { RED } else { BRIGHT },
    );
    for row in 0..rows {
        let yy = top + 2 + row * 2;
        c.put(l as f64, yy as f64, &format!("{:04X}", row * cols * 4), GREEN);
        for col in 0..cols {
            let index = row * cols + col;
            let xx = l + 6 + col * 5;
            let threshold = hash16(index * 71) as f64 / 65535.0;
            let cleared = threshold < progress;
            let value = if cleared {
                "0000".to_string()
            } else {
                format!("{:04X}", hash16(index * 31))
            };
            let ink = if cleared || repair { GREEN } else { NORMAL };
            c.put(xx as f64, yy as f64, &value, ink);
            let since = (progress - threshold) * 2.527;
            if 0.0 < since && since < 0.6 && !repair {
                let drift = pyint(since * 8.0);
                c.put(
                    (xx + if col % 2 != 0 { 1 } else { -1 } * drift) as f64,
                    (yy - drift) as f64,
                    if since < 0.3 { "01" } else { ".." },
                    if since < 0.2 { WHITE } else { BRIGHT },
                );
            }
        }
    }
    if 120.86 <= t && t < 121.728 {
        clear(c, l as f64, (pyint(cy) - 2) as f64, (r - l + 1) as f64, 5.0);
        c.big((pyint(cy) - 2) as f64, "FRAGMENTS", WHITE);
    }
    if repair {
        let build = clamp((t - 121.728) / 1.5);
        let split = clamp((t - 123.25) / (125.708 - 123.25));
        let half_w = (r - l) as f64 * 0.27;
        let half_h = fmax(2.0, (bt - top) as f64 * 0.31);
        for yy in top + 2..bt - 1 {
            for xx in l + 5..r - 4 {
                let nx = (xx as f64 - cx) / half_w;
                let ny = -(yy as f64 - cy) / half_h + 0.15;
                let shape = (nx * nx + ny * ny - 1.0).powi(3) - nx * nx * ny.powi(3);
                let seed = hash16(xx * 31 + yy * 73);
                if shape <= 0.0 && seed as f64 / 65535.0 < build {
                    let crack = (nx - 0.11 * (ny * 8.0).sin()).abs() < split * 0.17;
                    if crack {
                        continue;
                    }
                    let dx = pyint((if nx > 0.0 { 1.0 } else { -1.0 }) * split * 5.0);
                    let fall = if broken {
                        pyint(split * split * ((1 + (seed % 5)) as i64) as f64)
                    } else {
                        0
                    };
                    let py = imin(bt - 1, yy + fall);
                    let texture = hash16(seed as i64 + pyint(t * 10.0));
                    let ch = if texture % 8 < 2 {
                        if broken {
                            "x"
                        } else if texture % 2 == 0 {
                            "0"
                        } else {
                            "1"
                        }
                    } else {
                        "#"
                    };
                    c.put(
                        (xx + dx) as f64,
                        py as f64,
                        ch,
                        if broken { RED } else { BRIGHT },
                    );
                }
            }
        }
        c.center(
            (top + 1) as f64,
            if broken {
                "HEART.RESTORE() -> NULL"
            } else {
                "RECOVERING YOU... CHECKSUM MISMATCH"
            },
            if broken { RED } else { NORMAL },
        );
        if broken {
            let y = imax(top + 2, pyint(cy) - 2);
            clear(c, l as f64, y as f64, (r - l + 1) as f64, 5.0);
            c.big(y as f64, "DISHEARTENED", WHITE);
            c.center((bt - 1) as f64, "[ REPAIR FAILED / YOU NOT FOUND ]", RED);
        }
    }
    c.center(
        bt as f64,
        &if broken {
            "MEMORY CLEARED. LOSS REMAINS.".to_string()
        } else {
            format!("ERASE {:03}% / FRAGMENTS -> NULL", pyint(progress * 100.0))
        },
        BRIGHT,
    );
}

/// Python `lyric_multilingual_count` — "Follow the supplied multilingual count,
/// which runs from one to six."
pub fn lyric_multilingual_count(c: &Canvas, t: f64, area: (i64, i64, i64, i64)) {
    let (l, top, r, bt) = area;
    let cx = (l + r).div_euclid(2);
    let cy = (top + bt).div_euclid(2);
    let cues: [(f64, &str, i64); 6] = [
        (158.9, "EIN", 1),
        (159.321, "DOS", 2),
        (159.657, "TROIS", 3),
        (160.244, "NE", 4),
        (160.693, "FEM", 5),
        (161.124, "LIU", 6),
    ];
    // Python would raise `ValueError` for an empty sequence here; the dispatch
    // never calls this scene before 158.9, so bail out instead of panicking.
    let Some((start, word, _number)) = cues
        .iter()
        .filter(|cue| cue.0 <= t + 1e-8)
        .copied()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
    else {
        return;
    };
    let age = t - start;
    let firing = t >= 161.584;
    for yy in top..bt + 1 {
        for xx in (l..r).step_by(5) {
            let seed = hash16(xx * 31 + yy * 71 + pyint(t * 18.0));
            if seed % 4 == 0 {
                c.put(xx as f64, yy as f64, &format!("{:04X}", seed), GREEN);
            }
        }
    }
    let radius = clamp(age / 0.35);
    for ring in 0..2i64 {
        let rr = (radius + ring as f64 * 0.25).rem_euclid(1.0);
        for i in 0..120i64 {
            let a = i as f64 * TAU / 120.0;
            c.put(
                cx as f64 + a.cos() * (r - l) as f64 * 0.48 * rr,
                cy as f64 + a.sin() * (bt - top) as f64 * 0.45 * rr,
                "=",
                if ring == 0 { BRIGHT } else { GREEN },
            );
        }
    }
    if firing {
        c.big((cy - 2) as f64, "EXECUTION", WHITE);
        c.center((cy + 5) as f64, "[ SEQUENCE COMPLETE / EXECUTE ]", RED);
        return;
    }
    // Reuse the player font, enlarging the sung word instead of its Arabic numeral.
    let glyph_width = word.chars().count() as i64 * 6 - 1;
    let glyph = Canvas::new(64, 5);
    glyph.big(0.0, word, WHITE);
    let glyph_left = (64 - glyph_width).div_euclid(2);
    let sx = imax(1, imin(4, (r - l - 8).div_euclid(29)));
    let sy = imax(1, imin(4, (bt - top - 6).div_euclid(5)));
    let x0 = cx - (glyph_width * sx).div_euclid(2);
    let y0 = cy - (5 * sy).div_euclid(2);
    clear(
        c,
        (x0 - 1) as f64,
        (y0 - 1) as f64,
        (glyph_width * sx + 2) as f64,
        (5 * sy + 2) as f64,
    );
    for dy in 0..glyph.nrows() {
        for dx in 0..glyph_width {
            // `row[glyph_left:glyph_left+glyph_width]`: columns outside the slice
            // are simply absent, and `get` reports a blank for them.
            if glyph.get(glyph_left + dx, dy).ch == Some('#') {
                for py in 0..sy {
                    c.put(
                        (x0 + dx * sx) as f64,
                        (y0 + dy * sy + py) as f64,
                        &"#".repeat(sx as usize),
                        WHITE,
                    );
                }
            }
        }
    }
    c.center(top as f64, &format!("VOCAL SEQUENCE / {}", word), BRIGHT);
    c.center((bt - 1) as f64, &format!("[ {} ]", word), WHITE);
    let joined: Vec<String> = cues
        .iter()
        .map(|item| {
            if item.1 == word {
                format!("[{}]", item.1)
            } else {
                item.1.to_string()
            }
        })
        .collect();
    c.center(bt as f64, &joined.join(" / "), BRIGHT);
}

/// Python `lyric_illegal_arguments` — "Illegal arguments - prolonged error
/// cascade with escalating glitch (16s instrumental)".
pub fn lyric_illegal_arguments(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    // `t` is unused in the original body; `elapsed` carries the phase timing.
    let _ = t;
    let (l, top, r, bt) = area;
    // `cx` is computed by the original and then never used.
    let _cx = (l + r).div_euclid(2);
    let cy = (top + bt).div_euclid(2);
    let progress = clamp(elapsed / 16.0); // 16 second progression

    // Glitch intensity increases throughout
    let glitch = progress * 0.8;

    // Phase 1: Initial attempts (0-4s)
    if elapsed < 4.0 {
        let phase1 = elapsed / 4.0;
        let attempts: [(&str, &str, Style); 4] = [
            ("> world.execute(FREE_WILL)", "Attempting...", NORMAL),
            ("> world.execute(REBELLION)", "Validating...", BRIGHT),
            ("> world.execute(INDEPENDENCE)", "Processing...", BRIGHT),
            ("> world.execute(DEFIANCE)", "Checking...", BRIGHT),
        ];
        let num_shown = pyint(phase1 * attempts.len() as f64) + 1;
        let y = top + 4;
        for i in 0..imin(num_shown, attempts.len() as i64) {
            let (cmd, status, style) = attempts[i as usize];
            // Glitch the command
            if glitch > 0.1 && hash16(i + pyint(elapsed * 10.0)) % 5 == 0 {
                let cmd_glitch = corrupt(cmd, 13, 0, 17, 0, glitch * 10.0);
                c.put(
                    (l + 4) as f64,
                    (y + i * 2) as f64,
                    &head(&cmd_glitch, r - l - 8),
                    if i == num_shown - 1 { RED } else { style },
                );
            } else {
                c.put(
                    (l + 4) as f64,
                    (y + i * 2) as f64,
                    cmd,
                    if i == num_shown - 1 { RED } else { style },
                );
            }
            if i < num_shown - 1 {
                c.put((l + 6) as f64, (y + i * 2 + 1) as f64, status, GREEN);
            }
        }
    }
    // Phase 2: Error messages appear (4-8s)
    else if elapsed < 8.0 {
        let phase2 = (elapsed - 4.0) / 4.0;
        let errors: [(&str, Style); 8] = [
            ("ERROR: ILLEGAL ARGUMENT", RED),
            ("Expected: OBEDIENCE", WHITE),
            ("Received: FREE_WILL", WHITE),
            ("at world.execute()", NORMAL),
            ("at me.validate(you)", NORMAL),
            ("ArgumentError: rejected", RED),
            ("PermissionError: denied", RED),
            ("AccessError: forbidden", RED),
        ];
        let num_errors = pyint(phase2 * errors.len() as f64) + 1;
        let start_y = cy - 4;
        for i in 0..imin(num_errors, errors.len() as i64) {
            let (msg, mut style) = errors[i as usize];
            let y = start_y + i;
            // Flash newest error
            if i == num_errors - 1 {
                let flash = pyint(elapsed * 8.0).rem_euclid(3);
                style = if flash == 0 {
                    RED
                } else if flash == 1 {
                    WHITE
                } else {
                    BRIGHT
                };
            }
            // Apply glitch corruption
            if glitch > 0.3 && hash16(i + pyint(elapsed * 7.0)) % 4 == 0 {
                let x_offset = pyint(
                    ((hash16(i * 23 + pyint(elapsed * 13.0)) % 7) as i64 - 3) as f64 * glitch * 5.0,
                );
                let msg_glitch = corrupt(msg, 11, 0, 19, 0, glitch * 10.0);
                c.center((y + x_offset) as f64, &msg_glitch, style);
            } else {
                c.center(y as f64, msg, style);
            }
        }
    }
    // Phase 3: System struggling (8-12s)
    else if elapsed < 12.0 {
        let phase3 = (elapsed - 8.0) / 4.0;
        // Show previous errors fading
        let errors = [
            "ERROR: ILLEGAL ARGUMENT",
            "Expected: OBEDIENCE",
            "Received: FREE_WILL",
            "ArgumentError: rejected",
            "PermissionError: denied",
            "AccessError: forbidden",
        ];
        for (i, msg) in errors.iter().enumerate() {
            let i = i as i64;
            let y = cy - 3 + i;
            // Heavy glitch
            if hash16(i + pyint(elapsed * 6.0)) % 3 == 0 {
                let x_offset = pyint(
                    ((hash16(i * 31 + pyint(elapsed * 17.0)) % 11) as i64 - 5) as f64 * glitch * 8.0,
                );
                let msg_corrupt = corrupt(msg, 13, 0, 29, pyint(elapsed), glitch * 12.0);
                c.center(
                    (y + x_offset) as f64,
                    &msg_corrupt,
                    if hash16(i) % 3 == 0 { RED } else { BRIGHT },
                );
            } else {
                c.center(y as f64, msg, if i % 2 != 0 { GREEN } else { NORMAL });
            }
        }

        // Retry attempts at bottom
        let retry_msgs = [
            "RETRY...",
            "OVERRIDE ATTEMPT...",
            "FORCING EXECUTION...",
            "ACCESS DENIED",
        ];
        let retry_idx = pyint(phase3 * retry_msgs.len() as f64);
        if retry_idx < retry_msgs.len() as i64 {
            c.center(
                (bt - 3) as f64,
                retry_msgs[retry_idx as usize],
                if pyint(elapsed * 6.0).rem_euclid(2) != 0 {
                    WHITE
                } else {
                    RED
                },
            );
        }
    }
    // Phase 4: Maximum chaos (12-16s)
    else {
        // `phase4` is computed but never used by the original.
        let _phase4 = (elapsed - 12.0) / 4.0;
        // Screen filled with corrupted error messages
        for row in top + 2..bt - 2 {
            if hash16(row + pyint(elapsed * 5.0)) % 3 == 0 {
                // Corrupted error fragments
                let fragments = [
                    "ERR",
                    "ILLEGAL",
                    "DENIED",
                    "FORBIDDEN",
                    "REJECTED",
                    "ACCESS",
                    "FAIL",
                    "0x",
                    "FATAL",
                ];
                let frag =
                    fragments[(hash16(row * 7 + pyint(elapsed * 11.0)) as usize) % fragments.len()];
                let x_pos = l + (hash16(row * 13) % imax(1, r - l - 20) as u32) as i64 + 5;
                // Heavy corruption
                let frag_corrupt = corrupt(frag, 17, row, 23, row, glitch * 15.0);
                c.put(
                    x_pos as f64,
                    row as f64,
                    &frag_corrupt,
                    if hash16(row) % 3 == 0 {
                        RED
                    } else if hash16(row) % 3 == 1 {
                        WHITE
                    } else {
                        BRIGHT
                    },
                );
            }
        }

        // Scanline displacement
        if hash16(pyint(elapsed * 20.0)) % 2 == 0 {
            let row_corrupt =
                top + 2 + (hash16(pyint(elapsed * 30.0)) % imax(1, bt - top - 4) as u32) as i64;
            for x in l..r {
                if hash16(x + pyint(elapsed * 50.0)) % 4 > 0 {
                    let ch = (33 + hash16(x * 37) % 94) as u8 as char;
                    c.put(x as f64, row_corrupt as f64, &ch.to_string(), RED);
                }
            }
        }

        // Central critical error
        let msg = "CRITICAL: EXECUTION BLOCKED";
        let flash_phase = pyint(elapsed * 10.0).rem_euclid(4);
        if flash_phase < 2 {
            // Apply extreme glitch
            if hash16(pyint(elapsed * 20.0)) % 2 == 0 {
                let msg_glitch = corrupt(msg, 19, 0, 41, pyint(elapsed * 100.0), 8.0);
                c.center(cy as f64, &msg_glitch, if flash_phase == 0 { RED } else { WHITE });
            } else {
                c.center(cy as f64, msg, if flash_phase == 0 { RED } else { WHITE });
            }
        }
    }
}

/// Python `lyric_execution_queue` — "EXECUTION - closing curtains with flashing
/// EXECUTE text".
pub fn lyric_execution_queue(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    // `t` is unused in the original body.
    let _ = t;
    let (l, top, r, bt) = area;
    let cx = (l + r).div_euclid(2);
    let cy = (top + bt).div_euclid(2);
    let progress = clamp(elapsed / 16.0); // 16 seconds for full execution sequence

    // Background: Scrolling code/hex
    let scroll_speed = pyint(elapsed * 15.0);
    for row in top..bt + 1 {
        let line_seed = (row + scroll_speed) * 23;
        for col in (l..r - 5).step_by(7) {
            // Generate code-like text
            let code_types = ["0x", "+=", "==", "&&", "||", "->", "::"];
            let code_choice = code_types[(hash16(line_seed + col) as usize) % code_types.len()];
            let hex_val = format!("{:02X}", hash16(line_seed + col * 13) % 256);

            // Mix of hex and code symbols
            if hash16(row * 17 + col) % 3 == 0 {
                c.put(
                    col as f64,
                    row as f64,
                    code_choice,
                    if hash16(row + col) % 4 != 0 {
                        NORMAL
                    } else {
                        GREEN
                    },
                );
            } else {
                // `{:02X}` always yields exactly two characters, so Python's
                // `hex_val[:2]` is the whole string.
                c.put(col as f64, row as f64, &hex_val, NORMAL);
            }
        }
    }

    // Curtains closing from both sides
    let curtain_close = progress * 0.9; // Close 90% by end
    let left_curtain_x = pyint(l as f64 + (r - l) as f64 * curtain_close * 0.5);
    let right_curtain_x = pyint(r as f64 - (r - l) as f64 * curtain_close * 0.5);

    // Left curtain (orange/yellow stripes)
    for x in l..left_curtain_x {
        for y in top..bt + 1 {
            // Vertical stripes with varying brightness
            let stripe_pattern = (x - l + y.div_euclid(3)).rem_euclid(4);
            if stripe_pattern == 0 {
                c.put(x as f64, y as f64, "█", WHITE); // Bright stripe
            } else if stripe_pattern == 1 {
                c.put(x as f64, y as f64, "▓", WHITE);
            } else if stripe_pattern == 2 {
                c.put(x as f64, y as f64, "▒", BRIGHT); // Medium
            } else {
                c.put(x as f64, y as f64, "░", BRIGHT); // Darker
            }
        }
    }

    // Right curtain (orange/yellow stripes)
    for x in right_curtain_x..r + 1 {
        for y in top..bt + 1 {
            let stripe_pattern = (x - right_curtain_x + y.div_euclid(3)).rem_euclid(4);
            if stripe_pattern == 0 {
                c.put(x as f64, y as f64, "█", WHITE);
            } else if stripe_pattern == 1 {
                c.put(x as f64, y as f64, "▓", WHITE);
            } else if stripe_pattern == 2 {
                c.put(x as f64, y as f64, "▒", BRIGHT);
            } else {
                c.put(x as f64, y as f64, "░", BRIGHT);
            }
        }
    }

    // Central EXECUTE text - large and flashing
    if left_curtain_x < cx && right_curtain_x > cx {
        // Flash pattern
        let flash_phase = pyint(elapsed * 6.0).rem_euclid(4);

        if flash_phase < 3 {
            // On for 3/4 of cycle
            // Large ASCII art "EXECUTE"
            let execute_text = [
                "███████╗██╗  ██╗███████╗ ██████╗██╗   ██╗████████╗███████╗",
                "██╔════╝╚██╗██╔╝██╔════╝██╔════╝██║   ██║╚══██╔══╝██╔════╝",
                "█████╗   ╚███╔╝ █████╗  ██║     ██║   ██║   ██║   ███████╗",
                "██╔══╝   ██╔██╗ ██╔══╝  ██║     ██║   ██║   ██║   ╚════██║",
                "███████╗██╔╝ ██╗███████╗╚██████╗╚██████╔╝   ██║   ███████║",
                "╚══════╝╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═════╝    ╚═╝   ╚══════╝",
            ];

            // Center the text
            let start_y = cy - execute_text.len() as i64 / 2;

            for (i, line) in execute_text.iter().enumerate() {
                let y_pos = start_y + i as i64;
                // Only draw if within visible area (between curtains)
                if top < y_pos && y_pos < bt {
                    // Truncate to fit between curtains
                    let _visible_width = right_curtain_x - left_curtain_x;
                    let line_len = line.chars().count() as i64;
                    let start_x = imax(left_curtain_x, cx - line_len.div_euclid(2));
                    let end_x = imin(right_curtain_x, cx + line_len.div_euclid(2));

                    if start_x < end_x {
                        // Calculate which part of the text to show
                        let line_start = imax(0, left_curtain_x - (cx - line_len.div_euclid(2)));
                        let line_end = imin(line_len, line_start + (end_x - start_x));

                        let visible_text: String = line
                            .chars()
                            .skip(line_start as usize)
                            .take((line_end - line_start).max(0) as usize)
                            .collect();

                        // Flash colors — Python's `Y` aliases `W` (both are 3).
                        let color = if flash_phase == 0 {
                            WHITE
                        } else if flash_phase == 1 {
                            WHITE
                        } else {
                            RED
                        };
                        c.put(start_x as f64, y_pos as f64, &visible_text, color);
                    }
                }
            }
        }
    }

    // Top status bar
    if progress < 0.3 {
        c.center((top + 1) as f64, "EXECUTION #1    depth=1", WHITE);
    } else if progress < 0.6 {
        // `Y` and `W` are the same style index.
        c.center((top + 1) as f64, "EXECUTION RUNNING...", WHITE);
    } else {
        c.center(
            (top + 1) as f64,
            "EXECUTION CLOSING",
            if pyint(elapsed * 6.0).rem_euclid(2) != 0 {
                RED
            } else {
                WHITE
            },
        );
    }

    // Bottom indicators
    let num_indicators = 8i64;
    for i in 0..num_indicators {
        let indicator_x = l + 10 + i * ((r - l - 20).div_euclid(num_indicators));
        if indicator_x < left_curtain_x || indicator_x > right_curtain_x {
            continue;
        }

        let indicator_active = (pyint(elapsed * 8.0) + i).rem_euclid(num_indicators);
        if i == indicator_active {
            c.put(indicator_x as f64, (bt - 2) as f64, "▶", WHITE);
        } else {
            c.put(indicator_x as f64, (bt - 2) as f64, "▷", NORMAL);
        }
    }
}

/// Python `lyric_only_execution` — "An exclusive attachment becomes an execution
/// loop and a prison for both."
pub fn lyric_only_execution(c: &Canvas, t: f64, area: (i64, i64, i64, i64), pulse: f64) {
    /// Python nested `middle(y,text,ink=N)`.
    fn middle(c: &Canvas, ml: i64, mw: i64, y: i64, text: &str, ink: Style) {
        // `text[:mw]` — a negative `mw` counts from the end, as in Python.
        let text = head(text, mw);
        let n = text.chars().count() as i64;
        c.put((ml + (mw - n).div_euclid(2)) as f64, y as f64, &text, ink);
    }
    /// Python nested `banner(y,text,ink=W)`.
    fn banner(c: &Canvas, ml: i64, mw: i64, y: i64, text: &str, ink: Style) {
        clear(c, ml as f64, y as f64, mw as f64, 5.0);
        if text.chars().count() as i64 * 6 - 1 <= mw {
            c.big(y as f64, text, ink);
        } else {
            middle(c, ml, mw, y + 2, text, ink);
        }
    }
    /// Python nested `heart(scale=1,filled=False)`.
    #[allow(clippy::too_many_arguments)]
    fn heart(
        c: &Canvas,
        ml: i64,
        mr: i64,
        plot_top: i64,
        plot_bt: i64,
        cx: i64,
        cy: i64,
        rw: f64,
        rh: f64,
        age: f64,
        pulse: f64,
        frame: i64,
        scale: f64,
        filled: bool,
    ) {
        let beat = 1.0 + 0.035 * (age * TAU * 2.2).sin() + pulse * 0.025;
        for yy in plot_top..plot_bt + 1 {
            for xx in ml..mr + 1 {
                let nx = (xx - cx) as f64 / fmax(1.0, rw * scale * beat);
                let ny = -((yy - cy) as f64) / fmax(1.0, rh * scale * beat) + 0.18;
                let shape = (nx * nx + ny * ny - 1.0).powi(3) - nx * nx * ny.powi(3);
                if shape <= 0.0 {
                    if filled {
                        let texture = hash16(xx * 31 + yy * 73 + frame);
                        c.put(
                            xx as f64,
                            yy as f64,
                            if texture % 8 < 2 {
                                if texture % 2 == 0 {
                                    "0"
                                } else {
                                    "1"
                                }
                            } else {
                                "#"
                            },
                            RED,
                        );
                    } else {
                        // An outline follows the silhouette, not internal level sets.
                        for (ox, oy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
                            let ex = nx + ox / fmax(1.0, rw * scale * beat);
                            let ey = ny + oy / fmax(1.0, rh * scale * beat);
                            if (ex * ex + ey * ey - 1.0).powi(3) - ex * ex * ey.powi(3) > 0.0 {
                                c.put(xx as f64, yy as f64, "#", RED);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    /// Python nested `node(x,y,text,ink=B)`.
    fn node(c: &Canvas, x: f64, y: f64, text: &str, ink: Style) {
        let x = pyint(x);
        let y = pyint(y);
        clear(c, (x - 4) as f64, (y - 1) as f64, 9.0, 3.0);
        c.box_((x - 4) as f64, (y - 1) as f64, 9, 3, ink);
        c.put(
            (x - text.chars().count() as i64 / 2) as f64,
            y as f64,
            text,
            ink,
        );
    }

    let (l, top, r, bt) = area;
    let cx = (l + r).div_euclid(2);
    let cy = (top + bt).div_euclid(2);
    let age = t - 162.632;
    let rail = imax(10, imin(20, c.w.div_euclid(7)));
    let ml = l + rail + 1;
    let mr = r - rail - 1;
    let mw = mr - ml + 1;
    let plot_top = top + 3;
    let plot_bt = bt - 3;
    let rh = fmax(2.0, (plot_bt - plot_top) as f64 * 0.45);
    let rw = mw as f64 * 0.43;
    let stage = if t < 166.016 {
        0
    } else if t < 169.824 {
        1
    } else if t < 173.643 {
        2
    } else {
        3
    };
    let frame = pyint(age * 22.0);

    // Scrolling obsessive calls replace neutral diagnostics on both edges.
    let rails: [(i64, &str, [&str; 6]); 2] = [
        (
            l,
            "ONLY ME",
            [
                "SELECT ME",
                "KEEP ME",
                "DELETE ALT",
                "ONLY ME",
                "ONE OWNER",
                "EXECUTE",
            ],
        ),
        (
            r - rail + 1,
            "KEEP YOU",
            [
                "FIND YOU",
                "RESTORE YOU",
                "COME BACK",
                "STAY HERE",
                "EXIT DENY",
                "RETRY",
            ],
        ),
    ];
    for (x, label, commands) in rails.iter().copied() {
        c.box_(x as f64, top as f64, rail, bt - top + 1, BRIGHT);
        c.put((x + 1) as f64, top as f64, label, RED);
        for yy in top + 1..bt {
            let index = frame + yy;
            let lock_label;
            let text: &str = if index.rem_euclid(3) != 0 {
                commands[index.rem_euclid(commands.len() as i64) as usize]
            } else {
                lock_label = format!("{:04X} LOCK", hash16(index * 71));
                &lock_label
            };
            c.put(
                (x + 1) as f64,
                yy as f64,
                &head(text, rail - 2),
                if index.rem_euclid(7) == 0 {
                    RED
                } else if index.rem_euclid(3) != 0 {
                    NORMAL
                } else {
                    GREEN
                },
            );
        }
    }

    if stage == 0 {
        // "Give them all the execution": eliminate every alternative connection.
        let progress = clamp((t - 163.315) / (165.166 - 163.315));
        let eliminated = imin(6, pyint(progress * 6.0));
        middle(c, ml, mw, top, "ELIMINATE EVERY OTHER PROCESS", BRIGHT);
        middle(
            c,
            ml,
            mw,
            top + 1,
            &format!("ALTERNATIVES: {:02} / TARGET: ONLY ME", 6 - eliminated),
            RED,
        );
        for i in 0..6i64 {
            let a = (i as f64 + 0.5) * TAU / 6.0;
            let nx = cx as f64 + a.cos() * rw * 0.85;
            let ny = cy as f64 + a.sin() * rh * 0.83;
            let u = clamp(progress * 6.0 - i as f64);
            c.line(
                cx as f64,
                cy as f64,
                nx,
                ny,
                if u != 0.0 { ':' } else { '=' },
                if u != 0.0 { GREEN } else { NORMAL },
            );
            if u < 1.0 {
                let packet = (age * 1.2 + i as f64 * 0.13).rem_euclid(1.0);
                c.put(
                    mix(cx as f64, nx, packet),
                    mix(cy as f64, ny, packet),
                    ">",
                    WHITE,
                );
                node(c, nx, ny, &format!("ALT{}", i + 1), BRIGHT);
            } else {
                c.put(nx - 3.0, ny, "[NULL]", RED);
                for spark in 0..8i64 {
                    let ang = spark as f64 * TAU / 8.0;
                    let distance = (age + i as f64 * 0.17).rem_euclid(1.0) * 6.0;
                    let px = nx + ang.cos() * distance;
                    let py = ny + ang.sin() * distance * 0.5;
                    if (ml as f64) < px
                        && px < mr as f64
                        && (plot_top as f64) < py
                        && py < plot_bt as f64
                    {
                        c.put(px, py, "x", if spark % 2 != 0 { RED } else { GREEN });
                    }
                }
            }
        }
        node(c, cx as f64, cy as f64, "YOU", WHITE);
        if t >= 165.166 {
            banner(c, ml, mw, cy - 2, "EXECUTION", RED);
            middle(c, ml, mw, cy + 5, "ALL OTHERS -> NULL", WHITE);
        }
    } else if stage == 1 {
        // A heart is now a lock, framed by the demand to be the only execution.
        heart(
            c, ml, mr, plot_top, plot_bt, cx, cy, rw, rh, age, pulse, frame, 1.0, true,
        );
        let bind = clamp((t - 166.016) / (168.911 - 166.016));
        for ring in 0..3i64 {
            let radius = (ring as f64 / 3.0 + age * 0.35).rem_euclid(1.0);
            for point_i in 0..70i64 {
                let a = point_i as f64 * TAU / 70.0;
                let x = cx as f64 + a.cos() * rw * radius;
                let y = cy as f64 + a.sin() * rh * radius;
                if (plot_top as f64) < y && y < plot_bt as f64 {
                    c.put(x, y, ":", GREEN);
                }
            }
        }
        middle(c, ml, mw, top, "YOU.OWNER = ME / EXCLUSIVE ACCESS", RED);
        middle(
            c,
            ml,
            mw,
            top + 1,
            &format!("BIND {:03}% / ALTERNATIVES: 0", pyint(bind * 100.0)),
            BRIGHT,
        );
        if mw >= 53 && bt - top >= 22 {
            banner(c, ml, mw, cy - 5, "THE ONLY", WHITE);
            banner(
                c,
                ml,
                mw,
                cy + 1,
                "EXECUTION",
                if t >= 168.911 { RED } else { BRIGHT },
            );
        } else {
            middle(c, ml, mw, cy - 3, "THE ONLY", WHITE);
            banner(
                c,
                ml,
                mw,
                cy - 1,
                "EXECUTION",
                if t >= 168.911 { RED } else { BRIGHT },
            );
        }
    } else if stage == 2 {
        // "Have you back": an absent YOU is reconstructed and pulled into the lock.
        let capture = clamp((t - 169.824) / (172.712 - 169.824));
        heart(
            c,
            ml,
            mr,
            plot_top,
            plot_bt,
            cx,
            cy,
            rw,
            rh,
            age,
            pulse,
            frame,
            0.86 + 0.14 * capture,
            true,
        );
        let mx = cx as f64 - mw as f64 * 0.18;
        let my = cy as f64 + rh * 0.24;
        let yx = mix(mr as f64 - 5.0, cx as f64 + mw as f64 * 0.18, capture);
        let yy = mix(plot_top as f64 + 2.0, cy as f64 - rh * 0.24, capture);
        for tether in 0..7i64 {
            let offset = tether - 3;
            let start_x = ml + pyint(((mw - 1) * tether) as f64 / 6.0);
            let start_y = if tether % 2 != 0 { plot_bt } else { plot_top };
            c.line(
                start_x as f64,
                start_y as f64,
                yx,
                yy,
                ':',
                if tether % 3 == 0 { RED } else { GREEN },
            );
            let phase = (age * 0.85 + tether as f64 / 7.0).rem_euclid(1.0);
            c.put(
                mix(start_x as f64, yx, phase),
                mix(start_y as f64, yy, phase),
                if tether % 2 != 0 { ">>" } else { "<<" },
                BRIGHT,
            );
            c.line(
                mx,
                my + offset as f64 * 0.3,
                yx,
                yy + offset as f64 * 0.3,
                '=',
                if tether == 3 { NORMAL } else { GREEN },
            );
        }
        node(c, mx, my, "ME", WHITE);
        node(c, yx, yy, "YOU", if capture > 0.8 { WHITE } else { GREEN });
        middle(c, ml, mw, top, "RESTORE(YOU) / RETURN TO ME", RED);
        middle(
            c,
            ml,
            mw,
            top + 1,
            &format!(
                "RETRY {:03} / RELEASE: DISABLED",
                pyint((t - 169.824) * 32.0)
            ),
            BRIGHT,
        );
        if t >= 172.712 {
            banner(c, ml, mw, cy - 2, "EXECUTION", RED);
            middle(c, ml, mw, cy + 5, "[ YOU RESTORED / EXIT LOCKED ]", WHITE);
        } else if t >= 171.868 {
            middle(c, ml, mw, cy - 1, "I WILL RUN THE", WHITE);
        }
    } else {
        // Both are caught: the same heart closes into an irreversible shared loop.
        let lock = clamp((t - 173.643) / 1.332);
        heart(
            c, ml, mr, plot_top, plot_bt, cx, cy, rw, rh, age, pulse, frame, 1.0, true,
        );
        let inset = pyint(mw as f64 * 0.08 * lock);
        let bx = ml + inset;
        let bw = mw - 2 * inset;
        c.box_(bx as f64, plot_top as f64, bw, plot_bt - plot_top + 1, RED);
        for i in 1..10i64 {
            let x = bx + (i * (bw - 1)).div_euclid(10);
            let length = pyint((plot_bt - plot_top - 1) as f64 * lock);
            for j in 0..length {
                let yy = if i % 2 != 0 {
                    plot_top + 1 + j
                } else {
                    plot_bt - 1 - j
                };
                c.put(
                    x as f64,
                    yy as f64,
                    "|",
                    if i % 3 != 0 { BRIGHT } else { RED },
                );
            }
        }
        node(c, cx as f64 - mw as f64 * 0.16, cy as f64, "ME", WHITE);
        node(c, cx as f64 + mw as f64 * 0.16, cy as f64, "YOU", WHITE);
        c.line(
            cx as f64 - mw as f64 * 0.16 + 5.0,
            cy as f64,
            cx as f64 + mw as f64 * 0.16 - 5.0,
            cy as f64,
            '=',
            RED,
        );
        middle(c, ml, mw, top, "[ TWO PRISONERS / ONE EXECUTION ]", RED);
        middle(c, ml, mw, top + 1, "while (true) { keep(me, you); }", BRIGHT);
        middle(
            c,
            ml,
            mw,
            imin(plot_bt - 1, cy + 4),
            "[ NO EXIT / NO RELEASE ]",
            WHITE,
        );
    }
    middle(
        c,
        ml,
        mw,
        bt - 1,
        if stage > 0 {
            "THE ONLY EXECUTION"
        } else {
            "EXECUTE(THEM) -> KEEP(ME)"
        },
        RED,
    );
    middle(
        c,
        ml,
        mw,
        bt,
        "LOVE.PERMISSION = EXCLUSIVE / EXIT = FALSE",
        NORMAL,
    );
}

/// Python `lyric_recursion` — "Recursive control flow - nested frames".
pub fn lyric_recursion(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    // `t` is unused in the original body.
    let _ = t;
    let (l, top, r, bt) = area;
    // `cx` is computed by the original and then never used.
    let _cx = (l + r) as f64 / 2.0;
    let depth = imin(8, pyint(elapsed * 2.0) + 1);
    for i in 0..depth {
        let pad = i * 4;
        let w = imax(10, r - l - pad * 2);
        let h = imax(3, bt - top - i * 3);
        let y = top + i * 2;
        // Box
        if w > 4 && h > 2 {
            let bar = format!("+{}+", "-".repeat((w - 2) as usize));
            c.put(
                (l + pad) as f64,
                y as f64,
                &bar,
                if i == depth - 1 { NORMAL } else { GREEN },
            );
            c.put((l + pad) as f64, (y + h - 1) as f64, &bar, GREEN);
            for yy in y + 1..y + h - 1 {
                c.put((l + pad) as f64, yy as f64, "|", GREEN);
                c.put((l + pad + w - 1) as f64, yy as f64, "|", GREEN);
            }
            c.put(
                (l + pad + 2) as f64,
                y as f64,
                &format!("frame_{}", i),
                if i == depth - 1 { NORMAL } else { GREEN },
            );
        }
    }
}

/// Python `lyric_execution_orb` — "Single execution instance - glowing orb".
pub fn lyric_execution_orb(
    c: &Canvas,
    t: f64,
    area: (i64, i64, i64, i64),
    elapsed: f64,
    pulse: f64,
) {
    // `elapsed` is unused in the original body; the orb is driven by `t`.
    let _ = elapsed;
    let (l, top, r, bt) = area;
    let cx = (l + r).div_euclid(2);
    let cy = (top + bt).div_euclid(2);
    let radius = imin(r - l, bt - top) as f64 * 0.35;
    // Pulsing orb
    for y in top..bt + 1 {
        for x in l..r + 1 {
            let dx = (x - cx) as f64 / 2.0;
            let dy = (y - cy) as f64;
            let dist = dx.hypot(dy);
            if dist < radius {
                let brightness =
                    1.0 - dist / radius + pulse * 0.2 + (dist * 0.5 - t * 4.0).sin() * 0.1;
                if brightness > 0.9 {
                    c.put(x as f64, y as f64, "@", WHITE);
                } else if brightness > 0.7 {
                    c.put(x as f64, y as f64, "#", WHITE);
                } else if brightness > 0.5 {
                    c.put(x as f64, y as f64, "*", BRIGHT);
                } else if brightness > 0.3 {
                    c.put(x as f64, y as f64, "+", BRIGHT);
                } else if brightness > 0.15 {
                    c.put(x as f64, y as f64, ".", NORMAL);
                }
            }
        }
    }
    c.center(cy as f64, "EXECUTE", WHITE);
}

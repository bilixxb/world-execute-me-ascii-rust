//! Love equation → outro — port of `world.execute-me-ascii/scenes.py`
//! lines 2604–2774.
//!
//! Byte-identical port of `lyric_love_equation`, `lyric_trapped_loop` and
//! `lyric_outro_wait`.

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

/// Python `s.ljust(n, '.')` — pad on the right, never truncate.
fn ljust_dots(s: &str, n: i64) -> String {
    let mut out = s.to_string();
    let cur = out.chars().count() as i64;
    if cur < n {
        out.extend(std::iter::repeat('.').take((n - cur) as usize));
    }
    out
}

/// Python `lyric_love_equation` — "A fictional love model: learning, inference,
/// then numerical collapse."
pub fn lyric_love_equation(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    let w = r - l + 1;
    let h = bt - top + 1;
    let failure = clamp((t - 179.929) / 8.554);
    let tick = pyint(t * (10.0 + failure * 18.0));
    let stage = if t < 179.929 {
        0
    } else if t < 180.857 {
        1
    } else if t < 184.54 {
        2
    } else {
        3
    };
    let titles = [
        "01 / LEARN TO LOVE",
        "02 / ATTENTION FIXATION",
        "03 / AUTOREGRESSIVE ANSWER",
        "04 / LOSS OF CONTROL",
    ];
    let rail = if w >= 95 {
        imax(13, imin(22, w.div_euclid(6)))
    } else {
        0
    };
    let a = l + rail + if rail != 0 { 1 } else { 0 };
    let b = r - rail - if rail != 0 { 1 } else { 0 };
    let span = b - a + 1;
    if rail != 0 {
        let logs = [
            "LOAD CORPUS",
            "TOKEN -> ID",
            "EMBED + POS",
            "Q K V MATMUL",
            "CAUSAL MASK",
            "RESIDUAL ADD",
            "MLP FORWARD",
            "LOSS BACKPROP",
            "WEIGHT UPDATE",
            "KV CACHE",
        ];
        let errors = [
            "LOVE LOVE LOVE",
            "CACHE REPEAT",
            "GRAD EXPLODES",
            "WEIGHT = INF",
            "LOGITS = NaN",
            "EOS REJECTED",
            "TARGET: YOU",
            "RETRY FOREVER",
        ];
        for (x, right_side) in [(l, false), (r - rail + 1, true)] {
            c.box_(x as f64, top as f64, rail, h, GREEN);
            c.put(
                (x + 2) as f64,
                top as f64,
                if right_side { "DECODE" } else { "TRAIN" },
                BRIGHT,
            );
            for row in 1..h - 1 {
                let n = row + tick;
                let broken = ((hash16(n * 13 + if right_side { 1 } else { 0 }) % 100) as f64)
                    < failure * 85.0;
                let msg = if broken {
                    errors[n.rem_euclid(errors.len() as i64) as usize]
                } else {
                    logs[n.rem_euclid(logs.len() as i64) as usize]
                };
                c.put(
                    (x + 1) as f64,
                    (top + row) as f64,
                    &head(
                        &format!("{:02X} {}", n.rem_euclid(256), msg),
                        rail - 2,
                    ),
                    if broken { RED } else { DIM },
                );
            }
        }
    }
    c.put(
        a as f64,
        top as f64,
        &head(titles[stage as usize], span),
        if stage < 3 { WHITE } else { RED },
    );
    let query = "[HOW] [TO] [LOVE] [?] -> EMBEDDING + POSITION";
    c.put(a as f64, (top + 2) as f64, &head(query, span), BRIGHT);
    // Tokens stream into the model from the very first frame.
    let mut stream = "0048 0017 0911 003F ".repeat((span.div_euclid(20) + 2).max(0) as usize);
    if stage >= 2 {
        stream = "LOVE 0911 LOVE 0911 ".repeat((span.div_euclid(20) + 2).max(0) as usize);
    }
    let shift = pyint(elapsed * 15.0).rem_euclid(19);
    let stream_slice: String = stream
        .chars()
        .skip(shift as usize)
        .take(span.max(0) as usize)
        .collect();
    c.put(
        a as f64,
        (top + 3) as f64,
        &stream_slice,
        if stage == 3 { RED } else { DIM },
    );
    let panel_top = top + 5;
    let panel_bottom = imax(panel_top + 5, bt - 7);
    let ph = panel_bottom - panel_top + 1;
    let widths = [
        span.div_euclid(3),
        span.div_euclid(3),
        span - 2 * span.div_euclid(3),
    ];
    let xs = [a, a + widths[0], a + widths[0] + widths[1]];
    let panel_titles = ["Q K^T / MASK", "RESIDUAL / MLP", "NEXT TOKEN"];
    for i in 0..3usize {
        let (x, pw) = (xs[i], widths[i]);
        c.box_(x as f64, panel_top as f64, pw, ph, GREEN);
        c.put(
            (x + 1) as f64,
            panel_top as f64,
            &head(panel_titles[i], pw - 2),
            BRIGHT,
        );
    }
    // Causal attention map. As fixation grows, the LOVE key captures every row.
    let x = xs[0];
    let pw = widths[0];
    let count = imin(imin(8, imax(3, (pw - 3).div_euclid(2))), imax(3, ph - 4));
    let cellw = imax(1, (pw - 3).div_euclid(count));
    let love_key = imin(2, count - 1);
    for row in 0..count {
        let yy = panel_top + 2 + pyint((row * (ph - 4)) as f64 / count as f64);
        for col in 0..count {
            let xx = x + 2 + col * cellw;
            let value = (row as f64 * 1.7 + col as f64 * 0.8 + elapsed * 4.0)
                .sin()
                .abs();
            let (ch, style) = if col > row {
                ('.', GREEN)
            } else if stage >= 1 && col == love_key {
                ('#', if failure > 0.45 { RED } else { WHITE })
            } else if failure > 0.65 {
                ('?', RED)
            } else if value > 0.65 {
                ('O', BRIGHT)
            } else {
                (':', DIM)
            };
            c.put(
                xx as f64,
                yy as f64,
                &ch.to_string().repeat(imax(1, cellw - 1) as usize),
                style,
            );
        }
    }
    c.put(
        (x + 1) as f64,
        (panel_bottom - 1) as f64,
        &head(
            if stage != 0 {
                "LOVE <- ALL"
            } else {
                "CAUSAL SOFTMAX"
            },
            pw - 2,
        ),
        if stage != 0 { RED } else { DIM },
    );
    // Dense weighted layers with visible travelling activation packets.
    let x = xs[1];
    let pw = widths[1];
    let layers = 4i64;
    let rows = imax(3, imin(6, ph - 4));
    let mut nodes: Vec<Vec<(i64, i64)>> = Vec::new();
    for i in 0..layers {
        let mut layer_nodes = Vec::new();
        for j in 0..rows {
            layer_nodes.push((
                x + 2 + pyint((i * (pw - 5)) as f64 / 3.0),
                panel_top + 2 + pyint((j * (ph - 5)) as f64 / (rows - 1) as f64),
            ));
        }
        nodes.push(layer_nodes);
    }
    for layer in 0..layers - 1 {
        for (j, p) in nodes[layer as usize].iter().enumerate() {
            for (k, q) in nodes[(layer + 1) as usize].iter().enumerate() {
                if (j as i64 + k as i64 + layer) % 2 != 0 {
                    continue;
                }
                c.line(p.0 as f64, p.1 as f64, q.0 as f64, q.1 as f64, '.', GREEN);
                let u = (elapsed * (1.4 + failure * 3.0) + j as f64 * 0.13 + k as f64 * 0.09)
                    .rem_euclid(1.0);
                c.put(
                    mix(p.0 as f64, q.0 as f64, u),
                    mix(p.1 as f64, q.1 as f64, u),
                    ">",
                    if failure > 0.6 { RED } else { BRIGHT },
                );
            }
        }
    }
    for (layer, points) in nodes.iter().enumerate() {
        for (j, (xx, yy)) in points.iter().enumerate() {
            let unstable = ((hash16(tick + j as i64 * 11 + layer as i64 * 31) % 100) as f64)
                < failure * 80.0;
            c.put(
                *xx as f64,
                *yy as f64,
                if unstable { "X" } else { "O" },
                if unstable { RED } else { WHITE },
            );
        }
    }
    c.put(
        (x + 1) as f64,
        (panel_bottom - 1) as f64,
        &head(
            if stage == 3 {
                "W=NaN dW=INF"
            } else {
                "FORWARD / +RES"
            },
            pw - 2,
        ),
        if stage == 3 { RED } else { DIM },
    );
    // Sampled token distribution collapses to repetition and refuses EOS.
    let x = xs[2];
    let pw = widths[2];
    let choices = ["LOVE", "STAY", "YOU", "FREE", "EOS"];
    let probs = [
        0.38 + 0.61 * failure,
        0.25 * (1.0 - failure),
        0.19 * (1.0 - failure),
        0.12 * (1.0 - failure),
        0.06 * (1.0 - failure),
    ];
    for i in 0..5usize {
        let (word, p) = (choices[i], probs[i]);
        let yy = panel_top + 2 + pyint((i as i64 * imax(1, ph - 4)) as f64 / 5.0);
        c.put(
            (x + 1) as f64,
            yy as f64,
            &head(word, pw - 2),
            if failure > 0.4 && i == 0 { RED } else { NORMAL },
        );
        let barw = imax(1, pw - 8);
        let bar = ljust_dots(
            &"#".repeat(pyint(p * barw as f64).max(0) as usize),
            barw,
        );
        c.put(
            (x + 6) as f64,
            yy as f64,
            &bar,
            if failure > 0.4 && i == 0 { RED } else { BRIGHT },
        );
    }
    // Training loss is a visual metaphor: divergence, then undefined arithmetic.
    let graph_top = panel_bottom + 2;
    let graph_bottom = bt - 2;
    let gh = imax(1, graph_bottom - graph_top);
    let label = if stage < 2 {
        "LOSS / BACKPROP"
    } else {
        "CONTEXT -> SAMPLE -> APPEND -> CONTEXT"
    };
    c.put(a as f64, graph_top as f64, &head(label, span), NORMAL);
    let mut prev: Option<(i64, i64)> = None;
    for col in 0..span {
        let u = col as f64 / imax(1, span - 1) as f64;
        let mut v = if stage < 2 {
            0.65 * (-u * 4.0).exp()
        } else {
            0.1 + failure * u * u * 0.8
        };
        v += (col as f64 * 0.7 + elapsed * 9.0).sin() * failure * 0.15;
        let yy = graph_bottom - pyint(clamp(v) * imax(1, gh - 1) as f64);
        if let Some((px, py)) = prev {
            c.line(
                px as f64,
                py as f64,
                (a + col) as f64,
                yy as f64,
                '.',
                if stage == 3 { RED } else { DIM },
            );
        }
        prev = Some((a + col, yy));
    }
    let head_pos = a + pyint(elapsed * 22.0).rem_euclid(span);
    c.put(head_pos as f64, (graph_bottom - 1) as f64, "|", WHITE);
    let status = [
        "OPTIMIZER: ADAM / TARGET: LOVE",
        "ATTENTION LOCKED ON LOVE",
        "LOVE > LOVE > LOVE > LOVE / EOS: 0",
        "LOSS: NaN / GRAD: INF / NO EXIT",
    ][stage as usize];
    c.put(
        a as f64,
        bt as f64,
        &head(status, span),
        if stage >= 2 { RED } else { BRIGHT },
    );
    // Each sung LOVE stamps across the complete model; the panels remain beneath.
    let hit = [(179.929, 180.857), (183.646, 184.54), (187.665, 188.483)]
        .iter()
        .find(|&&(start, end)| start <= t && t < end)
        .map(|&(start, _)| start);
    if hit.is_some() {
        let yy = (panel_top + panel_bottom).div_euclid(2) - 2;
        clear(c, a as f64, yy as f64, span as f64, 5.0);
        c.big(yy as f64, "LOVE", if stage >= 2 { RED } else { WHITE });
        c.put(
            a as f64,
            (yy + 5) as f64,
            &head("P(LOVE) -> 1.0 / ALL OTHER TOKENS SUPPRESSED", span),
            RED,
        );
    }
    // Local corruption adds context duplication without erasing the model layout.
    if failure > 0.55 {
        for i in 0..(1 + pyint(failure * 4.0)) {
            let yy = panel_top + 1 + (hash16(tick + i * 41) % imax(1, ph - 2) as u32) as i64;
            let xx = a + (hash16(tick + i * 97) % imax(1, span - 12) as u32) as i64;
            c.put(
                xx as f64,
                yy as f64,
                if stage == 3 { "NaN NaN" } else { "LOVE LOVE" },
                RED,
            );
        }
    }
}

/// Python `lyric_trapped_loop` — "Trapped in loop - prison bars over heart".
pub fn lyric_trapped_loop(
    c: &Canvas,
    t: f64,
    area: (i64, i64, i64, i64),
    elapsed: f64,
    pulse: f64,
) {
    let (l, top, r, bt) = area;
    // `cx` is computed by the original and then never used.
    let _cx = (l + r).div_euclid(2);
    let cy = (top + bt).div_euclid(2);
    // Heart (reuse)
    crate::scenes::organic::lyric_heart(c, t, area, fmin(elapsed, 2.0), pulse);
    // Prison bars
    let num_bars = 8i64;
    for i in 0..num_bars {
        let x = l + 5 + i * (r - l - 10).div_euclid(num_bars - 1);
        for y in top..bt + 1 {
            c.put(x as f64, y as f64, "|", NORMAL);
            // Moving lock symbols
            if (y + pyint(t * 6.0)).rem_euclid(7) == 0 {
                c.put(x as f64, y as f64, "▓", WHITE);
            }
        }
    }
    if elapsed > 1.0 {
        c.center(
            (cy + pyint((bt - top) as f64 * 0.35)) as f64,
            "TRAPPED",
            RED,
        );
    }
}

/// Python `lyric_outro_wait` — "A stalled process becomes the final EXECUTION on
/// the lyric cue."
pub fn lyric_outro_wait(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    // `cx` is computed by the original and then never used.
    let _cx = (l + r).div_euclid(2);
    let cy = (top + bt).div_euclid(2);
    let execution_at = 205.811; // Final EXECUTION cue in lyrics.json.
    let executing = t >= execution_at;
    let bar_w = imin(96, c.w - 12);
    let bar_x = (c.w - bar_w).div_euclid(2);
    let inner = bar_w - 4;
    let progress = imin(
        99,
        pyint(99.0 * (1.0 - (1.0 - clamp(elapsed / 5.0)).powi(3))),
    );
    let stalled = progress == 99;
    let style = if executing { RED } else { BRIGHT };
    c.center(
        (cy - 6) as f64,
        "world.execute(me);",
        if executing { style } else { NORMAL },
    );
    c.box_(bar_x as f64, (cy - 2) as f64, bar_w, 5, style);
    let filled = imin(inner - 1, pyint((inner * progress) as f64 / 100.0));
    let sweep = pyint(t * 18.0).rem_euclid(imax(1, filled));
    for row in 0..3i64 {
        for col in 0..inner {
            let lit = col < filled;
            let ch = if lit {
                if row == 1 { "#" } else { "=" }
            } else {
                "."
            };
            let mut ink = if lit { style } else { GREEN };
            if lit && (col - sweep).rem_euclid(imax(1, filled)) < 3 {
                ink = if executing { RED } else { WHITE };
            }
            c.put((bar_x + 2 + col) as f64, (cy - 1 + row) as f64, ch, ink);
        }
    }
    if !executing {
        let spinner = ['|', '/', '-', '\\'][pyint(t * 8.0).rem_euclid(4) as usize];
        c.center(
            (cy + 4) as f64,
            &format!(
                "{:02}%  [{}]  {}",
                progress,
                spinner,
                if stalled {
                    "WAITING FOR RESPONSE"
                } else {
                    "EXECUTING"
                }
            ),
            BRIGHT,
        );
        if stalled {
            let retry = imax(1, pyint((elapsed - 5.0) * 2.0) + 1);
            c.center(
                (cy + 6) as f64,
                &format!("RETRY {:04}  /  ACK: --  /  REMAINING: 01%", retry),
                if pyint(t * 3.0).rem_euclid(2) != 0 {
                    NORMAL
                } else {
                    GREEN
                },
            );
        } else {
            c.center((cy + 6) as f64, "COMMITTING FINAL INSTRUCTION...", GREEN);
        }
        return;
    }

    // The same five-row bar breaks into red bitmap letters in place.
    let u = clamp((t - execution_at) / 0.48);
    // Snapshot the five bar rows *before* they are cleared and repainted.
    let before: Vec<Vec<crate::canvas::Cell>> = {
        let cells = c.cells.borrow();
        (cy - 2..cy + 3)
            .map(|y| {
                if y >= 0 && (y as usize) < cells.len() {
                    cells[y as usize].clone()
                } else {
                    Vec::new()
                }
            })
            .collect()
    };
    clear(c, l as f64, (cy - 2) as f64, (r - l + 1) as f64, 5.0);
    c.big((cy - 2) as f64, "EXECUTION", RED);
    if u < 1.0 {
        for dy in 0..5i64 {
            for x in l..r + 1 {
                if hash16(x * 71 + dy * 313) as f64 / 65535.0 > u {
                    // `ch,_=before[dy][x]` then `c.cells[cy-2+dy][x]=(ch,R)`.
                    if let Some(cell) = before[dy as usize].get(x as usize) {
                        c.set(x, cy - 2 + dy, cell.ch, RED);
                    }
                }
            }
        }
    }
    c.center((cy + 4) as f64, "[ PROCESS TERMINATED ]", RED);
    c.center(
        (cy + 6) as f64,
        "EXIT CODE: EXECUTION",
        if pyint(t * 2.0).rem_euclid(2) != 0 {
            RED
        } else {
            GREEN
        },
    );
}

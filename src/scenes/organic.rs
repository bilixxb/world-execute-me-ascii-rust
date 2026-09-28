//! Execution, heart, and the restored legacy organic choreography — port of
//! `world.execute-me-ascii/scenes.py` lines 1275–1600.

use std::collections::HashMap;

use crate::canvas::Canvas;
use crate::core::*;
use crate::scenes::common::*;
use crate::text::*;
use crate::TAU;

// `crate::core::clamp` (three arguments) and `crate::scenes::common::clamp`
// (Python's one-argument form) both arrive through the glob imports above, so
// the bare name has to be pinned down. Every `clamp(x)` below is the 1-arg form.
use crate::scenes::common::clamp;

/// Python `c.put(x, y, ch, style)`, where `ch` is a one-character string.
#[inline]
fn putc(c: &Canvas, x: f64, y: f64, ch: char, style: Style) {
    c.put(x, y, &ch.to_string(), style);
}

/// Python `len(s)` on a `str` — code points, not display columns.
#[inline]
fn plen(s: &str) -> i64 {
    s.chars().count() as i64
}

/// Python `s[:n]`. The non-negative form is the crate's width-aware `crop`
/// (every sliced literal in this section is ASCII, so the two agree); the
/// negative form trims `-n` code points off the end, as Python does.
#[inline]
fn cut(s: &str, n: i64) -> &str {
    if n < 0 {
        let keep = (plen(s) + n).max(0);
        return match s.char_indices().nth(keep as usize) {
            Some((i, _)) => &s[..i],
            None => s,
        };
    }
    crop(s, n as usize)
}

/// Python `point()` with a **fractional** area.
///
/// `legacy_organic` hands its mesh/ring helpers `(l, y, cx + 6, b)`, where
/// `cx = (l + r) / 2` is a half-integer whenever `l + r` is odd. Narrowing that
/// to the `(i64, i64, i64, i64)` area that `common::point` takes would shift
/// every projected point by up to half a column, so the organic helpers keep the
/// original's `(l + r) / 2` and `(r - l)` arithmetic in `f64`. This is
/// `common::point` with only the area type widened.
fn point_f(
    x: f64,
    y: f64,
    z: f64,
    area: (f64, f64, f64, f64),
    t: f64,
    rotate: bool,
) -> (f64, f64, f64) {
    let (x, y, z) = if rotate { rot(x, y, z, t) } else { (x, y, z) };
    let (l, top, r, bt) = area;
    let p = 3.7 / (3.7 + z);
    (
        (l + r) / 2.0 + x * (r - l) * 0.34 * p,
        (top + bt) / 2.0 + y * (bt - top) * 0.34 * p,
        z,
    )
}

/// `common::projected` with the same fractional-area support as [`point_f`].
fn projected_f(
    c: &Canvas,
    vertices: &[(f64, f64, f64)],
    edges: &[(usize, usize)],
    area: (f64, f64, f64, f64),
    t: f64,
    style: Style,
    reveal: f64,
) {
    let ps: Vec<(f64, f64, f64)> = vertices
        .iter()
        .map(|&(x, y, z)| point_f(x, y, z, area, t, true))
        .collect();
    let count = (edges.len() as f64 * clamp3(reveal, 0.0, 1.0)) as usize;
    for &(a, b) in edges.iter().take(count) {
        let (x, y, z) = ps[a];
        let (xx, yy, zz) = ps[b];
        c.line(
            x,
            y,
            xx,
            yy,
            if (z + zz) < 0.0 { ':' } else { '.' },
            if (z + zz) < 0.0 { style } else { GREEN },
        );
    }
    let limit = vertices.len() as f64 * reveal;
    for (i, &(x, y, z)) in ps.iter().enumerate() {
        if (i as f64) < limit {
            c.put(
                x,
                y,
                if z > 0.0 { "+" } else { "@" },
                if z > 0.0 { NORMAL } else { BRIGHT },
            );
        }
    }
}

/// A happiness condition feeds a self-execution, with live data on both sides.
pub fn lyric_happy_execution(c: &Canvas, t: f64, area: (i64, i64, i64, i64), pulse: f64) {
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let elapsed = t - 66.601;
    let loading = t >= 68.252;
    let executing = t >= 69.259;
    let rail = imax(13, imin(24, c.w.div_euclid(5)));
    let left = l;
    let right = r - rail + 1;
    let mid_l = left + rail + 2;
    let mid_r = right - 3;
    let mw = mid_r - mid_l + 1;
    let rate = if executing {
        30.0
    } else if loading {
        18.0
    } else {
        10.0
    };
    let frame = (elapsed * rate) as i64;
    let ops = ["READ", "LOAD", "PUSH", "COPY", "SYNC", "CALL", "EXEC", "WAIT"];
    // Scroll memory upward on the left and the instruction queue downward on the right.
    for (x, label, side) in [(left, "MEM / YOU", 0i64), (right, "EXEC / ME", 1i64)] {
        c.box_(x as f64, top as f64, rail, bt - top + 1, BRIGHT);
        c.put((x + 2) as f64, top as f64, cut(label, rail - 4), WHITE);
        for row in (top + 1)..bt {
            let index = if side == 0 {
                frame + (row - top)
            } else {
                frame - (row - top)
            };
            let value = hash16(index * 73 + side * 911);
            let address = (index * 16) & 65535;
            let text = if side == 0 {
                format!("{:04X} {:04X} {:04X}", address, value, hash16(index * 29))
            } else {
                let op = if executing && index % 3 == 0 {
                    "EXEC"
                } else {
                    ops[index.rem_euclid(8) as usize]
                };
                format!("{} {:04X} {:04X}", op, address, value)
            };
            let selected = (bt - top - 1) != 0 && (row - top + frame) % (bt - top - 1) == 0;
            let style = if selected {
                WHITE
            } else if index % 3 != 0 {
                NORMAL
            } else {
                GREEN
            };
            let line = format!(
                "{}{}",
                if selected { '>' } else { ' ' },
                cut(&text, rail - 3)
            );
            c.put((x + 1) as f64, row as f64, &line, style);
        }
    }
    let middle = |y: i64, text: &str, style: Style| {
        let text = cut(text, mw);
        c.put(
            (mid_l + (mw - plen(text)).div_euclid(2)) as f64,
            y as f64,
            text,
            style,
        );
    };

    middle(
        top,
        "IF (YOU.HAPPY) -> EXECUTE(ME)",
        if loading { WHITE } else { BRIGHT },
    );
    middle(
        top + 1,
        &format!(
            "SELF.EXECUTION / {}",
            if executing {
                "RUNNING"
            } else if loading {
                "ARMED"
            } else {
                "CONDITION"
            }
        ),
        NORMAL,
    );
    let content_top = top + 3;
    let content_bt = bt - 3;
    let center_y = (content_top + content_bt) as f64 / 2.0;
    let half_h = fmax(2.0, (content_bt - content_top) as f64 * 0.43);
    let half_w = fmax(5.0, mw as f64 * 0.40);
    let beat = 1.0 + 0.045 * (elapsed * TAU * 2.0).sin() + pulse * 0.04;
    let morph = if loading {
        clamp((t - 68.252) / (69.259 - 68.252))
    } else {
        0.0
    };
    // A large data-filled heart. Its own cells become the execution buffer.
    for yy in content_top..=content_bt {
        for xx in mid_l..=mid_r {
            let nx = (xx - cx) as f64 / (half_w * beat);
            let ny = -((yy - cy) as f64) / (half_h * beat) + 0.20;
            let heart = (nx * nx + ny * ny - 1.0).powf(3.0) - nx * nx * ny.powf(3.0);
            let seed = hash16(xx * 79 + yy * 233);
            if heart <= 0.0 && !executing {
                if seed as f64 / 65535.0 < morph {
                    // Removed heart cells spread out across the entire center.
                    let dx = ((xx - cx) as f64 * morph * 0.9) as i64;
                    let dy = ((yy - cy) as f64 * morph * 0.6) as i64;
                    let px = imax(mid_l, imin(mid_r, xx + dx));
                    let py = imax(content_top, imin(content_bt, yy + dy));
                    putc(
                        c,
                        px as f64,
                        py as f64,
                        ['0', '1', 'E', 'X'][(seed % 4) as usize],
                        if morph > 0.7 { GREEN } else { NORMAL },
                    );
                } else {
                    let scan = (yy - content_top - (elapsed * 9.0) as i64)
                        .rem_euclid(imax(1, content_bt - content_top + 1));
                    // Use the implicit equation only as a silhouette mask. Its magnitude
                    // has three interior basins, so thresholding it creates false holes.
                    let texture = hash16(seed as i64 + frame.div_euclid(2));
                    let ch = if texture % 8 < 2 {
                        ['0', '1'][(texture % 2) as usize]
                    } else {
                        '#'
                    };
                    putc(
                        c,
                        xx as f64,
                        yy as f64,
                        ch,
                        if scan < 2 { WHITE } else { BRIGHT },
                    );
                }
            } else if executing {
                // Expanding execution wave and dim opcode fragments behind the title.
                let radius = (xx - cx).abs() as f64 / fmax(1.0, mw as f64 / 2.0)
                    + (yy - cy).abs() as f64 / fmax(1.0, half_h);
                let wave = ((t - 69.259) * 3.0).rem_euclid(2.0);
                if (radius - wave).abs() < 0.12 {
                    putc(c, xx as f64, yy as f64, '=', BRIGHT);
                } else if seed % 31 == 0 {
                    putc(c, xx as f64, yy as f64, ['0', '1'][(seed % 2) as usize], GREEN);
                }
            }
        }
    }

    // Bright data packets flow from both edge panels into the center.
    for lane in 0..3 {
        let yy = center_y as i64 + (lane - 1) * imax(1, (half_h * 0.7) as i64);
        let path = imax(2, (mw - 6).div_euclid(2));
        let rate = if loading { 28.0 } else { 16.0 };
        let head =
            ((elapsed * rate + lane as f64 * 7.0) as i64).rem_euclid(path);
        for tail in 0..4 {
            let step = imax(0, head - tail);
            let ink = if tail == 0 {
                WHITE
            } else if tail < 2 {
                BRIGHT
            } else {
                GREEN
            };
            for (xx, head_char) in [(mid_l + step, '>'), (mid_r - step, '<')] {
                let nx = (xx - cx) as f64 / (half_w * beat);
                let ny = -((yy - cy) as f64) / (half_h * beat) + 0.20;
                let inside = (nx * nx + ny * ny - 1.0).powf(3.0) - nx * nx * ny.powf(3.0) <= 0.0;
                if inside && !loading {
                    // Packets brighten the solid fill instead of cutting dark dashes into it.
                    let ch = if tail == 0 {
                        '#'
                    } else {
                        ['0', '1'][((xx + yy + frame).rem_euclid(2)) as usize]
                    };
                    putc(c, xx as f64, yy as f64, ch, if tail == 0 { WHITE } else { BRIGHT });
                } else {
                    putc(
                        c,
                        xx as f64,
                        yy as f64,
                        if tail == 0 { head_char } else { '-' },
                        ink,
                    );
                }
            }
        }
    }
    if executing {
        let title_y = center_y as i64 - 2;
        // Reserve a clean five-row band so the word remains legible through the wave.
        clear(c, mid_l as f64, title_y as f64, mw as f64, 5.0);
        if mw >= 53 {
            c.big(title_y as f64, "EXECUTION", WHITE);
        } else {
            middle(title_y + 2, ">> EXECUTION <<", WHITE);
        }
        middle(title_y - 2, "[ YOU.HAPPY == TRUE ]", BRIGHT);
        middle(title_y + 6, "world.execute(me);", WHITE);
    } else if loading {
        middle(center_y as i64, "[ RUN THE EXECUTION ]", WHITE);
    } else {
        middle(center_y as i64, "YOU.HAPPY", WHITE);
    }
    middle(
        bt - 1,
        &format!(
            "EXECUTION: {}",
            if executing {
                "RUN"
            } else if loading {
                "QUEUED"
            } else {
                "READY"
            }
        ),
        BRIGHT,
    );
    middle(
        bt,
        &format!(
            "ME -> YOU / {}",
            if executing {
                "SELF COMMITTED"
            } else if loading {
                "COMPILING..."
            } else {
                "MAKE YOU HAPPY"
            }
        ),
        NORMAL,
    );
}

/// The execution closes a shared cage, then reveals an endless simulated space.
pub fn lyric_trapped_simulation(c: &Canvas, t: f64, area: (i64, i64, i64, i64), pulse: f64) {
    let _ = pulse; // The original signature carries `pulse`; this scene never reads it.
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let _ = cy; // computed by the original, never read in this scene
    let age = t - 70.084;
    let strange = clamp((t - 71.764) / 1.405);
    let reveal = t >= 73.169;
    let rail = imax(10, imin(19, c.w.div_euclid(6)));
    let ml = l + rail + 1;
    let mr = r - rail - 1;
    let mw = mr - ml + 1;
    let inner_top = top + 2;
    let inner_bt = bt - 2;
    let hh = fmax(2.0, (inner_bt - inner_top) as f64 / 2.0);
    let center_y = (inner_top + inner_bt) as f64 / 2.0;
    let frame = (age * (22.0 + strange * 20.0)) as i64;
    let middle = |y: i64, text: &str, style: Style| {
        let text = cut(text, mw);
        c.put(
            (ml + (mw - plen(text)).div_euclid(2)) as f64,
            y as f64,
            text,
            style,
        );
    };

    // Full-height memory walls scroll in opposite directions and keep resetting.
    for (x, label, direction) in [(l, "HEAP / ME", 1i64), (r - rail + 1, "STACK / YOU", -1i64)] {
        c.box_(x as f64, top as f64, rail, bt - top + 1, BRIGHT);
        c.put((x + 1) as f64, top as f64, cut(label, rail - 2), WHITE);
        for yy in (top + 1)..bt {
            let step = (frame * direction + yy - top).rem_euclid(256);
            let value = hash16(step * 53);
            let text = if step % 4 != 0 {
                format!("{:04X} {:04X}", step * 16, value)
            } else {
                format!(
                    "{}{:04X}",
                    if step % 8 != 0 { "LOOP " } else { "EXEC " },
                    value
                )
            };
            let style = if step % 13 == 0 {
                WHITE
            } else if step % 3 != 0 {
                NORMAL
            } else {
                GREEN
            };
            c.put((x + 1) as f64, yy as f64, cut(&text, rail - 2), style);
        }
    }

    // A warped coordinate lattice covers the whole central simulation volume.
    for yy in inner_top..=inner_bt {
        for xx in ml..=mr {
            let bend = (((yy as f64 - center_y) * 0.32) + age * 3.1).sin() * strange * 6.0;
            let gx = (xx as f64 + bend + age * 5.0) as i64;
            let gy =
                (yy as f64 + (((xx - cx) as f64 * 0.12 - age * 2.0).sin()) * strange * 3.0) as i64;
            if gx % 8 == 0 || gy % 4 == 0 {
                let ch = if gx % 8 == 0 && gy % 4 == 0 {
                    '+'
                } else if gx % 8 == 0 {
                    ':'
                } else {
                    '.'
                };
                putc(c, xx as f64, yy as f64, ch, GREEN);
            } else if hash16(xx * 17 + yy * 79 + frame.div_euclid(3)) % 109 == 0 {
                putc(
                    c,
                    xx as f64,
                    yy as f64,
                    ['0', '1'][(hash16(xx + yy) % 2) as usize],
                    NORMAL,
                );
            }
        }
    }

    // Recurring perspective frames: every apparent exit leads into another cell.
    for layer in 0..8 {
        let phase = (layer as f64 / 8.0 + age * (0.20 + strange * 0.25)).rem_euclid(1.0);
        let scale = 0.12 + 0.88 * phase.powf(1.5);
        let skew = (age * 2.0 + layer as f64 * 0.8).sin() * strange * mw as f64 * 0.10;
        let x0 = cx as f64 - mw as f64 * 0.48 * scale;
        let x1 = cx as f64 + mw as f64 * 0.48 * scale;
        let y0 = center_y - hh * 0.95 * scale;
        let y1 = center_y + hh * 0.95 * scale;
        let wobble = strange * (age * 3.0 + layer as f64).sin();
        let corners = [
            (x0 + skew, y0),
            (x1, y0 + wobble),
            (x1 - skew, y1),
            (x0, y1 - wobble),
        ];
        for index in 0..4usize {
            let (x, y) = corners[index];
            let (xx, yy) = corners[(index + 1) % 4];
            c.line(
                fmax(ml as f64, fmin(mr as f64, x)),
                y,
                fmax(ml as f64, fmin(mr as f64, xx)),
                yy,
                if layer % 3 == 0 { '=' } else { '-' },
                if layer % 3 == 0 { NORMAL } else { GREEN },
            );
        }
        if layer % 2 == 0 && scale > 0.6 {
            c.put(
                imax(ml, x0 as i64) as f64,
                y0 as i64 as f64,
                &format!("LOOP {:02}", layer),
                NORMAL,
            );
        }
    }

    // The outer gate visibly contracts around both entities on "we are trapped".
    let close = clamp(age / 0.85);
    let box_w = imax(20, (mw as f64 * (0.98 - 0.22 * close)) as i64);
    let box_h = imax(
        7,
        ((inner_bt - inner_top + 1) as f64 * (0.98 - 0.16 * close)) as i64,
    );
    let bx = cx - box_w.div_euclid(2);
    let by = center_y as i64 - box_h.div_euclid(2);
    if !reveal {
        c.box_(bx as f64, by as f64, box_w, box_h, BRIGHT);
        // Sliding bars lock from both sides; their gaps then oscillate unnaturally.
        for index in 1..7 {
            let xx = bx + (index * (box_w - 1)).div_euclid(7);
            let jitter = ((age * 5.0 + index as f64).sin() * strange * 2.0) as i64;
            let extent = ((box_h - 2) as f64 * close) as i64;
            for row in 0..extent {
                let yy = if index % 2 != 0 {
                    by + 1 + row
                } else {
                    by + box_h - 2 - row
                };
                putc(
                    c,
                    (xx + jitter) as f64,
                    yy as f64,
                    '|',
                    if index % 2 != 0 { NORMAL } else { BRIGHT },
                );
            }
        }
        middle(
            by,
            &format!(
                "[ CONTAINMENT {} ]",
                if close >= 1.0 { "LOCKED" } else { "CLOSING" }
            ),
            WHITE,
        );
        let node_y = center_y as i64;
        for (x, name) in [
            (cx - imax(6, box_w.div_euclid(4)), "ME"),
            (cx + imax(6, box_w.div_euclid(4)), "YOU"),
        ] {
            clear(c, (x - 4) as f64, (node_y - 1) as f64, 9.0, 3.0);
            c.box_((x - 4) as f64, (node_y - 1) as f64, 9, 3, WHITE);
            c.put(
                (x - plen(name).div_euclid(2)) as f64,
                node_y as f64,
                name,
                WHITE,
            );
        }
        c.line(
            (cx - box_w.div_euclid(4) + 5) as f64,
            node_y as f64,
            (cx + box_w.div_euclid(4) - 5) as f64,
            node_y as f64,
            '=',
            BRIGHT,
        );
        if t >= 71.764 {
            middle(
                by + box_h - 1,
                &format!("STRANGE / RECURSION {:03}", (age * 13.0) as i64),
                WHITE,
            );
        }
    } else {
        let title_y = center_y as i64 - 2;
        clear(c, ml as f64, title_y as f64, mw as f64, 5.0);
        if mw >= 59 {
            c.big(title_y as f64, "SIMULATION", WHITE);
        } else {
            middle(title_y + 2, ">> SIMULATION <<", WHITE);
        }
        middle(title_y - 2, "[ NO EXIT / SAME WORLD ]", BRIGHT);
        middle(title_y + 6, "[ ME ] <== LOOP ==> [ YOU ]", WHITE);
    }

    middle(top, "EXECUTION -> SIMULATION", BRIGHT);
    middle(top + 1, "world.simulate(me, you);", NORMAL);
    middle(
        bt - 1,
        &format!("EXIT: DENIED / RESTART: {:04}", (age * 17.0) as i64),
        BRIGHT,
    );
    middle(
        bt,
        "TRAPPED TOGETHER / LOOP FOREVER",
        if reveal { WHITE } else { NORMAL },
    );
}

/// Heart model - pulsing 3D heart
pub fn lyric_heart(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64, pulse: f64) {
    let _ = t; // The original signature carries `t`; this scene never reads it.
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    // Heart shape equation
    let scale = 0.5 + pulse * 0.15 + elapsed * 0.03;
    for angle_i in 0..60 {
        for rad_i in 0..15 {
            let angle = angle_i as f64 * TAU / 60.0;
            let rad = rad_i as f64 / 15.0;
            // Heart equation
            let x = 16.0 * angle.sin().powf(3.0);
            let y = -(13.0 * angle.cos()
                - 5.0 * (2.0 * angle).cos()
                - 2.0 * (3.0 * angle).cos()
                - (4.0 * angle).cos());
            let (x, y) = (x * scale * rad / 17.0, y * scale * rad / 17.0);
            let (px, py) = (cx as f64 + x * 4.0, cy as f64 + y * 2.0);
            if (l as f64) < px && px < r as f64 && (top as f64) < py && py < bt as f64 {
                let brightness = rad * (1.0 + pulse * 0.3);
                putc(
                    c,
                    px,
                    py,
                    if brightness > 0.8 {
                        '#'
                    } else if brightness > 0.5 {
                        '*'
                    } else {
                        '.'
                    },
                    if brightness > 0.9 {
                        WHITE
                    } else if brightness > 0.6 {
                        BRIGHT
                    } else {
                        NORMAL
                    },
                );
            }
        }
    }
    if elapsed > 1.0 {
        c.center(cy as f64, "♥", RED);
    }
}

// Original organic choreography restored from the first delivered version.
pub fn legacy_panel(c: &Canvas, x: f64, y: f64, w: i64, h: i64, label: &str) {
    clear(c, x, y, w as f64, h as f64);
    c.box_(x, y, w, h, DIM);
    c.put(
        x + 2.0,
        y,
        &format!(" {} ", cut(label, imax(0, w - 6))),
        NORMAL,
    );
}

pub fn legacy_workspace(
    c: &Canvas,
    t: f64,
    top: i64,
    bt: i64,
    label: &str,
    status: &str,
) -> (i64, i64, i64, i64) {
    c.put(2.0, top as f64, label, WHITE);
    c.put(
        imax(3, c.w - plen(status) - 3) as f64,
        top as f64,
        status,
        BRIGHT,
    );
    c.put(
        2.0,
        (top + 1) as f64,
        &"-".repeat(imax(0, c.w - 4) as usize),
        GREEN,
    );
    let side = if c.w >= 100 {
        imin(25, imax(17, c.w.div_euclid(6)))
    } else {
        0
    };
    if side != 0 {
        let lx = 2;
        let rx = c.w - side - 2;
        let ph = bt - top - 2;
        legacy_panel(c, lx as f64, (top + 2) as f64, side, ph, "REGISTER");
        legacy_panel(c, rx as f64, (top + 2) as f64, side, ph, "PROCESS");
        let k = (t * 7.0) as i64;
        let left = [
            "PID 0001 : ME".to_string(),
            "UID 0002 : YOU".to_string(),
            format!("PC  {:04X}", hash16(k)),
            format!("SP  {:04X}", hash16(k + 3)),
        ];
        let right = [
            format!("STATE {}", cut(status, 7)),
            format!("TICK {:06}", (t * 120.0) as i64),
            format!("CALL {:04X}", hash16(k + 7)),
            "FLAGS Z C O S".to_string(),
        ];
        let height = ph - 2;
        for i in 0..height {
            let yy = top + 3 + i;
            let (a, b): (String, String);
            if i < left.len() as i64 {
                a = left[i as usize].clone();
                b = right[i as usize].clone();
            } else if i == 5 {
                a = "HEAP ALLOCATION".to_string();
                b = "STACK TRACE".to_string();
            } else {
                let j = k + i;
                a = format!("{:04X} {:04X} {:04X}", i * 16, hash16(j), hash16(j + 19));
                let ops = ["LOAD", "PUSH", "CALL", "WAIT", "COPY", "SYNC", "RET ", "JMP "];
                b = format!("{} @{:04X}", ops[j.rem_euclid(8) as usize], hash16(j * 3));
            }
            let style = if i < 4 { NORMAL } else { GREEN };
            c.put((lx + 2) as f64, yy as f64, cut(&a, side - 4), style);
            c.put((rx + 2) as f64, yy as f64, cut(&b, side - 4), style);
        }
        let sweep = ((t * 8.0) as i64).rem_euclid(imax(1, height));
        c.put((lx + 1) as f64, (top + 3 + sweep) as f64, ">", BRIGHT);
        c.put(
            (rx + side - 2) as f64,
            (top + 3 + (height - 1 - sweep)) as f64,
            "<",
            BRIGHT,
        );
    }
    (
        if side != 0 { side + 4 } else { 4 },
        top + 3,
        if side != 0 { c.w - side - 5 } else { c.w - 5 },
        bt - 1,
    )
}

/// Depth-tested parametric point surfaces, lit in character luminance.
///
/// `legacy_organic` calls this with a fractional right edge (`cx + 6`), so the
/// area is kept in `f64`; the integer-area wrapper below preserves the public
/// signature for any other caller.
pub fn legacy_mesh(c: &Canvas, t: f64, area: (i64, i64, i64, i64), form: &str, pulse: f64) {
    legacy_mesh_f(
        c,
        t,
        (area.0 as f64, area.1 as f64, area.2 as f64, area.3 as f64),
        form,
        pulse,
    )
}

fn legacy_mesh_f(c: &Canvas, t: f64, area: (f64, f64, f64, f64), form: &str, pulse: f64) {
    // Python's `dict` keeps insertion order and the draw loop walks it in that
    // order, so the buffer is a `Vec` with the map only used for the `p in zbuf`
    // membership test. Overwriting an existing key keeps its original slot.
    let mut zbuf: Vec<((i64, i64), (f64, f64, f64))> = Vec::new();
    let mut index: HashMap<(i64, i64), usize> = HashMap::new();
    for u in 0..78 {
        let a = u as f64 * TAU / 78.0;
        for v in 0..26 {
            let b = v as f64 * TAU / 26.0;
            let (x, y, z) = if form == "torus" {
                (
                    (0.76 + 0.29 * b.cos()) * a.cos(),
                    (0.76 + 0.29 * b.cos()) * a.sin(),
                    0.29 * b.sin(),
                )
            } else if form == "sphere" {
                (b.sin() * a.cos(), b.cos(), b.sin() * a.sin())
            } else if form == "heart" {
                let radius = 0.5 + 0.5 * b.cos();
                let mut x = (16.0 * a.sin().powf(3.0) / 17.0) * radius;
                let mut y = (-(13.0 * a.cos()
                    - 5.0 * (2.0 * a).cos()
                    - 2.0 * (3.0 * a).cos()
                    - (4.0 * a).cos())
                    / 17.0)
                    * radius;
                let z = 0.38 * b.sin() * a.sin();
                x *= 1.0 + pulse * 0.12;
                y *= 1.0 + pulse * 0.12;
                (x, y, z)
            } else if form == "eggplant" {
                (
                    b.sin() * a.cos() * (0.43 + 0.16 * b.cos()),
                    b.cos() * 1.2,
                    b.sin() * a.sin() * 0.6,
                )
            } else if form == "tomato" {
                (b.sin() * a.cos(), b.cos() * 0.68, b.sin() * a.sin())
            } else {
                (
                    (0.68 + 0.25 * (3.0 * a + b).cos()) * (2.0 * a).cos(),
                    (0.68 + 0.25 * (3.0 * a + b).cos()) * (2.0 * a).sin(),
                    0.5 * (3.0 * a + b).sin(),
                )
            };
            let (xx, yy, zz) = point_f(
                x,
                y,
                z,
                area,
                if form != "heart" {
                    t
                } else {
                    (t * 0.45).sin() * 1.2
                },
                true,
            );
            let p = (pyround(xx) as i64, pyround(yy) as i64);
            match index.get(&p).copied() {
                Some(i) => {
                    if zz < zbuf[i].1 .0 {
                        zbuf[i] = (p, (zz, a, b));
                    }
                }
                None => {
                    index.insert(p, zbuf.len());
                    zbuf.push((p, (zz, a, b)));
                }
            }
        }
    }
    let ramp = ".,:;=+*#@";
    let (l, y, r, bt) = area;
    for &((x, yy), (z, a, b)) in zbuf.iter() {
        if !(l <= x as f64 && x as f64 <= r && y <= yy as f64 && yy as f64 <= bt) {
            continue;
        }
        let light = clamp(0.45 - z * 0.30 + (a * 2.0 + b + t * 0.3).sin() * 0.13);
        let ch = ramp.as_bytes()[(light * (ramp.len() - 1) as f64) as usize] as char;
        let style = if light > 0.77 {
            BRIGHT
        } else if light > 0.40 {
            NORMAL
        } else {
            GREEN
        };
        putc(c, x as f64, yy as f64, ch, style);
    }
}

pub fn legacy_ring(c: &Canvas, t: f64, area: (i64, i64, i64, i64), turns: i64) {
    legacy_ring_f(
        c,
        t,
        (area.0 as f64, area.1 as f64, area.2 as f64, area.3 as f64),
        turns,
    )
}

/// See [`legacy_mesh`] — the ring keeps its centre and radii in `f64` too.
fn legacy_ring_f(c: &Canvas, t: f64, area: (f64, f64, f64, f64), turns: i64) {
    let (l, y, r, b) = area;
    let cx = (l + r) / 2.0;
    let cy = (y + b) / 2.0;
    for k in 0..turns {
        let rr = 0.32 + k as f64 * 0.058;
        for i in 0..140 {
            let a = i as f64 * TAU / 140.0;
            if (i + k * 9).rem_euclid(23) < 5 {
                continue;
            }
            let x = cx + a.cos() * (r - l) * rr;
            let yy = cy + a.sin() * (b - y) * rr;
            putc(
                c,
                x,
                yy,
                if k % 2 != 0 { '.' } else { ':' },
                if k != 1 { GREEN } else { DIM },
            );
        }
        let a = t * (0.6 + k as f64 * 0.1) + k as f64 * 2.0;
        for j in 0..14 {
            let aa = a - j as f64 * 0.018;
            putc(
                c,
                cx + aa.cos() * (r - l) * rr,
                cy + aa.sin() * (b - y) * rr,
                if j == 0 { '+' } else { '.' },
                if j == 0 { WHITE } else { GREEN },
            );
        }
    }
}

pub fn legacy_organic(c: &Canvas, t: f64, top: i64, bt: i64, pulse: f64) {
    let _ = pulse; // The original signature carries `pulse`; this scene never reads it.
    let i = if t < 77.576 {
        0
    } else if t < 81.351 {
        1
    } else if t < 85.078 {
        2
    } else {
        3
    };
    let subject = ["EGGPLANT", "TOMATO", "TABBY CAT", "GOD"][i];
    let resource = ["NUTRIENTS", "ANTIOXIDANTS", "ENJOYMENT", "EXISTENCE"][i];
    let area = legacy_workspace(c, t, top, bt, &format!("TYPE CAST / {}", subject), "EXPORT");
    let (l, y, r, b) = area;
    let cx = (l + r) as f64 / 2.0;
    let cy = (y + b) as f64 / 2.0;
    if matches!(i, 0 | 1) {
        legacy_mesh_f(
            c,
            t,
            (l as f64, y as f64, cx + 6.0, b as f64),
            if i == 0 { "eggplant" } else { "tomato" },
            0.0,
        );
    } else if i == 2 {
        let vs = [
            (-0.9, -0.8, 0.0),
            (-0.75, 0.4, 0.0),
            (0.0, 0.75, 0.0),
            (0.75, 0.4, 0.0),
            (0.9, -0.8, 0.0),
            (0.4, -0.4, 0.0),
            (-0.4, -0.4, 0.0),
            (-0.28, 0.0, -0.1),
            (0.28, 0.0, -0.1),
            (0.0, 0.25, -0.2),
        ];
        let es = [
            (0usize, 1usize),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 0),
            (7, 9),
            (8, 9),
            (5, 8),
            (6, 7),
        ];
        projected_f(
            c,
            &vs,
            &es,
            (l as f64, y as f64, cx + 8.0, b as f64),
            t.sin() * 0.5,
            NORMAL,
            1.0,
        );
        for s in [-1i64, 1] {
            for j in 0..3 {
                c.line(
                    l as f64 + (cx - l as f64) / 2.0,
                    cy + 1.0,
                    l as f64 + (cx - l as f64) / 2.0 + s as f64 * 11.0,
                    cy + j as f64 - 1.0,
                    '.',
                    NORMAL,
                );
            }
        }
    } else {
        legacy_mesh_f(c, t, (l as f64, y as f64, cx + 8.0, b as f64), "sphere", 0.0);
        legacy_ring_f(c, t, (l as f64, y as f64, cx + 8.0, b as f64), 4);
    }
    let target = pyround(mix(cx, r as f64, 0.67));
    c.box_(target - 5.0, pyint(cy - 2.0) as f64, 11, 5, NORMAL);
    c.put(target - 3.0, cy, "YOU_02", WHITE);
    for k in 0..5 {
        let yy = cy - 2.0 + k as f64;
        c.line(cx - 1.0, yy, target - 6.0, yy, '.', GREEN);
        let xx = mix(cx, target - 6.0, (t * 0.8 + k as f64 * 0.2).rem_euclid(1.0));
        c.put(xx, yy, ">>", if k == 2 { BRIGHT } else { DIM });
    }
    c.center(
        y as f64,
        &format!("convert(self, {});", resource.to_lowercase()),
        WHITE,
    );
    c.center(
        b as f64,
        &format!(
            "TX {:04X}  |  {} -> YOU  |  ACK",
            (t.rem_euclid(3.0) / 3.0 * 65535.0) as i64,
            resource
        ),
        NORMAL,
    );
}

/// Luminance scan and short signal tears, confined above the captions.
pub fn legacy_phosphor(c: &Canvas, t: f64, top: i64, bt: i64) {
    let row = top + ((t * 9.0) as i64).rem_euclid(imax(1, bt - top + 1));
    for x in 2..(c.w - 2) {
        let cell = c.get(x, row);
        let filled = match cell.ch {
            Some(ch) => ch != ' ',
            None => false,
        };
        if filled && matches!(cell.style, DIM | NORMAL | GREEN) {
            c.set(
                x,
                row,
                cell.ch,
                if cell.style == GREEN { NORMAL } else { BRIGHT },
            );
        }
    }
    let tick = (t * 13.0) as i64;
    if 125.708 < t && t < 177.246 && matches!(tick.rem_euclid(17), 0 | 1) {
        let yy = top + (hash16(tick) % imax(1, bt - top) as u32) as i64;
        let shift: i64 = if tick % 2 != 0 { 2 } else { -3 };
        if c.w > 4 && yy >= 0 && yy < c.nrows() {
            // `c.cells[yy][2:-2] = source[-shift:] + source[:-shift]` rotates the
            // inner span only, so `Canvas::rotate_row` (which turns the whole
            // row) is not equivalent here.
            let n = (c.w - 4) as usize;
            let mut source: Vec<crate::canvas::Cell> =
                (0..n).map(|i| c.get(i as i64 + 2, yy)).collect();
            source.rotate_right(shift.rem_euclid(n as i64) as usize);
            for (i, cell) in source.iter().enumerate() {
                c.set(i as i64 + 2, yy, cell.ch, cell.style);
            }
        }
    }
}

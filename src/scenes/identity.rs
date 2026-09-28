//! Identity, clock, role switch and the CRT cardiograph — port of
//! `world.execute-me-ascii/scenes.py` lines 1601–2028.

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

/// Python `len(s)` on a `str`. Every string measured in this section is ASCII,
/// so the display width is the same number of columns as code points.
#[inline]
fn plen(s: &str) -> i64 {
    width(s) as i64
}

/// Eggplant/tomato - simple organic shape
pub fn lyric_eggplant_tomato(c: &Canvas, t: f64, area: (i64, i64, i64, i64), item: &str) {
    let _ = t; // The original signature carries `t`; this scene never reads it.
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    // Simple blob
    for y in top..bt {
        for x in l..r {
            let (dx, dy) = ((x - cx) as f64 / 4.0, (y - cy) as f64 / 2.0);
            let dist = dx.hypot(dy);
            if item == "eggplant" {
                // Elongated
                if dist < 6.0 && dy.abs() < 8.0 {
                    putc(
                        c,
                        x as f64,
                        y as f64,
                        if dist < 4.0 { '█' } else { '▓' },
                        if dy < 0.0 { BRIGHT } else { NORMAL },
                    );
                }
            } else {
                // Round
                if dist < 5.0 {
                    putc(
                        c,
                        x as f64,
                        y as f64,
                        if dist < 3.0 { '●' } else { 'o' },
                        if dist < 3.0 { RED } else { NORMAL },
                    );
                }
            }
        }
    }
    c.center(
        (cy + ((bt - top) as f64 * 0.3) as i64) as f64,
        &item.to_uppercase(),
        BRIGHT,
    );
}

/// Tabby cat - ASCII cat
pub fn lyric_cat(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let _ = t; // The original signature carries `t`; this scene never reads it.
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let cat = [
        "  /\\_/\\  ",
        " ( o.o ) ",
        "  > ^ <  ",
        " /|   |\\ ",
    ];
    for (i, line) in cat.iter().enumerate() {
        c.center((cy - 2 + i as i64) as f64, line, if i < 3 { BRIGHT } else { NORMAL });
    }
    // Purr waves
    if elapsed > 0.5 {
        for i in 0..5i64 {
            let phase = (elapsed * 2.0 + i as f64 * 0.2).rem_euclid(1.0);
            let x = cx + (phase * 20.0) as i64 - 10;
            // The original writes `''` on the tail of the fade, which is a no-op.
            if phase < 0.8 {
                putc(c, x as f64, (cy + 3) as f64, '~', GREEN);
            }
        }
    }
    c.center((cy + 5) as f64, "*purr*", GREEN);
}

/// God/existence - radiant aura
pub fn lyric_god_existence(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let _ = elapsed; // The original signature carries `elapsed`; this scene never reads it.
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    // Radiating light
    for ring in 0..8i64 {
        let radius = 5.0 + ring as f64 * 4.0 + (t * 2.0 + ring as f64).sin() * 2.0;
        let density = 60 - ring * 5;
        for i in 0..density {
            let spin = if ring % 2 == 0 { 1.0 } else { -1.0 };
            let angle = i as f64 * TAU / density as f64 + t * 0.1 * spin;
            let x = cx as f64 + angle.cos() * radius;
            let y = cy as f64 + angle.sin() * radius * 0.5;
            let brightness = 1.0 - ring as f64 / 8.0;
            putc(
                c,
                x,
                y,
                if brightness > 0.7 {
                    '*'
                } else if brightness > 0.4 {
                    '+'
                } else {
                    '.'
                },
                if brightness > 0.8 {
                    WHITE
                } else if brightness > 0.5 {
                    BRIGHT
                } else {
                    NORMAL
                },
            );
        }
    }
    c.center(cy as f64, "GOD", WHITE);
    c.center((cy + 2) as f64, "YOU", BRIGHT);
}

/// Draw `text` from the shared 5×5 bitmap font, `sx`×`sy` cells per pixel.
///
/// The original carries its own `glyphs` table for `F`, `M`, `A`, `P`; those
/// four entries are identical to the shared `FONT`, so this uses the font.
pub fn identity_bitmap(
    c: &Canvas,
    x: f64,
    y: f64,
    text: &str,
    sx: f64,
    sy: f64,
    ink: Style,
    fill: f64,
    seed: i64,
) {
    // Every caller passes integral scale factors, and the original feeds them
    // straight into `range(sx)` / `range(sy)`.
    let (sx, sy) = (sx as i64, sy as i64);
    for (index, ch) in text.chars().enumerate() {
        let index = index as i64;
        for (dy, row) in crate::font::glyph(ch).iter().enumerate() {
            let dy = dy as i64;
            for (dx, pixel) in row.chars().enumerate() {
                let dx = dx as i64;
                if pixel == '1' {
                    for py in 0..sy {
                        for px in 0..sx {
                            let xx = x + ((index * 6 + dx) * sx + px) as f64;
                            let yy = y + (dy * sy + py) as f64;
                            let on = hash16(
                                (index * 31 + dx) * 73 + dy * 137 + px * 19 + py + seed,
                            ) as f64
                                / 65535.0
                                <= fill;
                            putc(
                                c,
                                xx,
                                yy,
                                if on { '#' } else { '.' },
                                if on { ink } else { GREEN },
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Rewrite a gender field by disassembling and transmitting its character cells.
pub fn lyric_identity_rewrite(c: &Canvas, t: f64, area: (i64, i64, i64, i64)) {
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let age = t - 88.587;
    let u = clamp((t - 90.197) / 1.25);
    let sx = imax(2, imin(6, (r - l).div_euclid(22)));
    let sy = imax(1, imin(5, (bt - top - 8).div_euclid(5)));
    let glyph_w = 5 * sx;
    let glyph_h = 5 * sy;
    let left_c = (l + cx) / 2;
    let right_c = (cx + r) / 2;
    let y = cy - glyph_h.div_euclid(2);
    c.center(top as f64, "SELF.GENDER / PARAMETER REWRITE", WHITE);
    c.center((top + 1) as f64, "F -> M / TRANSMIT IDENTITY", BRIGHT);
    c.box_(l as f64, (top + 3) as f64, cx - l - 1, bt - top - 5, GREEN);
    c.box_(
        (cx + 2) as f64,
        (top + 3) as f64,
        r - cx - 1,
        bt - top - 5,
        GREEN,
    );
    // Scrolling bytes fill both panes, behind the large letter masks.
    for row in ((top + 4)..(bt - 2)).step_by(2) {
        let tick = (age * 18.0) as i64 + row;
        c.put((l + 2) as f64, row as f64, &format!("{:04X}", hash16(tick)), GREEN);
        c.put(
            (r - 5) as f64,
            row as f64,
            &format!("{:04X}", hash16(tick + 79)),
            GREEN,
        );
    }
    for i in 0..6i64 {
        let yy = top + 4 + i * imax(1, (bt - top - 8).div_euclid(5));
        c.line((l + 7) as f64, yy as f64, (r - 7) as f64, yy as f64, '.', GREEN);
        let progress = (age * 0.9 + i as f64 * 0.17).rem_euclid(1.0);
        let x = mix(left_c as f64, right_c as f64, progress);
        c.put(x, yy as f64, ">>", if u != 0.0 { WHITE } else { BRIGHT });
    }
    clear(
        c,
        (left_c - glyph_w.div_euclid(2) - 1) as f64,
        (y - 1) as f64,
        (glyph_w + 2) as f64,
        (glyph_h + 2) as f64,
    );
    clear(
        c,
        (right_c - glyph_w.div_euclid(2) - 1) as f64,
        (y - 1) as f64,
        (glyph_w + 2) as f64,
        (glyph_h + 2) as f64,
    );
    identity_bitmap(
        c,
        (left_c - glyph_w.div_euclid(2)) as f64,
        y as f64,
        "F",
        sx as f64,
        sy as f64,
        WHITE,
        1.0 - u,
        0,
    );
    identity_bitmap(
        c,
        (right_c - glyph_w.div_euclid(2)) as f64,
        y as f64,
        "M",
        sx as f64,
        sy as f64,
        WHITE,
        u,
        0,
    );
    // Released source cells travel in arcs and settle into the new character.
    if 0.0 < u && u < 1.0 {
        for i in 0..44i64 {
            let p = clamp(u * 1.6 - (i % 11) as f64 / 18.0);
            let x = mix(left_c as f64, right_c as f64, p);
            let yoff = (hash16(i * 31) % imax(1, glyph_h) as u32) as i64 as f64 - glyph_h as f64 / 2.0;
            let sign = if i % 2 != 0 { 1.0 } else { -1.0 };
            let yy = cy as f64 + yoff + (p * std::f64::consts::PI).sin() * sign * 3.0;
            putc(
                c,
                x,
                yy,
                ['0', '1', '#'][(i % 3) as usize],
                if i % 4 == 0 { WHITE } else { BRIGHT },
            );
        }
    }
    c.put(
        (left_c - 4) as f64,
        (bt - 3) as f64,
        "SOURCE F",
        if u < 1.0 { NORMAL } else { GREEN },
    );
    c.put(
        (right_c - 4) as f64,
        (bt - 3) as f64,
        "TARGET M",
        if u >= 1.0 { WHITE } else { NORMAL },
    );
    c.center(
        (bt - 1) as f64,
        &format!(
            "WRITE {:03}% / {}",
            (u * 100.0) as i64,
            if u >= 1.0 {
                "COMMITTED"
            } else if u == 0.0 {
                "COMPILING"
            } else {
                "REASSEMBLING"
            }
        ),
        BRIGHT,
    );
    c.center(
        bt as f64,
        if u >= 1.0 {
            "self.gender = 'M';"
        } else {
            "self.gender: F -> M"
        },
        WHITE,
    );
}

/// An oversized clock races through daylight and flips its AM/PM display.
pub fn lyric_daynight_clock(c: &Canvas, t: f64, area: (i64, i64, i64, i64)) {
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let age = t - 92.015;
    let flip_at = 94.55;
    let is_pm = t >= flip_at;
    let virtual_ = if !is_pm {
        6.0 + 6.0 * clamp((t - 92.015) / (flip_at - 92.015))
    } else {
        12.0 + 6.0 * clamp((t - flip_at) / (95.465 - flip_at))
    };
    let clock_x = (l + cx) / 2;
    let display_x = (cx + r) / 2;
    let rx = fmax(6.0, (cx - l) as f64 * 0.43);
    let ry = fmax(3.0, (bt - top - 6) as f64 * 0.43);
    c.center(top as f64, "CLOCK.CYCLE / AM -> PM", WHITE);
    c.center(
        (top + 1) as f64,
        "DAYLIGHT -> NIGHT / TIME ACCELERATING",
        BRIGHT,
    );
    // Clock face fills the left half instead of occupying a small central patch.
    for i in 0..180i64 {
        let a = i as f64 * TAU / 180.0;
        putc(
            c,
            clock_x as f64 + a.sin() * rx,
            cy as f64 - a.cos() * ry,
            '.',
            NORMAL,
        );
    }
    for hour in 0..12i64 {
        let a = hour as f64 * TAU / 12.0;
        c.line(
            clock_x as f64 + a.sin() * rx * 0.9,
            cy as f64 - a.cos() * ry * 0.9,
            clock_x as f64 + a.sin() * rx,
            cy as f64 - a.cos() * ry,
            '#',
            BRIGHT,
        );
        c.put(
            clock_x as f64 + a.sin() * rx * 0.77 - 1.0,
            cy as f64 - a.cos() * ry * 0.77,
            &format!("{}", if hour != 0 { hour } else { 12 }),
            NORMAL,
        );
    }
    let hour_angle = virtual_ / 12.0 * TAU;
    let minute_angle = virtual_.rem_euclid(1.0) * TAU;
    for trail in (1..=4i64).rev() {
        let a = minute_angle - trail as f64 * 0.13;
        c.line(
            clock_x as f64,
            cy as f64,
            clock_x as f64 + a.sin() * rx * 0.8,
            cy as f64 - a.cos() * ry * 0.8,
            '.',
            GREEN,
        );
    }
    c.line(
        clock_x as f64,
        cy as f64,
        clock_x as f64 + hour_angle.sin() * rx * 0.52,
        cy as f64 - hour_angle.cos() * ry * 0.52,
        '#',
        BRIGHT,
    );
    c.line(
        clock_x as f64,
        cy as f64,
        clock_x as f64 + minute_angle.sin() * rx * 0.83,
        cy as f64 - minute_angle.cos() * ry * 0.83,
        '*',
        WHITE,
    );
    c.put(clock_x as f64, cy as f64, "@", WHITE);
    c.line(cx as f64, (top + 3) as f64, cx as f64, (bt - 3) as f64, '|', GREEN);
    // Day/night symbol and huge AM/PM bitmap occupy the entire right pane.
    let radius = fmax(
        3.0,
        fmin((r - cx) as f64 * 0.22, (bt - top) as f64 * 0.34),
    );
    for i in 0..120i64 {
        let a = i as f64 * TAU / 120.0;
        let x = display_x as f64 + a.cos() * radius * 1.6;
        let y = cy as f64 + a.sin() * radius;
        if !is_pm || a.cos() < 0.45 {
            putc(c, x, y, ':', if is_pm { GREEN } else { NORMAL });
        }
    }
    if !is_pm {
        for ray in 0..16i64 {
            let a = ray as f64 * TAU / 16.0 + age * 0.25;
            c.line(
                display_x as f64 + a.cos() * radius * 1.8,
                cy as f64 + a.sin() * radius * 1.1,
                display_x as f64 + a.cos() * radius * 2.1,
                cy as f64 + a.sin() * radius * 1.3,
                '.',
                GREEN,
            );
        }
    } else {
        for star in 0..22i64 {
            let x = cx + 2 + (hash16(star * 31) % imax(1, r - cx - 4) as u32) as i64;
            let y = top + 3 + (hash16(star * 79) % imax(1, bt - top - 6) as u32) as i64;
            putc(
                c,
                x as f64,
                y as f64,
                if ((age * 5.0) as i64 + star).rem_euclid(5) == 0 {
                    '+'
                } else {
                    '.'
                },
                GREEN,
            );
        }
    }
    let sx = imax(1, imin(4, (r - cx - 6).div_euclid(11)));
    let sy = imax(1, imin(4, (bt - top - 8).div_euclid(5)));
    let glyph_w = 11 * sx;
    let glyph_h = 5 * sy;
    let y0 = cy - glyph_h.div_euclid(2);
    clear(
        c,
        (display_x - glyph_w.div_euclid(2) - 1) as f64,
        (y0 - 1) as f64,
        (glyph_w + 2) as f64,
        (glyph_h + 2) as f64,
    );
    identity_bitmap(
        c,
        (display_x - glyph_w.div_euclid(2)) as f64,
        y0 as f64,
        if is_pm { "PM" } else { "AM" },
        sx as f64,
        sy as f64,
        WHITE,
        1.0,
        0,
    );
    // A narrow scan sweeps down the display on the flip.
    if 0.0 <= t - flip_at && t - flip_at < 0.22 {
        let yy = y0 + ((t - flip_at) / 0.22 * glyph_h as f64) as i64;
        c.put(
            (display_x - glyph_w.div_euclid(2)) as f64,
            yy as f64,
            &"=".repeat(glyph_w as usize),
            WHITE,
        );
    }
    let hour = (virtual_ as i64).rem_euclid(24);
    let minute = ((virtual_.rem_euclid(1.0)) * 60.0) as i64;
    c.put(
        (display_x - 2) as f64,
        imin(bt - 3, y0 + glyph_h + 1) as f64,
        &format!("{:02}:{:02}", hour, minute),
        WHITE,
    );
    c.center(
        (bt - 1) as f64,
        if is_pm {
            "[ PM / NIGHT CYCLE ]"
        } else {
            "[ AM / DAY CYCLE ]"
        },
        BRIGHT,
    );
    c.center(bt as f64, "do_whatever();  // AM -> PM", NORMAL);
}

/// Gender/role switching - full screen symbolic transformation
pub fn lyric_gender_role_switch(
    c: &Canvas,
    t: f64,
    area: (i64, i64, i64, i64),
    elapsed: f64,
    from_label: &str,
    to_label: &str,
) {
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let progress = clamp(elapsed / 2.5);

    // ===== F→M: Gender symbols transformation =====
    if from_label == "F" && to_label == "M" {
        // Full screen: ♀ symbols morphing to ♂
        for i in 0..150i64 {
            let angle = i as f64 * 2.4 + t * 0.5;
            let radius_base = (i as f64).sqrt() * 4.0;
            let radius = radius_base + (t * 2.0 + i as f64 * 0.1).sin() * 3.0;
            let x = cx as f64 + angle.cos() * radius;
            let y = cy as f64 + angle.sin() * radius * 0.5;

            // Morph symbol based on progress
            let (symbol, style) = if progress < 0.3 {
                (
                    '♀',
                    if i % 5 == 0 {
                        WHITE
                    } else if i % 3 == 0 {
                        BRIGHT
                    } else {
                        NORMAL
                    },
                )
            } else if progress < 0.7 {
                // Transition phase - mix of both
                let symbol = if (i + (t * 10.0) as i64) % 2 == 0 {
                    '♀'
                } else {
                    '♂'
                };
                let style = if (i + (t * 20.0) as i64) % 3 == 0 {
                    RED
                } else if i % 4 == 0 {
                    WHITE
                } else {
                    BRIGHT
                };
                (symbol, style)
            } else {
                (
                    '♂',
                    if i % 5 == 0 {
                        WHITE
                    } else if i % 3 == 0 {
                        BRIGHT
                    } else {
                        NORMAL
                    },
                )
            };
            putc(c, x, y, symbol, style);
        }

        // Central explosion during transition
        if 0.3 < progress && progress < 0.7 {
            let burst_progress = (progress - 0.3) / 0.4;
            for burst_i in 0..60i64 {
                let burst_angle = burst_i as f64 * TAU / 60.0 + t * 3.0;
                let burst_r = burst_progress * 50.0;
                let bx = cx as f64 + burst_angle.cos() * burst_r;
                let by = cy as f64 + burst_angle.sin() * burst_r * 0.5;
                putc(
                    c,
                    bx,
                    by,
                    if burst_i % 3 == 0 { '⚥' } else { '*' },
                    WHITE,
                );
            }
        }

        // Large center symbol
        let scale = 8 + ((t * 2.0).sin() * 2.0) as i64;
        let center_symbol = if progress < 0.5 { '♀' } else { '♂' };
        for dy in -scale..=scale {
            for dx in (-scale * 2)..=(scale * 2) {
                let dist = (dx as f64 / 2.0).hypot(dy as f64);
                if dist < scale as f64 {
                    putc(
                        c,
                        (cx + dx) as f64,
                        (cy + dy) as f64,
                        center_symbol,
                        if dist < scale as f64 * 0.4 {
                            WHITE
                        } else if dist < scale as f64 * 0.7 {
                            BRIGHT
                        } else {
                            NORMAL
                        },
                    );
                }
            }
        }
    // ===== AM→PM: Clock progression =====
    } else if from_label == "AM" && to_label == "PM" {
        // Clock face with hands sweeping
        let radius_clock = (imin(r - l, bt - top) as f64 * 0.35) as i64;

        // Clock circle
        for i in 0..120i64 {
            let angle = i as f64 * TAU / 120.0;
            let x = cx as f64 + angle.cos() * radius_clock as f64;
            let y = cy as f64 + angle.sin() * radius_clock as f64 * 0.5;
            putc(c, x, y, if i % 10 == 0 { '○' } else { '·' }, BRIGHT);
        }

        // Hour markers (12, 3, 6, 9)
        for hour in [0i64, 3, 6, 9] {
            let angle = hour as f64 * TAU / 12.0 - TAU / 4.0; // -90deg offset
            let x = cx as f64 + angle.cos() * radius_clock as f64 * 0.85;
            let y = cy as f64 + angle.sin() * radius_clock as f64 * 0.85 * 0.5;
            c.put(
                x,
                y,
                &format!("{}", if hour != 0 { hour } else { 12 }),
                WHITE,
            );
        }

        // Clock hands rotating from AM (6:00) to PM (18:00)
        let start_hour = 6.0;
        let end_hour = 18.0;
        let current_hour = mix(start_hour, end_hour, progress);

        // Hour hand
        let hour_angle = current_hour * TAU / 12.0 - TAU / 4.0;
        let hour_length = (radius_clock as f64 * 0.5) as i64;
        for step in 0..hour_length {
            let x = cx as f64 + hour_angle.cos() * step as f64;
            let y = cy as f64 + hour_angle.sin() * step as f64 * 0.5;
            putc(
                c,
                x,
                y,
                '═',
                if step as f64 > hour_length as f64 * 0.7 {
                    WHITE
                } else {
                    BRIGHT
                },
            );
        }

        // Minute hand (spinning fast)
        let minute_angle = (t * 6.0).rem_euclid(TAU) - TAU / 4.0;
        let minute_length = (radius_clock as f64 * 0.7) as i64;
        for step in 0..minute_length {
            let x = cx as f64 + minute_angle.cos() * step as f64;
            let y = cy as f64 + minute_angle.sin() * step as f64 * 0.5;
            putc(
                c,
                x,
                y,
                '─',
                if step as f64 > minute_length as f64 * 0.8 {
                    BRIGHT
                } else {
                    NORMAL
                },
            );
        }

        // Time digits cascading
        for digit_y in 0..((bt - top) as f64 * 0.6) as i64 {
            let phase = (progress * 3.0 + digit_y as f64 * 0.05).rem_euclid(1.0);
            if phase < 0.8 {
                let digit_x = (cx as f64 + (phase * TAU).sin() * 30.0) as i64;
                let hour_shown = mix(6.0, 18.0, phase) as i64 % 24;
                c.put(
                    digit_x as f64,
                    (top + digit_y) as f64,
                    &format!("{:02}", hour_shown),
                    if phase > 0.6 {
                        WHITE
                    } else if phase > 0.3 {
                        BRIGHT
                    } else {
                        GREEN
                    },
                );
            }
        }

        // Central display
        let display_hour = current_hour as i64 % 24;
        c.center(
            (cy + ((bt - top) as f64 * 0.25) as i64) as f64,
            &format!("{:02}:00", display_hour),
            WHITE,
        );
        c.center(
            (cy + ((bt - top) as f64 * 0.32) as i64) as f64,
            if current_hour < 12.0 { "AM" } else { "PM" },
            if current_hour >= 12.0 { RED } else { BRIGHT },
        );
    // ===== S→M: Dominance to submission (chains/waves) =====
    } else {
        // S to M
        // Visual: Sharp edges (S) flowing into smooth curves (M)

        // Background: Transitioning pattern
        if progress < 0.5 {
            // S phase: Sharp angles, rigid structure
            for row in (top..=bt).step_by(3) {
                for x in (l..r).step_by(8) {
                    let offset = (t * 10.0 + row as f64).rem_euclid(8.0) as i64;
                    let pattern = if row.rem_euclid(6) < 3 {
                        ['╱', '╲']
                    } else {
                        ['╲', '╱']
                    };
                    putc(c, (x + offset) as f64, row as f64, pattern[0], NORMAL);
                    putc(c, (x + offset + 1) as f64, row as f64, pattern[1], NORMAL);
                }
            }
        } else {
            // M phase: Smooth waves, flowing
            for wave_y in 0..12i64 {
                let y = top + (wave_y as f64 * (bt - top) as f64 / 11.0) as i64;
                for x in l..r {
                    let wave = (x - l) as f64 / (r - l) as f64 * TAU * 3.0 - t * 2.0;
                    let amplitude = (bt - top) as f64 * 0.1 * (progress - 0.5) * 2.0;
                    let offset = (wave.sin() * amplitude) as i64;
                    putc(
                        c,
                        x as f64,
                        (y + offset) as f64,
                        if wave_y % 2 == 0 { '~' } else { '≈' },
                        if wave_y % 3 == 0 { BRIGHT } else { NORMAL },
                    );
                }
            }
        }

        // Center transformation
        if progress < 0.4 {
            // S: Angular crown/spikes
            let spike_count = 8.0;
            for i in 0..8i64 {
                let angle = i as f64 * TAU / spike_count + t * 0.5;
                let base_r = 15.0;
                for r_step in 0..20i64 {
                    let spike_r = base_r + r_step as f64 * 1.5;
                    let spike_x = cx as f64 + angle.cos() * spike_r;
                    let spike_y = cy as f64 + angle.sin() * spike_r * 0.5;
                    if (angle.rem_euclid(TAU / spike_count)).abs() < 0.2 {
                        // Make spikes
                        putc(
                            c,
                            spike_x,
                            spike_y,
                            if r_step % 2 == 0 { '▲' } else { '△' },
                            if r_step > 15 {
                                WHITE
                            } else if r_step > 10 {
                                BRIGHT
                            } else {
                                NORMAL
                            },
                        );
                    }
                }
            }
        } else if progress < 0.6 {
            // Transition: Explosion
            let trans = (progress - 0.4) / 0.2;
            for i in 0..80i64 {
                let angle = i as f64 * TAU / 80.0;
                let radius = trans * 60.0;
                let x = cx as f64
                    + angle.cos() * radius
                    + (t * 4.0 + i as f64).sin() * 5.0 * (1.0 - trans);
                let y = cy as f64
                    + angle.sin() * radius * 0.5
                    + (t * 4.0 + i as f64).cos() * 3.0 * (1.0 - trans);
                putc(
                    c,
                    x,
                    y,
                    if i % 3 == 0 { '*' } else { '·' },
                    if trans < 0.5 { WHITE } else { BRIGHT },
                );
            }
        } else {
            // M: Soft concentric circles
            let _circles = (progress - 0.6) / 0.4; // read by nothing in the original
            for ring in 0..8i64 {
                let ring_r = 8 + ring * 4;
                let density = ring_r * 6;
                for i in 0..density {
                    let spin = if ring % 2 == 0 { 1.0 } else { -1.0 };
                    let angle = i as f64 * TAU / density as f64 + t * 0.3 * spin;
                    let x = cx as f64 + angle.cos() * ring_r as f64;
                    let y = cy as f64 + angle.sin() * ring_r as f64 * 0.5;
                    putc(
                        c,
                        x,
                        y,
                        if ring % 2 == 0 { '○' } else { '◯' },
                        if ring < 3 {
                            WHITE
                        } else if ring < 5 {
                            BRIGHT
                        } else {
                            NORMAL
                        },
                    );
                }
            }
        }

        // Large central letter
        let size = (10.0 + (t * 1.5).sin() * 1.5) as i64;
        // The original's `if not isinstance(size, int): size=int(size)` cannot
        // fire, because `int(...)` already produced an `int`.
        let center_char = if progress < 0.5 { 'S' } else { 'M' };
        for dy in -size..=size {
            for dx in (-size * 2)..=(size * 2) {
                let dist = (dx as f64 / 2.0).hypot(dy as f64);
                if size as f64 * 0.3 < dist && dist < size as f64 * 0.8 {
                    putc(
                        c,
                        (cx + dx) as f64,
                        (cy + dy) as f64,
                        center_char,
                        if dist > size as f64 * 0.6 {
                            WHITE
                        } else {
                            BRIGHT
                        },
                    );
                }
            }
        }
    }

    // Common elements: Status and decorations
    c.center(
        top as f64,
        &format!("{} → {}", from_label, to_label),
        if progress > 0.8 { WHITE } else { BRIGHT },
    );
    c.center(
        bt as f64,
        &format!("TRANSFORMATION: {}%", (progress * 100.0) as i64),
        NORMAL,
    );
}

/// Stylized P-QRS-T trace, positive values point up on the monitor.
pub fn ecg_sample(phase: f64) -> f64 {
    let points = [
        (0.0, 0.0),
        (0.08, 0.0),
        (0.12, 0.15),
        (0.17, 0.0),
        (0.28, 0.0),
        (0.31, -0.18),
        (0.35, 1.0),
        (0.39, -0.32),
        (0.43, 0.0),
        (0.52, 0.0),
        (0.60, 0.25),
        (0.70, 0.0),
        (1.0, 0.0),
    ];
    let phase = phase.rem_euclid(1.0);
    for i in 1..points.len() {
        let (x1, y1) = points[i];
        let (x0, y0) = points[i - 1];
        if phase <= x1 {
            return mix(y0, y1, (phase - x0) / (x1 - x0));
        }
    }
    0.0
}

/// Swept CRT cardiograph: feel YOU, then bring ME into the same rhythm.
pub fn lyric_vibration_sync(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    let compact = bt - top < 19;
    let sync = clamp((t - 107.22) / (110.221 - 107.22));
    let complete = t >= 110.221;
    let phase_lag = 0.28 * (1.0 - sync);
    let bpm = 72.0;
    c.put(l as f64, top as f64, "ECG / DUAL CHANNEL", BRIGHT);
    c.put(
        (r - 10) as f64,
        top as f64,
        &format!("{:03} BPM", bpm as i64),
        WHITE,
    );
    if !compact {
        let title = if complete {
            "COMPLETION / RHYTHM LOCKED"
        } else if t >= 107.22 {
            "PHASE SYNCHRONIZING"
        } else if t >= 106.293 {
            "VIBRATIONS DETECTED"
        } else {
            "ACQUIRING YOUR HEARTBEAT"
        };
        c.center(
            (top + 1) as f64,
            title,
            if complete { WHITE } else { NORMAL },
        );
    }
    let plot_top = top + if compact { 2 } else { 4 };
    let plot_bt = bt - if compact { 1 } else { 3 };
    let split = (plot_top + plot_bt).div_euclid(2);
    let x0 = l + 1;
    let x1 = r - 1;
    let span = x1 - x0 + 1;
    // Three beats fit the screen. The head traverses it every 2.5 seconds.
    let speed = span as f64 / 2.5;
    let sweep = elapsed * speed + span as f64 * 0.30;
    let head = x0 + (sweep as i64).rem_euclid(span);
    let gap = imax(2, (span as f64 * 0.025) as i64);
    for yy in plot_top..=plot_bt {
        for xx in x0..=x1 {
            if (xx - x0) % 10 == 0 && (yy - plot_top) % 3 == 0 {
                putc(c, xx as f64, yy as f64, '+', GREEN);
            } else if (yy - plot_top) % 3 == 0 && (xx - x0) % 2 == 0 {
                putc(c, xx as f64, yy as f64, '.', GREEN);
            }
        }
    }
    for yy in plot_top..=plot_bt {
        putc(c, head as f64, yy as f64, ':', GREEN);
    }
    let lanes = [
        (plot_top, split, "YOU", 0.0),
        (split + 1, plot_bt, "ME", phase_lag),
    ];
    for (lane, (start, end, name, lag)) in lanes.iter().copied().enumerate() {
        let height = end - start + 1;
        let baseline = start + ((height - 1) as f64 * 0.68) as i64;
        let amplitude = fmax(1.0, (height - 2) as f64 * 0.58);
        c.put(
            x0 as f64,
            start as f64,
            name,
            if lane == 0 || complete { WHITE } else { NORMAL },
        );
        let mut previous: Option<(f64, f64)> = None;
        let mut previous_age = 0.0f64;
        // Sub-cell sampling connects the steep QRS spike instead of leaving isolated dots.
        for sample in 0..span * 4 {
            let xx = x0 as f64 + sample as f64 / 4.0;
            let age = (head as f64 - xx).rem_euclid(span as f64);
            if age > span as f64 - gap as f64 {
                previous = None;
                continue;
            }
            let signal_time = elapsed - age / speed;
            let phase = signal_time * bpm / 60.0 - lag;
            let mut yy = baseline as f64 - ecg_sample(phase) * amplitude;
            yy = fmax(start as f64, fmin(end as f64, yy));
            let mut ink = if age < span as f64 * 0.10 {
                WHITE
            } else if age < span as f64 * 0.50 {
                BRIGHT
            } else {
                NORMAL
            };
            if lane == 1 && !complete {
                ink = if age < span as f64 * 0.15 {
                    BRIGHT
                } else {
                    NORMAL
                };
            }
            if let Some((px, py)) = previous {
                // Crossing the sweep reset starts a new path; it must not create a false spike.
                if (age - previous_age).abs() < 2.0 {
                    let dy = yy - py;
                    let ch = if dy.abs() > 0.65 {
                        '|'
                    } else if dy < -0.13 {
                        '/'
                    } else if dy > 0.13 {
                        '\\'
                    } else {
                        '-'
                    };
                    c.line(px, py, xx, yy, ch, ink);
                }
            }
            previous = Some((xx, yy));
            previous_age = age;
        }
        let mut tip = baseline as f64 - ecg_sample(elapsed * bpm / 60.0 - lag) * amplitude;
        tip = fmax(start as f64, fmin(end as f64, tip));
        // Bright writing point with a short phosphor afterglow, independent of terminal theme.
        c.put((head - 1) as f64, tip, "=", BRIGHT);
        c.put(head as f64, tip, "@", WHITE);
        if !compact {
            c.put(
                (x1 - 7) as f64,
                start as f64,
                if complete {
                    "IN SYNC"
                } else if lane == 0 {
                    "SENSED"
                } else {
                    "SEEKING"
                },
                if complete { BRIGHT } else { NORMAL },
            );
        }
    }
    let indicator = if ecg_sample(elapsed * bpm / 60.0) > 0.65 {
        '*'
    } else {
        '.'
    };
    if !compact {
        c.put(
            l as f64,
            (bt - 1) as f64,
            &format!("BEAT [{}]  /  YOU -> ME", indicator),
            BRIGHT,
        );
        let status = format!(
            "SYNC {:03}%  DELAY {:03}ms",
            (sync * 100.0) as i64,
            (phase_lag * 1000.0 / (bpm / 60.0)) as i64
        );
        c.put(
            (r - plen(&status) + 1) as f64,
            (bt - 1) as f64,
            &status,
            if complete { WHITE } else { BRIGHT },
        );
    }
    c.center(
        bt as f64,
        if complete {
            "[ COMPLETION / HEARTBEATS SYNCHRONIZED ]"
        } else if t < 107.22 {
            "[ FEEL YOUR VIBRATIONS ]"
        } else {
            "[ MATCHING YOUR RHYTHM ]"
        },
        if complete { WHITE } else { NORMAL },
    );
}

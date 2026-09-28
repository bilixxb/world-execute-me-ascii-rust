//! Title takeover, the main scene dispatcher, and the phosphor scan line —
//! port of `scenes.py` lines 2777–3073.

use crate::canvas::Canvas;
use crate::core;
use crate::core::{hash16, pyround, BRIGHT, DARK, DIM, GREEN, NORMAL, RED, WHITE};
use crate::film::Film;
use crate::scenes::common::*;
use crate::scenes::{legacy_organic, legacy_phosphor};
use crate::TAU;

/// Python `title_pixels(w, h, font)` — the bitmap of the big centred title.
///
/// The original caches this in a module-level `TITLE_CACHE`; the cache lives on
/// `Film` here so the cache is owned rather than global, and is keyed by
/// `(w, h)` exactly as the original does.
pub fn title_pixels(film: &Film, w: i64, h: i64) -> Vec<(i64, i64)> {
    film.title_pixels(w, h)
}

/// Python `title_takeover(c, t, font, source=None)`.
///
/// The entire terminal disintegrates and reforms as a large title.
pub fn title_takeover(c: &Canvas, t: f64, source: Option<&Canvas>, film: &Film) {
    let elapsed = t - 16.0;
    let (w, h) = (c.w, c.h);
    let cx = (w - 1) as f64 / 2.0;
    let cy = (h - 1) as f64 / 2.0;
    let alphabet: Vec<char> = "0123456789ABCDEF<>[]{}();:=/\\|+-*#".chars().collect();
    let ink = title_pixels(film, w, h);
    let frame = (elapsed * 24.0) as i64;
    let lock = clamp((elapsed - 4.1) / 2.8);
    let dissolve = clamp((elapsed - 11.35) / 2.36);

    // PRE-TRANSITION EFFECT (15.8-16s becomes -0.2-0s elapsed)
    if elapsed < 0.0 {
        // Intensifying distortion before takeover
        let buildup = clamp((elapsed + 0.2) / 0.2); // 0 to 1 over 0.2 seconds

        // Flash effect - increasing frequency
        if ((t * 30.0 * buildup) as i64) % 2 == 0 {
            for y in 0..h {
                for x in 0..w {
                    if hash_below(hash16(x + y * w + (t * 100.0) as i64), 100, buildup * 50.0) {
                        c.set(
                            x,
                            y,
                            Some('█'),
                            if buildup > 0.7 { WHITE } else { BRIGHT },
                        );
                    }
                }
            }
        }

        // Compression waves from edges
        let wave_count = (buildup * 5.0) as i64 + 1;
        for i in 0..wave_count {
            let phase = (buildup * 3.0 - i as f64 * 0.15).rem_euclid(1.0);
            if phase < 0.0 {
                continue;
            }
            // Top/bottom waves
            let y_top = (phase * h as f64 / 2.0) as i64;
            let y_bot = h - 1 - y_top;
            for x in 0..w {
                if hash16(x + i) % 3 == 0 {
                    let ch = if phase > 0.7 { '=' } else { '-' };
                    let st = if phase > 0.8 { WHITE } else { BRIGHT };
                    c.put(x as f64, y_top as f64, &ch.to_string(), st);
                    c.put(x as f64, y_bot as f64, &ch.to_string(), st);
                }
            }
            // Left/right waves
            let x_left = (phase * w as f64 / 2.0) as i64;
            let x_right = w - 1 - x_left;
            for y in 0..h {
                if hash16(y + i * 7) % 3 == 0 {
                    let st = if phase > 0.8 { WHITE } else { BRIGHT };
                    c.put(x_left as f64, y as f64, "|", st);
                    c.put(x_right as f64, y as f64, "|", st);
                }
            }
        }

        // Center implosion hint
        if buildup > 0.5 {
            let radius = ((1.0 - buildup) * core::fmin(w as f64, h as f64) * 0.3) as i64;
            for i in 0..60 {
                let angle = i as f64 * TAU / 60.0 + t * 5.0;
                let x = cx + angle.cos() * radius as f64;
                let y = cy + angle.sin() * radius as f64 * 0.5;
                c.put(
                    x,
                    y,
                    if buildup > 0.8 { "*" } else { "+" },
                    if buildup > 0.9 { WHITE } else { BRIGHT },
                );
            }
        }

        // Glitch bars intensifying
        for i in 0..(buildup * 8.0) as i64 {
            let row = (hash16((t * 50.0) as i64 + i) % h.max(1) as u32) as i64;
            let shift = ((t * 20.0 + i as f64).sin() * buildup * 15.0) as i64;
            if shift != 0 {
                c.rotate_row(row, shift);
            }
        }

        return; // Don't run the rest of title_takeover yet
    }

    // ORIGINAL TITLE TAKEOVER CODE (16s onwards)
    let mut density = if elapsed < 5.0 {
        0.78
    } else {
        core::mix(0.60, 0.10, lock)
    };
    if dissolve > 0.0 {
        density = core::mix(0.1, 0.48, dissolve);
    }
    for yy in 0..h {
        let band = (yy as f64 * 0.18 - elapsed * 2.6).sin();
        let _ = band;
        let row_shift = ((elapsed * 4.0 + yy as f64 * 0.31).sin() * clamp(elapsed / 2.0) * 9.0) as i64;
        for xx in 0..w {
            let k = hash16(xx * 37 + yy * 911 + (elapsed * 7.0) as i64 * (3 + xx % 7));
            if k as f64 / 65535.0 > density {
                continue;
            }
            // Python `xx//7`: floor division. For non-negative `xx` this equals
            // truncation, but `div_euclid` states the intent and stays correct
            // if the range ever changes.
            let ch = alphabet[((k as i64 + frame + xx.div_euclid(7) * 13)
                .rem_euclid(alphabet.len() as i64)) as usize];
            let mut style = if (yy + frame.div_euclid(2)).rem_euclid(h.max(1)) < 2 {
                NORMAL
            } else if k % 17 == 0 {
                DIM
            } else {
                GREEN
            };
            if lock > 0.5 {
                style = if k % 5 == 0 { GREEN } else { DARK };
            }
            c.put(((xx + row_shift).rem_euclid(w.max(1))) as f64, yy as f64, &ch.to_string(), style);
        }
    }
    if let Some(source) = source {
        let e = clamp(elapsed / 2.1);
        for yy in 0..source.nrows() {
            for xx in 0..source.ncols() {
                let cell = source.get(xx, yy);
                let Some(mut ch) = cell.ch else { continue };
                if !ch.is_whitespace() {
                    let k = hash16(xx + yy * w);
                    let angle = ((yy as f64 - cy) * 2.0).atan2(xx as f64 - cx)
                        + e * (1.0 + (k % 7) as f64 * 0.13);
                    let radius = (xx as f64 - cx).hypot((yy as f64 - cy) * 2.0) * (1.0 + e * 0.9);
                    let px = cx + angle.cos() * radius + (yy as f64 * 0.45 + elapsed * 9.0).sin() * e * 6.0;
                    let py = cy + angle.sin() * radius / 2.0;
                    // Python: `if e>.3 and k%5<int(e*5)` — the right-hand side is
                    // truncated to an integer, so the comparison is integer-to-integer.
                    if e > 0.3 && k % 5 < (e * 5.0) as u32 {
                        ch = alphabet[((k as i64 + frame).rem_euclid(alphabet.len() as i64)) as usize];
                    } else if ch as u32 > 127 {
                        ch = alphabet[(k as usize) % alphabet.len()];
                    }
                    c.put(
                        (pyround(px).rem_euclid(w.max(1) as f64)) as f64,
                        (pyround(py).rem_euclid(h.max(1) as f64)) as f64,
                        &ch.to_string(),
                        if e < 0.5 { NORMAL } else { DIM },
                    );
                }
            }
        }
    }
    if elapsed < 4.6 {
        for strand in 0..7i64 {
            for xx in 0..w {
                let angle = xx as f64 / w as f64 * TAU * 1.7 - elapsed * 2.0 + strand as f64 * 0.39;
                let mut yy = cy + angle.sin() * h as f64 * 0.37;
                if strand % 2 != 0 {
                    yy += (xx as f64 * 0.19 + elapsed * 3.0).sin() * 2.0;
                }
                for trail in 0..3 {
                    let ch = if trail == 0 {
                        alphabet[((xx + frame + strand).rem_euclid(alphabet.len() as i64)) as usize]
                    } else {
                        '.'
                    };
                    c.put(
                        xx as f64,
                        yy + trail as f64,
                        &ch.to_string(),
                        if trail == 0 { BRIGHT } else { GREEN },
                    );
                }
            }
        }
    }
    if elapsed >= 3.1 {
        for (i, &(tx, ty)) in ink.iter().enumerate() {
            let i = i as i64;
            let k = hash16(i * 7 + 51);
            let delay = (k % 1000) as f64 / 1000.0 * 0.95;
            let u = clamp((elapsed - 3.1 - delay) / 3.0);
            let ease = 1.0 - (1.0 - u).powi(3);
            let ox = (hash16(i * 17) % w.max(1) as u32) as i64;
            let oy = (hash16(i * 29 + 10) % h.max(1) as u32) as i64;
            // The original skips this ink pixel before computing its position.
            if dissolve > 0.0 && (k % 100) as f64 / 100.0 < dissolve * 0.65 {
                continue;
            }
            let (x, y) = if dissolve > 0.0 {
                let angle = ((ty as f64 - cy) * 2.0).atan2(tx as f64 - cx) + dissolve * 0.75;
                let dist = (tx as f64 - cx).hypot((ty as f64 - cy) * 2.0)
                    + dissolve * (30.0 + (k % 40) as f64);
                (
                    cx + angle.cos() * dist,
                    cy + angle.sin() * dist / 2.0,
                )
            } else {
                let swirl = (u * std::f64::consts::PI).sin() * (1.0 - u);
                (
                    core::mix(ox as f64, tx as f64, ease) + (elapsed * 2.0 + i as f64 * 0.7).sin() * swirl * w as f64 * 0.24,
                    core::mix(oy as f64, ty as f64, ease) + (elapsed * 2.0 + i as f64 * 0.7).cos() * swirl * h as f64 * 0.24,
                )
            };
            let (ch, style) = if u > 0.98 && dissolve == 0.0 {
                let sweep = ((elapsed * 30.0) as i64).rem_euclid(w + 24) - 12;
                if (tx - sweep).abs() < 3 {
                    ('#', WHITE)
                } else {
                    (if k % 2 == 0 { '0' } else { '1' }, BRIGHT)
                }
            } else {
                (
                    alphabet[((k as i64 + frame).rem_euclid(alphabet.len() as i64)) as usize],
                    if k % 3 == 0 { BRIGHT } else { NORMAL },
                )
            };
            c.put(x, y, &ch.to_string(), style);
            if u < 0.98 || dissolve > 0.0 {
                c.put(x - 1.0, y, ".", GREEN);
            }
        }
    }
    if elapsed > 7.25 && elapsed < 11.35 {
        c.center(1.0, "M I L I", WHITE);
        c.center((h - 3) as f64, "world.execute(me);", WHITE);
    }
    if (0..=2).contains(&((elapsed * 12.0) as i64).rem_euclid(11)) {
        let row = (hash16(frame) % h.max(1) as u32) as i64;
        let shift = ((elapsed * 23.0).sin() * 7.0) as i64;
        if shift != 0 {
            c.rotate_row(row, shift);
        }
    }
}

/// Python `draw_scene` — route to the appropriate lyric-driven animation.
pub fn draw_scene(c: &Canvas, t: f64, top: i64, bt: i64, pulse: f64, e: Option<&crate::film::LyricLine>) {
    let area = simple_area(c, top, bt);

    // Get lyric text for context
    let lyric_time = e.map(|x| x.time).unwrap_or(t);
    let elapsed = t - lyric_time;

    // Boot sequence (0-16s)
    if t < 16.0 {
        if t < 1.74 {
            crate::scenes::lyric_power_line(c, t, area, t - 0.1);
        } else if t < 2.92 {
            // "Remember to put on" - continue showing boot complete
            crate::scenes::lyric_power_line(c, t, area, 1.6); // Frozen at end state
        } else if t < 3.873 {
            crate::scenes::lyric_protection(c, t, area, t - 2.92);
        } else if t < 5.491 {
            crate::scenes::lyric_lay_pieces(c, t, area, t - 3.873);
        } else if t < 6.38 {
            crate::scenes::lyric_lay_pieces(c, t, area, t - 3.873);
        } else if t < 7.446 {
            crate::scenes::lyric_object_creation(c, t, area, t - 6.38);
        } else if t < 10.091 {
            crate::scenes::lyric_data_parameters(c, t, area, t - 7.446);
        } else if t < 11.095 {
            crate::scenes::lyric_data_parameters(c, t, area, t - 7.446);
        } else {
            crate::scenes::lyric_simulation(c, t, area, t - 11.095);
        }
    }
    // Title takeover (handled in player.rs / film.rs)
    else if t < 29.709 {
        // pass
    }
    // Devotion section (29.7-59s) - use time-based routing to ensure no gaps
    else if t < 59.223 {
        if t < 33.412 {
            crate::scenes::lyric_points_dimension(c, t, area, t - 29.709);
        } else if t < 37.067 {
            crate::scenes::lyric_circle_circumference(c, t, area, t - 33.412);
        } else if t < 40.706 {
            crate::scenes::lyric_sine_tangent(c, t, area, t - 37.067);
        } else if t < 44.452 {
            crate::scenes::lyric_infinity_limit(c, t, area, t - 40.706);
        } else if t < 47.672 {
            crate::scenes::lyric_ac_dc(c, t, area, t - 44.452);
        } else if t < 51.363 {
            crate::scenes::lyric_dizzy(c, t, area, t - 47.672);
        } else if t < 55.083 {
            crate::scenes::lyric_time_travel(c, t, area, t - 51.363);
        } else {
            crate::scenes::lyric_unite_deeply(c, t, area, t - 55.083);
        }
    }
    // Heart/satisfaction section (59-74s)
    else if t < 74.045 {
        if t < 62.589 {
            crate::scenes::lyric_stimulation_satisfaction(c, t, area, t - 59.223);
        } else if t < 66.601 {
            crate::scenes::lyric_stimulation_satisfaction(c, t, area, t - 59.223);
        } else if t < 70.084 {
            crate::scenes::lyric_happy_execution(c, t, area, pulse);
        } else {
            crate::scenes::lyric_trapped_simulation(c, t, area, pulse);
        }
    }
    // Original eggplant, tomato and cat; keep the revised god scene.
    else if t < 85.078 {
        legacy_organic(c, t, top, bt, pulse);
    } else if t < 88.587 {
        crate::scenes::lyric_god_existence(c, t, area, t - 85.078);
    }
    // Identity switching (88-103s)
    else if t < 103.489 {
        if t < 92.015 {
            crate::scenes::lyric_identity_rewrite(c, t, area);
        } else if t < 95.465 {
            crate::scenes::lyric_daynight_clock(c, t, area);
        } else if t < 99.349 {
            crate::scenes::lyric_gender_role_switch(c, t, area, t - 95.465, "S", "M");
        } else {
            crate::scenes::lyric_dizzy(c, t, area, t - 99.349);
        }
    }
    // Vibration/completion (103-110s)
    else if t < 110.9 {
        crate::scenes::lyric_vibration_sync(c, t, area, t - 103.489);
    }
    // Isolation (110-118s)
    else if t < 118.333 {
        crate::scenes::lyric_isolation_disconnect(c, t, area, t - 110.9);
    }
    // Erase fragments (118-125s)
    else if t < 125.708 {
        crate::scenes::lyric_erase_fragments(c, t, area, t - 118.333);
    }
    // Illegal arguments (125-147s)
    else if t < 147.66 {
        crate::scenes::lyric_illegal_arguments(c, t, area, t - 125.708);
    }
    // EXECUTION section (147-177s)
    else if t < 177.246 {
        if t < 158.9 {
            crate::scenes::lyric_execution_queue(c, t, area, t - 147.66);
        } else if t < 162.632 {
            crate::scenes::lyric_multilingual_count(c, t, area);
        } else {
            crate::scenes::lyric_only_execution(c, t, area, pulse);
        }
    }
    // Love equation (177-192s)
    else if t < 192.5 {
        if t < 188.483 {
            crate::scenes::lyric_love_equation(c, t, area, t - 177.246);
        } else {
            crate::scenes::lyric_trapped_loop(c, t, area, core::fmin(t - 188.483, 4.0), pulse);
        }
    }
    // Outro (192+)
    else {
        crate::scenes::lyric_outro_wait(c, t, area, t - 192.5);
    }

    // Apply escalating glitch effect
    if t < 192.5 && !(74.045..85.078).contains(&t) && !(103.489..110.9).contains(&t) {
        apply_glitch(c, t, top, bt, None);
    }
}

/// Python `phosphor` — luminance scan line effect.
pub fn phosphor(c: &Canvas, t: f64, top: i64, bt: i64) {
    if (74.045..85.078).contains(&t) {
        legacy_phosphor(c, t, top, bt);
        return;
    }
    let intensity = glitch_intensity(t);
    // Less frequent scan in early sections, more in later
    if hash_below(hash16((t * 10.0) as i64), 100, intensity * 50.0) {
        let row = top + (t * 9.0) as i64 % core::fmax(1.0, (bt - top + 1) as f64) as i64;
        for x in 2..c.w - 2 {
            let cell = c.get(x, row);
            if let Some(ch) = cell.ch {
                if !ch.is_whitespace() && matches!(cell.style, DIM | NORMAL | GREEN) {
                    c.set(x, row, Some(ch), if cell.style == GREEN { NORMAL } else { BRIGHT });
                }
            }
        }
    }
}

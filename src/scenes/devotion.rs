//! `world.execute-me-ascii/scenes.py` lines 434–1274 — the "devotion" section.
//!
//! Ported statement by statement from the Python original. Branch order, loop
//! bounds, operation grouping and every magic number are preserved so the
//! rendered grid stays byte-identical:
//!
//! * integer division is `div_euclid` (Python's `//`),
//! * `int(f)` is `f as i64` (truncation towards zero),
//! * `hash16(x) % n` masks with the `u32` the helper returns,
//! * no coordinate is pre-rounded — `Canvas` applies Python's banker's rounding
//!   itself where the original relies on `round()`.

use crate::canvas::Canvas;
use crate::core::*;
use crate::scenes::common::*;
use crate::text::*;
use crate::TAU;
// `clamp` reaches this module through both globs above — `core::clamp(x, a, b)`
// and `common::clamp(x)` — which the compiler reports as ambiguous. The
// explicit import selects the single-argument (Python default) form that this
// whole section uses.
use crate::scenes::common::clamp;

/// If I'm a set of points - dimensional progression: 0D→1D→2D→3D
pub fn lyric_points_dimension(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let progress = clamp(elapsed / 3.5);

    // Phase 0: Points (0D) - 0-25%
    if progress < 0.25 {
        let phase = progress / 0.25;
        // Random points appearing
        let count = (phase * 120.0) as i64;
        for i in 0..count {
            let x = l + (hash16(i * 7) % (r - l).max(1) as u32) as i64;
            let y = top + (hash16(i * 13) % (bt - top).max(1) as u32) as i64;
            let brightness = 1.0 - (i as f64 / count as f64) * 0.5;
            c.put(
                x as f64,
                y as f64,
                if brightness < 0.7 {
                    "·"
                } else if brightness < 0.85 {
                    "+"
                } else {
                    "*"
                },
                if brightness > 0.9 {
                    WHITE
                } else if brightness > 0.7 {
                    BRIGHT
                } else {
                    NORMAL
                },
            );
        }
        c.center(
            (cy + ((bt - top) as f64 * 0.3) as i64) as f64,
            "0D: POINTS",
            if phase > 0.7 { WHITE } else { BRIGHT },
        );

    // Phase 1: Lines (1D) - 25-50%
    } else if progress < 0.5 {
        let phase = (progress - 0.25) / 0.25;
        // Points connecting into lines
        let num_lines = (phase * 15.0) as i64 + 5;
        for line_i in 0..num_lines {
            // Random line endpoints
            let x1 = l + (hash16(line_i * 11) % (r - l).max(1) as u32) as i64;
            let y1 = top + (hash16(line_i * 17) % (bt - top).max(1) as u32) as i64;
            let x2 = l + (hash16(line_i * 23) % (r - l).max(1) as u32) as i64;
            let y2 = top + (hash16(line_i * 29) % (bt - top).max(1) as u32) as i64;

            // Draw line with trail
            let steps = ((x2 - x1) as f64).hypot(((y2 - y1) * 2) as f64) as i64;
            let reveal = clamp(phase * 3.0 - line_i as f64 * 0.08);
            for s in 0..((steps as f64 * reveal) as i64) {
                let u = if steps > 0 { s as f64 / steps as f64 } else { 0.0 };
                let x = (x1 as f64 + (x2 - x1) as f64 * u) as i64;
                let y = (y1 as f64 + (y2 - y1) as f64 * u) as i64;
                let age = 1.0 - (u - reveal).abs() * 2.0;
                if age > 0.0 {
                    c.put(
                        x as f64,
                        y as f64,
                        if (x2 - x1).abs() > (y2 - y1).abs() * 2 {
                            "─"
                        } else if (y2 - y1).abs() > (x2 - x1).abs() {
                            "|"
                        } else {
                            "/"
                        },
                        if age > 0.8 {
                            WHITE
                        } else if age > 0.5 {
                            BRIGHT
                        } else {
                            NORMAL
                        },
                    );
                }
            }

            // Endpoints
            c.put(x1 as f64, y1 as f64, "●", WHITE);
            if reveal > 0.8 {
                c.put(x2 as f64, y2 as f64, "●", WHITE);
            }
        }

        c.center(
            (cy + ((bt - top) as f64 * 0.3) as i64) as f64,
            "1D: LINES",
            if phase > 0.7 { WHITE } else { BRIGHT },
        );

    // Phase 2: Surface (2D) - 50-75%
    } else if progress < 0.75 {
        let phase = (progress - 0.5) / 0.25;
        // Grid/mesh forming a surface
        let grid_density = (phase * 12.0) as i64 + 4;

        // Wave surface
        for gy in 0..grid_density {
            for gx in 0..grid_density {
                let u = if grid_density > 1 {
                    gx as f64 / (grid_density - 1) as f64
                } else {
                    0.5
                };
                let v = if grid_density > 1 {
                    gy as f64 / (grid_density - 1) as f64
                } else {
                    0.5
                };

                // Position on screen
                let x = l as f64 + (r - l) as f64 * u;
                let y = top as f64 + (bt - top) as f64 * v;

                // Wave displacement
                let wave = (u * TAU * 2.0 + t).sin() * (v * TAU * 2.0 - t * 0.7).cos() * phase;
                let y_displaced = y + wave * (bt - top) as f64 * 0.1;

                // Draw grid point
                let brightness = phase + wave * 0.3;
                c.put(
                    x,
                    y_displaced,
                    if brightness > 0.8 {
                        "█"
                    } else if brightness > 0.6 {
                        "▓"
                    } else if brightness > 0.4 {
                        "▒"
                    } else {
                        "░"
                    },
                    if brightness > 0.85 {
                        WHITE
                    } else if brightness > 0.6 {
                        BRIGHT
                    } else {
                        NORMAL
                    },
                );

                // Connect horizontally
                if gx < grid_density - 1 {
                    let next_u = (gx + 1) as f64 / (grid_density - 1) as f64;
                    let next_x = l as f64 + (r - l) as f64 * next_u;
                    c.line(x, y_displaced, next_x, y_displaced, '─', GREEN);
                }

                // Connect vertically
                if gy < grid_density - 1 {
                    let next_v = (gy + 1) as f64 / (grid_density - 1) as f64;
                    let next_y = top as f64 + (bt - top) as f64 * next_v;
                    let next_wave =
                        (u * TAU * 2.0 + t).sin() * (next_v * TAU * 2.0 - t * 0.7).cos() * phase;
                    let next_y_displaced = next_y + next_wave * (bt - top) as f64 * 0.1;
                    c.line(x, y_displaced, x, next_y_displaced, '|', GREEN);
                }
            }
        }

        c.center(
            (cy + ((bt - top) as f64 * 0.35) as i64) as f64,
            "2D: SURFACE",
            if phase > 0.7 { WHITE } else { BRIGHT },
        );

    // Phase 3: Volume (3D) - 75-100%
    } else {
        let phase = (progress - 0.75) / 0.25;
        // Rotating 3D cube with perspective

        // Cube vertices
        let size = 0.8;
        let mut vertices: Vec<(f64, f64, f64)> = Vec::new();
        for z in [-1.0f64, 1.0].iter() {
            for y in [-1.0f64, 1.0].iter() {
                for x in [-1.0f64, 1.0].iter() {
                    vertices.push((x * size, y * size, z * size));
                }
            }
        }

        // Rotate
        let angle_x = t * 0.5;
        let angle_y = t * 0.7;
        let angle_z = t * 0.3;

        let mut rotated: Vec<(f64, f64, f64)> = Vec::new();
        for &(x, y, z) in vertices.iter() {
            // Rotate around Y
            let (x, z) = (
                x * angle_y.cos() - z * angle_y.sin(),
                x * angle_y.sin() + z * angle_y.cos(),
            );
            // Rotate around X
            let (y, z) = (
                y * angle_x.cos() - z * angle_x.sin(),
                y * angle_x.sin() + z * angle_x.cos(),
            );
            // Rotate around Z
            let (x, y) = (
                x * angle_z.cos() - y * angle_z.sin(),
                x * angle_z.sin() + y * angle_z.cos(),
            );
            rotated.push((x, y, z));
        }

        // Project to 2D with perspective
        let mut projected: Vec<(f64, f64, f64)> = Vec::new();
        let scale = imin(r - l, bt - top) as f64 * 0.25;
        for &(x, y, z) in rotated.iter() {
            // Perspective projection
            let depth = 3.5 + z;
            let px = cx as f64 + x * scale / depth * 3.0;
            let py = cy as f64 + y * scale / depth * 1.5;
            projected.push((px, py, z));
        }

        // Draw edges
        let edges: [(usize, usize); 12] = [
            (0, 1),
            (1, 3),
            (3, 2),
            (2, 0), // Back face
            (4, 5),
            (5, 7),
            (7, 6),
            (6, 4), // Front face
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7), // Connecting edges
        ];

        for &(a, b) in edges.iter() {
            let (xa, ya, za) = projected[a];
            let (xb, yb, zb) = projected[b];
            let avg_z = (za + zb) / 2.0;
            // Hidden line removal
            let style = if avg_z < 0.0 {
                NORMAL
            } else if avg_z < 0.5 {
                BRIGHT
            } else {
                WHITE
            };
            let ch = if avg_z < 0.0 {
                ':'
            } else if avg_z < 0.5 {
                '.'
            } else {
                '='
            };
            c.line(xa, ya, xb, yb, ch, style);
        }

        // Draw vertices
        for (i, &(x, y, z)) in projected.iter().enumerate() {
            let label = if phase > 0.7 {
                format!("{i}")
            } else {
                "●".to_string()
            };
            c.put(
                x,
                y,
                &label,
                if z > 0.5 {
                    WHITE
                } else if z > 0.0 {
                    BRIGHT
                } else {
                    NORMAL
                },
            );
        }

        // Internal structure lines (showing volume)
        if phase > 0.5 {
            // Diagonals
            let diagonals: [(usize, usize); 4] = [(0, 7), (1, 6), (2, 5), (3, 4)];
            for &(a, b) in diagonals.iter() {
                let (xa, ya, _za) = projected[a];
                let (xb, yb, _zb) = projected[b];
                if (a as i64 + b as i64 + (t * 10.0) as i64).rem_euclid(3) == 0 {
                    c.line(xa, ya, xb, yb, '·', GREEN);
                }
            }
        }

        c.center(
            (cy + ((bt - top) as f64 * 0.35) as i64) as f64,
            "3D: VOLUME",
            if phase > 0.7 { WHITE } else { BRIGHT },
        );
    }

    // Top label
    let dim_label = ["0D", "1D", "2D", "3D"][imin(3, (progress * 4.0) as i64) as usize];
    c.center(
        top as f64,
        &format!("DIMENSIONAL PROGRESSION: {dim_label}"),
        WHITE,
    );
}

/// If I'm a circle - explosive full-screen circle with trailing particles and dynamic formula
pub fn lyric_circle_circumference(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let progress = clamp(elapsed / 3.5);

    // Radius grows explosively
    let max_radius = imin(r - l, bt - top) as f64 * 0.45;
    let radius = max_radius * progress * progress; // Quadratic growth for explosive feel

    // Phase 1: Circle explosion with particle trails (0-1.5s)
    if elapsed < 1.5 {
        let phase1 = elapsed / 1.5;

        // Draw main circle with varying density
        let circle_points = (120.0 + phase1 * 180.0) as i64;
        for i in 0..circle_points {
            let angle = i as f64 * TAU / circle_points as f64;
            let x = cx as f64 + angle.cos() * radius;
            let y = cy as f64 + angle.sin() * radius * 0.5;

            // Pulsing brightness
            let brightness = (elapsed * 4.0 + angle * 2.0).sin().abs();

            if brightness > 0.7 {
                c.put(x, y, "●", WHITE);
            } else if brightness > 0.4 {
                c.put(x, y, "○", BRIGHT);
            } else {
                c.put(x, y, "·", NORMAL);
            }

            // Trailing particles expanding outward
            if phase1 > 0.3 && i % 8 == 0 {
                for trail in 0..4 {
                    let trail_progress = (phase1 - 0.3) / 0.7 - trail as f64 * 0.08;
                    if trail_progress > 0.0 {
                        let trail_radius = radius * (1.0 + trail_progress * 0.5);
                        let tx = cx as f64 + angle.cos() * trail_radius;
                        let ty = cy as f64 + angle.sin() * trail_radius * 0.5;
                        if (l as f64) < tx
                            && tx < r as f64
                            && (top as f64) < ty
                            && ty < bt as f64
                        {
                            c.put(
                                tx,
                                ty,
                                if trail == 0 {
                                    "*"
                                } else if trail == 1 {
                                    "+"
                                } else {
                                    "·"
                                },
                                if trail == 0 {
                                    WHITE
                                } else if trail == 1 {
                                    BRIGHT
                                } else {
                                    NORMAL
                                },
                            );
                        }
                    }
                }
            }
        }

        // Center point
        c.put(cx as f64, cy as f64, "◉", RED);

        // Growing label
        if phase1 > 0.5 {
            c.center(
                (top + 2) as f64,
                "CIRCLE EXPANDING",
                if phase1 > 0.8 { WHITE } else { BRIGHT },
            );
        }

    // Phase 2: Rotating radii explosion (1.5-2.5s)
    } else if elapsed < 2.5 {
        let phase2 = (elapsed - 1.5) / 1.0;

        // Main circle fully formed
        for i in 0..180 {
            let angle = i as f64 * TAU / 180.0;
            let x = cx as f64 + angle.cos() * radius;
            let y = cy as f64 + angle.sin() * radius * 0.5;
            let brightness = i % 10 == 0;
            c.put(
                x,
                y,
                if brightness { "◉" } else { "o" },
                if brightness { WHITE } else { BRIGHT },
            );
        }

        // Multiple rotating radii
        let num_radii = (phase2 * 24.0) as i64 + 4;
        for i in 0..num_radii {
            let angle = i as f64 * TAU / num_radii as f64 + elapsed * 1.5;

            // Draw radius line with segments
            let segments = radius as i64 + 1;
            for seg in 0..segments {
                let r_progress = seg as f64 / segments as f64;
                let rx = cx as f64 + angle.cos() * seg as f64;
                let ry = cy as f64 + angle.sin() * seg as f64 * 0.5;

                // Gradient along radius
                if r_progress > 0.8 {
                    c.put(rx, ry, "═", WHITE);
                } else if r_progress > 0.5 {
                    c.put(rx, ry, "─", BRIGHT);
                } else {
                    c.put(rx, ry, "·", NORMAL);
                }
            }

            // Endpoint markers
            let ex = cx as f64 + angle.cos() * radius;
            let ey = cy as f64 + angle.sin() * radius * 0.5;
            c.put(ex, ey, "●", WHITE);
        }

        // Center
        c.put(cx as f64, cy as f64, "◉", RED);

        // Radii count
        c.center(
            (top + 2) as f64,
            &format!("RADII: {num_radii}"),
            if phase2 > 0.7 { WHITE } else { BRIGHT },
        );

        // Formula hint
        if phase2 > 0.5 {
            c.center(
                (cy - ((bt - top) as f64 * 0.35) as i64) as f64,
                "r",
                WHITE,
            );
        }

    // Phase 3: Formula revelation with particle burst (2.5-3.5s)
    } else {
        let phase3 = (elapsed - 2.5) / 1.0;

        // Full circle
        for i in 0..200 {
            let angle = i as f64 * TAU / 200.0;
            let x = cx as f64 + angle.cos() * radius;
            let y = cy as f64 + angle.sin() * radius * 0.5;
            let pulse = (elapsed * 3.0 + angle * 3.0).sin().abs();
            c.put(
                x,
                y,
                if pulse > 0.7 { "◉" } else { "o" },
                if pulse > 0.8 { WHITE } else { BRIGHT },
            );
        }

        // Selected radii
        for i in 0..12 {
            let angle = i as f64 * TAU / 12.0;
            for seg in 0..(radius as i64) {
                let rx = cx as f64 + angle.cos() * seg as f64;
                let ry = cy as f64 + angle.sin() * seg as f64 * 0.5;
                if seg % 3 == 0 {
                    c.put(rx, ry, "─", GREEN);
                }
            }
        }

        // Center
        c.put(cx as f64, cy as f64, "◉", RED);

        // Formula builds up character by character
        let circumference = format!("C ≈ {:.1}", 2.0 * 3.14159 * radius);
        let formulas: [(&str, f64, i64); 3] = [
            ("C = ?", 0.0, cy - ((bt - top) as f64 * 0.2) as i64),
            ("C = 2πr", 0.3, cy - ((bt - top) as f64 * 0.2) as i64),
            (&circumference, 0.6, cy),
        ];

        for &(formula, threshold, y_pos) in formulas.iter() {
            if phase3 >= threshold {
                let reveal_progress = (phase3 - threshold) * 5.0;
                let chars_shown = imin(
                    formula.chars().count() as i64,
                    (reveal_progress * formula.chars().count() as f64) as i64,
                );
                let shown: String = formula
                    .chars()
                    .take(chars_shown.max(0) as usize)
                    .collect();
                c.center(
                    y_pos as f64,
                    &shown,
                    if phase3 > threshold + 0.2 {
                        WHITE
                    } else {
                        BRIGHT
                    },
                );
            }
        }

        // Arc length markers radiating out
        if phase3 > 0.7 {
            for i in (0..180).step_by(15) {
                let angle = i as f64 * TAU / 180.0;
                for pulse_dist in 0..3 {
                    let px = cx as f64
                        + angle.cos() * (radius + 5.0 + pulse_dist as f64 * 3.0 + phase3 * 10.0);
                    let py = cy as f64
                        + angle.sin()
                            * (radius + 5.0 + pulse_dist as f64 * 3.0 + phase3 * 10.0)
                            * 0.5;
                    if (l as f64) < px
                        && px < r as f64
                        && (top as f64) < py
                        && py < bt as f64
                    {
                        c.put(
                            px,
                            py,
                            if pulse_dist == 0 { "*" } else { "·" },
                            if pulse_dist == 0 {
                                WHITE
                            } else if pulse_dist == 1 {
                                BRIGHT
                            } else {
                                NORMAL
                            },
                        );
                    }
                }
            }
        }

        // Circumference label
        if phase3 > 0.8 {
            c.center(
                (bt - 3) as f64,
                "CIRCUMFERENCE = 2πr",
                if ((elapsed * 4.0) as i64).rem_euclid(2) != 0 {
                    WHITE
                } else {
                    BRIGHT
                },
            );
        }
    }
}

/// If I'm a sine wave - sine with tangent lines demonstration
pub fn lyric_sine_tangent(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let progress = clamp(elapsed / 3.0);

    // Draw sine wave
    for x in (l + 1)..r {
        let angle = (x - l) as f64 / (r - l) as f64 * TAU * 2.5 - t * 0.5;
        let y = cy as f64 + angle.sin() * (bt - top) as f64 * 0.25;
        c.put(
            x as f64,
            y,
            if (x - l) % 2 != 0 { "~" } else { "≈" },
            if (x - l) % 3 != 0 { BRIGHT } else { NORMAL },
        );
    }

    // Draw tangent lines at specific points
    let num_tangents = (progress * 5.0) as i64 + 1;
    for i in 0..imin(num_tangents, 5) {
        // Position along the wave
        let t_point = i as f64 / 4.0;
        let wave_x = l as f64 + (r - l) as f64 * t_point;
        let angle = (wave_x - l as f64) / (r - l) as f64 * TAU * 2.5 - t * 0.5;
        let wave_y = cy as f64 + angle.sin() * (bt - top) as f64 * 0.25;

        // Draw point (`Y` is an alias of `W` in the original: both are style 3).
        c.put(wave_x, wave_y, "●", WHITE);

        // Calculate tangent slope: derivative of sin is cos
        let slope = angle.cos() * (bt - top) as f64 * 0.25 / (r - l) as f64 * TAU * 2.5;

        // Draw tangent line
        let tangent_len = imin(r - l, bt - top) as f64 * 0.15;
        for tx in -(tangent_len as i64)..(tangent_len as i64) {
            let tangent_x = wave_x + tx as f64;
            let tangent_y = wave_y + slope * tx as f64;
            if (l as f64) < tangent_x
                && tangent_x < r as f64
                && (top as f64) < tangent_y
                && tangent_y < bt as f64
            {
                c.put(
                    tangent_x,
                    tangent_y,
                    if slope.abs() < 0.3 {
                        "─"
                    } else if slope > 0.0 {
                        "/"
                    } else {
                        "\\"
                    },
                    if i == num_tangents - 1 { WHITE } else { GREEN },
                );
            }
        }
    }

    // Show derivative formula
    if progress > 0.3 {
        c.center((top + 2) as f64, "y = sin(x)", BRIGHT);
    }
    if progress > 0.6 {
        c.center((top + 4) as f64, "y' = cos(x)", WHITE);
    }
    if progress > 0.8 {
        c.center((bt - 2) as f64, "TANGENT LINES", GREEN);
    }
}

/// A luminous, data-bearing infinity ribbon is bounded by YOU.
pub fn lyric_infinity_limit(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    // NOTE: true division here (unlike the other scenes) — `cx`/`cy` are floats.
    let (cx, cy) = ((l + r) as f64 / 2.0, (top + bt) as f64 / 2.0);
    let rail = imax(9, imin(16, c.w.div_euclid(8)));
    let ml = l + rail + 2;
    let mr = r - rail - 2;
    let mw = mr - ml + 1;
    let growth = clamp(elapsed / 1.64);
    let closing = clamp((t - 42.346) / (43.507 - 42.346));
    let locked = t >= 43.507;
    let bound_l = ml + (mw as f64 * 0.075 * closing) as i64;
    let bound_r = mr - (mw as f64 * 0.075 * closing) as i64;
    let radius_x = mw as f64 * 0.47 * (0.76 + 0.24 * growth);
    let radius_y = fmax(2.0, (bt - top - 7) as f64 * 0.43);
    let counter = (8.0 + elapsed * elapsed * 39.0) as i64;
    c.center(top as f64, "INFINITE LOOP / n -> INF", WHITE);
    c.center(
        (top + 1) as f64,
        if locked {
            "[ LIMITATIONS / BOUND BY YOU ]"
        } else if closing != 0.0 {
            "YOU.LIMIT / BOUNDARY ACQUIRED"
        } else {
            "GROWTH RATE: EXPONENTIAL"
        },
        BRIGHT,
    );
    // Full-height registers convey growth, rather than a small isolated symbol.
    for &(side, x) in [(0i64, l), (1i64, r - rail + 1)].iter() {
        c.box_(x as f64, (top + 3) as f64, rail, bt - top - 4, GREEN);
        c.put(
            (x + 1) as f64,
            (top + 3) as f64,
            if side == 0 { "N -> INF" } else { "YOU.LIMIT" },
            BRIGHT,
        );
        for row in (top + 4)..(bt - 2) {
            let n = imax(0, counter + (row - top) * if side == 0 { 1 } else { -1 });
            let text = if side == 0 {
                format!("2^{n:04}")
            } else {
                format!("{:04X} {}", hash16(n * 17), if locked { "CAP" } else { "SET" })
            };
            c.put(
                (x + 1) as f64,
                row as f64,
                crop(&text, (rail - 2).max(0) as usize),
                if (row + (elapsed * 12.0) as i64).rem_euclid(7) == 0 {
                    WHITE
                } else if side == 0 {
                    NORMAL
                } else {
                    GREEN
                },
            );
        }
    }
    // A faint coordinate volume fills the area behind the interlaced ribbon.
    for yy in ((top + 3)..(bt - 2)).step_by(3) {
        for xx in (ml..(mr + 1)).step_by(5) {
            c.put(xx as f64, yy as f64, "+", GREEN);
        }
    }

    // Nested `def position(a, strand=0, scale=1)` from the original.
    let position = |a: f64, strand: f64, scale: f64| -> (f64, f64) {
        let sn = a.sin();
        let cs = a.cos();
        let denom = 1.0 + sn * sn;
        let mut x = cx + radius_x * cs / denom * scale;
        let mut y = cy + radius_y * 2.8 * sn * cs / denom * scale;
        // Cross-section twists turn the curves into a woven character ribbon.
        x += strand * (a * 3.0 + elapsed).cos() * 0.6;
        y += strand * (a * 3.0 + elapsed).sin() * 0.55;
        (fmax(bound_l as f64, fmin(bound_r as f64, x)), y)
    };

    // Dim, offset echoes exploit CRT persistence without obscuring the main shape.
    for &scale in [0.80f64, 1.10].iter() {
        for i in 0..210 {
            let a = i as f64 * TAU / 210.0;
            let (x, y) = position(a, 0.0, scale);
            if ((top + 3) as f64) < y && y < (bt - 2) as f64 {
                c.put(x, y, ".", GREEN);
            }
        }
    }

    let mut points: Vec<(f64, f64, f64, char, Style)> = Vec::new();
    for i in 0..320 {
        let a = i as f64 * TAU / 320.0;
        let z = (a + elapsed * 0.35).sin();
        for strand in -2i64..3 {
            let (x, y) = position(a, strand as f64, 1.0);
            if ((top + 3) as f64) < y && y < (bt - 2) as f64 {
                let char = if strand.abs() == 2 {
                    '#'
                } else if (i + (elapsed * 18.0) as i64).rem_euclid(2) != 0 {
                    '1'
                } else {
                    '0'
                };
                points.push((
                    z,
                    x,
                    y,
                    char,
                    if z > 0.65 && strand.abs() == 2 {
                        WHITE
                    } else if z > 0.0 {
                        BRIGHT
                    } else {
                        NORMAL
                    },
                ));
            }
        }
    }
    points.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    for &(_, x, y, char, ink) in points.iter() {
        c.put(x, y, &char.to_string(), ink);
    }

    // Bright trains orbit the complete figure-eight in both directions.
    for packet in 0..12 {
        let direction = if packet % 2 != 0 { 1i64 } else { -1i64 };
        let phase =
            direction as f64 * (elapsed * (1.6 + growth * 1.1)) + packet as f64 * TAU / 12.0;
        for tail in 0..7 {
            let (x, y) = position(phase - direction as f64 * tail as f64 * 0.025, 0.0, 1.0);
            if ((top + 3) as f64) < y && y < (bt - 2) as f64 {
                c.put(
                    x,
                    y,
                    if tail == 0 {
                        "@"
                    } else if tail < 3 {
                        "*"
                    } else {
                        "."
                    },
                    if tail < 2 {
                        WHITE
                    } else if tail < 4 {
                        BRIGHT
                    } else {
                        GREEN
                    },
                );
            }
        }
    }

    // Limit rails advance inward when "you can be my" begins.
    if closing > 0.0 {
        for &x in [bound_l, bound_r].iter() {
            c.line(
                x as f64,
                (top + 3) as f64,
                x as f64,
                (bt - 3) as f64,
                if locked { '|' } else { ':' },
                if locked { WHITE } else { BRIGHT },
            );
            for &yy in [top + 3, bt - 3].iter() {
                c.put((x - 1) as f64, yy as f64, "[+]", WHITE);
            }
        }
        c.put((bound_r - 2) as f64, cy, "YOU", WHITE);
    }
    clear(c, (cx as i64 - 3) as f64, cy as i64 as f64, 7.0, 1.0);
    c.put(cx - 2.0, cy, "[ME]", WHITE);
    let footer = if locked {
        "while (me < you.limit) { grow(); }".to_string()
    } else {
        format!("n = 2^{:04} / NO UPPER BOUND", counter)
    };
    c.center((bt - 1) as f64, &footer, BRIGHT);
    c.center(
        bt as f64,
        if locked {
            "LIMIT = YOU"
        } else {
            "DATA CIRCULATING / LOOP CONTINUES"
        },
        if locked { WHITE } else { NORMAL },
    );
}

/// Split-screen AC sine wave and a DC voltage step, like a CRT scope.
pub fn lyric_ac_dc(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let amplitude = fmax(2.0, (bt - top - 4) as f64 * 0.29);
    let left_span = imax(1, cx - l - 2);
    let phase = elapsed * 3.1;
    // A quiet, delayed trace supplies the phosphor tail behind the moving AC wave.
    for &trail in [1i64, 0].iter() {
        let mut previous: Option<(f64, f64)> = None;
        for sample in 0..(left_span * 3 + 1) {
            let xx = (l + 1) as f64 + sample as f64 / 3.0;
            let yy = cy as f64
                - ((xx - l as f64 - 1.0) / left_span as f64 * TAU * 2.0 - phase
                    + trail as f64 * 0.13)
                    .cos()
                    * amplitude;
            if let Some((px, py)) = previous {
                c.line(
                    px,
                    py,
                    xx,
                    yy,
                    if trail != 0 { ':' } else { '.' },
                    if trail != 0 { GREEN } else { NORMAL },
                );
            }
            previous = Some((xx, yy));
        }
    }
    // The new DC level sweeps across the old one, then holds steady.
    let dc_progress = clamp((t - 45.85) / 1.10);
    let edge = mix(r as f64, (cx + 1) as f64, dc_progress) as i64;
    let low = (cy as f64 + amplitude) as i64;
    let high = (cy as f64 - amplitude) as i64;
    if edge > cx + 1 {
        c.line((cx + 1) as f64, low as f64, edge as f64, low as f64, ':', NORMAL);
    }
    if edge < r {
        c.line(edge as f64, high as f64, r as f64, high as f64, ':', BRIGHT);
        if dc_progress < 1.0 {
            c.line(edge as f64, low as f64, edge as f64, high as f64, '|', GREEN);
            c.put(edge as f64, high as f64, "+", WHITE);
        }
    }
    // A full-height dividing line separates the two current modes.
    c.line(cx as f64, top as f64, cx as f64, (bt - 1) as f64, '|', BRIGHT);
    c.put(cx as f64, top as f64, "+", WHITE);
    c.put(cx as f64, (bt - 1) as f64, "+", BRIGHT);

    // The same five-row character shapes as the player's existing bitmap font.
    let sx = imax(1, imin(3, c.w.div_euclid(60)));
    let sy = imax(1, imin(3, (bt - top).div_euclid(12)));
    let label_w = 11 * sx;
    let label_h = 5 * sy;
    let label_y = cy - label_h.div_euclid(2);

    // Nested `def label(x, text, ink)` from the original.
    let label = |x: i64, text: &str, ink: Style| {
        clear(
            c,
            (x - 1) as f64,
            (label_y - 1) as f64,
            (label_w + 2) as f64,
            (label_h + 2) as f64,
        );
        for (index, ch) in text.chars().enumerate() {
            let rows: [&str; 5] = match ch {
                'A' => ["01110", "11011", "11111", "11011", "11011"],
                'C' => ["01111", "11000", "11000", "11000", "01111"],
                'D' => ["11110", "11011", "11011", "11011", "11110"],
                _ => ["00000"; 5],
            };
            for (dy, row) in rows.iter().enumerate() {
                for (dx, pixel) in row.chars().enumerate() {
                    if pixel == '1' {
                        for yy in 0..sy {
                            c.put(
                                (x + (index as i64 * 6 + dx as i64) * sx) as f64,
                                (label_y + dy as i64 * sy + yy) as f64,
                                &"#".repeat(sx as usize),
                                ink,
                            );
                        }
                    }
                }
            }
        }
    };

    // AC sits against the outer edge; DC occupies the center of the right half.
    label(l + 2, "AC", if t < 46.45 { WHITE } else { BRIGHT });
    label(
        (cx + r).div_euclid(2) - label_w.div_euclid(2),
        "DC",
        if t >= 45.85 { WHITE } else { NORMAL },
    );
    c.center(bt as f64, "to AC, to DC", BRIGHT);
}

/// Blind my vision - intense visual distortion with eyes everywhere
pub fn lyric_dizzy(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let progress = clamp(elapsed / 3.5);

    // Phase 1: Eyes opening everywhere (0-1s)
    if elapsed < 1.0 {
        let phase1 = elapsed / 1.0;
        let num_eyes = (phase1 * 30.0) as i64 + 5;

        for i in 0..num_eyes {
            // Random but stable positions
            let angle = i as f64 * TAU / 30.0 + hash16(i * 13) as f64 * 0.01;
            let radius =
                (hash16(i * 17) % imin(r - l, bt - top).div_euclid(3).max(1) as u32) as i64 + 10;
            let ex = cx as f64 + angle.cos() * radius as f64;
            let ey = cy as f64 + angle.sin() * radius as f64 * 0.5;

            // Eye blink cycle
            let blink_phase = (elapsed * 3.0 + i as f64 * 0.3).rem_euclid(1.0);
            if blink_phase < 0.7 {
                // Open
                // Draw eye
                c.put(ex - 1.0, ey, "(", NORMAL);
                c.put(ex, ey, "○", if i == num_eyes - 1 { WHITE } else { BRIGHT });
                c.put(ex + 1.0, ey, ")", NORMAL);
            } else if blink_phase < 0.85 {
                // Half closed
                c.put(ex - 1.0, ey, "(", GREEN);
                c.put(ex, ey, "-", BRIGHT);
                c.put(ex + 1.0, ey, ")", GREEN);
            }
        }

        c.center(
            (top + 2) as f64,
            "VISION",
            if phase1 > 0.7 { WHITE } else { BRIGHT },
        );

    // Phase 2: Spiral distortion with multiplying eyes (1-2s)
    } else if elapsed < 2.0 {
        let phase2 = (elapsed - 1.0) / 1.0;

        // Spiral vortex
        for ring in 0..20 {
            let ring_radius = ring as f64 * 4.0 + phase2 * 20.0;
            let points_in_ring = imax(8, ring * 2);
            for i in 0..points_in_ring {
                let angle =
                    i as f64 * TAU / points_in_ring as f64 + elapsed * 2.0 - ring as f64 * 0.3;
                let x = cx as f64 + angle.cos() * ring_radius;
                let y = cy as f64 + angle.sin() * ring_radius * 0.5;

                if (l as f64) < x && x < r as f64 && (top as f64) < y && y < bt as f64 {
                    // Eyes in spiral
                    if i % 3 == 0 {
                        c.put(
                            x,
                            y,
                            "◉",
                            if ring < 5 {
                                WHITE
                            } else if ring < 12 {
                                BRIGHT
                            } else {
                                NORMAL
                            },
                        );
                    } else {
                        c.put(x, y, "·", NORMAL);
                    }
                }
            }
        }

        // Large central eyes
        let num_center_eyes = (phase2 * 8.0) as i64 + 1;
        for i in 0..num_center_eyes {
            let eye_x = cx + ((elapsed * 4.0 + i as f64).sin() * 25.0) as i64;
            let eye_y = cy + ((elapsed * 3.0 + i as f64 * 0.7).cos() * 10.0) as i64;

            // Animated iris following rotation
            let iris_offset = ((elapsed * 5.0 + i as f64).sin() * 2.0) as i64;
            c.put((eye_x - 2) as f64, eye_y as f64, "(", BRIGHT);
            c.put(
                (eye_x - 1 + iris_offset) as f64,
                eye_y as f64,
                "●",
                if i % 3 == 0 { RED } else { WHITE },
            );
            c.put((eye_x + 2) as f64, eye_y as f64, ")", BRIGHT);
        }

        c.center(
            (cy - ((bt - top) as f64 * 0.3) as i64) as f64,
            "DIZZY",
            if ((elapsed * 6.0) as i64).rem_euclid(2) != 0 {
                WHITE
            } else {
                BRIGHT
            },
        );

    // Phase 3: Intense distortion - screen filled with eyes (2-3.5s)
    } else {
        let phase3 = (elapsed - 2.0) / 1.5;

        // Waves of distortion
        let wave_intensity = phase3 * 8.0;

        // Fill screen with eyes at varying depths
        for row in ((top + 2)..(bt - 2)).step_by(2) {
            for col in ((l + 3)..(r - 3)).step_by(8) {
                // Wave distortion
                let wave_x = ((row as f64 * 0.2 + elapsed * 3.0).sin() * wave_intensity) as i64;
                let wave_y =
                    ((col as f64 * 0.15 + elapsed * 2.5).cos() * wave_intensity * 0.5) as i64;

                let x = col + wave_x;
                let y = row + wave_y;

                if (l + 2) < x && x < (r - 2) && (top + 1) < y && y < (bt - 1) {
                    // Distance from center affects eye style
                    let (dx, dy) = (x - cx, (y - cy) * 2);
                    let dist = (dx as f64).hypot(dy as f64);

                    // Pupil direction follows wave
                    let pupil_dir = (dist * 0.1 + elapsed * 4.0).sin() as i64;

                    // Eye types based on distance
                    if dist < 20.0 {
                        // Close eyes - large and detailed
                        if hash16(row + col) % 4 == 0 {
                            c.put((x - 2) as f64, y as f64, "(", WHITE);
                            c.put((x - 1 + pupil_dir) as f64, y as f64, "●", RED);
                            c.put((x + 2) as f64, y as f64, ")", WHITE);
                        }
                    } else if dist < 50.0 {
                        // Medium eyes
                        if hash16(row * 7 + col * 11) % 3 == 0 {
                            c.put((x - 1) as f64, y as f64, "(", BRIGHT);
                            c.put(
                                (x + pupil_dir) as f64,
                                y as f64,
                                "○",
                                if ((elapsed * 8.0) as i64).rem_euclid(3) == 0 {
                                    WHITE
                                } else {
                                    BRIGHT
                                },
                            );
                            c.put((x + 1) as f64, y as f64, ")", BRIGHT);
                        }
                    } else {
                        // Distant eyes - small
                        if hash16(row * 13 + col * 17) % 5 == 0 {
                            c.put(
                                x as f64,
                                y as f64,
                                if hash16(row + col + (elapsed * 10.0) as i64) % 2 != 0 {
                                    "◉"
                                } else {
                                    "○"
                                },
                                if dist > 80.0 { NORMAL } else { GREEN },
                            );
                        }
                    }
                }
            }
        }

        // Giant central eye blinking
        if phase3 > 0.3 {
            let blink = (elapsed * 2.0).rem_euclid(1.0);
            if blink < 0.6 {
                // Open
                let eye_size = (8.0 + (elapsed * 5.0).sin() * 2.0) as i64;
                // Left eyelid
                for i in 0..eye_size {
                    c.put(
                        (cx - eye_size + i) as f64,
                        (cy - 2) as f64,
                        if i % 2 != 0 { "-" } else { "_" },
                        WHITE,
                    );
                }
                // Right eyelid top
                for i in 0..eye_size {
                    c.put(
                        (cx + i) as f64,
                        (cy - 2) as f64,
                        if i % 2 != 0 { "-" } else { "_" },
                        WHITE,
                    );
                }

                // Iris and pupil
                c.put((cx - 1) as f64, cy as f64, "(", BRIGHT);
                c.put(
                    cx as f64,
                    cy as f64,
                    "●",
                    if ((elapsed * 4.0) as i64).rem_euclid(2) != 0 {
                        RED
                    } else {
                        WHITE
                    },
                );
                c.put((cx + 1) as f64, cy as f64, ")", BRIGHT);

                // Bottom eyelid
                for i in 0..eye_size {
                    c.put(
                        (cx - eye_size + i) as f64,
                        (cy + 2) as f64,
                        if i % 2 != 0 { "_" } else { "-" },
                        WHITE,
                    );
                }
                for i in 0..eye_size {
                    c.put(
                        (cx + i) as f64,
                        (cy + 2) as f64,
                        if i % 2 != 0 { "_" } else { "-" },
                        WHITE,
                    );
                }
            } else {
                // Blinking
                for i in 0..16 {
                    c.put(
                        (cx - 8 + i) as f64,
                        cy as f64,
                        if i % 2 != 0 { "=" } else { "-" },
                        BRIGHT,
                    );
                }
            }
        }

        // Disorienting text
        let messages = ["VISION", "BLINDED", "DIZZY", "EYES", "SEEING", "BLIND"];
        if phase3 > 0.5 {
            for (i, msg) in messages.iter().enumerate() {
                let msg_x = cx + ((elapsed * 3.0 + i as f64).sin() * 40.0) as i64;
                let msg_y = cy + ((elapsed * 2.5 + i as f64 * 0.8).cos() * 15.0) as i64;
                if (top + 2) < msg_y && msg_y < (bt - 2) {
                    // Distorted text
                    let msg_distorted: String = msg
                        .chars()
                        .enumerate()
                        .map(|(j, ch)| {
                            if hash16(j as i64 * 19 + (elapsed * 10.0) as i64) % 4 > 0 {
                                ch
                            } else {
                                char::from_u32(33 + hash16(j as i64 * 23 + i as i64) % 94)
                                    .unwrap_or(ch)
                            }
                        })
                        .collect();
                    c.center(
                        msg_y as f64,
                        crop(&msg_distorted, 10),
                        if i as i64
                            == ((elapsed * 3.0) as i64).rem_euclid(messages.len() as i64)
                        {
                            WHITE
                        } else if i % 2 != 0 {
                            BRIGHT
                        } else {
                            NORMAL
                        },
                    );
                }
            }
        }

        // Final flash effect
        if phase3 > 0.9 && ((elapsed * 12.0) as i64).rem_euclid(3) == 0 {
            c.center(cy as f64, "BLIND", WHITE);
        }
    }
}

/// Travel AD to BC - timeline with sweeping beam and year markers
pub fn lyric_time_travel(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let progress = clamp(elapsed / 3.7);

    // Timeline horizontal line
    let timeline_y = cy;
    for x in (l + 5)..(r - 5) {
        c.put(
            x as f64,
            timeline_y as f64,
            "─",
            if x % 2 != 0 { BRIGHT } else { NORMAL },
        );
    }

    // Year markers along timeline
    // AD years on right, BC years on left
    let num_markers = 10;
    let marker_spacing = (r - l - 20).div_euclid(num_markers);

    for i in 0..num_markers {
        let marker_x = l + 10 + i * marker_spacing;

        // Tick marks
        for tick_y in -3i64..4 {
            if tick_y.abs() == 3 {
                c.put(
                    marker_x as f64,
                    (timeline_y + tick_y) as f64,
                    "|",
                    if i % 2 != 0 { GREEN } else { NORMAL },
                );
            } else if tick_y.abs() == 2 {
                c.put(marker_x as f64, (timeline_y + tick_y) as f64, "│", GREEN);
            }
        }

        // Year labels
        // Center is year 0, right is AD (positive), left is BC (negative)
        let year_offset = (i - num_markers.div_euclid(2)) * 500; // 500 year intervals
        if year_offset > 0 {
            let label = format!("{}AD", year_offset.abs());
            // `Y` is an alias of `W` in the original: both are style 3.
            c.center((timeline_y - 5) as f64, &label, WHITE);
            // Position near marker
            c.put(
                (marker_x - (label.chars().count() as i64).div_euclid(2)) as f64,
                (timeline_y - 5) as f64,
                &label,
                BRIGHT,
            );
        } else if year_offset < 0 {
            let label = format!("{}BC", year_offset.abs());
            // `Y` is an alias of `W` in the original: both are style 3.
            c.center((timeline_y - 5) as f64, &label, WHITE);
            c.put(
                (marker_x - (label.chars().count() as i64).div_euclid(2)) as f64,
                (timeline_y - 5) as f64,
                &label,
                BRIGHT,
            );
        } else {
            c.put((marker_x - 1) as f64, (timeline_y - 5) as f64, "0", WHITE);
        }
    }

    // Sweeping vertical beam traveling from right (AD) to left (BC)
    let beam_progress = progress;
    let beam_x = mix((r - 10) as f64, (l + 10) as f64, beam_progress) as i64;

    // Draw the beam
    for y in (top + 2)..(bt - 2) {
        // Intensity varies along beam height
        let intensity = 1.0 - (y - cy).abs() as f64 / (bt - top) as f64 * 2.0;

        if intensity > 0.7 {
            c.put(beam_x as f64, y as f64, "│", WHITE);
        } else if intensity > 0.4 {
            c.put(beam_x as f64, y as f64, "┊", BRIGHT);
        } else {
            c.put(beam_x as f64, y as f64, ":", NORMAL);
        }

        // Glow effect around beam
        for &glow_offset in [-2i64, -1, 1, 2].iter() {
            let gx = beam_x + glow_offset;
            if l < gx && gx < r && top < y && y < bt {
                if glow_offset.abs() == 1 {
                    c.put(
                        gx as f64,
                        y as f64,
                        "░",
                        if intensity > 0.5 { BRIGHT } else { NORMAL },
                    );
                } else {
                    c.put(gx as f64, y as f64, "·", NORMAL);
                }
            }
        }
    }

    // Particles flying past the beam
    if progress > 0.2 {
        for particle_i in 0i64..30 {
            // Particle position relative to beam
            let particle_phase = (elapsed * 2.0 + particle_i as f64 * 0.3).rem_euclid(1.0);

            // Horizontal position - moving left
            let px = beam_x + ((particle_phase - 0.5) * 60.0) as i64;

            // Vertical position - scattered
            let py = top + 5 + (particle_i * 7).rem_euclid(bt - top - 10);

            if l < px && px < r && top < py && py < bt {
                if particle_phase < 0.2 || particle_phase > 0.8 {
                    c.put(
                        px as f64,
                        py as f64,
                        "*",
                        if particle_phase < 0.1 { WHITE } else { BRIGHT },
                    );
                } else {
                    c.put(px as f64, py as f64, "·", NORMAL);
                }
            }
        }
    }

    // Current year display near beam
    if progress > 0.1 {
        let current_year = mix(2000.0, -2000.0, beam_progress) as i64;
        let year_label = format!(
            "{}{}",
            current_year.abs(),
            if current_year > 0 {
                "AD"
            } else if current_year < 0 {
                "BC"
            } else {
                ""
            }
        );

        // Display year near beam top
        let label_x = beam_x - (year_label.chars().count() as i64).div_euclid(2);
        if l + 5 < label_x && label_x < r - 15 {
            // `W` and `Y` are the same style in the original.
            c.put(label_x as f64, (top + 3) as f64, &year_label, WHITE);
        }
    }

    // Era labels
    if progress < 0.3 {
        c.center((top + 1) as f64, "FUTURE → PAST", BRIGHT);
    } else if progress < 0.7 {
        c.center(
            (top + 1) as f64,
            "TIME TRAVEL",
            if ((elapsed * 3.0) as i64).rem_euclid(2) != 0 {
                WHITE
            } else {
                BRIGHT
            },
        );
    } else {
        c.center((top + 1) as f64, "ANCIENT ERA", WHITE);
    }

    // Speed lines indicating motion
    if progress > 0.3 {
        for line_i in 0..15 {
            let line_y = top + 5 + line_i * (bt - top - 10).div_euclid(15);
            // Lines move from right to left
            let line_phase = (elapsed * 3.0 + line_i as f64 * 0.1).rem_euclid(1.0);
            let line_length = (line_phase * 20.0) as i64 + 5;

            for lx in imax(l + 5, beam_x + 10)..imin(r - 5, beam_x + 10 + line_length) {
                if hash16(line_i * 17 + (lx as f64 / 3.0) as i64) % 4 == 0 {
                    c.put(
                        lx as f64,
                        line_y as f64,
                        if line_phase > 0.7 { "=" } else { "-" },
                        if line_phase > 0.5 { BRIGHT } else { NORMAL },
                    );
                }
            }
        }
    }

    // Destination marker
    if progress > 0.8 {
        let dest_x = l + 15;
        c.put(dest_x as f64, (timeline_y - 2) as f64, "▼", RED);
        c.put((dest_x - 2) as f64, (timeline_y - 3) as f64, "BC", RED);

        // Arrival flash
        if progress > 0.95 {
            let flash_radius = ((progress - 0.95) * 60.0) as i64;
            for angle_i in 0..12 {
                let angle = angle_i as f64 * TAU / 12.0;
                let fx = dest_x + (angle.cos() * flash_radius as f64) as i64;
                let fy = timeline_y + (angle.sin() * flash_radius as f64 * 0.5) as i64;
                if l < fx && fx < r && top < fy && fy < bt {
                    c.put(
                        fx as f64,
                        fy as f64,
                        "*",
                        if flash_radius < 10 { WHITE } else { BRIGHT },
                    );
                }
            }
        }
    }
}

/// Unite so deeply - two forms merging
pub fn lyric_unite_deeply(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    let progress = clamp(elapsed / 1.8);
    // Two circles moving together
    let sep = mix(40.0, 0.0, progress);
    for &(side, label) in [(-1i64, "ME"), (1i64, "YOU")].iter() {
        let center_x = cx as f64 + side as f64 * sep;
        let radius = 15.0;
        for i in 0..60 {
            let angle = i as f64 * TAU / 60.0 + t * side as f64 * 0.2;
            let x = center_x + angle.cos() * radius;
            let y = cy as f64 + angle.sin() * radius * 0.5;
            c.put(
                x,
                y,
                if progress > 0.7 && sep < 5.0 {
                    "@"
                } else if progress > 0.4 {
                    "*"
                } else {
                    "."
                },
                if progress > 0.8 {
                    WHITE
                } else if progress > 0.5 {
                    BRIGHT
                } else {
                    NORMAL
                },
            );
        }
        if sep > 10.0 {
            c.put(center_x, (cy - 2) as f64, label, BRIGHT);
        }
    }
    if progress > 0.7 {
        c.center(cy as f64, "UNIFIED", WHITE);
    }
}

/// Give sensory impulses to YOU, then fill its satisfaction meter.
pub fn lyric_stimulation_satisfaction(c: &Canvas, t: f64, area: (i64, i64, i64, i64), elapsed: f64) {
    let (l, top, r, bt) = area;
    let (cx, cy) = ((l + r).div_euclid(2), (top + bt).div_euclid(2));
    if t >= 62.589 {
        // Reach maximum on the sung SATISFACTION, then hold it.
        let u = clamp((t - 62.589) / (65.397 - 62.589));
        let percent = (100.0 * u) as i64;
        let full = u >= 1.0;
        c.center(
            imax(top, cy - 7) as f64,
            &format!("SATISFACTION {:05.1}%", percent as f64),
            if full { WHITE } else { BRIGHT },
        );
        let bar_x = l + 3;
        let bar_w = r - l - 6;
        let inside = bar_w - 2;
        for lane in 0..3 {
            let y = cy - 5 + lane * 2;
            let amount = clamp(u - (1.0 - u) * lane as f64 * 0.13);
            let filled = (inside as f64 * amount) as i64;
            c.put(
                bar_x as f64,
                y as f64,
                &format!("[{}]", "-".repeat(inside.max(0) as usize)),
                NORMAL,
            );
            for col in 0..filled {
                let scan = (col - (t * 26.0) as i64 - lane * 7).rem_euclid(imax(1, inside));
                c.put(
                    (bar_x + 1 + col) as f64,
                    y as f64,
                    if lane == 1 { "#" } else { "=" },
                    if scan < 5 || full { WHITE } else { BRIGHT },
                );
            }
            if filled < inside {
                c.put((bar_x + 1 + filled) as f64, y as f64, ">", WHITE);
            }
        }
        c.big(
            (cy + 1) as f64,
            &percent.to_string(),
            if full { WHITE } else { BRIGHT },
        );
        c.center(
            imin(bt, cy + 7) as f64,
            if full {
                "[ MAXIMUM ]"
            } else {
                "[ FILLING SATISFACTION ]"
            },
            if full { WHITE } else { NORMAL },
        );
        return;
    }

    // Parallel sensory paths: the giver drives each receiver with more pulses.
    let strength = clamp(elapsed / (61.958 - 59.223));
    let accent = t >= 61.958;
    c.center(
        top as f64,
        "STIMULATION / SENSORY INPUT",
        if accent { WHITE } else { BRIGHT },
    );
    let left_x = l + 1;
    let right_x = r - 10;
    for &(x, label) in [(left_x, "ME"), (right_x, "YOU")].iter() {
        c.box_(x as f64, (cy - 2) as f64, 10, 5, BRIGHT);
        c.put((x + 3) as f64, cy as f64, label, WHITE);
    }
    let wire_l = left_x + 12;
    let wire_r = right_x - 3;
    let span = wire_r - wire_l;
    let lanes = if bt - top >= 22 { 5 } else { 3 };
    let gap = imax(2, imin(4, (bt - top - 6).div_euclid(imax(1, lanes - 1))));
    let names = ["TOUCH", "SOUND", "LIGHT", "REWARD", "FEEDBACK"];
    for lane in 0..lanes {
        let y = cy + (lane - lanes.div_euclid(2)) * gap;
        c.line(
            (left_x + 9) as f64,
            cy as f64,
            wire_l as f64,
            cy as f64,
            '-',
            GREEN,
        );
        c.line(wire_l as f64, cy as f64, wire_l as f64, y as f64, '|', GREEN);
        c.line(wire_l as f64, y as f64, wire_r as f64, y as f64, '-', NORMAL);
        c.line(wire_r as f64, y as f64, wire_r as f64, cy as f64, '|', GREEN);
        c.line(
            wire_r as f64,
            cy as f64,
            right_x as f64,
            cy as f64,
            '-',
            GREEN,
        );
        c.put(
            (wire_l + 2) as f64,
            (y - 1) as f64,
            names[lane as usize],
            NORMAL,
        );
        for &relay in [1i64, 2].iter() {
            let rx = wire_l + (span * relay).div_euclid(3);
            c.put(rx as f64, y as f64, "o", BRIGHT);
        }
        let speed = 0.65 + strength * 0.75;
        for packet in 0..3 {
            let phase =
                (elapsed * speed - lane as f64 * 0.17 - packet as f64 / 3.0).rem_euclid(1.0);
            let head = wire_l + (phase * span as f64) as i64;
            for trail in 0..5 {
                let xx = head - trail;
                if xx > wire_l {
                    c.put(
                        xx as f64,
                        y as f64,
                        if trail == 0 {
                            "*"
                        } else if trail < 3 {
                            "="
                        } else {
                            "."
                        },
                        if trail == 0 {
                            WHITE
                        } else if trail < 3 {
                            BRIGHT
                        } else {
                            GREEN
                        },
                    );
                }
            }
            if phase > 0.88 {
                c.put(wire_r as f64, y as f64, "#", WHITE);
                c.put((right_x + 1) as f64, (cy + 1) as f64, "ACTIVE", WHITE);
            }
        }
        if accent {
            for xx in (wire_l + 1)..wire_r {
                if (xx + (t * 30.0) as i64 + lane).rem_euclid(7) < 2 {
                    c.put(xx as f64, y as f64, "#", WHITE);
                }
            }
        }
    }
    c.center(
        bt as f64,
        &format!(
            "INPUT {:03}%  /  ALL CHANNELS {}",
            (strength * 100.0) as i64,
            if accent { "ACTIVE" } else { "CONNECTING" }
        ),
        BRIGHT,
    );
}

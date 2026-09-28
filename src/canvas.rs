//! The character grid and its ANSI serializer — port of `player.py`'s `Canvas`.
//!
//! `Cell::ch` is `Option<char>`: `None` is the original's `''`, the placeholder
//! written into the trailing column of a double-width glyph. Those cells are
//! skipped when serializing but still occupy a grid slot.

use std::cell::{Cell as StdCell, RefCell};

use crate::core::{pyround, Style, DIM, STYLES};
use crate::font;
use crate::text::{self, cw};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: Option<char>,
    pub style: Style,
}

impl Cell {
    pub const BLANK: Cell = Cell {
        ch: Some(' '),
        style: DIM,
    };

    /// Python `ch.strip()` — whitespace-only (and placeholder) cells are blank.
    #[inline]
    pub fn is_blank(&self) -> bool {
        match self.ch {
            None => true,
            Some(c) => c.is_whitespace(),
        }
    }

    /// The `ch` as a `char`, mapping the placeholder to `'\0'`-like emptiness.
    #[inline]
    pub fn c(&self) -> char {
        self.ch.unwrap_or(' ')
    }
}

pub struct Canvas {
    pub w: i64,
    pub h: i64,
    /// Active clip region. Interior mutability, like `cells`, so scene code can
    /// hold `&Canvas` exactly as the Python holds a bare `c`.
    clip: StdCell<Option<(i64, i64)>>,
    pub cells: RefCell<Vec<Vec<Cell>>>,
}

impl Canvas {
    pub fn new(w: i64, h: i64) -> Self {
        let w = w.max(0);
        let h = h.max(0);
        Canvas {
            w,
            h,
            clip: StdCell::new(None),
            cells: RefCell::new(vec![vec![Cell::BLANK; w as usize]; h as usize]),
        }
    }

    #[inline]
    pub fn clip(&self) -> Option<(i64, i64)> {
        self.clip.get()
    }

    #[inline]
    pub fn set_clip(&self, clip: Option<(i64, i64)>) {
        self.clip.set(clip);
    }

    /// Python `Canvas.put`.
    ///
    /// Coordinate truncation is `int()` (towards zero); the `x`/`y` guard
    /// against negative columns before comparing against `self.w`.
    pub fn put(&self, x: f64, y: f64, s: &str, style: Style) {
        let mut x = x as i64;
        let y = y as i64;
        if y < 0 || y >= self.h {
            return;
        }
        if let Some((a, b)) = self.clip.get() {
            if y < a || y > b {
                return;
            }
        }
        let mut cells = self.cells.borrow_mut();
        let row = &mut cells[y as usize];
        for ch in s.chars() {
            let k = cw(ch) as i64;
            if k == 0 {
                continue;
            }
            if x >= 0 && x + k <= self.w {
                row[x as usize] = Cell {
                    ch: Some(ch),
                    style,
                };
                if k == 2 {
                    row[(x + 1) as usize] = Cell {
                        ch: None,
                        style,
                    };
                }
            }
            x += k;
        }
    }

    /// Convenience for the common `c.put(x, y, &string, style)` call.
    pub fn puts(&self, x: f64, y: f64, s: String, style: Style) {
        self.put(x, y, &s, style);
    }

    /// Python `Canvas.center`.
    pub fn center(&self, y: f64, s: &str, style: Style) {
        let x = ((self.w - text::width(s) as i64) / 2) as f64;
        self.put(x, y, s, style);
    }

    /// Python `Canvas.line`.
    pub fn line(&self, x0: f64, y0: f64, x1: f64, y1: f64, ch: char, style: Style) {
        let steps = crate::core::line_steps(x1 - x0, y1 - y0);
        for i in 0..=steps {
            let u = i as f64 / steps as f64;
            let x = pyround(x0 + (x1 - x0) * u);
            let y = pyround(y0 + (y1 - y0) * u);
            self.put(x, y, &ch.to_string(), style);
        }
    }

    /// Python `Canvas.box`.
    pub fn box_(&self, x: f64, y: f64, w: i64, h: i64, style: Style) {
        if w < 2 || h < 2 {
            return;
        }
        let bar = format!("+{}+", "-".repeat((w - 2) as usize));
        self.put(x, y, &bar, style);
        self.put(x, y + (h - 1) as f64, &bar, style);
        for yy in (y as i64 + 1)..(y as i64 + h - 1) {
            self.put(x, yy as f64, "|", style);
            self.put(x + (w - 1) as f64, yy as f64, "|", style);
        }
    }

    /// Python `Canvas.big` — the 5-row bitmap headline.
    pub fn big(&self, y: f64, text: &str, style: Style) {
        let text = text.to_uppercase();
        let total = text.chars().count() as i64 * font::ADVANCE as i64 - 1;
        if total > self.w - 6 {
            self.center(y + 2.0, &text, style);
            return;
        }
        let left = (self.w - total) / 2;
        for (i, ch) in text.chars().enumerate() {
            let g = font::glyph(ch);
            for (dy, row) in g.iter().enumerate() {
                for (dx, p) in row.chars().enumerate() {
                    if p == '1' {
                        self.put(
                            (left + i as i64 * font::ADVANCE as i64 + dx as i64) as f64,
                            y + dy as f64,
                            "#",
                            style,
                        );
                    }
                }
            }
        }
    }

    /// Python `Canvas.ansi` — cursor-home, then one `\x1b[{y+1};1H` per row,
    /// emitting an SGR sequence only when the style changes.
    pub fn ansi(&self) -> String {
        let mut out = String::with_capacity((self.w * self.h * 2) as usize + 64);
        out.push_str("\x1b[H");
        let mut last: Option<Style> = None;
        for (y, row) in self.cells.borrow().iter().enumerate() {
            out.push_str(&format!("\x1b[{};1H", y + 1));
            for cell in row {
                let Some(ch) = cell.ch else { continue };
                if last != Some(cell.style) {
                    out.push_str(STYLES[cell.style as usize]);
                    last = Some(cell.style);
                }
                out.push(ch);
            }
        }
        out.push_str("\x1b[0m");
        out
    }

    /// Python `Canvas.plain` — one line per row, placeholders skipped.
    pub fn plain(&self) -> String {
        let cells = self.cells.borrow();
        let mut rows: Vec<String> = Vec::with_capacity(cells.len());
        for row in cells.iter() {
            let mut s = String::with_capacity(row.len());
            for cell in row {
                if let Some(ch) = cell.ch {
                    s.push(ch);
                }
            }
            rows.push(s);
        }
        rows.join("\n")
    }

    /// Python `c.cells[row] = cells[-shift:] + cells[:-shift]`.
    ///
    /// Python's negative slicing wraps, so this is a rotation in both
    /// directions for `|shift| < len`.
    pub fn rotate_row(&self, row: i64, shift: i64) {
        if shift == 0 {
            return;
        }
        let mut cells = self.cells.borrow_mut();
        if row < 0 || row as usize >= cells.len() {
            return;
        }
        let r = &mut cells[row as usize];
        let n = r.len() as i64;
        if n == 0 {
            return;
        }
        let s = shift.rem_euclid(n) as usize;
        r.rotate_right(s);
    }

    pub fn nrows(&self) -> i64 {
        self.cells.borrow().len() as i64
    }

    pub fn ncols(&self) -> i64 {
        self.w
    }

    /// Read a cell; out-of-range reads yield a blank, mirroring the guards the
    /// original applies at each read site.
    pub fn get(&self, x: i64, y: i64) -> Cell {
        let cells = self.cells.borrow();
        if y < 0 || y as usize >= cells.len() {
            return Cell::BLANK;
        }
        let row = &cells[y as usize];
        if x < 0 || x as usize >= row.len() {
            return Cell::BLANK;
        }
        row[x as usize]
    }

    pub fn set(&self, x: i64, y: i64, ch: Option<char>, style: Style) {
        let mut cells = self.cells.borrow_mut();
        if y < 0 || y as usize >= cells.len() {
            return;
        }
        let row = &mut cells[y as usize];
        if x < 0 || x as usize >= row.len() {
            return;
        }
        row[x as usize] = Cell { ch, style };
    }

    /// Apply `f` to the whole grid. The closure receives `(x, y, cell)`.
    pub fn for_each<F: FnMut(i64, i64, &mut Cell)>(&self, mut f: F) {
        let mut cells = self.cells.borrow_mut();
        for (y, row) in cells.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                f(x as i64, y as i64, cell);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{BRIGHT, NORMAL};

    #[test]
    fn put_is_column_truncating_and_width_aware() {
        let c = Canvas::new(5, 2);
        c.put(0.0, 0.0, "ab", NORMAL);
        assert_eq!(c.plain().lines().next().unwrap(), "ab   ");
        // A wide glyph claims two columns; the trailing slot is a placeholder.
        let c = Canvas::new(4, 1);
        c.put(0.0, 0.0, "我a", NORMAL);
        let rows = c.cells.borrow();
        assert_eq!(rows[0][0].ch, Some('我'));
        assert_eq!(rows[0][1].ch, None);
        assert_eq!(rows[0][2].ch, Some('a'));
    }

    #[test]
    fn put_truncates_fractional_coordinates_towards_zero() {
        let c = Canvas::new(8, 1);
        c.put(1.9, 0.0, "x", NORMAL);
        assert_eq!(c.get(1, 0).c(), 'x');
        c.put(-0.5, 0.0, "y", NORMAL);
        assert_eq!(c.get(0, 0).c(), 'y');
    }

    #[test]
    fn clip_restricts_vertical_writes_only() {
        let c = Canvas::new(4, 4);
        c.set_clip(Some((1, 2)));
        c.put(0.0, 0.0, "a", NORMAL);
        c.put(0.0, 1.0, "b", NORMAL);
        c.put(0.0, 3.0, "d", NORMAL);
        assert_eq!(c.get(0, 0).c(), ' ');
        assert_eq!(c.get(0, 1).c(), 'b');
        assert_eq!(c.get(0, 3).c(), ' ');
    }

    #[test]
    fn rotate_row_wraps_like_python_negative_slicing() {
        let c = Canvas::new(4, 1);
        for (i, ch) in "abcd".chars().enumerate() {
            c.put(i as f64, 0.0, &ch.to_string(), NORMAL);
        }
        c.rotate_row(0, 1);
        assert_eq!(c.plain(), "dabc");
        c.rotate_row(0, -1);
        assert_eq!(c.plain(), "abcd");
    }

    #[test]
    fn ansi_emits_style_runs_only() {
        let c = Canvas::new(2, 1);
        c.put(0.0, 0.0, "a", NORMAL);
        c.put(1.0, 0.0, "b", BRIGHT);
        let ansi = c.ansi();
        assert!(ansi.starts_with("\x1b[H\x1b[1;1H"));
        assert!(ansi.ends_with("\x1b[0m"));
        assert_eq!(ansi.matches(STYLES[NORMAL as usize]).count(), 1);
        assert_eq!(ansi.matches(STYLES[BRIGHT as usize]).count(), 1);
    }

    #[test]
    fn big_falls_back_to_plain_when_too_wide() {
        let c = Canvas::new(20, 8);
        // 14 chars * 6 - 1 = 83 > 20 - 6, so it degrades to a centered line.
        c.big(0.0, "EXECUTE(ME);", BRIGHT);
        assert_eq!(c.plain().lines().nth(2).unwrap().trim(), "EXECUTE(ME);");
    }
}

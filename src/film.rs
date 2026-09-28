//! `Film` — asset loading, lyric/spectrum lookup, and the top-level render
//! orchestration. Port of `player.py`'s `Film` class.

use std::cell::RefCell;
use std::collections::HashMap;

use serde::Deserialize;

use crate::canvas::Canvas;
use crate::core::{DIM, NORMAL, WHITE};
use crate::scenes;
use crate::text::{self, crop, wrap};
use crate::{core, font};

pub const SPECTRUM_BANDS: usize = 48;

#[derive(Deserialize, Debug, Clone)]
pub struct LyricLine {
    pub time: f64,
    pub en: String,
    pub zh: String,
    pub end: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub audio: String,
    pub duration: f64,
    #[serde(default)]
    pub subtitle_offset: f64,
}

/// Raw shape of `spectrum.json` before it is flattened into fixed-size rows.
///
/// `serde` has no `Deserialize` impl for `[T; 48]`, so rows arrive as `Vec` and
/// are validated into arrays below.
#[derive(Deserialize)]
struct RawSpectrum {
    fps: f64,
    frames: Vec<Vec<f64>>,
}

pub struct Spectrum {
    pub fps: f64,
    pub frames: Vec<[f64; SPECTRUM_BANDS]>,
}

/// `CHAPTERS` from `player.py`.
pub const CHAPTERS: [(f64, &str, &str); 5] = [
    (0.0, "01 / CREATION", "创建"),
    (29.709, "02 / DEVOTION", "献出自我"),
    (110.9, "03 / ISOLATION", "离开"),
    (125.708, "04 / EXECUTION", "失控"),
    (177.246, "05 / LOVE", "困于爱"),
];

pub struct Film {
    pub lyrics: Vec<LyricLine>,
    pub times: Vec<f64>,
    pub spectrum: Spectrum,
    pub config: Config,
    /// `TITLE_CACHE` — keyed by `(w, h)`.
    title_cache: RefCell<HashMap<(i64, i64), Vec<(i64, i64)>>>,
}

impl Film {
    pub fn load() -> Result<Film, String> {
        Self::from_assets(
            crate::assets::LYRICS,
            crate::assets::SPECTRUM,
            crate::assets::CONFIG,
        )
    }

    pub fn from_assets(
        lyrics_json: &str,
        spectrum_json: &str,
        config_json: &str,
    ) -> Result<Film, String> {
        let lyrics: Vec<LyricLine> =
            serde_json::from_str(lyrics_json).map_err(|e| format!("lyrics.json: {e}"))?;
        let raw: RawSpectrum =
            serde_json::from_str(spectrum_json).map_err(|e| format!("spectrum.json: {e}"))?;
        let config: Config =
            serde_json::from_str(config_json).map_err(|e| format!("config.json: {e}"))?;
        let times = lyrics.iter().map(|l| l.time).collect();
        if let Some(row) = raw.frames.iter().find(|r| r.len() != SPECTRUM_BANDS) {
            return Err(format!(
                "spectrum.json: 频谱行宽应为 {SPECTRUM_BANDS}，实际为 {}",
                row.len()
            ));
        }
        let frames = raw
            .frames
            .into_iter()
            .map(|row| {
                let mut arr = [0.0f64; SPECTRUM_BANDS];
                arr.copy_from_slice(&row);
                arr
            })
            .collect();
        Ok(Film {
            lyrics,
            times,
            spectrum: Spectrum {
                fps: raw.fps,
                frames,
            },
            config,
            title_cache: RefCell::new(HashMap::new()),
        })
    }

    /// Port of the cached `title_pixels`. The cache is keyed exactly as in the
    /// original so repeated calls at one size stay cheap.
    pub fn title_pixels(&self, w: i64, h: i64) -> Vec<(i64, i64)> {
        if let Some(hit) = self.title_cache.borrow().get(&(w, h)) {
            return hit.clone();
        }
        let line_h = core::fmax(5.0, core::fmin(12.0, (h as f64 * 0.23) as i64 as f64)) as i64;
        let total = line_h * 2 + 3;
        let top = core::fmax(3.0, ((h - total) / 2 - 1) as f64) as i64;
        let mut ink = Vec::new();
        let lines: [(&str, i64, i64); 2] = [
            ("WORLD.", top, (w as f64 * 0.86) as i64),
            ("EXECUTE(ME);", top + line_h + 3, w - 8),
        ];
        for (text, y0, span) in lines {
            let mut bitmap: Vec<String> = Vec::with_capacity(5);
            for row in 0..5 {
                let mut s = String::new();
                for (i, ch) in text.chars().enumerate() {
                    if i > 0 {
                        s.push('0');
                    }
                    s.push_str(font::glyph(ch)[row]);
                }
                bitmap.push(s);
            }
            let left = (w - span) / 2;
            for yy in 0..line_h {
                let bm_row = core::fmin(4.0, (yy * 5 / line_h) as f64) as i64 as usize;
                let row = &bitmap[bm_row];
                for xx in 0..span {
                    let idx = core::fmin(
                        (row.len() - 1) as f64,
                        (xx * row.len() as i64 / span) as f64,
                    ) as i64 as usize;
                    if row.as_bytes().get(idx) == Some(&b'1') {
                        ink.push((left + xx, y0 + yy));
                    }
                }
            }
        }
        self.title_cache.borrow_mut().insert((w, h), ink.clone());
        ink
    }

    /// Python `bisect.bisect_right(self.times, t) - 1`.
    fn cue_index(&self, t: f64) -> Option<usize> {
        let idx = self.times.partition_point(|&x| x <= t) as i64 - 1;
        if idx < 0 {
            None
        } else {
            Some(idx as usize)
        }
    }

    /// Python `Film.cue`.
    pub fn cue(&self, t: f64) -> Option<&LyricLine> {
        let idx = self.cue_index(t)?;
        let e = &self.lyrics[idx];
        if t < e.end {
            Some(e)
        } else {
            None
        }
    }

    /// Python `Film.energy`.
    pub fn energy(&self, t: f64) -> &[f64; SPECTRUM_BANDS] {
        let a = &self.spectrum.frames;
        let idx = core::fmin(
            (a.len() - 1) as f64,
            core::fmax(0.0, (t * self.spectrum.fps) as i64 as f64),
        ) as i64 as usize;
        &a[idx]
    }

    /// Python `Film.render`. Returns the finished canvas.
    pub fn render(
        &self,
        t: f64,
        w: i64,
        h: i64,
        paused: bool,
        offset: f64,
        help_on: bool,
        ready: bool,
    ) -> Canvas {
        let c = Canvas::new(w, h);
        if w < 64 || h < 24 {
            c.center(
                (h / 2 - 2) as f64,
                "WORLD.EXECUTE(ME);",
                crate::core::BRIGHT,
            );
            c.center((h / 2) as f64, "请放大窗口，或缩小终端字号", WHITE);
            c.center(
                (h / 2 + 2) as f64,
                &format!("{w} x {h} / minimum 64 x 24"),
                NORMAL,
            );
            c.center((h / 2 + 4) as f64, "SPACE pause  Q quit", DIM);
            return c;
        }

        if (15.8..29.709).contains(&t) && !ready {
            let source = if t < 18.1 {
                Some(self.render(15.799, w, h, paused, offset, false, false))
            } else {
                None
            };
            scenes::title_takeover(&c, t, source.as_ref(), self);
            if help_on {
                self.help(&c, offset);
            }
            return c;
        }

        let e = self.cue(t + offset);
        let tmax = core::fmax(0.0, t);
        let act = CHAPTERS
            .iter()
            .filter(|x| x.0 <= tmax)
            .next_back()
            .unwrap_or(&CHAPTERS[0]);

        c.put(2.0, 0.0, "WORLD.EXECUTE(ME);", crate::core::BRIGHT);
        let state = if ready {
            "READY"
        } else if paused {
            "PAUSED"
        } else {
            "RUNNING"
        };
        let clock = format!(
            "{:02}:{:02}.{} / 03:32  {}",
            (t as i64) / 60,
            (t as i64) % 60,
            ((t * 10.0) as i64) % 10,
            state
        );
        c.put((w - text::width(&clock) as i64 - 2) as f64, 0.0, &clock, DIM);
        c.put(2.0, 1.0, &"-".repeat((w - 4) as usize), DIM);
        c.put(2.0, 2.0, act.1, NORMAL);

        let top = 4i64;
        let bottom = h - 8;
        let spec = self.energy(t);
        let pulse: f64 = spec[..10].iter().sum::<f64>() / 10.0;

        c.set_clip(Some((top, bottom)));
        scenes::draw_scene(&c, t, top, bottom, pulse, e);
        scenes::phosphor(&c, t, top, bottom);
        c.set_clip(None);

        // Spectrum is measured from the supplied song, sampled on the audio clock.
        let sy = h - 6;
        let cols = core::fmin(80.0, (w - 8) as f64) as i64;
        let start = (w - cols) / 2;
        for i in 0..cols {
            let amp = spec[(i as f64 * 48.0 / cols as f64) as i64 as usize];
            c.put(
                (start + i) as f64,
                sy as f64,
                &RAMP[core::fmin(4.0, core::pyround(amp * 4.0)) as usize].to_string(),
                DIM,
            );
        }

        if ready {
            c.center((h - 5) as f64, "MILI  /  world.execute(me);", WHITE);
            c.center(
                (h - 3) as f64,
                "[ SPACE / ENTER TO START ]",
                crate::core::BRIGHT,
            );
        } else if let Some(e) = e {
            let ens = wrap(&e.en, (w - 8) as usize);
            let zhs = wrap(&e.zh, (w - 8) as usize);
            // Two reserved lines per language prevent changes in caption position.
            for (i, line) in ens.iter().take(2).enumerate() {
                c.center((h - 5 + i as i64) as f64, line, WHITE);
            }
            for (i, line) in zhs.iter().take(2).enumerate() {
                c.center((h - 3 + i as i64) as f64, line, crate::core::BRIGHT);
            }
        } else if t > 208.0 {
            c.center((h - 5) as f64, "PROCESS ENDED. THE LOOP REMAINS.", WHITE);
        } else {
            c.center((h - 5) as f64, "[ instrumental ]", DIM);
            c.center((h - 3) as f64, "[ 间奏 ]", DIM);
        }

        let hint = "SPACE play/pause   <- -> 5s   R restart   Q quit   H help";
        c.center((h - 1) as f64, crop(hint, (w - 4) as usize), DIM);
        if ready {
            self.slate(&c, top, bottom);
        }
        if help_on {
            self.help(&c, offset);
        }
        c
    }

    /// Port of `Film.slate`.
    pub fn slate(&self, c: &Canvas, top: i64, bottom: i64) {
        for y in top..=bottom {
            c.put(0.0, y as f64, &" ".repeat(c.w.max(0) as usize), DIM);
        }
        let cy = (top + bottom) / 2;
        c.center((top + 1) as f64, "A TERMINAL MUSIC VIDEO", DIM);
        c.big(
            core::fmax((top + 2) as f64, (cy - 4) as f64),
            "EXECUTE(ME);",
            crate::core::BRIGHT,
        );
        c.center((cy + 3) as f64, "M I L I", WHITE);
        c.center(
            core::fmin(bottom as f64, (cy + 6) as f64),
            "[ SPACE / ENTER TO START ]",
            crate::core::BRIGHT,
        );
    }

    /// Port of `Film.help`.
    pub fn help(&self, c: &Canvas, offset: f64) {
        let lines = [
            "CONTROLS / 操作".to_string(),
            "SPACE / ENTER   播放或暂停".to_string(),
            "LEFT / RIGHT    后退或前进 5 秒".to_string(),
            "R               从头播放".to_string(),
            "1 2 3 4 5       跳转五个章节".to_string(),
            "[ / ]           字幕提前 / 延后 0.1 秒".to_string(),
            ", / .           上一句 / 下一句".to_string(),
            "+ / -           音量".to_string(),
            "Q / ESC         退出".to_string(),
            "H               关闭帮助".to_string(),
            format!("字幕偏移 {offset:+.1}s"),
        ];
        let w = core::fmin((c.w - 4) as f64, 58.0) as i64;
        let x = (c.w - w) / 2;
        let y = (c.h - lines.len() as i64 - 3) / 2;
        for yy in y..y + lines.len() as i64 + 3 {
            c.put(x as f64, yy as f64, &" ".repeat(w.max(0) as usize), NORMAL);
        }
        c.box_(
            x as f64,
            y as f64,
            w,
            lines.len() as i64 + 3,
            crate::core::BRIGHT,
        );
        for (i, s) in lines.iter().enumerate() {
            c.put(
                (x + 3) as f64,
                (y + 2 + i as i64) as f64,
                s,
                if i == 0 { WHITE } else { NORMAL },
            );
        }
    }
}

/// `'._:=|'` — the spectrum ramp, indexed by `round(amp * 4)`.
const RAMP: [char; 5] = ['.', '_', ':', '=', '|'];

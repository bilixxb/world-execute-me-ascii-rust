//! Interactive playback: CLI, terminal lifecycle, input, and the frame loop —
//! port of `player.py`'s `run()` / `main()`.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::assets;
use crate::audio::{AudioEngine, DEFAULT_VOLUME};
use crate::canvas::Canvas;
use crate::film::CHAPTERS;
use crate::Film;

#[derive(Debug, Clone)]
pub struct Args {
    pub audio: Option<PathBuf>,
    pub start: f64,
    pub autoplay: bool,
    pub fps: u32,
    pub paused: bool,
    pub offset: Option<f64>,
    pub snapshot: Option<f64>,
    pub width: i64,
    pub height: i64,
    pub plain: bool,
    pub report: Option<PathBuf>,
    pub stop_after: Option<f64>,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            audio: None,
            start: 0.0,
            autoplay: false,
            fps: 24,
            paused: false,
            offset: None,
            snapshot: None,
            width: 120,
            height: 40,
            plain: false,
            report: None,
            stop_after: None,
        }
    }
}

pub const USAGE: &str = "\
world.execute(me); / bilingual terminal MV

USAGE:
    world-execute-me-rust [OPTIONS]

OPTIONS:
    --audio <FILE>     Audio file to play (default: the bundled song)
    --start <SECONDS>  Start position [default: 0]
    --autoplay         Begin playing immediately
    --paused           Start paused at --start without the ready slate
    --fps <N>          Target frame rate, 5-60 [default: 24]
    --offset <SECONDS> Subtitle offset
    --snapshot <T>     Render a single frame at time T and exit
    --width <COLS>     Snapshot width [default: 120]
    --height <ROWS>    Snapshot height [default: 40]
    --plain            With --snapshot, print text without ANSI escapes
    --report <FILE>    Write a JSON frame-timing report on exit
    --stop-after <T>   Exit once playback reaches time T
    -h, --help         Show this help
";

pub fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut a = Args::default();
    let mut it = argv.iter().skip(1);
    while let Some(arg) = it.next() {
        let mut value = |name: &str| -> Result<String, String> {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{name} requires a value"))
        };
        match arg.as_str() {
            "--audio" => a.audio = Some(PathBuf::from(value("--audio")?)),
            "--start" => {
                a.start = value("--start")?
                    .parse()
                    .map_err(|_| "--start requires a number".to_string())?
            }
            "--autoplay" => a.autoplay = true,
            "--paused" => a.paused = true,
            "--fps" => {
                a.fps = value("--fps")?
                    .parse()
                    .map_err(|_| "--fps requires an integer".to_string())?
            }
            "--offset" => {
                a.offset = Some(
                    value("--offset")?
                        .parse()
                        .map_err(|_| "--offset requires a number".to_string())?,
                )
            }
            "--snapshot" => {
                a.snapshot = Some(
                    value("--snapshot")?
                        .parse()
                        .map_err(|_| "--snapshot requires a number".to_string())?,
                )
            }
            "--width" => {
                a.width = value("--width")?
                    .parse()
                    .map_err(|_| "--width requires an integer".to_string())?
            }
            "--height" => {
                a.height = value("--height")?
                    .parse()
                    .map_err(|_| "--height requires an integer".to_string())?
            }
            "--plain" => a.plain = true,
            "--report" => a.report = Some(PathBuf::from(value("--report")?)),
            "--stop-after" => {
                a.stop_after = Some(
                    value("--stop-after")?
                        .parse()
                        .map_err(|_| "--stop-after requires a number".to_string())?,
                )
            }
            "-h" | "--help" => return Err(USAGE.to_string()),
            other => return Err(format!("未知参数：{other}\n\n{USAGE}")),
        }
    }
    if !(5..=60).contains(&a.fps) {
        return Err("--fps must be between 5 and 60".into());
    }
    Ok(a)
}

/// Keys the player reacts to, independent of how they arrive from the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// `q`, `Q`, `ESC`, `Ctrl-C`
    Quit,
    /// `space`, `Enter`
    Toggle,
    Restart,
    Chapter(u8),
    OffsetEarlier,
    OffsetLater,
    PrevLyric,
    NextLyric,
    Help,
    VolumeUp,
    VolumeDown,
    /// Left/right arrows — handled separately because they are seek commands.
    SeekBack,
    SeekForward,
}

pub struct RunOutput {
    pub frames: u64,
    pub last_time: f64,
    pub max_frame_render_seconds: f64,
    pub samples: Vec<Sample>,
}

#[derive(Debug, Clone)]
pub struct Sample {
    pub audio_time: f64,
    pub playing: bool,
    pub width: i64,
    pub height: i64,
    pub frames: u64,
}

/// Map a crossterm key event onto the original's byte-oriented key handling.
///
/// `player.py` reads raw bytes: `q`, `Q`, `\x1b`, `\x03`, `' '`, `\r`, `\n`,
/// `r`, `R`, `12345`, `[`, `]`, `,`, `.`, `h`, `H`, `+=`, `-`, and the
/// `\x1b[C` / `\x1b[D` arrow sequences.
pub fn map_key(k: &KeyEvent) -> Option<Key> {
    // Windows terminals deliver both press and release events; only act on
    // press, or every key would register twice.
    if k.kind != KeyEventKind::Press {
        return None;
    }
    if k.modifiers.contains(KeyModifiers::CONTROL) {
        return match k.code {
            KeyCode::Char('c') | KeyCode::Char('C') => Some(Key::Quit),
            _ => None,
        };
    }
    match k.code {
        KeyCode::Esc => Some(Key::Quit),
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(Key::Quit),
        KeyCode::Char(' ') | KeyCode::Enter => Some(Key::Toggle),
        KeyCode::Char('r') | KeyCode::Char('R') => Some(Key::Restart),
        KeyCode::Char(c @ '1'..='5') => Some(Key::Chapter(c as u8 - b'0')),
        KeyCode::Char('[') => Some(Key::OffsetEarlier),
        KeyCode::Char(']') => Some(Key::OffsetLater),
        KeyCode::Char(',') => Some(Key::PrevLyric),
        KeyCode::Char('.') => Some(Key::NextLyric),
        KeyCode::Char('h') | KeyCode::Char('H') => Some(Key::Help),
        KeyCode::Char('+') | KeyCode::Char('=') => Some(Key::VolumeUp),
        KeyCode::Char('-') => Some(Key::VolumeDown),
        KeyCode::Left => Some(Key::SeekBack),
        KeyCode::Right => Some(Key::SeekForward),
        _ => None,
    }
}

/// Restores the terminal on every exit path, including panics.
struct TerminalGuard {
    active: bool,
}

impl TerminalGuard {
    fn new() -> Result<Self, String> {
        crossterm::terminal::enable_raw_mode().map_err(|e| format!("无法进入终端原始模式：{e}"))?;
        Ok(TerminalGuard { active: true })
    }

    /// Alternate screen, hidden cursor, autowrap off (the original's
    /// `\x1b[?1049h\x1b[?25l\x1b[?7l\x1b[2J`).
    fn enter(&self, out: &mut impl Write) -> std::io::Result<()> {
        out.write_all(b"\x1b[?1049h\x1b[?25l\x1b[?7l\x1b[2J")?;
        out.flush()
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        let mut out = std::io::stdout();
        let _ = crossterm::execute!(out, crossterm::cursor::Show);
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = out.write_all(b"\x1b[0m\x1b[?7h\x1b[?25h\x1b[?1049l");
        let _ = out.flush();
    }
}

/// Resolve the audio source. `--audio` overrides the bundled song.
fn load_audio(args: &Args) -> Result<&'static [u8], String> {
    match &args.audio {
        None => Ok(assets::SONG),
        Some(path) => {
            let path: &Path = path;
            let bytes = std::fs::read(path).map_err(|e| {
                format!(
                    "找不到音频：{}\n请使用 --audio 指定 MP3 文件。({e})",
                    path.display()
                )
            })?;
            // The audio worker is a detached thread that outlives this
            // function; leaking the buffer is the simplest way to give it a
            // `'static` slice for the process lifetime.
            Ok(Box::leak(bytes.into_boxed_slice()))
        }
    }
}

pub fn run(args: Args, film: &Film) -> Result<RunOutput, String> {
    let bytes = load_audio(&args)?;
    let offset = args.offset.unwrap_or(film.config.subtitle_offset);
    let mut offset = offset;
    let mut started = args.autoplay || args.paused;
    let mut paused = !args.autoplay;
    let mut help_on = false;
    let mut volume = DEFAULT_VOLUME;
    let mut ready = !started;
    let mut playing_seen = false;
    let mut frames: u64 = 0;
    let mut max_render = 0.0f64;
    let mut size_last: Option<(i64, i64)> = None;
    let mut samples: Vec<Sample> = Vec::new();
    let mut current = args.start;

    let guard = TerminalGuard::new()?;
    let mut out = std::io::stdout();
    guard.enter(&mut out).map_err(|e| e.to_string())?;

    let mut audio = AudioEngine::start(bytes, args.start, args.autoplay, volume)?;
    let duration = audio.duration();

    let frame_dt = 1.0 / args.fps as f64;
    let mut next_frame = Instant::now();

    let result = (|| -> Result<(), String> {
        loop {
            let begin = Instant::now();
            let state = (audio.time(), audio.playing());
            current = state.0;
            if state.1 {
                playing_seen = true;
            }
            if playing_seen && !state.1 && !paused && current < 0.05 {
                current = duration;
                paused = true;
                audio.seek(duration - 0.02);
            }
            if current >= duration - 0.05 && !state.1 && started {
                paused = true;
            }

            // Read the real terminal size. Shell COLUMNS/LINES can be stale
            // after entering fullscreen.
            let (mut w, mut h) = match crossterm::terminal::size() {
                Ok((w, h)) => (w as i64, h as i64),
                Err(_) => (100, 36),
            };
            w = w.min(240);
            h = h.min(85);
            // Leave the last column unused. This avoids terminal autowrap
            // artifacts.
            let c = film.render(current, w - 1, h, paused, offset, help_on, ready);
            if size_last != Some((w, h)) {
                out.write_all(b"\x1b[2J").map_err(|e| e.to_string())?;
                size_last = Some((w, h));
            }
            out.write_all(c.ansi().as_bytes())
                .map_err(|e| e.to_string())?;
            out.flush().map_err(|e| e.to_string())?;
            frames += 1;
            max_render = max_render.max(begin.elapsed().as_secs_f64());
            if frames % 30 == 0 {
                samples.push(Sample {
                    audio_time: current,
                    playing: state.1,
                    width: w,
                    height: h,
                    frames,
                });
            }
            if let Some(stop) = args.stop_after {
                if current >= stop {
                    break;
                }
            }

            next_frame += Duration::from_secs_f64(frame_dt);
            let delay = next_frame.saturating_duration_since(Instant::now());
            if delay.is_zero() {
                next_frame = Instant::now();
            }
            if crossterm::event::poll(delay).map_err(|e| e.to_string())? {
                let ev = crossterm::event::read().map_err(|e| e.to_string())?;
                let Event::Key(key) = ev else { continue };
                let Some(key) = map_key(&key) else { continue };
                match key {
                    Key::Quit => break,
                    Key::Toggle => {
                        if !started {
                            started = true;
                            ready = false;
                            paused = false;
                            audio.play();
                        } else {
                            paused = !paused;
                            if paused {
                                audio.pause();
                            } else {
                                audio.play();
                            }
                        }
                    }
                    Key::Restart => {
                        audio.seek_and_play(0.0);
                        started = true;
                        ready = false;
                        paused = false;
                    }
                    Key::Chapter(n) => {
                        let idx = (n as usize).saturating_sub(1).min(CHAPTERS.len() - 1);
                        audio.seek_and_play(CHAPTERS[idx].0);
                        started = true;
                        ready = false;
                        paused = false;
                    }
                    Key::SeekForward | Key::SeekBack => {
                        let delta = if key == Key::SeekForward { 5.0 } else { -5.0 };
                        audio.seek((duration - 0.05f64).min((current + delta).max(0.0)));
                    }
                    Key::OffsetEarlier => offset = round2(offset + 0.1),
                    Key::OffsetLater => offset = round2(offset - 0.1),
                    Key::PrevLyric | Key::NextLyric => {
                        let mut i = film.times.partition_point(|&x| x <= current + 0.03) as i64 - 1;
                        i += if key == Key::NextLyric { 1 } else { -1 };
                        i = i.min(film.times.len() as i64 - 1).max(0);
                        audio.seek(film.times[i as usize]);
                        started = true;
                        ready = false;
                    }
                    Key::Help => help_on = !help_on,
                    Key::VolumeUp => {
                        volume = (volume + 0.05).min(1.0);
                        audio.set_volume(volume);
                    }
                    Key::VolumeDown => {
                        volume = (volume - 0.05).max(0.0);
                        audio.set_volume(volume);
                    }
                }
            }
        }
        Ok(())
    })();

    audio.close();
    result?;
    Ok(RunOutput {
        frames,
        last_time: current,
        max_frame_render_seconds: max_render,
        samples,
    })
}

/// Python's `round(offset, 2)`.
fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// `--snapshot`: render one frame and return it (optionally without escapes).
pub fn render_snapshot(film: &Film, args: &Args, t: f64) -> String {
    let c: Canvas = film.render(t, args.width, args.height, true, 0.0, false, false);
    if args.plain {
        c.plain()
    } else {
        c.ansi()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn maps_the_original_key_set() {
        assert_eq!(map_key(&key(KeyCode::Char('q'))), Some(Key::Quit));
        assert_eq!(map_key(&key(KeyCode::Char('Q'))), Some(Key::Quit));
        assert_eq!(map_key(&key(KeyCode::Esc)), Some(Key::Quit));
        assert_eq!(map_key(&key(KeyCode::Char(' '))), Some(Key::Toggle));
        assert_eq!(map_key(&key(KeyCode::Enter)), Some(Key::Toggle));
        assert_eq!(map_key(&key(KeyCode::Char('r'))), Some(Key::Restart));
        assert_eq!(map_key(&key(KeyCode::Char('3'))), Some(Key::Chapter(3)));
        assert_eq!(map_key(&key(KeyCode::Char('['))), Some(Key::OffsetEarlier));
        assert_eq!(map_key(&key(KeyCode::Char(']'))), Some(Key::OffsetLater));
        assert_eq!(map_key(&key(KeyCode::Char(','))), Some(Key::PrevLyric));
        assert_eq!(map_key(&key(KeyCode::Char('.'))), Some(Key::NextLyric));
        assert_eq!(map_key(&key(KeyCode::Char('H'))), Some(Key::Help));
        assert_eq!(map_key(&key(KeyCode::Char('+'))), Some(Key::VolumeUp));
        assert_eq!(map_key(&key(KeyCode::Char('='))), Some(Key::VolumeUp));
        assert_eq!(map_key(&key(KeyCode::Char('-'))), Some(Key::VolumeDown));
        assert_eq!(map_key(&key(KeyCode::Left)), Some(Key::SeekBack));
        assert_eq!(map_key(&key(KeyCode::Right)), Some(Key::SeekForward));
        // Unmapped keys must be ignored, matching the original's fall-through.
        assert_eq!(map_key(&key(KeyCode::Char('z'))), None);
        assert_eq!(map_key(&key(KeyCode::Char('0'))), None);
        assert_eq!(map_key(&key(KeyCode::Char('6'))), None);
    }

    #[test]
    fn ignores_key_release_events() {
        let release = KeyEvent::new_with_kind(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        );
        assert_eq!(map_key(&release), None);
    }

    #[test]
    fn ctrl_c_quits_but_other_ctrl_keys_do_not() {
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(map_key(&ctrl_c), Some(Key::Quit));
        let ctrl_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        assert_eq!(map_key(&ctrl_a), None);
    }

    #[test]
    fn fps_is_validated() {
        let err = parse_args(&["x".into(), "--fps".into(), "4".into()]).unwrap_err();
        assert!(err.contains("--fps must be between 5 and 60"));
        assert!(parse_args(&["x".into(), "--fps".into(), "5".into()]).is_ok());
        assert!(parse_args(&["x".into(), "--fps".into(), "60".into()]).is_ok());
        assert!(parse_args(&["x".into(), "--fps".into(), "61".into()]).is_err());
    }
}

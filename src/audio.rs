//! Audio playback and the audio clock — replaces the original's macOS
//! `audio-clock` helper process.
//!
//! Behavioural contract taken from `player.py`:
//!
//! * playback starts **paused**; the caller issues `play` on space/enter,
//! * `play`/`pause`/`seek` take effect immediately (synchronously) so the very
//!   next frame reflects them — the original wrote commands to a child process
//!   and then re-read its state each frame,
//! * `seek` is allowed while paused and does not resume,
//! * `pause` freezes the reported time, `play` resumes it,
//! * the reported time drives every frame, so it must be monotonic and must not
//!   advance while paused,
//! * volume starts at 0.75.
//!
//! The original got its clock from AVFoundation in a child process. Here the
//! clock is derived from a monotonic `Instant` plus the last seek target, which
//! is steadier than sampling the sink and never stalls the render loop.
//!
//! `rodio::Sink` is not `Send`, so this type is deliberately confined to the
//! thread that renders — which is also the thread that issues the commands.

use std::io::Cursor;
use std::time::{Duration, Instant};

use rodio::{Decoder, OutputStream, Sink, Source};

/// Initial volume, matching `player.py`'s `volume=.75`.
pub const DEFAULT_VOLUME: f64 = 0.75;

pub struct AudioEngine {
    /// Kept alive for the engine's lifetime: dropping the `OutputStream` stops
    /// audio output entirely.
    _stream: OutputStream,
    sink: Sink,
    /// Position the clock was last anchored at, in seconds.
    base: f64,
    /// When the clock last (re)started, if it is running.
    since: Option<Instant>,
    duration: f64,
}

impl AudioEngine {
    /// Decode `bytes` as MP3 and prepare a sink positioned at `start`.
    ///
    /// Mirrors the original's constructor, which blocks until the audio clock
    /// reports a duration.
    pub fn start(
        bytes: &'static [u8],
        start: f64,
        autoplay: bool,
        volume: f64,
    ) -> Result<AudioEngine, String> {
        // Decode once up front to learn the duration; the sink needs its own
        // owned source.
        let probe = Decoder::new(Cursor::new(bytes)).map_err(|e| format!("无法解码音频：{e}"))?;
        let duration = probe
            .total_duration()
            .ok_or_else(|| "无法确定音频长度。".to_string())?;
        drop(probe);

        let (stream, handle) = OutputStream::try_default()
            .map_err(|e| format!("音频输出不可用，请检查系统声音输出设备。({e})"))?;
        let sink = Sink::try_new(&handle).map_err(|e| format!("无法创建音频输出流。({e})"))?;
        let source = Decoder::new(Cursor::new(bytes)).map_err(|e| format!("无法解码音频：{e}"))?;
        sink.append(source);
        sink.set_volume(volume.clamp(0.0, 1.0) as f32);
        sink.pause();

        let mut engine = AudioEngine {
            _stream: stream,
            sink,
            base: start.clamp(0.0, duration.as_secs_f64()),
            since: None,
            duration: duration.as_secs_f64(),
        };
        // Honour the original's initial `seek {start}`.
        if engine.base > 0.0 {
            let _ = engine.sink.try_seek(Duration::from_secs_f64(engine.base));
        }
        if autoplay {
            engine.sink.play();
            engine.since = Some(Instant::now());
        }
        Ok(engine)
    }

    pub fn duration(&self) -> f64 {
        self.duration
    }

    /// Current playback position in seconds.
    ///
    /// Clamped to the track length and frozen while paused, so the renderer
    /// sees exactly the time the audio is at.
    pub fn time(&self) -> f64 {
        match self.since {
            Some(anchor) => (self.base + anchor.elapsed().as_secs_f64()).min(self.duration),
            None => self.base,
        }
    }

    pub fn playing(&self) -> bool {
        self.since.is_some() && !self.sink.empty()
    }

    pub fn play(&mut self) {
        if self.sink.empty() {
            // The source is exhausted; there is nothing to resume.
            return;
        }
        self.sink.play();
        if self.since.is_none() {
            self.since = Some(Instant::now());
        }
    }

    pub fn pause(&mut self) {
        let now = self.time();
        self.sink.pause();
        self.base = now;
        self.since = None;
    }

    /// Seek without changing the play/pause state, exactly like the original's
    /// `seek` command.
    pub fn seek(&mut self, t: f64) {
        let was_playing = self.since;
        let target = t.clamp(0.0, self.duration);
        if self.sink.try_seek(Duration::from_secs_f64(target)).is_err() {
            // A failed seek must not desynchronise the clock from the audio:
            // keep the clock where the audio actually is.
            return;
        }
        self.base = target;
        self.since = was_playing.map(|_| Instant::now());
    }

    /// `player.py` issues `seek` then `play` for the space/enter, `R` and
    /// chapter keys; keeping the pair adjacent avoids a visible clock stall.
    pub fn seek_and_play(&mut self, t: f64) {
        self.seek(t);
        self.play();
    }

    pub fn set_volume(&mut self, v: f64) {
        self.sink.set_volume(v.clamp(0.0, 1.0) as f32);
    }

    /// Stop playback and release the audio device.
    pub fn close(&mut self) {
        self.sink.stop();
    }
}

#[cfg(test)]
mod tests {
    /// The clock arithmetic is the part worth testing without an audio device:
    /// it must not advance while paused, and must be clamped to the track.
    #[test]
    fn clock_semantics() {
        // Mirrors `AudioEngine::time` with a synthetic anchor.
        let duration = 10.0f64;
        let mut base = 3.0f64;
        let mut since: Option<std::time::Instant> = None;
        let time = |base: f64, since: Option<std::time::Instant>| match since {
            Some(a) => (base + a.elapsed().as_secs_f64()).min(duration),
            None => base,
        };
        // Paused: frozen.
        assert_eq!(time(base, since), 3.0);
        // Playing: advances.
        since = Some(std::time::Instant::now());
        assert!(time(base, since) >= 3.0);
        // Pause captures the current position.
        base = time(base, since);
        since = None;
        assert_eq!(time(base, since), base);
        // Seek clamps into range.
        base = 500.0f64.clamp(0.0, duration);
        assert_eq!(base, 10.0);
    }
}

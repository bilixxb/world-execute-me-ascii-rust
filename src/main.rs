//! world.execute(me); — bilingual terminal MV.

use std::path::Path;

use world_execute_me_rust::player::{self, Args};
use world_execute_me_rust::Film;

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let args = match player::parse_args(&argv) {
        Ok(a) => a,
        Err(msg) => {
            // `--help` and usage errors both land here; the original's
            // argparse prints usage and exits 2 on error, 0 on --help.
            if msg == player::USAGE {
                println!("{msg}");
                std::process::exit(0);
            }
            eprintln!("{msg}");
            std::process::exit(2);
        }
    };

    let film = match Film::load() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("播放失败：{e}");
            std::process::exit(1);
        }
    };

    if let Some(t) = args.snapshot {
        print!("{}", player::render_snapshot(&film, &args, t));
        return;
    }

    match player::run(args.clone(), &film) {
        Ok(out) => {
            if let Some(path) = &args.report {
                if let Err(e) = write_report(path, &out) {
                    eprintln!("无法写入报告：{e}");
                }
            }
        }
        Err(e) => {
            eprintln!("播放失败：{e}");
            std::process::exit(1);
        }
    }
}

/// Mirror of `player.py`'s `--report` JSON.
fn write_report(path: &Path, out: &player::RunOutput) -> std::io::Result<()> {
    let samples: Vec<String> = out
        .samples
        .iter()
        .map(|s| {
            format!(
                "    {{\n      \"audio_time\": {},\n      \"playing\": {},\n      \"width\": {},\n      \"height\": {},\n      \"frames\": {}\n    }}",
                s.audio_time, s.playing, s.width, s.height, s.frames
            )
        })
        .collect();
    let json = format!(
        "{{\n  \"frames\": {},\n  \"last_time\": {},\n  \"max_frame_render_seconds\": {},\n  \"samples\": [\n{}\n  ]\n}}",
        out.frames,
        out.last_time,
        out.max_frame_render_seconds,
        samples.join(",\n")
    );
    std::fs::write(path, json)
}

#!/usr/bin/env python3
"""Golden-output oracle: import the original player's render path headlessly.

The original player.py imports POSIX-only modules (termios/tty/select). Those are
stubbed out here so that only the *rendering* path is exercised -- which is exactly
what the Rust port must reproduce byte-for-byte.

Usage:
    python tools/oracle.py render --time 30.0 --width 120 --height 40 [--ansi]
    python tools/oracle.py batch --out goldens.json
"""
from __future__ import annotations

import argparse
import json
import sys
import types
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / 'world.execute-me-ascii'

# The original reads its JSON assets with Path.read_text(), which on Windows
# defaults to the ANSI code page. Force UTF-8 so the oracle matches macOS.
if not sys.flags.utf8_mode:
    import os

    os.environ['PYTHONUTF8'] = '1'
    os.execv(sys.executable, [sys.executable, *sys.argv])


def _stub_posix_modules() -> None:
    """Make player.py importable on non-POSIX hosts."""
    if 'termios' not in sys.modules:
        m = types.ModuleType('termios')
        m.TCSADRAIN = 1
        m.tcgetattr = lambda fd: []
        m.tcsetattr = lambda fd, when, attrs: None
        sys.modules['termios'] = m
    if 'tty' not in sys.modules:
        m = types.ModuleType('tty')
        m.setcbreak = lambda fd: None
        sys.modules['tty'] = m
    if 'fcntl' not in sys.modules:
        sys.modules['fcntl'] = types.ModuleType('fcntl')
    if 'select' not in sys.modules:
        m = types.ModuleType('select')
        m.select = lambda *a, **k: ([], [], [])
        sys.modules['select'] = m


def load_film():
    _stub_posix_modules()
    sys.path.insert(0, str(SRC))
    import player  # noqa: E402

    return player.Film()


def main() -> int:
    p = argparse.ArgumentParser()
    sub = p.add_subparsers(dest='cmd', required=True)

    r = sub.add_parser('render')
    r.add_argument('--time', type=float, required=True)
    r.add_argument('--width', type=int, default=120)
    r.add_argument('--height', type=int, default=40)
    r.add_argument('--offset', type=float, default=0.0)
    r.add_argument('--paused', action='store_true')
    r.add_argument('--help-on', action='store_true')
    r.add_argument('--ready', action='store_true')
    r.add_argument('--plain', action='store_true')

    b = sub.add_parser('batch')
    b.add_argument('--out', default=str(ROOT / 'goldens.json'))

    a = p.parse_args()
    film = load_film()

    if a.cmd == 'render':
        c = film.render(a.time, a.width, a.height, a.paused, a.offset, a.help_on, a.ready)
        sys.stdout.write(c.plain() if a.plain else c.ansi())
        return 0

    frames = json.loads((ROOT / 'tools' / 'golden_frames.json').read_text())
    out = {}
    for spec in frames:
        c = film.render(
            spec['t'], spec['w'], spec['h'],
            spec.get('paused', False), spec.get('offset', 0.0),
            spec.get('help', False), spec.get('ready', False),
        )
        out[spec['id']] = spec | {'ansi': c.ansi(), 'plain': c.plain()}
    Path(a.out).write_text(json.dumps(out, ensure_ascii=False, indent=1), encoding='utf-8')
    print(f'wrote {len(out)} frames -> {a.out}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

"""Generate golden frames, one fresh Python process per frame (avoids cross-frame cache state).

Produces tools/goldens.json: {id: {t, w, h, ansi, plain, ...}}.
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PY = sys.executable
ORACLE = ROOT / 'tools' / 'oracle.py'
OUT = ROOT / 'tools' / 'goldens.json'


def build_specs() -> list[dict]:
    specs: list[dict] = []

    def add(t, w=120, h=40, **kw):
        specs.append({'id': '', 't': round(t, 4), 'w': w, 'h': h} | kw)

    # Boot sequence boundaries and interior (0-16s)
    for t in (0.0, 0.1, 0.5, 1.0, 1.739, 1.74, 2.0, 2.919, 2.92, 3.5, 3.872,
              3.873, 4.5, 5.49, 5.491, 6.0, 6.379, 6.38, 7.0, 7.445, 7.446,
              8.5, 10.09, 10.091, 11.0, 11.094, 11.095, 12.0, 13.5, 15.0, 15.799):
        add(t)

    # Title takeover, including the negative-elapsed pre-transition (15.8-16s)
    for t in (15.8, 15.85, 15.9, 15.95, 15.999, 16.0, 16.05, 16.3, 16.5, 17.0,
              17.5, 18.099, 18.1, 18.5, 19.0, 19.2, 19.6, 20.0, 20.5, 21.0,
              23.0, 25.0, 27.0, 27.5, 28.5, 29.0, 29.4, 29.708):
        add(t)

    # Devotion 29.709-59.223
    for t in (29.709, 30.0, 31.0, 32.0, 33.411, 33.412, 34.5, 36.0, 37.066,
              37.067, 38.0, 39.5, 40.705, 40.706, 42.0, 44.0, 44.451, 44.452,
              45.5, 47.0, 47.671, 47.672, 48.5, 50.0, 51.362, 51.363, 52.5,
              54.0, 55.082, 55.083, 56.5, 58.0, 59.222):
        add(t)

    # Heart section 59.223-74.045
    for t in (59.223, 60.0, 61.0, 62.588, 62.589, 64.0, 66.0, 66.6, 66.601,
              68.0, 70.0, 70.083, 70.084, 72.0, 74.0, 74.044):
        add(t)

    # Legacy organic 74.045-85.078
    for t in (74.045, 76.0, 78.0, 80.0, 82.0, 84.0, 85.077):
        add(t)

    # God existence 85.078-88.587
    for t in (85.078, 86.0, 87.5, 88.586):
        add(t)

    # Identity switching 88.587-103.489
    for t in (88.587, 90.0, 92.014, 92.015, 93.5, 95.0, 95.464, 95.465, 97.0,
              99.0, 99.348, 99.349, 101.0, 103.488):
        add(t)

    # Vibration 103.489-110.9
    for t in (103.489, 105.0, 107.0, 108.0, 109.0, 110.5, 110.899):
        add(t)

    # Isolation 110.9-118.333
    for t in (110.9, 111.5, 112.22, 113.1, 114.18, 114.92, 115.78, 117.274,
              118.0, 118.332):
        add(t)

    # Erase fragments 118.333-125.708
    for t in (118.333, 120.0, 122.0, 124.0, 125.0, 125.707):
        add(t)

    # Illegal arguments 125.708-147.66
    for t in (125.708, 127.0, 130.0, 135.0, 140.0, 145.0, 147.0, 147.659):
        add(t)

    # Execution 147.66-177.246
    for t in (147.66, 150.0, 155.0, 158.899, 158.9, 160.0, 162.0, 162.631,
              162.632, 165.0, 170.0, 176.0, 177.245):
        add(t)

    # Love equation 177.246-192.5
    for t in (177.246, 180.0, 185.0, 188.0, 188.482, 188.483, 190.0, 192.0,
              192.499, 192.5, 195.0, 200.0, 208.0, 208.001, 211.0, 211.9):
        add(t)

    # Glitch-heavy dense sampling (hash-driven effects need coverage)
    for i in range(60):
        add(110.9 + i * 0.5)
    for i in range(40):
        add(147.66 + i * 0.5)

    # Non-120x40 geometries: minimum, clamped-clamp boundaries, wide, tall
    for (w, h) in ((64, 24), (65, 25), (80, 30), (100, 36), (119, 39),
                   (121, 41), (160, 50), (239, 84), (240, 85), (63, 24),
                   (64, 23), (20, 10), (1, 1), (32, 8), (256, 100)):
        for t in (0.0, 1.0, 8.0, 17.5, 25.0, 30.0, 50.0, 75.0, 90.0, 111.0,
                  120.0, 130.0, 150.0, 170.0, 180.0, 195.0):
            add(t, w, h)

    # Overlays / states
    for t in (0.0, 5.0, 17.5, 30.0, 75.0, 111.0, 150.0, 180.0, 195.0):
        add(t, help=True)
        add(t, ready=True)
        add(t, paused=True)
        add(t, offset=0.7)
        add(t, offset=-0.4)
        add(t, ready=True, help=True)

    for i, s in enumerate(specs):
        s['id'] = f'f{i:04d}'
    return specs


def main() -> int:
    specs = build_specs()
    (ROOT / 'tools' / 'golden_frames.json').write_text(
        json.dumps(specs, indent=1), encoding='utf-8')
    out: dict[str, dict] = {}
    for i, s in enumerate(specs):
        cmd = [PY, str(ORACLE), 'render', '--time', str(s['t']),
               '--width', str(s['w']), '--height', str(s['h'])]
        for flag, key in (('--help-on', 'help'), ('--ready', 'ready'), ('--paused', 'paused')):
            if s.get(key):
                cmd.append(flag)
        if s.get('offset'):
            cmd += ['--offset', str(s['offset'])]
        r = subprocess.run(cmd, capture_output=True, text=True, encoding='utf-8')
        if r.returncode != 0:
            print(f"FAILED {s['id']} t={s['t']}: {r.stderr[-400:]}", file=sys.stderr)
            return 1
        out[s['id']] = s | {'ansi': r.stdout}
        if (i + 1) % 50 == 0:
            print(f'{i + 1}/{len(specs)}', file=sys.stderr)
    OUT.write_text(json.dumps(out, ensure_ascii=False), encoding='utf-8')
    print(f'wrote {len(out)} goldens -> {OUT} ({OUT.stat().st_size} bytes)')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

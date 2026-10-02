"""Requisiten für Tauwinkel und die Blütenwiesen (Playtest A1.9, E-280 bis E-283) im Stil der Gebäude.

Aufruf: python3 tools/design/tauwinkel_requisiten.py && cargo xtask svg-preview \
        docs/release-2/design/tauwinkel-requisiten.svg docs/release-2/design/tauwinkel-requisiten.png 1400

Welteinheiten (1 Tile = 32), Ursprung unten in der Mitte.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))

from tauwinkel_gebaeude import (OUT, STONE, STONE_LINE, WOOD, WOOD_DARK, SHINGLE, SHINGLE_LINE,
                                TEXT, st, text)

THORN = '#4f5f3a'
THORN_DARK = '#38452a'
SPIKE = '#d8ccb0'


def dornen():
    """Dornenranke über einer Grube (E-283), 2 Tiles breit, ein Tile hoch; nebeneinander kachelbar."""
    s = ''
    for (x0, k) in ((-32, 0), (0, 1)):
        s += (f'<path d="M {x0},0 C {x0 + 4},-20 {x0 + 14},-30 {x0 + 16},-18 C {x0 + 18},-30 {x0 + 28},-22 '
              f'{x0 + 32},0 Z" fill="{THORN}" {st()}/>')
        s += (f'<path d="M {x0 + 3},-4 C {x0 + 10},-18 {x0 + 22},-20 {x0 + 30},-6" fill="none" '
              f'stroke="{THORN_DARK}" stroke-width="2.4"/>')
        for (dx, dy, a) in ((6, -14, -40), (16, -22, 0), (26, -14, 40), (11, -7, -70), (22, -8, 70)):
            x, y = x0 + dx, dy - k * 2
            s += (f'<path d="M {x - 3},{y + 2} L {x},{y - 7} L {x + 3},{y + 2} Z" fill="{SPIKE}" '
                  f'{st(1.2)} transform="rotate({a} {x} {y})"/>')
    return s


def bank():
    s = f'<rect x="-34" y="-22" width="6" height="22" fill="{WOOD_DARK}" {st()}/><rect x="28" y="-22" width="6" height="22" fill="{WOOD_DARK}" {st()}/>'
    s += f'<rect x="-40" y="-28" width="80" height="8" rx="2" fill="{WOOD}" {st()}/>'
    s += f'<rect x="-36" y="-50" width="5" height="22" fill="{WOOD_DARK}" {st(1.6)}/><rect x="31" y="-50" width="5" height="22" fill="{WOOD_DARK}" {st(1.6)}/>'
    s += f'<rect x="-40" y="-52" width="80" height="9" rx="2" fill="{WOOD}" {st()}/>'
    return s


def laterne():
    s = f'<rect x="-4" y="-120" width="8" height="120" fill="#4a4a48" {st(1.6)}/>'
    s += f'<rect x="-10" y="-6" width="20" height="6" fill="#4a4a48" {st(1.6)}/>'
    s += f'<path d="M -14,-128 H 14 L 10,-150 H -10 Z" fill="#f6dc8a" {st()}/>'
    s += f'<path d="M 0,-128 V -150 M -12,-139 H 12" stroke="#4a4a48" stroke-width="1.8"/>'
    s += f'<path d="M -18,-150 H 18 L 0,-164 Z" fill="#4a4a48" {st()}/>'
    return s


def fass(x=0, base=0, w=34, h=42):
    s = f'<rect x="{x - w / 2}" y="{base - h}" width="{w}" height="{h}" rx="8" fill="{WOOD}" {st()}/>'
    for y in (base - h * 0.2, base - h * 0.8):
        s += f'<path d="M {x - w / 2 + 1},{y} H {x + w / 2 - 1}" stroke="#5a5a58" stroke-width="3.5"/>'
    s += f'<path d="M {x - 6},{base - h + 3} V {base - 3} M {x + 6},{base - h + 3} V {base - 3}" stroke="{WOOD_DARK}" stroke-width="1.4"/>'
    return s


def faesser():
    return fass(-20) + fass(18, 0, 30, 36) + fass(-2, -42, 30, 36)


def holzstapel():
    s = f'<rect x="-44" y="-4" width="88" height="4" fill="{WOOD_DARK}" {st(1.4)}/>'
    for row, n in ((0, 5), (1, 4), (2, 3)):
        for i in range(n):
            cx = -32 + i * 16 + row * 8
            cy = -12 - row * 15
            s += f'<circle cx="{cx}" cy="{cy}" r="8" fill="#c79a66" {st(1.6)}/><circle cx="{cx}" cy="{cy}" r="3" fill="none" stroke="{WOOD_DARK}" stroke-width="1.2"/>'
    return s


def karren():
    s = f'<path d="M -50,-44 H 34 L 30,-18 H -46 Z" fill="{WOOD}" {st()}/>'
    s += f'<path d="M -46,-31 H 32" stroke="{WOOD_DARK}" stroke-width="1.6"/>'
    s += f'<path d="M 34,-36 L 66,-26" stroke="{WOOD_DARK}" stroke-width="5" stroke-linecap="round"/>'
    s += f'<path d="M -40,-48 Q -26,-62 -12,-50 Q 2,-64 18,-48 Z" fill="#c9a45c" {st(1.6)}/>'
    s += f'<circle cx="-8" cy="-16" r="16" fill="{WOOD_DARK}" {st()}/><circle cx="-8" cy="-16" r="5" fill="#5a5a58" {st(1.4)}/>'
    for a in range(0, 180, 45):
        s += f'<path d="M -8,-16 l 0,-14" stroke="#c79a66" stroke-width="2.4" transform="rotate({a} -8 -16)"/><path d="M -8,-16 l 0,14" stroke="#c79a66" stroke-width="2.4" transform="rotate({a} -8 -16)"/>'
    return s


def waescheleine():
    s = f'<rect x="-74" y="-96" width="6" height="96" fill="{WOOD}" {st(1.6)}/><rect x="68" y="-96" width="6" height="96" fill="{WOOD}" {st(1.6)}/>'
    s += f'<path d="M -70,-90 Q 0,-76 70,-90" fill="none" stroke="#7a7266" stroke-width="1.6"/>'
    for (x, w, h, c) in ((-52, 22, 30, '#d9cfc0'), (-22, 30, 24, '#a9b8c8'), (16, 20, 34, '#d6b6a0'), (44, 18, 22, '#c8c2a8')):
        y = -90 + 14 * (1 - ((x + w / 2) / 70) ** 2)
        s += f'<path d="M {x},{y} H {x + w} L {x + w - 2},{y + h} H {x + 2} Z" fill="{c}" {st(1.6)}/>'
        s += f'<path d="M {x + 4},{y} V {y - 4} M {x + w - 4},{y} V {y - 4}" stroke="{WOOD_DARK}" stroke-width="2.2"/>'
    return s


def heuballen():
    s = f'<rect x="-34" y="-40" width="68" height="40" rx="10" fill="#d9b860" {st()}/>'
    for y in (-28, -14):
        s += f'<path d="M -30,{y} Q 0,{y - 4} 30,{y}" fill="none" stroke="#b08f3c" stroke-width="1.6"/>'
    s += f'<path d="M -12,-40 V 0 M 12,-40 V 0" stroke="#8a6a30" stroke-width="2.4"/>'
    return s


def mauer():
    """Niedrige Feldsteinmauer, 3 Tiles breit."""
    s = f'<path d="M -48,0 V -26 Q -48,-32 -40,-32 H 40 Q 48,-32 48,-26 V 0 Z" fill="{STONE}" {st()}/>'
    for (x, y, w) in ((-44, -22, 20), (-20, -24, 26), (10, -22, 18), (30, -24, 14), (-36, -10, 24), (-8, -12, 22), (18, -10, 24)):
        s += f'<rect x="{x}" y="{y}" width="{w}" height="10" rx="3" fill="#bdb7ac" stroke="{STONE_LINE}" stroke-width="1.4"/>'
    s += f'<path d="M -48,-32 Q -30,-38 -10,-33 Q 14,-38 48,-32" fill="none" stroke="#6f9a4a" stroke-width="4" stroke-linecap="round"/>'
    return s


PROPS = {
    'dornen': (dornen, '-36 -34 72 36', 'Dornenranke über Gruben (E-283)'),
    'bank': (bank, '-44 -56 88 58', 'Bank'),
    'laterne': (laterne, '-22 -168 44 170', 'Laterne'),
    'faesser': (faesser, '-40 -82 80 84', 'Fässer'),
    'holzstapel': (holzstapel, '-46 -54 92 56', 'Holzstapel'),
    'karren': (karren, '-56 -68 128 70', 'Heukarren'),
    'waescheleine': (waescheleine, '-78 -100 156 102', 'Wäscheleine'),
    'heuballen': (heuballen, '-38 -44 76 46', 'Heuballen'),
    'mauer': (mauer, '-52 -40 104 42', 'Feldsteinmauer'),
}


def sheet():
    cell, cols = 220, 5
    rows = (len(PROPS) + cols - 1) // cols
    w, h = cell * cols, 70 + rows * cell
    s = f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}"><rect width="{w}" height="{h}" fill="#f4efe4"/>'
    s += text(w / 2, 40, 'Tauwinkel – Requisiten (Playtest A1.9)', 22)
    for i, (name, (fn, _, note)) in enumerate(PROPS.items()):
        cx = cell * (i % cols) + cell / 2
        base = 70 + cell * (i // cols) + cell - 50
        s += f'<path d="M {cx - 90},{base} H {cx + 90}" stroke="#7d9a5a" stroke-width="6"/>'
        s += f'<g transform="translate({cx} {base})">{fn()}</g>'
        s += text(cx, base + 28, note, 13, TEXT)
    return s + '</svg>'


def export():
    for name, (fn, vb, note) in PROPS.items():
        with open(f'assets/map/decor/{name}.svg', 'w') as f:
            f.write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}">\n'
                    f'  <!-- {note} (A1.9, aus tools/design/tauwinkel_requisiten.py). Ursprung unten in der Mitte. -->\n'
                    f'  {fn()}\n</svg>\n')


if __name__ == '__main__':
    open('docs/release-2/design/tauwinkel-requisiten.svg', 'w').write(sheet())
    export()

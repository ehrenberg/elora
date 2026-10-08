"""Props for Tauwinkel and the Blütenwiesen (playtest A1.9, E-280 to E-283) in the style of the buildings.

Usage: python3 tools/design/tauwinkel_props.py && cargo xtask svg-preview \
        docs/release-2/design/tauwinkel-props.svg docs/release-2/design/tauwinkel-props.png 1400

World units (1 tile = 32), origin at the bottom centre.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))

from tauwinkel_buildings import (OUT, STONE, STONE_LINE, WOOD, WOOD_DARK, SHINGLE, SHINGLE_LINE,
                                TEXT, st, text)

THORN = '#4f5f3a'
THORN_DARK = '#38452a'
SPIKE = '#d8ccb0'


def thorns():
    """Thorn vine over a pit (E-283), 2 tiles wide, one tile high; tileable side by side."""
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


def bench():
    s = f'<rect x="-34" y="-22" width="6" height="22" fill="{WOOD_DARK}" {st()}/><rect x="28" y="-22" width="6" height="22" fill="{WOOD_DARK}" {st()}/>'
    s += f'<rect x="-40" y="-28" width="80" height="8" rx="2" fill="{WOOD}" {st()}/>'
    s += f'<rect x="-36" y="-50" width="5" height="22" fill="{WOOD_DARK}" {st(1.6)}/><rect x="31" y="-50" width="5" height="22" fill="{WOOD_DARK}" {st(1.6)}/>'
    s += f'<rect x="-40" y="-52" width="80" height="9" rx="2" fill="{WOOD}" {st()}/>'
    return s


def lantern():
    s = f'<rect x="-4" y="-120" width="8" height="120" fill="#4a4a48" {st(1.6)}/>'
    s += f'<rect x="-10" y="-6" width="20" height="6" fill="#4a4a48" {st(1.6)}/>'
    s += f'<path d="M -14,-128 H 14 L 10,-150 H -10 Z" fill="#f6dc8a" {st()}/>'
    s += f'<path d="M 0,-128 V -150 M -12,-139 H 12" stroke="#4a4a48" stroke-width="1.8"/>'
    s += f'<path d="M -18,-150 H 18 L 0,-164 Z" fill="#4a4a48" {st()}/>'
    return s


def barrel(x=0, base=0, w=34, h=42):
    s = f'<rect x="{x - w / 2}" y="{base - h}" width="{w}" height="{h}" rx="8" fill="{WOOD}" {st()}/>'
    for y in (base - h * 0.2, base - h * 0.8):
        s += f'<path d="M {x - w / 2 + 1},{y} H {x + w / 2 - 1}" stroke="#5a5a58" stroke-width="3.5"/>'
    s += f'<path d="M {x - 6},{base - h + 3} V {base - 3} M {x + 6},{base - h + 3} V {base - 3}" stroke="{WOOD_DARK}" stroke-width="1.4"/>'
    return s


def barrels():
    return barrel(-20) + barrel(18, 0, 30, 36) + barrel(-2, -42, 30, 36)


def woodpile():
    s = f'<rect x="-44" y="-4" width="88" height="4" fill="{WOOD_DARK}" {st(1.4)}/>'
    for row, n in ((0, 5), (1, 4), (2, 3)):
        for i in range(n):
            cx = -32 + i * 16 + row * 8
            cy = -12 - row * 15
            s += f'<circle cx="{cx}" cy="{cy}" r="8" fill="#c79a66" {st(1.6)}/><circle cx="{cx}" cy="{cy}" r="3" fill="none" stroke="{WOOD_DARK}" stroke-width="1.2"/>'
    return s


def cart():
    s = f'<path d="M -50,-44 H 34 L 30,-18 H -46 Z" fill="{WOOD}" {st()}/>'
    s += f'<path d="M -46,-31 H 32" stroke="{WOOD_DARK}" stroke-width="1.6"/>'
    s += f'<path d="M 34,-36 L 66,-26" stroke="{WOOD_DARK}" stroke-width="5" stroke-linecap="round"/>'
    s += f'<path d="M -40,-48 Q -26,-62 -12,-50 Q 2,-64 18,-48 Z" fill="#c9a45c" {st(1.6)}/>'
    s += f'<circle cx="-8" cy="-16" r="16" fill="{WOOD_DARK}" {st()}/><circle cx="-8" cy="-16" r="5" fill="#5a5a58" {st(1.4)}/>'
    for a in range(0, 180, 45):
        s += f'<path d="M -8,-16 l 0,-14" stroke="#c79a66" stroke-width="2.4" transform="rotate({a} -8 -16)"/><path d="M -8,-16 l 0,14" stroke="#c79a66" stroke-width="2.4" transform="rotate({a} -8 -16)"/>'
    return s


def clothesline():
    s = f'<rect x="-74" y="-96" width="6" height="96" fill="{WOOD}" {st(1.6)}/><rect x="68" y="-96" width="6" height="96" fill="{WOOD}" {st(1.6)}/>'
    s += f'<path d="M -70,-90 Q 0,-76 70,-90" fill="none" stroke="#7a7266" stroke-width="1.6"/>'
    for (x, w, h, c) in ((-52, 22, 30, '#d9cfc0'), (-22, 30, 24, '#a9b8c8'), (16, 20, 34, '#d6b6a0'), (44, 18, 22, '#c8c2a8')):
        y = -90 + 14 * (1 - ((x + w / 2) / 70) ** 2)
        s += f'<path d="M {x},{y} H {x + w} L {x + w - 2},{y + h} H {x + 2} Z" fill="{c}" {st(1.6)}/>'
        s += f'<path d="M {x + 4},{y} V {y - 4} M {x + w - 4},{y} V {y - 4}" stroke="{WOOD_DARK}" stroke-width="2.2"/>'
    return s


def hay_bales():
    s = f'<rect x="-34" y="-40" width="68" height="40" rx="10" fill="#d9b860" {st()}/>'
    for y in (-28, -14):
        s += f'<path d="M -30,{y} Q 0,{y - 4} 30,{y}" fill="none" stroke="#b08f3c" stroke-width="1.6"/>'
    s += f'<path d="M -12,-40 V 0 M 12,-40 V 0" stroke="#8a6a30" stroke-width="2.4"/>'
    return s


def field_wall():
    """Low fieldstone wall, 3 tiles wide."""
    s = f'<path d="M -48,0 V -26 Q -48,-32 -40,-32 H 40 Q 48,-32 48,-26 V 0 Z" fill="{STONE}" {st()}/>'
    for (x, y, w) in ((-44, -22, 20), (-20, -24, 26), (10, -22, 18), (30, -24, 14), (-36, -10, 24), (-8, -12, 22), (18, -10, 24)):
        s += f'<rect x="{x}" y="{y}" width="{w}" height="10" rx="3" fill="#bdb7ac" stroke="{STONE_LINE}" stroke-width="1.4"/>'
    s += f'<path d="M -48,-32 Q -30,-38 -10,-33 Q 14,-38 48,-32" fill="none" stroke="#6f9a4a" stroke-width="4" stroke-linecap="round"/>'
    return s


def watering_can():
    s = f'<path d="M -14,0 V -24 Q -14,-28 -10,-28 H 10 Q 14,-28 14,-24 V 0 Z" fill="#7f9aa6" {st()}/>'
    s += f'<path d="M 14,-20 L 30,-34 L 33,-31" fill="none" stroke="{OUT}" stroke-width="4.5" stroke-linecap="round"/>'
    s += f'<path d="M 14,-20 L 30,-34 L 33,-31" fill="none" stroke="#7f9aa6" stroke-width="2.5" stroke-linecap="round"/>'
    s += f'<path d="M -10,-28 Q 0,-42 10,-28" fill="none" stroke="{OUT}" stroke-width="3"/>'
    return s


def birdhouse():
    s = f'<rect x="-3" y="-80" width="6" height="80" fill="{WOOD}" {st(1.6)}/>'
    s += f'<rect x="-16" y="-104" width="32" height="26" fill="#c79a66" {st()}/>'
    s += f'<path d="M -22,-102 L 0,-120 L 22,-102 Z" fill="{SHINGLE}" {st()}/>'
    s += f'<circle cx="0" cy="-92" r="5" fill="{OUT}"/><path d="M -4,-82 H 4" stroke="{WOOD_DARK}" stroke-width="2.4"/>'
    return s


def mailbox():
    s = f'<rect x="-3" y="-46" width="6" height="46" fill="{WOOD}" {st(1.6)}/>'
    s += f'<path d="M -14,-46 V -60 Q -14,-68 0,-68 Q 14,-68 14,-60 V -46 Z" fill="#c8a24a" {st()}/>'
    s += f'<path d="M -8,-58 H 8" stroke="{OUT}" stroke-width="2"/><path d="M 14,-62 h 6 v 8" fill="none" stroke="#b5523b" stroke-width="2.5"/>'
    return s


def basket():
    s = ''
    for (x, y) in ((-10, -22), (2, -24), (12, -21), (-4, -30), (8, -30)):
        s += f'<circle cx="{x}" cy="{y}" r="6" fill="#c8483a" {st(1.4)}/><path d="M {x},{y - 6} v -3" stroke="#5e4430" stroke-width="1.4"/>'
    s += f'<path d="M -20,-20 H 22 L 16,0 H -14 Z" fill="#c9a45c" {st()}/>'
    s += f'<path d="M -17,-12 H 19 M -6,-20 L -4,0 M 6,-20 L 6,0" stroke="#a3813f" stroke-width="1.4"/>'
    return s


def pumpkins():
    s = ''
    for (x, r, c) in ((-16, 14, '#d9822b'), (12, 11, '#e09a3c'), (-1, 9, '#c8702a')):
        s += f'<ellipse cx="{x}" cy="{-r}" rx="{r * 1.2}" ry="{r}" fill="{c}" {st()}/>'
        s += f'<path d="M {x},{-2 * r + 1} Q {x - r * 0.5},{-r} {x},{-1} M {x},{-2 * r + 1} Q {x + r * 0.5},{-r} {x},{-1}" fill="none" stroke="#a5561e" stroke-width="1.2"/>'
        s += f'<path d="M {x},{-2 * r} q 2,-6 6,-6" fill="none" stroke="#5e7a32" stroke-width="2.4"/>'
    return s


def flower_pot(pale=True):
    from tauwinkel_buildings import flowers
    s = f'<path d="M -10,0 L -13,-16 H 13 L 10,0 Z" fill="#b8673f" {st()}/>'
    return s + flowers([-6, 0, 6], -16, pale)


def cat():
    """Sleeping cat (on a bench, wall or ground)."""
    s = f'<path d="M -22,0 Q -24,-16 -6,-18 Q 12,-20 18,-8 Q 20,0 12,0 Z" fill="#8a8580" {st()}/>'
    s += f'<path d="M 10,-14 L 12,-24 L 17,-16 L 22,-22 L 22,-10 Q 20,-4 14,-6 Z" fill="#8a8580" {st()}/>'
    s += f'<path d="M 15,-12 q 1.5,1.5 3,0" fill="none" stroke="{OUT}" stroke-width="1.4"/>'
    s += f'<path d="M -22,-2 Q -32,-2 -30,-10 Q -28,-14 -22,-12" fill="none" stroke="{OUT}" stroke-width="4.5" stroke-linecap="round"/>'
    s += f'<path d="M -22,-2 Q -32,-2 -30,-10 Q -28,-14 -22,-12" fill="none" stroke="#8a8580" stroke-width="2.6" stroke-linecap="round"/>'
    s += f'<path d="M -8,-12 q 4,-3 8,0 M 0,-8 q 4,-3 8,0" fill="none" stroke="#6e6a66" stroke-width="1.4"/>'
    return s


def bird():
    """Perched bird (roof ridge, fence)."""
    s = f'<path d="M -8,-2 Q -10,-12 -2,-14 Q 6,-16 8,-8 Q 8,-2 2,-1 Z" fill="#5b7fa6" {st(1.6)}/>'
    s += f'<circle cx="5" cy="-12" r="1.4" fill="{OUT}"/><path d="M 8,-11 l 4,1 l -4,1.5 Z" fill="#e0a23a" {st(1)}/>'
    s += f'<path d="M -8,-4 l -6,-2 l 2,4 Z" fill="#45648a" {st(1)}/><path d="M -1,-1 v 2 M 2,-1 v 2" stroke="{OUT}" stroke-width="1.2"/>'
    return s


def butterfly():
    s = f'<path d="M 0,-6 Q -10,-18 -12,-8 Q -12,-2 0,-6 Z M 0,-6 Q 10,-18 12,-8 Q 12,-2 0,-6 Z" fill="#f2c14e" {st(1.2)}/>'
    s += f'<path d="M 0,-6 Q -8,0 -6,3 Q -2,2 0,-6 Z M 0,-6 Q 8,0 6,3 Q 2,2 0,-6 Z" fill="#e8925a" {st(1.2)}/>'
    s += f'<path d="M 0,-10 V 0" stroke="{OUT}" stroke-width="1.8" stroke-linecap="round"/>'
    return s


def smoke():
    return (f'<circle cx="0" cy="-10" r="10" fill="#e6e2dc" stroke="#b8b2aa" stroke-width="1.4"/>'
            f'<circle cx="9" cy="-15" r="7" fill="#e6e2dc" stroke="#b8b2aa" stroke-width="1.4"/>')


def tree_stump():
    s = f'<path d="M -22,0 Q -16,-4 -16,-26 H 16 Q 16,-4 22,0 Z" fill="#8a6040" {st()}/>'
    s += f'<ellipse cx="0" cy="-26" rx="16" ry="5" fill="#d9b080" {st(1.6)}/>'
    s += f'<ellipse cx="0" cy="-26" rx="8" ry="2.4" fill="none" stroke="#a8804f" stroke-width="1.2"/>'
    s += f'<path d="M 12,-18 q 8,-6 10,-14" fill="none" stroke="#5e7a32" stroke-width="2"/><circle cx="22" cy="-33" r="3" fill="#6f9a4a"/>'
    return s


def fern():
    s = ''
    for (a, l) in ((-50, 34), (-22, 42), (0, 46), (24, 40), (52, 32)):
        s += f'<g transform="rotate({a})"><path d="M 0,0 Q 4,{-l / 2} 0,{-l}" fill="none" stroke="#3f6e34" stroke-width="2"/>'
        for k in range(1, 6):
            y = -l * k / 6
            w = 8 * (1 - k / 7)
            s += f'<path d="M 0,{y} q {-w},-2 {-w - 2},-6 M 0,{y} q {w},-2 {w + 2},-6" fill="none" stroke="#5b9d42" stroke-width="2.4" stroke-linecap="round"/>'
        s += '</g>'
    return s


def berry_bush():
    s = f'<path d="M -30,0 Q -36,-24 -14,-30 Q -2,-44 14,-32 Q 34,-28 30,0 Z" fill="#4f8f3a" {st()}/>'
    for (x, y) in ((-18, -14), (-8, -22), (4, -12), (14, -22), (20, -10), (-2, -30), (-22, -6)):
        s += f'<circle cx="{x}" cy="{y}" r="3.4" fill="#7a3fa0" stroke="{OUT}" stroke-width="1"/>'
    return s


def dandelion():
    s = ''
    for (x, h) in ((-8, 26), (4, 34), (12, 20)):
        s += f'<path d="M {x},0 Q {x - 2},{-h / 2} {x},{-h}" fill="none" stroke="#6f9a4a" stroke-width="2"/>'
        s += f'<circle cx="{x}" cy="{-h}" r="6" fill="#f4f1ea" stroke="#c8c2b4" stroke-width="1.2"/>'
        for a in range(0, 360, 45):
            s += f'<path d="M {x},{-h} l 0,-6" stroke="#d8d2c4" stroke-width="1" transform="rotate({a} {x} {-h})"/>'
    s += f'<path d="M -16,0 q 6,-8 10,-2 q 6,-8 12,0 q 6,-6 10,0" fill="#5b9d42" {st(1.2)}/>'
    return s


def stepping_stones():
    s = ''
    for (x, w) in ((-40, 22), (-12, 18), (14, 24), (40, 16)):
        s += f'<ellipse cx="{x}" cy="-2" rx="{w / 2}" ry="4" fill="#bdb7ac" stroke="{STONE_LINE}" stroke-width="1.4"/>'
    return s


PROPS = {
    'dornen': (thorns, '-36 -34 72 36', 'Dornenranke über Gruben (E-283)'),
    'bank': (bench, '-44 -56 88 58', 'Bank'),
    'laterne': (lantern, '-22 -168 44 170', 'Laterne'),
    'faesser': (barrels, '-40 -82 80 84', 'Fässer'),
    'holzstapel': (woodpile, '-46 -54 92 56', 'Holzstapel'),
    'karren': (cart, '-56 -68 128 70', 'Heukarren'),
    'waescheleine': (clothesline, '-78 -100 156 102', 'Wäscheleine'),
    'heuballen': (hay_bales, '-38 -44 76 46', 'Heuballen'),
    'mauer': (field_wall, '-52 -40 104 42', 'Feldsteinmauer'),
    'giesskanne': (watering_can, '-18 -46 56 48', 'Gießkanne'),
    'vogelhaus': (birdhouse, '-26 -124 52 126', 'Vogelhaus'),
    'briefkasten': (mailbox, '-18 -72 42 74', 'Briefkasten'),
    'korb': (basket, '-24 -40 50 42', 'Apfelkorb'),
    'kuerbisse': (pumpkins, '-36 -38 70 40', 'Kürbisse'),
    'katze': (cat, '-36 -28 62 30', 'schlafende Katze'),
    'vogel': (bird, '-16 -18 30 20', 'Vogel'),
    'schmetterling': (butterfly, '-14 -20 28 26', 'Schmetterling (bewegt)'),
    'rauch': (smoke, '-12 -24 30 26', 'Rauch (steigt auf)'),
    'baumstumpf': (tree_stump, '-26 -40 54 42', 'Baumstumpf'),
    'farn': (fern, '-44 -50 88 52', 'Farn'),
    'beerenbusch': (berry_bush, '-36 -46 72 48', 'Beerenbusch'),
    'loewenzahn': (dandelion, '-20 -44 40 46', 'Löwenzahn'),
    'trittsteine': (stepping_stones, '-54 -8 108 10', 'Trittsteine'),
}
VARIANTS = {
    'blumentopf': (flower_pot, '-16 -34 32 36', 'Blumentopf'),
}


def sheet():
    cell, cols = 220, 5
    rows = (len(PROPS) + 2 * len(VARIANTS) + cols - 1) // cols
    w, h = cell * cols, 70 + rows * cell
    s = f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}"><rect width="{w}" height="{h}" fill="#f4efe4"/>'
    s += text(w / 2, 40, 'Tauwinkel – Requisiten (Playtest A1.9)', 22)
    items = list(PROPS.items()) + [(n + '-blass', (lambda f=f: f(True), vb, note + ' (verblasst)')) for n, (f, vb, note) in VARIANTS.items()] \
        + [(n + '-bunt', (lambda f=f: f(False), vb, note + ' (farbig)')) for n, (f, vb, note) in VARIANTS.items()]
    for i, (name, (fn, _, note)) in enumerate(items):
        cx = cell * (i % cols) + cell / 2
        base = 70 + cell * (i // cols) + cell - 50
        s += f'<path d="M {cx - 90},{base} H {cx + 90}" stroke="#7d9a5a" stroke-width="6"/>'
        s += f'<g transform="translate({cx} {base})">{fn()}</g>'
        s += text(cx, base + 28, note, 13, TEXT)
    return s + '</svg>'


def export():
    items = dict(PROPS)
    for n, (f, vb, note) in VARIANTS.items():
        items[n + '-blass'] = (lambda f=f: f(True), vb, note + ', verblasst (E-277)')
        items[n + '-bunt'] = (lambda f=f: f(False), vb, note + ', farbig (E-277)')
    for name, (fn, vb, note) in items.items():
        with open(f'assets/map/decor/{name}.svg', 'w') as f:
            f.write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}">\n'
                    f'  <!-- {note} (A1.9, aus tools/design/tauwinkel_requisiten.py). Ursprung unten in der Mitte. -->\n'
                    f'  {fn()}\n</svg>\n')


if __name__ == '__main__':
    open('docs/release-2/design/tauwinkel-props.svg', 'w').write(sheet())
    export()

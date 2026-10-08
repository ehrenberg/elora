"""Generates docs/archive/release-1/design/elora-kartenlook.svg – three drafts for the map look (M6.0, E-130).

Usage: python3 tools/design/elora_kartenlook.py && cargo xtask svg-preview \
        docs/archive/release-1/design/elora-kartenlook.svg docs/archive/release-1/design/elora-kartenlook.png 1400

Each row shows the same scene: playing area with outer/inner corners, unhookable
stone (U), death (x), platform (~), ice (I), jump pad (^), booster (>),
decoration, background layers and Elora for size comparison.
"""
import math
import re

T = 28            # pixels per tile on the sheet (32 units in the game)
OUT = '#2b2b2b'
SCENE = [
    "..........................................",
    "...............~~~~~......................",
    "....######...............>>>>>>.......##..",
    "....######.........................####...",
    "..................IIII.............##.....",
    "........^......#########...........#......",
    "######################..#######xxxx#######",
    "######UUUU############..##################",
]
W_T, H_T = len(SCENE[0]), len(SCENE)
SW = W_T * T
SH = H_T * T + 2 * T   # air above the scene


def at(x, y):
    if 0 <= y < H_T and 0 <= x < W_T:
        return SCENE[y][x]
    return '#' if y >= H_T else '.'


SOLIDS = set('#U')


def solid(x, y):
    return at(x, y) in SOLIDS


def elora(x, y, s, body='f2c14e', feet='d9a43a'):
    src = open('assets/elora/elora.svg').read()
    inner = src[src.index('>', src.index('<svg')) + 1:src.rindex('</svg>')]
    inner = re.sub(r'<!--.*?-->', '', inner, flags=re.S)
    inner = inner.replace('#f2c14e', '#' + body).replace('#d9a43a', '#' + feet)
    return f'<g transform="translate({x},{y}) scale({s})">{inner}</g>'


def tile_path(x, y, r, same):
    """Rectangle of the tile with rounded outer corners (convex if both neighbours are free)."""
    px, py = x * T, y * T
    up, down = not same(x, y - 1), not same(x, y + 1)
    left, right = not same(x - 1, y), not same(x + 1, y)
    tl = r if up and left else 0
    tr = r if up and right else 0
    br = r if down and right else 0
    bl = r if down and left else 0
    return (f'M {px + tl},{py} H {px + T - tr} Q {px + T},{py} {px + T},{py + tr} '
            f'V {py + T - br} Q {px + T},{py + T} {px + T - br},{py + T} '
            f'H {px + bl} Q {px},{py + T} {px},{py + T - bl} V {py + tl} Q {px},{py} {px + tl},{py} Z')


# ── Styles ──────────────────────────────────────────────────────────────

STYLES = {
    'A': dict(title='A – Weich & lebendig', desc='runde Außenecken, Grasnarbe, Erde mit weichen Flecken, Büsche und Blumen',
              sky=('#a9cde8', '#e6f2f8'), earth='#a87a52', earth_dark='#8a6040', top='#7bbf55', top_dark='#5f9c42',
              stone='#7d8790', stone_dark='#5f6870', round=9, far='#b9d6e9', near='#9fc7a0'),
    'B': dict(title='B – Kristall', desc='facettierte Kanten, Glanzlinien, kühle Farben; Deko als Kristalle und Gräser',
              sky=('#9fc0e0', '#dde9f5'), earth='#6c6f9c', earth_dark='#545784', top='#9ad4e8', top_dark='#6fb3cc',
              stone='#4a4f63', stone_dark='#353a4c', round=0, far='#b3c8e6', near='#8fa7cf'),
    'C': dict(title='C – Scherenschnitt', desc='flache Papierlagen mit versetztem Schatten, warme Töne, Deko als ausgeschnittene Formen',
              sky=('#f4dcc2', '#fbefe1'), earth='#d98e5f', earth_dark='#c27849', top='#e9b46f', top_dark='#d39b55',
              stone='#9b8a82', stone_dark='#857470', round=4, far='#efc9a8', near='#e3ae86'),
}


def background(st, key):
    s = [f'<defs><linearGradient id="sky{key}" x1="0" y1="0" x2="0" y2="1">'
         f'<stop offset="0" stop-color="{st["sky"][0]}"/><stop offset="1" stop-color="{st["sky"][1]}"/></linearGradient></defs>',
         f'<rect x="0" y="{-2 * T}" width="{SW}" height="{SH}" fill="url(#sky{key})"/>']
    # far and near layer (parallax)
    if key == 'B':
        pts_far = ' '.join(f'{x},{50 + 30 * math.sin(x / 60) + (x * 7 % 23)}' for x in range(0, SW + 40, 40))
        s.append(f'<polygon points="0,{H_T * T} {pts_far} {SW},{H_T * T}" fill="{st["far"]}"/>')
        for i in range(9):
            x = 60 + i * 140
            s.append(f'<polygon points="{x},{H_T * T} {x + 30},{70 + (i * 37) % 50} {x + 60},{H_T * T}" fill="{st["near"]}" opacity="0.8"/>')
    elif key == 'C':
        for i, (yb, c) in enumerate([(70, st['far']), (110, st['near'])]):
            pts = ' '.join(f'{x},{yb + 18 * math.sin(x / (90 + 30 * i) + i)}' for x in range(0, SW + 20, 20))
            s.append(f'<polygon points="0,{H_T * T} {pts} {SW},{H_T * T}" fill="#000" opacity="0.08" transform="translate(4,4)"/>')
            s.append(f'<polygon points="0,{H_T * T} {pts} {SW},{H_T * T}" fill="{c}"/>')
    else:
        for cx, cy, r in [(150, 20, 22), (520, 5, 28), (900, 30, 20), (1100, 0, 24)]:
            for dx, rr in [(-r * 0.9, 0.7), (0, 1), (r * 0.9, 0.75)]:
                s.append(f'<circle cx="{cx + dx}" cy="{cy}" r="{r * rr}" fill="#ffffff" opacity="0.85"/>')
        s.append(f'<ellipse cx="{SW * 0.3}" cy="{H_T * T + 40}" rx="{SW * 0.4}" ry="170" fill="{st["far"]}"/>')
        s.append(f'<ellipse cx="{SW * 0.8}" cy="{H_T * T + 60}" rx="{SW * 0.35}" ry="150" fill="{st["near"]}"/>')
    return ''.join(s)


def terrain(st, key):
    s = []
    same = solid
    # shadow (C only)
    if key == 'C':
        for y in range(H_T):
            for x in range(W_T):
                if solid(x, y):
                    s.append(f'<path d="{tile_path(x, y, st["round"], same)}" fill="#000" opacity="0.18" transform="translate(5,5)"/>')
    for y in range(H_T):
        for x in range(W_T):
            c = at(x, y)
            if c not in SOLIDS:
                continue
            fill = st['stone'] if c == 'U' else st['earth']
            # thin border in fill colour covers seams between tiles
            s.append(f'<path d="{tile_path(x, y, st["round"], same)}" fill="{fill}" stroke="{fill}" stroke-width="0.8"/>')
            px, py = x * T, y * T
            if c == 'U':
                # stone: cracks or facets or hatching, respectively
                if key == 'A':
                    s.append(f'<path d="M {px + 6},{py + 8} l 6,5 l -2,7 M {px + 18},{py + 4} l -3,6 l 5,4" stroke="{st["stone_dark"]}" stroke-width="2" fill="none" stroke-linecap="round"/>')
                elif key == 'B':
                    s.append(f'<polygon points="{px},{py} {px + T},{py} {px + T / 2},{py + T / 2}" fill="{st["stone_dark"]}"/>')
                else:
                    for k in range(0, T, 7):
                        s.append(f'<line x1="{px + k}" y1="{py}" x2="{px}" y2="{py + k}" stroke="{st["stone_dark"]}" stroke-width="1.5"/>')
            else:
                if key == 'A' and (x * 7 + y * 3) % 5 == 0:
                    s.append(f'<ellipse cx="{px + 10 + (x * 13) % 9}" cy="{py + 14 + (y * 5) % 7}" rx="4" ry="3" fill="{st["earth_dark"]}"/>')
                if key == 'B' and (x + y) % 2 == 0:
                    s.append(f'<polygon points="{px},{py + T} {px + T},{py} {px + T},{py + T}" fill="{st["earth_dark"]}" opacity="0.6"/>')
            # top edge
            if not solid(x, y - 1) and c != 'U':
                if key == 'A':
                    wave = ' '.join(f'Q {px + i + 2},{py + 12} {px + i + 4},{py + 9}' for i in range(0, T, 4))
                    s.append(f'<path d="M {px},{py} H {px + T} V {py + 9} {wave} L {px},{py + 9} Z" fill="{st["top"]}"/>')
                    s.append(f'<rect x="{px}" y="{py}" width="{T}" height="3" fill="{st["top_dark"]}" opacity="0.5"/>')
                elif key == 'B':
                    s.append(f'<polygon points="{px},{py} {px + T},{py} {px + T - 6},{py + 8} {px + 6},{py + 8}" fill="{st["top"]}"/>')
                    s.append(f'<line x1="{px + 4}" y1="{py + 2}" x2="{px + T - 4}" y2="{py + 2}" stroke="#ffffff" stroke-width="1.5" opacity="0.7"/>')
                else:
                    s.append(f'<rect x="{px}" y="{py}" width="{T}" height="7" fill="{st["top"]}"/>')
    # outline (A, B)
    if key in 'AB':
        for y in range(H_T):
            for x in range(W_T):
                if solid(x, y):
                    px, py = x * T, y * T
                    for dx, dy, line in [(0, -1, (px, py, px + T, py)), (0, 1, (px, py + T, px + T, py + T)),
                                         (-1, 0, (px, py, px, py + T)), (1, 0, (px + T, py, px + T, py + T))]:
                        if not solid(x + dx, y + dy):
                            r = st['round']
                            x1, y1, x2, y2 = line
                            # shorten at rounded corners
                            s.append(f'<line x1="{x1 + (r if dy and not solid(x - 1, y) else 0)}" y1="{y1 + (r if dx and not solid(x, y - 1) else 0)}" '
                                     f'x2="{x2 - (r if dy and not solid(x + 1, y) else 0)}" y2="{y2 - (r if dx and not solid(x, y + 1) else 0)}" '
                                     f'stroke="{OUT}" stroke-width="2.5" stroke-linecap="round"/>')
    return ''.join(s)


def specials(st, key):
    s = []
    for y in range(H_T):
        for x in range(W_T):
            c = at(x, y)
            px, py = x * T, y * T
            if c == '~':      # platform
                fill = {'A': '#c9955c', 'B': '#cfe9f5', 'C': '#e9b46f'}[key]
                s.append(f'<rect x="{px}" y="{py}" width="{T}" height="9" rx="{3 if key != "B" else 0}" fill="{fill}" stroke="{OUT}" stroke-width="2"/>')
                if key == 'A':
                    s.append(f'<line x1="{px + T / 2}" y1="{py + 9}" x2="{px + T / 2}" y2="{py + 16}" stroke="{OUT}" stroke-width="2"/>')
            elif c == 'I':    # ice
                s.append(f'<rect x="{px}" y="{py}" width="{T}" height="{T}" fill="#bfe6f5" stroke="{OUT}" stroke-width="2"/>')
                s.append(f'<line x1="{px + 5}" y1="{py + 20}" x2="{px + 15}" y2="{py + 8}" stroke="#ffffff" stroke-width="3" stroke-linecap="round"/>')
            elif c == '^':    # jump pad (on the ground)
                s.append(f'<rect x="{px + 2}" y="{py + 18}" width="{T - 4}" height="10" rx="4" fill="#ef7fb0" stroke="{OUT}" stroke-width="2"/>')
                s.append(f'<path d="M {px + 8},{py + 16} L {px + T / 2},{py + 6} L {px + T - 8},{py + 16}" fill="none" stroke="{OUT}" stroke-width="2.5" stroke-linecap="round"/>')
            elif c == '>':    # booster
                s.append(f'<rect x="{px}" y="{py}" width="{T}" height="{T}" fill="#6c7a89" stroke="{OUT}" stroke-width="2"/>')
                s.append(f'<path d="M {px + 8},{py + 7} L {px + 18},{py + T / 2} L {px + 8},{py + T - 7}" fill="none" stroke="#f2c14e" stroke-width="3" stroke-linecap="round"/>')
            elif c == 'x':    # death
                col = {'A': '#c94f4f', 'B': '#ff5a6e', 'C': '#c0392b'}[key]
                for k in range(3):
                    bx = px + k * T / 3
                    s.append(f'<polygon points="{bx},{py + T} {bx + T / 6},{py + 6} {bx + T / 3},{py + T}" fill="{col}" stroke="{OUT}" stroke-width="1.5"/>')
    return ''.join(s)


def deko(st, key):
    s = []
    ground = 6 * T  # top edge of the ground
    if key == 'A':
        for x in [60, 280, 700, 1000]:
            s.append(f'<ellipse cx="{x}" cy="{ground - 8}" rx="22" ry="14" fill="#5f9c42" stroke="{OUT}" stroke-width="2"/>')
            s.append(f'<ellipse cx="{x + 16}" cy="{ground - 12}" rx="14" ry="10" fill="#7bbf55" stroke="{OUT}" stroke-width="2"/>')
        for x in [160, 420, 860, 1120]:
            s.append(f'<line x1="{x}" y1="{ground}" x2="{x}" y2="{ground - 14}" stroke="#4f8a3a" stroke-width="2"/>')
            s.append(f'<circle cx="{x}" cy="{ground - 16}" r="5" fill="#ef7fb0" stroke="{OUT}" stroke-width="1.5"/>')
    elif key == 'B':
        for x in [80, 300, 720, 1050]:
            s.append(f'<polygon points="{x},{ground} {x + 8},{ground - 26} {x + 16},{ground}" fill="#9ad4e8" stroke="{OUT}" stroke-width="2"/>')
            s.append(f'<polygon points="{x + 12},{ground} {x + 22},{ground - 16} {x + 28},{ground}" fill="#c6a6ef" stroke="{OUT}" stroke-width="2"/>')
    else:
        for x in [90, 330, 760, 1040]:
            s.append(f'<path d="M {x},{ground} C {x - 10},{ground - 30} {x + 30},{ground - 40} {x + 22},{ground}" fill="#000" opacity="0.15" transform="translate(3,3)"/>')
            s.append(f'<path d="M {x},{ground} C {x - 10},{ground - 30} {x + 30},{ground - 40} {x + 22},{ground}" fill="#a8c47a"/>')
    return ''.join(s)


def labels():
    items = [('Plattform', 15, 1), ('Beschleuniger', 25, 2), ('Eis', 18, 4), ('Sprungfeld', 8, 5), ('Tod', 31, 6), ('nicht hookbar', 6, 7)]
    s = []
    for text, x, y in items:
        s.append(f'<text x="{x * T}" y="{y * T - 4}" font-size="11" fill="#1e2a36" opacity="0.8">{text}</text>')
    return ''.join(s)


def sheet():
    total_h = 90 + 3 * (SH + 50)
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{SW + 60}" height="{total_h}" viewBox="0 0 {SW + 60} {total_h}" font-family="Inter">',
           f'<rect width="{SW + 60}" height="{total_h}" fill="#d8e3ec"/>',
           '<text x="30" y="44" font-size="28" fill="#1e2a36" font-weight="bold">Kartenlook – drei Entwürfe (M6.0, E-130)</text>',
           '<text x="30" y="70" font-size="14" fill="#1e2a36">Dieselbe Szene je Stil; Spielfläche aus Kacheln mit automatischen Kanten, dazu Deko und Hintergrund-Ebenen. Elora zum Größenvergleich.</text>']
    for i, key in enumerate('ABC'):
        st = STYLES[key]
        oy = 90 + i * (SH + 50)
        out.append(f'<text x="30" y="{oy + 22}" font-size="20" fill="#1e2a36" font-weight="bold">{st["title"].replace("&", "&amp;")}</text>')
        out.append(f'<text x="330" y="{oy + 22}" font-size="13" fill="#1e2a36">{st["desc"]}</text>')
        out.append(f'<clipPath id="clip{key}"><rect x="0" y="{-2 * T}" width="{SW}" height="{SH}" rx="10"/></clipPath>')
        out.append(f'<g transform="translate(30,{oy + 34 + 2 * T})"><g clip-path="url(#clip{key})">')
        out.append(background(st, key))
        out.append(deko(st, key))
        out.append(terrain(st, key))
        out.append(specials(st, key))
        scale = 0.36 * T / 32  # as in the game
        out.append(elora(13 * T, 6 * T, scale))
        out.append(elora(17 * T, 1 * T, scale, body='5aaee8', feet='6a78e0'))
        out.append(labels())
        out.append('</g></g>')
    out.append('</svg>')
    return '\n'.join(out)


if __name__ == '__main__':
    with open('docs/archive/release-1/design/elora-kartenlook.svg', 'w') as fh:
        fh.write(sheet())

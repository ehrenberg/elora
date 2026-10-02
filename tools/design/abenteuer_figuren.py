"""Erzeugt docs/release-2/design/abenteuer-figuren.svg – NPCs, Gegner und Objekte (R2-M1, A1.0).

Aufruf: python3 tools/design/abenteuer_figuren.py && cargo xtask svg-preview \
        docs/release-2/design/abenteuer-figuren.svg docs/release-2/design/abenteuer-figuren.png 1400

Alle Figuren nutzen Eloras Tropfenform (assets/elora/elora.svg) und dieselbe Linienstärke;
Zubehör liegt in Eloras Koordinaten (Ursprung = Bodenkontakt, Blick nach rechts).
"""
import math
import re

OUT = '#2b2b2b'
TEXT = '#3b3024'
DIM = '#8a7a66'


def lighten(c, a):
    v = [int(c[i:i + 2], 16) for i in (0, 2, 4)]
    return ''.join(f'{round(x + (255 - x) * a):02x}' for x in v)


def darken(c, a):
    v = [int(c[i:i + 2], 16) for i in (0, 2, 4)]
    return ''.join(f'{round(x * (1 - a)):02x}' for x in v)


def drop(body, feet, eyes='2b2b2b', mouth=True):
    src = open('assets/elora/elora.svg').read()
    inner = src[src.index('>', src.index('<svg')) + 1:src.rindex('</svg>')]
    inner = re.sub(r'<!--.*?-->', '', inner, flags=re.S)
    if not mouth:
        inner = re.sub(r'<path d="M 10,-41[^>]*/>', '', inner)

    def recolor(m):
        tag = m.group(0)
        key = re.search(r'id="tint-(\d)(-l(\d+))?', tag)
        if not key:
            return tag
        c = {'1': eyes, '2': body, '3': feet}[key.group(1)]
        if key.group(3):
            c = lighten(c, int(key.group(3)) / 100)
        return re.sub(r'fill="#\w+"', f'fill="#{c}"', tag)
    return re.sub(r'<(ellipse|path|circle)[^>]*>', recolor, inner)


def g(x, y, s, inner, flip=False):
    return f'<g transform="translate({x},{y}) scale({-s if flip else s},{s})">{inner}</g>'


def text(x, y, t, size=14, color=TEXT, anchor='middle', weight='normal', style='normal'):
    return (f'<text x="{x}" y="{y}" font-size="{size}" fill="{color}" text-anchor="{anchor}" '
            f'font-weight="{weight}" font-style="{style}">{t}</text>')


def star(cx, cy, r, color='#ffffff', stroke=OUT, sw=1.5):
    pts = []
    for i in range(8):
        a = math.pi / 4 * i - math.pi / 2
        rr = r if i % 2 == 0 else r * 0.38
        pts.append(f'{cx + rr * math.cos(a):.1f},{cy + rr * math.sin(a):.1f}')
    return f'<polygon points="{" ".join(pts)}" fill="{color}" stroke="{stroke}" stroke-width="{sw}" stroke-linejoin="round"/>'


# ── NPCs ────────────────────────────────────────────────────────────────

def oma_pfuetze():
    body = 'b9a3e3'
    s = '<path d="M 58,-4 L 62,-74 Q 64,-90 50,-88" fill="none" stroke="#2b2b2b" stroke-width="10" stroke-linecap="round"/>'
    s += '<path d="M 58,-4 L 62,-74 Q 64,-90 50,-88" fill="none" stroke="#a8744a" stroke-width="5" stroke-linecap="round"/>'
    s += drop(body, '8a6fb8', mouth=False)
    s += '<circle cx="14" cy="-128" r="12" fill="#e9e6ef" stroke="#2b2b2b" stroke-width="4"/>'
    # halb geschlossene Lider
    for ex in (2, 26):
        s += f'<path d="M {ex - 7.5},-63 A 7.5,11 0 0 1 {ex + 7.5},-63 Z" fill="#{body}" stroke="#2b2b2b" stroke-width="2.5"/>'
        s += f'<circle cx="{ex}" cy="-63" r="13" fill="#ffffff" fill-opacity="0.18" stroke="#2b2b2b" stroke-width="3"/>'
    s += '<path d="M 15,-64 Q 14,-67 13,-64" fill="none" stroke="#2b2b2b" stroke-width="3"/>'
    s += '<path d="M 10,-43 Q 16,-38 22,-43" fill="none" stroke="#2b2b2b" stroke-width="3" stroke-linecap="round"/>'
    # Schultertuch
    s += ('<path d="M -49,-50 Q 0,-26 49,-52 L 47,-36 Q 0,-12 -46,-34 Z" fill="#e8685a" stroke="#2b2b2b" '
          'stroke-width="4" stroke-linejoin="round"/>')
    for x in (-30, -12, 6, 24, 40):
        s += f'<circle cx="{x}" cy="{-36 + abs(x) * 0.08:.0f}" r="3" fill="#f6c8c0"/>'
    return s


def klonk():
    body, feet = 'a8744a', '6b4a32'
    s = drop(body, feet, mouth=False)
    # Schürze
    s += '<path d="M -40,-36 L -34,-84 M 44,-36 L 40,-80" fill="none" stroke="#2b2b2b" stroke-width="4"/>'
    s += ('<path d="M -42,-36 L 46,-36 L 46,-18 Q 4,-6 -44,-18 Z" fill="#5a5a5a" stroke="#2b2b2b" '
          'stroke-width="4" stroke-linejoin="round"/>')
    s += '<rect x="-6" y="-32" width="22" height="12" rx="3" fill="#4a4a4a" stroke="#2b2b2b" stroke-width="2.5"/>'
    # buschige Brauen und Bart
    s += '<path d="M -10,-82 L 12,-76" stroke="#2b2b2b" stroke-width="7" stroke-linecap="round"/>'
    s += '<path d="M 18,-76 L 38,-82" stroke="#2b2b2b" stroke-width="7" stroke-linecap="round"/>'
    s += '<path d="M -4,-46 Q 14,-58 32,-46 Q 26,-38 14,-42 Q 2,-38 -4,-46 Z" fill="#4a3424" stroke="#2b2b2b" stroke-width="3"/>'
    # Schmiedehammer
    s += '<path d="M 46,-14 L 66,-78" stroke="#2b2b2b" stroke-width="10" stroke-linecap="round"/>'
    s += '<path d="M 46,-14 L 66,-78" stroke="#8a6a4a" stroke-width="5" stroke-linecap="round"/>'
    s += '<rect x="50" y="-98" width="34" height="22" rx="5" fill="#c8ced6" stroke="#2b2b2b" stroke-width="4" transform="rotate(17 67 -87)"/>'
    return s


def lotte():
    body, feet = '8fd0f0', '5aaee8'
    s = '<path d="M -40,-58 L 22,-14" stroke="#2b2b2b" stroke-width="4"/>'
    s += drop(body, feet)
    # Umhängetasche
    s += '<path d="M -40,-86 L -52,-50" stroke="#a8744a" stroke-width="6" stroke-linecap="round"/>'
    s += '<rect x="-74" y="-52" width="40" height="32" rx="8" fill="#e0c89a" stroke="#2b2b2b" stroke-width="4"/>'
    s += '<path d="M -74,-42 L -34,-42" stroke="#2b2b2b" stroke-width="3"/>'
    s += star(-54, -30, 7, '#f2c14e')
    # Glitzerkette
    for i in range(7):
        x = -26 + i * 9.5
        s += f'<circle cx="{x:.1f}" cy="{-30 + (abs(x - 2) ** 2) * 0.008:.1f}" r="3.6" fill="#{["ef7fb0", "f2c14e", "a77be0"][i % 3]}" stroke="#2b2b2b" stroke-width="1.5"/>'
    # Sonnenhut mit Blume
    s += '<ellipse cx="10" cy="-104" rx="46" ry="9" fill="#f0ece4" stroke="#2b2b2b" stroke-width="4"/>'
    s += '<path d="M -16,-106 Q -14,-142 14,-142 Q 38,-142 38,-106 Z" fill="#f0ece4" stroke="#2b2b2b" stroke-width="4" stroke-linejoin="round"/>'
    s += '<path d="M -15,-114 Q 12,-108 37,-114 L 37,-106 Q 12,-100 -15,-106 Z" fill="#ef7fb0"/>'
    for a in range(5):
        r = math.radians(a * 72)
        s += f'<circle cx="{30 + 6 * math.cos(r):.1f}" cy="{-122 + 6 * math.sin(r):.1f}" r="5" fill="#f2c14e" stroke="#2b2b2b" stroke-width="1.5"/>'
    s += '<circle cx="30" cy="-122" r="3.5" fill="#e8685a"/>'
    s += star(66, -96, 8) + star(-64, -94, 6)
    return s


def tueftel():
    body, feet = '7fd99a', '3fc1b0'
    s = '<path d="M 19,-128 q 10,-6 0,-12 q -10,-6 0,-12 q 10,-6 0,-12" fill="none" stroke="#2b2b2b" stroke-width="3.5"/>'
    s += '<circle cx="19" cy="-168" r="7" fill="#f2c14e" stroke="#2b2b2b" stroke-width="3"/>'
    s += drop(body, feet, mouth=False)
    s += '<path d="M 10,-41 Q 16,-37 24,-43" fill="none" stroke="#2b2b2b" stroke-width="3" stroke-linecap="round"/>'
    s += '<path d="M 21,-42 Q 23,-36 26,-41" fill="#e8685a" stroke="#2b2b2b" stroke-width="2"/>'
    # Schutzbrille auf der Stirn
    s += '<path d="M -36,-94 Q 6,-104 42,-90" fill="none" stroke="#6b4a32" stroke-width="7"/>'
    for ex in (4, 28):
        s += f'<circle cx="{ex}" cy="-96" r="11" fill="#bfe6f5" stroke="#2b2b2b" stroke-width="4"/>'
        s += f'<path d="M {ex - 5},-99 Q {ex - 2},-103 {ex + 2},-102" fill="none" stroke="#ffffff" stroke-width="2.5"/>'
    # Schraubenschlüssel
    s += '<path d="M 44,-20 L 62,-62" stroke="#2b2b2b" stroke-width="10" stroke-linecap="round"/>'
    s += '<path d="M 44,-20 L 62,-62" stroke="#c8ced6" stroke-width="5" stroke-linecap="round"/>'
    s += '<path d="M 56,-62 a 11,11 0 1 1 14,4 l -4,-9 l -6,3 Z" fill="#c8ced6" stroke="#2b2b2b" stroke-width="3.5" stroke-linejoin="round"/>'
    # Ölfleck
    s += '<ellipse cx="-24" cy="-62" rx="7" ry="5" fill="#3fa090" opacity="0.7"/>'
    return s


def pip():
    body, feet = 'f28c3a', 'd94a4a'
    s = drop(body, feet)
    for ex in (2, 26):
        s += f'<circle cx="{ex - 2}" cy="-58" r="1.8" fill="#ffffff"/>'
    # verkehrt herum getragene Mütze
    s += '<path d="M -18,-104 Q -4,-146 36,-112 Z" fill="#5aaee8" stroke="#2b2b2b" stroke-width="4" stroke-linejoin="round"/>'
    s += '<path d="M -18,-104 Q -40,-104 -44,-96 Q -28,-92 -10,-100" fill="#4a8ec8" stroke="#2b2b2b" stroke-width="4" stroke-linejoin="round"/>'
    s += '<circle cx="14" cy="-128" r="4" fill="#f0ece4" stroke="#2b2b2b" stroke-width="2"/>'
    # Pflaster
    s += '<rect x="30" y="-58" width="16" height="8" rx="3" fill="#f0ece4" stroke="#2b2b2b" stroke-width="2" transform="rotate(-25 38 -54)"/>'
    s += '<circle cx="-36" cy="-60" r="5" fill="#e8685a" opacity="0.35"/>'
    return s


# ── Gegner ──────────────────────────────────────────────────────────────

def angry_eye(x, y, r=7, brow=1):
    return (f'<ellipse cx="{x}" cy="{y}" rx="{r}" ry="{r * 1.15:.1f}" fill="#ffffff" stroke="#2b2b2b" stroke-width="2.5"/>'
            f'<circle cx="{x + r * 0.3:.1f}" cy="{y + 1}" r="{r * 0.5:.1f}" fill="#2b2b2b"/>'
            f'<path d="M {x - r * 1.2:.1f},{y - r * 1.1 - 3 * brow:.1f} L {x + r * 1.1:.1f},{y - r * 0.7 + 2 * brow:.1f}" '
            f'stroke="#2b2b2b" stroke-width="4" stroke-linecap="round"/>')


def stachelkaefer():
    s = ''
    for x in (-24, 0, 24):
        s += f'<path d="M {x},-14 l -6,12 l -6,0" fill="none" stroke="#2b2b2b" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"/>'
    s += '<path d="M 44,-30 q 10,-24 24,-26" fill="none" stroke="#2b2b2b" stroke-width="3.5"/><circle cx="68" cy="-56" r="4" fill="#2b2b2b"/>'
    s += '<path d="M 48,-28 q 18,-14 30,-10" fill="none" stroke="#2b2b2b" stroke-width="3.5"/><circle cx="78" cy="-18" r="4" fill="#2b2b2b"/>'
    # Stacheln auf dem Panzer
    for i in range(6):
        a = math.radians(200 + i * 28)
        bx, by = 46 * math.cos(a), -14 + 38 * math.sin(a)
        tx, ty = 64 * math.cos(a), -14 + 56 * math.sin(a)
        nx, ny = -math.sin(a) * 8, math.cos(a) * 8
        s += (f'<path d="M {bx - nx:.1f},{by - ny:.1f} L {tx:.1f},{ty:.1f} L {bx + nx:.1f},{by + ny:.1f} Z" '
              'fill="#f0e0c0" stroke="#2b2b2b" stroke-width="3" stroke-linejoin="round"/>')
    s += '<path d="M -48,-14 A 48,40 0 0 1 48,-14 Z" fill="#c94a4a" stroke="#2b2b2b" stroke-width="5" stroke-linejoin="round"/>'
    s += '<path d="M 0,-54 L 0,-14" stroke="#2b2b2b" stroke-width="3"/>'
    for cx, cy in ((-24, -30), (22, -34), (-14, -46), (14, -22)):
        s += f'<circle cx="{cx}" cy="{cy}" r="5" fill="#2b2b2b" opacity="0.75"/>'
    s += '<path d="M -36,-36 Q -26,-48 -12,-50" fill="none" stroke="#ffffff" stroke-width="4" stroke-linecap="round" opacity="0.5"/>'
    s += '<circle cx="50" cy="-22" r="20" fill="#3b3024" stroke="#2b2b2b" stroke-width="4"/>'
    s += angry_eye(56, -26, 8)
    return s


def pollenblaeser(shots=True):
    s = '<path d="M 0,0 Q -6,-40 0,-70" fill="none" stroke="#2b2b2b" stroke-width="10"/>'
    s += '<path d="M 0,0 Q -6,-40 0,-70" fill="none" stroke="#4f9a3a" stroke-width="5"/>'
    s += '<path d="M -3,-26 Q -36,-40 -40,-18 Q -20,-12 -3,-26 Z" fill="#6cbf4a" stroke="#2b2b2b" stroke-width="3.5"/>'
    s += '<path d="M -1,-40 Q 30,-58 36,-34 Q 16,-28 -1,-40 Z" fill="#6cbf4a" stroke="#2b2b2b" stroke-width="3.5"/>'
    cx, cy = 0, -96
    for i in range(8):
        a = math.radians(i * 45 + 22)
        px, py = cx + 30 * math.cos(a), cy + 30 * math.sin(a)
        s += f'<ellipse cx="{px:.1f}" cy="{py:.1f}" rx="16" ry="10" transform="rotate({i * 45 + 22} {px:.1f} {py:.1f})" fill="#ef7fb0" stroke="#2b2b2b" stroke-width="3.5"/>'
    s += f'<circle cx="{cx}" cy="{cy}" r="24" fill="#f2c14e" stroke="#2b2b2b" stroke-width="4"/>'
    for dx, dy in ((-12, -10), (10, 12), (-8, 14), (14, -12)):
        s += f'<circle cx="{cx + dx}" cy="{cy + dy}" r="2" fill="#c89a2a"/>'
    s += angry_eye(cx - 8, cy - 4, 5) + angry_eye(cx + 8, cy - 4, 5)
    # Blasrohr nach rechts
    s += f'<path d="M {cx + 10},{cy + 4} L {cx + 44},{cy - 2} L {cx + 50},{cy - 12} L {cx + 52},{cy + 14} L {cx + 44},{cy + 8} L {cx + 10},{cy + 12} Z" fill="#e0c89a" stroke="#2b2b2b" stroke-width="3.5" stroke-linejoin="round"/>'
    # Pollenkugeln (Geschoss)
    for i, (px, r) in enumerate(((84, 9), (116, 7)) if shots else ()):
        py = cy + 2 - i * 3
        s += f'<circle cx="{px}" cy="{py}" r="{r + 5}" fill="#f2c14e" opacity="0.35"/>'
        s += f'<circle cx="{px}" cy="{py}" r="{r}" fill="#f8dd6e" stroke="#2b2b2b" stroke-width="2.5"/>'
        for k in range(6):
            a = math.radians(k * 60)
            s += f'<circle cx="{px + r * 0.5 * math.cos(a):.1f}" cy="{py + r * 0.5 * math.sin(a):.1f}" r="1.2" fill="#c89a2a"/>'
    return s


def grashuepfer(jump=False, arc=True):
    s = ''
    if jump and arc:
        s += '<path d="M -80,30 Q -40,-60 10,-30" fill="none" stroke="#8a7a66" stroke-width="3" stroke-dasharray="6 7"/>'
    # Hinterbein
    leg = 'M -20,-34 L -46,-62 L -54,-6 L -36,-4' if not jump else 'M -20,-34 L -60,-46 L -86,-20 L -96,-28'
    s += f'<path d="{leg}" fill="none" stroke="#2b2b2b" stroke-width="11" stroke-linejoin="round" stroke-linecap="round"/>'
    s += f'<path d="{leg}" fill="none" stroke="#5c9a32" stroke-width="6" stroke-linejoin="round" stroke-linecap="round"/>'
    for x in (6, 22):
        y1 = -4 if not jump else -12
        s += f'<path d="M {x},-20 L {x + 4},{y1}" stroke="#2b2b2b" stroke-width="4" stroke-linecap="round"/>'
    s += '<ellipse cx="0" cy="-34" rx="36" ry="22" fill="#8fcf5a" stroke="#2b2b2b" stroke-width="4.5"/>'
    s += '<path d="M -26,-26 Q 0,-16 24,-26" fill="none" stroke="#6cae3a" stroke-width="3"/>'
    s += '<path d="M -30,-44 Q -6,-62 16,-52" fill="#b8e08a" stroke="#2b2b2b" stroke-width="3"/>'
    s += '<circle cx="32" cy="-50" r="18" fill="#8fcf5a" stroke="#2b2b2b" stroke-width="4.5"/>'
    s += '<path d="M 30,-66 Q 34,-92 54,-96" fill="none" stroke="#2b2b2b" stroke-width="3"/>'
    s += '<path d="M 38,-66 Q 48,-86 66,-84" fill="none" stroke="#2b2b2b" stroke-width="3"/>'
    s += angry_eye(38, -54, 7, brow=0)
    s += '<path d="M 36,-40 Q 42,-36 48,-42" fill="none" stroke="#2b2b2b" stroke-width="3" stroke-linecap="round"/>'
    return s


# ── Objekte ─────────────────────────────────────────────────────────────

def truhe(open_=False):
    s = '<rect x="-34" y="-40" width="68" height="40" rx="6" fill="#a8744a" stroke="#2b2b2b" stroke-width="4"/>'
    s += '<rect x="-34" y="-28" width="68" height="7" fill="#e0b85a" stroke="#2b2b2b" stroke-width="2.5"/>'
    if open_:
        s = ('<path d="M -20,-44 L -40,-110 M 0,-44 L 0,-118 M 20,-44 L 40,-110" stroke="#f2c14e" stroke-width="7" '
             'stroke-linecap="round" opacity="0.5"/>') + s
        s += '<path d="M -34,-40 L -40,-74 Q 0,-90 40,-74 L 34,-40" fill="#8a5a36" stroke="#2b2b2b" stroke-width="4" stroke-linejoin="round"/>'
        s += star(-10, -62, 9, '#f8dd6e') + star(14, -70, 6, '#ffffff')
    else:
        s += '<path d="M -34,-40 Q -34,-66 0,-66 Q 34,-66 34,-40 Z" fill="#8a5a36" stroke="#2b2b2b" stroke-width="4"/>'
        s += '<rect x="-7" y="-46" width="14" height="16" rx="3" fill="#e0b85a" stroke="#2b2b2b" stroke-width="2.5"/>'
    return s


def glanztropfen(scale=1.0):
    d = 'M 0,-30 C 3,-20 14,-12 14,-2 C 14,7 7,13 0,13 C -7,13 -14,7 -14,-2 C -14,-12 -3,-20 0,-30 Z'
    s = f'<circle cx="0" cy="-6" r="22" fill="#7fe3f0" opacity="0.3"/>'
    s += f'<path d="{d}" fill="#5ad0e8" stroke="#2b2b2b" stroke-width="3" stroke-linejoin="round"/>'
    s += '<path d="M -7,-6 Q -6,4 2,7" fill="none" stroke="#ffffff" stroke-width="3" stroke-linecap="round"/>'
    s += star(6, -10, 6, '#ffffff', '#ffffff', 0.5)
    return f'<g transform="scale({scale})">{s}</g>'


def quellstein(active):
    glow = '#5ad0e8' if active else '#9aa4ae'
    s = ''
    if active:
        s += '<ellipse cx="0" cy="-50" rx="46" ry="56" fill="#7fe3f0" opacity="0.28"/>'
    s += '<path d="M -30,0 L -26,-62 Q 0,-92 26,-62 L 30,0 Z" fill="#b8bfc6" stroke="#2b2b2b" stroke-width="4.5" stroke-linejoin="round"/>'
    s += '<path d="M -18,-10 L -16,-58" stroke="#ffffff" stroke-width="3" opacity="0.4"/>'
    s += f'<path d="M 0,-74 C 2,-66 10,-60 10,-52 C 10,-45 5,-40 0,-40 C -5,-40 -10,-45 -10,-52 C -10,-60 -2,-66 0,-74 Z" fill="{glow}" stroke="#2b2b2b" stroke-width="3"/>'
    s += f'<path d="M -16,-30 Q 0,-22 16,-30" fill="none" stroke="{glow}" stroke-width="4" stroke-linecap="round"/>'
    s += '<rect x="-38" y="-6" width="76" height="8" rx="4" fill="#8a949e" stroke="#2b2b2b" stroke-width="3"/>'
    if active:
        s += star(-30, -86, 6) + star(32, -74, 5)
    return s


def schalter(on):
    a = 30 if on else -30
    s = (f'<g transform="rotate({a} 0 -14)"><path d="M 0,-14 L 0,-56" stroke="#2b2b2b" stroke-width="10" stroke-linecap="round"/>'
         '<path d="M 0,-14 L 0,-56" stroke="#8a6a4a" stroke-width="5" stroke-linecap="round"/>'
         f'<circle cx="0" cy="-58" r="9" fill="#{"6cbf4a" if on else "e8685a"}" stroke="#2b2b2b" stroke-width="3.5"/></g>')
    s += '<path d="M -26,0 L -20,-20 L 20,-20 L 26,0 Z" fill="#9aa4ae" stroke="#2b2b2b" stroke-width="4" stroke-linejoin="round"/>'
    return s


def tuer():
    s = '<rect x="-30" y="-96" width="60" height="96" rx="6" fill="#a8744a" stroke="#2b2b2b" stroke-width="4.5"/>'
    for x in (-15, 0, 15):
        s += f'<path d="M {x},-92 L {x},-4" stroke="#6b4a32" stroke-width="3"/>'
    for y in (-76, -24):
        s += f'<rect x="-32" y="{y}" width="64" height="10" rx="3" fill="#7a828a" stroke="#2b2b2b" stroke-width="3"/>'
    s += '<circle cx="0" cy="-50" r="10" fill="#f2c14e" stroke="#2b2b2b" stroke-width="3"/>'
    s += '<path d="M 0,-54 L 0,-46" stroke="#2b2b2b" stroke-width="3" stroke-linecap="round"/>'
    return s


def glitzerstein():
    s = '<ellipse cx="0" cy="-20" rx="26" ry="26" fill="#ef7fb0" opacity="0.25"/>'
    s += ('<path d="M -16,-20 L -8,-36 L 8,-36 L 16,-20 L 0,0 Z" fill="#c9a0f0" stroke="#2b2b2b" '
          'stroke-width="3.5" stroke-linejoin="round"/>')
    s += '<path d="M -16,-20 L 16,-20 M -8,-36 L -4,-20 L 0,0 M 8,-36 L 4,-20" fill="none" stroke="#2b2b2b" stroke-width="2"/>'
    s += '<path d="M -6,-31 L -2,-24" stroke="#ffffff" stroke-width="3" stroke-linecap="round"/>'
    s += star(18, -36, 6)
    return s


def sheet():
    W, H = 1400, 1180
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
         '<defs><linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#a9cde8"/>'
         '<stop offset="1" stop-color="#eef4f8"/></linearGradient></defs>',
         f'<rect width="{W}" height="{H}" fill="url(#sky)"/>',
         text(40, 50, 'Abenteuer – Figuren &amp; Objekte (R2-M1, A1.0)', 28, TEXT, 'start', 'bold'),
         text(40, 78, 'Alle Bewohner sind Tropfenwesen wie Elora (gleiche Form, Linie und Augen), erkennbar an Farbe und Zubehör. '
              'Gegner: rund, bunt, deutlich böse geguckt – ohne Blut (E-211).', 15, TEXT, 'start')]

    def card(x, y, w, h, title):
        return (f'<rect x="{x + 3}" y="{y + 5}" width="{w}" height="{h}" rx="18" fill="#1e2a36" fill-opacity="0.10"/>'
                f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="18" fill="#fffaf0" stroke="#e7dcc8" stroke-width="1.5"/>'
                + text(x + 22, y + 32, title, 19, TEXT, 'start', 'bold'))

    def ground(x, y, w):
        return f'<rect x="{x}" y="{y}" width="{w}" height="6" rx="3" fill="#8fbf7a"/>'

    # NPCs
    o.append(card(30, 100, 1340, 390, 'NPCs in Tauwinkel'))
    npcs = [
        ('Oma Pfütze', 'Älteste · Hauptaufgaben', 'Haarknoten, Brille, Tuch, Stock', oma_pfuetze(), 0.95),
        ('Klonk', 'Schmied · Waffen-Ausbau', 'groß, Schürze, Brauen, Bart, Hammer', klonk(), 1.2),
        ('Lotte Lichtblau', 'Händlerin', 'Sonnenhut, Glitzerkette, Tasche', lotte(), 1.0),
        ('Tüftel', 'Erfinder · Fähigkeiten', 'Schutzbrille, Glühbirnen-Feder, Schlüssel', tueftel(), 1.0),
        ('Pip', 'kleiner Bruder · Nebenaufgaben', 'klein, Mütze verkehrt, Pflaster', pip(), 0.7),
    ]
    for i, (name, role, note, art, sc) in enumerate(npcs):
        cx = 150 + i * 268
        o.append(ground(cx - 100, 386, 200))
        o.append(g(cx, 386, sc, art))
        o.append(text(cx, 424, name, 17, TEXT, weight='bold'))
        o.append(text(cx, 446, role, 13, DIM))
        o.append(text(cx, 466, note, 12, DIM, style='italic'))
    o.append(g(1316, 386, 0.45, drop('f2c14e', 'd9a43a')))
    o.append(text(1316, 404, 'Elora', 11, DIM))

    # Gegner
    o.append(card(30, 510, 1340, 330, 'Gegner der Blütenwiesen'))
    foes = [
        (230, 'Stachelkäfer', 'läuft hin und her · Stacheln oben: nicht draufspringen', g(210, 760, 1.15, stachelkaefer())),
        (540, 'Pollenbläser', 'steht fest · schießt langsame Pollenkugeln', g(500, 760, 1.0, pollenblaeser())),
        (920, 'Gras-Hüpfer', 'wartet und springt Elora an', g(860, 760, 1.05, grashuepfer()) + g(1040, 720, 1.05, grashuepfer(True))),
    ]
    for x, name, note, art in foes:
        o.append(ground(x - 150, 760, 300 if x < 900 else 420))
        o.append(art)
        o.append(text(x, 794, name, 17, TEXT, weight='bold'))
        o.append(text(x, 816, note, 13, DIM))
    o.append(text(1215, 640, 'Ruhe / Sprung', 12, DIM))

    # Objekte
    o.append(card(30, 860, 1340, 290, 'Abenteuer-Objekte'))
    objs = [
        ('Truhe', 'zu / offen', g(110, 1060, 1.0, truhe()) + g(210, 1060, 1.0, truhe(True)), 160),
        ('Glanztropfen', 'Währung', g(350, 1050, 1.6, glanztropfen()), 350),
        ('Speicherpunkt „Quellstein“', 'aus / aktiv', g(500, 1060, 1.0, quellstein(False)) + g(610, 1060, 1.0, quellstein(True)), 555),
        ('Schalter', 'aus / an', g(750, 1060, 1.0, schalter(False)) + g(830, 1060, 1.0, schalter(True)), 790),
        ('Tür / Tor', 'öffnet per Schalter oder Aufgabe', g(980, 1060, 1.0, tuer()), 980),
        ('Glitzerstein', 'Sammelstück', g(1150, 1050, 1.5, glitzerstein()), 1150),
    ]
    for name, note, art, tx in objs:
        o.append(ground(tx - 90, 1060, 180))
        o.append(art)
        o.append(text(tx, 1094, name, 16, TEXT, weight='bold'))
        o.append(text(tx, 1114, note, 12, DIM))
    o.append(g(1290, 1060, 0.42, drop('f2c14e', 'd9a43a')))
    o.append(text(1290, 1080, 'Elora', 11, DIM))
    o.append('</svg>')
    return '\n'.join(o)


if __name__ == '__main__':
    import os
    os.makedirs('docs/release-2/design', exist_ok=True)
    open('docs/release-2/design/abenteuer-figuren.svg', 'w').write(sheet())

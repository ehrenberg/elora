"""Erzeugt docs/design/elora-emotes.svg – Vorschlag für 8 Emoticons und das Emote-Rad (M5.9, E-091).

Aufruf: python3 tools/design/elora_emotes.py && cargo xtask svg-preview \
        docs/design/elora-emotes.svg docs/design/elora-emotes.png 1260

Emoticons in Welteinheiten (Blase ≈ 24 breit), auf dem Blatt ×3.
"""
import math
import re

OUT = '#2b2b2b'


def bubble(symbol):
    return (f'<path d="M -12,-10 Q -12,-14 -8,-14 H 8 Q 12,-14 12,-10 V 6 Q 12,10 8,10 H 3 L 0,15 L -3,10 H -8 '
            f'Q -12,10 -12,6 Z" fill="#ffffff" stroke="{OUT}" stroke-width="1.4" stroke-linejoin="round"/>'
            + symbol)


EMOTES = [
    ('Herz', f'<path d="M 0,6 C -9,0 -9,-8 -4,-8 C -2,-8 0,-6 0,-4.5 C 0,-6 2,-8 4,-8 C 9,-8 9,0 0,6 Z" fill="#e05a7a" stroke="{OUT}" stroke-width="1"/>'),
    ('Lachen', f'<path d="M -6,-4 Q -4,-7 -2,-4 M 2,-4 Q 4,-7 6,-4" fill="none" stroke="{OUT}" stroke-width="1.4" stroke-linecap="round"/>'
               f'<path d="M -6,0 Q 0,8 6,0 Z" fill="#e05a7a" stroke="{OUT}" stroke-width="1.2" stroke-linejoin="round"/>'),
    ('Wut', f'<path d="M -7,-7 L -2,-4 M 7,-7 L 2,-4" stroke="{OUT}" stroke-width="1.6" stroke-linecap="round"/>'
            f'<circle cx="-4" cy="-1.5" r="1.4" fill="{OUT}"/><circle cx="4" cy="-1.5" r="1.4" fill="{OUT}"/>'
            f'<path d="M -4,5 Q 0,2 4,5" fill="none" stroke="{OUT}" stroke-width="1.4" stroke-linecap="round"/>'
            f'<path d="M 7,-11 l 1.5,2 l 2,-1.5 l -0.5,2.5 l 2.5,0.5 l -2.5,1 l 1,2.3 l -2.3,-1 l -0.9,2.4 l -0.6,-2.5 l -2.5,0.4 l 1.6,-2 l -1.8,-1.7 l 2.5,0.2 Z" fill="#e0574f"/>'),
    ('Traurig', f'<circle cx="-4" cy="-3" r="1.4" fill="{OUT}"/><circle cx="4" cy="-3" r="1.4" fill="{OUT}"/>'
                f'<path d="M -4,5 Q 0,1.5 4,5" fill="none" stroke="{OUT}" stroke-width="1.4" stroke-linecap="round"/>'
                f'<path d="M -5,0 Q -6.5,3 -5,4 Q -3.5,3 -5,0 Z" fill="#5aaee8"/>'),
    ('Staunen', f'<path d="M 0,-9 V 1" stroke="#e0b85a" stroke-width="3.2" stroke-linecap="round"/><circle cx="0" cy="6" r="1.8" fill="#e0b85a"/>'),
    ('Frage', f'<path d="M -3.5,-5 Q -3.5,-9 0,-9 Q 4,-9 4,-5.5 Q 4,-3 0.5,-1.5 V 1.5" fill="none" stroke="#6a78e0" stroke-width="2.6" stroke-linecap="round"/><circle cx="0.5" cy="6" r="1.8" fill="#6a78e0"/>'),
    ('GG', f'<text x="0" y="4" font-size="11" font-weight="bold" text-anchor="middle" fill="#6cbf4a" stroke="{OUT}" stroke-width="0.5">GG</text>'),
    ('Schlaf', f'<text x="-4" y="5" font-size="10" font-weight="bold" text-anchor="middle" fill="#a77be0">Z</text>'
               f'<text x="3" y="-1" font-size="7" font-weight="bold" text-anchor="middle" fill="#a77be0">z</text>'
               f'<text x="7" y="-6" font-size="5" font-weight="bold" text-anchor="middle" fill="#a77be0">z</text>'),
]


def elora(x, y, s):
    src = open('assets/elora/elora.svg').read()
    inner = src[src.index('>', src.index('<svg')) + 1:src.rindex('</svg>')]
    return f'<g transform="translate({x},{y}) scale({s})">{re.sub(r"<!--.*?-->", "", inner, flags=re.S)}</g>'


def sheet():
    W, H = 1260, 640
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
           f'<rect width="{W}" height="{H}" fill="#8fb8d9"/>',
           '<text x="30" y="44" font-size="28" fill="#1e2a36" font-weight="bold">Emotes – Vorschlag (M5.9, E-091)</text>',
           '<text x="30" y="70" font-size="15" fill="#1e2a36">8 Emoticons als Sprechblase über dem Kopf (×3 gezeigt). Rad: E halten, Maus in Richtung zeigen, loslassen.</text>']
    for i, (name, sym) in enumerate(EMOTES):
        x, y = 80 + i * 90, 150
        out.append(f'<g transform="translate({x},{y}) scale(3)">{bubble(sym)}</g>')
        out.append(f'<text x="{x}" y="{y + 70}" font-size="15" text-anchor="middle" fill="#1e2a36">{i + 1} {name}</text>')
    # Rad
    cx, cy, r = 280, 440, 130
    out.append(f'<circle cx="{cx}" cy="{cy}" r="{r + 40}" fill="#1e2a36" opacity="0.45"/>')
    out.append(f'<circle cx="{cx}" cy="{cy}" r="38" fill="#1e2a36" opacity="0.5"/>')
    for i, (name, sym) in enumerate(EMOTES):
        a = math.radians(-90 + i * 45)
        x, y = cx + r * math.cos(a), cy + r * math.sin(a)
        sel = i == 1
        if sel:
            a0, a1 = math.radians(-90 + i * 45 - 22.5), math.radians(-90 + i * 45 + 22.5)
            ro, ri = r + 40, 40
            out.append(f'<path d="M {cx + ri * math.cos(a0):.1f},{cy + ri * math.sin(a0):.1f} L {cx + ro * math.cos(a0):.1f},{cy + ro * math.sin(a0):.1f} '
                       f'A {ro},{ro} 0 0 1 {cx + ro * math.cos(a1):.1f},{cy + ro * math.sin(a1):.1f} L {cx + ri * math.cos(a1):.1f},{cy + ri * math.sin(a1):.1f} '
                       f'A {ri},{ri} 0 0 0 {cx + ri * math.cos(a0):.1f},{cy + ri * math.sin(a0):.1f} Z" fill="#ffffff" opacity="0.3"/>')
        out.append(f'<g transform="translate({x:.1f},{y:.1f}) scale({2.4 if sel else 2})">{bubble(sym)}</g>')
    out.append(f'<line x1="{cx + 28}" y1="{cy - 28}" x2="{cx + 60}" y2="{cy - 60}" stroke="#ffffff" stroke-width="3" stroke-linecap="round"/>')
    out.append(f'<circle cx="{cx + 60}" cy="{cy - 60}" r="6" fill="#ffffff"/>')
    out.append(f'<text x="{cx}" y="{cy + 5}" font-size="14" text-anchor="middle" fill="#ffffff">E</text>')
    # Beispiel im Spiel
    gx, gy = 900, 560
    out.append(f'<rect x="620" y="300" width="610" height="310" rx="16" fill="#ffffff" opacity="0.35"/>')
    out.append(f'<text x="640" y="330" font-size="18" fill="#1e2a36" font-weight="bold">Im Spiel (×2): Blase über dem Kopf, ca. 2 s</text>')
    out.append(f'<line x1="640" y1="{gy}" x2="1210" y2="{gy}" stroke="#5b6b7c" stroke-width="10"/>')
    out.append(elora(gx - 150, gy - 5, 0.72))
    out.append(f'<g transform="translate({gx - 150},{gy - 5 - 132}) scale(2)">{bubble(EMOTES[1][1])}</g>')
    out.append(elora(gx + 150, gy - 5, 0.72))
    out.append(f'<g transform="translate({gx + 150},{gy - 5 - 132}) scale(2)">{bubble(EMOTES[6][1])}</g>')
    out.append('</svg>')
    return '\n'.join(out)


if __name__ == '__main__':
    with open('docs/design/elora-emotes.svg', 'w') as fh:
        fh.write(sheet())

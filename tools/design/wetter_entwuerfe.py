"""Drafts for the weather (R2-W1, step W1.0, E-329 to E-336).

The same small scene (hill, house, tree, Elora) in all eight weathers, plus the shapes of the
weather particles and the warning before a lightning strike.

Usage: python3 tools/design/wetter_entwuerfe.py && cargo xtask svg-preview \
        docs/release-2/design/wetter-entwuerfe.svg docs/release-2/design/wetter-entwuerfe.png 1600
"""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(__file__))
from abenteuer_figuren import DIM, OUT, TEXT, text  # noqa: E402
from kapitel1_entwuerfe import card, elora  # noqa: E402

PW, PH = 360, 240  # size of one image


def st(w=3.0):
    return f'stroke="{OUT}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'


# mood per weather: sky top/bottom, hills far/near, ground, darkening (0..1), tint
MOODS = {
    'schoen': ('#8fc3ea', '#e8f3fa', '#a9cf9a', '#7fbf6a', '#8fbf5a', 0.0, None),
    'regen': ('#7d8a99', '#c3cbd3', '#93a89a', '#6f9a6a', '#6f9a52', 0.12, None),
    'gewitter': ('#3c4250', '#7c8494', '#6d7f78', '#55785a', '#557a45', 0.3, None),
    'nebel': ('#c9d1d6', '#e6eaec', '#c4cfc8', '#a8bfa6', '#8fae78', 0.0, None),
    'wind': ('#8fc3ea', '#e8f3fa', '#a9cf9a', '#7fbf6a', '#8fbf5a', 0.0, None),
    'sandsturm': ('#c9a86a', '#e8d2a0', '#d6b77a', '#c9a35e', '#d9b56e', 0.1, '#d9a85a'),
    'schnee': ('#b9c6d3', '#eef2f6', '#dfe8ee', '#f2f6f9', '#f6f9fb', 0.0, None),
    'schneesturm': ('#9aa8b6', '#dde4ea', '#cfd9e0', '#e8eef2', '#f2f6f9', 0.08, None),
}

NAMES = {
    'schoen': ('Schön', 'wie bisher'),
    'regen': ('Regen', 'schräge Tropfen nach dem Wind, Spritzer, Himmel grau'),
    'gewitter': ('Gewitter', 'dunkel, Blitze hellen das Bild auf; Warnung am Boden, dann Einschlag'),
    'nebel': ('Nebel', 'Schleier nach unten dichter, ferne Hügel verschwinden'),
    'wind': ('Wind', 'Blätter und Blüten wirbeln, der Baum wiegt sich'),
    'sandsturm': ('Sandsturm', 'Sandschleier waagerecht, Bild gelb-braun, Sicht kurz'),
    'schnee': ('Schnee', 'ruhige Flocken, Boden und Dächer weiß'),
    'schneesturm': ('Schneesturm', 'dichte schräge Flocken, weißer Schleier, Sicht kurz'),
}


def scene(kind, lean=0.0):
    """Base scene in the image (0..PW, 0..PH); `lean` tilts the tree (wind)."""
    top, bottom, far, near, ground_c, dark, tint = MOODS[kind]
    gid = f'sky-{kind}'
    s = (f'<defs><linearGradient id="{gid}" x1="0" y1="0" x2="0" y2="1">'
         f'<stop offset="0" stop-color="{top}"/><stop offset="1" stop-color="{bottom}"/>'
         f'</linearGradient></defs><rect width="{PW}" height="{PH}" fill="url(#{gid})"/>')
    if kind == 'schoen' or kind == 'wind':
        s += '<circle cx="300" cy="46" r="22" fill="#ffd27a"/><circle cx="300" cy="46" r="34" fill="#ffd27a" opacity="0.25"/>'
    s += f'<path d="M 0,170 Q 60,120 130,150 Q 200,110 270,145 Q 320,125 360,140 V 240 H 0 Z" fill="{far}"/>'
    s += f'<path d="M 0,190 Q 90,160 180,185 Q 270,165 360,180 V 240 H 0 Z" fill="{near}"/>'
    s += f'<rect y="200" width="{PW}" height="40" fill="{ground_c}" {st(0)}/>'
    s += f'<path d="M 0,200 H {PW}" stroke="{OUT}" stroke-width="3"/>'
    # house
    roof = '#f2f6f9' if kind.startswith('schnee') else '#c8483a'
    s += f'<rect x="40" y="150" width="70" height="50" fill="#f3e6cc" {st()}/>'
    s += f'<path d="M 32,152 L 75,118 L 118,152 Z" fill="{roof}" {st()}/>'
    s += f'<rect x="66" y="172" width="18" height="28" fill="#a87a52" {st(2.5)}/>'
    window = '#ffe9a0' if dark > 0.2 else '#bfe6f5'
    s += f'<rect x="88" y="160" width="14" height="12" fill="{window}" {st(2)}/>'
    # tree (sways in the wind around its base)
    crown = '#e8eef2' if kind.startswith('schnee') else '#5b9d42'
    s += (f'<g transform="rotate({lean} 280 200)"><path d="M 276,200 V 150" stroke="{OUT}" stroke-width="11"/>'
          f'<path d="M 276,200 V 150" stroke="#8a6040" stroke-width="6"/>'
          f'<circle cx="276" cy="132" r="30" fill="{crown}" {st()}/>'
          f'<circle cx="258" cy="146" r="20" fill="{crown}" {st()}/>'
          f'<circle cx="296" cy="146" r="20" fill="{crown}" {st()}/></g>')
    s += elora(185, 200, 0.42)
    if tint:
        s += f'<rect width="{PW}" height="{PH}" fill="{tint}" opacity="0.18"/>'
    if dark:
        s += f'<rect width="{PW}" height="{PH}" fill="#0e1420" opacity="{dark}"/>'
    return s


def rain(rng, n, wind, length, color='#dfe8f2', width=1.6):
    s = ''
    for _ in range(n):
        x, y = rng.uniform(-40, PW + 40), rng.uniform(-20, PH)
        s += f'<path d="M {x:.1f},{y:.1f} l {wind * length:.1f},{length}" stroke="{color}" stroke-width="{width}" stroke-linecap="round" opacity="0.8"/>'
    return s


def splashes(rng, n):
    s = ''
    for _ in range(n):
        x = rng.uniform(10, PW - 10)
        s += f'<path d="M {x - 5:.1f},199 q 2,-6 5,-1 q 3,-5 5,1" fill="none" stroke="#e6eef6" stroke-width="1.6"/>'
    return s


def flakes(rng, n, size, drift=0.0):
    s = ''
    for _ in range(n):
        x, y = rng.uniform(0, PW), rng.uniform(0, PH)
        r = rng.uniform(size * 0.6, size)
        if drift:
            s += f'<path d="M {x:.1f},{y:.1f} l {drift * r * 3:.1f},{r * 1.2:.1f}" stroke="#ffffff" stroke-width="{r:.1f}" stroke-linecap="round" opacity="0.9"/>'
        else:
            s += f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r:.1f}" fill="#ffffff" opacity="0.92"/>'
    return s


def leaf(x, y, a, color):
    return (f'<g transform="translate({x:.1f},{y:.1f}) rotate({a:.0f})">'
            f'<path d="M -6,0 Q 0,-5 6,0 Q 0,5 -6,0 Z" fill="{color}" stroke="{OUT}" stroke-width="1"/></g>')


def petal(x, y, a, color):
    return (f'<g transform="translate({x:.1f},{y:.1f}) rotate({a:.0f})">'
            f'<ellipse rx="5" ry="3" fill="{color}" stroke="{OUT}" stroke-width="0.8"/></g>')


def veil(color, top_opacity, bottom_opacity, gid):
    return (f'<defs><linearGradient id="{gid}" x1="0" y1="0" x2="0" y2="1">'
            f'<stop offset="0" stop-color="{color}" stop-opacity="{top_opacity}"/>'
            f'<stop offset="1" stop-color="{color}" stop-opacity="{bottom_opacity}"/></linearGradient></defs>'
            f'<rect width="{PW}" height="{PH}" fill="url(#{gid})"/>')


def bolt(x, y0, y1):
    pts = [(x, y0)]
    yy = y0
    rng = random.Random(7)
    while yy < y1:
        yy += rng.uniform(16, 26)
        pts.append((x + rng.uniform(-12, 12), min(yy, y1)))
    d = 'M ' + ' L '.join(f'{px:.0f},{py:.0f}' for px, py in pts)
    return (f'<path d="{d}" fill="none" stroke="#fff6c8" stroke-width="7" stroke-linejoin="round" opacity="0.5"/>'
            f'<path d="{d}" fill="none" stroke="#ffffff" stroke-width="3" stroke-linejoin="round"/>')


def warning(x, y):
    """Lightning warning: the ground glows, small sparks rise."""
    s = f'<ellipse cx="{x}" cy="{y}" rx="26" ry="6" fill="#fff0a0" opacity="0.45"/>'
    s += f'<ellipse cx="{x}" cy="{y}" rx="14" ry="3.5" fill="#fff6c8" opacity="0.8"/>'
    for dx, dy in ((-10, -8), (6, -14), (12, -6), (-2, -18)):
        s += f'<circle cx="{x + dx}" cy="{y + dy}" r="1.8" fill="#fff6c8"/>'
    return s


def panel(kind):
    rng = random.Random(hash(kind) & 0xffff)
    if kind == 'schoen':
        return scene(kind)
    if kind == 'regen':
        return scene(kind) + rain(rng, 90, 0.25, 16) + splashes(rng, 14)
    if kind == 'gewitter':
        s = scene(kind) + rain(rng, 120, 0.4, 18, '#c9d4e0')
        s += f'<rect width="{PW}" height="{PH}" fill="#eef4ff" opacity="0.18"/>'
        s += bolt(120, 0, 196) + '<circle cx="120" cy="198" r="10" fill="#fff6c8" opacity="0.7"/>'
        s += warning(250, 200)
        s += text(250, 225, 'Warnung', 10, '#fff6c8')
        return s
    if kind == 'nebel':
        s = scene(kind)
        s += veil('#eef1f3', 0.35, 0.85, 'fog-a')
        s += '<ellipse cx="90" cy="190" rx="120" ry="18" fill="#f4f6f7" opacity="0.6"/>'
        s += '<ellipse cx="290" cy="180" rx="110" ry="14" fill="#f4f6f7" opacity="0.5"/>'
        return s
    if kind == 'wind':
        s = scene(kind, lean=8)
        for _ in range(16):
            s += leaf(rng.uniform(0, PW), rng.uniform(30, 190), rng.uniform(0, 360), rng.choice(['#6cbf4a', '#a8c84a', '#d9a03a']))
        for _ in range(12):
            s += petal(rng.uniform(0, PW), rng.uniform(30, 190), rng.uniform(0, 360), rng.choice(['#ef7fb0', '#f8dd6e', '#ffffff']))
        for y in (60, 110, 150):
            s += f'<path d="M 20,{y} q 60,-14 120,0 t 120,0" fill="none" stroke="#ffffff" stroke-width="2" opacity="0.6"/>'
        return s
    if kind == 'sandsturm':
        s = scene(kind)
        s += veil('#d9b06a', 0.25, 0.65, 'sand-a')
        for _ in range(70):
            x, y = rng.uniform(-30, PW), rng.uniform(0, PH)
            s += f'<path d="M {x:.1f},{y:.1f} h {rng.uniform(14, 34):.1f}" stroke="#f0d79a" stroke-width="1.6" stroke-linecap="round" opacity="0.8"/>'
        return s
    if kind == 'schnee':
        return scene(kind) + flakes(rng, 70, 3.2)
    if kind == 'schneesturm':
        s = scene(kind)
        s += veil('#ffffff', 0.25, 0.6, 'snow-a')
        s += flakes(rng, 140, 3.0, drift=1.2)
        return s
    return scene(kind)


def particle_row(x0, y):
    s = ''
    items = [
        ('Regentropfen', f'<path d="M 0,-14 l 4,22" stroke="#7d9ab8" stroke-width="3" stroke-linecap="round"/>'),
        ('Spritzer', '<path d="M -8,0 q 3,-9 8,-2 q 5,-7 8,2" fill="none" stroke="#7d9ab8" stroke-width="2.5"/>'),
        ('Schneeflocke', '<circle r="5" fill="#ffffff" stroke="#9aa8b6" stroke-width="1.5"/>'),
        ('Blatt', leaf(0, 0, 30, '#6cbf4a')),
        ('Blütenblatt', petal(0, 0, -20, '#ef7fb0')),
        ('Sandkorn', '<path d="M -12,0 h 24" stroke="#c9a25e" stroke-width="3" stroke-linecap="round"/>'),
        ('Blitz-Warnung', '<g transform="translate(0,6)">' + warning(0, 0) + '</g>'),
        ('Einschlag', '<g transform="translate(0,-40)">' + bolt(0, 0, 40) + '</g><circle cy="0" r="8" fill="#fff6c8" opacity="0.8"/>'),
    ]
    for k, (name, art) in enumerate(items):
        x = x0 + k * 180
        s += f'<g transform="translate({x},{y}) scale(1.6)">{art}</g>'
        s += text(x, y + 46, name, 13, TEXT)
    return s


def sheet():
    W, H = 1600, 1290
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
         f'<rect width="{W}" height="{H}" fill="#eef2f5"/>',
         text(40, 50, 'Wetter – Entwürfe (R2-W1, W1.0)', 28, TEXT, 'start', 'bold'),
         text(40, 78, 'Dieselbe Szene in allen acht Wettern (E-333). Im Spiel bewegen sich Partikel, Blitze '
              'und Schleier; Abdunkeln und Tönung kommen aus dem Post-Shader.', 15, TEXT, 'start')]
    kinds = list(NAMES)
    for i, kind in enumerate(kinds):
        col, row = i % 4, i // 4
        x, y = 30 + col * 390, 100 + row * 330
        o.append(card(x, y, 375, 315, NAMES[kind][0]))
        o.append(f'<svg x="{x + 8}" y="{y + 38}" width="{PW}" height="{PH}" viewBox="0 0 {PW} {PH}">'
                 f'<clipPath id="c-{kind}"><rect width="{PW}" height="{PH}" rx="8"/></clipPath>'
                 f'<g clip-path="url(#c-{kind})">{panel(kind)}</g></svg>')
        o.append(text(x + 187, y + 298, NAMES[kind][1], 11, DIM))
    o.append(card(30, 770, 1540, 200, 'Partikel und Blitz'))
    o.append(particle_row(120, 870))
    o.append(card(30, 990, 1540, 270, 'Einstellung „Wetter“ (E-335)'))
    for k, (name, rngn, note) in enumerate((('voll', 90, 'alle Partikel, Blitze, Schleier'),
                                            ('sanft', 30, 'weniger Partikel, kein Aufblitzen'),
                                            ('aus', 0, 'kein Wetter zu sehen – die Wirkung im Abenteuer bleibt'))):
        x = 60 + k * 500
        rng = random.Random(3)
        o.append(f'<svg x="{x}" y="1030" width="{PW}" height="{PH * 0.75}" viewBox="0 0 {PW} {PH}">'
                 f'<clipPath id="s{k}"><rect width="{PW}" height="{PH}" rx="8"/></clipPath>'
                 f'<g clip-path="url(#s{k})">{scene("regen") if rngn else scene("schoen")}{rain(rng, rngn, 0.25, 16)}</g></svg>')
        o.append(text(x + 180, 1230, f'{name}: {note}', 13, TEXT))
    o.append('</svg>')
    return '\n'.join(o)


if __name__ == '__main__':
    os.makedirs('docs/release-2/design', exist_ok=True)
    open('docs/release-2/design/wetter-entwuerfe.svg', 'w').write(sheet())

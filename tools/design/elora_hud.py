"""Erzeugt docs/design/elora-hud.svg – drei HUD-Entwürfe (M5.8, E-090 „modern, am Fadenkreuz“).

Aufruf: python3 tools/design/elora_hud.py && cargo xtask svg-preview \
        docs/design/elora-hud.svg docs/design/elora-hud.png 1260

Jede Karte zeigt denselben Spielmoment: 7/10 Leben, 4/10 Rüstung, Granate mit 6/10 Munition,
DM mit 12 Punkten, Timer 3:24.
"""
import math
import re

OUT = '#2b2b2b'
HEALTH, ARMOR = '#e05a7a', '#e0b85a'
HP, AR, AMMO, MAXV = 7, 4, 6, 10
W, H = 400, 300  # Kartengröße (Bildschirmausschnitt)


def elora(x, y, s):
    src = open('assets/elora/elora.svg').read()
    inner = src[src.index('>', src.index('<svg')) + 1:src.rindex('</svg>')]
    return f'<g transform="translate({x},{y}) scale({s})">{re.sub(r"<!--.*?-->", "", inner, flags=re.S)}</g>'


def item(name, x, y, s, rot=0):
    src = open(f'assets/items/{name}.svg').read()
    inner = src[src.index('>', src.index('<svg')) + 1:src.rindex('</svg>')]
    return f'<g transform="translate({x},{y}) rotate({rot}) scale({s})">{re.sub(r"<!--.*?-->", "", inner, flags=re.S)}</g>'


def arc(cx, cy, r, a0, a1, color, width, opacity=1.0, cap='round'):
    x0, y0 = cx + r * math.cos(math.radians(a0)), cy + r * math.sin(math.radians(a0))
    x1, y1 = cx + r * math.cos(math.radians(a1)), cy + r * math.sin(math.radians(a1))
    large = 1 if abs(a1 - a0) > 180 else 0
    sweep = 1 if a1 > a0 else 0
    return (f'<path d="M {x0:.1f},{y0:.1f} A {r},{r} 0 {large} {sweep} {x1:.1f},{y1:.1f}" fill="none" '
            f'stroke="{color}" stroke-width="{width}" stroke-linecap="{cap}" opacity="{opacity}"/>')


def segmented_arc(cx, cy, r, a0, a1, value, color, width):
    out = []
    step = (a1 - a0) / MAXV
    for i in range(MAXV):
        s0, s1 = a0 + i * step + step * 0.12, a0 + (i + 1) * step - step * 0.12
        on = i < value
        out.append(arc(cx, cy, r, s0, s1, color if on else '#ffffff', width, 1.0 if on else 0.25, 'butt'))
    return ''.join(out)


def scene(ox, oy):
    """Gemeinsamer Hintergrund: Himmel, Boden, Elora, Fadenkreuz."""
    return (f'<rect x="{ox}" y="{oy}" width="{W}" height="{H}" rx="12" fill="#98bfdf"/>'
            f'<rect x="{ox}" y="{oy + 230}" width="{W}" height="70" fill="#5b6b7c"/>'
            f'<rect x="{ox}" y="{oy + 230}" width="{W}" height="4" fill="#2f3944"/>'
            + elora(ox + 110, oy + 230, 0.5)
            + item('grenade', ox + 112, oy + 205, 1.4, -18))


def crosshair(x, y):
    return (f'<circle cx="{x}" cy="{y}" r="8" fill="none" stroke="#ffffff" stroke-width="2"/>'
            f'<circle cx="{x}" cy="{y}" r="1.5" fill="#ffffff"/>')


def top_info(ox, oy):
    return (f'<rect x="{ox + W / 2 - 60}" y="{oy + 10}" width="120" height="28" rx="14" fill="#1e2a36" opacity="0.55"/>'
            f'<text x="{ox + W / 2}" y="{oy + 29}" font-size="15" text-anchor="middle" fill="#ffffff">3:24 · 12 Punkte</text>')


def variant_a(ox, oy):
    cx, cy = ox + 260, oy + 150
    s = scene(ox, oy) + crosshair(cx, cy) + top_info(ox, oy)
    s += segmented_arc(cx, cy, 30, 110, 250, HP, HEALTH, 5)
    s += segmented_arc(cx, cy, 30, 70, -70, AR, ARMOR, 5)
    for i in range(MAXV):
        on = i < AMMO
        s += f'<circle cx="{cx - 22 + i * 4.9:.1f}" cy="{cy + 44}" r="1.8" fill="{"#ffffff" if on else "#ffffff"}" opacity="{1 if on else 0.25}"/>'
    return s


def variant_b(ox, oy):
    cx, cy = ox + 260, oy + 150
    s = scene(ox, oy) + crosshair(cx, cy) + top_info(ox, oy)
    # unten mittig: Leiste mit Balken und Waffen
    bx, by = ox + W / 2 - 110, oy + 250
    s += f'<rect x="{bx}" y="{by}" width="220" height="40" rx="12" fill="#1e2a36" opacity="0.6"/>'
    s += f'<rect x="{bx + 12}" y="{by + 10}" width="90" height="8" rx="4" fill="#ffffff" opacity="0.2"/>'
    s += f'<rect x="{bx + 12}" y="{by + 10}" width="{90 * HP / MAXV}" height="8" rx="4" fill="{HEALTH}"/>'
    s += f'<rect x="{bx + 12}" y="{by + 23}" width="90" height="6" rx="3" fill="#ffffff" opacity="0.2"/>'
    s += f'<rect x="{bx + 12}" y="{by + 23}" width="{90 * AR / MAXV}" height="6" rx="3" fill="{ARMOR}"/>'
    for i, (name, sel) in enumerate([('hammer', False), ('grenade', True), ('laser', False)]):
        x = bx + 118 + i * 34
        if sel:
            s += f'<rect x="{x - 3}" y="{by + 5}" width="32" height="30" rx="8" fill="#ffffff" opacity="0.25"/>'
        s += item(name, x, by + 17, 0.68)
        if sel:
            s += f'<text x="{x + 13}" y="{by + 32}" font-size="10" text-anchor="middle" fill="#ffffff">{AMMO}</text>'
    return s


def variant_c(ox, oy):
    cx, cy = ox + 260, oy + 150
    s = scene(ox, oy) + crosshair(cx, cy) + top_info(ox, oy)
    # dünne durchgehende Bögen am Fadenkreuz, Munition als Ring-Anteil
    s += arc(cx, cy, 24, 120, 240, '#ffffff', 3, 0.25)
    s += arc(cx, cy, 24, 240 - 120 * HP / MAXV, 240, HEALTH, 3)
    s += arc(cx, cy, 24, 60, -60, '#ffffff', 3, 0.25)
    s += arc(cx, cy, 24, -60 + 120 * AR / MAXV, -60, ARMOR, 3)
    s += arc(cx, cy, 15, 20, 160, '#ffffff', 2, 0.25)
    s += arc(cx, cy, 15, 160 - 140 * AMMO / MAXV, 160, '#ffffff', 2)
    # unten mittig nur die Waffe mit Zahl
    s += f'<rect x="{ox + W / 2 - 38}" y="{oy + 254}" width="76" height="32" rx="10" fill="#1e2a36" opacity="0.55"/>'
    s += item('grenade', ox + W / 2 - 30, oy + 270, 0.7)
    s += f'<text x="{ox + W / 2 + 26}" y="{oy + 276}" font-size="15" text-anchor="middle" fill="#ffffff">{AMMO}</text>'
    return s


def sheet():
    SW, SH = 1290, 470
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{SW}" height="{SH}" viewBox="0 0 {SW} {SH}" font-family="Inter">',
           f'<rect width="{SW}" height="{SH}" fill="#8fb8d9"/>',
           '<text x="30" y="44" font-size="28" fill="#1e2a36" font-weight="bold">HUD – drei Entwürfe (M5.8, E-090)</text>',
           '<text x="30" y="70" font-size="15" fill="#1e2a36">Gleicher Moment: 7/10 Leben, 4/10 Rüstung, Granate 6/10, 12 Punkte, 3:24. Timer und Punkte oben mittig in allen Entwürfen.</text>']
    cards = [
        ('A – Segmente am Fadenkreuz', 'Leben links, Rüstung rechts, Munition als Punkte', variant_a),
        ('B – Leiste unten mittig', 'Balken + Waffenwahl; Fadenkreuz bleibt frei', variant_b),
        ('C – Feine Ringe + Waffe unten', 'dünne Bögen am Fadenkreuz, Waffe mit Zahl unten', variant_c),
    ]
    for i, (title, desc, fn) in enumerate(cards):
        ox, oy = 30 + i * 420, 130
        out.append(f'<text x="{ox}" y="{oy - 28}" font-size="20" fill="#1e2a36" font-weight="bold">{title}</text>')
        out.append(f'<text x="{ox}" y="{oy - 8}" font-size="14" fill="#1e2a36">{desc}</text>')
        out.append(fn(ox, oy))
    out.append('</svg>')
    return '\n'.join(out)


if __name__ == '__main__':
    with open('docs/design/elora-hud.svg', 'w') as fh:
        fh.write(sheet())

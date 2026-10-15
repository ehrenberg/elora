"""Drafts for the world map (R2-WM, step WM.0, E-370 to E-373).

Usage: python3 tools/design/world_map_drafts.py && cargo xtask svg-preview \
        docs/release-2/design/world-map-drafts.svg docs/release-2/design/world-map-drafts.png 1600

Two looks of the painted overview of the Tauland (A storybook parchment, B bright game colours),
both in the same example state: meadows and forest visited, Elora in the middle of forest-2,
desert started, frost peaks and gorge still under clouds. Below: markers, the detail window of
the current section and the full-screen frame (key M).
"""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(__file__))
from adventure_figures import DIM, OUT, TEXT, drop, g, star, text  # noqa: E402
from chapter1_drafts import card  # noqa: E402

# areas as in assets/adventure/worldmap.toml: id, position, colour, sections
AREAS = [
    ('tauwinkel', 'Tauwinkel', (0.5, 0.56), '#f2c14e', 0),
    ('meadow', 'Blütenwiesen', (0.24, 0.72), '#ef7fb0', 3),
    ('forest', 'Flüsterwald', (0.18, 0.32), '#6cbf4a', 3),
    ('desert', 'Glutsandwüste', (0.5, 0.18), '#e0b85a', 3),
    ('frost', 'Frostspitzen', (0.82, 0.3), '#8fd0f0', 3),
    ('gorge', 'Sternschlucht', (0.8, 0.74), '#a77be0', 3),
]
# example state: visited sections per area (0 = area unknown), current section
VISITED = {'meadow': 4, 'forest': 2, 'desert': 1}
HERE = ('forest', 1, 0.6)  # area, section index, fraction along it
FREED = {'meadow', 'forest'}

LOOKS = {
    'A': dict(name='A – Bilderbuch (Pergament, Tusche)', sea='#c9dfe0', sea_line='#9fbfc4',
              land='#efe3c2', land_edge='#b59c6e', ink='#6b5a43', region_mix=0.55, road='#a8946e',
              label='#4a3b28', bg='#f6eedb'),
    'B': dict(name='B – Leuchtend (Spielfarben, dunkle Kontur)', sea='#8ccfe8', sea_line='#5fb0d4',
              land='#bfe3a0', land_edge=OUT, ink=OUT, region_mix=0.25, road='#c9955c',
              label=OUT, bg='#dff1fa'),
}


def mix(c, white):
    v = [int(c[i:i + 2], 16) for i in (1, 3, 5)]
    return '#' + ''.join(f'{round(x + (255 - x) * white):02x}' for x in v)


def darker(c, k):
    v = [int(c[i:i + 2], 16) for i in (1, 3, 5)]
    return '#' + ''.join(f'{round(x * (1 - k)):02x}' for x in v)


def smooth_blob(cx, cy, r, seed, wob=0.18, n=48, squash=1.0):
    rnd = random.Random(seed)
    ph = [rnd.uniform(0, 6.3) for _ in range(3)]
    pts = []
    for i in range(n):
        a = 2 * math.pi * i / n
        k = 1 + wob * (0.6 * math.sin(3 * a + ph[0]) + 0.3 * math.sin(5 * a + ph[1]) + 0.2 * math.sin(7 * a + ph[2]))
        pts.append((cx + r * k * math.cos(a), cy + r * squash * k * math.sin(a)))
    d = f'M {pts[0][0]:.1f},{pts[0][1]:.1f} '
    for i in range(n):
        p0, p1 = pts[i], pts[(i + 1) % n]
        mid = ((p0[0] + p1[0]) / 2, (p0[1] + p1[1]) / 2)
        d += f'Q {p0[0]:.1f},{p0[1]:.1f} {mid[0]:.1f},{mid[1]:.1f} '
    return d + 'Z'


def cloud(cx, cy, s, opacity):
    puffs = [(-30, 6, 22), (-8, -10, 28), (20, -4, 24), (40, 8, 18), (6, 12, 24), (-24, 16, 16)]
    o = f'<g transform="translate({cx:.1f},{cy:.1f}) scale({s})" opacity="{opacity}">'
    o += ''.join(f'<circle cx="{x}" cy="{y + 3}" r="{r + 2}" fill="#9fb4c4"/>' for x, y, r in puffs)
    o += ''.join(f'<circle cx="{x}" cy="{y}" r="{r}" fill="#ffffff"/>' for x, y, r in puffs)
    return o + '</g>'


def section_paths(W, H, x0, y0):
    """Path pieces per area: a chain of three bends leading away from Tauwinkel, arena at the end."""
    hub = (x0 + W * 0.5, y0 + H * 0.56)
    out = {}
    for aid, _, (px, py), _, n in AREAS[1:]:
        c = (x0 + W * px, y0 + H * py)
        dx, dy = c[0] - hub[0], c[1] - hub[1]
        ln = math.hypot(dx, dy)
        ux, uy = dx / ln, dy / ln
        nx, ny = -uy, ux
        start = (c[0] - ux * 85, c[1] - uy * 85)
        pieces, p = [], start
        for k in range(n):
            bend = 16 if k % 2 == 0 else -16
            q = (p[0] + ux * 50 + nx * bend, p[1] + uy * 50 + ny * bend)
            pieces.append((p, q))
            p = q
        out[aid] = (start, pieces, (p[0] + ux * 30, p[1] + uy * 30))
    return hub, out


def icon_spring_stone(x, y, s=1.0):
    return (f'<g transform="translate({x},{y}) scale({s})"><path d="M -11,6 Q -13,-10 0,-13 Q 13,-10 11,6 Z" '
            f'fill="#9aa4ad" stroke="{OUT}" stroke-width="2.2"/><circle cx="0" cy="-3" r="4" fill="#7fe8d6" '
            f'stroke="{OUT}" stroke-width="1.2"/></g>')


def icon_quest(x, y, s=1.0):
    return f'<g transform="translate({x},{y}) scale({s})">{star(0, 0, 13, "#f2c14e", OUT, 2)}</g>'


def icon_spring(x, y, freed, s=1.0):
    col = '#5fc8e8' if freed else '#a3a3ab'
    o = f'<g transform="translate({x},{y}) scale({s})"><ellipse cx="0" cy="6" rx="13" ry="5" fill="{mix(col, 0.3)}" stroke="{OUT}" stroke-width="2"/>'
    o += f'<path d="M 0,-14 Q 9,-3 0,4 Q -9,-3 0,-14 Z" fill="{col}" stroke="{OUT}" stroke-width="2"/>'
    if freed:
        o += ''.join(f'<circle cx="{12 * math.cos(a):.1f}" cy="{-4 + 12 * math.sin(a):.1f}" r="1.8" fill="#fff2b0"/>'
                     for a in [k * math.pi / 3 for k in range(6)])
    return o + '</g>'


def icon_guardian(x, y, beaten, s=1.0):
    col = '#8fd06a' if beaten else '#e8685a'
    o = f'<g transform="translate({x},{y}) scale({s})"><path d="M -12,-12 L 12,-12 L 12,2 Q 12,12 0,16 Q -12,12 -12,2 Z" fill="{col}" stroke="{OUT}" stroke-width="2.2"/>'
    if beaten:
        o += f'<path d="M -6,1 L -1,6 L 7,-5" fill="none" stroke="{OUT}" stroke-width="2.6" stroke-linecap="round"/>'
    else:
        o += f'<path d="M -6,-5 L 6,7 M 6,-5 L -6,7" stroke="{OUT}" stroke-width="2.6" stroke-linecap="round"/>'
    return o + '</g>'


def icon_shop(x, y, s=1.0):
    return (f'<g transform="translate({x},{y}) scale({s})"><path d="M -12,-4 L 12,-4 L 10,12 L -10,12 Z" fill="#c9955c" '
            f'stroke="{OUT}" stroke-width="2"/><path d="M -6,-4 Q -6,-13 0,-13 Q 6,-13 6,-4" fill="none" '
            f'stroke="{OUT}" stroke-width="2"/><circle cx="0" cy="4" r="3" fill="#f2c14e" stroke="{OUT}" stroke-width="1.2"/></g>')


def icon_person(x, y, s=1.0):
    return (f'<g transform="translate({x},{y}) scale({s})"><path d="M -12,-12 H 12 Q 15,-12 15,-9 V 4 Q 15,7 12,7 H -2 L -8,13 '
            f'L -7,7 H -12 Q -15,7 -15,4 V -9 Q -15,-12 -12,-12 Z" fill="#fffaf0" stroke="{OUT}" stroke-width="2"/>'
            f'<circle cx="-6" cy="-3" r="2" fill="{OUT}"/><circle cx="0" cy="-3" r="2" fill="{OUT}"/><circle cx="6" cy="-3" r="2" fill="{OUT}"/></g>')


def elora_pin(x, y, s=1.0):
    o = f'<g transform="translate({x:.1f},{y:.1f}) scale({s})">'
    o += f'<circle cx="0" cy="-30" r="26" fill="#fff2b0" opacity="0.6"/>'
    o += f'<path d="M -20,-34 Q -20,-56 0,-56 Q 20,-56 20,-34 Q 20,-20 0,0 Q -20,-20 -20,-34 Z" fill="#fffaf0" stroke="{OUT}" stroke-width="3"/>'
    o += g(0, -24, 0.26, drop('f2c14e', 'd9a43a'))
    return o + '</g>'


def decoration(aid, cx, cy, look, rnd):
    """Small painted details per area."""
    o = ''
    ink = look['ink']
    if aid == 'meadow':
        for _ in range(60):
            x, y = cx + rnd.uniform(-140, 140), cy + rnd.uniform(-85, 85)
            c = rnd.choice(['#ef7fb0', '#f2c14e', '#6fa8e8', '#ffffff'])
            o += f'<circle cx="{x:.0f}" cy="{y:.0f}" r="3.2" fill="{c}" stroke="{ink}" stroke-width="0.8"/>'
    elif aid == 'forest':
        for _ in range(34):
            x, y = cx + rnd.uniform(-140, 140), cy + rnd.uniform(-85, 85)
            o += f'<path d="M {x:.0f},{y + 10:.0f} v 8" stroke="#8a6040" stroke-width="3"/>'
            o += f'<circle cx="{x:.0f}" cy="{y:.0f}" r="{rnd.uniform(9, 13):.0f}" fill="{rnd.choice(["#4f8a3a", "#5f9c42", "#6aae4a"])}" stroke="{ink}" stroke-width="1.6"/>'
    elif aid == 'desert':
        for _ in range(18):
            x, y = cx + rnd.uniform(-140, 140), cy + rnd.uniform(-80, 80)
            o += f'<path d="M {x - 22:.0f},{y:.0f} Q {x:.0f},{y - 16:.0f} {x + 22:.0f},{y:.0f}" fill="none" stroke="#c49a58" stroke-width="3" stroke-linecap="round"/>'
        o += f'<path d="M {cx + 40},{cy - 10} v -22 m 0,8 h -8 v -8 m 8,4 h 8 v -10" fill="none" stroke="#5f9c42" stroke-width="4" stroke-linecap="round"/>'
    elif aid == 'frost':
        for k, (dx, h) in enumerate([(-120, 50), (-70, 70), (-20, 96), (34, 80), (86, 62), (128, 44)]):
            x, y = cx + dx, cy + 30
            o += f'<path d="M {x - 40},{y} L {x},{y - h} L {x + 40},{y} Z" fill="#9aa8b8" stroke="{ink}" stroke-width="2"/>'
            o += f'<path d="M {x - 13},{y - h + 26} L {x},{y - h} L {x + 13},{y - h + 26} L {x + 5},{y - h + 20} L {x - 4},{y - h + 27} Z" fill="#ffffff" stroke="{ink}" stroke-width="1.4"/>'
    elif aid == 'gorge':
        o += f'<path d="M {cx - 90},{cy - 20} Q {cx - 30},{cy + 10} {cx},{cy - 5} Q {cx + 40},{cy - 25} {cx + 90},{cy + 10}" fill="none" stroke="#2f3778" stroke-width="16" stroke-linecap="round"/>'
        for _ in range(22):
            o += star(cx + rnd.uniform(-140, 140), cy + rnd.uniform(-80, 70), 4, '#fff3b0', ink, 0.8)
    elif aid == 'tauwinkel':
        for k, (dx, dy, col) in enumerate([(-34, 6, '#e8685a'), (-6, -10, '#5fa8d8'), (24, 4, '#e0a040'), (6, 22, '#8fbf7a')]):
            x, y = cx + dx, cy + dy
            o += f'<rect x="{x - 10}" y="{y - 6}" width="20" height="16" fill="#fffaf0" stroke="{ink}" stroke-width="1.6"/>'
            o += f'<path d="M {x - 13},{y - 6} L {x},{y - 18} L {x + 13},{y - 6} Z" fill="{col}" stroke="{ink}" stroke-width="1.6"/>'
        o += f'<circle cx="{cx - 8}" cy="{cy + 28}" r="7" fill="#9aa4ad" stroke="{ink}" stroke-width="1.6"/><circle cx="{cx - 8}" cy="{cy + 28}" r="3" fill="#5fc8e8"/>'
    return o


def overview(x0, y0, W, H, key):
    look = LOOKS[key]
    rnd = random.Random(7)
    o = f'<clipPath id="clip{key}"><rect x="{x0}" y="{y0}" width="{W}" height="{H}" rx="14"/></clipPath>'
    o += f'<g clip-path="url(#clip{key})">'
    o += f'<rect x="{x0}" y="{y0}" width="{W}" height="{H}" fill="{look["sea"]}"/>'
    for _ in range(40):
        x, y = x0 + rnd.uniform(0, W), y0 + rnd.uniform(0, H)
        o += f'<path d="M {x:.0f},{y:.0f} q 8,-6 16,0 q 8,6 16,0" fill="none" stroke="{look["sea_line"]}" stroke-width="2" stroke-linecap="round"/>'
    # land
    o += f'<path d="{smooth_blob(x0 + W * 0.5, y0 + H * 0.5, W * 0.43, 3, 0.1, squash=0.6)}" fill="{look["land"]}" stroke="{look["land_edge"]}" stroke-width="4"/>'
    # open land between the areas: hills, bushes, a lake
    for _ in range(34):
        x, y = x0 + rnd.uniform(0.12, 0.88) * W, y0 + rnd.uniform(0.15, 0.85) * H
        if any(math.hypot(x - (x0 + W * a[2][0]), (y - (y0 + H * a[2][1])) / 0.7) < 190 for a in AREAS):
            continue
        if rnd.random() < 0.5:
            o += f'<path d="M {x - 26:.0f},{y:.0f} Q {x:.0f},{y - 22:.0f} {x + 26:.0f},{y:.0f}" fill="none" stroke="{look["land_edge"]}" stroke-width="2.5" stroke-linecap="round" opacity="0.7"/>'
        else:
            o += f'<circle cx="{x:.0f}" cy="{y:.0f}" r="7" fill="#9cc77a" stroke="{look["ink"]}" stroke-width="1.4"/>'
    o += f'<path d="{smooth_blob(x0 + W * 0.36, y0 + H * 0.5, 42, 9, 0.2, squash=0.6)}" fill="{look["sea"]}" stroke="{look["land_edge"]}" stroke-width="2.5"/>'
    # regions
    for aid, name, (px, py), col, n in AREAS:
        cx, cy = x0 + W * px, y0 + H * py
        r = 78 if aid == 'tauwinkel' else 170
        o += f'<path d="{smooth_blob(cx, cy, r, sum(map(ord, aid)) % 100, 0.2, squash=0.72)}" fill="{mix(col, look["region_mix"])}" stroke="{darker(col, 0.25)}" stroke-width="2.5" stroke-dasharray="{"6 5" if key == "A" else "none"}"/>'
        o += decoration(aid, cx, cy, look, random.Random(sum(map(ord, aid)) % 1000))
    # river from the frost peaks past Tauwinkel to the sea
    o += (f'<path d="M {x0 + W * 0.8},{y0 + H * 0.36} Q {x0 + W * 0.66},{y0 + H * 0.42} {x0 + W * 0.6},{y0 + H * 0.56} '
          f'Q {x0 + W * 0.55},{y0 + H * 0.72} {x0 + W * 0.58},{y0 + H}" fill="none" stroke="#6fb8e0" stroke-width="10" stroke-linecap="round"/>')
    hub, sections = section_paths(W, H, x0, y0)
    # roads from Tauwinkel
    for aid, (start, _, _) in sections.items():
        dx, dy = start[0] - hub[0], start[1] - hub[1]
        ln = math.hypot(dx, dy)
        edge = (hub[0] + dx / ln * 62, hub[1] + dy / ln * 48)
        o += f'<path d="M {edge[0]:.0f},{edge[1]:.0f} L {start[0]:.0f},{start[1]:.0f}" stroke="{look["road"]}" stroke-width="5" stroke-dasharray="10 8" stroke-linecap="round"/>'
    # section pieces
    for aid, name, (px, py), col, n in AREAS[1:]:
        start, pieces, arena = sections[aid]
        seen = VISITED.get(aid, 0)
        for k, (p, q) in enumerate(pieces):
            mid = ((p[0] + q[0]) / 2 + (q[1] - p[1]) * 0.15, (p[1] + q[1]) / 2 - (q[0] - p[0]) * 0.15)
            d = f'M {p[0]:.1f},{p[1]:.1f} Q {mid[0]:.1f},{mid[1]:.1f} {q[0]:.1f},{q[1]:.1f}'
            if k < seen:
                current = HERE[0] == aid and HERE[1] == k
                if current:
                    o += f'<path d="{d}" fill="none" stroke="#fff2b0" stroke-width="20" stroke-linecap="round" opacity="0.8"/>'
                o += f'<path d="{d}" fill="none" stroke="{look["ink"]}" stroke-width="11" stroke-linecap="round"/>'
                o += f'<path d="{d}" fill="none" stroke="{darker(col, 0.1)}" stroke-width="7" stroke-linecap="round"/>'
            else:
                o += f'<path d="{d}" fill="none" stroke="{look["road"]}" stroke-width="4" stroke-dasharray="4 7" stroke-linecap="round"/>'
            o += f'<circle cx="{q[0]:.1f}" cy="{q[1]:.1f}" r="5" fill="#fffaf0" stroke="{look["ink"]}" stroke-width="2"/>'
        o += icon_guardian(arena[0], arena[1], aid in FREED, 1.1)
    # markers: springs, spring stones, shops in Tauwinkel, quest star, a character
    for aid, name, (px, py), col, n in AREAS[1:]:
        if VISITED.get(aid):
            o += icon_spring(x0 + W * px + 60, y0 + H * py - 46, aid in FREED)
    o += icon_spring_stone(sections['meadow'][1][1][1][0] + 18, sections['meadow'][1][1][1][1] - 16, 0.9)
    o += icon_spring_stone(sections['forest'][1][0][1][0] - 18, sections['forest'][1][0][1][1] - 16, 0.9)
    o += icon_shop(hub[0] + 50, hub[1] - 34, 0.9) + icon_shop(hub[0] + 74, hub[1] - 12, 0.9)
    o += icon_person(sections['desert'][1][0][1][0] + 20, sections['desert'][1][0][1][1] - 22, 0.8)
    o += icon_quest(sections['desert'][1][1][1][0] + 4, sections['desert'][1][1][1][1] - 4, 1.1)
    # clouds: thick over unknown areas, thin over unvisited sections of known areas
    for aid, name, (px, py), col, n in AREAS[1:]:
        cx, cy = x0 + W * px, y0 + H * py
        if not VISITED.get(aid):
            for dx, dy, s in ((-100, -30, 1.5), (-20, -50, 1.4), (70, -30, 1.5), (0, 30, 1.7), (100, 30, 1.3),
                              (-100, 45, 1.2), (130, -10, 1.0), (-140, 5, 1.0)):
                o += cloud(cx + dx, cy + dy, s, 0.95)
        elif VISITED[aid] < n + 1:
            start, pieces, arena = sections[aid]
            for k in range(VISITED[aid], n):
                q = pieces[k][1]
                o += cloud(q[0], q[1], 0.75, 0.55)
            o += cloud(arena[0], arena[1], 0.7, 0.5)
    # area names (only known ones)
    for aid, name, (px, py), col, n in AREAS:
        cx, cy = x0 + W * px, y0 + H * py
        known = aid == 'tauwinkel' or VISITED.get(aid)
        if known:
            o += (f'<text x="{cx}" y="{cy + (56 if aid == "tauwinkel" else 112)}" font-size="20" font-weight="bold" '
                  f'text-anchor="middle" fill="{look["label"]}" stroke="{look["bg"]}" stroke-width="5" paint-order="stroke">{name}</text>')
        else:
            o += text(cx, cy + 6, '?', 34, '#7f93a3', weight='bold')
    # Elora on her section
    p, q = sections[HERE[0]][1][HERE[1]]
    t = HERE[2]
    mid = ((p[0] + q[0]) / 2 + (q[1] - p[1]) * 0.15, (p[1] + q[1]) / 2 - (q[0] - p[0]) * 0.15)
    ex = (1 - t) ** 2 * p[0] + 2 * (1 - t) * t * mid[0] + t * t * q[0]
    ey = (1 - t) ** 2 * p[1] + 2 * (1 - t) * t * mid[1] + t * t * q[1]
    o += elora_pin(ex, ey, 1.0)
    o += '</g>'
    o += f'<rect x="{x0}" y="{y0}" width="{W}" height="{H}" rx="14" fill="none" stroke="{look["ink"]}" stroke-width="3"/>'
    return o


def detail_window(x, y, w, h):
    """Small map of the current section (forest-2) from its tiles: terrain outline, Elora, exits."""
    o = f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="12" fill="#e9f3e2" stroke="{OUT}" stroke-width="3"/>'
    o += text(x + 14, y + 26, 'Flüsterwald · Abschnitt 2', 15, TEXT, 'start', 'bold')
    gx, gy, gw, gh = x + 14, y + 40, w - 28, h - 56
    o += f'<rect x="{gx}" y="{gy}" width="{gw}" height="{gh}" rx="6" fill="#cfe6f2"/>'
    # ground: a stepped profile
    prof = [0.75, 0.75, 0.68, 0.68, 0.8, 0.8, 0.62, 0.55, 0.55, 0.7, 0.7, 0.85, 0.85, 0.66, 0.66, 0.6, 0.6, 0.72, 0.72, 0.72]
    step = gw / (len(prof) - 1)
    pts = ' '.join(f'{gx + i * step:.0f},{gy + gh * v:.0f}' for i, v in enumerate(prof))
    o += f'<polygon points="{gx},{gy + gh} {pts} {gx + gw},{gy + gh}" fill="#6aae4a" stroke="{OUT}" stroke-width="1.5"/>'
    o += f'<rect x="{gx + gw * 0.36:.0f}" y="{gy + gh * 0.36:.0f}" width="{gw * 0.12:.0f}" height="5" fill="#a87a52"/>'
    o += f'<rect x="{gx + gw * 0.6:.0f}" y="{gy + gh * 0.3:.0f}" width="{gw * 0.1:.0f}" height="5" fill="#a87a52"/>'
    o += f'<rect x="{gx + gw * 0.52:.0f}" y="{gy + gh * 0.82:.0f}" width="{gw * 0.07:.0f}" height="{gh * 0.18:.0f}" fill="#5fa8d8" opacity="0.8"/>'
    # exits
    o += f'<path d="M {gx + 8},{gy + gh * 0.5} l 14,-10 v 20 Z" fill="#fffaf0" stroke="{OUT}" stroke-width="2"/>'
    o += text(gx + 26, gy + gh * 0.5 + 18, 'Abschnitt 1', 10, TEXT, 'start')
    o += f'<path d="M {gx + gw - 8},{gy + gh * 0.45} l -14,-10 v 20 Z" fill="#fffaf0" stroke="{OUT}" stroke-width="2"/>'
    o += text(gx + gw - 26, gy + gh * 0.45 + 18, 'Abschnitt 3', 10, TEXT, 'end')
    # markers and Elora
    o += icon_spring_stone(gx + gw * 0.18, gy + gh * 0.66, 0.8)
    o += icon_person(gx + gw * 0.82, gy + gh * 0.68 - 16, 0.7)
    ex, ey = gx + gw * 0.43, gy + gh * 0.55 - 8
    o += f'<circle cx="{ex:.0f}" cy="{ey:.0f}" r="11" fill="#fff2b0" opacity="0.7"/>'
    o += f'<circle cx="{ex:.0f}" cy="{ey:.0f}" r="7" fill="#f2c14e" stroke="{OUT}" stroke-width="2"/>'
    return o


def sheet():
    W, H = 1600, 2600
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
         f'<rect width="{W}" height="{H}" fill="#eef4f8"/>',
         text(40, 50, 'Weltkarte – Entwürfe (R2-WM, WM.0)', 28, TEXT, 'start', 'bold'),
         text(40, 78, 'Gemalte Übersicht in zwei Looks, gleiche Beispiel-Lage: Wiesen und Wald besucht, Elora mitten in '
              'Wald-Abschnitt 2, Wüste begonnen, Frostspitzen und Schlucht noch unter Wolken.', 15, TEXT, 'start')]
    for k, key in enumerate(('A', 'B')):
        y = 100 + k * 900
        o.append(card(30, y, 1540, 880, 'Übersicht ' + LOOKS[key]['name']))
        o.append(overview(60, y + 50, 1480, 800, key))
    # markers and legend
    y = 1920
    o.append(card(30, y, 760, 300, 'Markierungen'))
    items = [(icon_spring_stone, 'Quellstein (gespeichert)'), (icon_quest, 'Quest-Ziel'),
             (lambda x, y_, s: icon_spring(x, y_, False, s), 'Quelle verstummt'),
             (lambda x, y_, s: icon_spring(x, y_, True, s), 'Quelle befreit'),
             (lambda x, y_, s: icon_guardian(x, y_, False, s), 'Hüter wartet'),
             (lambda x, y_, s: icon_guardian(x, y_, True, s), 'Hüter beruhigt'),
             (icon_shop, 'Laden'), (icon_person, 'Person')]
    for i, (fn, label) in enumerate(items):
        cx, cy = 130 + (i % 4) * 175, y + 110 + (i // 4) * 100
        o.append(fn(cx, cy, 1.6))
        o.append(text(cx, cy + 44, label, 13, TEXT))
    o.append(card(810, y, 760, 300, 'Wolken · Elora-Markierung · Wegstücke'))
    o.append(f'<rect x="840" y="{y + 50}" width="700" height="230" rx="10" fill="#efe3c2"/>')
    o.append(cloud(920, y + 140, 1.4, 0.95) + text(920, y + 230, 'Gebiet unbekannt', 13, TEXT))
    o.append(f'<path d="M 1040,{y + 150} q 50,-30 100,0" fill="none" stroke="#a8946e" stroke-width="4" stroke-dasharray="4 7"/>')
    o.append(cloud(1090, y + 140, 0.75, 0.55) + text(1090, y + 230, 'Abschnitt unbesucht', 13, TEXT))
    o.append(f'<path d="M 1200,{y + 160} q 60,-36 120,0" fill="none" stroke="#fff2b0" stroke-width="20" stroke-linecap="round" opacity="0.8"/>')
    o.append(f'<path d="M 1200,{y + 160} q 60,-36 120,0" fill="none" stroke="{OUT}" stroke-width="11" stroke-linecap="round"/>')
    o.append(f'<path d="M 1200,{y + 160} q 60,-36 120,0" fill="none" stroke="#5f9c42" stroke-width="7" stroke-linecap="round"/>')
    o.append(elora_pin(1272, y + 144, 1.0) + text(1260, y + 230, 'aktueller Abschnitt + Elora', 13, TEXT))
    o.append(f'<path d="M 1380,{y + 160} q 60,-36 120,0" fill="none" stroke="{OUT}" stroke-width="11" stroke-linecap="round"/>')
    o.append(f'<path d="M 1380,{y + 160} q 60,-36 120,0" fill="none" stroke="#ef7fb0" stroke-width="7" stroke-linecap="round"/>')
    o.append(text(1440, y + 230, 'besucht', 13, TEXT))
    # detail window and full-screen frame
    y = 2240
    o.append(card(30, y, 760, 340, 'Detailfenster des aktuellen Abschnitts'))
    o.append(detail_window(60, y + 50, 700, 270))
    o.append(card(810, y, 760, 340, 'Vollbild auf Taste M (Aufbau)'))
    fx, fy = 840, y + 50
    o.append(f'<rect x="{fx}" y="{fy}" width="700" height="270" rx="12" fill="#1e2a36" fill-opacity="0.85"/>')
    o.append(f'<rect x="{fx + 20}" y="{fy + 40}" width="440" height="210" rx="8" fill="#efe3c2" stroke="#6b5a43" stroke-width="2"/>')
    o.append(text(fx + 240, fy + 150, 'Übersicht', 18, '#6b5a43', weight='bold'))
    o.append(f'<rect x="{fx + 480}" y="{fy + 40}" width="200" height="120" rx="8" fill="#e9f3e2" stroke="{OUT}" stroke-width="2"/>')
    o.append(text(fx + 580, fy + 105, 'Detailfenster', 14, TEXT, weight='bold'))
    o.append(f'<rect x="{fx + 480}" y="{fy + 170}" width="200" height="80" rx="8" fill="#fffaf0" stroke="{OUT}" stroke-width="2"/>')
    o.append(text(fx + 580, fy + 205, 'Legende · Quest-Schritt', 12, TEXT, weight='bold'))
    o.append(text(fx + 20, fy + 28, 'Tauland', 18, '#ffffff', 'start', 'bold'))
    o.append(text(fx + 680, fy + 28, 'M / Esc schließen', 13, '#cfd8e0', 'end'))
    o.append('</svg>')
    return '\n'.join(o)


if __name__ == '__main__':
    os.makedirs('docs/release-2/design', exist_ok=True)
    open('docs/release-2/design/world-map-drafts.svg', 'w').write(sheet())

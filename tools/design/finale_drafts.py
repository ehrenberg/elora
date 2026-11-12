"""Drafts for the finale "The Old Well" (R2-M2.6, step M2.6.0, E-380, E-381).

Usage: python3 tools/design/finale_drafts.py && cargo xtask svg-preview \
        docs/release-2/design/finale-drafts.svg docs/release-2/design/finale-drafts.png 1600

Style like chapters 1 to 5: dark outline, flat colours. Two variants for Born (the reawakened
guardian of the Origin Spring, your choice), one draft for everything else: the Withered One
curled up and in his colour stages, the five colour drops, the well (sealed, opening, open with
the Origin Spring), the Origin Cave, the dew crown, key pictures of the ending scene and the
spring trials (Born's menu, medals).
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from adventure_figures import DIM, OUT, TEXT, drop, g, star, text  # noqa: E402
from chapter1_drafts import card, elora  # noqa: E402
from chapter5_drafts import (GREY, GREY_DARK, GREY_LIGHT, NIGHT, NIGHT_MID, STARLIGHT, STONE,  # noqa: E402
                             STONE_DARK, STONE_LIGHT, glow_circle, note, st, withered_one)
from tauwinkel_buildings import well  # noqa: E402

# the five spring colours (world book §2) in the order of the chapters
SPRINGS = [('Blütenquelle', '#7fd99a', '#4fa86a'), ('Moosquelle', '#b08a5a', '#7a5a3e'),
           ('Glutquelle', '#f2c14e', '#c9952a'), ('Frostquelle', '#8fcde8', '#5aa0c0'),
           ('Sternenquelle', '#a77be0', '#6e4aa8')]
WATER = '#bfe9f5'
WATER_DARK = '#7cc4dc'
ORIGIN = '#e8f8ff'


def sparkle(cx, cy, r, color=STARLIGHT):
    return star(cx, cy, r, color, OUT, 1.2)


def ground(x, y, w, color=STONE_LIGHT):
    return f'<rect x="{x}" y="{y}" width="{w}" height="6" rx="3" fill="{color}"/>'


# ── Born ────────────────────────────────────────────────────────────────

def five_streams(cx, top, spread=46, height=70):
    """A crown of five coloured streams rising from a head (Born's sign)."""
    s = ''
    for k, (_, c, _d) in enumerate(SPRINGS):
        x = cx + (k - 2) * spread / 2
        bend = (k - 2) * 14
        s += (f'<path d="M {x},{top} Q {x + bend * 0.4},{top - height * 0.6} {x + bend},{top - height}" fill="none" '
              f'stroke="{OUT}" stroke-width="10" stroke-linecap="round"/>'
              f'<path d="M {x},{top} Q {x + bend * 0.4},{top - height * 0.6} {x + bend},{top - height}" fill="none" '
              f'stroke="{c}" stroke-width="6" stroke-linecap="round"/>'
              f'<circle cx="{x + bend}" cy="{top - height - 2}" r="6" fill="{c}" {st(2)}/>')
    return s


def born(variant='A', pose='ruhig'):
    """Born, the reawakened guardian of the Origin Spring (D-M26-07).
    A: the Withered One's tall shape, now clear blue-white water with a crown of five streams.
    B: an old, wide drop with a beard of falling water and five colour lights circling him.
    Poses: erwacht (eyes just opening), ruhig (calm), froh (glad, at the well)."""
    s = glow_circle(0, -150, 120, ORIGIN, 0.5 if pose != 'erwacht' else 0.25)
    if variant == 'A':
        s += f'<path d="M 0,-250 Q 58,-160 58,-110 Q 58,-58 0,-58 Q -58,-58 -58,-110 Q -58,-160 0,-250 Z" fill="{WATER}" {st(5)}/>'
        s += f'<path d="M -26,-200 Q -40,-160 -36,-120" fill="none" stroke="#ffffff" stroke-width="8" stroke-linecap="round" opacity="0.8"/>'
        # the old cracks are healed lines of light
        s += f'<path d="M -10,-200 l 12,24 l -10,18 M 24,-150 l -10,20 l 10,14" fill="none" stroke="{ORIGIN}" stroke-width="3"/>'
        s += f'<path d="M -18,-60 L -22,0 M 18,-60 L 22,0" stroke="{OUT}" stroke-width="10" stroke-linecap="round"/>'
        s += f'<path d="M -18,-60 L -22,0 M 18,-60 L 22,0" stroke="{WATER_DARK}" stroke-width="6" stroke-linecap="round"/>'
        arms = {'froh': 'M 48,-130 Q 90,-170 110,-200 M -48,-130 Q -90,-170 -110,-200',
                'ruhig': 'M 48,-120 Q 78,-96 72,-64 M -48,-120 Q -78,-96 -72,-64',
                'erwacht': 'M 48,-118 Q 70,-90 60,-64 M -48,-118 Q -70,-90 -60,-64'}[pose]
        s += f'<path d="{arms}" fill="none" stroke="{OUT}" stroke-width="12" stroke-linecap="round"/>'
        s += f'<path d="{arms}" fill="none" stroke="{WATER_DARK}" stroke-width="7" stroke-linecap="round"/>'
        s += five_streams(0, -238, 40, 64)
        face_y = -150
    else:
        s += f'<path d="M 0,-230 Q 84,-150 84,-90 Q 84,-30 0,-30 Q -84,-30 -84,-90 Q -84,-150 0,-230 Z" fill="{WATER}" {st(5)}/>'
        s += f'<path d="M -40,-180 Q -60,-140 -56,-100" fill="none" stroke="#ffffff" stroke-width="8" stroke-linecap="round" opacity="0.8"/>'
        # beard of falling water
        s += (f'<path d="M -40,-112 Q -30,-40 -10,-14 Q 0,-4 10,-14 Q 30,-40 40,-112 Q 0,-90 -40,-112 Z" '
              f'fill="{ORIGIN}" {st(3)}/>')
        for x in (-20, 0, 20):
            s += f'<path d="M {x},-96 q 4,30 0,60" fill="none" stroke="{WATER_DARK}" stroke-width="2.5"/>'
        s += f'<ellipse cx="-30" cy="-26" rx="20" ry="10" fill="{WATER_DARK}" {st(3)}/><ellipse cx="30" cy="-26" rx="20" ry="10" fill="{WATER_DARK}" {st(3)}/>'
        # five colour lights circling him
        for k, (_, c, _d) in enumerate(SPRINGS):
            a = math.radians(-160 + k * 35)
            s += f'<circle cx="{110 * math.cos(a):.1f}" cy="{-130 + 90 * math.sin(a):.1f}" r="11" fill="{c}" {st(2.5)}/>'
        face_y = -140
    # face
    if pose == 'erwacht':
        s += (f'<path d="M -22,{face_y} q 8,5 16,0 M 6,{face_y} q 8,5 16,0" fill="none" stroke="{OUT}" '
              f'stroke-width="4" stroke-linecap="round"/>')
        s += f'<ellipse cx="-14" cy="{face_y + 1}" rx="4" ry="2" fill="{OUT}"/><ellipse cx="14" cy="{face_y + 1}" rx="4" ry="2" fill="{OUT}"/>'
    else:
        s += f'<ellipse cx="-14" cy="{face_y}" rx="7" ry="10" fill="{OUT}"/><ellipse cx="14" cy="{face_y}" rx="7" ry="10" fill="{OUT}"/>'
        s += f'<circle cx="-12" cy="{face_y - 4}" r="2.5" fill="#ffffff"/><circle cx="16" cy="{face_y - 4}" r="2.5" fill="#ffffff"/>'
    mouth_y = face_y + 28 if variant == 'A' else face_y + 22
    if pose == 'froh':
        s += f'<path d="M -12,{mouth_y} q 12,12 24,0 Z" fill="{OUT}"/>'
    elif pose == 'ruhig':
        s += f'<path d="M -10,{mouth_y} q 10,8 20,0" fill="none" stroke="{OUT}" stroke-width="3.5" stroke-linecap="round"/>'
    return s


# ── The Withered One in the heart of the cave ───────────────────────────

def withered_curled(colours=0):
    """The Withered One curled up beside the dry basin (D-M26-04); with every colour he gets
    back one coloured band shows on his body and he lifts his head a little."""
    lift = colours * 8
    s = ''
    if colours:
        s += glow_circle(0, -70, 70 + colours * 8, ORIGIN, 0.08 * colours)
    s += (f'<path d="M -90,0 Q -100,-70 -50,-{100 + lift} Q 0,-{130 + lift} 50,-{100 + lift} Q 96,-60 90,0 Z" '
          f'fill="{GREY}" {st(5)}/>')
    s += f'<path d="M -50,-70 l 14,18 l -8,16 M 30,-90 l -10,18" fill="none" stroke="{GREY_DARK}" stroke-width="3"/>'
    for k in range(colours):
        c = SPRINGS[k][1]
        y = -14 - k * 16
        s += f'<path d="M -84,{y} Q 0,{y - 16} 84,{y}" fill="none" stroke="{c}" stroke-width="7" stroke-linecap="round" opacity="0.9"/>'
    # twig arms around the knees
    s += f'<path d="M -70,-40 Q -20,-10 30,-30 L 50,-20" fill="none" stroke="{OUT}" stroke-width="9" stroke-linecap="round"/>'
    s += f'<path d="M -70,-40 Q -20,-10 30,-30 L 50,-20" fill="none" stroke="#8a7a66" stroke-width="5" stroke-linecap="round"/>'
    # face: eyes closed, opening with the colours
    fy = -86 - lift
    if colours < 3:
        s += f'<path d="M 4,{fy} q 8,5 16,0 M 32,{fy} q 8,5 16,0" fill="none" stroke="{OUT}" stroke-width="3.5" stroke-linecap="round"/>'
    else:
        s += f'<ellipse cx="12" cy="{fy}" rx="6" ry="{3 + colours}" fill="{OUT}"/><ellipse cx="40" cy="{fy}" rx="6" ry="{3 + colours}" fill="{OUT}"/>'
    if colours == 0:
        # a single tear
        s += f'<path d="M 42,{fy + 6} q -4,10 0,14 q 4,-4 0,-14 Z" fill="{WATER}" {st(1.5)}/>'
    return s


# ── Colour drops, crown, medals ─────────────────────────────────────────

def colour_drop(color, dark, carried=False):
    """A colour drop of one spring (D-M26-03/04): floating, glowing; carried it trails sparkles."""
    s = glow_circle(0, -22, 26, color, 0.4)
    s += f'<path d="M 0,-48 Q 18,-24 18,-14 Q 18,2 0,2 Q -18,2 -18,-14 Q -18,-24 0,-48 Z" fill="{color}" {st(3)}/>'
    s += f'<path d="M -7,-30 Q -11,-20 -9,-12" fill="none" stroke="#ffffff" stroke-width="3" stroke-linecap="round" opacity="0.9"/>'
    s += f'<path d="M 4,-12 q 5,4 9,0" fill="none" stroke="{dark}" stroke-width="2"/>'
    if carried:
        for k, (dx, dy) in enumerate(((-30, -14), (-46, -4), (-60, -18))):
            s += sparkle(dx, dy, 5 - k, color)
    return s


def dew_crown():
    """Dew crown (D-M26-10): hat, +1 health, glows in all spring colours."""
    s = f'<path d="M -40,0 L -46,-36 L -24,-18 L -12,-46 L 0,-20 L 12,-46 L 24,-18 L 46,-36 L 40,0 Z" fill="{WATER}" {st(3.5)}/>'
    s += f'<rect x="-42" y="-8" width="84" height="10" rx="4" fill="{WATER_DARK}" {st(3)}/>'
    for k, (_, c, _d) in enumerate(SPRINGS):
        x = -32 + k * 16
        s += f'<circle cx="{x}" cy="-3" r="4.5" fill="{c}" {st(1.5)}/>'
    for x, y in ((-46, -36), (-12, -46), (12, -46), (46, -36)):
        s += f'<path d="M {x},{y} q 3,-8 0,-12 q -3,4 0,12 Z" fill="{ORIGIN}" {st(1.5)}/>'
    return s


def medal(kind):
    """Trial medal (E-381): bronze, silver, gold."""
    c, d = {'bronze': ('#d08a4e', '#9a5e2e'), 'silber': ('#d6dbe2', '#9aa2ad'), 'gold': ('#f2c14e', '#c9952a')}[kind]
    s = f'<path d="M -14,-60 L -4,-30 M 14,-60 L 4,-30" stroke="{OUT}" stroke-width="10"/>'
    s += f'<path d="M -14,-60 L -4,-30 M 14,-60 L 4,-30" stroke="#7fd99a" stroke-width="6"/>'
    s += f'<circle cx="0" cy="-16" r="20" fill="{c}" {st(3.5)}/><circle cx="0" cy="-16" r="13" fill="none" stroke="{d}" stroke-width="2.5"/>'
    s += f'<path d="M 0,-26 Q 8,-16 8,-12 Q 8,-6 0,-6 Q -8,-6 -8,-12 Q -8,-16 0,-26 Z" fill="{d}"/>'
    return s


# ── The well ────────────────────────────────────────────────────────────

def well_opening():
    """The lid cracks and lifts while Oma sings (D-M26-01)."""
    s = glow_circle(0, -60, 70, ORIGIN, 0.45)
    base = well(False)
    # replace nothing – draw the lifted lid and light over the sealed well
    s += base
    s += f'<path d="M -50,-46 Q 0,-62 50,-46" fill="none" stroke="{ORIGIN}" stroke-width="6"/>'
    s += f'<g transform="translate(0,-26) rotate(-12)"><path d="M -50,-46 Q 0,-62 50,-46 Z" fill="#c9955c" {st(3)}/></g>'
    for k, (_, c, _d) in enumerate(SPRINGS):
        s += note(-60 + k * 30, -150 - (k % 2) * 16, 0.8, c)
    return s


def well_open():
    """The open well: the lid gone, the Origin Spring rises as a fountain in five colours."""
    s = glow_circle(0, -110, 110, ORIGIN, 0.5)
    for k, (_, c, _d) in enumerate(SPRINGS):
        a = -60 + k * 30
        s += (f'<path d="M 0,-48 Q {a * 0.4},-200 {a * 1.6},-90" fill="none" stroke="{OUT}" stroke-width="11" stroke-linecap="round"/>'
              f'<path d="M 0,-48 Q {a * 0.4},-200 {a * 1.6},-90" fill="none" stroke="{c}" stroke-width="7" stroke-linecap="round"/>')
    s += f'<path d="M -8,-48 Q 0,-220 8,-48 Z" fill="{ORIGIN}" {st(3)}/>'
    from tauwinkel_buildings import flowers, stone_wall
    s += f'<path d="M -84,0 Q -70,-10 -56,0 Z M 56,0 Q 70,-10 84,0 Z" fill="#7a5a3e" {st(1.6)}/>'
    s += flowers([-80, -73, -66, -59], -3, False) + flowers([60, 67, 74, 81], -3, False)
    s += stone_wall(-48, 48, -44, 0)
    s += f'<path d="M -48,-44 H 48" stroke="{WATER_DARK}" stroke-width="8" stroke-linecap="round"/>'
    return s


# ── The Origin Cave ─────────────────────────────────────────────────────

def cave_shaft(x, y, w, h):
    """`well-shaft`: a deep round shaft with stone steps, roots and glowing water veins."""
    o = [f'<g transform="translate({x},{y})">',
         f'<rect width="{w}" height="{h}" rx="12" fill="{NIGHT}"/>',
         f'<rect x="{w * 0.18}" y="0" width="{w * 0.64}" height="{h}" fill="{NIGHT_MID}"/>']
    for k in range(7):
        side = k % 2
        sx = w * 0.18 if side == 0 else w * 0.82 - 70
        sy = 40 + k * (h - 60) / 7
        o.append(f'<rect x="{sx}" y="{sy}" width="70" height="14" rx="3" fill="{STONE_LIGHT}" {st(2.5)}/>')
    for k in range(3):
        rx = w * 0.18 + 10 + k * 70
        o.append(f'<path d="M {rx},0 q 10,60 -6,120 q -8,30 4,50" fill="none" stroke="#7a5a3e" stroke-width="5"/>')
    for k, (_, c, _d) in enumerate(SPRINGS):
        vy = h * 0.55 + k * 22
        o.append(f'<path d="M {w * 0.18},{vy} q 40,-12 {w * 0.64},6" fill="none" stroke="{c}" stroke-width="3" opacity="{0.25 + k * 0.15}"/>')
    o.append(g(w / 2, h - 30, 0.22, drop('f2c14e', 'd9a43a')))
    o.append('</g>')
    return ''.join(o)


def cave_veins(x, y, w, h):
    """`origin-cave`: a hall with five side passages in the spring colours, each holding a colour
    drop behind its ability spot; the stone door to the heart in the middle."""
    o = [f'<g transform="translate({x},{y})">', f'<rect width="{w}" height="{h}" rx="12" fill="{NIGHT}"/>',
         f'<rect x="0" y="{h - 40}" width="{w}" height="40" fill="{STONE}"/>']
    labels = ['Hook-Ruck', 'Zughaken', 'Stampfen', 'Steigkrallen', 'Gleiten']
    pw = w / 5
    for k, (_, c, d) in enumerate(SPRINGS):
        px = k * pw
        o.append(f'<rect x="{px + 8}" y="40" width="{pw - 16}" height="{h - 90}" rx="10" fill="{c}" fill-opacity="0.22" stroke="{c}" stroke-width="3"/>')
        o.append(g(px + pw / 2, 110, 0.9, colour_drop(c, d)))
        o.append(text(px + pw / 2, h - 58, labels[k], 12, '#ffffff', weight='bold'))
    o.append(f'<rect x="{w / 2 - 26}" y="{h - 110}" width="52" height="70" rx="6" fill="{STONE_DARK}" {st(3)}/>')
    o.append(f'<circle cx="{w / 2}" cy="{h - 80}" r="10" fill="none" stroke="{GREY_LIGHT}" stroke-width="3"/>')
    o.append('</g>')
    return ''.join(o)


def cave_heart(x, y, w, h, colours):
    """`origin-heart`: the dry basin with the Withered One; with every colour the cave gets light,
    moss and water; with all five the note stones appear for the well song."""
    bg = NIGHT if colours == 0 else '#3a4a7a' if colours < 5 else '#4f6fa8'
    o = [f'<g transform="translate({x},{y})">', f'<rect width="{w}" height="{h}" rx="12" fill="{bg}"/>',
         f'<path d="M 0,{h - 40} Q {w / 2},{h - 80} {w},{h - 40} V {h} H 0 Z" fill="{STONE}"/>']
    # basin
    o.append(f'<path d="M {w / 2 - 90},{h - 62} Q {w / 2},{h - 20} {w / 2 + 90},{h - 62} Z" fill="{STONE_DARK}" {st(2.5)}/>')
    if colours:
        lvl = 8 + colours * 5
        o.append(f'<path d="M {w / 2 - 80},{h - 58} Q {w / 2},{h - 58 + lvl} {w / 2 + 80},{h - 58} Z" fill="{WATER}" opacity="0.9"/>')
    for k in range(colours):
        c = SPRINGS[k][1]
        mx = 30 + k * (w - 60) / 4
        o.append(f'<path d="M {mx - 18},{h - 44} q 18,-22 36,0 Z" fill="{c}" {st(2)}/>')
    o.append(g(w / 2, h - 60, 0.55, withered_curled(colours)))
    if colours == 5:
        for k, (_, c, _d) in enumerate(SPRINGS):
            nx = w / 2 - 100 + k * 50
            o.append(f'<rect x="{nx - 14}" y="{h - 84}" width="28" height="26" rx="6" fill="{c}" {st(2.5)}/>')
            o.append(note(nx + 4, h - 100, 0.6, c))
    o.append('</g>')
    return ''.join(o)


# ── Ending key pictures ─────────────────────────────────────────────────

def ending_frame(x, y, w, h, n):
    """Key pictures of the ending scene (D-M26-08): 1 water and colour rise through the well,
    2 the five guardians come to the well square, 3 Oma sings with everyone."""
    sky = ['#2f3778', '#8fb8e0', '#f2c58a'][n - 1]
    o = [f'<g transform="translate({x},{y})">', f'<rect width="{w}" height="{h}" rx="12" fill="{sky}"/>',
         f'<rect x="0" y="{h - 34}" width="{w}" height="34" fill="#8fbf7a"/>']
    if n == 1:
        o.append(g(w / 2, h - 34, 0.75, well_open()))
    elif n == 2:
        o.append(g(w / 2, h - 34, 0.45, well_open()))
        for k, (_, c, d) in enumerate(SPRINGS):
            gx = 30 + k * (w - 60) / 4
            if abs(gx - w / 2) < 50:
                gx += 70
            o.append(g(gx, h - 34, 0.22, drop(c[1:], d[1:])))
        o.append(text(w / 2, 24, 'die fünf Wächter (Platzhalter)', 11, '#ffffff', style='italic'))
    else:
        o.append(g(w / 2, h - 34, 0.45, well_open()))
        o.append(g(w / 2 - 80, h - 34, 0.3, drop('c8a0e8', '9a70c0')))
        o.append(elora(w / 2 + 70, h - 34, 0.28))
        for k in range(4):
            o.append(note(w / 2 - 110 + k * 50, 40 + (k % 2) * 20, 0.9, SPRINGS[k][1]))
        o.append(text(w / 2 - 80, h - 80, 'Oma', 11, TEXT, weight='bold'))
    o.append(text(16, h - 12, str(n), 18, '#ffffff', 'start', 'bold'))
    o.append('</g>')
    return ''.join(o)


# ── Spring trials ───────────────────────────────────────────────────────

def trial_menu(x, y):
    """Born's trial menu (E-381): a list per area with best time and medal."""
    rows = [('Blütenwiesen 1', '1:12,4', 'gold'), ('Blütenwiesen 2', '2:03,9', 'silber'),
            ('Blütenwiesen 3', '1:48,0', 'bronze'), ('Murmelwald 1', '–', None), ('Brunnenschacht', '–', None)]
    o = [f'<g transform="translate({x},{y})">',
         f'<rect width="420" height="300" rx="14" fill="#fffaf0" stroke="#c9b48e" stroke-width="3"/>',
         text(210, 34, 'Quellen-Prüfungen', 20, TEXT, weight='bold'),
         text(210, 54, 'schwerer, auf Zeit · Bestzeit und Medaille', 12, DIM)]
    for k, (name, t, m) in enumerate(rows):
        ry = 76 + k * 42
        fill = '#f3ead8' if k == 0 else '#fffaf0'
        o.append(f'<rect x="16" y="{ry}" width="388" height="36" rx="8" fill="{fill}" stroke="#e7dcc8"/>')
        o.append(text(32, ry + 24, name, 14, TEXT, 'start', 'bold' if k == 0 else 'normal'))
        o.append(text(300, ry + 24, t, 14, TEXT, 'end'))
        if m:
            o.append(g(350, ry + 40, 0.55, medal(m)))
    o.append('</g>')
    return ''.join(o)


# ── Sheet ───────────────────────────────────────────────────────────────

def sheet():
    W, H = 1600, 2380
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
         '<defs><linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#3a4290"/>'
         '<stop offset="1" stop-color="#bfe9f5"/></linearGradient></defs>',
         f'<rect width="{W}" height="{H}" fill="url(#sky)"/>',
         text(40, 50, 'Finale „Der alte Brunnen“ – Entwürfe (R2-M2.6, M2.6.0)', 28, '#ffffff', 'start', 'bold'),
         text(40, 78, 'Born in zwei Varianten zur Auswahl; der Dürre in der Höhle, Farbtropfen, Brunnen, Urquellen-Höhle, '
              'Tau-Krone, Schlussszene und Quellen-Prüfungen. Elora klein daneben als Maßstab.', 15, '#ffffff', 'start')]

    # Born
    o.append(card(30, 100, 1540, 470, 'Born, Hüter der Urquelle (D-M26-07) – Variante A (klare Gestalt mit Krone aus fünf Strömen) '
                  'oder B (alter Tropfen mit Wasserbart, fünf Farblichter)'))
    o.append(ground(60, 480, 1480))
    for x, var, pose, name in ((170, 'A', 'erwacht', 'A: erwacht'), (400, 'A', 'ruhig', 'A: ruhig'),
                               (630, 'A', 'froh', 'A: froh'), (960, 'B', 'erwacht', 'B: erwacht'),
                               (1190, 'B', 'ruhig', 'B: ruhig'), (1430, 'B', 'froh', 'B: froh')):
        o.append(g(x, 480, 0.8, born(var, pose)))
        o.append(text(x, 516, name, 15, TEXT, weight='bold'))
    o.append(elora(800, 480))
    o.append(text(800, 546, 'aus dem Dürren wird wieder der Hüter: klar, ruhig, in allen fünf Farben', 12, DIM, style='italic'))

    # Withered One in the heart, colour drops
    o.append(card(30, 590, 1000, 400, 'Der Dürre im Herz der Höhle (D-M26-04): zusammengerollt, mit jeder Farbe ein Stück wacher'))
    o.append(ground(60, 900, 940))
    for x, n in ((150, 0), (390, 2), (630, 4), (870, 5)):
        o.append(g(x, 900, 0.9, withered_curled(n)))
        o.append(text(x, 936, f'{n} von 5 Farben', 15, TEXT, weight='bold'))
    o.append(text(530, 966, 'Augen öffnen sich ab der dritten Farbe; keine Wunde, kein Kampf', 12, DIM, style='italic'))
    o.append(card(1050, 590, 520, 400, 'Farbtropfen (D-M26-03/04)'))
    o.append(ground(1080, 900, 460))
    for k, (name, c, d) in enumerate(SPRINGS):
        tx = 1110 + k * 100
        o.append(g(tx, 860, 1.0, colour_drop(c, d)))
        o.append(text(tx, 936, name.replace('quelle', ''), 12, TEXT, weight='bold'))
    o.append(elora(1210, 770) + g(1170, 760, 0.8, colour_drop(SPRINGS[2][1], SPRINGS[2][2], True)))
    o.append(text(1310, 750, 'getragen: schwebt hinter Elora,', 12, DIM, 'start'))
    o.append(text(1310, 766, 'ein Treffer lässt ihn fallen', 12, DIM, 'start'))

    # the well
    o.append(card(30, 1010, 1000, 420, 'Der alte Brunnen (D-M26-01, D-M26-08): versiegelt · öffnet sich beim Lied · offen mit der Urquelle'))
    o.append(ground(60, 1360, 940, '#8fbf7a'))
    o.append(g(190, 1360, 1.2, well(False)))
    o.append(text(190, 1396, 'versiegelt (heute)', 15, TEXT, weight='bold'))
    o.append(g(520, 1360, 1.2, well_opening()))
    o.append(text(520, 1396, 'Deckel hebt sich', 15, TEXT, weight='bold'))
    o.append(g(850, 1360, 1.2, well_open()))
    o.append(text(850, 1396, 'offen: die Urquelle', 15, TEXT, weight='bold'))
    o.append(card(1050, 1010, 520, 420, 'Belohnung (D-M26-10) und Medaillen (E-381)'))
    o.append(ground(1080, 1360, 460))
    o.append(g(1180, 1350, 1.6, dew_crown()))
    o.append(text(1180, 1396, 'Tau-Krone', 15, TEXT, weight='bold'))
    o.append(text(1180, 1414, '+1 Leben, leuchtet in allen Farben', 12, DIM))
    for k, m in enumerate(('bronze', 'silber', 'gold')):
        o.append(g(1350 + k * 70, 1350, 1.4, medal(m)))
        o.append(text(1350 + k * 70, 1396, m.capitalize(), 14, TEXT, weight='bold'))

    # the cave
    o.append(card(30, 1450, 1540, 470, 'Urquellen-Höhle (D-M26-02): Brunnenschacht · Farbadern · Herz (vorher, drei Farben, alle fünf mit Notensteinen)'))
    o.append(cave_shaft(60, 1500, 220, 360))
    o.append(text(170, 1890, 'well-shaft', 14, TEXT, weight='bold'))
    o.append(cave_veins(300, 1500, 440, 360))
    o.append(text(520, 1890, 'origin-cave', 14, TEXT, weight='bold'))
    for k, n in enumerate((0, 3, 5)):
        hx = 760 + k * 270
        o.append(cave_heart(hx, 1500, 260, 360, n))
        o.append(text(hx + 130, 1890, f'origin-heart · {n}/5', 14, TEXT, weight='bold'))

    # ending and trials
    o.append(card(30, 1940, 1040, 420, 'Schlussszene (D-M26-08): Schlüsselbilder'))
    for n in (1, 2, 3):
        o.append(ending_frame(60 + (n - 1) * 335, 1990, 315, 300, n))
    o.append(text(550, 2320, '1 Wasser und Farbe steigen durch den Brunnen · 2 die Wächter kommen · 3 Oma singt mit allen, dann Abspann',
                  13, DIM, style='italic'))
    o.append(card(1090, 1940, 480, 420, 'Quellen-Prüfungen bei Born (E-381)'))
    o.append(trial_menu(1120, 1990))
    o.append(text(1330, 2320, 'nur besuchte Abschnitte; Ergebnis danach bei Born', 12, DIM, style='italic'))
    o.append('</svg>')
    return '\n'.join(o)


if __name__ == '__main__':
    os.makedirs('docs/release-2/design', exist_ok=True)
    open('docs/release-2/design/finale-drafts.svg', 'w').write(sheet())

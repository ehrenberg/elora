"""Entwürfe für Kapitel 2 „Murmelwald“ (R2-M2.2, Schritt M2.2.0, E-306 bis E-311).

Aufruf: python3 tools/design/kapitel2_entwuerfe.py && cargo xtask svg-preview \
        docs/release-2/design/kapitel2-entwuerfe.svg docs/release-2/design/kapitel2-entwuerfe.png 1600

Stil wie Kapitel 1 (kapitel1_entwuerfe.py): dunkler Umriss, flache Farben; Gegner rund und
frech, ohne Blut. Der Wald ist schattiger, aber freundlich: Moos, leuchtende Pilze, Baumhäuser.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from abenteuer_figuren import DIM, OUT, TEXT, angry_eye, drop, g, star, text  # noqa: E402
from kapitel1_entwuerfe import card, elora, ground  # noqa: E402

BARK = '#8a6040'
BARK_DARK = '#5e4430'
MOSS = '#6f9a4a'
MOSS_DARK = '#4f7a34'
LEAF = '#4f8f3a'
LEAF_LIGHT = '#6cbf4a'
GLOW = '#9ff2d8'
AMBER = '#f2b33a'
NUT = '#a8703c'


def st(w=3.5):
    return f'stroke="{OUT}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'


def blob(cx, cy, rx, ry, fill, n=40, jag=0.05, w=4):
    """Gezackter Umriss (Fell, Federn, Moos)."""
    pts = []
    for i in range(n):
        a = 2 * math.pi * i / n
        r = 1 + (jag if i % 2 else 0)
        pts.append(f'{cx + rx * r * math.cos(a):.1f},{cy + ry * r * math.sin(a):.1f}')
    return f'<polygon points="{" ".join(pts)}" fill="{fill}" {st(w)}/>'


# ── Figuren ─────────────────────────────────────────────────────────────

def plumm():
    """Der alte Uhu Plumm: rund, Federohren, Brille, ein Buch voller Geschichten."""
    s = blob(0, -62, 52, 62, '#9a7a5a', 44, 0.04)
    s += f'<ellipse cx="0" cy="-44" rx="32" ry="38" fill="#e8d8b8" {st(3)}/>'
    for k in range(4):
        y = -60 + k * 12
        s += f'<path d="M -16,{y} q 4,5 8,0 q 4,5 8,0 q 4,5 8,0" fill="none" stroke="#b8a07a" stroke-width="2"/>'
    # Federohren
    s += f'<path d="M -40,-112 L -50,-142 L -22,-120 Z" fill="#9a7a5a" {st(3.5)}/>'
    s += f'<path d="M 40,-112 L 50,-142 L 22,-120 Z" fill="#9a7a5a" {st(3.5)}/>'
    # Augen mit Brille
    for ex in (-18, 18):
        s += f'<circle cx="{ex}" cy="-96" r="17" fill="#f6efdf" {st(3)}/>'
        s += f'<circle cx="{ex}" cy="-94" r="8" fill="#e8a83a"/><circle cx="{ex}" cy="-94" r="4.5" fill="{OUT}"/>'
        s += f'<circle cx="{ex - 2}" cy="-96" r="1.6" fill="#ffffff"/>'
        s += f'<circle cx="{ex}" cy="-96" r="20" fill="none" stroke="#5a4a3a" stroke-width="3"/>'
    s += '<path d="M -2,-96 H 2" stroke="#5a4a3a" stroke-width="3"/>'
    s += f'<path d="M -6,-80 L 0,-68 L 6,-80 Z" fill="#e8a83a" {st(2.5)}/>'
    # Flügel mit Buch
    s += f'<path d="M -48,-70 Q -66,-40 -40,-20 Q -30,-40 -36,-66 Z" fill="#8a6a4a" {st(3.5)}/>'
    s += f'<rect x="20" y="-48" width="40" height="30" rx="3" fill="#5aaee8" {st(3)} transform="rotate(-12 40 -33)"/>'
    s += f'<path d="M 40,-48 V -18" stroke="#ffffff" stroke-width="2" transform="rotate(-12 40 -33)"/>'
    s += f'<path d="M 48,-68 Q 66,-40 44,-24" fill="#8a6a4a" {st(3.5)}/>'
    # Füße
    for x in (-14, 14):
        s += f'<path d="M {x - 8},0 l 4,-8 l 4,8 l 4,-8 l 4,8" fill="none" stroke="#e8a83a" stroke-width="4" stroke-linejoin="round"/>'
    return s


def pilzkind(sad=True):
    """Kleines Pilzkind: Tropfenkörper mit rotem Pilzhut, verweint oder froh."""
    s = drop('f6efdf', 'd8c8a8')
    s += f'<path d="M -70,-92 Q -60,-150 0,-156 Q 60,-150 70,-92 Q 0,-104 -70,-92 Z" fill="#e8685a" {st(4.5)}/>'
    for (x, y, r) in ((-34, -126, 9), (6, -140, 11), (38, -118, 8), (-6, -112, 6)):
        s += f'<circle cx="{x}" cy="{y}" r="{r}" fill="#fff6e8" stroke="{OUT}" stroke-width="2"/>'
    if sad:
        for x in (2, 30):
            s += f'<path d="M {x},-58 q 3,10 0,16" fill="none" stroke="#7fc8e8" stroke-width="5" stroke-linecap="round"/>'
    return s


def pilzmama():
    """Mutter des Pilzkinds: größer, brauner Hut mit hellen Tupfen, Schürze."""
    s = drop('f6efdf', 'd8c8a8')
    s += f'<path d="M -46,-40 Q 4,-30 52,-40 L 50,-14 Q 4,-6 -44,-14 Z" fill="#7fd99a" {st(3.5)}/>'
    s += f'<path d="M -78,-92 Q -66,-160 0,-166 Q 66,-160 78,-92 Q 0,-106 -78,-92 Z" fill="#a8703c" {st(4.5)}/>'
    for (x, y, r) in ((-40, -128, 10), (4, -146, 12), (44, -122, 9), (-8, -114, 6)):
        s += f'<circle cx="{x}" cy="{y}" r="{r}" fill="#f6e8c8" stroke="{OUT}" stroke-width="2"/>'
    return s


# ── Gegner ──────────────────────────────────────────────────────────────

def wurzelschlange(out=True):
    """Wurzelschlange: knorrige Wurzel mit Blättern; `out` = aufgetaucht, sonst nur Erdhügel."""
    s = f'<path d="M -40,0 Q -36,-14 -20,-16 H 20 Q 36,-14 40,0 Z" fill="#7a5a3e" {st(3.5)}/>'
    for x in (-24, -6, 14, 28):
        s += f'<circle cx="{x}" cy="-16" r="4" fill="#5e4430"/>'
    if not out:
        s += f'<path d="M -6,-16 q 4,-10 10,-4" fill="none" stroke="{MOSS_DARK}" stroke-width="4" stroke-linecap="round"/>'
        return s
    s += f'<path d="M 0,-14 C -26,-50 30,-70 4,-110 C -6,-126 18,-140 30,-132" fill="none" stroke="{OUT}" stroke-width="26" stroke-linecap="round"/>'
    s += f'<path d="M 0,-14 C -26,-50 30,-70 4,-110 C -6,-126 18,-140 30,-132" fill="none" stroke="{BARK}" stroke-width="18" stroke-linecap="round"/>'
    s += f'<path d="M -6,-40 q 8,-6 14,0 M 8,-76 q 8,-6 12,2 M -2,-104 q 6,-6 12,0" fill="none" stroke="{BARK_DARK}" stroke-width="3"/>'
    s += f'<path d="M 16,-60 Q 38,-70 40,-54 Q 28,-50 16,-60 Z" fill="{LEAF_LIGHT}" {st(2.5)}/>'
    s += f'<ellipse cx="34" cy="-134" rx="22" ry="16" fill="{BARK}" {st(4)}/>'
    s += angry_eye(40, -138, 6)
    s += f'<path d="M 46,-126 L 60,-124" stroke="{OUT}" stroke-width="3" stroke-linecap="round"/>'
    s += f'<path d="M 18,-148 q -6,-12 4,-16" fill="none" stroke="{LEAF_LIGHT}" stroke-width="5" stroke-linecap="round"/>'
    return s


def nuss(x, y, r=11):
    return (f'<ellipse cx="{x}" cy="{y}" rx="{r}" ry="{r * 1.2:.1f}" fill="{NUT}" stroke="{OUT}" stroke-width="2.5"/>'
            f'<path d="M {x - r},{y - r * 0.5:.1f} Q {x},{y - r * 1.6:.1f} {x + r},{y - r * 0.5:.1f} Z" fill="#6e4a2a" stroke="{OUT}" stroke-width="2.5"/>')


def eichhornpirat():
    """Eichhörnchen mit Kopftuch und Augenklappe, wirft Nüsse."""
    s = f'<path d="M -30,-30 C -90,-40 -96,-130 -40,-130 C -20,-130 -14,-100 -30,-90 C -60,-90 -56,-50 -26,-56" fill="#d9822b" {st(4)}/>'
    s += f'<path d="M -52,-110 q 8,-8 18,-4" fill="none" stroke="#f2b36a" stroke-width="4" stroke-linecap="round"/>'
    s += blob(0, -40, 30, 38, '#d9822b', 30, 0.05)
    s += f'<ellipse cx="6" cy="-30" rx="16" ry="22" fill="#f6dcb0" {st(2.5)}/>'
    s += f'<circle cx="20" cy="-84" r="26" fill="#d9822b" {st(4)}/>'
    s += f'<path d="M 6,-104 L 4,-124 L 18,-108 Z" fill="#d9822b" {st(3)}/>'
    # Kopftuch
    s += f'<path d="M -4,-94 Q 18,-120 44,-96 Q 22,-102 -4,-94 Z" fill="#c8483a" {st(3)}/>'
    s += f'<path d="M -4,-96 L -18,-90 L -10,-84" fill="#c8483a" {st(2.5)}/>'
    for x in (8, 20, 32):
        s += f'<circle cx="{x}" cy="-104" r="2" fill="#ffffff"/>'
    # Augenklappe und Auge
    s += f'<path d="M 0,-92 L 44,-74" stroke="{OUT}" stroke-width="2.5"/><ellipse cx="34" cy="-80" rx="7" ry="8" fill="{OUT}"/>'
    s += angry_eye(14, -82, 6)
    s += f'<circle cx="44" cy="-70" r="3" fill="{OUT}"/>'
    s += f'<path d="M 30,-64 q 6,4 12,0" fill="none" stroke="{OUT}" stroke-width="2.5"/>'
    s += f'<rect x="34" y="-66" width="6" height="7" fill="#ffffff" stroke="{OUT}" stroke-width="1.5"/>'
    # Pfote mit Nuss
    s += f'<path d="M 18,-52 Q 40,-62 44,-44" fill="none" stroke="{OUT}" stroke-width="9" stroke-linecap="round"/>'
    s += f'<path d="M 18,-52 Q 40,-62 44,-44" fill="none" stroke="#d9822b" stroke-width="5" stroke-linecap="round"/>'
    s += nuss(50, -46, 10)
    for x in (-12, 14):
        s += f'<ellipse cx="{x}" cy="-2" rx="12" ry="6" fill="#b8682a" {st(3)}/>'
    return s


def pilzwicht():
    """Pilzwicht: kleiner frecher Pilz mit lila Hut, stößt bunte Sporen aus."""
    s = f'<path d="M -22,0 Q -26,-34 -18,-48 H 18 Q 26,-34 22,0 Z" fill="#f2e6d0" {st(4)}/>'
    s += angry_eye(-7, -30, 6) + angry_eye(11, -30, 6)
    s += f'<path d="M -6,-14 q 8,6 16,0" fill="none" stroke="{OUT}" stroke-width="3" stroke-linecap="round"/>'
    s += f'<path d="M -54,-46 Q -48,-96 0,-100 Q 48,-96 54,-46 Q 0,-58 -54,-46 Z" fill="#a77be0" {st(4.5)}/>'
    for (x, y, r) in ((-26, -72, 8), (8, -84, 10), (34, -64, 7)):
        s += f'<circle cx="{x}" cy="{y}" r="{r}" fill="#f2c14e" stroke="{OUT}" stroke-width="2"/>'
    for k, c in enumerate(('#e8685a', '#f2c14e', '#7fd99a', '#5aaee8', '#a77be0')):
        a = math.radians(-150 + k * 30)
        s += f'<circle cx="{70 * math.cos(a):.1f}" cy="{-90 + 40 * math.sin(a):.1f}" r="6" fill="{c}" opacity="0.8"/>'
    for x in (-10, 10):
        s += f'<ellipse cx="{x}" cy="2" rx="10" ry="5" fill="#8a6040" {st(3)}/>'
    return s


def rausch_elora():
    """Elora im bunten Rausch (E-311): Regenbogenschimmer, Sternchen, langsamer."""
    colors = ('e8685a', 'f2c14e', '7fd99a', '5aaee8', 'a77be0')
    s = ''
    for k, c in enumerate(colors):
        s += f'<g opacity="0.35" transform="translate({(k - 2) * 7},{(k % 2) * 4})">{drop(c, c)}</g>'
    s += drop('f2c14e', 'd9a43a')
    for k in range(5):
        a = math.radians(k * 72 - 90)
        s += star(70 * math.cos(a), -70 + 60 * math.sin(a), 9, '#' + colors[k])
    return s


# ── Hüter ───────────────────────────────────────────────────────────────

def kern(x, y, r=14, lit=True):
    c = AMBER if lit else '#8a7a5a'
    s = f'<circle cx="{x}" cy="{y}" r="{r + 8}" fill="{AMBER}" opacity="0.3"/>' if lit else ''
    s += f'<ellipse cx="{x}" cy="{y}" rx="{r}" ry="{r * 1.2:.1f}" fill="{c}" stroke="{OUT}" stroke-width="3"/>'
    if lit:
        s += f'<path d="M {x - r * 0.4:.1f},{y - r * 0.5:.1f} q 2,-4 6,-4" fill="none" stroke="#fff6d8" stroke-width="3" stroke-linecap="round"/>'
    return s


def branch(path, w=22, fill=None):
    """Ast/Glied: dunkler Umriss, Rinde innen."""
    fill = fill or BARK
    return (f'<path d="{path}" fill="none" stroke="{OUT}" stroke-width="{w + 8}" stroke-linecap="round" stroke-linejoin="round"/>'
            f'<path d="{path}" fill="none" stroke="{fill}" stroke-width="{w}" stroke-linecap="round" stroke-linejoin="round"/>')


def twigs(x, y, ang, n=3, length=34):
    """Zweig-Finger am Ende eines Arms (Winkel in Grad)."""
    s = ''
    for k in range(n):
        a = math.radians(ang + (k - (n - 1) / 2) * 28)
        s += branch(f'M {x},{y} l {length * math.cos(a):.1f},{length * math.sin(a):.1f}', 7)
        s += (f'<ellipse cx="{x + (length + 6) * math.cos(a):.1f}" cy="{y + (length + 6) * math.sin(a):.1f}" '
              f'rx="9" ry="5" fill="{LEAF_LIGHT}" stroke="{OUT}" stroke-width="2" '
              f'transform="rotate({math.degrees(a):.0f} {x + (length + 6) * math.cos(a):.1f} {y + (length + 6) * math.sin(a):.1f})"/>')
    return s


def wurzelwaechter(pose='schlaf'):
    """Wurzelwächter: ein uralter, wandelnder Baumriese (Baumhirte) – Wurzelbeine, lange Astarme mit
    Zweigfingern, Rindengesicht mit schweren Brauen, Knollennase und langem Bart aus Moos und
    Zweigen, Blätterkrone. Drei Kerne in der Brust.
    Posen: schlaf, angriff (Arm erhoben, Wurzeln brechen aus dem Boden), offen (ein Kern gezogen),
    ruhig (nach dem Kampf)."""
    s = ''
    # Beine aus Wurzeln mit gespreizten Zehen
    for side in (-1, 1):
        x = side * 44
        s += branch(f'M {x},-190 Q {x + side * 10},-100 {x + side * 4},-20', 42)
        for k, dx in enumerate((-30, 0, 30)):
            s += branch(f'M {x + side * 4},-20 Q {x + side * 4 + dx * 0.6},-4 {x + side * 4 + dx * 1.4},0', 12)
    # Rumpf: hoher, leicht schiefer Stamm
    s += (f'<path d="M -78,-170 Q -92,-320 -70,-440 Q -40,-470 0,-472 Q 44,-470 72,-440 '
          f'Q 94,-320 80,-170 Q 0,-150 -78,-170 Z" fill="{BARK}" {st(6)}/>')
    for x, c in ((-52, -6), (-20, 4), (18, -4), (50, 6)):
        s += f'<path d="M {x},-180 Q {x + c * 3},-300 {x - c},-440" fill="none" stroke="{BARK_DARK}" stroke-width="3.5"/>'
    s += f'<ellipse cx="34" cy="-210" rx="10" ry="14" fill="{BARK_DARK}" opacity="0.7"/>'
    # Arme: Schulter bei y −410; lang, knorrig
    if pose in ('angriff', 'angriff_ohne'):
        left = 'M -70,-410 Q -150,-470 -160,-560'
        lend = (-160, -560, -90)
        right = 'M 72,-410 Q 150,-330 170,-230'
        rend = (170, -230, 70)
    elif pose == 'offen':
        left = 'M -70,-410 Q -170,-380 -200,-300'
        lend = (-200, -300, 120)
        right = 'M 72,-410 Q 170,-380 200,-300'
        rend = (200, -300, 60)
    elif pose == 'ruhig':
        left = 'M -70,-410 Q -120,-320 -110,-210'
        lend = (-110, -210, 100)
        right = 'M 72,-410 Q 150,-380 160,-330'
        rend = (160, -330, -40)
    else:
        left = 'M -70,-410 Q -130,-310 -118,-190'
        lend = (-118, -190, 95)
        right = 'M 72,-410 Q 130,-310 118,-190'
        rend = (118, -190, 85)
    for path, end in ((left, lend), (right, rend)):
        s += branch(path, 26)
        s += twigs(*end)
    # Kerne in der Brust
    for k, (x, y) in enumerate(((-34, -250), (6, -300), (40, -236))):
        if pose == 'offen' and k == 1:
            s += f'<ellipse cx="{x}" cy="{y}" rx="18" ry="22" fill="#3a2a1a" stroke="{OUT}" stroke-width="3"/>'
            s += f'<ellipse cx="{x}" cy="{y}" rx="10" ry="12" fill="#ffd27a" opacity="0.85"/>'
        else:
            s += kern(x, y, 13, pose != 'ruhig')
    # Kopf: oberer Teil des Stamms, lang gezogen
    s += f'<path d="M -64,-430 Q -74,-540 -30,-580 Q 0,-596 34,-580 Q 76,-540 66,-430 Z" fill="{BARK}" {st(5)}/>'
    # Krone aus Blättern und Zweigen wie Haar
    for (x, y, rx, ry, c) in ((-46, -590, 46, 30, LEAF), (10, -612, 58, 34, LEAF_LIGHT), (58, -586, 42, 28, LEAF),
                              (-70, -556, 28, 22, MOSS), (78, -552, 26, 20, MOSS)):
        s += blob(x, y, rx, ry, c, 26, 0.14)
    for (x, ang) in ((-30, -120), (20, -80), (50, -60)):
        s += branch(f'M {x},-600 l {30 * math.cos(math.radians(ang)):.0f},{30 * math.sin(math.radians(ang)):.0f}', 6)
    # schwere Brauen aus Rinde
    for side in (-1, 1):
        x = side * 26
        lift = -4 if pose.startswith('angriff') else 0
        s += (f'<path d="M {x - side * 26},{-528 + lift * side} Q {x},{-552} {x + side * 26},{-534 - lift * side} '
              f'L {x + side * 24},{-522} Q {x},{-534} {x - side * 22},{-518} Z" fill="{BARK_DARK}" {st(3)}/>')
    # Augen tief in der Rinde
    for side in (-1, 1):
        x = side * 26
        s += f'<ellipse cx="{x}" cy="-508" rx="17" ry="12" fill="#3a2a1a" stroke="{OUT}" stroke-width="3"/>'
        if pose == 'schlaf':
            s += f'<path d="M {x - 12},-508 Q {x},-502 {x + 12},-508" fill="none" stroke="#a8946e" stroke-width="3"/>'
        elif pose == 'ruhig':
            s += f'<ellipse cx="{x}" cy="-507" rx="8" ry="6" fill="#ffd27a"/><circle cx="{x - 2}" cy="-509" r="2" fill="#ffffff"/>'
        else:
            glow = '#ffb84a' if pose.startswith('angriff') else '#ffd27a'
            s += f'<ellipse cx="{x}" cy="-507" rx="7" ry="7" fill="{glow}"/><circle cx="{x + 2}" cy="-506" r="3" fill="{OUT}"/>'
    # Knollennase
    s += f'<path d="M -6,-500 Q -14,-470 0,-462 Q 16,-466 10,-500 Z" fill="#9a6e48" {st(3.5)}/>'
    # Mund im Bart
    roar = f'<path d="M -22,-446 Q 0,-466 22,-446 Q 0,-436 -22,-446 Z" fill="#3a2a1a" {st(3)}/>'
    mouth = {'angriff': roar, 'angriff_ohne': roar,
             'offen': f'<ellipse cx="0" cy="-446" rx="12" ry="9" fill="#3a2a1a" {st(3)}/>',
             'ruhig': f'<path d="M -18,-448 Q 0,-436 18,-448" fill="none" stroke="{OUT}" stroke-width="4" stroke-linecap="round"/>'}
    # langer Bart aus Moos und Zweigen
    beard = (f'<path d="M -58,-470 Q -66,-420 -46,-380 Q -40,-350 -24,-336 Q -16,-360 -8,-330 Q 0,-356 10,-328 '
             f'Q 18,-356 26,-336 Q 42,-352 48,-382 Q 68,-420 60,-470 Q 0,-438 -58,-470 Z" fill="{MOSS}" {st(4)}/>')
    s += beard
    for x in (-40, -20, 0, 20, 40):
        s += f'<path d="M {x},-454 Q {x + 4},-410 {x - 2},-360" fill="none" stroke="{MOSS_DARK}" stroke-width="3"/>'
    s += branch('M -30,-380 l -6,24', 4) + branch('M 32,-376 l 8,22', 4)
    s += mouth.get(pose, f'<path d="M -16,-448 H 16" stroke="{OUT}" stroke-width="4" stroke-linecap="round"/>')
    if pose == 'schlaf':
        s += text(110, -560, 'Z', 34, '#5e4430', weight='bold') + text(140, -592, 'z', 24, '#5e4430', weight='bold')
    if pose == 'angriff':
        for x in (-250, 240):
            s += f'<path d="M {x - 34},0 L {x - 6},-130 L {x + 4},-96 L {x + 30},0 Z" fill="{BARK}" {st(5)}/>'
            s += f'<path d="M {x - 50},0 l 12,-12 l 12,6 l 12,-10 l 14,8 l 12,-8 l 16,16" fill="none" stroke="#7a5a3e" stroke-width="5"/>'
    return s


# ── Quelle, Runen, Deko ─────────────────────────────────────────────────

def rune(lit=True):
    """Erinnerungsrune: moosiger Stein mit leuchtendem Zeichen."""
    s = f'<path d="M -30,0 Q -36,-50 -18,-70 Q 0,-80 18,-70 Q 36,-50 30,0 Z" fill="#9a958a" {st(4)}/>'
    s += f'<path d="M -30,-8 Q -10,-20 6,-10 Q 20,-2 30,-10" fill="none" stroke="{MOSS}" stroke-width="7" stroke-linecap="round"/>'
    c = GLOW if lit else '#6a6a62'
    if lit:
        s += '<circle cx="0" cy="-42" r="22" fill="#9ff2d8" opacity="0.3"/>'
    s += f'<path d="M -10,-56 L 0,-28 L 10,-56 M -12,-42 H 12" fill="none" stroke="{c}" stroke-width="5" stroke-linecap="round"/>'
    return s


def waldquelle(freed=True):
    """Waldquelle zwischen großen Wurzeln: dunkles Becken; befreit klar und leuchtend."""
    s = ''
    if freed:
        s += '<ellipse cx="0" cy="-60" rx="150" ry="90" fill="#9ff2d8" opacity="0.25"/>'
    for (x, dx) in ((-110, -40), (110, 40)):
        s += f'<path d="M {x},0 Q {x},-90 {x + dx},-140" fill="none" stroke="{OUT}" stroke-width="30" stroke-linecap="round"/>'
        s += f'<path d="M {x},0 Q {x},-90 {x + dx},-140" fill="none" stroke="{BARK}" stroke-width="22" stroke-linecap="round"/>'
    s += f'<ellipse cx="0" cy="-14" rx="100" ry="20" fill="{"#5fc8c0" if freed else "#6a6250"}" {st(4)}/>'
    if freed:
        s += '<path d="M -50,-16 q 12,-5 24,0 M 20,-12 q 12,-5 24,0" fill="none" stroke="#ffffff" stroke-width="3" stroke-linecap="round"/>'
        for (x, y) in ((-60, -60), (30, -90), (70, -50), (-20, -110)):
            s += star(x, y, 8, '#d8fff2')
    else:
        s += '<path d="M -40,-16 l 12,6 l 10,-6 l 14,5" fill="none" stroke="#4a4232" stroke-width="2.5"/>'
    s += blob(-80, -8, 30, 10, MOSS if freed else '#8a8a70', 16, 0.2, 3)
    s += blob(84, -8, 26, 9, MOSS if freed else '#8a8a70', 16, 0.2, 3)
    return s


def waldbaum():
    """Hoher Waldbaum (viel Vertikale): breite Wurzeln, gefurchte Rinde mit Astloch und Moos,
    Seitenäste, Krone aus mehreren Blattwolken mit Licht und Schatten, ein paar Blätter."""
    s = ''
    # Wurzeln
    for (x, dx) in ((-34, -50), (-20, -26), (22, 30), (34, 54)):
        s += f'<path d="M {x},-40 Q {x + dx * 0.4},-10 {x + dx},4 L {x + dx * 0.7},4 Q {x + dx * 0.2},-6 {x * 0.6},-20 Z" fill="{BARK}" {st(4)}/>'
    # Stamm, leicht geschwungen
    s += f'<path d="M -40,0 Q -30,-140 -26,-260 Q -24,-360 -18,-430 H 18 Q 26,-360 28,-260 Q 32,-140 42,0 Z" fill="{BARK}" {st(5)}/>'
    # Furchen und Licht auf der Rinde
    for (x0, c, w) in ((-24, -8, 3.5), (-10, 6, 3), (6, -6, 3), (20, 8, 3.5)):
        s += f'<path d="M {x0},-12 Q {x0 + c},-140 {x0 - c * 0.5},-250 Q {x0 + c * 0.6},-330 {x0 * 0.5},-420" fill="none" stroke="{BARK_DARK}" stroke-width="{w}" stroke-linecap="round"/>'
    s += '<path d="M -30,-30 Q -24,-160 -20,-300" fill="none" stroke="#a57a54" stroke-width="5" stroke-linecap="round" opacity="0.7"/>'
    # Astloch und Moos
    s += f'<ellipse cx="10" cy="-190" rx="11" ry="15" fill="#4a3222" {st(3)}/><ellipse cx="10" cy="-186" rx="6" ry="9" fill="#2b1e14"/>'
    s += blob(-26, -24, 22, 10, MOSS, 14, 0.25, 3) + blob(30, -110, 14, 8, MOSS, 12, 0.25, 2.5)
    # Seitenäste
    for (x, y, dx, dy) in ((-20, -330, -90, -60), (22, -360, 100, -50), (-18, -400, -60, -80)):
        s += f'<path d="M {x},{y} q {dx * 0.5},{dy * 0.2} {dx},{dy}" fill="none" stroke="{OUT}" stroke-width="18" stroke-linecap="round"/>'
        s += f'<path d="M {x},{y} q {dx * 0.5},{dy * 0.2} {dx},{dy}" fill="none" stroke="{BARK}" stroke-width="11" stroke-linecap="round"/>'
    # Krone: dunkle Schatten hinten, helle Wolken vorn, Lichtkanten
    back = ((-110, -430, 70, 52), (110, -440, 76, 54), (-60, -520, 84, 62), (60, -530, 90, 64), (0, -600, 86, 56))
    for (x, y, rx, ry) in back:
        s += blob(x, y, rx, ry, '#3f7a30', 32, 0.12)
    front = ((-80, -470, 60, 44, LEAF), (80, -480, 64, 46, LEAF), (-20, -540, 70, 52, LEAF_LIGHT),
             (40, -580, 60, 44, LEAF_LIGHT), (-90, -540, 46, 36, LEAF), (0, -460, 56, 40, LEAF))
    for (x, y, rx, ry, c) in front:
        s += blob(x, y, rx, ry, c, 28, 0.12)
        s += f'<path d="M {x - rx * 0.6},{y - ry * 0.3} q {rx * 0.3},{-ry * 0.5} {rx * 0.7},{-ry * 0.4}" fill="none" stroke="#9fdc7a" stroke-width="5" stroke-linecap="round" opacity="0.8"/>'
    # einzelne Blätter und Beeren
    for (x, y, a) in ((-130, -380, 30), (126, -392, -20), (-40, -620, 10), (70, -620, -30)):
        s += f'<ellipse cx="{x}" cy="{y}" rx="10" ry="5" fill="{LEAF_LIGHT}" stroke="{OUT}" stroke-width="2" transform="rotate({a} {x} {y})"/>'
    for (x, y) in ((-40, -500), (30, -520), (-90, -450), (90, -470)):
        s += f'<circle cx="{x}" cy="{y}" r="5" fill="#e8685a" stroke="{OUT}" stroke-width="1.6"/>'
    return s


def baumhaus():
    """Baumhaus der Siedlung mit Plattform und Tür."""
    s = f'<rect x="-90" y="-12" width="180" height="12" fill="#a87a52" {st(4)}/>'
    for x in (-80, 80):
        s += f'<path d="M {x},0 V 40" stroke="{OUT}" stroke-width="8"/><path d="M {x},0 V 40" stroke="#a87a52" stroke-width="4"/>'
    s += f'<rect x="-60" y="-100" width="120" height="88" rx="6" fill="#c79a66" {st(4)}/>'
    for x in range(-48, 60, 16):
        s += f'<path d="M {x},-100 V -12" stroke="#a87a52" stroke-width="2"/>'
    s += f'<path d="M -76,-96 L 0,-150 L 76,-96 Z" fill="{MOSS}" {st(4)}/>'
    s += f'<path d="M -12,-12 V -54 Q 0,-66 12,-54 V -12 Z" fill="#7a5434" {st(3)}/>'
    s += f'<circle cx="34" cy="-64" r="12" fill="#ffd27a" {st(3)}/>'
    s += f'<path d="M 34,-76 V -52 M 22,-64 H 46" stroke="{OUT}" stroke-width="2"/>'
    return s


def haengebruecke():
    s = f'<path d="M -150,-60 Q 0,-10 150,-60" fill="none" stroke="#8a7050" stroke-width="3"/>'
    for k in range(13):
        x = -144 + k * 24
        y = -20 + 28 * (1 - (x / 150) ** 2) * 0.9 - 28
        s += f'<rect x="{x - 9}" y="{y:.1f}" width="18" height="8" rx="2" fill="#a87a52" {st(2)}/>'
        s += f'<path d="M {x},{y:.1f} V {-60 + 50 * (1 - (x / 150) ** 2) * 0.9:.1f}" stroke="#8a7050" stroke-width="1.6"/>'
    return s


def leuchtpilze():
    """Leuchtpilze: kräftiger Lichtschein, leuchtende Hüte mit hellen Punkten (Sternchen steigen
    im Spiel als Teilchen auf)."""
    s = '<ellipse cx="0" cy="-26" rx="70" ry="44" fill="#9ff2ff" opacity="0.22"/>'
    s += '<ellipse cx="0" cy="-22" rx="46" ry="28" fill="#c8f8ff" opacity="0.3"/>'
    for (x, h, r, c) in ((-26, 26, 14, '#5fd8e8'), (0, 40, 19, '#8ff0ff'), (24, 22, 12, '#5fd8e8'), (-12, 14, 8, '#8ff0ff')):
        s += f'<path d="M {x},0 V {-h}" stroke="{OUT}" stroke-width="7"/><path d="M {x},0 V {-h}" stroke="#e8fbff" stroke-width="3.5"/>'
        s += f'<path d="M {x - r},{-h} Q {x},{-h - r * 1.5} {x + r},{-h} Z" fill="{c}" {st(3)}/>'
        s += f'<path d="M {x - r * 0.6},{-h - r * 0.3} Q {x},{-h - r * 1.1} {x + r * 0.5},{-h - r * 0.5}" fill="none" stroke="#ffffff" stroke-width="2.5" stroke-linecap="round" opacity="0.9"/>'
        s += f'<circle cx="{x - r * 0.3}" cy="{-h - r * 0.55}" r="2.4" fill="#ffffff"/>'
    for (x, y) in ((-44, -54), (36, -64), (8, -78)):
        s += star(x, y, 5, '#bff4ff')
    return s


def wurzelbogen():
    s = f'<path d="M -120,0 Q -110,-140 0,-150 Q 110,-140 120,0" fill="none" stroke="{OUT}" stroke-width="34" stroke-linecap="round"/>'
    s += f'<path d="M -120,0 Q -110,-140 0,-150 Q 110,-140 120,0" fill="none" stroke="{BARK}" stroke-width="26" stroke-linecap="round"/>'
    s += blob(-40, -150, 40, 14, MOSS, 18, 0.18, 3) + blob(50, -146, 30, 12, MOSS, 16, 0.2, 3)
    return s


# ── Bogen ───────────────────────────────────────────────────────────────

def sheet():
    W, H = 1600, 1740
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
         '<defs><linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#8fb7a8"/>'
         '<stop offset="1" stop-color="#e4eee6"/></linearGradient></defs>',
         f'<rect width="{W}" height="{H}" fill="url(#sky)"/>',
         text(40, 50, 'Kapitel 2 „Murmelwald“ – Entwürfe (R2-M2.2, M2.2.0)', 28, TEXT, 'start', 'bold'),
         text(40, 78, 'Figuren, Gegner, Hüter, Gegenstände und Deko des Waldes im Stil von Kapitel 1. '
              'Elora klein daneben als Maßstab.', 15, TEXT, 'start')]

    o.append(card(30, 100, 760, 400, 'Figuren'))
    o.append(ground(60, 410, 300) + g(180, 410, 1.0, plumm()) + elora(320, 410))
    o.append(text(190, 446, 'Uhu Plumm', 17, TEXT, weight='bold'))
    o.append(text(190, 468, 'sammelt Geschichten, Brille, Buch', 12, DIM, style='italic'))
    o.append(ground(430, 410, 330) + g(510, 410, 0.55, pilzkind(True)) + g(680, 410, 0.55, pilzkind(False)))
    o.append(text(595, 446, 'Pilzkind verirrt / froh', 17, TEXT, weight='bold'))
    o.append(text(595, 468, 'folgt Elora nach Hause (E-308)', 12, DIM, style='italic'))

    o.append(card(810, 100, 760, 400, 'Gegenstände und Quelle'))
    o.append(ground(840, 410, 200) + g(900, 410, 1.0, rune(True)) + g(990, 410, 1.0, rune(False)))
    o.append(text(945, 446, 'Erinnerungsrune', 15, TEXT, weight='bold'))
    o.append(text(945, 466, 'leuchtet, bis Elora sie liest', 12, DIM))
    o.append(kern(1100, 360, 20) + text(1100, 446, 'Kern', 15, TEXT, weight='bold'))
    o.append(text(1100, 466, 'aus der Rinde gezogen', 12, DIM))
    o.append(ground(1180, 410, 370) + g(1270, 410, 0.55, waldquelle(False)) + g(1460, 410, 0.55, waldquelle(True)))
    o.append(text(1365, 446, 'Waldquelle verdorrt / befreit', 15, TEXT, weight='bold'))

    o.append(card(30, 520, 1540, 380, 'Gegner'))
    o.append(ground(60, 820, 340) + g(140, 820, 0.9, wurzelschlange(False)) + g(300, 820, 0.9, wurzelschlange(True)))
    o.append(text(220, 856, 'Wurzelschlange', 17, TEXT, weight='bold'))
    o.append(text(220, 878, 'versteckt / schießt aus dem Boden', 12, DIM))
    o.append(ground(460, 820, 320) + g(580, 820, 1.0, eichhornpirat()) + nuss(700, 700) + nuss(740, 730, 9))
    o.append('<path d="M 640,760 Q 690,660 740,720" fill="none" stroke="#a8946e" stroke-width="2" stroke-dasharray="5 6"/>')
    o.append(text(620, 856, 'Eichhornpirat', 17, TEXT, weight='bold'))
    o.append(text(620, 878, 'wirft Nüsse im Bogen', 12, DIM))
    o.append(ground(840, 820, 320) + g(930, 820, 1.0, pilzwicht()) + g(1080, 820, 0.36, rausch_elora()))
    o.append(text(1000, 856, 'Pilzwicht und der bunte Rausch', 17, TEXT, weight='bold'))
    o.append(text(1000, 878, 'Elora schimmert bunt, läuft langsamer (E-311)', 12, DIM))
    o.append(elora(1250, 820) + text(1250, 856, 'Maßstab', 13, DIM))

    o.append(card(30, 920, 1540, 440, 'Hüter: Wurzelwächter (E-307)'))
    for x, pose, name, note in ((230, 'schlaf', 'schläft auf der Quelle', 'wacht auf, wenn Elora kommt'),
                                (610, 'angriff', 'Wurzelangriff', 'Boden bebt vorher'),
                                (990, 'offen', 'Kern gezogen', 'jetzt verwundbar'),
                                (1360, 'ruhig', 'nach dem Kampf', 'müde, nicht böse')):
        o.append(ground(x - 170, 1290, 340) + g(x, 1290, 0.5, wurzelwaechter(pose)))
        o.append(text(x, 1322, name, 17, TEXT, weight='bold'))
        o.append(text(x, 1344, note, 12, DIM))
    o.append(elora(1500, 1290))

    o.append(card(30, 1380, 1540, 330, 'Deko des Waldes'))
    o.append(ground(60, 1665, 1480))
    o.append(g(150, 1665, 0.34, waldbaum()) + g(380, 1625, 0.75, baumhaus()))
    o.append(g(650, 1645, 0.8, haengebruecke()) + g(880, 1665, 1.0, leuchtpilze()) + g(1100, 1665, 0.8, wurzelbogen()))
    o.append(elora(1300, 1665))
    for x, t in ((140, 'Waldbaum'), (380, 'Baumhaus'), (650, 'Hängebrücke'), (880, 'Leuchtpilze'), (1100, 'Wurzelbogen')):
        o.append(text(x, 1694, t, 14, TEXT, weight='bold'))
    o.append('</svg>')
    return '\n'.join(o)


def pilzring():
    """Ring aus Pilzen (Zuhause der Familie Morchel), flach am Boden."""
    s = '<ellipse cx="0" cy="-6" rx="120" ry="16" fill="#9ff2d8" opacity="0.2"/>'
    for k, (x, h, r, c) in enumerate(((-110, 18, 12, '#e8685a'), (-80, 26, 15, '#a8703c'), (-46, 20, 12, '#e8685a'),
                                      (-14, 30, 16, '#5fd8c8'), (20, 22, 13, '#a8703c'), (54, 28, 15, '#e8685a'),
                                      (86, 20, 12, '#5fd8c8'), (112, 16, 11, '#a8703c'))):
        s += f'<path d="M {x},0 V {-h}" stroke="{OUT}" stroke-width="7"/><path d="M {x},0 V {-h}" stroke="#f2e6d0" stroke-width="3.5"/>'
        s += f'<path d="M {x - r},{-h} Q {x},{-h - r * 1.4} {x + r},{-h} Z" fill="{c}" {st(3)}/>'
        if c == '#e8685a':
            s += f'<circle cx="{x - 4}" cy="{-h - 6}" r="2.5" fill="#fff6e8"/>'
    return s


# Deko für die Karten von Kapitel 2 (E-312): Name: (Zeichnung, viewBox), Ursprung unten in der Mitte.
DECOR = {
    'waldbaum': (waldbaum(), '-200 -680 400 690'),
    'waldhaus': (baumhaus(), '-100 -160 200 204'),
    'haengebruecke': (haengebruecke(), '-160 -70 320 72'),
    'leuchtpilze': (leuchtpilze(), '-74 -92 148 94'),
    'wurzelbogen': (wurzelbogen(), '-140 -180 280 182'),
    'pilzring': (pilzring(), '-130 -56 260 58'),
    'waldquelle-verdorrt': (waldquelle(False), '-160 -160 320 162'),
    'waldquelle-befreit': (waldquelle(True), '-160 -160 320 162'),
}


def export_decor():
    for name, (art, vb) in DECOR.items():
        with open(f'assets/map/decor/{name}.svg', 'w') as f:
            f.write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}">\n'
                    f'  <!-- Kapitel 2: {name} (R2-M2.2, aus tools/design/kapitel2_entwuerfe.py). Ursprung unten in der Mitte. -->\n'
                    f'  {art}\n</svg>\n')


if __name__ == '__main__':
    os.makedirs('docs/release-2/design', exist_ok=True)
    open('docs/release-2/design/kapitel2-entwuerfe.svg', 'w').write(sheet())
    export_decor()

"""Entwürfe für Kapitel 4 „Frostspitzen“ (R2-M2.4, Schritt M2.4.0, E-340 bis E-343).

Aufruf: python3 tools/design/kapitel4_entwuerfe.py && cargo xtask svg-preview \
        docs/release-2/design/kapitel4-entwuerfe.svg docs/release-2/design/kapitel4-entwuerfe.png 1600

Stil wie Kapitel 1 bis 3: dunkler Umriss, flache Farben; hier kühle Eisblau-, Schnee- und
Holztöne. Gegner frech, nicht blutig; Kristella edel und streng, nach dem Kampf sanft.
Die Deko für die Karten schreibt erst M2.4.6 (`--decor`).
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from abenteuer_figuren import DIM, OUT, TEXT, angry_eye, drop, g, star, text  # noqa: E402
from kapitel1_entwuerfe import card, elora  # noqa: E402
from kapitel2_entwuerfe import blob  # noqa: E402

SNOW = '#f6fafd'
SNOW_SHADE = '#d4e2ee'
ICE = '#bfe6f5'
ICE_MID = '#8fcde8'
ICE_DARK = '#5fa8d0'
FROST = '#e8f6ff'
DEEP = '#2f6f9a'
WOOD = '#8a5a3a'
WOOD_DARK = '#5e3c24'
PINE = '#3f7a5a'
PINE_DARK = '#2c5a42'
FIRE = '#ff9a3c'
FIRE_HOT = '#ffd36a'
ROCK = '#8f9aa6'
ROCK_DARK = '#66717c'
WATER = '#3f8fc0'


def st(w=3.5):
    return f'stroke="{OUT}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'


def ground(x, y, w):
    """Schneeboden statt Wiese."""
    return (f'<rect x="{x}" y="{y}" width="{w}" height="8" rx="4" fill="{SNOW_SHADE}"/>'
            f'<rect x="{x}" y="{y}" width="{w}" height="3" rx="1.5" fill="#ffffff"/>')


def flake(cx, cy, r, color='#ffffff', w=2.2):
    """Schneeflocke mit sechs Armen."""
    s = ''
    for k in range(6):
        a = math.pi / 3 * k
        x, y = cx + r * math.cos(a), cy + r * math.sin(a)
        s += f'<path d="M {cx:.1f},{cy:.1f} L {x:.1f},{y:.1f}" stroke="{color}" stroke-width="{w}" stroke-linecap="round"/>'
        bx, by = cx + r * 0.6 * math.cos(a), cy + r * 0.6 * math.sin(a)
        for d in (-0.6, 0.6):
            ex, ey = bx + r * 0.3 * math.cos(a + d), by + r * 0.3 * math.sin(a + d)
            s += f'<path d="M {bx:.1f},{by:.1f} L {ex:.1f},{ey:.1f}" stroke="{color}" stroke-width="{w * 0.8:.1f}" stroke-linecap="round"/>'
    return s


def crystal(cx, cy, h, fill=ICE, sw=3):
    """Eiskristall (Säule mit Spitze)."""
    w = h * 0.32
    return (f'<path d="M {cx - w},{cy} L {cx - w},{cy - h * 0.7:.1f} L {cx},{cy - h} L {cx + w},{cy - h * 0.7:.1f} L {cx + w},{cy} Z" '
            f'fill="{fill}" {st(sw)}/>'
            f'<path d="M {cx},{cy - h} L {cx},{cy}" stroke="#ffffff" stroke-width="{sw * 0.8:.1f}" opacity="0.7"/>')


# ── Figuren ─────────────────────────────────────────────────────────────

def flocke():
    """Bergführerin Flocke: Pudelmütze mit Bommel, langer Schal, Seil über der Schulter, Eispickel."""
    s = f'<path d="M 58,-120 V -4" stroke="{WOOD_DARK}" stroke-width="7" stroke-linecap="round"/>'
    s += f'<path d="M 58,-120 V -4" stroke="#b07a4a" stroke-width="3.5" stroke-linecap="round"/>'
    s += f'<path d="M 40,-118 Q 58,-132 76,-122 L 72,-114 Q 58,-120 46,-110 Z" fill="{ROCK}" {st(3)}/>'
    s += drop('9fd0ea', '6aa6cc')
    # Seil über der Schulter
    s += f'<path d="M -44,-70 Q 0,-20 44,-30" fill="none" stroke="{OUT}" stroke-width="9" stroke-linecap="round"/>'
    s += f'<path d="M -44,-70 Q 0,-20 44,-30" fill="none" stroke="#d9a066" stroke-width="5" stroke-linecap="round" stroke-dasharray="6 4"/>'
    # Schal
    s += f'<path d="M -40,-50 Q 0,-36 40,-50 L 40,-40 Q 0,-26 -40,-40 Z" fill="#e8685a" {st(3)}/>'
    s += f'<path d="M 28,-42 L 36,-6 L 22,-6 L 18,-40 Z" fill="#e8685a" {st(3)}/>'
    s += '<path d="M 24,-30 h 10 M 26,-18 h 10" stroke="#ffffff" stroke-width="3"/>'
    # Mütze mit Bommel
    s += f'<path d="M -46,-108 Q -44,-160 0,-162 Q 44,-160 46,-108 Q 0,-118 -46,-108 Z" fill="#e8685a" {st(4)}/>'
    s += f'<path d="M -48,-112 Q 0,-124 48,-112 L 48,-100 Q 0,-112 -48,-100 Z" fill="#ffffff" {st(3)}/>'
    s += blob(0, -170, 14, 13, '#ffffff', n=24, jag=0.12, w=3)
    for x in (-24, 0, 24):
        s += f'<path d="M {x},-150 v 30" stroke="#c8483a" stroke-width="3"/>'
    return s


def kletterer(body='f2a65a', feet='c97f3a', helmet='#f2c14e'):
    """Verlorener Kletterer: Helm mit Lampe, Rucksack, winkt um Hilfe."""
    s = f'<rect x="-62" y="-92" width="26" height="56" rx="9" fill="{WOOD}" {st(3)}/>'
    s += drop(body, feet)
    s += f'<path d="M -44,-110 Q -42,-150 0,-152 Q 42,-150 44,-110 Z" fill="{helmet}" {st(4)}/>'
    s += f'<rect x="-10" y="-150" width="20" height="14" rx="4" fill="#fff7c0" {st(2.5)}/>'
    # winkender Arm
    s += f'<path d="M 40,-70 Q 70,-90 66,-128" fill="none" stroke="{OUT}" stroke-width="12" stroke-linecap="round"/>'
    s += f'<path d="M 40,-70 Q 70,-90 66,-128" fill="none" stroke="#{body}" stroke-width="7" stroke-linecap="round"/>'
    s += f'<path d="M 58,-140 q 8,-6 16,0 M 54,-150 q 12,-10 24,0" fill="none" stroke="{DIM}" stroke-width="2.5"/>'
    return s


# ── Gegner ──────────────────────────────────────────────────────────────

def robbe(pose='rutschen'):
    """Schneeballrobbe: rundlich, Bauch hell; `rutschen` auf dem Bauch, `werfen` aufgerichtet mit Schneeball."""
    if pose == 'rutschen':
        s = f'<path d="M -70,-6 Q -74,-44 -20,-46 Q 40,-50 60,-30 Q 74,-18 70,-6 Q 0,4 -70,-6 Z" fill="#9aaab8" {st(4)}/>'
        s += f'<path d="M -60,-8 Q 0,-2 64,-10 Q 40,-20 0,-18 Q -40,-18 -60,-8 Z" fill="{SNOW}" stroke="none"/>'
        s += f'<path d="M -70,-20 l -20,-14 l 4,22 Z" fill="#8a9aa8" {st(3)}/>'
        s += f'<path d="M 20,-8 l 10,10 l 10,-10" fill="#8a9aa8" {st(3)}/>'
        s += angry_eye(44, -34, 6) + f'<circle cx="62" cy="-24" r="4" fill="{OUT}"/>'
        s += '<path d="M 52,-20 l -14,-2 M 52,-17 l -14,4" stroke="#5a6470" stroke-width="2"/>'
        for x in (-110, -96, -84):
            s += f'<path d="M {x},-4 h 14" stroke="{ICE_DARK}" stroke-width="3" stroke-linecap="round"/>'
        return s
    s = f'<path d="M -40,0 Q -50,-60 -20,-96 Q 0,-110 20,-96 Q 50,-60 40,0 Q 0,8 -40,0 Z" fill="#9aaab8" {st(4)}/>'
    s += f'<path d="M -26,-4 Q -30,-50 0,-70 Q 30,-50 26,-4 Z" fill="{SNOW}"/>'
    s += f'<path d="M -38,-4 l -18,8 l 22,2 Z M 38,-4 l 18,8 l -22,2 Z" fill="#8a9aa8" {st(3)}/>'
    # Flosse mit Schneeball über dem Kopf
    s += f'<path d="M 30,-70 Q 54,-96 46,-126" fill="none" stroke="{OUT}" stroke-width="13" stroke-linecap="round"/>'
    s += f'<path d="M 30,-70 Q 54,-96 46,-126" fill="none" stroke="#8a9aa8" stroke-width="8" stroke-linecap="round"/>'
    s += f'<circle cx="44" cy="-140" r="16" fill="#ffffff" {st(3)}/>'
    s += f'<path d="M 36,-146 q 6,-4 12,0" fill="none" stroke="{SNOW_SHADE}" stroke-width="3"/>'
    s += angry_eye(-10, -80, 6) + angry_eye(12, -80, 6)
    s += f'<circle cx="0" cy="-66" r="4" fill="{OUT}"/>'
    s += '<path d="M -4,-62 l -16,0 M 4,-62 l 16,0" stroke="#5a6470" stroke-width="2"/>'
    return s


def fledermaus(pose='haengt', bar=True):
    """Eisspitzen-Fledermaus: Flügel mit Eiszacken; `haengt` schlafend kopfüber (mit `bar` die
    Felsdecke dazu), `sturz` im Sturzflug."""
    if pose == 'haengt':
        s = f'<rect x="-60" y="-4" width="120" height="10" fill="{ROCK}" {st(3)}/>' if bar else ''
        s += f'<path d="M -6,6 v 10 M 6,6 v 10" stroke="{OUT}" stroke-width="3.5"/>'
        s += f'<path d="M -26,16 Q -34,60 0,74 Q 34,60 26,16 Z" fill="#6a7aa0" {st(4)}/>'
        for x, h in ((-20, 18), (-8, 26), (8, 22), (20, 16)):
            s += f'<path d="M {x - 5},{60 - (abs(x) // 4)} L {x},{60 + h} L {x + 5},{58 - (abs(x) // 4)} Z" fill="{ICE}" {st(2)}/>'
        s += f'<path d="M -10,40 q 4,4 8,0 M 2,40 q 4,4 8,0" fill="none" stroke="{OUT}" stroke-width="2.5"/>'
        s += f'<path d="M -10,20 l -6,-10 l 10,4 M 10,20 l 6,-10 l -10,4" fill="#6a7aa0" {st(2)}/>'
        s += text(34, 34, 'z', 14, DIM) + text(44, 22, 'z', 11, DIM)
        return s
    s = ''
    for side in (-1, 1):
        s += (f'<path d="M 0,-40 Q {side * 40},-90 {side * 86},-70 L {side * 76},-56 L {side * 64},-64 L {side * 54},-48 '
              f'L {side * 40},-58 L {side * 30},-40 Q {side * 14},-36 0,-34 Z" fill="#6a7aa0" {st(3.5)}/>')
        for k in range(3):
            x = side * (36 + k * 14)
            s += f'<path d="M {x - 4},{-58 + k * 3} L {x},{-40 + k * 3} L {x + 4},{-58 + k * 3} Z" fill="{ICE}" {st(1.8)}/>'
    s += f'<ellipse cx="0" cy="-36" rx="16" ry="20" fill="#5a6a90" {st(3.5)}/>'
    s += f'<path d="M -10,-52 l -6,-14 l 10,6 M 10,-52 l 6,-14 l -10,6" fill="#5a6a90" {st(2.5)}/>'
    s += angry_eye(-6, -40, 4.5) + angry_eye(6, -40, 4.5)
    s += f'<path d="M -4,-26 l 2,5 l 2,-5 M 2,-26 l 2,5 l 2,-5" fill="#ffffff" stroke="{OUT}" stroke-width="1.5"/>'
    for y in (-90, -110):
        s += f'<path d="M -20,{y} v 18 M 0,{y - 8} v 18 M 20,{y} v 18" stroke="{ICE_DARK}" stroke-width="2.5" opacity="0.7"/>'
    return s


def frostgeist(in_wall=False):
    """Frostgeist: halb durchsichtiger Nebelleib mit Eiskrone, schwebt durch Wände."""
    s = ''
    if in_wall:
        s += f'<rect x="-70" y="-150" width="60" height="150" fill="{ROCK}" {st(4)}/>'
        s += f'<path d="M -66,-110 h 52 M -66,-60 h 52" stroke="{ROCK_DARK}" stroke-width="3"/>'
    s += '<g opacity="0.82">'
    s += (f'<path d="M -36,-60 Q -40,-120 0,-124 Q 40,-120 36,-60 Q 34,-30 22,-16 Q 14,-30 8,-12 Q 0,-28 -8,-12 '
          f'Q -14,-30 -22,-16 Q -34,-30 -36,-60 Z" fill="{FROST}" {st(4)}/>')
    s += f'<path d="M -26,-70 Q -24,-110 0,-112" fill="none" stroke="#ffffff" stroke-width="5" opacity="0.8"/>'
    s += '</g>'
    for x, h in ((-18, 14), (0, 22), (18, 14)):
        s += f'<path d="M {x - 7},-120 L {x},{-120 - h} L {x + 7},-120 Z" fill="{ICE_MID}" {st(2.5)}/>'
    s += f'<ellipse cx="-12" cy="-84" rx="7" ry="9" fill="{DEEP}"/><ellipse cx="12" cy="-84" rx="7" ry="9" fill="{DEEP}"/>'
    s += '<circle cx="-10" cy="-87" r="2.5" fill="#ffffff"/><circle cx="14" cy="-87" r="2.5" fill="#ffffff"/>'
    s += f'<path d="M -8,-66 Q 0,-72 8,-66" fill="none" stroke="{DEEP}" stroke-width="3" stroke-linecap="round"/>'
    for (x, y, r) in ((40, -90, 7), (-46, -40, 6), (34, -30, 5)):
        s += flake(x, y, r, ICE_DARK, 1.8)
    return s


def erstarrt():
    """Elora kurz erstarrt (Berührung des Frostgeists): im Eisblock."""
    s = elora(0, 0, 0.36)
    s += f'<rect x="-24" y="-46" width="48" height="50" rx="6" fill="{ICE}" fill-opacity="0.6" {st(3)}/>'
    s += '<path d="M -16,-38 l 10,0 M 10,-40 l 6,6" stroke="#ffffff" stroke-width="3" stroke-linecap="round"/>'
    return s


# ── Hüterin ─────────────────────────────────────────────────────────────

def kristella(pose='schwebend'):
    """Eiskönigin Kristella: großes eisblaues Wesen mit Kristallkrone und Eisschleppe.
    Posen: schwebend, hauch (Frosthauch auf den Boden), erschoepft (herabgesunken), ruhig."""
    lift = {'schwebend': -70, 'hauch': -80, 'erschoepft': 0, 'ruhig': -20}[pose]
    s = ''
    if pose in ('schwebend', 'hauch'):
        s += f'<ellipse cx="0" cy="-6" rx="70" ry="9" fill="{ICE_DARK}" opacity="0.25"/>'
    o = f'<g transform="translate(0,{lift})">'
    # Schleppe aus Eis
    hem = ' '.join(f'L {x},{(-8 if i % 2 else 10)}' for i, x in enumerate(range(-90, 91, 18)))
    o += f'<path d="M -70,-120 Q -96,-40 -90,0 {hem} Q 96,-40 70,-120 Z" fill="{ICE_MID}" {st(4.5)}/>'
    o += f'<path d="M -50,-110 Q -66,-40 -60,-4 M 50,-110 Q 66,-40 60,-4 M 0,-100 V -4" fill="none" stroke="#ffffff" stroke-width="3" opacity="0.6"/>'
    # Körper (Tropfen)
    o += f'<path d="M 0,-250 Q 70,-170 70,-130 Q 70,-80 0,-80 Q -70,-80 -70,-130 Q -70,-170 0,-250 Z" fill="{ICE}" {st(5)}/>'
    o += f'<path d="M -30,-200 Q -48,-160 -44,-130" fill="none" stroke="#ffffff" stroke-width="7" stroke-linecap="round" opacity="0.8"/>'
    # Arme / Ärmel
    arms = {'schwebend': ((-60, -130, -110, -170), (60, -130, 110, -170)),
            'hauch': ((-60, -130, -100, -110), (60, -130, 100, -110)),
            'erschoepft': ((-60, -120, -90, -86), (60, -120, 90, -86)),
            'ruhig': ((-56, -120, -20, -104), (56, -120, 20, -104))}[pose]
    for (x0, y0, x1, y1) in arms:
        o += f'<path d="M {x0},{y0} Q {(x0 + x1) / 2},{min(y0, y1) - 20} {x1},{y1}" fill="none" stroke="{OUT}" stroke-width="20" stroke-linecap="round"/>'
        o += f'<path d="M {x0},{y0} Q {(x0 + x1) / 2},{min(y0, y1) - 20} {x1},{y1}" fill="none" stroke="{ICE_MID}" stroke-width="13" stroke-linecap="round"/>'
    # Krone
    tilt = 'rotate(-14 0 -236)' if pose == 'erschoepft' else ''
    crown = ''
    for x, h in ((-30, 30), (-15, 44), (0, 58), (15, 44), (30, 30)):
        crown += f'<path d="M {x - 9},-226 L {x},{-226 - h} L {x + 9},-226 Z" fill="{FROST}" {st(3)}/>'
    crown += f'<path d="M -40,-222 Q 0,-236 40,-222 L 38,-212 Q 0,-224 -38,-212 Z" fill="{ICE_DARK}" {st(3)}/>'
    crown += f'<circle cx="0" cy="-222" r="7" fill="#a77be0" {st(2.5)}/>'
    o += f'<g transform="{tilt}">{crown}</g>'
    # Gesicht
    if pose == 'erschoepft':
        o += f'<path d="M -30,-150 q 10,8 20,0 M 10,-150 q 10,8 20,0" fill="none" stroke="{OUT}" stroke-width="4" stroke-linecap="round"/>'
        o += f'<path d="M -10,-120 q 10,-6 20,0" fill="none" stroke="{OUT}" stroke-width="3.5" stroke-linecap="round"/>'
    elif pose == 'ruhig':
        o += f'<path d="M -30,-148 q 10,-10 20,0 M 10,-148 q 10,-10 20,0" fill="none" stroke="{OUT}" stroke-width="4" stroke-linecap="round"/>'
        o += f'<path d="M -12,-122 q 12,10 24,0" fill="none" stroke="{OUT}" stroke-width="3.5" stroke-linecap="round"/>'
        o += '<ellipse cx="-34" cy="-130" rx="8" ry="4" fill="#ef9fc0" opacity="0.7"/><ellipse cx="34" cy="-130" rx="8" ry="4" fill="#ef9fc0" opacity="0.7"/>'
    else:
        o += angry_eye(-20, -150, 10) + angry_eye(20, -150, 10, brow=-1)
        if pose == 'hauch':
            o += f'<ellipse cx="0" cy="-118" rx="10" ry="8" fill="{DEEP}" {st(2.5)}/>'
        else:
            o += f'<path d="M -12,-118 L 12,-118" stroke="{OUT}" stroke-width="3.5" stroke-linecap="round"/>'
    o += '</g>'
    s += o
    if pose == 'hauch':
        # Frosthauch nach unten, am Boden kriecht die Frostwelle
        s += f'<path d="M -6,{lift - 112} Q -40,{lift - 40} -120,-10 L -60,-6 Q -10,{lift - 30} 6,{lift - 110} Z" fill="{FROST}" fill-opacity="0.8" {st(2.5)}/>'
        for (x, y) in ((-40, lift - 60), (-80, -40), (-20, lift - 30)):
            s += flake(x, y, 7, ICE_DARK, 2)
    if pose == 'erschoepft':
        for k in range(3):
            a = math.radians(k * 120)
            s += star(56 * math.cos(a), -280 + 10 * math.sin(a), 10, '#bfe6f5')
    if pose == 'ruhig':
        for (x, y) in ((-100, -200), (96, -180), (110, -100)):
            s += flake(x, y, 9, ICE_DARK, 2.2)
    return s


def frostwelle():
    """Frostwelle über den Boden: Reif kriecht voraus (Warnung), dahinter frischer Frost (schadet), Eis bleibt."""
    s = f'<rect x="-200" y="0" width="400" height="14" fill="{SNOW_SHADE}" {st(3)}/>'
    s += f'<rect x="-200" y="0" width="150" height="14" fill="{ICE}" {st(3)}/>'
    s += '<path d="M -190,4 h 30 M -130,8 h 40" stroke="#ffffff" stroke-width="2.5"/>'
    # frischer Frost: Zacken
    for x in range(-50, 60, 14):
        h = 26 - abs(x - 5) * 0.2
        s += f'<path d="M {x - 7},0 L {x},{-h:.0f} L {x + 7},0 Z" fill="{FROST}" {st(2.5)}/>'
    # Reif kriecht voraus
    for x in range(64, 140, 12):
        s += f'<path d="M {x},0 l 4,-7 l 4,7" fill="none" stroke="{ICE_DARK}" stroke-width="2" opacity="{max(0.25, 1 - (x - 64) / 90):.2f}"/>'
    s += f'<path d="M 50,-36 l 30,0 l -8,-6 m 8,6 l -8,6" fill="none" stroke="{ICE_DARK}" stroke-width="3" stroke-linecap="round"/>'
    return s


# ── Gelände und Gegenstände ─────────────────────────────────────────────

def eiszapfen(state='haengt'):
    """Eiszapfen an der Decke: hängt, zittert (Warnung), fällt, zerschellt."""
    s = f'<rect x="-34" y="-150" width="68" height="14" fill="{ROCK}" {st(3)}/>'
    if state == 'zerschellt':
        for (x, y, a) in ((-24, -4, -30), (-6, -10, 15), (12, -6, -10), (26, -2, 40)):
            s += f'<path d="M {x - 5},{y} l 5,-12 l 5,12 Z" fill="{ICE}" {st(2)} transform="rotate({a} {x} {y})"/>'
        for (x, y) in ((-30, -18), (30, -22), (0, -26)):
            s += flake(x, y, 5, ICE_DARK, 1.6)
        return s
    dy = {'haengt': 0, 'zittert': 0, 'faellt': 70}[state]
    s += f'<path d="M -14,{-136 + dy} L 0,{-70 + dy} L 14,{-136 + dy} Z" fill="{ICE}" {st(3)}/>'
    s += f'<path d="M -4,{-130 + dy} L 0,{-90 + dy}" stroke="#ffffff" stroke-width="3" stroke-linecap="round"/>'
    if state == 'zittert':
        s += f'<path d="M -24,-120 q -6,8 0,16 M 24,-120 q 6,8 0,16" fill="none" stroke="{OUT}" stroke-width="2.5" stroke-linecap="round"/>'
        s += f'<path d="M -8,-138 l 4,-6 M 8,-138 l -4,-6" stroke="{ROCK_DARK}" stroke-width="2.5"/>'
    if state == 'faellt':
        s += f'<path d="M -6,{-140 + dy} v -24 M 6,{-140 + dy} v -18" stroke="{DIM}" stroke-width="2.5" stroke-dasharray="4 4"/>'
    return s


def eiszapfen_figur():
    """Eiszapfen als Gegner (Spitze unten am Ursprung, 96 hoch)."""
    s = f'<path d="M -16,-96 L 0,0 L 16,-96 Z" fill="{ICE}" {st(3.5)}/>'
    s += f'<path d="M -5,-90 L 0,-40" stroke="#ffffff" stroke-width="4" stroke-linecap="round"/>'
    s += f'<path d="M 6,-92 L 4,-70" stroke="{ICE_DARK}" stroke-width="2.5" stroke-linecap="round"/>'
    return s


def duennes_eis(state='ganz'):
    """Dünnes Eis über Eiswasser (je zwei Tiles): ganz, Risse (gleich bricht es), gebrochen."""
    s = f'<rect x="-64" y="0" width="128" height="40" fill="{WATER}" {st(3)}/>'
    s += '<path d="M -50,16 q 10,-5 20,0 q 10,5 20,0 M 10,26 q 10,-5 20,0 q 10,5 20,0" fill="none" stroke="#9fd8f0" stroke-width="2.5"/>'
    if state == 'gebrochen':
        for (x, a) in ((-44, -12), (-8, 18), (30, -8)):
            s += f'<path d="M {x - 12},-2 h 24 v 8 h -24 Z" fill="{ICE}" {st(2.5)} transform="rotate({a} {x} 2)"/>'
        return s
    s += f'<rect x="-64" y="-12" width="128" height="14" fill="{ICE}" fill-opacity="0.85" {st(3)}/>'
    s += '<path d="M -56,-8 h 24 M 10,-6 h 30" stroke="#ffffff" stroke-width="2.5"/>'
    if state == 'risse':
        s += f'<path d="M -20,-12 l 8,6 l -6,4 l 10,4 M 18,-12 l -4,7 l 8,5" fill="none" stroke="{DEEP}" stroke-width="2.5"/>'
    return s


def schneebrocken(dust=True):
    """Lawinen-Schneebrocken: rollt hangabwärts, Schneestaub dahinter (`dust`)."""
    s = blob(0, -30, 30, 28, '#ffffff', n=30, jag=0.08, w=4)
    s += f'<path d="M -14,-44 q 8,-6 18,0 M -6,-22 q 10,6 20,-2" fill="none" stroke="{SNOW_SHADE}" stroke-width="3"/>'
    if not dust:
        return s
    for (x, y, r) in ((-48, -20, 10), (-66, -12, 8), (-82, -6, 6)):
        s += f'<circle cx="{x}" cy="{y}" r="{r}" fill="#ffffff" stroke="{SNOW_SHADE}" stroke-width="2"/>'
    s += f'<path d="M 10,-66 a 26,26 0 0 1 22,20" fill="none" stroke="{DIM}" stroke-width="2.5"/>'
    return s


def lawine():
    """Lawinenhang: Schnee bricht oben los, Brocken rollen; Nische als Schutz."""
    s = f'<path d="M -150,0 L 150,0 L 150,-30 L -150,-150 Z" fill="{SNOW}" {st(4)}/>'
    s += f'<path d="M -150,-150 L -100,-130 M -96,-128 l 6,-10 l 8,12" fill="none" stroke="{DEEP}" stroke-width="3"/>'
    s += f'<g transform="translate(-40,-96) scale(0.6)">{schneebrocken()}</g>'
    s += f'<g transform="translate(40,-62) scale(0.5)">{schneebrocken()}</g>'
    s += f'<path d="M 92,0 v -40 h 40 v 40" fill="{ROCK}" {st(3)}/>'
    s += f'<path d="M 100,0 v -28 h 24 v 28 Z" fill="#3f4a56"/>'
    return s


def feuerstelle(lit=True):
    """Feuerstelle (wärmt die Kälte-Leiste): Steinring, Scheite, Flammen, Wärmeschein."""
    s = ''
    if lit:
        s += f'<circle cx="0" cy="-30" r="70" fill="{FIRE}" opacity="0.14"/><circle cx="0" cy="-26" r="44" fill="{FIRE_HOT}" opacity="0.18"/>'
    s += f'<path d="M -30,-6 L 26,-18 M -26,-18 L 30,-6" stroke="{OUT}" stroke-width="12" stroke-linecap="round"/>'
    s += f'<path d="M -30,-6 L 26,-18 M -26,-18 L 30,-6" stroke="{WOOD}" stroke-width="7" stroke-linecap="round"/>'
    if lit:
        s += f'<path d="M -18,-14 Q -24,-44 -6,-62 Q -8,-44 2,-40 Q 4,-70 18,-80 Q 14,-50 22,-30 Q 22,-14 -18,-14 Z" fill="{FIRE}" {st(3)}/>'
        s += f'<path d="M -6,-16 Q -8,-34 2,-44 Q 4,-32 12,-26 Q 12,-16 -6,-16 Z" fill="{FIRE_HOT}"/>'
    for x in (-40, -22, 22, 40):
        s += f'<ellipse cx="{x}" cy="-4" rx="10" ry="7" fill="{ROCK}" {st(2.5)}/>'
    return s


def eisblock():
    """Eisblock am Bergsteig (zerbricht beim Stampfen, D-M24-01)."""
    s = f'<path d="M -50,0 L -46,-90 L 44,-96 L 50,0 Z" fill="{ICE}" {st(4.5)}/>'
    s += f'<path d="M -34,-80 L -30,-20 M -20,-84 L -18,-50" stroke="#ffffff" stroke-width="5" stroke-linecap="round" opacity="0.8"/>'
    s += f'<path d="M 10,-96 l -6,22 l 12,10 l -8,20" fill="none" stroke="{ICE_DARK}" stroke-width="3"/>'
    s += f'<path d="M -66,-40 l 14,10 M -64,-60 l 12,4 M 64,-40 l -14,10" stroke="{DIM}" stroke-width="3" stroke-linecap="round"/>'
    return s


def steigkrallen():
    """Steigkrallen (Eisgriff): Handschuh mit drei Eiskrallen."""
    s = f'<path d="M -26,0 Q -34,-40 -18,-56 L 22,-56 Q 34,-40 26,0 Z" fill="#c8483a" {st(4)}/>'
    s += f'<rect x="-28" y="-8" width="56" height="12" rx="5" fill="#ffffff" {st(3)}/>'
    for x in (-14, 0, 14):
        s += f'<path d="M {x - 5},-54 L {x},-82 L {x + 5},-54 Z" fill="{FROST}" {st(2.5)}/>'
    s += f'<path d="M 26,-30 q 18,-4 18,-18" fill="none" stroke="{OUT}" stroke-width="9" stroke-linecap="round"/>'
    s += f'<path d="M 26,-30 q 18,-4 18,-18" fill="none" stroke="#c8483a" stroke-width="5" stroke-linecap="round"/>'
    return s


def seil():
    s = ''
    for k, r in enumerate((30, 24, 18)):
        s += f'<ellipse cx="0" cy="{-30 + k * 2}" rx="{r + 6}" ry="{r}" fill="none" stroke="{OUT}" stroke-width="9"/>'
        s += f'<ellipse cx="0" cy="{-30 + k * 2}" rx="{r + 6}" ry="{r}" fill="none" stroke="#d9a066" stroke-width="5" stroke-dasharray="7 4"/>'
    return s


def eiskristall_item():
    s = crystal(-12, 0, 44, ICE_MID) + crystal(12, 0, 56, ICE) + crystal(0, 4, 34, FROST)
    return s


def kaelteleiste():
    """Kälte-Leiste im HUD (Gegenstück zur Hitze-Leiste) mit Frostrand am Bildrand."""
    s = ('<defs><clipPath id="hud-clip"><rect x="0" y="0" width="300" height="150" rx="12"/></clipPath></defs>'
         '<rect x="0" y="0" width="300" height="150" rx="12" fill="#1c2833"/><g clip-path="url(#hud-clip)">')
    # Frostrand (Post-Shader): Eisblumen an den Ecken
    for (cx, cy, sx, sy) in ((0, 0, 1, 1), (300, 0, -1, 1), (0, 150, 1, -1), (300, 150, -1, -1)):
        s += f'<circle cx="{cx}" cy="{cy}" r="46" fill="{FROST}" opacity="0.35"/>'
        s += flake(cx + sx * 26, cy + sy * 24, 10, '#ffffff', 2)
    s += '</g>'
    s += '<rect x="70" y="58" width="160" height="22" rx="11" fill="#0f1820" stroke="#ffffff" stroke-width="2"/>'
    s += f'<rect x="72" y="60" width="118" height="18" rx="9" fill="{ICE_MID}"/>'
    s += flake(52, 69, 13, ICE, 2.6)
    s += text(150, 112, 'Kälte 74 %', 15, '#ffffff', 'middle', 'bold')
    s += text(150, 132, 'am Feuer wärmen', 12, '#c8d8e4', 'middle')
    return s


# ── Quelle und Deko ─────────────────────────────────────────────────────

def frostquelle(free=False):
    """Frostquelle: vereist (grau-blass, zugefroren) / befreit (eisblau, schimmernd, Kristalle)."""
    if not free:
        s = f'<ellipse cx="0" cy="-10" rx="110" ry="22" fill="#b8c4cc" {st(4)}/>'
        s += f'<path d="M -70,-14 l 20,-6 l 10,8 l 24,-4 M 20,-16 l 18,6 l 22,-8" fill="none" stroke="#8a969e" stroke-width="3"/>'
        s += crystal(-80, -16, 40, '#c8d0d6') + crystal(86, -14, 30, '#c8d0d6')
        return s
    s = f'<ellipse cx="0" cy="-10" rx="140" ry="26" fill="{ICE_MID}" {st(4)}/>'
    s += '<ellipse cx="0" cy="-12" rx="90" ry="12" fill="#e8f8ff" opacity="0.8"/>'
    s += f'<path d="M -6,-20 Q -22,-80 0,-124 Q 22,-80 6,-20 Z" fill="{FROST}" {st(3)}/>'
    for (x, h) in ((-110, 70), (-80, 46), (90, 60), (120, 40)):
        s += crystal(x, -14, h, ICE)
    for (x, y) in ((-40, -90), (46, -110), (0, -140), (70, -70)):
        s += flake(x, y, 9, '#ffffff', 2.2)
    return s


def gipfel():
    s = f'<path d="M -220,0 L -90,-220 L -40,-150 L 20,-260 L 220,0 Z" fill="{ROCK}" {st(5)}/>'
    s += f'<path d="M -90,-220 L -120,-170 L -96,-176 L -80,-158 L -62,-182 Z" fill="#ffffff" {st(3)}/>'
    s += f'<path d="M 20,-260 L -16,-196 L 6,-204 L 24,-180 L 42,-208 L 60,-196 Z" fill="#ffffff" {st(3)}/>'
    s += f'<path d="M 40,-150 L 100,-60 M -60,-100 L -30,-40" stroke="{ROCK_DARK}" stroke-width="4"/>'
    return s


def tanne_schnee():
    s = f'<rect x="-8" y="-30" width="16" height="30" fill="{WOOD}" {st(3.5)}/>'
    for k, (w, y) in enumerate(((70, -30), (56, -80), (40, -126))):
        s += f'<path d="M {-w},{y} L 0,{y - 70} L {w},{y} Z" fill="{PINE}" {st(4)}/>'
        s += f'<path d="M {-w + 6},{y - 4} Q {-w / 2},{y - 18} 0,{y - 10} Q {w / 2},{y - 18} {w - 6},{y - 4} Q 0,{y - 2} {-w + 6},{y - 4} Z" fill="#ffffff" {st(2.5)}/>'
    s += f'<path d="M -10,-188 Q 0,-206 10,-188 Z" fill="#ffffff" {st(2.5)}/>'
    return s


def berghuette():
    s = f'<rect x="-90" y="-100" width="180" height="100" fill="{WOOD}" {st(5)}/>'
    for y in (-80, -60, -40, -20):
        s += f'<path d="M -90,{y} H 90" stroke="{WOOD_DARK}" stroke-width="3"/>'
    s += f'<path d="M -114,-96 L 0,-176 L 114,-96 Z" fill="#a8483a" {st(5)}/>'
    s += f'<path d="M -118,-92 Q -60,-120 0,-180 Q 60,-120 118,-92 Q 110,-110 0,-168 Q -110,-110 -118,-92 Z" fill="#ffffff" {st(3)}/>'
    s += f'<rect x="-24" y="-60" width="40" height="60" rx="4" fill="{WOOD_DARK}" {st(3.5)}/>'
    s += f'<rect x="34" y="-76" width="34" height="30" fill="{FIRE_HOT}" {st(3)}/><path d="M 51,-76 V -46 M 34,-61 H 68" stroke="{OUT}" stroke-width="2.5"/>'
    s += f'<rect x="50" y="-176" width="20" height="40" fill="{ROCK}" {st(3)}/>'
    s += f'<path d="M 60,-182 q -10,-14 4,-24 q 12,-10 0,-24" fill="none" stroke="#c8d0d8" stroke-width="5" stroke-linecap="round"/>'
    return s


def seilbruecke():
    s = f'<path d="M -150,-60 V 0 M 150,-60 V 0" stroke="{OUT}" stroke-width="10"/><path d="M -150,-60 V 0 M 150,-60 V 0" stroke="{WOOD}" stroke-width="6"/>'
    s += f'<path d="M -150,-56 Q 0,-10 150,-56" fill="none" stroke="#d9a066" stroke-width="4"/>'
    s += f'<path d="M -150,-14 Q 0,30 150,-14" fill="none" stroke="#d9a066" stroke-width="4"/>'
    for k in range(-6, 7):
        x = k * 22
        y = -14 + 44 * (1 - (x / 150) ** 2) * 0.5
        s += f'<rect x="{x - 9}" y="{y - 4:.1f}" width="18" height="8" fill="{WOOD}" {st(2)}/>'
        s += f'<path d="M {x},{y - 4:.1f} V {-56 + 46 * (1 - (x / 150) ** 2) * 0.5:.1f}" stroke="#d9a066" stroke-width="2"/>'
    return s


def schneewehe():
    s = f'<path d="M -120,0 Q -100,-50 -30,-56 Q 40,-60 90,-30 Q 120,-14 120,0 Z" fill="#ffffff" {st(4)}/>'
    s += f'<path d="M -70,-40 Q -20,-50 30,-44" fill="none" stroke="{SNOW_SHADE}" stroke-width="4"/>'
    return s


def gletscher():
    s = f'<path d="M -160,0 L -140,-80 L -60,-110 L 40,-96 L 140,-120 L 160,0 Z" fill="{ICE}" {st(5)}/>'
    s += f'<path d="M -40,0 L -30,-60 L -20,0 Z M 60,0 L 72,-80 L 84,0 Z" fill="{ICE_DARK}" {st(3)}/>'
    s += f'<path d="M -130,-60 L -80,-90 M 100,-100 L 130,-80" stroke="#ffffff" stroke-width="5" stroke-linecap="round"/>'
    return s


def grauspur_eis():
    """Spur des Dürren: eine Stelle, an der das Eis seine Farbe verloren hat (grau, stumpf)."""
    s = f'<ellipse cx="0" cy="-6" rx="80" ry="12" fill="#a8a8a6" opacity="0.8"/>'
    s += crystal(-40, -8, 40, '#b8b8b6') + crystal(30, -8, 52, '#c4c4c2')
    for k in range(3):
        x = 80 + k * 40
        s += f'<ellipse cx="{x}" cy="-3" rx="9" ry="3.5" fill="#9a9894" opacity="0.85"/>'
    return s


DECOR = {
    'gipfel': (gipfel(), '-226 -266 452 270'),
    'tanne-schnee': (tanne_schnee(), '-76 -210 152 214'),
    'berghuette': (berghuette(), '-122 -210 244 214'),
    'seilbruecke': (seilbruecke(), '-158 -66 316 82'),
    'schneewehe': (schneewehe(), '-126 -66 252 70'),
    'gletscher': (gletscher(), '-166 -126 332 130'),
    'feuerstelle': (feuerstelle(True), '-76 -104 152 108'),
    'eisblock': (eisblock(), '-70 -102 140 106'),
    'frostquelle-vereist': (frostquelle(False), '-120 -62 240 66'),
    'frostquelle-befreit': (frostquelle(True), '-150 -156 300 160'),
    'grauspur-eis': (grauspur_eis(), '-86 -64 262 68'),
}


def export_decor():
    for name, (art, vb) in DECOR.items():
        with open(f'assets/map/decor/{name}.svg', 'w') as f:
            f.write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}">\n'
                    f'  <!-- Kapitel 4: {name} (R2-M2.4, aus tools/design/kapitel4_entwuerfe.py). Ursprung unten in der Mitte. -->\n'
                    f'  {art}\n</svg>\n')


# ── Bogen ───────────────────────────────────────────────────────────────

def sheet():
    W, H = 1600, 2360
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
         '<defs><linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#9cc4e4"/>'
         '<stop offset="1" stop-color="#eef5fb"/></linearGradient></defs>',
         f'<rect width="{W}" height="{H}" fill="url(#sky)"/>',
         text(40, 50, 'Kapitel 4 „Frostspitzen“ – Entwürfe (R2-M2.4, M2.4.0)', 28, TEXT, 'start', 'bold'),
         text(40, 78, 'Figuren, Gegner, Hüterin, Gelände, Kälte und Deko der Berge im Stil der Kapitel 1 bis 3. '
              'Elora klein daneben als Maßstab.', 15, TEXT, 'start')]

    # Figuren und Gegenstände
    o.append(card(30, 100, 1000, 400, 'Figuren'))
    o.append(ground(60, 410, 940))
    o.append(g(160, 410, 1.0, flocke()) + elora(270, 410))
    o.append(text(170, 446, 'Bergführerin Flocke', 17, TEXT, weight='bold'))
    o.append(text(170, 468, 'Pudelmütze, Schal, Seil, Eispickel', 12, DIM, style='italic'))
    for x, (b, f, h) in zip((470, 660, 850), (('f2a65a', 'c97f3a', '#f2c14e'),
                                               ('8fd06a', '5fa03a', '#e8685a'),
                                               ('c8a0e8', '9a70c0', '#5fc8e8'))):
        o.append(g(x, 410, 0.85, kletterer(b, f, h)))
    o.append(text(660, 446, 'Drei verlorene Kletterer', 17, TEXT, weight='bold'))
    o.append(text(660, 468, 'Helm mit Lampe, Rucksack, winken um Hilfe', 12, DIM, style='italic'))

    o.append(card(1050, 100, 520, 400, 'Gegenstände'))
    o.append(ground(1080, 410, 460))
    o.append(g(1130, 410, 1.2, steigkrallen()))
    o.append(text(1130, 446, 'Steigkrallen', 15, TEXT, weight='bold'))
    o.append(text(1130, 466, '= Eisgriff', 12, DIM))
    o.append(g(1250, 410, 1.1, seil()))
    o.append(text(1250, 446, 'Flockes Seil', 15, TEXT, weight='bold'))
    o.append(g(1360, 410, 1.3, eiskristall_item()))
    o.append(text(1360, 446, 'Eiskristall', 15, TEXT, weight='bold'))
    o.append(g(1480, 410, 0.9, eisblock()))
    o.append(text(1480, 446, 'Eisblock am Bergsteig', 15, TEXT, weight='bold'))
    o.append(text(1480, 466, 'bricht mit Stampfen', 12, DIM))

    # Gegner
    o.append(card(30, 520, 1540, 380, 'Gegner'))
    o.append(ground(60, 820, 1480))
    o.append(g(170, 820, 1.0, robbe('rutschen')) + g(340, 820, 1.0, robbe('werfen')))
    o.append(text(250, 856, 'Schneeballrobbe: rutscht / wirft', 17, TEXT, weight='bold'))
    o.append(text(250, 878, 'auf dem Bauch heran, Schneebälle im Bogen', 12, DIM))
    o.append(f'<rect x="520" y="596" width="240" height="14" fill="{ROCK}" stroke="{OUT}" stroke-width="3"/>')
    o.append(g(580, 610, 1.0, fledermaus('haengt', bar=False)))
    o.append(g(700, 790, 1.0, fledermaus('sturz')))
    o.append(text(640, 856, 'Eisspitzen-Fledermaus: schläft / stürzt', 17, TEXT, weight='bold'))
    o.append(text(640, 878, 'hängt an der Decke, stürzt herab, flattert zurück', 12, DIM))
    o.append(g(1000, 820, 1.0, frostgeist()) + g(1200, 820, 1.0, frostgeist(True)))
    o.append(text(1100, 856, 'Frostgeist: schwebt / durch die Wand', 17, TEXT, weight='bold'))
    o.append(text(1100, 878, 'Berührung lässt Elora kurz erstarren', 12, DIM))
    o.append(g(1440, 820, 1.6, erstarrt()))
    o.append(text(1440, 856, 'Elora erstarrt', 15, TEXT, weight='bold'))
    o.append(text(1440, 878, '0,6 s im Eis', 12, DIM))

    # Hüterin
    o.append(card(30, 920, 1540, 520, 'Hüterin: Eiskönigin Kristella (E-341)'))
    o.append(ground(60, 1350, 1480))
    for x, pose, name, note in ((200, 'schwebend', 'schwebt über der Halle', 'unerreichbar'),
                                (520, 'hauch', 'Frosthauch', 'Frostwelle über den Boden'),
                                (900, 'erschoepft', 'erschöpft', 'herabgesunken: verwundbar'),
                                (1360, 'ruhig', 'nach dem Kampf', '„Er ist einsam, nicht böse.“')):
        o.append(g(x, 1350, 0.95, kristella(pose)))
        o.append(text(x, 1386, name, 17, TEXT, weight='bold'))
        o.append(text(x, 1408, note, 12, DIM))
    o.append(g(1130, 1250, 0.6, frostwelle()))
    o.append(text(1130, 1200, 'Frostwelle', 14, TEXT, weight='bold'))
    o.append(text(1130, 1290, 'Eis · frischer Frost (schadet) · Reif (Warnung)', 11, DIM))
    o.append(elora(700, 1350))

    # Gelände und Kälte
    o.append(card(30, 1460, 1540, 400, 'Gelände und Kälte (E-342, E-343)'))
    o.append(ground(60, 1770, 1480))
    for x, state in ((120, 'haengt'), (200, 'zittert'), (280, 'faellt'), (360, 'zerschellt')):
        o.append(g(x, 1770, 0.9, eiszapfen(state)))
    o.append(text(240, 1806, 'Eiszapfen: hängt · zittert · fällt · zerschellt', 15, TEXT, weight='bold'))
    for x, state in ((520, 'ganz'), (670, 'risse'), (820, 'gebrochen')):
        o.append(g(x, 1730, 0.9, duennes_eis(state)))
    o.append(text(670, 1806, 'Dünnes Eis über Eiswasser: ganz · Risse · gebrochen', 15, TEXT, weight='bold'))
    o.append(g(1030, 1770, 0.7, lawine()))
    o.append(text(1030, 1806, 'Lawine mit Schutz-Nische', 15, TEXT, weight='bold'))
    o.append(g(1190, 1770, 0.8, feuerstelle()))
    o.append(text(1190, 1806, 'Feuerstelle', 15, TEXT, weight='bold'))
    o.append(text(1190, 1826, 'wärmt auf', 12, DIM))
    o.append(g(1270, 1560, 0.9, kaelteleiste()))
    o.append(text(1405, 1806, 'Kälte-Leiste + Frostrand', 15, TEXT, weight='bold'))
    o.append(text(1405, 1826, 'voll = langsamer', 12, DIM))

    # Quelle und Deko
    o.append(card(30, 1880, 1540, 450, 'Quelle und Deko der Berge'))
    o.append(ground(60, 2260, 1480))
    o.append(g(160, 2260, 0.65, frostquelle(False)) + g(350, 2260, 0.6, frostquelle(True)))
    o.append(text(255, 2294, 'Frostquelle vereist / befreit', 15, TEXT, weight='bold'))
    o.append(g(570, 2260, 0.55, gipfel()) + g(760, 2260, 0.7, tanne_schnee()) + g(900, 2260, 0.6, berghuette()))
    o.append(g(1090, 2260, 0.5, seilbruecke()) + g(1240, 2260, 0.55, gletscher()) + g(1365, 2260, 0.45, schneewehe()))
    o.append(g(1470, 2260, 0.4, grauspur_eis()))
    for x, t in ((570, 'Gipfel'), (760, 'Tanne'), (900, 'Berghütte'), (1080, 'Seilbrücke'), (1240, 'Gletscher'),
                 (1365, 'Schneewehe'), (1490, 'graue Stelle')):
        o.append(text(x, 2294, t, 14, TEXT, weight='bold'))
    o.append(elora(985, 2260))
    o.append('</svg>')
    return '\n'.join(o)


if __name__ == '__main__':
    os.makedirs('docs/release-2/design', exist_ok=True)
    open('docs/release-2/design/kapitel4-entwuerfe.svg', 'w').write(sheet())
    if '--decor' in sys.argv:
        export_decor()

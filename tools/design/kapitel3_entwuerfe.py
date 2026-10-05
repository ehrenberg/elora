"""Entwürfe für Kapitel 3 „Glutsandwüste“ (R2-M2.3, Schritt M2.3.0, E-315 bis E-320).

Aufruf: python3 tools/design/kapitel3_entwuerfe.py && cargo xtask svg-preview \
        docs/release-2/design/kapitel3-entwuerfe.svg docs/release-2/design/kapitel3-entwuerfe.png 1600

Stil wie Kapitel 1 und 2: dunkler Umriss, flache Farben, warme Sand- und Ockertöne; Gegner
frech, nicht blutig. Die Wüste ist heiß und hell, die Ruinen alt und geheimnisvoll.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from abenteuer_figuren import DIM, OUT, TEXT, angry_eye, drop, g, star, text  # noqa: E402
from kapitel1_entwuerfe import card, elora, ground  # noqa: E402
from kapitel2_entwuerfe import blob  # noqa: E402

SAND = '#e8c88a'
SAND_DARK = '#c9a25e'
SAND_LIGHT = '#f6e2b4'
STONE = '#c8b49a'
STONE_DARK = '#9a8468'
OCHRE = '#d9822b'
TEAL = '#3fa38f'
EMBER = '#ff9a3c'
WATER = '#5fc8e8'
CACTUS = '#6f9a4a'
CACTUS_DARK = '#4f7a34'


def st(w=3.5):
    return f'stroke="{OUT}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'


# ── Figuren ─────────────────────────────────────────────────────────────

def sirup():
    """Karawanenführer Sirup: warmer Ockerton, Turban mit Edelstein, Weste, Laternenstab."""
    s = '<path d="M 56,-150 V 0" stroke="#5e4430" stroke-width="7"/>'
    s += f'<path d="M 56,-150 V 0" stroke="#a87a52" stroke-width="3.5"/>'
    s += f'<rect x="44" y="-178" width="24" height="28" rx="5" fill="#ffd27a" {st(3)}/><path d="M 44,-178 L 56,-190 L 68,-178" fill="#a87a52" {st(3)}/>'
    s += drop('d9a066', 'a8703c')
    # Weste mit Bordüre
    s += f'<path d="M -46,-12 Q -50,-34 -34,-44 L -22,-10 Z M 46,-12 Q 50,-34 34,-44 L 22,-10 Z" fill="{TEAL}" {st(3)}/>'
    s += '<path d="M -42,-16 Q -42,-32 -34,-38 M 42,-16 Q 42,-32 34,-38" fill="none" stroke="#f2c14e" stroke-width="3"/>'
    # Turban
    s += f'<path d="M -48,-112 Q -50,-156 0,-160 Q 50,-156 48,-112 Q 0,-124 -48,-112 Z" fill="#f6efdf" {st(4)}/>'
    for k in range(3):
        y = -150 + k * 12
        s += f'<path d="M {-44 + k * 4},{y + 10} Q 0,{y - 6} {44 - k * 4},{y + 10}" fill="none" stroke="#d8ccb8" stroke-width="2.5"/>'
    s += f'<circle cx="0" cy="-136" r="8" fill="#e8685a" {st(2.5)}/>'
    s += f'<path d="M 0,-144 q 8,-18 20,-20" fill="none" stroke="{TEAL}" stroke-width="5" stroke-linecap="round"/>'
    # Schnurrbart
    s += f'<path d="M 18,-44 Q 8,-40 2,-48 Q 8,-38 18,-40 Q 28,-38 34,-48 Q 28,-40 18,-44 Z" fill="#5e4430" {st(1.6)}/>'
    return s


def palma():
    """Oasen-Hüterin Palma: kühles Blau, Kopftuch, Wasserkrug auf der Schulter."""
    s = drop('7fc8e8', '4f9ac0')
    s += f'<path d="M -52,-104 Q -50,-150 0,-152 Q 50,-150 52,-104 Q 54,-80 46,-60 Q 30,-110 0,-112 Q -30,-110 -46,-60 Q -54,-80 -52,-104 Z" fill="#f2c14e" {st(4)}/>'
    for x in (-30, -10, 10, 30):
        s += f'<circle cx="{x}" cy="-132" r="3" fill="#e8685a"/>'
    s += f'<path d="M -70,-110 Q -86,-80 -70,-60 L -52,-66 Q -60,-84 -50,-104 Z" fill="#c87a4a" {st(3.5)}/>'
    s += f'<ellipse cx="-62" cy="-112" rx="14" ry="6" fill="#a8603a" {st(2.5)}/>'
    s += f'<path d="M -66,-118 q -4,-10 4,-14" fill="none" stroke="{WATER}" stroke-width="4" stroke-linecap="round"/>'
    return s


def kamel():
    """Karawanenkamel (Deko im Lager): freundlich, mit Satteltaschen."""
    s = ''
    for x in (-50, -26, 30, 54):
        s += f'<path d="M {x},-70 V 0" stroke="{OUT}" stroke-width="13" stroke-linecap="round"/><path d="M {x},-70 V 0" stroke="#d9a066" stroke-width="8" stroke-linecap="round"/>'
    s += f'<path d="M -70,-70 Q -76,-120 -40,-128 Q -20,-160 0,-128 Q 16,-150 34,-128 Q 66,-122 70,-80 Q 60,-62 0,-62 Q -60,-62 -70,-70 Z" fill="#d9a066" {st(4)}/>'
    s += f'<path d="M 64,-100 Q 90,-110 96,-150 Q 98,-176 120,-178 Q 146,-176 144,-158 Q 140,-146 124,-148 Q 112,-120 92,-90 Z" fill="#d9a066" {st(4)}/>'
    s += f'<circle cx="128" cy="-166" r="4" fill="{OUT}"/><path d="M 136,-152 q 6,2 8,-2" fill="none" stroke="{OUT}" stroke-width="2.5"/>'
    s += f'<rect x="-52" y="-112" width="34" height="40" rx="8" fill="{TEAL}" {st(3)}/><rect x="14" y="-108" width="34" height="38" rx="8" fill="#c8483a" {st(3)}/>'
    s += f'<path d="M -48,-100 H -22 M 18,-96 H 44" stroke="#f2c14e" stroke-width="3"/>'
    return s


# ── Gegner ──────────────────────────────────────────────────────────────

def sandkrabbe():
    """Sandkrabbe: gepanzert (Panzer an den Seiten, oben weich und hell), große Scheren."""
    s = ''
    for x in (-30, -14, 14, 30):
        s += f'<path d="M {x},-14 l {6 if x > 0 else -6},14" stroke="{OUT}" stroke-width="5" stroke-linecap="round"/>'
    # Scheren
    for side in (-1, 1):
        x = side * 52
        s += f'<path d="M {side * 30},-30 Q {x},-40 {x},-56" fill="none" stroke="{OUT}" stroke-width="9" stroke-linecap="round"/>'
        s += f'<path d="M {side * 30},-30 Q {x},-40 {x},-56" fill="none" stroke="{OCHRE}" stroke-width="5" stroke-linecap="round"/>'
        s += f'<path d="M {x - 14},-56 Q {x},-84 {x + 14},-58 L {x + 4},-58 Q {x},-70 {x - 6},-58 Z" fill="{OCHRE}" {st(3)}/>'
    # Panzer
    s += f'<path d="M -44,-16 Q -48,-52 0,-56 Q 48,-52 44,-16 Q 0,-8 -44,-16 Z" fill="#b8682a" {st(4.5)}/>'
    for x in (-30, -16, 16, 30):
        s += f'<path d="M {x},-20 Q {x * 1.1},-38 {x * 0.8},-50" fill="none" stroke="#8a4a1e" stroke-width="3"/>'
    s += f'<ellipse cx="0" cy="-46" rx="18" ry="8" fill="{SAND_LIGHT}" {st(2.5)}/>'
    s += f'<path d="M -8,-46 L 8,-46 M 0,-52 V -40" stroke="#c94a4a" stroke-width="2.5" stroke-linecap="round"/>'
    # Stielaugen
    for x in (-10, 10):
        s += f'<path d="M {x},-54 V -70" stroke="{OUT}" stroke-width="4"/>'
        s += angry_eye(x, -74, 6)
    return s


def duenenwurm(pose='bogen'):
    """Dünenwurm: segmentiert, Mundzangen; `spur` = Sandwelle, `bogen` = springt heraus."""
    if pose == 'spur':
        s = f'<path d="M -60,0 Q -40,-18 -20,-8 Q 0,-22 20,-8 Q 40,-18 60,0 Z" fill="{SAND}" {st(3)}/>'
        for x in (-34, 6, 40):
            s += f'<circle cx="{x}" cy="-10" r="3" fill="{SAND_DARK}"/>'
        return s
    s = f'<path d="M -70,0 Q -50,-14 -30,-6 Z" fill="{SAND}" {st(3)}/>'
    pts = [(-60 + 22 * k, -10 - 110 * math.sin(math.pi * k / 8)) for k in range(8)]
    for k, (x, y) in enumerate(pts):
        r = 20 - abs(k - 3.5) * 1.5
        s += f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r:.1f}" fill="{"#c88a5a" if k % 2 else "#b87a4a"}" {st(3.5)}/>'
    hx, hy = pts[-1]
    s += f'<path d="M {hx + 12},{hy - 8} l 16,-8 l -6,10 M {hx + 12},{hy + 8} l 16,8 l -6,-10" fill="{OCHRE}" stroke="{OUT}" stroke-width="3" stroke-linejoin="round"/>'
    s += angry_eye(hx - 4, hy - 8, 6)
    return s


def funkenmotte():
    """Funkenmotte: glühende Flügel, Funken fallen."""
    s = '<circle cx="0" cy="-40" r="46" fill="#ffb84a" opacity="0.18"/>'
    for side in (-1, 1):
        s += f'<path d="M 0,-44 Q {side * 50},-90 {side * 60},-50 Q {side * 52},-24 0,-36 Z" fill="{EMBER}" {st(3.5)}/>'
        s += f'<path d="M 0,-36 Q {side * 40},-26 {side * 36},-6 Q {side * 16},-8 0,-30 Z" fill="#ffcf6a" {st(3)}/>'
        s += f'<circle cx="{side * 36}" cy="-56" r="7" fill="#fff0b0" stroke="{OUT}" stroke-width="2"/>'
    s += f'<ellipse cx="0" cy="-38" rx="8" ry="20" fill="#5e4430" {st(3)}/>'
    s += f'<path d="M -3,-56 q -8,-14 -16,-14 M 3,-56 q 8,-14 16,-14" fill="none" stroke="{OUT}" stroke-width="2.5"/>'
    s += angry_eye(-4, -50, 3.5) + angry_eye(5, -50, 3.5)
    for (x, y) in ((-10, 4), (8, 16), (-4, 28)):
        s += star(x, y, 5, '#ffcf6a')
    return s


# ── Hüter ───────────────────────────────────────────────────────────────

def sandschlange(pose='bogen'):
    """Sandschlange: große Schlange mit Sandschuppen und Rautenmuster, gelbe Augen.
    Posen: spur (nur Sandwelle), auftauchen, bogen (in der Luft), benommen, ruhig."""
    if pose == 'spur':
        s = f'<path d="M -150,0 Q -110,-30 -70,-12 Q -30,-36 10,-12 Q 50,-36 90,-12 Q 130,-30 150,0 Z" fill="{SAND}" {st(4)}/>'
        s += '<path d="M -120,-8 q 20,-10 40,0 M 40,-8 q 20,-10 40,0" fill="none" stroke="#c9a25e" stroke-width="3"/>'
        return s
    body = {
        'auftauchen': [(0, 0), (6, -60), (-4, -120), (10, -170), (36, -200)],
        'bogen': [(-180, 0), (-140, -110), (-60, -190), (40, -200), (120, -150), (170, -60)],
        'benommen': [(-150, -14), (-90, -24), (-30, -14), (30, -24), (90, -16), (140, -24)],
        'ruhig': [(-130, -14), (-70, -30), (-10, -20), (40, -50), (60, -110), (80, -140)],
    }[pose]
    s = ''
    if pose == 'auftauchen':
        s += f'<path d="M -80,0 Q -50,-30 -20,-10 Q 0,-40 30,-10 Q 60,-30 90,0 Z" fill="{SAND}" {st(4)}/>'
        for k in range(5):
            a = math.radians(-150 + k * 30)
            s += f'<circle cx="{60 * math.cos(a):.0f}" cy="{-20 + 40 * math.sin(a):.0f}" r="6" fill="{SAND_LIGHT}" stroke="{OUT}" stroke-width="2"/>'
    path = f'M {body[0][0]},{body[0][1]}'
    for (x0, y0), (x1, y1) in zip(body, body[1:]):
        path += f' Q {x0 + (x1 - x0) * 0.5 + (y1 - y0) * 0.2:.0f},{y0 + (y1 - y0) * 0.5 - (x1 - x0) * 0.2:.0f} {x1},{y1}'
    s += f'<path d="{path}" fill="none" stroke="{OUT}" stroke-width="58" stroke-linecap="round" stroke-linejoin="round"/>'
    s += f'<path d="{path}" fill="none" stroke="{SAND_DARK}" stroke-width="48" stroke-linecap="round" stroke-linejoin="round"/>'
    s += f'<path d="{path}" fill="none" stroke="#e8b46a" stroke-width="20" stroke-dasharray="18 22" stroke-linecap="round"/>'
    hx, hy = body[-1]
    ang = math.degrees(math.atan2(body[-1][1] - body[-2][1], body[-1][0] - body[-2][0]))
    head = f'<ellipse cx="0" cy="0" rx="46" ry="32" fill="{SAND_DARK}" {st(5)}/>'
    head += f'<path d="M 10,-30 Q 30,-36 44,-20" fill="none" stroke="#a8803c" stroke-width="4"/>'
    if pose == 'benommen':
        head += f'<path d="M 12,-12 l 10,10 M 22,-12 l -10,10" stroke="{OUT}" stroke-width="4" stroke-linecap="round"/>'
    elif pose == 'ruhig':
        head += f'<path d="M 8,-10 Q 18,-18 28,-10" fill="none" stroke="{OUT}" stroke-width="4" stroke-linecap="round"/>'
    else:
        head += f'<ellipse cx="18" cy="-10" rx="10" ry="8" fill="#f2c14e" {st(3)}/><ellipse cx="20" cy="-10" rx="3" ry="7" fill="{OUT}"/>'
    head += f'<path d="M 40,8 l 18,4 l -6,4 m 6,-4 l -6,8" fill="none" stroke="#c94a4a" stroke-width="3.5" stroke-linecap="round"/>'
    s += f'<g transform="translate({hx},{hy}) rotate({ang if pose != "benommen" else 0})">{head}</g>'
    if pose == 'benommen':
        for k in range(3):
            a = math.radians(k * 120)
            s += star(hx + 30 * math.cos(a), hy - 54 + 8 * math.sin(a), 9, '#f2c14e')
    return s


# ── Gegenstände, Quelle, Deko ───────────────────────────────────────────

def wasserschlauch(full=True):
    s = f'<path d="M -22,-8 Q -30,-40 -10,-54 L 10,-54 Q 30,-40 22,-8 Q 0,4 -22,-8 Z" fill="{"#a8703c" if full else "#c8a07a"}" {st(3.5)}/>'
    s += f'<rect x="-6" y="-66" width="12" height="14" rx="3" fill="#5e4430" {st(2.5)}/>'
    if full:
        s += f'<path d="M -12,-30 Q 0,-24 12,-30" fill="none" stroke="{WATER}" stroke-width="5" stroke-linecap="round"/>'
    return s


def steintafel():
    """Ruinentafel mit eingeritztem grauen Abdruck (Spur des Wesens, das Farbe trinkt)."""
    s = f'<path d="M -40,0 L -36,-90 Q 0,-104 36,-90 L 40,0 Z" fill="{STONE}" {st(4)}/>'
    s += f'<path d="M -28,-74 H 28 M -28,-60 H 16 M -28,-46 H 24" stroke="{STONE_DARK}" stroke-width="3"/>'
    s += '<path d="M -10,-34 q -6,-10 0,-16 q 4,8 10,0 q 6,6 0,16 Z" fill="#8a8a88" stroke="#5a5a58" stroke-width="2"/>'
    s += f'<path d="M -36,-10 l 10,-6 l 8,8" fill="none" stroke="{STONE_DARK}" stroke-width="2.5"/>'
    return s


def glutquelle(freed=True):
    """Glutquelle im Sandkessel: Steinbecken, warmes Leuchten; verdorrt = grau und rissig."""
    s = ''
    if freed:
        s += '<ellipse cx="0" cy="-50" rx="150" ry="80" fill="#ffcf6a" opacity="0.25"/>'
    s += f'<path d="M -110,0 Q -116,-34 -94,-40 H 94 Q 116,-34 110,0 Z" fill="{STONE}" {st(5)}/>'
    for x in range(-90, 100, 30):
        s += f'<path d="M {x},-40 V 0" stroke="{STONE_DARK}" stroke-width="2"/>'
    s += f'<ellipse cx="0" cy="-40" rx="90" ry="14" fill="{"#ffb84a" if freed else "#9a8a70"}" {st(4)}/>'
    if freed:
        for (x, y) in ((-40, -70), (10, -100), (50, -64), (-10, -120)):
            s += star(x, y, 9, '#fff0b0')
        s += '<path d="M -30,-42 q 10,-5 20,0 M 20,-38 q 10,-5 20,0" fill="none" stroke="#fff6d8" stroke-width="3" stroke-linecap="round"/>'
    else:
        s += '<path d="M -40,-40 l 12,6 l 10,-6 l 14,5 M 20,-42 l 8,6 l 12,-4" fill="none" stroke="#6a5a40" stroke-width="2.5"/>'
    return s


def duene():
    return (f'<path d="M -200,0 Q -120,-90 -20,-70 Q 60,-120 200,0 Z" fill="{SAND}" {st(4)}/>'
            f'<path d="M -120,-40 Q -60,-70 0,-50 M 40,-60 Q 100,-80 150,-30" fill="none" stroke="{SAND_LIGHT}" stroke-width="5" stroke-linecap="round"/>')


def felsbogen():
    s = f'<path d="M -130,0 Q -140,-160 0,-180 Q 140,-160 130,0 L 80,0 Q 90,-110 0,-120 Q -90,-110 -80,0 Z" fill="#d9a066" {st(5)}/>'
    s += f'<path d="M -110,-60 Q -100,-120 -40,-150 M 60,-140 Q 100,-120 110,-60" fill="none" stroke="#b8804a" stroke-width="4"/>'
    s += f'<path d="M -126,-20 H -86 M 90,-30 H 124" stroke="#b8804a" stroke-width="3"/>'
    return s


def saeule(broken=False):
    h = 140 if broken else 200
    s = f'<rect x="-30" y="-12" width="60" height="12" fill="{STONE}" {st(3.5)}/>'
    s += f'<rect x="-22" y="{-h}" width="44" height="{h - 12}" fill="{STONE}" {st(4)}/>'
    for x in (-10, 0, 10):
        s += f'<path d="M {x},{-h + 6} V -16" stroke="{STONE_DARK}" stroke-width="2"/>'
    if broken:
        s += f'<path d="M -22,{-h} l 10,-14 l 8,10 l 12,-16 l 14,20" fill="{STONE}" {st(3.5)}/>'
    else:
        s += f'<rect x="-32" y="{-h - 16}" width="64" height="16" fill="{STONE}" {st(3.5)}/>'
    return s


def ruinentor():
    s = f'<g transform="translate(-80,0)">{saeule()}</g><g transform="translate(80,0)">{saeule()}</g>'
    s += f'<path d="M -118,-216 H 118 V -244 H -118 Z" fill="{STONE}" {st(4)}/>'
    s += f'<path d="M -40,-230 h 20 m 10,0 h 20 m 10,0 h 20" stroke="{STONE_DARK}" stroke-width="4"/>'
    s += f'<circle cx="0" cy="-230" r="8" fill="#8a8a88" stroke="#5a5a58" stroke-width="2"/>'
    return s


def kaktus():
    s = f'<path d="M -14,0 V -110 Q -14,-130 0,-130 Q 14,-130 14,-110 V 0 Z" fill="{CACTUS}" {st(4)}/>'
    s += f'<path d="M -14,-60 H -34 Q -44,-60 -44,-70 V -90 Q -44,-100 -36,-100 Q -28,-100 -28,-90 V -76 H -14" fill="{CACTUS}" {st(3.5)}/>'
    s += f'<path d="M 14,-40 H 30 Q 40,-40 40,-50 V -76 Q 40,-86 32,-86 Q 24,-86 24,-76 V -56 H 14" fill="{CACTUS}" {st(3.5)}/>'
    for y in range(-120, -10, 18):
        s += f'<path d="M -4,{y} v 8 M 4,{y + 9} v 8" stroke="{CACTUS_DARK}" stroke-width="2"/>'
    s += f'<circle cx="0" cy="-134" r="7" fill="#ef7fb0" {st(2)}/>'
    return s


def palme():
    s = f'<path d="M -10,0 Q -4,-120 20,-220 L 32,-218 Q 10,-120 10,0 Z" fill="#a87a52" {st(4)}/>'
    for y in range(-200, -10, 20):
        s += f'<path d="M -6,{y * -0.5 - 200 + 100} q 8,4 18,0" fill="none" stroke="#7a5434" stroke-width="2"/>'
    for a in (-150, -110, -60, -20, 20):
        r = math.radians(a)
        x, y = 26 + 110 * math.cos(r), -220 + 60 * math.sin(r) + 40
        s += f'<path d="M 26,-222 Q {26 + 60 * math.cos(r):.0f},{-260 + 30 * math.sin(r):.0f} {x:.0f},{y:.0f}" fill="none" stroke="{OUT}" stroke-width="20" stroke-linecap="round"/>'
        s += f'<path d="M 26,-222 Q {26 + 60 * math.cos(r):.0f},{-260 + 30 * math.sin(r):.0f} {x:.0f},{y:.0f}" fill="none" stroke="#5b9d42" stroke-width="13" stroke-linecap="round"/>'
    for (x, y) in ((16, -214), (30, -210)):
        s += f'<circle cx="{x}" cy="{y}" r="9" fill="#8a6040" {st(2.5)}/>'
    return s


def zelt():
    s = f'<path d="M -110,0 L 0,-120 L 110,0 Z" fill="#e8d0a0" {st(5)}/>'
    s += f'<path d="M -110,0 L 0,-120 L -40,0 Z" fill="#c8483a" {st(4)}/>'
    s += f'<path d="M -20,0 L 0,-80 L 20,0 Z" fill="#5e4430" {st(3)}/>'
    s += f'<path d="M 0,-120 V -150" stroke="{OUT}" stroke-width="4"/><path d="M 0,-150 l 24,6 l -24,6 Z" fill="{TEAL}" {st(2.5)}/>'
    s += f'<path d="M -70,-44 L 70,-44" stroke="#c8b08a" stroke-width="3"/>'
    return s


def oase():
    s = f'<ellipse cx="0" cy="-6" rx="150" ry="20" fill="{WATER}" {st(4)}/>'
    s += '<path d="M -80,-10 q 14,-6 28,0 M 30,-8 q 14,-6 28,0" fill="none" stroke="#ffffff" stroke-width="3" stroke-linecap="round"/>'
    for x in (-150, -130, 132, 150):
        s += f'<path d="M {x},-6 q -6,-30 4,-60 M {x + 8},-6 q 2,-26 10,-46" fill="none" stroke="#4f8f3a" stroke-width="5" stroke-linecap="round"/>'
    return s


def giessstelle(bloom=False):
    """Verdorrte Stelle der Oase / nach dem Gießen aufgeblüht."""
    if not bloom:
        s = f'<ellipse cx="0" cy="-4" rx="40" ry="8" fill="#b89a6a" {st(3)}/>'
        s += f'<path d="M -20,-6 q 4,-20 -4,-28 M 10,-6 q -2,-18 8,-24" fill="none" stroke="#9a8a60" stroke-width="4" stroke-linecap="round"/>'
        return s
    s = f'<ellipse cx="0" cy="-4" rx="40" ry="8" fill="#6f9a4a" {st(3)}/>'
    for (x, h, c) in ((-24, 30, '#ef7fb0'), (-6, 42, '#f2c14e'), (12, 34, '#a77be0'), (28, 26, '#ef7fb0')):
        s += f'<path d="M {x},-6 V {-h}" stroke="{OUT}" stroke-width="6"/><path d="M {x},-6 V {-h}" stroke="#4f9a3a" stroke-width="3"/>'
        s += f'<circle cx="{x}" cy="{-h - 6}" r="8" fill="{c}" {st(2.5)}/>'
    return s


# ── Bogen ───────────────────────────────────────────────────────────────

def sheet():
    W, H = 1600, 1880
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
         '<defs><linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#8fbcdf"/>'
         '<stop offset="1" stop-color="#f6e2bf"/></linearGradient></defs>',
         f'<rect width="{W}" height="{H}" fill="url(#sky)"/>',
         text(40, 50, 'Kapitel 3 „Glutsandwüste“ – Entwürfe (R2-M2.3, M2.3.0)', 28, TEXT, 'start', 'bold'),
         text(40, 78, 'Figuren, Gegner, Hüter, Gegenstände und Deko der Wüste im Stil der Kapitel 1 und 2. '
              'Elora klein daneben als Maßstab.', 15, TEXT, 'start')]

    o.append(card(30, 100, 1000, 400, 'Figuren und Karawane'))
    o.append(ground(60, 410, 940))
    o.append(g(160, 410, 1.0, sirup()) + elora(270, 410))
    o.append(text(170, 446, 'Karawanenführer Sirup', 17, TEXT, weight='bold'))
    o.append(text(170, 468, 'Turban, Weste, Laternenstab; seltene Waren', 12, DIM, style='italic'))
    o.append(g(420, 410, 1.0, palma()))
    o.append(text(420, 446, 'Oasen-Hüterin Palma', 17, TEXT, weight='bold'))
    o.append(text(420, 468, 'Kopftuch, Wasserkrug', 12, DIM, style='italic'))
    o.append(g(640, 410, 0.9, kamel()) + g(880, 410, 0.9, zelt()))
    o.append(text(690, 446, 'Kamel mit Satteltaschen', 15, TEXT, weight='bold'))
    o.append(text(880, 446, 'Zelt der Karawane', 15, TEXT, weight='bold'))

    o.append(card(1050, 100, 520, 400, 'Gegenstände'))
    o.append(ground(1080, 410, 460))
    o.append(g(1130, 410, 1.2, wasserschlauch(False)) + g(1210, 410, 1.2, wasserschlauch(True)))
    o.append(text(1170, 446, 'Wasserschlauch leer / voll', 15, TEXT, weight='bold'))
    o.append(g(1320, 410, 1.0, steintafel()))
    o.append(text(1320, 446, 'Ruinentafel', 15, TEXT, weight='bold'))
    o.append(text(1320, 466, 'grauer Abdruck', 12, DIM))
    o.append(g(1420, 410, 0.8, giessstelle(False)) + g(1500, 410, 0.8, giessstelle(True)))
    o.append(text(1460, 446, 'Gießstelle', 15, TEXT, weight='bold'))

    o.append(card(30, 520, 1540, 360, 'Gegner'))
    o.append(ground(60, 800, 1480))
    o.append(g(170, 800, 1.2, sandkrabbe()))
    o.append(text(170, 836, 'Sandkrabbe', 17, TEXT, weight='bold'))
    o.append(text(170, 858, 'Panzer an den Seiten, nur von oben verwundbar', 12, DIM))
    o.append(g(470, 800, 1.0, duenenwurm('spur')) + g(660, 800, 1.0, duenenwurm('bogen')))
    o.append(text(570, 836, 'Dünenwurm: Sandspur / Sprung', 17, TEXT, weight='bold'))
    o.append(text(570, 858, 'wandert unter dem Sand, springt im Bogen', 12, DIM))
    o.append(g(980, 740, 1.2, funkenmotte()))
    o.append(text(980, 836, 'Funkenmotte', 17, TEXT, weight='bold'))
    o.append(text(980, 858, 'lässt glühende Funken fallen', 12, DIM))
    o.append(elora(1200, 800) + text(1200, 836, 'Maßstab', 13, DIM))

    o.append(card(30, 900, 1540, 480, 'Hüter: Sandschlange (E-316)'))
    o.append(ground(60, 1290, 1480))
    for x, pose, name, note in ((180, 'spur', 'unter dem Sand', 'nur die Spur ist zu sehen'),
                                (470, 'auftauchen', 'Auftauchen', 'Sand bebt vorher'),
                                (800, 'bogen', 'Bogen durch die Luft', 'jetzt verwundbar'),
                                (1150, 'benommen', 'benommen', 'Hammer und Granaten'),
                                (1420, 'ruhig', 'nach dem Kampf', 'müde, nicht böse')):
        sc = 0.7 if pose in ('bogen',) else 0.75
        o.append(g(x, 1290, sc, sandschlange(pose)))
        o.append(text(x, 1324, name, 17, TEXT, weight='bold'))
        o.append(text(x, 1346, note, 12, DIM))

    o.append(card(30, 1400, 1540, 450, 'Quelle und Deko der Wüste'))
    o.append(ground(60, 1780, 1480))
    o.append(g(150, 1780, 0.6, glutquelle(False)) + g(310, 1780, 0.6, glutquelle(True)))
    o.append(text(230, 1814, 'Glutquelle verdorrt / befreit', 15, TEXT, weight='bold'))
    o.append(g(480, 1780, 0.5, oase()) + g(700, 1780, 0.5, duene()) + g(900, 1780, 0.55, felsbogen()))
    o.append(g(1030, 1780, 0.7, saeule()) + g(1090, 1780, 0.7, saeule(True)))
    o.append(g(1220, 1780, 0.55, ruinentor()) + g(1350, 1780, 0.8, kaktus()) + g(1440, 1780, 0.7, palme()))
    for x, t in ((480, 'Oase'), (700, 'Düne'), (900, 'Felsbogen'), (1060, 'Säulen'), (1220, 'Ruinentor'), (1350, 'Kaktus'), (1450, 'Palme')):
        o.append(text(x, 1814, t, 14, TEXT, weight='bold'))
    o.append(elora(1540, 1780))
    o.append('</svg>')
    return '\n'.join(o)


if __name__ == '__main__':
    os.makedirs('docs/release-2/design', exist_ok=True)
    open('docs/release-2/design/kapitel3-entwuerfe.svg', 'w').write(sheet())

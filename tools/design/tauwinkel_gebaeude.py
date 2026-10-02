"""Entwürfe der Gebäude von Tauwinkel (A1.9, E-274, E-276, E-277) als Karten-Deko im Stil A.

Aufruf: python3 tools/design/tauwinkel_gebaeude.py && cargo xtask svg-preview \
        docs/release-2/design/tauwinkel-gebaeude.svg docs/release-2/design/tauwinkel-gebaeude.png 1600

Zweite Fassung: richtige Dächer (Ziegel, Schiefer, Reet, Schindeln), Fachwerk, Stein und Holz;
zurückhaltende, warme Farben. Jede Figur hat ihre Farbe nur in Details (Fensterläden, Tür,
Schild). Verblassen (E-277): Blumen, Beete und Fahnen gibt es grau und bunt.

Welteinheiten (1 Tile = 32, Elora ≈ 47 hoch), Ursprung unten in der Mitte.
"""
import os
import re

OUT = '#2b2b2b'
SW = 2.0
PLASTER = '#efe3cc'
PLASTER_SHADE = '#dccbae'
BEAM = '#6e4c32'
STONE = '#aca69c'
STONE_LINE = '#857f76'
WOOD = '#a87a52'
WOOD_DARK = '#7a5434'
GLASS = '#a9cfe0'
TILE = '#b5523b'
TILE_LINE = '#8e3d2b'
SLATE = '#5b6672'
SLATE_LINE = '#434c56'
THATCH = '#c9a45c'
THATCH_LINE = '#a3813f'
SHINGLE = '#8a6040'
SHINGLE_LINE = '#6a4830'
TEXT = '#3b3024'


def st(w=SW):
    return f'stroke="{OUT}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'


def gable(cx, base, w, h, color, line, rows=5, overhang=14):
    """Satteldach (Giebel von vorn) mit Reihen aus Ziegeln/Schindeln."""
    l, r, ty = cx - w / 2 - overhang, cx + w / 2 + overhang, base - h
    s = f'<path d="M {l},{base} L {cx},{ty} L {r},{base} Z" fill="{color}" {st()}/>'
    for i in range(1, rows):
        t = i / rows
        y = ty + h * t
        hx = (w / 2 + overhang) * t
        s += f'<path d="M {cx - hx + 3},{y} H {cx + hx - 3}" stroke="{line}" stroke-width="1.6"/>'
    s += f'<path d="M {l - 2},{base} H {r + 2}" stroke="{OUT}" stroke-width="5" stroke-linecap="round"/>'
    return s


def side_roof(x0, x1, base, h, color, line, rows=4):
    """Dach von der Traufseite: Trapez mit Reihen."""
    s = f'<path d="M {x0 - 12},{base} L {x0 + 18},{base - h} H {x1 - 18} L {x1 + 12},{base} Z" fill="{color}" {st()}/>'
    for i in range(1, rows):
        y = base - h * i / rows
        k = 30 * i / rows
        s += f'<path d="M {x0 - 12 + k},{y} H {x1 + 12 - k}" stroke="{line}" stroke-width="1.6"/>'
    return s


def foundation(x0, x1, base, h=18):
    s = f'<rect x="{x0}" y="{base - h}" width="{x1 - x0}" height="{h}" fill="{STONE}" {st()}/>'
    x, row = x0, 0
    while x < x1:
        s += f'<path d="M {x},{base - h} V {base}" stroke="{STONE_LINE}" stroke-width="1.4"/>'
        x += 22 if row % 2 == 0 else 17
        row += 1
    s += f'<path d="M {x0},{base - h / 2} H {x1}" stroke="{STONE_LINE}" stroke-width="1.4"/>'
    return s


def timber(x0, x1, top, bottom, posts=3, braces=True):
    """Putzwand mit Fachwerk."""
    s = f'<rect x="{x0}" y="{top}" width="{x1 - x0}" height="{bottom - top}" fill="{PLASTER}" {st()}/>'
    s += f'<rect x="{x0 + 2}" y="{bottom - 8}" width="{x1 - x0 - 4}" height="6" fill="{PLASTER_SHADE}"/>'
    for y in (top, bottom):
        s += f'<path d="M {x0},{y} H {x1}" stroke="{BEAM}" stroke-width="5"/>'
    xs = [x0 + (x1 - x0) * i / posts for i in range(posts + 1)]
    for x in xs:
        s += f'<path d="M {x},{top} V {bottom}" stroke="{BEAM}" stroke-width="5"/>'
    if braces:
        s += f'<path d="M {xs[0]},{bottom} L {xs[1]},{top + (bottom - top) * 0.45}" stroke="{BEAM}" stroke-width="4"/>'
        s += f'<path d="M {xs[-1]},{bottom} L {xs[-2]},{top + (bottom - top) * 0.45}" stroke="{BEAM}" stroke-width="4"/>'
    s += f'<rect x="{x0}" y="{top}" width="{x1 - x0}" height="{bottom - top}" fill="none" {st()}/>'
    return s


def stone_wall(x0, x1, top, bottom, color=STONE):
    s = f'<rect x="{x0}" y="{top}" width="{x1 - x0}" height="{bottom - top}" fill="{color}" {st()}/>'
    y, row = top + 14, 0
    while y < bottom:
        s += f'<path d="M {x0},{y} H {x1}" stroke="{STONE_LINE}" stroke-width="1.4"/>'
        off = 0 if row % 2 == 0 else 13
        for x in range(int(x0 + off + 26), int(x1), 26):
            s += f'<path d="M {x},{y - 14} V {y}" stroke="{STONE_LINE}" stroke-width="1.4"/>'
        y += 14
        row += 1
    return s


def window(cx, cy, w=26, h=30, shutter=None):
    s = ''
    if shutter:
        s += f'<rect x="{cx - w / 2 - 13}" y="{cy - h / 2}" width="11" height="{h}" rx="1.5" fill="{shutter}" {st(1.6)}/>'
        s += f'<rect x="{cx + w / 2 + 2}" y="{cy - h / 2}" width="11" height="{h}" rx="1.5" fill="{shutter}" {st(1.6)}/>'
    s += f'<rect x="{cx - w / 2}" y="{cy - h / 2}" width="{w}" height="{h}" fill="{GLASS}" {st()}/>'
    s += f'<path d="M {cx - w / 2},{cy} H {cx + w / 2} M {cx},{cy - h / 2} V {cy + h / 2}" stroke="{OUT}" stroke-width="1.8"/>'
    s += f'<rect x="{cx - w / 2 - 3}" y="{cy + h / 2}" width="{w + 6}" height="4" fill="{WOOD}" {st(1.4)}/>'
    return s


def door(cx, base, w=30, h=58, color=WOOD):
    s = f'<path d="M {cx - w / 2 - 4},{base} V {base - h + 4} Q {cx},{base - h - 8} {cx + w / 2 + 4},{base - h + 4} V {base}" fill="{BEAM}" {st()}/>'
    s += f'<path d="M {cx - w / 2},{base} V {base - h + 6} Q {cx},{base - h - 3} {cx + w / 2},{base - h + 6} V {base} Z" fill="{color}" {st(1.6)}/>'
    for k in (-1, 1):
        s += f'<path d="M {cx + k * w / 6},{base - h + 4} V {base}" stroke="{WOOD_DARK}" stroke-width="1.6"/>'
    s += f'<circle cx="{cx + w / 2 - 6}" cy="{base - h * 0.45}" r="2" fill="{OUT}"/>'
    return s


def chimney(x, top, w=16, h=46):
    s = f'<rect x="{x}" y="{top}" width="{w}" height="{h}" fill="#9c5a44" {st()}/>'
    s += f'<rect x="{x - 3}" y="{top - 4}" width="{w + 6}" height="6" fill="#7a4434" {st(1.6)}/>'
    s += f'<path d="M {x},{top + 14} H {x + w} M {x},{top + 28} H {x + w}" stroke="#7a4434" stroke-width="1.4"/>'
    return s


def hanging_sign(x, y, icon, w=34):
    """Wandarm mit hängendem Schild."""
    s = f'<path d="M {x - 22},{y - 18} H {x + 4}" stroke="{OUT}" stroke-width="3"/>'
    s += f'<path d="M {x - 8},{y - 18} V {y - 10} M {x + 2},{y - 18} V {y - 10}" stroke="{OUT}" stroke-width="1.4"/>'
    s += f'<rect x="{x - 3 - w / 2}" y="{y - 10}" width="{w}" height="26" rx="3" fill="{WOOD}" {st()}/>'
    return s + icon(x - 3, y + 3)


def flowers(xs, base, pale):
    if pale is None:
        return ''
    colors = ('#b9b4ad', '#a9a49d', '#c4bfb8') if pale else ('#e8708f', '#f2c14e', '#7aa8d8')
    stem = '#8f9488' if pale else '#4f9a3a'
    s = ''
    for i, x in enumerate(xs):
        h = 8 if pale else 12
        bend = 4 if pale else 0
        s += f'<path d="M {x},{base} Q {x + bend},{base - h / 2} {x + bend},{base - h}" stroke="{stem}" stroke-width="2" fill="none"/>'
        s += f'<circle cx="{x + bend}" cy="{base - h - 2}" r="{3.2 if pale else 4.5}" fill="{colors[i % 3]}" stroke="{OUT}" stroke-width="1.2"/>'
    return s


def flower_box(x0, x1, y, pale):
    if pale is None:
        return ''
    s = f'<rect x="{x0}" y="{y}" width="{x1 - x0}" height="9" rx="2" fill="{WOOD}" {st(1.6)}/>'
    xs = [x0 + 5 + i * 7 for i in range(int((x1 - x0 - 6) / 7))]
    return flowers(xs, y, pale) + s


# ---------------------------------------------------------------- Gebäude (Ursprung unten Mitte)

def haus_elora(pale=True):
    s = chimney(40, -232)
    s += foundation(-86, 86, 0)
    s += timber(-86, 86, -150, -18, posts=4)
    s += gable(0, -150, 172, 96, TILE, TILE_LINE)
    s += window(0, -186, 22, 24)  # Giebelfenster
    s += door(-40, -18, color='#c9a24a')
    s += window(34, -86, 28, 34, '#d9a93c')
    s += flower_box(16, 52, -64, pale)
    return s


def haus_oma(pale=True):
    s = stone_wall(-78, 78, -120, 0, '#b8b1a6')
    s += f'<path d="M -98,-112 Q -60,-196 0,-206 Q 60,-196 98,-112 Z" fill="{THATCH}" {st()}/>'
    for i, y in enumerate((-130, -150, -170, -188)):
        w = 92 - i * 18
        s += f'<path d="M {-w},{y} Q 0,{y - 12} {w},{y}" fill="none" stroke="{THATCH_LINE}" stroke-width="1.8"/>'
    s += f'<path d="M -98,-112 Q 0,-122 98,-112" fill="none" stroke="{OUT}" stroke-width="4" stroke-linecap="round"/>'
    s += door(22, 0, 30, 58, '#8f7ab8')
    # Rundbogenfenster
    s += f'<path d="M -54,-50 V -80 Q -38,-98 -22,-80 V -50 Z" fill="{GLASS}" {st()}/>'
    s += f'<path d="M -38,-92 V -50 M -54,-66 H -22" stroke="{OUT}" stroke-width="1.8"/>'
    # Bank und Kräuterbeet
    s += f'<rect x="-76" y="-22" width="44" height="6" rx="2" fill="{WOOD}" {st(1.6)}/><path d="M -70,-16 V 0 M -38,-16 V 0" stroke="{OUT}" stroke-width="3"/>'
    s += f'<path d="M 50,0 Q 64,-8 78,0 Z" fill="#7a5a3e" {st(1.6)}/>'
    s += flowers([54, 61, 68, 75], -3, pale)
    return s


def brunnen(pale=True):
    s = ''
    # Beet ringsum
    s += f'<path d="M -84,0 Q -70,-10 -56,0 Z M 56,0 Q 70,-10 84,0 Z" fill="#7a5a3e" {st(1.6)}/>'
    s += flowers([-80, -73, -66, -59], -3, pale) + flowers([60, 67, 74, 81], -3, pale)
    s += stone_wall(-48, 48, -44, 0)
    s += f'<path d="M -52,-44 H 52" stroke="{OUT}" stroke-width="6" stroke-linecap="round"/>'
    # verschlossener Deckel mit Eisenbändern und Schloss (Weltbuch §2)
    s += f'<path d="M -50,-46 Q 0,-62 50,-46 Z" fill="{WOOD}" {st()}/>'
    for x in (-24, 24):
        s += f'<path d="M {x},-56 V -46" stroke="#5a5a5a" stroke-width="4"/>'
    s += f'<rect x="-6" y="-60" width="12" height="11" rx="2" fill="#8a8f96" {st(1.6)}/><path d="M -3,-60 Q 0,-67 3,-60" fill="none" {st(1.6)}/>'
    # Gestell, Kurbel, Eimer, Schindeldach
    for x in (-42, 42):
        s += f'<rect x="{x - 5}" y="-128" width="10" height="84" fill="{WOOD}" {st()}/>'
    s += f'<path d="M -42,-104 H 52" stroke="{OUT}" stroke-width="4"/><path d="M 52,-104 V -94 H 60" fill="none" stroke="{OUT}" stroke-width="3"/>'
    s += f'<path d="M 0,-104 V -84" stroke="{OUT}" stroke-width="1.5"/><path d="M -9,-84 H 9 L 7,-70 H -7 Z" fill="{WOOD}" {st(1.6)}/>'
    s += gable(0, -124, 96, 46, SHINGLE, SHINGLE_LINE, rows=3, overhang=10)
    return s


def werkstatt_tueftel(pale=True):
    s = chimney(-60, -250, 18, 60)
    s += f'<path d="M -51,-256 V -276 M -63,-276 L -39,-276" stroke="{OUT}" stroke-width="2.5"/><path d="M -51,-276 l 10,-6 l -10,-6 l -10,6 Z" fill="#c8ced6" {st(1.4)}/>'
    s += stone_wall(-92, 92, -96, 0, '#b48a6e')  # Ziegel unten
    s += timber(-92, 92, -168, -96, posts=4, braces=False)
    s += gable(0, -168, 184, 84, SLATE, SLATE_LINE)
    # großes Holztor
    s += f'<rect x="-70" y="-74" width="64" height="74" fill="{WOOD}" {st()}/><path d="M -38,-74 V 0 M -70,-50 H -6 M -70,-24 H -6" stroke="{WOOD_DARK}" stroke-width="2"/>'
    s += window(44, -48, 30, 30, '#3fa38f')
    s += window(-40, -132, 26, 24, '#3fa38f') + window(40, -132, 26, 24, '#3fa38f')
    # Zahnrad-Schild am Wandarm
    def gear(x, y):
        import math
        pts = []
        for i in range(16):
            a = math.pi * 2 * i / 16
            rr = 10 if i % 2 == 0 else 7.6
            pts.append(f'{x + rr * math.cos(a):.1f},{y + rr * math.sin(a):.1f}')
        return f'<polygon points="{" ".join(pts)}" fill="#c8ced6" {st(1.4)}/><circle cx="{x}" cy="{y}" r="3" fill="{WOOD}"/>'
    s += hanging_sign(114, -96, gear)
    s += f'<path d="M 92,-114 H 108" stroke="{OUT}" stroke-width="3"/>'
    return s


def schmiede_klonk(pale=True):
    s = f'<rect x="58" y="-262" width="28" height="110" fill="{STONE}" {st()}/><rect x="54" y="-268" width="36" height="8" fill="{STONE_LINE}" {st(1.6)}/>'
    s += f'<ellipse cx="72" cy="-282" rx="18" ry="9" fill="#d6d6d6" opacity="0.85"/><ellipse cx="80" cy="-300" rx="12" ry="6" fill="#e2e2e2" opacity="0.7"/>'
    s += stone_wall(-96, 96, -140, 0)
    s += side_roof(-96, 96, -140, 62, SLATE, SLATE_LINE)
    # offene Esse
    s += f'<path d="M -70,0 V -70 Q -36,-104 -2,-70 V 0 Z" fill="#2f2620" {st()}/>'
    s += f'<ellipse cx="-36" cy="-14" rx="26" ry="10" fill="#f28c3a"/><ellipse cx="-36" cy="-16" rx="14" ry="5" fill="#f8c060"/>'
    # Amboss auf Klotz
    s += f'<rect x="26" y="-20" width="34" height="20" fill="{WOOD_DARK}" {st()}/>'
    s += f'<path d="M 18,-20 H 70 Q 76,-34 62,-34 H 34 Q 18,-34 18,-20 Z" fill="#5a5a5a" {st()}/>'
    # Werkzeugbrett und Schild
    s += f'<path d="M 74,-96 H 92" stroke="{OUT}" stroke-width="3"/>'
    def hammer(x, y):
        return (f'<path d="M {x - 8},{y + 8} L {x + 6},{y - 6}" stroke="{OUT}" stroke-width="3.5" stroke-linecap="round"/>'
                f'<rect x="{x + 1}" y="{y - 12}" width="12" height="8" rx="1.5" fill="#c8ced6" {st(1.4)} transform="rotate(-45 {x + 7} {y - 8})"/>')
    s += hanging_sign(118, -80, hammer)
    return s


def laden_lotte(pale=True):
    s = chimney(-56, -238)
    s += foundation(-92, 92, 0)
    s += timber(-92, 92, -160, -18, posts=4, braces=False)
    s += gable(0, -160, 184, 90, TILE, TILE_LINE)
    s += window(0, -192, 22, 24)
    # Schaufenster mit Auslage
    s += f'<rect x="-76" y="-96" width="96" height="58" fill="{GLASS}" {st()}/><path d="M -28,-96 V -38" stroke="{OUT}" stroke-width="2"/>'
    for x, c in ((-64, '#c2506a'), (-50, '#4f96cf'), (-16, '#d9a93c'), (2, '#8a6fb8')):
        s += f'<circle cx="{x}" cy="-48" r="6" fill="{c}" {st(1.4)}/><rect x="{x - 2}" y="-58" width="4" height="5" fill="{WOOD}"/>'
    s += f'<rect x="-80" y="-38" width="104" height="6" fill="{WOOD}" {st(1.6)}/>'
    # Markise (Stoff, zweifarbig gedeckt)
    s += f'<path d="M -84,-112 H 28 L 36,-94 H -92 Z" fill="#3f6f9a" {st()}/>'
    for i in range(6):
        x = -84 + i * 20
        s += f'<path d="M {x + 10},-112 H {x + 20} L {x + 21.5},-94 H {x + 11.5} Z" fill="#e8dfc8"/>'
    s += f'<path d="M -84,-112 H 28 L 36,-94 H -92 Z" fill="none" {st()}/>'
    s += door(58, -18, 30, 58, '#3f6f9a')
    # Fässer und Kiste
    s += f'<rect x="-120" y="-30" width="26" height="30" rx="5" fill="{WOOD}" {st()}/><path d="M -120,-20 H -94 M -120,-10 H -94" stroke="{WOOD_DARK}" stroke-width="2"/>'
    s += f'<rect x="96" y="-24" width="26" height="24" fill="#c9955c" {st()}/><path d="M 96,-24 L 122,0 M 122,-24 L 96,0" stroke="{WOOD_DARK}" stroke-width="1.6"/>'
    def potion(x, y):
        return f'<circle cx="{x}" cy="{y + 2}" r="6.5" fill="#c2506a" {st(1.4)}/><rect x="{x - 2}" y="{y - 9}" width="4" height="6" fill="{WOOD}" {st(1)}/>'
    s += hanging_sign(-116, -130, potion)
    s += f'<path d="M -92,-148 H -136" stroke="{OUT}" stroke-width="0"/>'
    return s


def baumhaus_pip(pale=True):
    s = f'<path d="M -30,0 Q -16,-10 -16,-40 L -18,-250 H 18 L 16,-40 Q 16,-10 30,0 Z" fill="#7a5a3e" {st()}/>'
    s += f'<path d="M -8,-60 Q -4,-120 -8,-180 M 6,-90 Q 8,-130 5,-160" stroke="#5e4430" stroke-width="2" fill="none"/>'
    for (x, y, r, c) in ((-84, -282, 52, '#4f8f3a'), (78, -292, 56, '#4f8f3a'), (-10, -340, 66, '#5b9d42'),
                         (-46, -262, 44, '#5b9d42'), (50, -252, 44, '#5b9d42')):
        s += f'<circle cx="{x}" cy="{y}" r="{r + 2}" fill="{OUT}"/>'
    for (x, y, r, c) in ((-84, -282, 52, '#4f8f3a'), (78, -292, 56, '#4f8f3a'), (-10, -340, 66, '#5b9d42'),
                         (-46, -262, 44, '#5b9d42'), (50, -252, 44, '#5b9d42')):
        s += f'<circle cx="{x}" cy="{y}" r="{r}" fill="{c}"/>'
    # Plattform (5 Tiles hoch) mit Geländer und Bretterhütte
    s += f'<rect x="-80" y="-168" width="160" height="10" fill="{WOOD}" {st()}/>'
    s += f'<path d="M -80,-158 L -40,-120 M 80,-158 L 40,-120" stroke="{WOOD_DARK}" stroke-width="5"/>'
    s += f'<rect x="-56" y="-236" width="88" height="68" fill="#b8865a" {st()}/>'
    for x in range(-48, 32, 12):
        s += f'<path d="M {x},-236 V -168" stroke="{WOOD_DARK}" stroke-width="1.4"/>'
    s += side_roof(-56, 32, -236, 30, SHINGLE, SHINGLE_LINE, rows=3)
    s += window(-14, -204, 22, 22, '#e08a3c')
    s += f'<path d="M 58,-168 V -200 M 80,-168 V -200 M 58,-188 H 80" stroke="{WOOD_DARK}" stroke-width="3"/>'
    # Strickleiter
    s += f'<path d="M 50,-158 V 0 M 66,-158 V 0" stroke="#8a7050" stroke-width="2.5"/>'
    for y in range(-148, 0, 18):
        s += f'<path d="M 50,{y} H 66" stroke="{WOOD_DARK}" stroke-width="3" stroke-linecap="round"/>'
    return s


def anschlagbrett(pale=True):
    s = f'<rect x="-40" y="-70" width="8" height="70" fill="{WOOD}" {st()}/><rect x="32" y="-70" width="8" height="70" fill="{WOOD}" {st()}/>'
    s += f'<rect x="-46" y="-104" width="92" height="58" rx="3" fill="#b8865a" {st()}/>'
    s += gable(0, -104, 92, 22, SHINGLE, SHINGLE_LINE, rows=2, overhang=8)
    for (x, y, w, h, r) in ((-38, -96, 24, 28, -5), (-8, -94, 22, 24, 3), (20, -98, 20, 32, -2)):
        s += f'<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="#f6efdf" stroke="{OUT}" stroke-width="1.2" transform="rotate({r} {x + w / 2} {y + h / 2})"/>'
        s += f'<path d="M {x + 4},{y + 10} H {x + w - 4} M {x + 4},{y + 16} H {x + w - 6}" stroke="#9a8e7a" stroke-width="1.2" transform="rotate({r} {x + w / 2} {y + h / 2})"/>'
    return s


def wegweiser(pale=True):
    s = f'<rect x="-4" y="-92" width="8" height="92" fill="{WOOD}" {st()}/>'
    s += f'<path d="M -6,-84 H 42 L 52,-75 L 42,-66 H -6 Z" fill="#c9955c" {st()}/>'
    s += f'<path d="M 6,-58 H -42 L -52,-49 L -42,-40 H 6 Z" fill="#b8865a" {st()}/>'
    s += f'<path d="M 4,-78 H 34 M 4,-72 H 24 M -8,-52 H -36 M -8,-46 H -26" stroke="{WOOD_DARK}" stroke-width="1.6" stroke-linecap="round"/>'
    return s


def fahne(pale=True):
    cloth = '#c5c0b8' if pale else '#4f8f3a'
    band = '#b0aaa2' if pale else '#f2c14e'
    s = f'<rect x="-3" y="-150" width="6" height="150" fill="{WOOD}" {st(1.6)}/><circle cx="0" cy="-152" r="5" fill="#d9a93c" {st(1.4)}/>'
    s += f'<path d="M 3,-144 H 58 L 50,-122 L 58,-100 H 3 Z" fill="{cloth}" {st()}/>'
    s += f'<path d="M 3,-126 H 52" stroke="{band}" stroke-width="5"/>'
    return s


def beet(pale=True):
    s = f'<path d="M -50,0 Q -48,-10 -36,-12 H 36 Q 48,-10 50,0 Z" fill="#7a5a3e" {st()}/>'
    return s + flowers([-38, -30, -22, -14, -6, 2, 10, 18, 26, 34], -10, pale)


BUILDINGS = [
    ('haus-elora', 'Eloras Haus', 'Fachwerk, Ziegeldach, gelbe Läden, Blumenkasten', haus_elora, 190),
    ('haus-oma', 'Haus von Oma Pfütze', 'altes Steinhaus, Reetdach, Bank und Kräuterbeet', haus_oma, 200),
    ('brunnen', 'Dorfbrunnen', 'Deckel mit Eisenbändern und Schloss (Weltbuch §2), Schindeldach', brunnen, 180),
    ('werkstatt', 'Tüftels Werkstatt', 'Ziegel und Fachwerk, Schieferdach, Holztor, Zahnrad-Schild', werkstatt_tueftel, 240),
    ('schmiede', 'Klonks Schmiede', 'Stein, offene Esse, Amboss, Schornstein, Hammer-Schild', schmiede_klonk, 240),
    ('laden', 'Lottes Laden', 'Schaufenster mit Tränken, Markise, Fässer, Trank-Schild', laden_lotte, 260),
    ('baumhaus', 'Pips Baumhaus', 'Bretterhütte auf Plattform (5 Tiles), Strickleiter', baumhaus_pip, 180),
    ('anschlagbrett', 'Anschlagbrett', 'Aufgaben am Brunnenplatz', anschlagbrett, 100),
    ('wegweiser', 'Wegweiser', 'zum Lesen mit E (E-273)', wegweiser, 110),
    ('fahne', 'Fahne', 'verblasst / farbig (E-277)', fahne, 70),
    ('beet', 'Blumenbeet', 'verblasst / farbig (E-277)', beet, 110),
]


def elora():
    src = open('assets/elora/elora.svg').read()
    inner = src[src.index('>', src.index('<svg')) + 1:src.rindex('</svg>')]
    return re.sub(r'<!--.*?-->', '', inner, flags=re.S)


def text(x, y, t, size=15, color=TEXT, anchor='middle'):
    return f'<text x="{x}" y="{y}" font-size="{size}" fill="{color}" text-anchor="{anchor}">{t}</text>'


def scene(oy, pale, title):
    W = 1600
    o = [f'<rect x="0" y="{oy}" width="{W}" height="400" fill="url(#sky)"/>',
         f'<ellipse cx="300" cy="{oy + 380}" rx="520" ry="110" fill="#b5d3a0"/>',
         f'<ellipse cx="1200" cy="{oy + 390}" rx="560" ry="120" fill="#a6c990"/>',
         f'<rect x="0" y="{oy + 340}" width="{W}" height="60" fill="#a87a52"/>',
         f'<rect x="0" y="{oy + 334}" width="{W}" height="12" rx="6" fill="#6cbf4a"/>',
         text(30, oy + 30, title, 16, anchor='start')]
    ground = oy + 336
    scale = 0.58
    x = 40
    for _, _, _, fn, w in BUILDINGS:
        w *= scale
        o.append(f'<g transform="translate({x + w / 2},{ground}) scale({scale})">{fn(pale)}</g>')
        x += w + 14
    o.append(f'<g transform="translate({x + 8},{ground}) scale({0.36 * scale})">{elora()}</g>')
    return '\n'.join(o)


def sheet():
    W = 1600
    cols, cw, ch = 4, 380, 300
    rows = (len(BUILDINGS) + cols - 1) // cols
    top = 100
    scenes = top + rows * (ch + 10) + 20
    H = scenes + 2 * 410
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
         '<defs><linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#a9cde8"/>'
         '<stop offset="1" stop-color="#eef4f8"/></linearGradient></defs>',
         f'<rect width="{W}" height="{H}" fill="#e8eef3"/>',
         text(40, 46, 'Tauwinkel – Gebäude, zweite Fassung (A1.9, E-276, E-277)', 28, anchor='start'),
         text(40, 74, 'Richtige Dächer, Fachwerk, Stein und Holz in warmen, gedeckten Farben; die Farbe der Bewohner nur in Details. '
              'Maßstab: Elora rechts.', 15, anchor='start')]
    for i, (key, name, note, fn, _) in enumerate(BUILDINGS):
        cx = 30 + (i % cols) * (cw + 10)
        cy = top + (i // cols) * (ch + 10)
        o.append(f'<rect x="{cx}" y="{cy}" width="{cw}" height="{ch}" rx="16" fill="#fffaf0" stroke="#e7dcc8"/>')
        gy = cy + ch - 50
        o.append(f'<rect x="{cx + 16}" y="{gy}" width="{cw - 32}" height="8" rx="4" fill="#8fbf7a"/>')
        s = 0.58 if key in ('baumhaus',) else 0.8
        pale_pair = key in ('fahne', 'beet')
        if pale_pair:
            o.append(f'<g transform="translate({cx + cw / 2 - 90},{gy}) scale({s})">{fn(True)}</g>')
            o.append(f'<g transform="translate({cx + cw / 2 + 40},{gy}) scale({s})">{fn(False)}</g>')
        else:
            o.append(f'<g transform="translate({cx + cw / 2 - 26},{gy}) scale({s})">{fn(True)}</g>')
        o.append(f'<g transform="translate({cx + cw - 36},{gy}) scale({0.36 * s})">{elora()}</g>')
        o.append(text(cx + 18, cy + 28, name, 16, anchor='start'))
        o.append(text(cx + 18, cy + ch - 18, note, 11, '#8a7a66', anchor='start'))
    o.append(scene(scenes, True, 'Zu Beginn: Blumen, Beete und Fahnen verblasst (E-277)'))
    o.append(scene(scenes + 410, False, 'Nach den Quellen: Farben zurück'))
    o.append('</svg>')
    return '\n'.join(o)


def blumenkasten(pale=True):
    """Blumenkasten unter einem Fenster (eigene Deko, damit er verblassen kann)."""
    return flower_box(-18, 18, -9, pale)


def kraeuterbeet(pale=True):
    s = f'<path d="M -16,0 Q -2,-8 12,0 Z" fill="#7a5a3e" {st(1.6)}/>'
    return s + flowers([-12, -5, 2, 9], -3, pale)


# Deko für die Karten (E-278): Gebäude ohne Blumen, Blumen/Beete/Fahnen als Varianten (E-277)
DECOR = {
    'haus-elora': (haus_elora, '-140 -290 280 292'),
    'haus-oma': (haus_oma, '-120 -220 240 222'),
    'brunnen': (brunnen, '-100 -180 200 182'),
    'werkstatt': (werkstatt_tueftel, '-120 -300 270 302'),
    'schmiede': (schmiede_klonk, '-120 -320 270 322'),
    'laden': (laden_lotte, '-150 -260 300 262'),
    'baumhaus': (baumhaus_pip, '-150 -420 300 422'),
    'anschlagbrett': (anschlagbrett, '-60 -140 120 142'),
    'wegweiser': (wegweiser, '-60 -100 120 102'),
}
VARIANTS = {
    'blumenkasten': (blumenkasten, '-30 -30 60 32'),
    'beet': (beet, '-60 -30 120 32'),
    'kraeuterbeet': (kraeuterbeet, '-24 -24 48 26'),
    'fahne': (fahne, '-10 -160 80 162'),
}


def export_decor():
    out = 'assets/map/decor'
    def write(name, body, vb, note):
        with open(f'{out}/{name}.svg', 'w') as f:
            f.write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}">\n'
                    f'  <!-- {note} (A1.9, aus tools/design/tauwinkel_gebaeude.py). Ursprung unten in der Mitte. -->\n'
                    f'  {body}\n</svg>\n')
    for name, (fn, vb) in DECOR.items():
        write(name, fn(None), vb, f'Tauwinkel: {name}')
    for name, (fn, vb) in VARIANTS.items():
        write(f'{name}-blass', fn(True), vb, f'{name}, verblasst (E-277)')
        write(f'{name}-bunt', fn(False), vb, f'{name}, farbig (E-277)')


if __name__ == '__main__':
    os.makedirs('docs/release-2/design', exist_ok=True)
    open('docs/release-2/design/tauwinkel-gebaeude.svg', 'w').write(sheet())
    export_decor()

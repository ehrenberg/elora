"""Erzeugt die Kartengrafik im Stil A (M6.3, E-139) unter assets/map/.

Aufruf: python3 tools/design/elora_map_assets.py

Einmaliger Ausgangspunkt: Die erzeugten SVGs sind normale Assets und dürfen danach von Hand
geändert werden (dann dieses Skript nicht erneut ausführen oder vorher anpassen).

Koordinaten in Welteinheiten (1 Tile = 32):
- Materialien und Spezial-Tiles: viewBox 0 0 32 32 (ein Tile)
- Deko: Ursprung unten in der Mitte (steht auf dem Boden, y nach oben negativ)
- Hintergrund-Streifen: Ursprung unten links, 1024 breit und nahtlos wiederholbar
- Wolken, Mond: Ursprung in der Mitte; Sterne: Ursprung oben links, 1024 × 640
"""
import math
import os

OUT = '#2b2b2b'
T = 32
R = 10  # Rundung der Außenecken (wie materials.toml)


def write(path, view_box, body, comment):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w') as fh:
        fh.write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{view_box}">\n')
        fh.write(f'  <!-- {comment} -->\n')
        for line in body:
            fh.write(f'  {line}\n')
        fh.write('</svg>\n')


def g(part, *shapes):
    return f'<g id="{part}">' + ''.join(shapes) + '</g>'


def f(v):
    return f'{v:.2f}'.rstrip('0').rstrip('.')


# ── Kappen (Oberkante eines Materials) ──────────────────────────────────

def cap_outline(left, right, depth, wave, amp):
    """Pfad einer Kappe: oben bündig mit dem Tile, Ecken wie der Körper gerundet,
    unten gewellt (wave = Abstand der Bögen, amp = Tiefe der Bögen)."""
    d = []
    if left:
        d.append(f'M 0,{f(depth + amp)} L 0,{R} Q 0,0 {R},0')
    else:
        d.append('M 0,0')
    if right:
        d.append(f'H {T - R} Q {T},0 {T},{R} L {T},{f(depth + amp)}')
    else:
        d.append(f'H {T} V {f(depth)}')
    # Wellen von rechts nach links
    x = T
    while x > 0:
        nx = max(0, x - wave)
        d.append(f'Q {f((x + nx) / 2)},{f(depth + amp * 2)} {f(nx)},{f(depth)}')
        x = nx
    d.append('Z')
    return ' '.join(d)


def rim(left, right, h):
    """Schmaler Streifen direkt unter der Oberkante (Glanz/Schatten)."""
    a = f'M 0,{R} Q 0,0 {R},0' if left else 'M 0,0'
    b = f'H {T - R} Q {T},0 {T},{R} L {T - h},{R} Q {T - h},{h} {T - R},{h}' if right else f'H {T} V {h}'
    c = f'H {R} Q {h},{h} {h},{R} Z' if left else 'H 0 Z'
    return f'{a} {b} {c}'


CAP_PARTS = [('cap', False, False), ('cap-left', True, False), ('cap-right', False, True), ('cap-single', True, True)]


def material(name, comment, cap=None, details=()):
    body = []
    if cap:
        for part, left, right in CAP_PARTS:
            body.append(g(part, *cap(left, right)))
    for i, shapes in enumerate(details, 1):
        body.append(g(f'detail-{i}', *shapes))
    write(f'assets/map/materials/{name}.svg', f'0 0 {T} {T}', body, comment)


def grass(left, right):
    return [f'<path d="{cap_outline(left, right, 9, 4, 1.6)}" fill="#7bbf55"/>',
            f'<path d="{rim(left, right, 3)}" fill="#5f9c42" fill-opacity="0.5"/>',
            '<path d="M 7,7 l 1,-3 M 19,6 l 1,-3 M 26,7 l -1,-3" stroke="#a4dc7a" stroke-width="1.2" stroke-linecap="round" fill="none"/>']


def sand_cap(left, right):
    return [f'<path d="{cap_outline(left, right, 7, 8, 0.8)}" fill="#ecd08f"/>',
            '<path d="M 4,4 q 3,-1.5 6,0 M 17,5 q 3,-1.5 6,0" stroke="#d9b26f" stroke-width="1.2" stroke-linecap="round" fill="none"/>']


def snow_cap(left, right):
    return [f'<path d="{cap_outline(left, right, 9, 8, 2.2)}" fill="#f4f8fb"/>',
            f'<path d="M 3,{f(12.5)} q 1.5,3 3,0 Z M 21,{f(12.5)} q 1.5,4 3,0 Z" fill="#f4f8fb"/>',
            f'<path d="{rim(left, right, 2)}" fill="#ffffff"/>',
            '<path d="M 5,9.5 q 6,2 12,0" stroke="#cfdde8" stroke-width="1.2" stroke-linecap="round" fill="none"/>']


def ice_cap(left, right):
    return [f'<path d="{cap_outline(left, right, 4, 16, 0.5)}" fill="#e3f5fc"/>']


def spot(cx, cy, rx, ry, color):
    return f'<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{color}"/>'


def crack(d, color, w=2):
    return f'<path d="{d}" stroke="{color}" stroke-width="{w}" stroke-linecap="round" stroke-linejoin="round" fill="none"/>'


def materials():
    material('earth', 'Erde mit Grasnarbe (Stil A). Teile: Kappen für freie Oberkanten, Details im Inneren.', grass,
             [[spot(13, 20, 4, 3, '#8a6040')],
              [spot(20, 22, 2.5, 2, '#8a6040'), spot(11, 25, 1.6, 1.3, '#8a6040')],
              [crack('M 10,24 q 4,-3 7,-1 q 2,1 4,-2', '#8a6040', 1.6)]])
    material('sand', 'Sand (Stil A).', sand_cap,
             [[spot(14, 20, 1.5, 1.5, '#c39a55'), spot(19, 23, 1.2, 1.2, '#c39a55'), spot(11, 25, 1, 1, '#c39a55')],
              [crack('M 9,21 q 4,-2 8,0 q 4,2 7,0', '#c39a55', 1.4)],
              [spot(18, 19, 3, 2, '#e6c585')]])
    material('snow', 'Gefrorene Erde mit Schneedecke (Stil A).', snow_cap,
             [[spot(14, 21, 3.5, 2.6, '#74645a')],
              [spot(20, 23, 2, 1.6, '#74645a'), spot(11, 19, 1.4, 1.2, '#74645a')]])
    material('stone', 'Stein (nicht hookbar, Stil A): Risse statt Kappe.', None,
             [[crack('M 7,9 l 6,5 l -2,7', '#5f6870')],
              [crack('M 20,6 l -3,6 l 5,4', '#5f6870')],
              [crack('M 10,22 l 5,2 l 6,-3 M 15,24 l 1,4', '#5f6870')]])
    material('ice', 'Eis (Stil A): heller Rand und Glanzlinien.', ice_cap,
             [[crack('M 7,22 L 17,10', '#ffffff', 3)],
              [crack('M 13,25 L 21,15 M 19,26 L 23,21', '#ffffff', 2.2)]])


# ── Spezial-Tiles ───────────────────────────────────────────────────────

def specials():
    spikes = []
    for k in range(3):
        bx = k * T / 3
        spikes.append(f'<path d="M {f(bx + 0.5)},32 L {f(bx + T / 6)},7 L {f(bx + T / 3 - 0.5)},32 Z" fill="#c94f4f" stroke="{OUT}" stroke-width="1.6" stroke-linejoin="round"/>')
        spikes.append(f'<path d="M {f(bx + T / 6 - 0.5)},12 L {f(bx + T / 6 - 2.5)},24" stroke="#ef9a9a" stroke-width="1.4" stroke-linecap="round"/>')
    write('assets/map/tiles/death.svg', f'0 0 {T} {T}',
          [g('up', '<rect x="0" y="27" width="32" height="5" fill="#7c2a2a"/>', *spikes)],
          'Tod (Stil A): Stacheln nach oben; andere Richtungen werden gedreht.')

    def plank(left, right):
        l = 4 if left else 0
        r = 4 if right else 0
        d = (f'M {l},0 H {T - r} ' + (f'Q {T},0 {T},{r} V {10 - r} Q {T},10 {T - r},10 ' if right else f'H {T} V 10 ') +
             f'H {l} ' + (f'Q 0,10 0,{10 - l} V {l} Q 0,0 {l},0 Z' if left else 'H 0 V 0 Z'))
        edge = []
        edge.append(f'<path d="M {0 if not left else l},0 H {T if not right else T - r}" stroke="{OUT}" stroke-width="2"/>')
        edge.append(f'<path d="M {0 if not left else l},10 H {T if not right else T - r}" stroke="{OUT}" stroke-width="2"/>')
        if left:
            edge.append(f'<path d="M {l},0 Q 0,0 0,{l} V {10 - l} Q 0,10 {l},10" stroke="{OUT}" stroke-width="2" fill="none"/>')
        if right:
            edge.append(f'<path d="M {T - r},0 Q {T},0 {T},{r} V {10 - r} Q {T},10 {T - r},10" stroke="{OUT}" stroke-width="2" fill="none"/>')
        return [f'<path d="{d}" fill="#c9955c"/>',
                '<path d="M 5,5 H 14 M 19,4 H 27" stroke="#a87a52" stroke-width="1.3" stroke-linecap="round"/>',
                f'<path d="M 16,10 V 17" stroke="{OUT}" stroke-width="2" stroke-linecap="round"/>',
                *edge]

    write('assets/map/tiles/platform.svg', f'0 0 {T} {T}',
          [g('mid', *plank(False, False)), g('left', *plank(True, False)),
           g('right', *plank(False, True)), g('single', *plank(True, True))],
          'Plattform (Stil A): Brett an der Oberkante, Teile für Mitte, Enden und Einzelstück.')

    chevron = lambda y: f'<path d="M 9,{y + 7} L 16,{y} L 23,{y + 7}" stroke="#ffffff" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" fill="none"/>'
    pad = [f'<rect x="1" y="1" width="30" height="30" rx="5" fill="#ef7fb0" stroke="{OUT}" stroke-width="2"/>',
           '<rect x="4" y="22" width="24" height="6" rx="3" fill="#d0628f"/>',
           f'<rect x="3" y="1" width="26" height="5" rx="2.5" fill="#f7b3d1"/>']
    write('assets/map/tiles/jump.svg', f'0 0 {T} {T}',
          [g('up', *pad, chevron(6), chevron(13)),
           g('diag', *pad, f'<g transform="rotate(45 16 16)">{chevron(6)}{chevron(13)}</g>')],
          'Sprungfeld (Stil A): nach oben bzw. schräg nach rechts oben (links = gespiegelt).')

    write('assets/map/tiles/conveyor.svg', f'0 0 {T} {T}',
          [g('base', f'<rect x="1" y="1" width="30" height="30" rx="4" fill="#6c7a89" stroke="{OUT}" stroke-width="2"/>',
             '<rect x="3" y="3" width="26" height="4" rx="2" fill="#4f5b68"/>',
             '<circle cx="8" cy="24" r="3" fill="#9aa6b2"/>', '<circle cx="24" cy="24" r="3" fill="#9aa6b2"/>'),
           g('arrow', '<path d="M 11,9 L 19,15 L 11,21" stroke="#f2c14e" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round" fill="none"/>')],
          'Beschleuniger (Stil A): Gehäuse und Pfeil nach rechts (links = gespiegelt, Pfeil läuft später mit).')


# ── Deko ────────────────────────────────────────────────────────────────

def deco(name, view_box, shapes, comment):
    write(f'assets/map/decor/{name}.svg', view_box, shapes, comment + ' Ursprung unten in der Mitte.')


def stroke(w=2):
    return f'stroke="{OUT}" stroke-width="{w}"'


def decor():
    deco('bush-1', '-40 -32 80 34', [
        f'<ellipse cx="-6" cy="-12" rx="24" ry="14" fill="#5f9c42" {stroke()}/>',
        f'<ellipse cx="12" cy="-17" rx="15" ry="11" fill="#7bbf55" {stroke()}/>',
        '<ellipse cx="9" cy="-21" rx="5" ry="2.5" fill="#a4dc7a"/>'], 'Busch (Stil A).')
    deco('bush-2', '-50 -40 100 42', [
        f'<ellipse cx="-20" cy="-12" rx="18" ry="12" fill="#5f9c42" {stroke()}/>',
        f'<ellipse cx="18" cy="-12" rx="20" ry="13" fill="#5f9c42" {stroke()}/>',
        f'<ellipse cx="0" cy="-20" rx="20" ry="15" fill="#7bbf55" {stroke()}/>',
        '<ellipse cx="-4" cy="-28" rx="7" ry="3" fill="#a4dc7a"/>'], 'Breiter Busch (Stil A).')
    for name, petal, center in [('flower-pink', '#ef7fb0', '#f2c14e'), ('flower-yellow', '#f2c14e', '#e0574f'), ('flower-blue', '#6fa8e8', '#f4f8fb')]:
        petals = ''.join(f'<circle cx="{f(5 * math.cos(a))}" cy="{f(-20 + 5 * math.sin(a))}" r="3.6" fill="{petal}" {stroke(1.4)}/>'
                         for a in [i * 2 * math.pi / 5 - math.pi / 2 for i in range(5)])
        deco(name, '-12 -30 24 32', [
            '<path d="M 0,0 Q -2,-10 0,-18" stroke="#4f8a3a" stroke-width="2" fill="none" stroke-linecap="round"/>',
            f'<path d="M -1,-7 Q -7,-11 -8,-6 Q -4,-4 -1,-7 Z" fill="#7bbf55" {stroke(1.2)}/>',
            petals, f'<circle cx="0" cy="-20" r="2.6" fill="{center}" {stroke(1.2)}/>'], 'Blume (Stil A).')
    deco('grass-1', '-14 -18 28 19', [
        '<path d="M -10,0 Q -9,-8 -12,-14 Q -5,-8 -4,0 Z M -3,0 Q -1,-10 0,-16 Q 3,-9 3,0 Z M 4,0 Q 7,-7 11,-11 Q 10,-4 10,0 Z" fill="#7bbf55" stroke="#4f8a3a" stroke-width="1.2" stroke-linejoin="round"/>'],
        'Grasbüschel (Stil A).')
    deco('grass-2', '-22 -14 44 15', [
        '<path d="M -18,0 Q -17,-6 -20,-10 Q -13,-6 -12,0 Z M -8,0 Q -7,-8 -6,-12 Q -3,-7 -2,0 Z M 2,0 Q 5,-6 8,-9 Q 8,-4 8,0 Z M 12,0 Q 14,-8 18,-11 Q 17,-4 17,0 Z" fill="#5f9c42" stroke="#4f8a3a" stroke-width="1.2" stroke-linejoin="round"/>'],
        'Flaches Gras (Stil A).')
    deco('rock-1', '-20 -20 40 21', [
        f'<path d="M -16,0 Q -18,-12 -6,-16 Q 8,-19 14,-9 Q 18,-2 16,0 Z" fill="#9aa4ad" {stroke()}/>',
        '<path d="M -8,-12 Q -2,-15 4,-13" stroke="#c3cad1" stroke-width="2" stroke-linecap="round" fill="none"/>'], 'Stein (Stil A).')
    deco('rock-2', '-30 -16 60 17', [
        f'<path d="M -26,0 Q -26,-9 -16,-11 Q -10,-14 -4,-9 L -4,0 Z" fill="#8a949d" {stroke()}/>',
        f'<path d="M -6,0 Q -8,-12 6,-13 Q 20,-14 24,-5 Q 26,0 24,0 Z" fill="#9aa4ad" {stroke()}/>'], 'Steingruppe (Stil A).')
    deco('mushroom-red', '-14 -24 28 25', [
        f'<path d="M -4,0 Q -5,-8 -3,-12 H 3 Q 5,-8 4,0 Z" fill="#f4ecd8" {stroke(1.6)}/>',
        f'<path d="M -12,-11 Q -12,-22 0,-22 Q 12,-22 12,-11 Z" fill="#e0574f" {stroke(1.6)}/>',
        '<circle cx="-5" cy="-16" r="2" fill="#ffffff"/><circle cx="4" cy="-18" r="1.6" fill="#ffffff"/><circle cx="6" cy="-13.5" r="1.3" fill="#ffffff"/>'],
        'Fliegenpilz (Stil A).')
    deco('mushroom-brown', '-16 -20 32 21', [
        f'<path d="M -3,0 Q -4,-6 -2,-9 H 2 Q 4,-6 3,0 Z" fill="#f4ecd8" {stroke(1.6)}/>',
        f'<path d="M -9,-8 Q -9,-16 0,-16 Q 9,-16 9,-8 Z" fill="#a87a52" {stroke(1.6)}/>',
        f'<path d="M 6,0 Q 5,-4 7,-6 H 10 Q 11,-4 10,0 Z" fill="#f4ecd8" {stroke(1.4)}/>',
        f'<path d="M 4,-5 Q 4,-11 8.5,-11 Q 13,-11 13,-5 Z" fill="#c9955c" {stroke(1.4)}/>'], 'Pilzgruppe (Stil A).')
    deco('tree-round', '-60 -160 120 162', [
        f'<path d="M -8,0 Q -6,-30 -7,-70 H 7 Q 6,-30 8,0 Z" fill="#8a6040" {stroke()}/>',
        f'<path d="M -2,-50 Q -14,-60 -20,-58" stroke="{OUT}" stroke-width="2" fill="none" stroke-linecap="round"/>',
        f'<circle cx="-24" cy="-88" r="28" fill="#5f9c42" {stroke()}/>',
        f'<circle cx="24" cy="-90" r="26" fill="#5f9c42" {stroke()}/>',
        f'<circle cx="0" cy="-118" r="34" fill="#7bbf55" {stroke()}/>',
        '<ellipse cx="-10" cy="-132" rx="12" ry="6" fill="#a4dc7a"/>'], 'Laubbaum (Stil A).')
    layers = []
    for i, (y, w) in enumerate([(-36, 46), (-76, 38), (-112, 28)]):
        top = y - 44 + i * 6
        layers.append(f'<path d="M {-w},{y} Q 0,{y + 8} {w},{y} L 4,{top} Q 0,{top - 4} -4,{top} Z" fill="{"#4f8a3a" if i % 2 == 0 else "#5f9c42"}" {stroke()} stroke-linejoin="round"/>')
    deco('tree-pine', '-50 -170 100 172', [
        f'<rect x="-6" y="-30" width="12" height="30" rx="2" fill="#8a6040" {stroke()}/>', *layers], 'Nadelbaum (Stil A), passt zu Schnee.')
    deco('fence', '-34 -30 68 31', [
        f'<rect x="-30" y="-20" width="60" height="5" rx="2" fill="#c9955c" {stroke(1.6)}/>',
        f'<rect x="-30" y="-10" width="60" height="5" rx="2" fill="#c9955c" {stroke(1.6)}/>',
        f'<path d="M -28,0 V -24 L -24,-28 L -20,-24 V 0 Z" fill="#d9a86e" {stroke(1.6)} stroke-linejoin="round"/>',
        f'<path d="M 20,0 V -24 L 24,-28 L 28,-24 V 0 Z" fill="#d9a86e" {stroke(1.6)} stroke-linejoin="round"/>'], 'Zaun (Stil A), 64 breit.')
    deco('sign-arrow', '-26 -50 52 51', [
        f'<rect x="-3" y="-34" width="6" height="34" rx="2" fill="#a87a52" {stroke(1.6)}/>',
        f'<path d="M -20,-46 H 12 L 22,-38 L 12,-30 H -20 Z" fill="#c9955c" {stroke(1.8)} stroke-linejoin="round"/>',
        '<path d="M -14,-38 H 8" stroke="#8a6040" stroke-width="2" stroke-linecap="round"/>'], 'Wegweiser nach rechts (Stil A).')
    deco('sign-board', '-24 -52 48 53', [
        f'<rect x="-3" y="-30" width="6" height="30" rx="2" fill="#a87a52" {stroke(1.6)}/>',
        f'<rect x="-20" y="-48" width="40" height="22" rx="4" fill="#c9955c" {stroke(1.8)}/>',
        '<path d="M -13,-41 H 13 M -13,-34 H 6" stroke="#8a6040" stroke-width="2" stroke-linecap="round"/>'], 'Schild (Stil A).')


# ── Hintergründe ────────────────────────────────────────────────────────

W = 1024


def strip(name, height, fn, color, extra='', comment=''):
    """Periodischer Höhenzug über 1024 Einheiten (fn: x → Höhe über dem Boden)."""
    pts = [f'{x},{f(-fn(x))}' for x in range(0, W + 1, 16)]
    shape = f'<path d="M 0,0 L {" L ".join(pts)} L {W},0 Z" fill="{color}"/>'
    write(f'assets/map/backgrounds/{name}.svg', f'0 {-height} {W} {height}', [shape, extra],
          f'{comment} Ursprung unten links, {W} breit, nahtlos wiederholbar.')


def sines(x, parts):
    return sum(a * math.sin(2 * math.pi * k * x / W + p) for a, k, p in parts)


def backgrounds():
    strip('hills-far', 260, lambda x: 170 + sines(x, [(45, 2, 0.3), (20, 5, 1.1)]), '#b9d6e9', comment='Ferne Hügel (Stil A).')
    strip('hills-near', 200, lambda x: 110 + sines(x, [(40, 3, 2.0), (15, 7, 0.4)]), '#9fc7a0', comment='Nahe Hügel (Stil A).')

    # Berge: weiche Gipfel aus überlagerten Spitzen, Schneekappen
    peaks = [(80, 300, 150), (330, 360, 180), (560, 280, 140), (800, 340, 170), (1000, 260, 120)]

    def mountain(x):
        h = 60
        for px, ph, pw in peaks:
            for off in (-W, 0, W):
                d = abs(x - px - off)
                if d < pw:
                    t = d / pw
                    h = max(h, ph * (1 - t * t * (3 - 2 * t)) + 60 * t)
        return h
    caps = []
    for px, ph, pw in peaks:
        cw = pw * 0.28
        caps.append(f'<path d="M {f(px - cw)},{f(-ph + cw * 0.9)} Q {px},{-ph - 6} {f(px + cw)},{f(-ph + cw * 0.9)} '
                    f'Q {f(px + cw * 0.4)},{f(-ph + cw * 0.6)} {px},{f(-ph + cw * 1.0)} Q {f(px - cw * 0.4)},{f(-ph + cw * 0.6)} {f(px - cw)},{f(-ph + cw * 0.9)} Z" fill="#eef4f8"/>')
    strip('mountains', 380, mountain, '#a9bccf', ''.join(caps), 'Bergkette mit Schneekappen (Stil A).')

    # Wald: Reihe runder Baumkronen (Silhouette)
    trees = []
    for i in range(16):
        x = i * 64 + 32
        h = 120 + 30 * math.sin(i * 1.7)
        trees.append(f'<rect x="{x - 5}" y="{f(-h * 0.45)}" width="10" height="{f(h * 0.45)}" fill="#6a9a72"/>')
        trees.append(f'<circle cx="{x}" cy="{f(-h * 0.62)}" r="{f(h * 0.3)}" fill="#7fae86"/>')
        trees.append(f'<circle cx="{x + 18}" cy="{f(-h * 0.5)}" r="{f(h * 0.22)}" fill="#7fae86"/>')
    write('assets/map/backgrounds/forest.svg', f'0 -200 {W} 200',
          [f'<rect x="0" y="-40" width="{W}" height="40" fill="#7fae86"/>', *trees],
          f'Waldsilhouette (Stil A). Ursprung unten links, {W} breit, nahtlos wiederholbar.')

    for i, blobs in enumerate([[(-30, 0, 20), (0, -8, 28), (30, 0, 22)],
                               [(-45, 4, 16), (-18, -6, 24), (14, -10, 26), (44, 2, 18)],
                               [(-16, 0, 14), (6, -5, 18), (24, 1, 13)]], 1):
        shapes = [f'<ellipse cx="0" cy="12" rx="{f(abs(blobs[0][0]) + abs(blobs[-1][0]) + 10)}" ry="8" fill="#ffffff" fill-opacity="0.85"/>']
        shapes += [f'<circle cx="{x}" cy="{y}" r="{r}" fill="#ffffff" fill-opacity="0.85"/>' for x, y, r in blobs]
        write(f'assets/map/backgrounds/cloud-{i}.svg', '-80 -50 160 80', shapes, 'Wolke (Stil A). Ursprung in der Mitte.')

    stars = []
    seed = 7
    for _ in range(90):
        seed = (seed * 1103515245 + 12345) % 2 ** 31
        x = seed % W
        seed = (seed * 1103515245 + 12345) % 2 ** 31
        y = seed % 640
        seed = (seed * 1103515245 + 12345) % 2 ** 31
        big = seed % 9 == 0
        if big:
            stars.append(f'<path d="M {x},{y - 6} Q {x + 1},{y - 1} {x + 6},{y} Q {x + 1},{y + 1} {x},{y + 6} Q {x - 1},{y + 1} {x - 6},{y} Q {x - 1},{y - 1} {x},{y - 6} Z" fill="#fff6d8"/>')
        else:
            stars.append(f'<circle cx="{x}" cy="{y}" r="{1 + seed % 3 * 0.5}" fill="#ffffff" fill-opacity="{0.6 + seed % 4 * 0.1:.1f}"/>')
    write('assets/map/backgrounds/stars.svg', f'0 0 {W} 640', stars, f'Sternenfeld (Stil A). Ursprung oben links, {W} × 640, wiederholbar.')

    write('assets/map/backgrounds/moon.svg', '-60 -60 120 120', [
        '<circle cx="0" cy="0" r="52" fill="#f4f1d6" fill-opacity="0.18"/>',
        '<circle cx="0" cy="0" r="40" fill="#f4f1d6"/>',
        '<circle cx="-12" cy="-8" r="7" fill="#e2dcb8"/><circle cx="10" cy="12" r="5" fill="#e2dcb8"/><circle cx="14" cy="-14" r="3.5" fill="#e2dcb8"/>'],
        'Mond (Stil A) mit Schein. Ursprung in der Mitte.')


if __name__ == '__main__':
    materials()
    specials()
    decor()
    backgrounds()

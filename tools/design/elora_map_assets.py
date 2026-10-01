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


def stone_cap(left, right):
    return [f'<path d="{rim(left, right, 4)}" fill="#a5aeb6"/>']


def stone_blocks(rects):
    """Felsblöcke (x0, y0, x1, y1) mit abgerundeten Ecken: Fläche, Lichtkante oben, Schatten unten."""
    out = []
    for x0, y0, x1, y1 in rects:
        w, h = x1 - x0 - 1, y1 - y0 - 1
        out.append(f'<rect x="{x0 + 0.5}" y="{y0 + 0.5}" width="{w}" height="{h}" rx="3" fill="#8a949d"/>')
        out.append(f'<path d="M {x0 + 3},{y0 + 2} H {x1 - 3}" stroke="#a5aeb6" stroke-width="1.6" stroke-linecap="round"/>')
        out.append(f'<path d="M {x0 + 3},{y1 - 1.5} H {x1 - 3}" stroke="#66707a" stroke-width="1.6" stroke-linecap="round"/>')
        if w > 10 and h > 10:
            cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
            out.append(crack(f'M {f(cx - 3)},{f(cy - 2)} l 2,2 l -1,3', '#66707a', 1.2))
    return out


def rocks(polys):
    """Unregelmäßige Felsbrocken: Fläche, Lichtkante an der Oberseite, Schatten unten, Fugen dazwischen."""
    out = []
    for pts in polys:
        d = 'M ' + ' L '.join(f'{x},{y}' for x, y in pts) + ' Z'
        out.append(f'<path d="{d}" fill="#7d8790" stroke="#4f5860" stroke-width="1.4" stroke-linejoin="round"/>')
        (x0, y0), (x1, y1) = pts[0], pts[1]
        out.append(f'<path d="M {f(x0 + (x1 - x0) * 0.15)},{f(y0 + (y1 - y0) * 0.15 + 1.6)} L {f(x0 + (x1 - x0) * 0.85)},{f(y0 + (y1 - y0) * 0.85 + 1.6)}" stroke="#a5aeb6" stroke-width="1.8" stroke-linecap="round"/>')
        xs = [p[0] for p in pts]
        ys = [p[1] for p in pts]
        cx, cy = sum(xs) / len(xs), sum(ys) / len(ys)
        if max(xs) - min(xs) > 12 and max(ys) - min(ys) > 10:
            out.append(crack(f'M {f(cx - 3)},{f(cy - 1)} l 3,2 l -1,3', '#5f6870', 1.2))
    return out


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
    material('stone', 'Stein (nicht hookbar, Stil A): Felsblöcke mit Fugen, heller Grat an freien Oberkanten.', stone_cap,
             [rocks([[(2, 3), (17, 1), (22, 9), (16, 18), (3, 16)],
                     [(24, 4), (31, 6), (30, 19), (19, 20)],
                     [(3, 20), (14, 21), (20, 31), (2, 30)]]),
              rocks([[(1, 2), (13, 2), (15, 14), (2, 13)],
                     [(17, 1), (31, 3), (30, 17), (20, 15)],
                     [(4, 17), (27, 20), (29, 31), (6, 30)]]),
              rocks([[(3, 1), (28, 2), (30, 12), (5, 13)],
                     [(2, 16), (16, 15), (18, 30), (1, 29)],
                     [(20, 17), (31, 16), (30, 30), (22, 31)]])])
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
           g('arrow', f'<path d="M 7,15 H 24 M 18,9 L 24,15 L 18,21" stroke="{OUT}" stroke-width="6" stroke-linecap="round" stroke-linejoin="round" fill="none"/>',
             '<path d="M 7,15 H 24 M 18,9 L 24,15 L 18,21" stroke="#f2c14e" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round" fill="none"/>')],
          'Beschleuniger (Stil A): Gehäuse und Pfeil nach rechts (links = gespiegelt, Pfeil läuft später mit).')


# ── Deko ────────────────────────────────────────────────────────────────

def deco(name, view_box, shapes, comment):
    write(f'assets/map/decor/{name}.svg', view_box, shapes, comment + ' Ursprung unten in der Mitte.')


def stroke(w=2):
    return f'stroke="{OUT}" stroke-width="{w}"'


def foliage(blobs, dark, light, highlight, leaves=()):
    """Laubmasse aus Kreisen: erst alle mit Kontur, dann alle ohne darüber → eine Außenkontur.
    Unterer Teil dunkler (Schatten), oben Lichter und einzelne Blattbögen."""
    out = [f'<circle cx="{x}" cy="{y}" r="{r + 1.2}" fill="{OUT}"/>' for x, y, r in blobs]
    out += [f'<circle cx="{x}" cy="{y}" r="{r}" fill="{dark}"/>' for x, y, r in blobs]
    out += [f'<circle cx="{f(x - r * 0.12)}" cy="{f(y - r * 0.18)}" r="{f(r * 0.8)}" fill="{light}"/>' for x, y, r in blobs]
    out += [f'<ellipse cx="{f(x - r * 0.3)}" cy="{f(y - r * 0.45)}" rx="{f(r * 0.32)}" ry="{f(r * 0.18)}" fill="{highlight}"/>' for x, y, r in blobs if r > 8]
    for x, y, r in leaves:
        out.append(f'<path d="M {f(x - r)},{y} Q {x},{f(y + r * 0.9)} {f(x + r)},{y}" stroke="{dark}" stroke-width="1.6" stroke-linecap="round" fill="none"/>')
    return out


def decor():
    deco('bush-1', '-40 -40 80 42', foliage(
        [(-22, -7, 8), (-12, -13, 11), (2, -19, 12), (15, -13, 11), (24, -7, 8), (-4, -8, 11), (10, -6, 10)],
        '#4f8a3a', '#6aae4a', '#a4dc7a', [(-10, -10, 3), (6, -14, 3), (16, -6, 2.5)]),
        'Busch (Stil A): Laub aus vielen Blattbögen.')
    deco('bush-2', '-56 -50 112 52', foliage(
        [(-38, -7, 9), (-28, -15, 12), (-14, -22, 13), (2, -27, 14), (18, -22, 13), (32, -15, 12), (42, -7, 9),
         (-20, -8, 12), (0, -10, 13), (20, -8, 12)],
        '#4f8a3a', '#6aae4a', '#a4dc7a', [(-24, -12, 3), (-4, -18, 3.5), (14, -12, 3), (30, -8, 2.5)])
        + ['<circle cx="-16" cy="-14" r="2.2" fill="#e0574f"/><circle cx="8" cy="-22" r="2.2" fill="#e0574f"/><circle cx="26" cy="-12" r="2.2" fill="#e0574f"/>'],
        'Breiter Busch mit Beeren (Stil A).')
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
    trunk = [f'<path d="M -16,0 Q -9,-4 -8,-14 Q -7,-40 -9,-70 L -24,-92 L -19,-95 L -4,-78 L 0,-100 L 6,-99 L 5,-78 L 20,-96 L 25,-92 L 9,-68 Q 7,-40 8,-14 Q 9,-4 16,0 Z" fill="#8a6040" {stroke()} stroke-linejoin="round"/>',
             '<path d="M -2,-12 Q -3,-34 -1,-56 M 3,-24 Q 4,-36 3,-46" stroke="#6e4c32" stroke-width="1.6" stroke-linecap="round" fill="none"/>']
    crown = foliage(
        [(-38, -96, 18), (-24, -116, 22), (-2, -130, 26), (22, -118, 22), (38, -98, 18),
         (-20, -92, 20), (2, -100, 22), (24, -90, 18), (-8, -150, 18), (14, -146, 16)],
        '#4f8a3a', '#6aae4a', '#a4dc7a',
        [(-26, -104, 4), (-4, -118, 5), (18, -104, 4), (4, -138, 4), (30, -92, 3), (-34, -90, 3)])
    deco('tree-round', '-64 -172 128 174', trunk + crown + [
        '<path d="M -8,-86 Q -2,-80 4,-86" stroke="#6e4c32" stroke-width="2" stroke-linecap="round" fill="none"/>'],
        'Laubbaum (Stil A): Stamm mit Ästen, Krone aus Blattbögen.')
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
    for i in range(20):
        x = i * 51.2 + 25
        h = 230 + 40 * math.sin(i * 1.7) + 20 * math.sin(i * 3.1)
        if i % 3 == 1:
            # Nadelbaum
            trees.append(f'<rect x="{f(x - 4)}" y="{f(-h * 0.3)}" width="8" height="{f(h * 0.3)}" fill="#55805c"/>')
            for k in range(3):
                yb, wb = -h * (0.22 + k * 0.2), 40 - k * 9
                trees.append(f'<path d="M {f(x - wb)},{f(yb)} Q {f(x)},{f(yb + 6)} {f(x + wb)},{f(yb)} L {f(x + 6)},{f(yb - h * 0.24)} Q {f(x)},{f(yb - h * 0.28)} {f(x - 6)},{f(yb - h * 0.24)} Z" fill="#5f8f68"/>')
        else:
            trees.append(f'<rect x="{f(x - 5)}" y="{f(-h * 0.5)}" width="10" height="{f(h * 0.5)}" fill="#55805c"/>')
            for dx, dy, r in [(0, 0.72, 0.2), (-16, 0.6, 0.15), (16, 0.6, 0.15), (0, 0.86, 0.13)]:
                trees.append(f'<circle cx="{f(x + dx)}" cy="{f(-h * dy)}" r="{f(h * r)}" fill="#6a9a72"/>')
            trees.append(f'<circle cx="{f(x - 8)}" cy="{f(-h * 0.8)}" r="{f(h * 0.06)}" fill="#86b48d"/>')
    write('assets/map/backgrounds/forest.svg', f'0 -300 {W} 300',
          [f'<rect x="0" y="-60" width="{W}" height="60" fill="#6a9a72"/>', *trees],
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

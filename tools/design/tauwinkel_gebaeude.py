"""Entwürfe der Gebäude von Tauwinkel (A1.9, E-274) als Karten-Deko im Stil A.

Aufruf: python3 tools/design/tauwinkel_gebaeude.py && cargo xtask svg-preview \
        docs/release-2/design/tauwinkel-gebaeude.svg docs/release-2/design/tauwinkel-gebaeude.png 1600

Alle Gebäude in Welteinheiten (1 Tile = 32, Elora ≈ 47 hoch), Ursprung unten in der Mitte –
so können sie nach der Auswahl direkt als Deko (`assets/map/decor/`) übernommen werden.
Idee: Die Tropfenwesen wohnen in Häusern mit Tropfendach; jede Figur hat ihre Farbe.
"""
import math
import os
import re

OUT = '#2b2b2b'
SW = 2.2
WALL = '#f3e6cc'
WALL_SHADE = '#e2d0ae'
WOOD = '#a87a52'
WOOD_DARK = '#7a5434'
STONE = '#b8bfc6'
STONE_DARK = '#8f979f'
GLASS = '#bfe6f5'
TEXT = '#3b3024'


def st(w=SW):
    return f'stroke="{OUT}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'


def drop_roof(cx, base, w, h, color, tip=0.18):
    """Tropfendach: breite runde Kuppe, Spitze leicht zur Seite geneigt."""
    l, r = cx - w / 2, cx + w / 2
    tx, ty = cx + w * tip, base - h
    return (f'<path d="M {l},{base} C {l},{base - h * 0.55} {cx - w * 0.15},{base - h * 0.78} {tx},{ty} '
            f'C {cx + w * 0.38},{base - h * 0.7} {r},{base - h * 0.5} {r},{base} Z" fill="{color}" {st()}/>'
            f'<path d="M {l + w * 0.12},{base - h * 0.28} Q {cx - w * 0.1},{base - h * 0.62} {tx - w * 0.06},{ty + h * 0.12}" '
            f'fill="none" stroke="#ffffff" stroke-width="3" stroke-linecap="round" opacity="0.45"/>')


def round_door(cx, base, w=30, h=50, color=WOOD):
    return (f'<path d="M {cx - w / 2},{base} V {base - h + w / 2} A {w / 2},{w / 2} 0 0 1 {cx + w / 2},{base - h + w / 2} V {base} Z" fill="{color}" {st()}/>'
            f'<circle cx="{cx + w / 2 - 6}" cy="{base - h * 0.42}" r="2.2" fill="{OUT}"/>')


def round_window(cx, cy, r=10):
    return (f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="{GLASS}" {st()}/>'
            f'<path d="M {cx - r},{cy} H {cx + r} M {cx},{cy - r} V {cy + r}" stroke="{OUT}" stroke-width="1.6"/>')


def wall(x0, x1, base, h, color=WALL):
    return (f'<rect x="{x0}" y="{base - h}" width="{x1 - x0}" height="{h}" rx="6" fill="{color}" {st()}/>'
            f'<rect x="{x0 + 3}" y="{base - 10}" width="{x1 - x0 - 6}" height="7" rx="3" fill="{WALL_SHADE}"/>')


def flowers(xs, base, colors=('#ef7fb0', '#f2c14e', '#5aaee8')):
    s = ''
    for i, x in enumerate(xs):
        c = colors[i % len(colors)]
        s += f'<path d="M {x},{base} V {base - 12}" stroke="#4f9a3a" stroke-width="2"/>'
        s += f'<circle cx="{x}" cy="{base - 14}" r="4.5" fill="{c}" stroke="{OUT}" stroke-width="1.4"/>'
    return s


# ---------------------------------------------------------------- Gebäude (Ursprung unten Mitte)

def haus_elora():
    s = f'<rect x="38" y="-170" width="16" height="40" rx="3" fill="{STONE}" {st()}/>'
    s += wall(-80, 80, 0, 92)
    s += drop_roof(0, -88, 196, 104, '#f2c14e')
    s += round_door(-30, 0)
    s += round_window(34, -52, 13)
    s += f'<rect x="20" y="-34" width="28" height="9" rx="3" fill="{WOOD}" {st(1.6)}/>'
    s += flowers([24, 31, 38, 45], -34)
    return s


def haus_oma():
    s = wall(-70, 70, 0, 84)
    s += drop_roof(0, -80, 176, 92, '#b9a3e3', tip=-0.16)
    s += round_door(10, 0, color='#8a6fb8')
    s += round_window(-38, -50, 12)
    # Schaukelstuhl
    s += f'<path d="M -60,0 Q -48,6 -34,0" fill="none" {st()}/><path d="M -56,-2 L -54,-30 M -40,-2 L -40,-18 M -56,-18 H -38" fill="none" {st()}/>'
    s += f'<path d="M -66,-92 Q -40,-128 -10,-148" fill="none" stroke="#ef7fb0" stroke-width="3" stroke-dasharray="1 7" stroke-linecap="round"/>'
    return s


def brunnen():
    s = f'<path d="M -46,0 L -50,-38 Q 0,-48 50,-38 L 46,0 Z" fill="{STONE}" {st()}/>'
    for y in (-12, -26):
        s += f'<path d="M -47,{y} Q 0,{y - 8} 47,{y}" fill="none" stroke="{STONE_DARK}" stroke-width="2"/>'
    for x in (-30, -6, 18, 38):
        s += f'<path d="M {x},-2 V -10 M {x + 10},-16 V -24" stroke="{STONE_DARK}" stroke-width="2"/>'
    # verschlossener Deckel (Weltbuch §2)
    s += f'<ellipse cx="0" cy="-42" rx="50" ry="9" fill="{WOOD}" {st()}/>'
    s += f'<path d="M -30,-46 L -30,-38 M -10,-50 L -10,-34 M 10,-50 L 10,-34 M 30,-46 L 30,-38" stroke="{WOOD_DARK}" stroke-width="2"/>'
    s += f'<rect x="-8" y="-50" width="16" height="10" rx="3" fill="#e0b85a" {st(1.8)}/>'
    # Gestell mit Dach und Eimer
    s += f'<path d="M -44,-40 V -112 M 44,-40 V -112" stroke="{OUT}" stroke-width="9" stroke-linecap="round"/>'
    s += f'<path d="M -44,-40 V -112 M 44,-40 V -112" stroke="{WOOD}" stroke-width="5" stroke-linecap="round"/>'
    s += f'<path d="M -62,-104 L 0,-138 L 62,-104 Z" fill="#e8685a" {st()}/>'
    s += f'<path d="M -44,-96 H 44" stroke="{OUT}" stroke-width="4"/>'
    s += f'<path d="M 0,-96 V -74" stroke="{OUT}" stroke-width="1.5"/><path d="M -9,-74 H 9 L 6,-60 H -6 Z" fill="{WOOD}" {st(1.6)}/>'
    return s


def werkstatt_tueftel():
    s = f'<rect x="30" y="-196" width="14" height="70" rx="3" fill="{STONE}" {st()}/>'
    s += f'<path d="M 37,-198 l -14,-6 l 14,-6 l 14,6 Z" fill="#c8ced6" {st(1.6)}/>'
    s += wall(-84, 84, 0, 96)
    s += drop_roof(0, -92, 204, 98, '#7fd99a')
    s += f'<rect x="-64" y="-64" width="54" height="64" rx="6" fill="{WOOD}" {st()}/>'
    s += f'<path d="M -64,-44 H -10 M -64,-24 H -10" stroke="{WOOD_DARK}" stroke-width="2"/>'
    s += round_window(40, -56, 14)
    # Zahnrad-Schild
    cx, cy, r = 6, -128, 16
    pts = []
    for i in range(16):
        a = math.pi * 2 * i / 16
        rr = r if i % 2 == 0 else r * 0.78
        pts.append(f'{cx + rr * math.cos(a):.1f},{cy + rr * math.sin(a):.1f}')
    s += f'<polygon points="{" ".join(pts)}" fill="#c8ced6" {st(1.8)}/><circle cx="{cx}" cy="{cy}" r="5" fill="{WOOD}" {st(1.6)}/>'
    s += f'<circle cx="70" cy="-18" r="9" fill="#f2c14e" {st()}/><path d="M 70,-27 V -40" stroke="{OUT}" stroke-width="2"/>'
    return s


def schmiede_klonk():
    s = f'<rect x="-76" y="-190" width="22" height="80" rx="3" fill="{STONE_DARK}" {st()}/>'
    s += f'<ellipse cx="-65" cy="-196" rx="16" ry="7" fill="#d8d8d8" opacity="0.8"/>'
    s += f'<rect x="-86" y="-96" width="172" height="96" rx="6" fill="{STONE}" {st()}/>'
    for y in (-72, -48, -24):
        s += f'<path d="M -86,{y} H 86" stroke="{STONE_DARK}" stroke-width="2"/>'
    s += drop_roof(0, -92, 204, 86, '#a8744a', tip=-0.2)
    s += f'<path d="M -50,0 V -54 Q -20,-80 10,-54 V 0 Z" fill="#3b2a1e" {st()}/><ellipse cx="-20" cy="-14" rx="22" ry="10" fill="#f28c3a" opacity="0.85"/>'
    # Amboss
    s += f'<path d="M 30,0 L 36,-14 H 62 L 68,0 Z M 26,-14 H 76 Q 82,-26 70,-26 H 36 Q 26,-26 26,-14 Z" fill="#5a5a5a" {st()}/>'
    # Hammer-Schild
    s += f'<rect x="20" y="-140" width="58" height="34" rx="5" fill="{WOOD}" {st()}/>'
    s += f'<path d="M 34,-114 L 60,-130" stroke="{OUT}" stroke-width="5" stroke-linecap="round"/><rect x="52" y="-138" width="16" height="10" rx="2" fill="#c8ced6" {st(1.6)} transform="rotate(-30 60 -133)"/>'
    return s


def laden_lotte():
    s = wall(-80, 80, 0, 96)
    s += drop_roof(0, -92, 196, 90, '#8fd0f0', tip=0.22)
    # gestreifte Markise
    s += f'<path d="M -84,-82 H 84 L 76,-58 H -76 Z" fill="#ffffff" {st()}/>'
    for i in range(8):
        x = -84 + i * 21
        s += f'<path d="M {x},-82 H {x + 10.5} L {x + 9},-58 H {x - 1} Z" fill="#5aaee8"/>'
    s += f'<path d="M -84,-82 H 84 L 76,-58 H -76 Z" fill="none" {st()}/>'
    s += f'<rect x="-70" y="-40" width="100" height="40" rx="4" fill="{WOOD}" {st()}/>'
    s += f'<path d="M -70,-26 H 30" stroke="{WOOD_DARK}" stroke-width="2"/>'
    for x, c in ((-56, '#e05a7a'), (-36, '#5aaee8'), (-14, '#f2c14e')):
        s += f'<circle cx="{x}" cy="-48" r="7" fill="{c}" {st(1.6)}/>'
    s += round_door(56, 0, 26, 50)
    # Kisten
    s += f'<rect x="-104" y="-26" width="26" height="26" rx="3" fill="#c9955c" {st()}/><path d="M -104,-26 L -78,0 M -78,-26 L -104,0" stroke="{WOOD_DARK}" stroke-width="2"/>'
    for (x, y) in ((-60, -120), (40, -130), (-10, -150)):
        s += f'<path d="M {x},{y - 6} L {x + 2},{y - 2} L {x + 6},{y} L {x + 2},{y + 2} L {x},{y + 6} L {x - 2},{y + 2} L {x - 6},{y} L {x - 2},{y - 2} Z" fill="#ffffff" {st(1.2)}/>'
    return s


def baumhaus_pip():
    # Stamm
    s = f'<path d="M -22,0 Q -14,-6 -14,-30 V -230 H 14 V -30 Q 14,-6 22,0 Z" fill="#8a6040" {st()}/>'
    # Krone hinten
    for (x, y, r) in ((-70, -270, 46), (60, -280, 50), (0, -320, 58), (-30, -250, 40), (40, -240, 40)):
        s += f'<circle cx="{x}" cy="{y}" r="{r + 2}" fill="{OUT}"/>'
    for (x, y, r) in ((-70, -270, 46), (60, -280, 50), (0, -320, 58), (-30, -250, 40), (40, -240, 40)):
        s += f'<circle cx="{x}" cy="{y}" r="{r}" fill="#6cbf4a"/>'
    # Häuschen auf der Plattform (Boden bei −160 = 5 Tiles)
    s += f'<rect x="-70" y="-168" width="140" height="10" rx="3" fill="{WOOD}" {st()}/>'
    s += f'<rect x="-48" y="-232" width="82" height="64" rx="6" fill="#f28c3a" {st()}/>'
    s += f'<path d="M -56,-230 L -6,-262 L 42,-230 Z" fill="#d94a4a" {st()}/>'
    s += round_window(-8, -204, 10)
    s += f'<rect x="16" y="-212" width="12" height="16" rx="2" fill="#5aaee8" {st(1.4)}/>'
    # Strickleiter
    s += f'<path d="M 50,-160 V 0 M 66,-160 V 0" stroke="{WOOD_DARK}" stroke-width="2.5"/>'
    for y in range(-150, 0, 18):
        s += f'<path d="M 50,{y} H 66" stroke="{WOOD_DARK}" stroke-width="3" stroke-linecap="round"/>'
    # Fähnchen
    s += f'<path d="M -46,-232 V -262 L -26,-254 L -46,-248" fill="#f2c14e" {st(1.6)}/>'
    return s


def anschlagbrett():
    s = f'<path d="M -34,0 V -70 M 34,0 V -70" stroke="{OUT}" stroke-width="8" stroke-linecap="round"/>'
    s += f'<path d="M -34,0 V -70 M 34,0 V -70" stroke="{WOOD}" stroke-width="4" stroke-linecap="round"/>'
    s += f'<rect x="-42" y="-96" width="84" height="56" rx="5" fill="#c9955c" {st()}/>'
    s += f'<path d="M -48,-98 L 0,-116 L 48,-98 Z" fill="#a8744a" {st()}/>'
    for (x, y, w, h, r) in ((-34, -88, 22, 26, -6), (-6, -86, 20, 22, 4), (18, -90, 18, 30, -3)):
        s += f'<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="#fffaf0" stroke="{OUT}" stroke-width="1.2" transform="rotate({r} {x + w / 2} {y + h / 2})"/>'
        s += f'<circle cx="{x + w / 2}" cy="{y + 3}" r="2" fill="#e8685a"/>'
    return s


def wegweiser():
    s = f'<path d="M 0,0 V -86" stroke="{OUT}" stroke-width="9" stroke-linecap="round"/>'
    s += f'<path d="M 0,0 V -86" stroke="{WOOD}" stroke-width="5" stroke-linecap="round"/>'
    s += f'<path d="M -6,-80 H 40 L 50,-72 L 40,-64 H -6 Z" fill="#c9955c" {st()}/>'
    s += f'<path d="M 6,-56 H -40 L -50,-48 L -40,-40 H 6 Z" fill="#e0c89a" {st()}/>'
    s += f'<path d="M 4,-74 H 32 M -8,-48 H -36" stroke="{WOOD_DARK}" stroke-width="2" stroke-linecap="round"/>'
    return s


BUILDINGS = [
    ('haus-elora', 'Eloras Haus', 'gelbes Tropfendach, Rundtür, Blumenkasten', haus_elora),
    ('haus-oma', 'Haus von Oma Pfütze', 'lila Dach, Schaukelstuhl, ein Lied in der Luft', haus_oma),
    ('brunnen', 'Dorfbrunnen', 'verschlossener Holzdeckel (Weltbuch §2), Eimer, Dach', brunnen),
    ('werkstatt', 'Tüftels Werkstatt', 'Zahnrad-Schild, Rohr mit Kappe, Lampe', werkstatt_tueftel),
    ('schmiede', 'Klonks Schmiede', 'Stein, Glut im Tor, Amboss, Hammer-Schild', schmiede_klonk),
    ('laden', 'Lottes Laden', 'gestreifte Markise, Tresen mit Tränken, Glitzer', laden_lotte),
    ('baumhaus', 'Pips Baumhaus', 'Häuschen auf einer Plattform (5 Tiles hoch), Strickleiter', baumhaus_pip),
    ('anschlagbrett', 'Anschlagbrett', 'Aufgaben am Brunnenplatz', anschlagbrett),
    ('wegweiser', 'Wegweiser', 'Schild zum Lesen (E-273)', wegweiser),
]


def elora():
    src = open('assets/elora/elora.svg').read()
    inner = src[src.index('>', src.index('<svg')) + 1:src.rindex('</svg>')]
    return re.sub(r'<!--.*?-->', '', inner, flags=re.S)


def text(x, y, t, size=15, color=TEXT, anchor='middle'):
    return f'<text x="{x}" y="{y}" font-size="{size}" fill="{color}" text-anchor="{anchor}">{t}</text>'


def scene(oy, pale):
    """Eine Reihe aller Gebäude auf Wiese; `pale` = Filter wie im Dorf (E-275)."""
    W = 1600
    o = [f'<rect x="0" y="{oy}" width="{W}" height="420" fill="url(#sky)"/>',
         f'<ellipse cx="300" cy="{oy + 400}" rx="520" ry="120" fill="#b5d9a0"/>',
         f'<ellipse cx="1200" cy="{oy + 410}" rx="560" ry="130" fill="#a6cf90"/>',
         f'<rect x="0" y="{oy + 360}" width="{W}" height="60" fill="#a87a52"/>',
         f'<rect x="0" y="{oy + 354}" width="{W}" height="12" rx="6" fill="#6cbf4a"/>']
    ground = oy + 356
    scale = 0.62
    x = 70
    widths = {'haus-elora': 200, 'haus-oma': 180, 'brunnen': 130, 'werkstatt': 210, 'schmiede': 210,
              'laden': 220, 'baumhaus': 170, 'anschlagbrett': 100, 'wegweiser': 110}
    for key, _, _, fn in BUILDINGS:
        w = widths[key] * scale
        o.append(f'<g transform="translate({x + w / 2},{ground}) scale({scale})">{fn()}</g>')
        x += w + 26
    o.append(f'<g transform="translate({x + 10},{ground}) scale({0.36 * scale})">{elora()}</g>')
    if pale:
        o.append(f'<rect x="0" y="{oy}" width="{W}" height="420" fill="#d9dde2" opacity="0.5"/>')
        o.append(f'<g transform="translate({x + 10},{ground}) scale({0.36 * scale})">{elora()}</g>')
    return '\n'.join(o)


def sheet():
    W, H = 1600, 1480
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
         '<defs><linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#a9cde8"/>'
         '<stop offset="1" stop-color="#eef4f8"/></linearGradient></defs>',
         f'<rect width="{W}" height="{H}" fill="#e8eef3"/>',
         text(40, 46, 'Tauwinkel – Gebäude (A1.9, E-274)', 28, anchor='start'),
         text(40, 74, 'Häuser mit Tropfendach in der Farbe ihrer Bewohner; Stil A wie die Kartenteile. Maßstab: Elora rechts. '
              'Ursprung unten Mitte, später direkt als Deko.', 15, anchor='start')]
    # Einzelansichten mit Beschriftung
    cols = 3
    cw, ch = 500, 300
    for i, (key, name, note, fn) in enumerate(BUILDINGS):
        cx = 40 + (i % cols) * (cw + 10)
        cy = 100 + (i // cols) * (ch + 10)
        o.append(f'<rect x="{cx}" y="{cy}" width="{cw}" height="{ch}" rx="16" fill="#fffaf0" stroke="#e7dcc8"/>')
        gy = cy + ch - 56
        o.append(f'<rect x="{cx + 20}" y="{gy}" width="{cw - 40}" height="8" rx="4" fill="#8fbf7a"/>')
        s = 0.62 if key == 'baumhaus' else 0.9
        o.append(f'<g transform="translate({cx + cw / 2 - 30},{gy}) scale({s})">{fn()}</g>')
        o.append(f'<g transform="translate({cx + cw - 50},{gy}) scale({0.36 * s})">{elora()}</g>')
        o.append(text(cx + 20, cy + 30, name, 17, anchor='start'))
        o.append(text(cx + 20, cy + ch - 20, note, 12, '#8a7a66', anchor='start'))
    o.append(text(40, 1068, 'Im Dorf: bunt (nach allen Quellen) und blass (zu Beginn, E-275 – Figuren bleiben bunt)', 17, anchor='start'))
    o.append(scene(1080, False).replace('url(#sky)', 'url(#sky)'))
    o.append('</svg>')
    s = '\n'.join(o)
    # zweite Reihe blass unter die erste – Höhe reicht nicht, also Seite verlängern
    return s


def sheet_full():
    s = sheet()
    s = s.replace('height="1480" viewBox="0 0 1600 1480"', 'height="1940" viewBox="0 0 1600 1940"')
    s = s.replace('<rect width="1600" height="1480"', '<rect width="1600" height="1940"')
    return s.replace('</svg>', scene(1510, True) + '\n</svg>')


if __name__ == '__main__':
    os.makedirs('docs/release-2/design', exist_ok=True)
    open('docs/release-2/design/tauwinkel-gebaeude.svg', 'w').write(sheet_full())

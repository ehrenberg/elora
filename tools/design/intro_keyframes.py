"""Generates the key frames and reference images for the adventure intro video (E-355, E-357):
start/end frames of the two 30-second clips and reference sheets for Elora, Oma Pfütze and the
style of Tauwinkel. Everything is composed from the game's own SVG assets.

Usage: python3 tools/design/intro_keyframes.py
       then for f in docs/release-2/design/intro/*.svg; do
              cargo xtask svg-preview "$f" "${f%.svg}.png" 1920; done
"""

import os
import re

ROOT = os.path.join(os.path.dirname(__file__), '..', '..')
OUT = os.path.join(ROOT, 'docs', 'release-2', 'design', 'intro')
W, H = 1920, 1080


def inner(path):
    """Content of an asset SVG without the outer <svg> element."""
    with open(os.path.join(ROOT, path), encoding='utf-8') as f:
        s = f.read()
    s = re.sub(r'<\?xml[^>]*>', '', s)
    s = re.sub(r'<svg[^>]*>', '', s, count=1)
    return s.rsplit('</svg>', 1)[0]


def place(path, x, y, scale=1.0, flip=False, grey=False):
    """Asset with its origin (bottom centre) at x, y."""
    sx = -scale if flip else scale
    filt = ' filter="url(#grey)"' if grey else ''
    return f'<g transform="translate({x},{y}) scale({sx},{scale})"{filt}>{inner(path)}</g>'


def band(path, y, scale, opacity=1.0, grey=False):
    """Background band (1024 wide, origin bottom left) repeated across the width."""
    out = ''
    step = 1024 * scale
    x = -step * 0.3
    while x < W:
        filt = ' filter="url(#grey)"' if grey else ''
        out += (f'<g transform="translate({x:.1f},{y}) scale({scale})" opacity="{opacity}"{filt}>'
                f'{inner(path)}</g>')
        x += step
    return out


def defs(sky_top, sky_bottom):
    return f'''<defs>
  <linearGradient id="sky" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0" stop-color="{sky_top}"/><stop offset="1" stop-color="{sky_bottom}"/>
  </linearGradient>
  <radialGradient id="glow"><stop offset="0" stop-color="#ffffff" stop-opacity="0.9"/>
    <stop offset="1" stop-color="#ffffff" stop-opacity="0"/></radialGradient>
  <filter id="grey"><feColorMatrix type="saturate" values="0.12"/></filter>
</defs>'''


def frame(body, sky_top, sky_bottom):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">\n'
            f'{defs(sky_top, sky_bottom)}\n<rect width="{W}" height="{H}" fill="url(#sky)"/>\n{body}\n</svg>\n')


def spring_glow(x, y, color, r=90):
    return (f'<circle cx="{x}" cy="{y}" r="{r}" fill="{color}" opacity="0.35"/>'
            f'<circle cx="{x}" cy="{y}" r="{r * 0.45}" fill="url(#glow)"/>')


def landscape(grey):
    """Panorama of the Tauland: mountains, hills, five springs, Tauwinkel in the hollow."""
    s = ''
    if not grey:
        s += '<circle cx="1540" cy="190" r="95" fill="#ffd36b"/>'
        s += '<circle cx="1540" cy="190" r="170" fill="#ffe9a8" opacity="0.35"/>'
    s += band('assets/map/backgrounds/mountains.svg', 640, 1.1, 1.0, grey)
    s += f'<rect x="0" y="0" width="{W}" height="640" fill="#ffffff" opacity="0.12"/>'
    s += band('assets/map/backgrounds/hills-far.svg', 760, 1.3, 1.0, grey)
    s += band('assets/map/backgrounds/forest.svg', 820, 0.9, 1.0, grey)
    # the five springs: meadow, forest, desert, frost; the star spring as a violet glow
    springs = [
        ('assets/map/decor/bluetenquelle', 300, 760, '#7fd99a'),
        ('assets/map/decor/waldquelle', 640, 700, '#a8744a'),
        ('assets/map/decor/glutquelle', 1280, 720, '#f2c14e'),
        ('assets/map/decor/frostquelle', 1650, 600, '#9fd8f0'),
    ]
    for base, x, y, color in springs:
        state = 'verdorrt' if grey else 'befreit'
        if not grey:
            s += spring_glow(x, y - 60, color)
        s += place(f'{base}-{state}.svg', x, y, 0.55)
    if not grey:
        s += spring_glow(1020, 420, '#a77be0', 70)
    ground = '#8fb37f' if grey else '#7fbf6a'
    s += f'<rect x="0" y="812" width="{W}" height="{H - 812}" fill="{ground}"' + (
        ' filter="url(#grey)"' if grey else '') + '/>'
    s += band('assets/map/backgrounds/hills-near.svg', 1080, 1.6, 1.0, grey)
    # Tauwinkel in the hollow
    village = [
        ('assets/map/decor/tree-round.svg', 600, 0.85),
        ('assets/map/decor/haus-oma.svg', 700, 0.62),
        ('assets/map/decor/werkstatt.svg', 840, 0.52),
        ('assets/map/decor/brunnen.svg', 960, 0.7),
        ('assets/map/decor/haus-elora.svg', 1090, 0.5),
        ('assets/map/decor/schmiede.svg', 1240, 0.52),
        ('assets/map/decor/tree-round.svg', 1350, 0.8),
    ]
    for path, x, sc in village:
        s += place(path, x, 1010, sc, grey=grey)
    return s


def square(grey):
    """The well square of Tauwinkel with Oma Pfütze and Elora."""
    s = band('assets/map/backgrounds/hills-far.svg', 700, 1.4, 1.0, grey)
    s += band('assets/map/backgrounds/hills-near.svg', 860, 1.5, 1.0, grey)
    s += f'<rect x="0" y="900" width="{W}" height="{H - 900}" fill="#8fbf7a"' + (
        ' filter="url(#grey)"' if grey else '') + '/>'
    s += '<rect x="0" y="900" width="1920" height="10" fill="#6a9a58" opacity="0.6"/>'
    s += place('assets/map/decor/haus-oma.svg', 330, 910, 1.4, grey=grey)
    s += place('assets/map/decor/tree-round.svg', 70, 910, 1.4, grey=grey)
    s += place('assets/map/decor/brunnen.svg', 980, 910, 1.5, grey=grey)
    s += place('assets/map/decor/werkstatt.svg', 1640, 910, 1.1, grey=grey)
    s += place('assets/map/decor/blumenkasten-blass.svg', 1400, 910, 1.2, grey=grey)
    # characters keep their colours: they are the ones who still care
    s += place('assets/adventure/characters/oma.svg', 700, 912, 3.0)
    s += place('assets/elora/elora.svg', 1250, 912, 1.15)
    return s


def sunrise():
    """Elora on a hilltop at sunrise, the colourful hills ahead (end of the intro)."""
    s = '<circle cx="1500" cy="250" r="130" fill="#ffd36b"/>'
    s += '<circle cx="1500" cy="250" r="240" fill="#ffe9a8" opacity="0.35"/>'
    s += band('assets/map/backgrounds/mountains.svg', 700, 1.1)
    s += f'<rect x="0" y="0" width="{W}" height="700" fill="#ffffff" opacity="0.12"/>'
    s += band('assets/map/backgrounds/hills-far.svg', 820, 1.3)
    s += spring_glow(1180, 700, '#7fd99a', 70)
    s += f'<rect x="0" y="815" width="{W}" height="{H - 815}" fill="#7fbf6a"/>'
    s += band('assets/map/backgrounds/hills-near.svg', 1080, 1.8)
    s += place('assets/map/decor/riesenblume-gelb.svg', 260, 1000, 1.0)
    s += place('assets/map/decor/riesenblume-rosa.svg', 1700, 1020, 0.9)
    s += place('assets/elora/elora.svg', 640, 905, 0.9)
    return s


def sheet(items, title_color='#2b2b2b'):
    """White reference sheet with assets side by side."""
    body = f'<rect width="{W}" height="{H}" fill="#ffffff"/>'
    for path, x, y, sc, flip in items:
        body += place(path, x, y, sc, flip)
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">'
            f'{body}</svg>\n')


def main():
    os.makedirs(OUT, exist_ok=True)
    files = {
        'clip1-start.svg': frame(landscape(False), '#8fc8f0', '#ffe2b8'),
        'clip1-end.svg': frame(landscape(True), '#a9b2ba', '#d8d6d2'),
        'clip2-start.svg': frame(square(True), '#aab3bb', '#dcdad6'),
        'clip2-end.svg': frame(sunrise(), '#7fb8e8', '#ffd9a0'),
        'ref-elora.svg': sheet([
            ('assets/elora/elora.svg', 480, 900, 4.2, False),
            ('assets/elora/elora.svg', 1440, 900, 4.2, True),
        ]),
        'ref-oma.svg': sheet([('assets/adventure/characters/oma.svg', 960, 950, 9.0, False)]),
        'ref-tauwinkel.svg': sheet([
            ('assets/map/decor/haus-oma.svg', 260, 700, 1.4, False),
            ('assets/map/decor/brunnen.svg', 720, 700, 1.6, False),
            ('assets/map/decor/werkstatt.svg', 1180, 700, 1.1, False),
            ('assets/map/decor/schmiede.svg', 1660, 700, 1.1, False),
            ('assets/map/decor/tree-round.svg', 300, 1060, 1.5, False),
            ('assets/map/decor/bluetenquelle-befreit.svg', 960, 1060, 1.3, False),
            ('assets/map/decor/riesenblume-rosa.svg', 1600, 1060, 1.2, False),
        ]),
    }
    for name, svg in files.items():
        with open(os.path.join(OUT, name), 'w', encoding='utf-8') as f:
            f.write(svg)
    print(f'{len(files)} files written to {os.path.relpath(OUT, ROOT)}')


if __name__ == '__main__':
    main()

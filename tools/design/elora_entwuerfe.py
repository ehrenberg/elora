"""Erzeugt docs/archive/release-1/design/elora-entwuerfe.svg – drei Tropfen-Entwürfe von Elora (E-085).

Aufruf: python3 tools/design/elora_entwuerfe.py && cargo xtask svg-preview \
        docs/archive/release-1/design/elora-entwuerfe.svg docs/archive/release-1/design/elora-entwuerfe.png 1260

Maßstab: Figur lokal ~100 breit → Spielgröße 36 Einheiten (E-087), Faktor 0.36.
Die Füße (lokal y = 67) stehen auf der Hitbox-Unterkante (+14 Einheiten).
"""

OUT = '#2b2b2b'; BODY = '#f2c14e'; FEET = '#d9a43a'; LIGHT = '#fbe3a1'
GAME_SCALE = 0.36
FEET_BOTTOM = 67


def drop(asym=0.0):
    tx = asym * 22
    return (f"M {tx:.1f},-64 C {18 + tx * 0.4:.1f},-40 50,-18 50,12 C 50,40 28,58 0,58 "
            f"C -28,58 -50,40 -50,12 C -50,-18 {-18 + tx * 0.6:.1f},-40 {tx:.1f},-64 Z")


def figure(kind, sx=1.0, sy=1.0):
    g = [f'<g transform="scale({sx},{sy})">']
    if kind == 'A':
        for x in (-22, 22):
            g.append(f'<ellipse cx="{x}" cy="60" rx="17" ry="9" fill="{FEET}" stroke="{OUT}" stroke-width="4"/>')
        g.append(f'<path d="{drop()}" fill="{BODY}" stroke="{OUT}" stroke-width="5" stroke-linejoin="round"/>')
        g.append(f'<path d="M -30,-8 C -30,-26 -18,-38 -8,-46" fill="none" stroke="{LIGHT}" stroke-width="7" stroke-linecap="round"/>')
        for x in (-2, 24):
            g.append(f'<ellipse cx="{x}" cy="8" rx="10" ry="14" fill="#ffffff" stroke="{OUT}" stroke-width="3"/>')
            g.append(f'<ellipse cx="{x + 4}" cy="10" rx="5" ry="7" fill="{OUT}"/>')
        g.append('<ellipse cx="-24" cy="30" rx="7" ry="4" fill="#f29a6b" opacity="0.7"/>')
    elif kind == 'B':
        for x in (-20, 22):
            g.append(f'<ellipse cx="{x}" cy="59" rx="14" ry="8" fill="{FEET}" stroke="{OUT}" stroke-width="4"/>')
        g.append(f'<path d="{drop(0.9)}" fill="{BODY}" stroke="{OUT}" stroke-width="5" stroke-linejoin="round"/>')
        g.append(f'<path d="M -34,26 C -30,46 -12,52 4,52 C 22,52 36,42 40,26 C 26,34 -18,36 -34,26 Z" fill="{LIGHT}"/>')
        for x in (2, 26):
            g.append(f'<ellipse cx="{x}" cy="4" rx="6.5" ry="11" fill="{OUT}"/>')
            g.append(f'<circle cx="{x + 2}" cy="-1" r="2.6" fill="#ffffff"/>')
        g.append(f'<path d="M 10,26 Q 16,31 22,26" fill="none" stroke="{OUT}" stroke-width="3" stroke-linecap="round"/>')
    else:
        for x in (-36, 8):
            g.append(f'<rect x="{x}" y="50" width="30" height="14" rx="7" fill="{FEET}" stroke="{OUT}" stroke-width="4"/>')
        g.append(f'<path d="{drop()}" fill="{BODY}" stroke="{OUT}" stroke-width="5" stroke-linejoin="round"/>')
        g.append(f'<rect x="-30" y="-6" width="74" height="26" rx="13" fill="#34495e" stroke="{OUT}" stroke-width="3"/>')
        for x in (0, 26):
            g.append(f'<ellipse cx="{x}" cy="7" rx="7" ry="6" fill="#7df9ff"/>')
            g.append(f'<ellipse cx="{x}" cy="7" rx="3.5" ry="3" fill="#ffffff"/>')
    g.append('</g>')
    return "\n".join(g)


NAMES = {'A': ('A – Klassisch', 'große Augen, runde Füße, Glanz'),
         'B': ('B – Wirbel', 'Spitze zur Seite, Bauchfleck (Muster)'),
         'C': ('C – Visier', 'Visier-Band mit leuchtenden Augen')}


def sheet():
    W, H = 1260, 640
    svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
           f'<rect width="{W}" height="{H}" fill="#8fb8d9"/>',
           '<text x="30" y="44" font-size="28" fill="#1e2a36" font-weight="bold">Elora – drei Entwürfe (Tropfenform, E-085)</text>']
    for k, kind in enumerate('ABC'):
        cx = 210 + k * 420
        title, desc = NAMES[kind]
        svg.append(f'<rect x="{cx - 195}" y="70" width="390" height="530" rx="16" fill="#ffffff" opacity="0.35"/>')
        svg.append(f'<text x="{cx}" y="106" font-size="24" text-anchor="middle" fill="#1e2a36" font-weight="bold">{title}</text>')
        svg.append(f'<text x="{cx}" y="132" font-size="16" text-anchor="middle" fill="#1e2a36">{desc}</text>')
        svg.append(f'<g transform="translate({cx},260) scale(1.45)">{figure(kind)}</g>')
        svg.append(f'<g transform="translate({cx - 125},470) scale(0.62)">{figure(kind, 0.82, 1.2)}</g>')
        svg.append(f'<text x="{cx - 125}" y="560" font-size="14" text-anchor="middle" fill="#1e2a36">Sprung</text>')
        svg.append(f'<g transform="translate({cx - 20},480) scale(0.62)">{figure(kind, 1.22, 0.78)}</g>')
        svg.append(f'<text x="{cx - 20}" y="560" font-size="14" text-anchor="middle" fill="#1e2a36">Landung</text>')
        # Spielgröße ×2: Hitbox (Mitte gx,gy), Boden-Tile ab Hitbox-Unterkante
        z = 2.0
        gx, gy = cx + 115, 478
        floor = gy + 14 * z
        svg.append(f'<rect x="{gx - 32 * z / 2}" y="{floor}" width="{32 * z}" height="{12 * z}" fill="#5b6b7c"/>')
        fy = floor - FEET_BOTTOM * GAME_SCALE * z
        svg.append(f'<g transform="translate({gx},{fy}) scale({GAME_SCALE * z})">{figure(kind)}</g>')
        svg.append(f'<rect x="{gx - 14 * z}" y="{gy - 14 * z}" width="{28 * z}" height="{28 * z}" fill="none" stroke="#d9534f" stroke-width="2" stroke-dasharray="5,4"/>')
        svg.append(f'<text x="{gx}" y="560" font-size="14" text-anchor="middle" fill="#1e2a36">im Spiel ×2</text>')
    svg.append('<text x="30" y="626" font-size="13" fill="#1e2a36">Rot gestrichelt: Hitbox 28 × 28 · grau: Boden (1 Tile = 32) · Farben sind Platzhalter für die Skin-Farben (E-086)</text>')
    svg.append('</svg>')
    return "\n".join(svg)


if __name__ == '__main__':
    with open('docs/archive/release-1/design/elora-entwuerfe.svg', 'w') as f:
        f.write(sheet())

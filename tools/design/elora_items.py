"""Erzeugt docs/archive/release-1/design/elora-items.svg – Entwürfe für Pickups, Waffen, Flaggen (M5.5).

Aufruf: python3 tools/design/elora_items.py && cargo xtask svg-preview \
        docs/archive/release-1/design/elora-items.svg docs/archive/release-1/design/elora-items.png 1260

Alle Motive sind in Welteinheiten gezeichnet und werden auf dem Blatt ×2.5 gezeigt.
"""
import re

OUT = '#2b2b2b'
Z = 2.5


def elora():
    src = open('assets/elora/elora.svg').read()
    inner = src[src.index('>', src.index('<svg')) + 1:src.rindex('</svg>')]
    return re.sub(r'<!--.*?-->', '', inner, flags=re.S)


def g(x, y, body, scale=Z):
    return f'<g transform="translate({x},{y}) scale({scale})">{body}</g>'


# --- Stil A: rund, freundlich (passend zu Elora) ---
A = {
    'Leben': f'''<path d="M 0,9 C -13,1 -13,-10 -5.5,-10 C -2.5,-10 0,-7.5 0,-5 C 0,-7.5 2.5,-10 5.5,-10 C 13,-10 13,1 0,9 Z"
        fill="#e05a7a" stroke="{OUT}" stroke-width="1.6" stroke-linejoin="round"/>
        <ellipse cx="-5" cy="-5.5" rx="2.6" ry="1.8" fill="#ffffff" opacity="0.8"/>''',
    'Rüstung': f'''<path d="M -9,-10 L 9,-10 L 9,-1 C 9,5 4,9 0,11 C -4,9 -9,5 -9,-1 Z" fill="#e0b85a" stroke="{OUT}" stroke-width="1.6" stroke-linejoin="round"/>
        <path d="M -5,-6.5 L 5,-6.5 L 5,-1 C 5,3 2.5,5.5 0,7 C -2.5,5.5 -5,3 -5,-1 Z" fill="#f3d990"/>''',
    'Hammer': f'''<rect x="-2" y="-2" width="26" height="4" rx="2" fill="#8a6a4a" stroke="{OUT}" stroke-width="1.2"/>
        <rect x="22" y="-9" width="12" height="18" rx="4" fill="#c8ced6" stroke="{OUT}" stroke-width="1.6"/>
        <rect x="24" y="-7" width="3" height="14" rx="1.5" fill="#eef1f4"/>''',
    'Granate': f'''<rect x="0" y="-5.5" width="30" height="11" rx="5.5" fill="#6fbf4f" stroke="{OUT}" stroke-width="1.6"/>
        <rect x="27" y="-3.5" width="11" height="7" rx="2" fill="#3f5a36" stroke="{OUT}" stroke-width="1.2"/>
        <circle cx="9" cy="3" r="5" fill="#4f9a38" stroke="{OUT}" stroke-width="1.4"/>
        <rect x="4" y="-3.5" width="18" height="2" rx="1" fill="#a6dc8c"/>''',
    'Laser': f'''<rect x="0" y="-4" width="34" height="8" rx="4" fill="#e8f4f8" stroke="{OUT}" stroke-width="1.6"/>
        <rect x="4" y="-1.2" width="24" height="2.4" rx="1.2" fill="#5ad0e0"/>
        <circle cx="37" cy="0" r="3.2" fill="#5ad0e0" stroke="{OUT}" stroke-width="1.2"/>
        <circle cx="37" cy="0" r="1.4" fill="#ffffff"/>''',
}


def flag_a(color):
    return f'''<line x1="0" y1="14" x2="0" y2="-38" stroke="{OUT}" stroke-width="3" stroke-linecap="round"/>
        <circle cx="0" cy="-39" r="2.6" fill="#e8e8e8" stroke="{OUT}" stroke-width="1.2"/>
        <path d="M 1.5,-36 C 12,-40 20,-30 32,-30 C 24,-24 12,-20 1.5,-16 Z" fill="{color}" stroke="{OUT}" stroke-width="1.6" stroke-linejoin="round"/>'''


# --- Stil B: kantig, technisch (Kontrast zur runden Figur) ---
B = {
    'Leben': f'''<rect x="-11" y="-11" width="22" height="22" rx="5" fill="#ffffff" stroke="{OUT}" stroke-width="1.6"/>
        <path d="M -3,-7 H 3 V -3 H 7 V 3 H 3 V 7 H -3 V 3 H -7 V -3 H -3 Z" fill="#e05a7a"/>''',
    'Rüstung': f'''<path d="M 0,-12 L 10,-6 L 10,6 L 0,12 L -10,6 L -10,-6 Z" fill="#e0b85a" stroke="{OUT}" stroke-width="1.6" stroke-linejoin="round"/>
        <path d="M 0,-12 L 0,0 L 10,-6 Z" fill="#f3d990"/><path d="M 0,0 L -10,6 L 0,12 Z" fill="#b8913c"/>''',
    'Hammer': f'''<rect x="-2" y="-2" width="26" height="4" fill="#8a6a4a" stroke="{OUT}" stroke-width="1.2"/>
        <path d="M 21,-10 L 32,-8 L 35,0 L 32,8 L 21,10 Z" fill="#c8ced6" stroke="{OUT}" stroke-width="1.6" stroke-linejoin="round"/>''',
    'Granate': f'''<path d="M 0,-6 L 26,-6 L 30,-3 L 38,-3 L 38,3 L 30,3 L 26,6 L 0,6 Z" fill="#6fbf4f" stroke="{OUT}" stroke-width="1.6" stroke-linejoin="round"/>
        <path d="M 6,6 L 14,6 L 12,11 L 7,11 Z" fill="#4f9a38" stroke="{OUT}" stroke-width="1.2" stroke-linejoin="round"/>''',
    'Laser': f'''<path d="M 0,-4 L 30,-4 L 38,0 L 30,4 L 0,4 Z" fill="#e8f4f8" stroke="{OUT}" stroke-width="1.6" stroke-linejoin="round"/>
        <path d="M 8,-4 L 14,-9 L 18,-9 L 16,-4 Z" fill="#5ad0e0" stroke="{OUT}" stroke-width="1.2" stroke-linejoin="round"/>
        <line x1="3" y1="0" x2="30" y2="0" stroke="#5ad0e0" stroke-width="2"/>''',
}


def flag_b(color):
    return f'''<line x1="0" y1="14" x2="0" y2="-38" stroke="{OUT}" stroke-width="3" stroke-linecap="round"/>
        <path d="M 1.5,-37 H 30 L 24,-28 L 30,-19 H 1.5 Z" fill="{color}" stroke="{OUT}" stroke-width="1.6" stroke-linejoin="round"/>
        <circle cx="12" cy="-28" r="4" fill="#ffffff" opacity="0.85"/>'''


def row(y, title, desc, items, flag):
    out = [f'<rect x="15" y="{y}" width="1230" height="360" rx="16" fill="#ffffff" opacity="0.35"/>',
           f'<text x="35" y="{y + 36}" font-size="24" fill="#1e2a36" font-weight="bold">{title}</text>',
           f'<text x="35" y="{y + 62}" font-size="16" fill="#1e2a36">{desc}</text>']
    base = y + 200
    xs = {'Leben': 90, 'Rüstung': 210, 'Hammer': 320, 'Granate': 480, 'Laser': 660}
    for name, x in xs.items():
        body = items[name]
        out.append(g(x, base, body))
        cx = x if name in ('Leben', 'Rüstung') else x + 45
        out.append(f'<text x="{cx}" y="{base + 70}" font-size="15" text-anchor="middle" fill="#1e2a36">{name}</text>')
    out.append(g(880, base + 20, flag('#e0574f')))
    out.append(g(990, base + 20, flag('#4f86e0')))
    out.append(f'<text x="955" y="{base + 70}" font-size="15" text-anchor="middle" fill="#1e2a36">Flaggen</text>')
    # Größenvergleich: Elora mit Laser
    ground = base + 14 * Z + 30
    out.append(g(1120, ground, elora(), 0.36 * Z))
    out.append(g(1120 + 2 * Z, ground - 12 * Z, items['Laser'], Z))
    out.append(f'<line x1="1050" y1="{ground}" x2="1230" y2="{ground}" stroke="#5b6b7c" stroke-width="6"/>')
    out.append(f'<text x="1140" y="{base + 100}" font-size="13" text-anchor="middle" fill="#1e2a36">Größenvergleich</text>')
    return out


def sheet():
    W, H = 1260, 860
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
           f'<rect width="{W}" height="{H}" fill="#8fb8d9"/>',
           '<text x="30" y="44" font-size="28" fill="#1e2a36" font-weight="bold">Pickups, Waffen, Flaggen – zwei Stile (M5.5)</text>',
           '<text x="30" y="70" font-size="15" fill="#1e2a36">Gezeigt ×2.5 der Spielgröße. Waffen-Pickups zeigen dieselbe Grafik wie die Waffe in der Hand.</text>']
    out += row(90, 'A – Rund', 'weiche Formen und Glanzlichter, passend zu Elora', A, flag_a)
    out += row(475, 'B – Kantig', 'klare Kanten und Facetten, Kontrast zur runden Figur', B, flag_b)
    out.append('</svg>')
    return '\n'.join(out)


if __name__ == '__main__':
    with open('docs/archive/release-1/design/elora-items.svg', 'w') as fh:
        fh.write(sheet())

"""Erzeugt docs/archiv/release-1/design/elora-palette.svg – Paletten-Entwurf für Skins (E-095, E-096).

Aufruf: python3 tools/design/elora_palette.py && cargo xtask svg-preview \
        docs/archiv/release-1/design/elora-palette.svg docs/archiv/release-1/design/elora-palette.png 1260
"""
import re

BODY = [  # Körper und Füße (16)
    ('Sonne', 'f2c14e'), ('Orange', 'f28c3a'), ('Koralle', 'e8685a'), ('Rot', 'd94a4a'),
    ('Rosa', 'ef7fb0'), ('Violett', 'a77be0'), ('Indigo', '6a78e0'), ('Himmel', '5aaee8'),
    ('Türkis', '3fc1b0'), ('Mint', '7fd99a'), ('Grün', '6cbf4a'), ('Limette', 'b8d94a'),
    ('Sand', 'e0c89a'), ('Braun', 'a8744a'), ('Grau', '9aa4ae'), ('Weiß', 'f0ece4'),
]
EYES = [  # Augen (8)
    ('Schwarz', '2b2b2b'), ('Nachtblau', '2e3f86'), ('Tannengrün', '2f6b4a'), ('Kastanie', '6b3a2a'),
    ('Wein', '7a2a4a'), ('Pflaume', '4a2a7a'), ('Petrol', '1f6470'), ('Schiefer', '5a5a5a'),
]
EXAMPLES = [  # (Körper, Füße, Augen) als Indizes
    (0, 0, 0), (7, 6, 1), (4, 5, 4), (10, 13, 2), (15, 14, 0), (3, 13, 3), (8, 9, 6), (5, 6, 5),
]


def lighten(hex_, amount):
    c = [int(hex_[i:i + 2], 16) for i in (0, 2, 4)]
    return ''.join(f'{round(v + (255 - v) * amount):02x}' for v in c)


def elora(body, feet, eyes):
    src = open('assets/elora/elora.svg').read()
    inner = src[src.index('>', src.index('<svg')) + 1:src.rindex('</svg>')]
    inner = re.sub(r'<!--.*?-->', '', inner, flags=re.S)

    def recolor(m):
        tag = m.group(0)
        key = re.search(r'id="tint-(\d)(-l(\d+))?', tag)
        if not key:
            return tag
        slot, light = key.group(1), key.group(3)
        color = {'1': eyes, '2': body, '3': feet}[slot]
        if light:
            color = lighten(color, int(light) / 100)
        return re.sub(r'fill="#\w+"', f'fill="#{color}"', tag)

    return re.sub(r'<(ellipse|path|circle)[^>]*>', recolor, inner)


def sheet():
    W, H = 1260, 700
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
           f'<rect width="{W}" height="{H}" fill="#8fb8d9"/>',
           '<text x="30" y="44" font-size="28" fill="#1e2a36" font-weight="bold">Elora – Paletten-Entwurf (E-095, E-096)</text>',
           '<text x="30" y="72" font-size="16" fill="#1e2a36">Körper und Füße wählen je eine der 16 Farben, Augen eine der 8. Bauchfleck = Körper +45 % heller (E-097).</text>']
    out.append('<text x="30" y="112" font-size="18" fill="#1e2a36" font-weight="bold">Körper / Füße (16)</text>')
    for i, (name, c) in enumerate(BODY):
        x, y = 30 + (i % 8) * 150, 126 + (i // 8) * 78
        out.append(f'<rect x="{x}" y="{y}" width="56" height="44" rx="10" fill="#{c}" stroke="#2b2b2b" stroke-width="3"/>')
        out.append(f'<text x="{x + 64}" y="{y + 20}" font-size="15" fill="#1e2a36">{i + 1:2d} {name}</text>')
        out.append(f'<text x="{x + 64}" y="{y + 38}" font-size="12" fill="#1e2a36">#{c}</text>')
    out.append('<text x="30" y="306" font-size="18" fill="#1e2a36" font-weight="bold">Augen (8)</text>')
    for i, (name, c) in enumerate(EYES):
        x, y = 30 + i * 150, 320
        out.append(f'<ellipse cx="{x + 20}" cy="{y + 22}" rx="12" ry="20" fill="#{c}"/>')
        out.append(f'<circle cx="{x + 24}" cy="{y + 13}" r="4.5" fill="#ffffff"/>')
        out.append(f'<text x="{x + 44}" y="{y + 20}" font-size="15" fill="#1e2a36">{i + 1} {name}</text>')
        out.append(f'<text x="{x + 44}" y="{y + 38}" font-size="12" fill="#1e2a36">#{c}</text>')
    out.append('<text x="30" y="420" font-size="18" fill="#1e2a36" font-weight="bold">Beispiele (Körper · Füße · Augen)</text>')
    for i, (b, f, e) in enumerate(EXAMPLES):
        cx = 95 + i * 150
        out.append(f'<g transform="translate({cx},640) scale(1.3)">{elora(BODY[b][1], BODY[f][1], EYES[e][1])}</g>')
        out.append(f'<text x="{cx}" y="676" font-size="13" text-anchor="middle" fill="#1e2a36">{b + 1} · {f + 1} · {e + 1}</text>')
    out.append('</svg>')
    return '\n'.join(out)


if __name__ == '__main__':
    with open('docs/archiv/release-1/design/elora-palette.svg', 'w') as fh:
        fh.write(sheet())

"""Erzeugt die Spielgrafiken der Abenteuer-Gegner und der Beute aus den angenommenen
Entwürfen (E-225) nach assets/adventure/.

Aufruf: python3 tools/design/abenteuer_assets.py

Welteinheiten wie assets/elora/elora.svg: Ursprung in der Mitte der Kollisionsbox
(siehe assets/adventure/creatures.toml), Blick nach rechts.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
import re  # noqa: E402

from abenteuer_figuren import (glanztropfen, glitzerstein, grashuepfer, klonk, lotte, oma_pfuetze, pip,  # noqa: E402
                               pollenblaeser, quellstein, schalter, stachelkaefer, truhe, tueftel)

# Name: (Boxhöhe, Maßstab, {Teil: Zeichnung})
CREATURES = {
    'stachelkaefer': (26, 0.36, {'idle': stachelkaefer()}),
    'pollenblaeser': (60, 0.42, {'idle': pollenblaeser(shots=False)}),
    'grashuepfer': (28, 0.36, {'idle': grashuepfer(), 'air': grashuepfer(True, arc=False)}),
}


# NPCs: Ursprung am Boden zwischen den Füßen, Maßstab wie Elora (0,36) × Größe aus dem Entwurf
CHARACTERS = {
    'oma': (0.95, oma_pfuetze()),
    'klonk': (1.2, klonk()),
    'lotte': (1.0, lotte()),
    'tueftel': (1.0, tueftel()),
    'pip': (0.7, pip()),
}


def heilpflanze():
    """Heilpflanze (E-258): Blüte mit Herz, freundlich – Gegenstück zum Pollenbläser."""
    s = '<path d="M 0,0 Q -4,-30 0,-56" fill="none" stroke="#2b2b2b" stroke-width="9"/>'
    s += '<path d="M 0,0 Q -4,-30 0,-56" fill="none" stroke="#4f9a3a" stroke-width="4.5"/>'
    s += '<path d="M -2,-22 Q -30,-34 -32,-14 Q -16,-8 -2,-22 Z" fill="#6cbf4a" stroke="#2b2b2b" stroke-width="3"/>'
    s += '<path d="M -1,-34 Q 26,-48 30,-28 Q 14,-22 -1,-34 Z" fill="#6cbf4a" stroke="#2b2b2b" stroke-width="3"/>'
    import math
    for i in range(6):
        a = math.radians(i * 60)
        x, y = 18 * math.cos(a), -74 + 18 * math.sin(a)
        s += f'<circle cx="{x:.1f}" cy="{y:.1f}" r="12" fill="#ef7fb0" stroke="#2b2b2b" stroke-width="3"/>'
    s += '<circle cx="0" cy="-74" r="15" fill="#f8dd9e" stroke="#2b2b2b" stroke-width="3.5"/>'
    s += '<path d="M 0,-66 C -10,-74 -10,-84 -4,-84 C -2,-84 0,-82 0,-80 C 0,-82 2,-84 4,-84 C 10,-84 10,-74 0,-66 Z" fill="#e8685a"/>'
    return s


def welk():
    """Verbrauchte Heilpflanze: kleiner, blass."""
    s = '<path d="M 0,0 Q -6,-18 2,-30" fill="none" stroke="#2b2b2b" stroke-width="8"/>'
    s += '<path d="M 0,0 Q -6,-18 2,-30" fill="none" stroke="#7f9a6a" stroke-width="4"/>'
    s += '<circle cx="4" cy="-34" r="9" fill="#d8ccc4" stroke="#2b2b2b" stroke-width="3"/>'
    return s


# Objekte: Ursprung am Boden, (Maßstab, {Teil: Zeichnung})
OBJECTS = {
    'truhe': (0.45, {'closed': truhe(), 'open': truhe(True)}),
    'quellstein': (0.45, {'off': quellstein(False), 'on': quellstein(True)}),
    'schalter': (0.5, {'off': schalter(False), 'on': schalter(True)}),
    'heilpflanze': (0.3, {'fresh': heilpflanze(), 'used': welk()}),
}


def plain(art):
    """Farbschlüssel-Ids aus Eloras Vorlage entfernen (feste Farben im Spiel)."""
    return re.sub(r' id="tint-[^"]*"', '', art)


def svg(parts, comment):
    body = ''.join(f'<g id="{k}">{v}</g>' for k, v in parts.items())
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="-48 -48 96 96">\n'
            f'  <!-- {comment} -->\n  {body}\n</svg>\n')


def main():
    out = 'assets/adventure/creatures'
    os.makedirs(out, exist_ok=True)
    for name, (h, scale, parts) in CREATURES.items():
        placed = {k: f'<g transform="translate(0,{h / 2}) scale({scale})">{v}</g>' for k, v in parts.items()}
        with open(f'{out}/{name}.svg', 'w') as f:
            f.write(svg(placed, f'{name} (A1.2, aus tools/design/abenteuer_assets.py). Ursprung = Boxmitte, Blick nach rechts.'))
    for folder, table in (('characters', {k: (v[0] * 0.36, {'figure': v[1]}) for k, v in CHARACTERS.items()}),
                          ('objects', OBJECTS)):
        out = f'assets/adventure/{folder}'
        os.makedirs(out, exist_ok=True)
        for name, (scale, parts) in table.items():
            placed = {k: f'<g transform="scale({scale})">{plain(v)}</g>' for k, v in parts.items()}
            body = ''.join(f'<g id="{k}">{v}</g>' if k else v for k, v in placed.items())
            with open(f'{out}/{name}.svg', 'w') as f:
                f.write('<svg xmlns="http://www.w3.org/2000/svg" viewBox="-60 -80 120 90">\n'
                        f'  <!-- {name} (A1.7, aus tools/design/abenteuer_assets.py). Ursprung = Boden, Blick nach rechts. -->\n'
                        f'  {body}\n</svg>\n')
    items = 'assets/adventure/items'
    os.makedirs(items, exist_ok=True)
    for name, art, scale in (('glanztropfen', glanztropfen(), 0.5), ('item', glitzerstein(), 0.4)):
        with open(f'{items}/{name}.svg', 'w') as f:
            dy = 8 if name == 'item' else 3
            f.write(svg({'': f'<g transform="translate(0,{dy}) scale({scale})">{art}</g>'},
                        f'Beute „{name}“ (A1.2). Ursprung = Mitte.'))


if __name__ == '__main__':
    main()

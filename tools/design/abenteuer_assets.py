"""Erzeugt die Spielgrafiken der Abenteuer-Gegner und der Beute aus den angenommenen
Entwürfen (E-225) nach assets/adventure/.

Aufruf: python3 tools/design/abenteuer_assets.py

Welteinheiten wie assets/elora/elora.svg: Ursprung in der Mitte der Kollisionsbox
(siehe assets/adventure/creatures.toml), Blick nach rechts.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from abenteuer_figuren import glanztropfen, glitzerstein, grashuepfer, pollenblaeser, stachelkaefer  # noqa: E402

# Name: (Boxhöhe, Maßstab, {Teil: Zeichnung})
CREATURES = {
    'stachelkaefer': (26, 0.36, {'idle': stachelkaefer()}),
    'pollenblaeser': (60, 0.42, {'idle': pollenblaeser(shots=False)}),
    'grashuepfer': (28, 0.36, {'idle': grashuepfer(), 'air': grashuepfer(True, arc=False)}),
}


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
    items = 'assets/adventure/items'
    os.makedirs(items, exist_ok=True)
    for name, art, scale in (('glanztropfen', glanztropfen(), 0.5), ('item', glitzerstein(), 0.4)):
        with open(f'{items}/{name}.svg', 'w') as f:
            dy = 8 if name == 'item' else 3
            f.write(svg({'': f'<g transform="translate(0,{dy}) scale({scale})">{art}</g>'},
                        f'Beute „{name}“ (A1.2). Ursprung = Mitte.'))


if __name__ == '__main__':
    main()

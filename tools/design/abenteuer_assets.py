"""Generates the game graphics of the adventure enemies and the loot from the accepted
drafts (E-225) into assets/adventure/.

Usage: python3 tools/design/abenteuer_assets.py

World units like assets/elora/elora.svg: origin in the centre of the collision box
(see assets/adventure/creatures.toml), facing right.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
import re  # noqa: E402

from abenteuer_figuren import (glanztropfen, glitzerstein, grashuepfer, klonk, lotte, oma_pfuetze, pip,  # noqa: E402
                               pollenblaeser, quellstein, schalter, stachelkaefer, truhe, tueftel)

from kapitel1_entwuerfe import biene, hummel, quellfunke, wabe, wabenhut  # noqa: E402
from kapitel2_entwuerfe import (eichhornpirat, pilzkind, pilzmama, pilzwicht, plumm, rune,  # noqa: E402
                                wurzelschlange, wurzelwaechter)
from kapitel3_entwuerfe import (duenenwurm, funkenmotte, giessstelle, palma, ruinenquelle,  # noqa: E402
                                sandkrabbe, sandschlange, sirup, steintafel, wasserschlauch)
from kapitel4_entwuerfe import (eiszapfen_figur, fledermaus, flocke, frostgeist, grauspur_eis,  # noqa: E402
                                kletterer, kristella, robbe, schneebrocken)


def strohpuppe():
    """Training dummy at Klonk (A1.9): post, straw sack, target."""
    s = '<path d="M 0,0 V -20" stroke="#2b2b2b" stroke-width="10"/><path d="M 0,0 V -20" stroke="#a87a52" stroke-width="5"/>'
    s += '<path d="M -26,-34 H 26" stroke="#2b2b2b" stroke-width="9" stroke-linecap="round"/><path d="M -26,-34 H 26" stroke="#a87a52" stroke-width="5" stroke-linecap="round"/>'
    s += '<ellipse cx="0" cy="-44" rx="22" ry="26" fill="#e0c070" stroke="#2b2b2b" stroke-width="4"/>'
    for y in (-58, -44, -30):
        s += f'<path d="M -16,{y} Q 0,{y + 4} 16,{y}" fill="none" stroke="#b8963c" stroke-width="2.5"/>'
    s += '<circle cx="0" cy="-46" r="9" fill="#e8685a" stroke="#2b2b2b" stroke-width="3"/><circle cx="0" cy="-46" r="3.5" fill="#fffaf0"/>'
    s += '<circle cx="0" cy="-78" r="11" fill="#e0c070" stroke="#2b2b2b" stroke-width="4"/>'
    s += '<path d="M -6,-88 l -4,-8 M 0,-89 l 0,-9 M 6,-88 l 4,-8" stroke="#b8963c" stroke-width="2.5" stroke-linecap="round"/>'
    return s


def wegweiser_npc():
    """Readable sign (E-273), same shape as the decoration."""
    s = '<rect x="-4" y="-92" width="8" height="92" fill="#a87a52" stroke="#2b2b2b" stroke-width="2"/>'
    s += '<path d="M -6,-84 H 42 L 52,-75 L 42,-66 H -6 Z" fill="#c9955c" stroke="#2b2b2b" stroke-width="2" stroke-linejoin="round"/>'
    s += '<path d="M 6,-58 H -42 L -52,-49 L -42,-40 H 6 Z" fill="#b8865a" stroke="#2b2b2b" stroke-width="2" stroke-linejoin="round"/>'
    s += '<path d="M 4,-78 H 34 M 4,-72 H 24 M -8,-52 H -36 M -8,-46 H -26" stroke="#7a5434" stroke-width="1.6" stroke-linecap="round"/>'
    return s


# Name: (box height, scale, {part: drawing})
CREATURES = {
    'stachelkaefer': (26, 0.36, {'idle': stachelkaefer()}),
    'pollenblaeser': (60, 0.42, {'idle': pollenblaeser(shots=False)}),
    'grashuepfer': (28, 0.36, {'idle': grashuepfer(), 'air': grashuepfer(True, arc=False)}),
    'strohpuppe': (40, 0.5, {'idle': strohpuppe()}),
    # Chapter 1 (R2-M2.1): warden with flight, dive and dazed pose, confused bee
    'brummbaer': (120, 0.62, {'idle': f'<g transform="translate(0,-26)">{hummel("flug")}</g>',
                              'dive': f'<g transform="translate(0,-40)">{hummel("sturz")}</g>',
                              'stunned': hummel('benommen')}),
    'wirrbiene': (24, 0.5, {'idle': f'<g transform="translate(0,-6)">{biene(1.0, 0, 0, angry=True)}</g>'}),
    # Chapter 2 (R2-M2.2): root snake outside / hidden, squirrel pirate, mushroom imp, mushroom child
    'wurzelschlange': (110, 0.72, {'idle': wurzelschlange(True), 'hidden': wurzelschlange(False)}),
    'eichhornpirat': (52, 0.4, {'idle': eichhornpirat()}),
    'pilzwicht': (40, 0.4, {'idle': pilzwicht()}),
    'pilzkind': (34, 0.21, {'idle': pilzkind(False)}),
    # Warden: parts per state m0 sleeps, m2 root attack, m3 core pulled (creature::warden)
    'wurzelwaechter': (280, 0.45, {'idle': wurzelwaechter('wach'), 'm0': wurzelwaechter('schlaf'),
                                   'm2': wurzelwaechter('angriff_ohne'), 'm3': wurzelwaechter('offen')}),
    # Chapter 3 (R2-M2.3): sand crab; dune worm under the sand (hidden = state 0), m1 warning,
    # otherwise mid-leap; spark moth
    'sandkrabbe': (30, 0.36, {'idle': sandkrabbe()}),
    'duenenwurm': (36, 0.42, {'idle': f'<g transform="translate(0,-6)">{duenenwurm("flug")}</g>',
                              'hidden': duenenwurm('spur'), 'm1': duenenwurm('warnung')}),
    'funkenmotte': (32, 0.36, {'idle': f'<g transform="translate(0,4)">{funkenmotte(False)}</g>'}),
    # Warden: m0 sleeps, m1 sand trail, m2 sand quakes, m3 arc (idle), m4 dazed (creature::serpent)
    'sandschlange': (70, 0.42, {'idle': f'<g transform="translate(20,-60)">{sandschlange("flug")}</g>',
                                'm0': sandschlange('spur'), 'm1': sandschlange('spur'),
                                'm2': sandschlange('beben'),
                                'm4': f'<g transform="translate(-60,0)">{sandschlange("benommen")}</g>'}),
    # Chapter 4 (R2-M2.4): icicle (tip down; trembles and falls in the same pose),
    # snow boulder of the avalanches (rolls, the client rotates it)
    'eiszapfen': (48, 0.5, {'idle': eiszapfen_figur()}),
    'schneebrocken': (36, 0.62, {'idle': f'<g transform="translate(0,1)">{schneebrocken(False)}</g>'}),
    # Enemies: seal slides (idle) and throws upright (m1 = creature::seal::THROW);
    # bat flies (idle) and sleeps upside down (m0 = creature::bat::HANG); frost ghost
    'schneeballrobbe': (28, 0.4, {'idle': robbe('rutschen'), 'm1': robbe('werfen')}),
    'fledermaus': (36, 0.4, {'idle': f'<g transform="translate(0,26)">{fledermaus("sturz")}</g>',
                             'm0': f'<g transform="translate(0,-86)">{fledermaus("haengt", bar=False)}</g>'}),
    'frostgeist': (52, 0.42, {'idle': frostgeist()}),
    # Warden (female): m0 sleeps (calm), idle floats, m2 frost breath, m3 exhausted (creature::queen)
    'kristella': (110, 0.36, {'idle': kristella('schwebend', False), 'm0': kristella('ruhig', False),
                              'm2': kristella('hauch', False), 'm3': kristella('erschoepft', False)}),
}


# NPCs: origin on the ground between the feet, scale like Elora (0.36) × size from the draft
CHARACTERS = {
    'oma': (0.95, oma_pfuetze()),
    'klonk': (1.2, klonk()),
    'lotte': (1.0, lotte()),
    'tueftel': (1.0, tueftel()),
    'pip': (0.7, pip()),
    # same size as the decoration (world units): 1 / 0.36
    'wegweiser': (1 / 0.36 * 0.55, wegweiser_npc()),
    # Chapter 1 (R2-M2.1): beekeeper Wabe, bumblebee as speaker after the fight
    'wabe': (1.0, wabe()),
    'hummel': (0.6, f'<g transform="translate(0,-60)">{hummel("ruhig")}</g>'),
    # Chapter 2 (R2-M2.2)
    'plumm': (1.0, plumm()),
    'pilzkind': (0.6, pilzkind(True)),
    'pilzkind_froh': (0.6, pilzkind(False)),
    'pilzmama': (0.85, pilzmama()),
    # friendly after the fight (conversation figure), same size as the warden
    'waechter': (1.25, wurzelwaechter('ruhig')),
    # Chapter 3 (R2-M2.3); fixed things at decoration size (1 / 0.36 × scale)
    'sirup': (1.0, sirup()),
    'palma': (1.0, palma()),
    'schlange': (1.1, f'<g transform="translate(-20,0)">{sandschlange("ruhig")}</g>'),
    'tafel': (1 / 0.36 * 0.5, steintafel()),
    'ruinenquelle': (1 / 0.36 * 0.5, ruinenquelle()),
    'giessstelle': (1 / 0.36 * 0.55, giessstelle(False)),
    'giessstelle_bluete': (1 / 0.36 * 0.55, giessstelle(True)),
    # Chapter 4 (R2-M2.4): Flocke, three climbers, Kristella after the fight (size of the warden)
    'flocke': (1.0, flocke()),
    'bolle': (0.85, kletterer('f2a65a', 'c97f3a', '#f2c14e')),
    'kiesel': (0.85, kletterer('8fd06a', '5fa03a', '#e8685a')),
    'wicke': (0.85, kletterer('c8a0e8', '9a70c0', '#5fc8e8')),
    'kristella': (1.0, kristella('ruhig', False)),
    'graue_stelle': (1 / 0.36 * 0.5, grauspur_eis()),
}


def heilpflanze():
    """Healing plant (E-258): blossom with a heart, friendly – counterpart to the pollen blower."""
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
    """Used-up healing plant: smaller, pale."""
    s = '<path d="M 0,0 Q -6,-18 2,-30" fill="none" stroke="#2b2b2b" stroke-width="8"/>'
    s += '<path d="M 0,0 Q -6,-18 2,-30" fill="none" stroke="#7f9a6a" stroke-width="4"/>'
    s += '<circle cx="4" cy="-34" r="9" fill="#d8ccc4" stroke="#2b2b2b" stroke-width="3"/>'
    return s


# Objects: origin on the ground, (scale, {part: drawing})
OBJECTS = {
    'truhe': (0.45, {'closed': truhe(), 'open': truhe(True)}),
    'quellstein': (0.45, {'off': quellstein(False), 'on': quellstein(True)}),
    'schalter': (0.5, {'off': schalter(False), 'on': schalter(True)}),
    'heilpflanze': (0.3, {'fresh': heilpflanze(), 'used': welk()}),
}


def plain(art):
    """Remove colour key ids from Elora's template (fixed colours in the game)."""
    return re.sub(r' id="tint-[^"]*"', '', art)


def svg(parts, comment, half=48):
    body = ''.join(f'<g id="{k}">{v}</g>' for k, v in parts.items())
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{-half} {-half} {2 * half} {2 * half}">\n'
            f'  <!-- {comment} -->\n  {body}\n</svg>\n')


def main():
    out = 'assets/adventure/creatures'
    os.makedirs(out, exist_ok=True)
    for name, (h, scale, parts) in CREATURES.items():
        placed = {k: f'<g transform="translate(0,{h / 2}) scale({scale})">{plain(v)}</g>' for k, v in parts.items()}
        with open(f'{out}/{name}.svg', 'w') as f:
            f.write(svg(placed, f'{name} (A1.2, aus tools/design/abenteuer_assets.py). Ursprung = Boxmitte, Blick nach rechts.',
                        max(48, round(h * 1.2))))
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
    for name, art, scale in (('glanztropfen', glanztropfen(), 0.5), ('item', glitzerstein(), 0.4),
                             # own images per item (R2-M2.1), otherwise `item` applies
                             # same size as the other icons (about 22 units)
                             ('biene', biene(1.0, 0, 0), 0.31), ('quellfunke', quellfunke(), 0.33),
                             ('wabenhut', wabenhut(), 0.25), ('rune', rune(True), 0.32),
                             # Chapter 3: empty and full waterskin
                             ('wasserschlauch', wasserschlauch(False), 0.32), ('wasser', wasserschlauch(True), 0.32)):
        with open(f'{items}/{name}.svg', 'w') as f:
            dy = {'item': 8, 'glanztropfen': 3, 'biene': 5.5, 'quellfunke': 10, 'wabenhut': 4, 'rune': 12,
                  'wasserschlauch': 10, 'wasser': 10}.get(name, 0)
            f.write(svg({'': f'<g transform="translate(0,{dy}) scale({scale})">{art}</g>'},
                        f'Beute „{name}“ (A1.2). Ursprung = Mitte.'))


if __name__ == '__main__':
    main()

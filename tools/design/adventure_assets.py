"""Generates the game graphics of the adventure enemies and the loot from the accepted
drafts (E-225) into assets/adventure/.

Usage: python3 tools/design/adventure_assets.py

World units like assets/elora/elora.svg: origin in the centre of the collision box
(see assets/adventure/creatures.toml), facing right.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
import re  # noqa: E402

from adventure_figures import (gleam_drop, glitter_stone, grasshopper, klonk, lotte, oma_pfuetze, pip,  # noqa: E402
                               pollen_blower, spring_stone, switch, spike_beetle, chest, tueftel)

from chapter1_drafts import bee, bumblebee, spring_spark, wabe, honeycomb_hat  # noqa: E402
from chapter2_drafts import (squirrel_pirate, mushroom_child, mushroom_mama, mushroom_imp, plumm, rune,  # noqa: E402
                                root_snake, root_warden)
from chapter3_drafts import (dune_worm, spark_moth, watering_spot, palma, ruin_spring,  # noqa: E402
                                sand_crab, sand_serpent, sirup, stone_tablet, water_skin)
from chapter4_drafts import (icicle_enemy, bat, flocke, frost_ghost, grey_trail_ice,  # noqa: E402
                                climber, kristella, seal, snow_boulder)


def straw_dummy():
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


def signpost_npc():
    """Readable sign (E-273), shape of the decoration; about twice as high as it was, with a
    longer post (playtest, E-354)."""
    s = '<rect x="-4" y="-135" width="8" height="135" fill="#a87a52" stroke="#2b2b2b" stroke-width="2"/>'
    s += '<g transform="translate(0,-43)">'
    s += '<path d="M -6,-84 H 42 L 52,-75 L 42,-66 H -6 Z" fill="#c9955c" stroke="#2b2b2b" stroke-width="2" stroke-linejoin="round"/>'
    s += '<path d="M 6,-58 H -42 L -52,-49 L -42,-40 H 6 Z" fill="#b8865a" stroke="#2b2b2b" stroke-width="2" stroke-linejoin="round"/>'
    s += '<path d="M 4,-78 H 34 M 4,-72 H 24 M -8,-52 H -36 M -8,-46 H -26" stroke="#7a5434" stroke-width="1.6" stroke-linecap="round"/>'
    s += '</g>'
    return s


# Name: (box height, scale, {part: drawing})
CREATURES = {
    'stachelkaefer': (26, 0.36, {'idle': spike_beetle()}),
    'pollenblaeser': (60, 0.42, {'idle': pollen_blower(shots=False)}),
    'grashuepfer': (28, 0.36, {'idle': grasshopper(), 'air': grasshopper(True, arc=False)}),
    'strohpuppe': (40, 0.5, {'idle': straw_dummy()}),
    # Chapter 1 (R2-M2.1): warden with flight, dive and dazed pose, confused bee
    'brummbaer': (120, 0.62, {'idle': f'<g transform="translate(0,-26)">{bumblebee("flug")}</g>',
                              'dive': f'<g transform="translate(0,-40)">{bumblebee("sturz")}</g>',
                              'stunned': bumblebee('benommen')}),
    'wirrbiene': (24, 0.5, {'idle': f'<g transform="translate(0,-6)">{bee(1.0, 0, 0, angry=True)}</g>'}),
    # Chapter 2 (R2-M2.2): root snake outside / hidden, squirrel pirate, mushroom imp, mushroom child
    'wurzelschlange': (110, 0.72, {'idle': root_snake(True), 'hidden': root_snake(False)}),
    'eichhornpirat': (52, 0.4, {'idle': squirrel_pirate()}),
    'pilzwicht': (40, 0.4, {'idle': mushroom_imp()}),
    'pilzkind': (34, 0.21, {'idle': mushroom_child(False)}),
    # Warden: parts per state m0 sleeps, m2 root attack, m3 core pulled (creature::warden)
    'wurzelwaechter': (280, 0.45, {'idle': root_warden('wach'), 'm0': root_warden('schlaf'),
                                   'm2': root_warden('angriff_ohne'), 'm3': root_warden('offen')}),
    # Chapter 3 (R2-M2.3): sand crab; dune worm under the sand (hidden = state 0), m1 warning,
    # otherwise mid-leap; spark moth
    'sandkrabbe': (30, 0.36, {'idle': sand_crab()}),
    'duenenwurm': (36, 0.42, {'idle': f'<g transform="translate(0,-6)">{dune_worm("flug")}</g>',
                              'hidden': dune_worm('spur'), 'm1': dune_worm('warnung')}),
    'funkenmotte': (32, 0.36, {'idle': f'<g transform="translate(0,4)">{spark_moth(False)}</g>'}),
    # Warden: m0 sleeps, m1 sand trail, m2 sand quakes, m3 arc (idle), m4 dazed (creature::serpent)
    'sandschlange': (70, 0.42, {'idle': f'<g transform="translate(20,-60)">{sand_serpent("flug")}</g>',
                                'm0': sand_serpent('spur'), 'm1': sand_serpent('spur'),
                                'm2': sand_serpent('beben'),
                                'm4': f'<g transform="translate(-60,0)">{sand_serpent("benommen")}</g>'}),
    # Chapter 4 (R2-M2.4): icicle (tip down; trembles and falls in the same pose),
    # snow boulder of the avalanches (rolls, the client rotates it)
    'eiszapfen': (48, 0.5, {'idle': icicle_enemy()}),
    'schneebrocken': (36, 0.62, {'idle': f'<g transform="translate(0,1)">{snow_boulder(False)}</g>'}),
    # Enemies: seal slides (idle) and throws upright (m1 = creature::seal::THROW);
    # bat flies (idle) and sleeps upside down (m0 = creature::bat::HANG); frost ghost
    'schneeballrobbe': (28, 0.4, {'idle': seal('rutschen'), 'm1': seal('werfen')}),
    'fledermaus': (36, 0.4, {'idle': f'<g transform="translate(0,26)">{bat("sturz")}</g>',
                             'm0': f'<g transform="translate(0,-86)">{bat("haengt", bar=False)}</g>'}),
    'frostgeist': (52, 0.42, {'idle': frost_ghost()}),
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
    # decoration scale (world units: 1 / 0.36), about twice as high as the decoration (E-354)
    'wegweiser': (1 / 0.36 * 0.75, signpost_npc()),
    # Chapter 1 (R2-M2.1): beekeeper Wabe, bumblebee as speaker after the fight
    'wabe': (1.0, wabe()),
    'hummel': (0.6, f'<g transform="translate(0,-60)">{bumblebee("ruhig")}</g>'),
    # Chapter 2 (R2-M2.2)
    'plumm': (1.0, plumm()),
    'pilzkind': (0.6, mushroom_child(True)),
    'pilzkind_froh': (0.6, mushroom_child(False)),
    'pilzmama': (0.85, mushroom_mama()),
    # friendly after the fight (conversation figure), same size as the warden
    'waechter': (1.25, root_warden('ruhig')),
    # Chapter 3 (R2-M2.3); fixed things at decoration size (1 / 0.36 × scale)
    'sirup': (1.0, sirup()),
    'palma': (1.0, palma()),
    'schlange': (1.1, f'<g transform="translate(-20,0)">{sand_serpent("ruhig")}</g>'),
    'tafel': (1 / 0.36 * 0.5, stone_tablet()),
    'ruinenquelle': (1 / 0.36 * 0.5, ruin_spring()),
    'giessstelle': (1 / 0.36 * 0.55, watering_spot(False)),
    'giessstelle_bluete': (1 / 0.36 * 0.55, watering_spot(True)),
    # Chapter 4 (R2-M2.4): Flocke, three climbers, Kristella after the fight (size of the warden)
    'flocke': (1.0, flocke()),
    'bolle': (0.85, climber('f2a65a', 'c97f3a', '#f2c14e')),
    'kiesel': (0.85, climber('8fd06a', '5fa03a', '#e8685a')),
    'wicke': (0.85, climber('c8a0e8', '9a70c0', '#5fc8e8')),
    'kristella': (1.0, kristella('ruhig', False)),
    'graue_stelle': (1 / 0.36 * 0.5, grey_trail_ice()),
}


def healing_plant():
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


def withered_plant():
    """Used-up healing plant: smaller, pale."""
    s = '<path d="M 0,0 Q -6,-18 2,-30" fill="none" stroke="#2b2b2b" stroke-width="8"/>'
    s += '<path d="M 0,0 Q -6,-18 2,-30" fill="none" stroke="#7f9a6a" stroke-width="4"/>'
    s += '<circle cx="4" cy="-34" r="9" fill="#d8ccc4" stroke="#2b2b2b" stroke-width="3"/>'
    return s


# Objects: origin on the ground, (scale, {part: drawing})
OBJECTS = {
    'truhe': (0.45, {'closed': chest(), 'open': chest(True)}),
    'quellstein': (0.45, {'off': spring_stone(False), 'on': spring_stone(True)}),
    'schalter': (0.5, {'off': switch(False), 'on': switch(True)}),
    'heilpflanze': (0.3, {'fresh': healing_plant(), 'used': withered_plant()}),
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
    for name, art, scale in (('glanztropfen', gleam_drop(), 0.5), ('item', glitter_stone(), 0.4),
                             # own images per item (R2-M2.1), otherwise `item` applies
                             # same size as the other icons (about 22 units)
                             ('biene', bee(1.0, 0, 0), 0.31), ('quellfunke', spring_spark(), 0.33),
                             ('wabenhut', honeycomb_hat(), 0.25), ('rune', rune(True), 0.32),
                             # Chapter 3: empty and full waterskin
                             ('wasserschlauch', water_skin(False), 0.32), ('wasser', water_skin(True), 0.32)):
        with open(f'{items}/{name}.svg', 'w') as f:
            dy = {'item': 8, 'glanztropfen': 3, 'biene': 5.5, 'quellfunke': 10, 'wabenhut': 4, 'rune': 12,
                  'wasserschlauch': 10, 'wasser': 10}.get(name, 0)
            f.write(svg({'': f'<g transform="translate(0,{dy}) scale({scale})">{art}</g>'},
                        f'Beute „{name}“ (A1.2). Ursprung = Mitte.'))


if __name__ == '__main__':
    main()

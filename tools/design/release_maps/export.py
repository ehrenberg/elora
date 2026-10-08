"""Checks the release maps and writes their layouts to apps/elora-client/src/editor/release_layouts.rs.

Usage (from the project folder): python3 tools/design/release_maps/export.py
Afterwards: cargo test -p elora-client --bin elora write_release_maps -- --ignored  (writes maps/*.emap)
"""
import importlib
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from kit import check  # noqa: E402

MAPS = [('wiese', 'WIESE', False), ('wueste', 'WUESTE', False), ('winter', 'WINTER', False),
        ('wald', 'WALD', True), ('nacht', 'NACHT', True)]

out = ['//! Layouts der Release-Karten (M6.10, E-134, E-155 bis E-158), erzeugt mit dem Baukasten',
       '//! `tools/design/release_maps/` und dort geprüft (Erreichbarkeit, Spawns, Spiegelung).',
       '//! Zeichen wie in den Aufzeichnungen; Entities `S R B r b h a L G`.', '']
failed = False
for mod, const, ctf in MAPS:
    g = importlib.import_module(mod).build()
    errs, unreach = check(g, ctf=ctf, name=mod)
    failed |= bool(errs or unreach)
    out.append(f'pub const {const}: &[&str] = &[')
    out += [f'    r"{r}",' for r in g.rows()]
    out += ['];', '']
if failed:
    sys.exit('Prüfung fehlgeschlagen – nichts geschrieben')
with open('apps/elora-client/src/editor/release_layouts.rs', 'w') as fh:
    fh.write('\n'.join(out))
print('geschrieben')

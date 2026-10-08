"""Generates docs/release-2/design/abenteuer-ui.svg – user interface of the adventure (R2-M1, A1.0).

Usage: python3 tools/design/abenteuer_ui.py && cargo xtask svg-preview \
        docs/release-2/design/abenteuer-ui.svg docs/release-2/design/abenteuer-ui.png 1400

Six screens (16:9): game with HUD, conversation, inventory, skills, quests, merchant.
Game HUD in the style of the multiplayer bar (dark, bottom), menus in the "light & soft" style (E-123).
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from abenteuer_figuren import (drop, g, glanztropfen, glitzerstein, grashuepfer, klonk, lotte, oma_pfuetze,  # noqa: E402
                               pip, stachelkaefer, star, truhe)

OUT = '#2b2b2b'
TEXT = '#3b3024'
DIM = '#8a7a66'
HUD = 'fill="#1e2a36" fill-opacity="0.72"'
W, H = 660, 371
ELORA = drop('f2c14e', 'd9a43a')


def text(x, y, t, size=12, color=TEXT, anchor='start', weight='normal', style='normal', opacity=1):
    t = t.replace('&', '&amp;')
    return (f'<text x="{x}" y="{y}" font-size="{size}" fill="{color}" text-anchor="{anchor}" '
            f'font-weight="{weight}" font-style="{style}" opacity="{opacity}">{t}</text>')


def card(x, y, w, h, r=16):
    return (f'<rect x="{x + 3}" y="{y + 5}" width="{w}" height="{h}" rx="{r}" fill="#1e2a36" fill-opacity="0.12"/>'
            f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="#fffaf0" stroke="#e7dcc8" stroke-width="1.5"/>')


def pill(x, y, w, t, color, h=22, size=11, fg='#ffffff'):
    return (f'<rect x="{x}" y="{y + 3}" width="{w}" height="{h}" rx="{h / 2}" fill="#{color}" fill-opacity="0.45"/>'
            f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{h / 2}" fill="#{color}" stroke="{OUT}" stroke-width="1.5"/>'
            + text(x + w / 2, y + h / 2 + size * 0.36, t, size, fg, 'middle'))


def bar(x, y, w, h, frac, color, back='#ffffff', back_op=0.25):
    return (f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{h / 2}" fill="{back}" fill-opacity="{back_op}"/>'
            f'<rect x="{x}" y="{y}" width="{w * frac:.1f}" height="{h}" rx="{h / 2}" fill="{color}"/>')


def world(ox, oy, uid, dim=False):
    """Flower meadow excerpt: sky, hills, ground with grass, flowers."""
    s = f'''<defs><linearGradient id="sky{uid}" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#a9cde8"/><stop offset="1" stop-color="#e8f1f7"/></linearGradient>
      <clipPath id="clip{uid}"><rect x="{ox}" y="{oy}" width="{W}" height="{H}" rx="12"/></clipPath></defs>
    <g clip-path="url(#clip{uid})">
      <rect x="{ox}" y="{oy}" width="{W}" height="{H}" fill="url(#sky{uid})"/>
      <ellipse cx="{ox + 180}" cy="{oy + 60}" rx="46" ry="13" fill="#ffffff" opacity="0.8"/>
      <ellipse cx="{ox + 470}" cy="{oy + 44}" rx="60" ry="15" fill="#ffffff" opacity="0.7"/>
      <ellipse cx="{ox + 150}" cy="{oy + 300}" rx="260" ry="90" fill="#b5d9a0"/>
      <ellipse cx="{ox + 520}" cy="{oy + 310}" rx="240" ry="100" fill="#a6cf90"/>
      <rect x="{ox}" y="{oy + 290}" width="{W}" height="90" fill="#a8744a"/>
      <rect x="{ox}" y="{oy + 284}" width="{W}" height="14" rx="7" fill="#6cbf4a"/>
      <rect x="{ox + 420}" y="{oy + 210}" width="120" height="16" rx="8" fill="#6cbf4a"/>
      <rect x="{ox + 424}" y="{oy + 222}" width="112" height="18" rx="6" fill="#a8744a"/>'''
    for i, (x, c) in enumerate(((60, 'ef7fb0'), (250, 'f2c14e'), (330, '5aaee8'), (600, 'ef7fb0'))):
        s += (f'<path d="M {ox + x},{oy + 286} L {ox + x},{oy + 272}" stroke="#4f9a3a" stroke-width="2.5"/>'
              f'<circle cx="{ox + x}" cy="{oy + 270}" r="5" fill="#{c}" stroke="{OUT}" stroke-width="1.5"/>')
    return s


def world_end(ox, oy, dim=False):
    s = f'<rect x="{ox}" y="{oy}" width="{W}" height="{H}" fill="#1e2a36" fill-opacity="0.35"/>' if dim else ''
    return s + '</g>'


def hud_bar(ox, oy):
    """Bottom bar as in multiplayer, extended by level and experience."""
    x, y, w = ox + W / 2 - 170, oy + H - 50, 340
    s = f'<rect x="{x}" y="{y}" width="{w}" height="40" rx="12" {HUD}/>'
    # level badge
    s += f'<circle cx="{x + 22}" cy="{y + 20}" r="14" fill="#f2c14e" stroke="{OUT}" stroke-width="2"/>'
    s += text(x + 22, y + 25, '3', 13, TEXT, 'middle', 'bold')
    s += bar(x + 44, y + 9, 110, 7, 0.7, '#e05a7a')
    s += text(x + 158, y + 16, '7/10', 8, '#ffffff', opacity=0.8)
    s += bar(x + 44, y + 24, 110, 6, 0.45, '#7fd99a')
    s += text(x + 158, y + 30, 'EP 45 %', 8, '#ffffff', opacity=0.8)
    # weapons
    s += f'<rect x="{x + 204}" y="{y + 5}" width="30" height="30" rx="8" fill="#ffffff" fill-opacity="0.22"/>'
    s += f'<path d="M {x + 208},{y + 20} L {x + 222},{y + 20}" stroke="#8a6a4a" stroke-width="3.5" stroke-linecap="round"/>'
    s += f'<rect x="{x + 222}" y="{y + 12}" width="8" height="16" rx="3" fill="#c8ced6" stroke="{OUT}" stroke-width="1.5"/>'
    s += f'<rect x="{x + 246}" y="{y + 15}" width="24" height="9" rx="4" fill="#6cbf4a" stroke="{OUT}" stroke-width="1.5" opacity="0.5"/>'
    s += f'<rect x="{x + 282}" y="{y + 17}" width="26" height="5" rx="2.5" fill="#5aaee8" stroke="{OUT}" stroke-width="1.2" opacity="0.5"/>'
    s += text(x + 324, y + 25, '🔒', 9, '#ffffff', 'middle', opacity=0.0)
    return s


def drops_counter(ox, oy, n='128'):
    s = f'<rect x="{ox + 14}" y="{oy + 14}" width="92" height="30" rx="15" {HUD}/>'
    s += g(ox + 32, oy + 32, 0.55, glanztropfen())
    s += text(ox + 50, oy + 34, n, 13, '#ffffff')
    return s


def quest_tracker(ox, oy):
    x, y = ox + W - 220, oy + 14
    s = f'<rect x="{x}" y="{y}" width="206" height="62" rx="12" {HUD}/>'
    s += text(x + 12, y + 18, 'AUFGABE', 8, '#f2c14e')
    s += text(x + 12, y + 35, 'Der blasse Brunnen', 12, '#ffffff')
    s += f'<rect x="{x + 12}" y="{y + 44}" width="9" height="9" rx="2" fill="none" stroke="#ffffff" stroke-width="1.5"/>'
    s += text(x + 27, y + 52, 'Wiesenpfad bis zur Brücke (0/1)', 10, '#ffffff', opacity=0.85)
    return s


def bubble(x, y, t, w=None):
    w = w or 12 + len(t) * 6.6
    return (f'<rect x="{x - w / 2}" y="{y - 26}" width="{w}" height="24" rx="12" fill="#ffffff" stroke="{OUT}" stroke-width="2"/>'
            f'<path d="M {x - 6},{y - 3} L {x},{y + 7} L {x + 4},{y - 3} Z" fill="#ffffff" stroke="{OUT}" stroke-width="2" stroke-linejoin="round"/>'
            f'<rect x="{x - 7}" y="{y - 5}" width="13" height="4" fill="#ffffff"/>'
            + text(x, y - 10, t, 11, TEXT, 'middle'))


def key_hint(x, y, key, label):
    return (f'<rect x="{x - 9}" y="{y - 14}" width="18" height="18" rx="5" fill="#ffffff" stroke="{OUT}" stroke-width="2"/>'
            + text(x, y, key, 11, TEXT, 'middle', 'bold')
            + text(x + 14, y, label, 11, '#ffffff', 'start') )


def screen_game(ox, oy):
    s = world(ox, oy, 'g')
    s += g(ox + 110, oy + 286, 0.42, pip())
    s += bubble(ox + 112, oy + 232, 'Pass auf die Käfer auf!')
    s += g(ox + 250, oy + 286, 0.5, ELORA)
    s += f'<path d="M {ox + 262},{oy + 244} L {ox + 446},{oy + 212}" stroke="#2b2b2b" stroke-width="2"/>'
    s += f'<circle cx="{ox + 446}" cy="{oy + 212}" r="4" fill="#9aa4ae" stroke="{OUT}" stroke-width="1.5"/>'
    s += g(ox + 400, oy + 286, 0.5, stachelkaefer(), flip=True)
    s += f'<rect x="{ox + 378}" y="{oy + 238}" width="44" height="6" rx="3" fill="#1e2a36" fill-opacity="0.5"/><rect x="{ox + 378}" y="{oy + 238}" width="26" height="6" rx="3" fill="#e05a7a"/>'
    s += text(ox + 376, oy + 228, '−3', 13, '#ffffff', weight='bold')
    s += g(ox + 590, oy + 286, 0.7, truhe())
    s += f'<rect x="{ox + 548}" y="{oy + 210}" width="84" height="24" rx="12" {HUD}/>' + key_hint(ox + 566, oy + 227, 'E', 'Öffnen')
    s += g(ox + 330, oy + 190, 0.5, glanztropfen()) + g(ox + 352, oy + 182, 0.5, glanztropfen())
    s += world_end(ox, oy)
    s += drops_counter(ox, oy) + quest_tracker(ox, oy) + hud_bar(ox, oy)
    return s


def screen_dialog(ox, oy):
    s = world(ox, oy, 'd')
    s += g(ox + 380, oy + 286, 0.6, oma_pfuetze(), flip=True)
    s += g(ox + 300, oy + 286, 0.5, ELORA)
    s += world_end(ox, oy, dim=True)
    x, y, w, h = ox + 24, oy + 176, W - 48, 180
    s += card(x, y, w, h, 18)
    # picture of the figure
    s += f'<circle cx="{x + 64}" cy="{y + 70}" r="48" fill="#b9a3e3" fill-opacity="0.25" stroke="#e7dcc8" stroke-width="1.5"/>'
    s += (f'<g clip-path="url(#portrait)"><defs><clipPath id="portrait"><circle cx="{x + 64}" cy="{y + 70}" r="48"/></clipPath></defs>'
          + g(x + 50, y + 134, 0.95, oma_pfuetze()) + '</g>')
    s += pill(x + 24, y + 128, 80, 'Oma Pfütze', 'a77be0', 20, 10)
    s += text(x + 128, y + 32, 'Ach, Elora, Kind. Siehst du, wie blass die Blumen am Brunnen sind?', 12)
    s += text(x + 128, y + 50, 'Als ich so klein war wie du, sang die Blütenquelle noch Morgenlieder …', 12)
    choices = [('f2c14e', 'freundlich', 'Ich finde heraus, was mit der Quelle los ist, versprochen!'),
               ('5aaee8', 'neugierig', 'Morgenlieder? Erzähl mir mehr von den Quellen.'),
               ('e8685a', 'frech', 'Vielleicht sind die Blumen einfach müde, Oma.')]
    for i, (c, tone, t) in enumerate(choices):
        cy = y + 70 + i * 30
        if i == 0:
            s += f'<rect x="{x + 122}" y="{cy}" width="{w - 140}" height="26" rx="13" fill="#f2c14e" fill-opacity="0.35"/>'
        s += f'<circle cx="{x + 138}" cy="{cy + 13}" r="6" fill="#{c}" stroke="{OUT}" stroke-width="1.5"/>'
        s += text(x + 152, cy + 17, f'{i + 1}', 11, DIM, weight='bold')
        s += text(x + 166, cy + 17, t, 12)
        s += text(x + w - 26, cy + 17, tone, 10, DIM, 'end', style='italic')
    s += text(x + w - 18, y + h - 10, 'Leertaste: weiter · 1–3: wählen', 9, DIM, 'end')
    return s


def menu_frame(ox, oy, uid, active):
    s = world(ox, oy, uid) + world_end(ox, oy, dim=True)
    s += card(ox + 16, oy + 14, W - 32, H - 28, 18)
    tabs = [('Inventar', 'f28c3a'), ('Fähigkeiten', '6cbf4a'), ('Aufgaben', '5aaee8'), ('Karte', 'a77be0')]
    for i, (t, c) in enumerate(tabs):
        s += pill(ox + 30 + i * 88, oy + 26, 80, t, c if i == active else 'e0c89a', 22, 11)
    s += g(ox + W - 110, oy + 40, 0.5, glanztropfen()) + text(ox + W - 96, oy + 43, '128', 12)
    s += f'<circle cx="{ox + W - 48}" cy="{oy + 38}" r="11" fill="#f2c14e" stroke="{OUT}" stroke-width="1.5"/>'
    s += text(ox + W - 48, oy + 42, '3', 11, TEXT, 'middle', 'bold')
    return s


def slot(x, y, size=40, fill='#ffffff', sel=False):
    st = f'stroke="{TEXT}" stroke-width="2.5"' if sel else 'stroke="#e7dcc8" stroke-width="1.5"'
    return f'<rect x="{x}" y="{y}" width="{size}" height="{size}" rx="10" fill="{fill}" {st}/>'


def icon_hat(cx, cy):
    return (f'<ellipse cx="{cx}" cy="{cy + 6}" rx="15" ry="4" fill="#a8744a" stroke="{OUT}" stroke-width="1.5"/>'
            f'<path d="M {cx - 9},{cy + 5} Q {cx - 8},{cy - 10} {cx},{cy - 10} Q {cx + 8},{cy - 10} {cx + 9},{cy + 5} Z" fill="#a8744a" stroke="{OUT}" stroke-width="1.5"/>'
            f'<path d="M {cx - 9},{cy + 1} L {cx + 9},{cy + 1}" stroke="#6cbf4a" stroke-width="3"/>')


def icon_boots(cx, cy):
    return (f'<path d="M {cx - 8},{cy - 10} L {cx - 8},{cy + 6} L {cx + 10},{cy + 6} Q {cx + 10},{cy} {cx},{cy - 1} L {cx},{cy - 10} Z" '
            f'fill="#e0c89a" stroke="{OUT}" stroke-width="1.5" stroke-linejoin="round"/>')


def icon_cape(cx, cy):
    return (f'<path d="M {cx - 6},{cy - 10} L {cx + 6},{cy - 10} L {cx + 12},{cy + 10} Q {cx},{cy + 6} {cx - 12},{cy + 10} Z" '
            f'fill="#5aaee8" stroke="{OUT}" stroke-width="1.5" stroke-linejoin="round"/>')


def icon_amulet(cx, cy):
    return (f'<path d="M {cx - 8},{cy - 10} Q {cx},{cy} {cx + 8},{cy - 10}" fill="none" stroke="{OUT}" stroke-width="1.5"/>'
            f'<circle cx="{cx}" cy="{cy + 3}" r="6" fill="#ef7fb0" stroke="{OUT}" stroke-width="1.5"/>')


def icon_potion(cx, cy, c='e05a7a'):
    return (f'<rect x="{cx - 3}" y="{cy - 12}" width="6" height="6" rx="1.5" fill="#a8744a" stroke="{OUT}" stroke-width="1.2"/>'
            f'<circle cx="{cx}" cy="{cy + 2}" r="9" fill="#{c}" stroke="{OUT}" stroke-width="1.5"/>'
            f'<path d="M {cx - 4},{cy - 1} Q {cx - 3},{cy - 4} {cx},{cy - 5}" stroke="#ffffff" stroke-width="2" fill="none"/>')


def icon_material(cx, cy, c='e0b85a'):
    return f'<path d="M {cx - 9},{cy + 6} L {cx - 5},{cy - 8} L {cx + 6},{cy - 9} L {cx + 10},{cy + 4} L {cx},{cy + 9} Z" fill="#{c}" stroke="{OUT}" stroke-width="1.5" stroke-linejoin="round"/>'


def screen_inventory(ox, oy):
    s = menu_frame(ox, oy, 'i', 0)
    # equipment around Elora
    s += f'<circle cx="{ox + 130}" cy="{oy + 190}" r="70" fill="#f2c14e" fill-opacity="0.15"/>'
    s += g(ox + 130, oy + 250, 0.85, ELORA)
    s += g(ox + 140, oy + 250, 0.85, '<ellipse cx="10" cy="-104" rx="40" ry="8" fill="#a8744a" stroke="#2b2b2b" stroke-width="4"/>'
           '<path d="M -14,-106 Q -10,-140 14,-140 Q 36,-140 34,-106 Z" fill="#a8744a" stroke="#2b2b2b" stroke-width="4"/>'
           '<path d="M -13,-114 L 34,-114 L 34,-106 L -13,-106 Z" fill="#6cbf4a"/>')
    for (x, y, ic, lbl) in ((40, 76, icon_hat, 'Hut'), (180, 76, icon_cape, 'Umhang'),
                            (40, 260, icon_boots, 'Stiefel'), (180, 260, icon_amulet, 'Anhänger')):
        s += slot(ox + x, oy + y, 40)
        if lbl != 'Umhang':
            s += ic(ox + x + 20, oy + y + 20)
        s += text(ox + x + 20, oy + y + 54, lbl, 9, DIM, 'middle')
    # backpack
    s += text(ox + 260, oy + 76, 'Rucksack', 13, TEXT, weight='bold')
    items = [(icon_potion, 'e05a7a', '3'), (icon_potion, '5aaee8', '1'), (icon_material, 'e0b85a', '5'), (icon_cape, None, ''),
             (icon_material, 'bfe6f5', '2'), (icon_amulet, None, ''), None, None, None, None, None, None]
    for i, it in enumerate(items):
        x, y = ox + 260 + (i % 6) * 48, oy + 86 + (i // 6) * 48
        s += slot(x, y, 42, sel=(i == 3))
        if it:
            ic, c, n = it
            s += ic(x + 21, y + 21, c) if c else ic(x + 21, y + 21)
            if n:
                s += text(x + 37, y + 38, n, 9, TEXT, 'end', 'bold')
    # description
    s += f'<rect x="{ox + 260}" y="{oy + 190}" width="282" height="110" rx="12" fill="#ffffff" stroke="#e7dcc8"/>'
    s += icon_cape(ox + 284, oy + 216)
    s += text(ox + 304, oy + 212, 'Tauumhang', 13, TEXT, weight='bold')
    s += text(ox + 304, oy + 228, 'selten · aus den Blütenwiesen', 10, '#5aaee8')
    s += text(ox + 276, oy + 252, '+1 Leben · Gleiten 10 % länger', 11)
    s += text(ox + 276, oy + 270, '„Riecht ein bisschen nach Morgen.“', 10, DIM, style='italic')
    s += pill(ox + 440, oy + 266, 90, 'Anlegen', '6cbf4a', 22, 11)
    return s


def screen_skills(ox, oy):
    s = menu_frame(ox, oy, 's', 1)
    s += text(ox + 34, oy + 76, 'Fähigkeitenpunkte: 2', 12, TEXT, weight='bold')
    s += text(ox + W - 34, oy + 76, 'Neue Fähigkeiten bringt Tüftel aus Quellfunken', 10, DIM, 'end', style='italic')
    branches = [('Bewegung', '5aaee8', ['Hook-Ruck', 'Längerer Hook', 'Luftsprung+', 'Gleiten+']),
                ('Kampf', 'e8685a', ['Hammer-Wucht', 'Granaten-Splitter', 'Laser-Abpraller', 'Stampf-Welle']),
                ('Quelle', '3fc1b0', ['Mehr Leben', 'Tropfen-Magnet', 'Heilende Quelle', 'Zweite Chance'])]
    for b, (name, c, nodes) in enumerate(branches):
        cx = ox + 100 + b * 200
        s += pill(cx - 55, oy + 90, 110, name, c, 22, 11)
        for i, n in enumerate(nodes):
            y = oy + 140 + i * 52
            state = 'have' if i < [2, 1, 1][b] else ('next' if i == [2, 1, 1][b] else 'lock')
            if i < len(nodes) - 1:
                s += f'<path d="M {cx},{y + 15} L {cx},{y + 37}" stroke="{"#" + c if state == "have" else "#d8ccb8"}" stroke-width="4"/>'
            fill = '#' + c if state == 'have' else '#ffffff'
            st = f'stroke="{OUT}" stroke-width="2"' if state != 'lock' else 'stroke="#d8ccb8" stroke-width="2" stroke-dasharray="4 3"'
            s += f'<circle cx="{cx}" cy="{y}" r="15" fill="{fill}" {st}/>'
            if state == 'have':
                s += f'<path d="M {cx - 6},{y} L {cx - 1},{y + 5} L {cx + 7},{y - 5}" fill="none" stroke="#ffffff" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>'
            elif state == 'next':
                s += text(cx, y + 5, '+', 16, TEXT, 'middle', 'bold')
                s += f'<circle cx="{cx}" cy="{y}" r="20" fill="none" stroke="#f2c14e" stroke-width="3" opacity="0.8"/>'
            s += text(cx + 24, y + 4, n, 11, TEXT if state != 'lock' else DIM)
    return s


def screen_quests(ox, oy):
    s = menu_frame(ox, oy, 'q', 2)
    quests = [('Hauptaufgabe', 'Der blasse Brunnen', True, 'f2c14e'),
              ('Hauptaufgabe', 'Oma Pfützes Lied', False, 'f2c14e'),
              ('Pip', 'Der Glitzerstein im Gras', False, 'f28c3a'),
              ('Anschlagbrett', 'Käferplage am Zaun', False, '9aa4ae'),
              ('Klonk', 'Bernstein für den Hammer', False, 'a8744a')]
    for i, (src, name, sel, c) in enumerate(quests):
        y = oy + 66 + i * 50
        if sel:
            s += f'<rect x="{ox + 28}" y="{y}" width="236" height="44" rx="12" fill="#f2c14e" fill-opacity="0.35"/>'
        s += f'<circle cx="{ox + 46}" cy="{y + 22}" r="7" fill="#{c}" stroke="{OUT}" stroke-width="1.5"/>'
        s += text(ox + 60, y + 18, src, 9, DIM)
        s += text(ox + 60, y + 34, name, 12)
    x = ox + 280
    s += f'<rect x="{x}" y="{oy + 66}" width="{W - 312}" height="240" rx="12" fill="#ffffff" stroke="#e7dcc8"/>'
    s += text(x + 16, oy + 92, 'Der blasse Brunnen', 15, TEXT, weight='bold')
    s += text(x + 16, oy + 110, 'Hauptaufgabe · Prolog', 10, DIM)
    s += text(x + 16, oy + 136, 'Die Blumen am Dorfbrunnen verlieren ihre Farbe.', 11)
    s += text(x + 16, oy + 152, 'Oma Pfütze glaubt, die Blütenquelle sei verstummt.', 11)
    steps = [('Mit Oma Pfütze sprechen', True), ('Bei Tüftel den Hook abholen', True),
             ('Wiesenpfad bis zur Brücke', False), ('Die Blütenquelle finden', None)]
    for i, (t, done) in enumerate(steps):
        y = oy + 182 + i * 24
        if done is None:
            s += text(x + 16, y + 9, '?', 11, DIM, weight='bold') + text(x + 34, y + 9, '…', 11, DIM)
            continue
        s += f'<rect x="{x + 14}" y="{y}" width="12" height="12" rx="3" fill="{"#6cbf4a" if done else "#ffffff"}" stroke="{OUT}" stroke-width="1.5"/>'
        if done:
            s += f'<path d="M {x + 17},{y + 6} L {x + 20},{y + 9} L {x + 24},{y + 3}" fill="none" stroke="#ffffff" stroke-width="2"/>'
        s += text(x + 34, y + 10, t, 11, DIM if done else TEXT)
    s += text(x + 16, oy + 292, 'Belohnung: 50 EP · 30', 10, DIM) + g(x + 136, oy + 291, 0.32, glanztropfen())
    return s


def screen_shop(ox, oy):
    s = world(ox, oy, 'h') + world_end(ox, oy, dim=True)
    s += card(ox + 16, oy + 14, W - 32, H - 28, 18)
    s += f'<circle cx="{ox + 100}" cy="{oy + 150}" r="66" fill="#8fd0f0" fill-opacity="0.25"/>'
    s += g(ox + 100, oy + 214, 0.85, lotte())
    s += pill(ox + 46, oy + 236, 108, 'Lottes Laden', '5aaee8', 22, 11)
    s += bubble(ox + 100, oy + 72, 'Kaum gebraucht!', 112)
    for i, (t, c) in enumerate([('Kaufen', 'f28c3a'), ('Verkaufen', 'e0c89a')]):
        s += pill(ox + 200 + i * 92, oy + 26, 84, t, c, 22, 11)
    s += g(ox + W - 110, oy + 40, 0.5, glanztropfen()) + text(ox + W - 96, oy + 43, '128', 12)
    rows = [(icon_potion, ('e05a7a',), 'Heiltrank', 'stellt 5 Leben her', '15'),
            (icon_boots, (), 'Sandstiefel', 'kein Einsinken in Treibsand', '80'),
            (icon_amulet, (), 'Glücksanhänger', 'mehr Beute aus Truhen', '120'),
            (icon_potion, ('5aaee8',), 'Tautrank', 'kurz schneller hooken', '25'),
            (icon_hat, (), 'Strohhut', '+1 Leben', '200')]
    for i, (ic, a, name, note, price) in enumerate(rows):
        y = oy + 62 + i * 46
        sel = i == 1
        s += f'<rect x="{ox + 196}" y="{y}" width="{W - 228}" height="40" rx="12" fill="{"#f2c14e" if sel else "#ffffff"}" fill-opacity="{0.35 if sel else 1}" stroke="#e7dcc8"/>'
        s += ic(ox + 218, y + 20, *a)
        s += text(ox + 242, y + 17, name, 12)
        s += text(ox + 242, y + 32, note, 9, DIM)
        too_much = int(price) > 128
        s += text(ox + W - 62, y + 25, price, 12, '#d94a4a' if too_much else TEXT, 'end', 'bold')
        s += g(ox + W - 50, y + 26, 0.4, glanztropfen())
    s += pill(ox + W - 132, oy + H - 50, 100, 'Kaufen · 80', '6cbf4a', 22, 11)
    return s


def sheet():
    SW = 1400
    gap, top = 26, 110
    rows = [('Spiel: HUD, Aufgabe, Zuruf als Sprechblase', screen_game, 'Gespräch: Textfeld mit Bild und Auswahl (E-222)', screen_dialog),
            ('Abenteuer-Menü: Inventar und Ausrüstung', screen_inventory, 'Abenteuer-Menü: Fähigkeitenbaum (drei Zweige)', screen_skills),
            ('Abenteuer-Menü: Aufgaben', screen_quests, 'Händlerin Lotte (Schmied Klonk gleich aufgebaut)', screen_shop)]
    SH = top + len(rows) * (H + 60) + 20
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{SW}" height="{SH}" viewBox="0 0 {SW} {SH}" font-family="Inter">',
         f'<rect width="{SW}" height="{SH}" fill="#e8eef3"/>',
         text(40, 50, 'Abenteuer – Oberfläche (R2-M1, A1.0)', 28, TEXT, weight='bold'),
         text(40, 78, 'HUD wie im Mehrspieler (dunkle Leiste unten) plus Stufe, Erfahrung, Glanztropfen und Aufgabe; '
              'Menüs, Gespräche und Läden im Stil „hell & weich“.', 15, TEXT)]
    for r, (t1, f1, t2, f2) in enumerate(rows):
        y = top + r * (H + 60)
        for c, (t, f) in enumerate(((t1, f1), (t2, f2))):
            x = 30 + c * (W + gap + 4)
            o.append(text(x + 4, y + 18, f'{r * 2 + c + 1}  {t}', 15, TEXT))
            o.append(f'<rect x="{x - 1}" y="{y + 29}" width="{W + 2}" height="{H + 2}" rx="13" fill="#2b2b2b" fill-opacity="0.25"/>')
            o.append(f[0](x, y + 30) if isinstance(f, tuple) else f(x, y + 30))
    o.append('</svg>')
    return '\n'.join(o)


if __name__ == '__main__':
    os.makedirs('docs/release-2/design', exist_ok=True)
    open('docs/release-2/design/abenteuer-ui.svg', 'w').write(sheet())

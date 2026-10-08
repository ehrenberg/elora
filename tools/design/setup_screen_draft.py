"""Generates docs/release-2/design/setup-screen.svg – draft of the first-start setup screen
(RF-30, E-351) in the style of the main menu ("light & soft", E-123).

Usage: python3 tools/design/setup_screen_draft.py && cargo xtask svg-preview \
        docs/release-2/design/setup-screen.svg docs/release-2/design/setup-screen.png 1600

Two states: German with a name (button active), English with an empty name (button greyed,
hint below the field).
"""
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from elora_menu import BODY, EYES, B_TEXT, b_card, b_pill, background, elora, swatches, text

W, H = 440, 248


def screen(ox, oy, uid, de, name):
    t = {
        'title': ('Willkommen in Elora!', 'Welcome to Elora!'),
        'sub': ('Bevor es losgeht: Wie heißt du, und wie soll Elora aussehen?',
                'Before you start: what is your name, and how should Elora look?'),
        'lang': ('Sprache', 'Language'), 'name': ('Name', 'Name'),
        'body': ('Körper', 'Body'), 'feet': ('Füße', 'Feet'), 'eyes': ('Augen', 'Eyes'),
        'go': ("Los geht's!", "Let's go!"),
        'hint': ('Bitte gib einen Namen ein.', 'Please enter a name.'),
        'later': ('Alles lässt sich später unter Einstellungen ändern.',
                  'You can change everything later under Settings.'),
    }
    g = lambda k: t[k][0 if de else 1]
    s = background(ox, oy, uid, figures=False)
    s += b_card(ox + 16, oy + 14, W - 32, H - 28)
    s += text(ox + 34, oy + 40, g('title'), 16, B_TEXT, weight='bold')
    s += text(ox + 34, oy + 56, g('sub'), 8.5, '#8a7a66')
    # language switch, top right: changes the screen at once
    s += text(ox + 300, oy + 37, g('lang'), 8, '#8a7a66')
    s += b_pill(ox + 300, oy + 41, 48, 'Deutsch', 'f28c3a' if de else 'e0c89a', 16, 7.5)
    s += b_pill(ox + 352, oy + 41, 48, 'English', 'e0c89a' if de else 'f28c3a', 16, 7.5)
    # name
    s += text(ox + 34, oy + 80, g('name'), 9, '#8a7a66')
    border = '#e7dcc8' if name else '#e8685a'
    s += f'<rect x="{ox + 34}" y="{oy + 85}" width="150" height="20" rx="10" fill="#ffffff" stroke="{border}" stroke-width="1.5"/>'
    s += text(ox + 44, oy + 99, name or '|', 10, B_TEXT)
    if not name:
        s += text(ox + 34, oy + 116, g('hint'), 8, '#d94a4a')
    # colours with the shared palettes of the player settings
    for i, (k, cols, sel) in enumerate([('body', BODY, 0), ('feet', BODY, 1), ('eyes', EYES, 0)]):
        y = oy + 128 + i * 26
        s += text(ox + 34, y, g(k), 9, '#8a7a66')
        s += swatches(ox + 34, y + 4, cols, sel, 9, 2).replace(
            'stroke="#ffffff" stroke-width="2"', 'stroke="#3b3024" stroke-width="2.5"')
    # live preview
    s += f'<circle cx="{ox + 330}" cy="{oy + 130}" r="46" fill="#f2c14e" fill-opacity="0.18"/>'
    s += elora(ox + 330, oy + 176, 0.62, body='f2c14e', feet='f28c3a', eyes='2b2b2b')
    s += text(ox + 330, oy + 196, name or '…', 10, B_TEXT, 'middle', 'bold')
    # confirm
    s += b_pill(ox + 286, oy + 205, 96, g('go'), '6cbf4a' if name else 'b8b1a6', 22, 10)
    s += text(ox + 34, oy + 222, g('later'), 7.5, '#8a7a66')
    return s


def main():
    gap = 24
    total_w = 2 * W + 3 * gap
    total_h = H + 2 * gap + 18
    body = f'<rect width="{total_w}" height="{total_h}" fill="#f4efe6"/>'
    body += text(gap, 20, 'RF-30 Einrichtung beim ersten Start (E-351) – Entwurf', 11, B_TEXT, weight='bold')
    body += screen(gap, gap + 12, 'a', True, 'Basti')
    body += screen(2 * gap + W, gap + 12, 'b', False, '')
    svg = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total_w} {total_h}" '
           f'font-family="Inter, sans-serif">{body}</svg>\n')
    with open('docs/release-2/design/setup-screen.svg', 'w', encoding='utf-8') as f:
        f.write(svg)


if __name__ == '__main__':
    main()

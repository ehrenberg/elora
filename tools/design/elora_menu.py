"""Generates docs/archive/release-1/design/elora-menue.svg – three menu drafts (M7.0, E-123).

Usage: python3 tools/design/elora_menu.py && cargo xtask svg-preview \
        docs/archive/release-1/design/elora-menue.svg docs/archive/release-1/design/elora-menue.png 1400

Three screens per style (16:9): main menu, server browser, settings (player & skin),
all over the same calm background image (E-113).
"""
import re

OUT = '#2b2b2b'
W, H = 440, 248            # one screen on the sheet
BODY = ['f2c14e', 'f28c3a', 'e8685a', 'd94a4a', 'ef7fb0', 'a77be0', '6a78e0', '5aaee8',
        '3fc1b0', '7fd99a', '6cbf4a', 'b8d94a', 'e0c89a', 'a8744a', '9aa4ae', 'f0ece4']
EYES = ['2b2b2b', '2e3f86', '2f6b4a', '6b3a2a', '7a2a4a', '4a2a7a', '1f6470', '5a5a5a']
SERVERS = [('Eloras Wiese', 'ctf-test', 'CTF', '6/8', 24), ('Tropfen-Arena', 'sandbox', 'DM', '3/8', 41),
           ('Nachtschicht', 'sandbox', 'iDM', '8/8', 63), ('Team Blau', 'ctf-test', 'TDM', '2/12', 88),
           ('LAN-Party', 'sandbox', 'LMS', '0/8', 12)]


def elora(x, y, s, body='f2c14e', feet='d9a43a', eyes='2b2b2b', flip=False):
    src = open('assets/elora/elora.svg').read()
    inner = src[src.index('>', src.index('<svg')) + 1:src.rindex('</svg>')]
    inner = re.sub(r'<!--.*?-->', '', inner, flags=re.S)

    def recolor(m):
        tag = m.group(0)
        key = re.search(r'id="tint-(\d)(-l(\d+))?', tag)
        if not key:
            return tag
        c = {'1': eyes, '2': body, '3': feet}[key.group(1)]
        if key.group(3):
            v = [int(c[i:i + 2], 16) for i in (0, 2, 4)]
            c = ''.join(f'{round(x + (255 - x) * 0.45):02x}' for x in v)
        if 'fill="none"' in tag:
            return re.sub(r'stroke="#\w+"', f'stroke="#{c}"', tag)
        return re.sub(r'fill="#\w+"', f'fill="#{c}"', tag)
    inner = re.sub(r'<(ellipse|path|circle)[^>]*>', recolor, inner)
    sx = -s if flip else s
    return f'<g transform="translate({x},{y}) scale({sx},{s})">{inner}</g>'


def background(ox, oy, uid, figures=True):
    """Calm image: sky gradient, gentle hills, clouds, large Elora (main menu only)."""
    figs = (elora(ox + 360, oy + H - 22, 0.55)
            + elora(ox + 300, oy + H - 12, 0.3, body='5aaee8', feet='6a78e0', eyes='2e3f86', flip=True)) if figures else ''
    return f'''<defs><linearGradient id="sky{uid}" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#a9cde8"/><stop offset="1" stop-color="#e8f1f7"/></linearGradient>
      <clipPath id="clip{uid}"><rect x="{ox}" y="{oy}" width="{W}" height="{H}" rx="10"/></clipPath></defs>
    <g clip-path="url(#clip{uid})">
      <rect x="{ox}" y="{oy}" width="{W}" height="{H}" fill="url(#sky{uid})"/>
      <ellipse cx="{ox + 230}" cy="{oy + 58}" rx="34" ry="11" fill="#ffffff" opacity="0.8"/>
      <ellipse cx="{ox + 330}" cy="{oy + 36}" rx="44" ry="12" fill="#ffffff" opacity="0.7"/>
      <ellipse cx="{ox + 120}" cy="{oy + H + 40}" rx="200" ry="90" fill="#8fbf7a"/>
      <ellipse cx="{ox + 380}" cy="{oy + H + 55}" rx="190" ry="95" fill="#7aae6a"/>
      {figs}
    </g>'''


def text(x, y, t, size=11, color='#ffffff', anchor='start', weight='normal'):
    t = t.replace('&', '&amp;')
    return f'<text x="{x}" y="{y}" font-size="{size}" fill="{color}" text-anchor="{anchor}" font-weight="{weight}">{t}</text>'


# ── Style A: dark cards (HUD style) ─────────────────────────────────────
A_PANEL = 'fill="#1e2a36" fill-opacity="0.72"'


def a_main(ox, oy):
    s = text(ox + 36, oy + 58, 'Elora', 34, '#1e2a36', weight='bold')
    s += f'<rect x="{ox + 28}" y="{oy + 74}" width="150" height="150" rx="14" {A_PANEL}/>'
    for i, (t, hl) in enumerate([('Spielen', True), ('Training', False), ('Server erstellen', False),
                                 ('Einstellungen', False), ('Beenden', False)]):
        y = oy + 84 + i * 27
        if hl:
            s += f'<rect x="{ox + 36}" y="{y}" width="134" height="22" rx="8" fill="#ffffff" fill-opacity="0.22"/>'
        s += text(ox + 48, y + 15, t, 12)
    return s


def a_browser(ox, oy):
    s = f'<rect x="{ox + 16}" y="{oy + 14}" width="{W - 32}" height="{H - 28}" rx="14" {A_PANEL}/>'
    for i, t in enumerate(['Internet', 'LAN', 'Favoriten']):
        x = ox + 28 + i * 70
        if i == 0:
            s += f'<rect x="{x - 6}" y="{oy + 22}" width="64" height="20" rx="8" fill="#ffffff" fill-opacity="0.22"/>'
        s += text(x, oy + 36, t, 11)
    cols = [(28, 'Name'), (160, 'Karte'), (240, 'Modus'), (300, 'Spieler'), (370, 'Ping')]
    for x, t in cols:
        s += text(ox + x, oy + 60, t, 9, '#ffffff" fill-opacity="0.6')
    for r, (n, m, g, p, ping) in enumerate(SERVERS):
        y = oy + 66 + r * 22
        if r == 1:
            s += f'<rect x="{ox + 22}" y="{y}" width="{W - 44}" height="20" rx="6" fill="#ffffff" fill-opacity="0.18"/>'
        elif r % 2 == 0:
            s += f'<rect x="{ox + 22}" y="{y}" width="{W - 44}" height="20" rx="6" fill="#ffffff" fill-opacity="0.05"/>'
        for (x, _), v in zip(cols, [n, m, g, p, f'{ping} ms']):
            s += text(ox + x, y + 14, v, 10)
    s += f'<rect x="{ox + 28}" y="{oy + H - 48}" width="220" height="22" rx="8" fill="#ffffff" fill-opacity="0.12"/>'
    s += text(ox + 36, oy + H - 33, 'Adresse: 127.0.0.1:8303', 10, '#ffffff" fill-opacity="0.7')
    s += f'<rect x="{ox + W - 118}" y="{oy + H - 48}" width="90" height="22" rx="8" fill="#6cbf4a"/>'
    s += text(ox + W - 73, oy + H - 33, 'Verbinden', 11, anchor='middle', weight='bold')
    return s


def swatches(x, y, colors, sel, size=12, gap=3):
    s = ''
    for i, c in enumerate(colors):
        cx = x + i * (size + gap)
        stroke = '#ffffff' if i == sel else OUT
        sw = 2 if i == sel else 1
        s += f'<rect x="{cx}" y="{y}" width="{size}" height="{size}" rx="3" fill="#{c}" stroke="{stroke}" stroke-width="{sw}"/>'
    return s


def a_settings(ox, oy):
    s = f'<rect x="{ox + 16}" y="{oy + 14}" width="{W - 32}" height="{H - 28}" rx="14" {A_PANEL}/>'
    for i, t in enumerate(['Spieler', 'Steuerung', 'Grafik', 'Ton', 'Sprache']):
        x = ox + 28 + i * 70
        if i == 0:
            s += f'<rect x="{x - 6}" y="{oy + 22}" width="62" height="20" rx="8" fill="#ffffff" fill-opacity="0.22"/>'
        s += text(x, oy + 36, t, 11)
    s += text(ox + 28, oy + 66, 'Name', 10, '#ffffff" fill-opacity="0.7')
    s += f'<rect x="{ox + 28}" y="{oy + 72}" width="180" height="22" rx="8" fill="#ffffff" fill-opacity="0.12"/>'
    s += text(ox + 36, oy + 87, 'Elora', 11)
    for i, (lbl, cols, sel) in enumerate([('Körper', BODY, 7), ('Füße', BODY, 6), ('Augen', EYES, 1)]):
        y = oy + 112 + i * 34
        s += text(ox + 28, y, lbl, 10, '#ffffff" fill-opacity="0.7')
        s += swatches(ox + 28, y + 5, cols, sel, 11, 2)
    s += f'<rect x="{ox + 256}" y="{oy + 58}" width="150" height="160" rx="12" fill="#ffffff" fill-opacity="0.08"/>'
    s += elora(ox + 331, oy + 200, 0.95, body='5aaee8', feet='6a78e0', eyes='2e3f86')
    return s


# ── Style B: light & soft ───────────────────────────────────────────────
B_PANEL = 'fill="#fffaf0" stroke="#e7dcc8" stroke-width="1.5"'
B_SHADOW = 'fill="#1e2a36" fill-opacity="0.12"'
B_TEXT = '#3b3024'


def b_card(x, y, w, h):
    return (f'<rect x="{x + 3}" y="{y + 5}" width="{w}" height="{h}" rx="18" {B_SHADOW}/>'
            f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="18" {B_PANEL}/>')


def b_pill(x, y, w, t, color, h=24, size=12):
    return (f'<rect x="{x}" y="{y + 3}" width="{w}" height="{h}" rx="{h / 2}" fill="#{color}" fill-opacity="0.45"/>'
            f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{h / 2}" fill="#{color}" stroke="{OUT}" stroke-width="1.5"/>'
            + text(x + w / 2, y + h / 2 + size * 0.36, t, size, '#ffffff', 'middle', 'bold'))


def b_main(ox, oy):
    s = b_card(ox + 26, oy + 20, 170, 208)
    s += text(ox + 111, oy + 58, 'Elora', 30, B_TEXT, 'middle', 'bold')
    for i, (t, c) in enumerate([('Spielen', '6cbf4a'), ('Training', '5aaee8'), ('Server erstellen', 'a77be0'),
                                ('Einstellungen', 'f28c3a'), ('Beenden', '9aa4ae')]):
        s += b_pill(ox + 44, oy + 74 + i * 29, 134, t, c)
    return s


def b_browser(ox, oy):
    s = b_card(ox + 16, oy + 14, W - 32, H - 28)
    for i, (t, c) in enumerate([('Internet', '5aaee8'), ('LAN', 'e0c89a'), ('Favoriten', 'e0c89a')]):
        s += b_pill(ox + 28 + i * 76, oy + 24, 68, t, c, 20, 10)
    cols = [(28, 'Name'), (160, 'Karte'), (240, 'Modus'), (300, 'Spieler'), (370, 'Ping')]
    for x, t in cols:
        s += text(ox + x, oy + 62, t, 9, '#8a7a66')
    for r, (n, m, g, p, ping) in enumerate(SERVERS):
        y = oy + 68 + r * 22
        if r == 1:
            s += f'<rect x="{ox + 22}" y="{y}" width="{W - 44}" height="20" rx="10" fill="#f2c14e" fill-opacity="0.35"/>'
        for (x, _), v in zip(cols, [n, m, g, p, f'{ping} ms']):
            s += text(ox + x, y + 14, v, 10, B_TEXT)
    s += f'<rect x="{ox + 28}" y="{oy + H - 50}" width="220" height="24" rx="12" fill="#ffffff" stroke="#e7dcc8"/>'
    s += text(ox + 40, oy + H - 34, 'Adresse: 127.0.0.1:8303', 10, '#8a7a66')
    s += b_pill(ox + W - 120, oy + H - 50, 92, 'Verbinden', '6cbf4a')
    return s


def b_settings(ox, oy):
    s = b_card(ox + 16, oy + 14, W - 32, H - 28)
    for i, t in enumerate(['Spieler', 'Steuerung', 'Grafik', 'Ton', 'Sprache']):
        s += b_pill(ox + 26 + i * 72, oy + 24, 66, t, 'f28c3a' if i == 0 else 'e0c89a', 20, 10)
    s += text(ox + 28, oy + 66, 'Name', 10, '#8a7a66')
    s += f'<rect x="{ox + 28}" y="{oy + 72}" width="180" height="22" rx="11" fill="#ffffff" stroke="#e7dcc8"/>'
    s += text(ox + 38, oy + 87, 'Elora', 11, B_TEXT)
    for i, (lbl, cols, sel) in enumerate([('Körper', BODY, 7), ('Füße', BODY, 6), ('Augen', EYES, 1)]):
        y = oy + 112 + i * 34
        s += text(ox + 28, y, lbl, 10, '#8a7a66')
        s += swatches(ox + 28, y + 5, cols, sel, 11, 2).replace('stroke="#ffffff" stroke-width="2"', 'stroke="#3b3024" stroke-width="2.5"')
    s += f'<circle cx="{ox + 331}" cy="{oy + 140}" r="72" fill="#f2c14e" fill-opacity="0.18"/>'
    s += elora(ox + 331, oy + 200, 0.95, body='5aaee8', feet='6a78e0', eyes='2e3f86')
    return s


# ── Style C: bar at the top ─────────────────────────────────────────────
C_BAR = 'fill="#1e2a36" fill-opacity="0.85"'


def c_bar(ox, oy, active):
    s = f'<rect x="{ox}" y="{oy}" width="{W}" height="32" {C_BAR}/>'
    s += text(ox + 14, oy + 22, 'Elora', 15, '#f2c14e', weight='bold')
    x = ox + 78
    for i, t in enumerate(['Spielen', 'Training', 'Server erstellen', 'Einstellungen', 'Beenden']):
        if i == active:
            s += f'<rect x="{x - 6}" y="{oy + 28}" width="{len(t) * 5.3 + 12}" height="4" rx="2" fill="#f2c14e"/>'
        s += text(x, oy + 20, t, 10, '#ffffff' if i == active else '#ffffff" fill-opacity="0.65')
        x += len(t) * 5.0 + 16
    return s


def c_main(ox, oy):
    s = c_bar(ox, oy, 0)
    s += f'<rect x="{ox + 22}" y="{oy + 140}" width="200" height="84" rx="12" {C_BAR}/>'
    s += text(ox + 36, oy + 162, 'Schnell spielen', 13, weight='bold')
    s += text(ox + 36, oy + 180, 'Tropfen-Arena · DM · 3/8', 10, '#ffffff" fill-opacity="0.7')
    s += f'<rect x="{ox + 36}" y="{oy + 190}" width="80" height="22" rx="8" fill="#6cbf4a"/>'
    s += text(ox + 76, oy + 205, 'Los!', 11, anchor='middle', weight='bold')
    s += text(ox + 26, oy + 90, 'Willkommen zurück,', 13, '#1e2a36')
    s += text(ox + 26, oy + 116, 'Elora!', 24, '#1e2a36', weight='bold')
    return s


def c_browser(ox, oy):
    s = c_bar(ox, oy, 0)
    s += f'<rect x="{ox + 14}" y="{oy + 42}" width="{W - 28}" height="{H - 54}" rx="12" {C_BAR}/>'
    for i, t in enumerate(['Internet', 'LAN', 'Favoriten']):
        x = ox + 26 + i * 70
        s += text(x, oy + 60, t, 11, '#f2c14e' if i == 0 else '#ffffff" fill-opacity="0.65', weight='bold' if i == 0 else 'normal')
    cols = [(26, 'Name'), (158, 'Karte'), (238, 'Modus'), (298, 'Spieler'), (368, 'Ping')]
    for x, t in cols:
        s += text(ox + x, oy + 80, t, 9, '#ffffff" fill-opacity="0.55')
    for r, (n, m, g, p, ping) in enumerate(SERVERS):
        y = oy + 86 + r * 22
        if r == 1:
            s += f'<rect x="{ox + 20}" y="{y}" width="{W - 40}" height="20" rx="4" fill="#f2c14e" fill-opacity="0.25"/>'
        for (x, _), v in zip(cols, [n, m, g, p, f'{ping} ms']):
            s += text(ox + x, y + 14, v, 10)
    s += f'<rect x="{ox + W - 112}" y="{oy + H - 38}" width="86" height="22" rx="4" fill="#6cbf4a"/>'
    s += text(ox + W - 69, oy + H - 23, 'Verbinden', 11, anchor='middle', weight='bold')
    return s


def c_settings(ox, oy):
    s = c_bar(ox, oy, 3)
    s += f'<rect x="{ox + 14}" y="{oy + 42}" width="110" height="{H - 54}" rx="12" {C_BAR}/>'
    for i, t in enumerate(['Spieler', 'Steuerung', 'Grafik', 'Ton', 'Sprache']):
        y = oy + 64 + i * 24
        if i == 0:
            s += f'<rect x="{ox + 20}" y="{y - 14}" width="98" height="20" rx="4" fill="#f2c14e" fill-opacity="0.25"/>'
        s += text(ox + 30, y, t, 11)
    s += f'<rect x="{ox + 132}" y="{oy + 42}" width="{W - 146}" height="{H - 54}" rx="12" {C_BAR}/>'
    s += text(ox + 144, oy + 64, 'Name', 10, '#ffffff" fill-opacity="0.7')
    s += f'<rect x="{ox + 144}" y="{oy + 70}" width="150" height="22" rx="4" fill="#ffffff" fill-opacity="0.12"/>'
    s += text(ox + 152, oy + 85, 'Elora', 11)
    for i, (lbl, cols, sel) in enumerate([('Körper', BODY, 7), ('Füße', BODY, 6), ('Augen', EYES, 1)]):
        y = oy + 110 + i * 34
        s += text(ox + 144, y, lbl, 10, '#ffffff" fill-opacity="0.7')
        s += swatches(ox + 144, y + 5, cols, sel, 10, 2)
    s += elora(ox + 375, oy + 206, 0.7, body='5aaee8', feet='6a78e0', eyes='2e3f86')
    return s


STYLES = [
    ('A – Dunkle Karten', 'wie das HUD: dunkle, halbtransparente Flächen; Menü links als Liste', [a_main, a_browser, a_settings]),
    ('B – Hell & weich', 'cremefarbene Karten mit weichem Schatten, farbige Pillen-Knöpfe aus der Palette', [b_main, b_browser, b_settings]),
    ('C – Leiste oben', 'Reiterleiste oben, Inhalt darunter; „Schnell spielen“ auf der Startseite', [c_main, c_browser, c_settings]),
]


def sheet():
    SW = 30 + 3 * (W + 20) + 10
    SH = 90 + 3 * (H + 60)
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{SW}" height="{SH}" viewBox="0 0 {SW} {SH}" font-family="Inter">',
           f'<rect width="{SW}" height="{SH}" fill="#d8e3ec"/>',
           text(30, 44, 'Menü – drei Entwürfe (M7.0, E-123)', 28, '#1e2a36', weight='bold'),
           text(30, 70, 'Je Stil: Hauptmenü · Server-Browser · Einstellungen (Spieler & Skin). Hintergrund: ruhiges Bild (E-113).', 15, '#1e2a36')]
    uid = 0
    for r, (title, desc, screens) in enumerate(STYLES):
        y = 90 + r * (H + 60)
        out.append(text(30, y + 22, title, 20, '#1e2a36', weight='bold'))
        out.append(text(30 + len(title) * 11.5 + 16, y + 22, desc, 13, '#1e2a36'))
        for c, fn in enumerate(screens):
            ox, oy = 30 + c * (W + 20), y + 34
            uid += 1
            out.append(background(ox, oy, uid, figures=c == 0))
            out.append(fn(ox, oy))
    out.append('</svg>')
    return '\n'.join(out)


# ── Chosen (E-125): layout of C in the colours and shapes of B ──────────
def d_bar(ox, oy, active):
    s = (f'<rect x="{ox}" y="{oy + 4}" width="{W}" height="36" {B_SHADOW}/>'
         f'<rect x="{ox}" y="{oy}" width="{W}" height="36" fill="#fffaf0" stroke="#e7dcc8" stroke-width="1.5"/>')
    s += text(ox + 14, oy + 24, 'Elora', 16, '#e8a53a', weight='bold')
    x = ox + 72
    colors = ['6cbf4a', '5aaee8', 'a77be0', 'f28c3a', '9aa4ae']
    for i, t in enumerate(['Spielen', 'Training', 'Server erstellen', 'Einstellungen', 'Beenden']):
        w = len(t) * 5.2 + 16
        if i == active:
            s += b_pill(x, oy + 7, w, t, colors[i], 22, 10)
        else:
            s += text(x + w / 2, oy + 22, t, 10, B_TEXT, 'middle')
        x += w + 4
    return s


def d_main(ox, oy):
    s = d_bar(ox, oy, 0)
    s += text(ox + 26, oy + 86, 'Willkommen zurück,', 13, B_TEXT)
    s += text(ox + 26, oy + 112, 'Elora!', 24, B_TEXT, weight='bold')
    s += b_card(ox + 22, oy + 132, 200, 92)
    s += text(ox + 38, oy + 156, 'Schnell spielen', 13, B_TEXT, weight='bold')
    s += text(ox + 38, oy + 174, 'Tropfen-Arena · DM · 3/8', 10, '#8a7a66')
    s += b_pill(ox + 38, oy + 186, 80, 'Los!', '6cbf4a')
    return s


def d_browser(ox, oy):
    s = d_bar(ox, oy, 0)
    s += b_card(ox + 14, oy + 46, W - 28, H - 58)
    for i, (t, c) in enumerate([('Internet', '5aaee8'), ('LAN', 'e0c89a'), ('Favoriten', 'e0c89a')]):
        s += b_pill(ox + 26 + i * 76, oy + 54, 68, t, c, 20, 10)
    cols = [(26, 'Name'), (158, 'Karte'), (238, 'Modus'), (298, 'Spieler'), (368, 'Ping')]
    for x, t in cols:
        s += text(ox + x, oy + 90, t, 9, '#8a7a66')
    for r, (n, m, g, p, ping) in enumerate(SERVERS[:4]):
        y = oy + 96 + r * 21
        if r == 1:
            s += f'<rect x="{ox + 20}" y="{y}" width="{W - 40}" height="19" rx="9.5" fill="#f2c14e" fill-opacity="0.35"/>'
        for (x, _), v in zip(cols, [n, m, g, p, f'{ping} ms']):
            s += text(ox + x, y + 13, v, 10, B_TEXT)
    s += b_pill(ox + W - 118, oy + H - 40, 92, 'Verbinden', '6cbf4a')
    return s


def d_settings(ox, oy):
    s = d_bar(ox, oy, 3)
    s += b_card(ox + 14, oy + 46, 108, H - 58)
    for i, t in enumerate(['Spieler', 'Steuerung', 'Grafik', 'Ton', 'Sprache']):
        y = oy + 58 + i * 28
        if i == 0:
            s += b_pill(ox + 22, y, 92, t, 'f28c3a', 22, 10)
        else:
            s += text(ox + 68, y + 15, t, 10, B_TEXT, 'middle')
    s += b_card(ox + 130, oy + 46, W - 144, H - 58)
    s += text(ox + 144, oy + 68, 'Name', 10, '#8a7a66')
    s += f'<rect x="{ox + 144}" y="{oy + 74}" width="140" height="22" rx="11" fill="#ffffff" stroke="#e7dcc8"/>'
    s += text(ox + 154, oy + 89, 'Elora', 11, B_TEXT)
    for i, (lbl, cols, sel) in enumerate([('Körper', BODY, 7), ('Füße', BODY, 6), ('Augen', EYES, 1)]):
        y = oy + 114 + i * 34
        s += text(ox + 144, y, lbl, 10, '#8a7a66')
        s += swatches(ox + 144, y + 5, cols, sel, 10, 2).replace('stroke="#ffffff" stroke-width="2"', 'stroke="#3b3024" stroke-width="2.5"')
    s += f'<circle cx="{ox + 378}" cy="{oy + 160}" r="44" fill="#f2c14e" fill-opacity="0.2"/>'
    s += elora(ox + 378, oy + 204, 0.62, body='5aaee8', feet='6a78e0', eyes='2e3f86')
    return s


def sheet_chosen():
    SW = 30 + 3 * (W + 20) + 10
    SH = 90 + H + 60
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{SW}" height="{SH}" viewBox="0 0 {SW} {SH}" font-family="Inter">',
           f'<rect width="{SW}" height="{SH}" fill="#d8e3ec"/>',
           text(30, 44, 'Menü – gewählter Stil (E-125)', 28, '#1e2a36', weight='bold'),
           text(30, 70, 'Aufbau von C „Leiste oben“ in den Farben und Formen von B „Hell &amp; weich“.'.replace('&amp;', '&'), 15, '#1e2a36')]
    for c, fn in enumerate([d_main, d_browser, d_settings]):
        ox, oy = 30 + c * (W + 20), 100
        out.append(background(ox, oy, 100 + c, figures=c == 0))
        out.append(fn(ox, oy))
    out.append('</svg>')
    return '\n'.join(out)


if __name__ == '__main__':
    with open('docs/archive/release-1/design/elora-menue.svg', 'w') as fh:
        fh.write(sheet())
    with open('docs/archive/release-1/design/elora-menue-gewaehlt.svg', 'w') as fh:
        fh.write(sheet_chosen())

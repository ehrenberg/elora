"""Drafts for chapter 1 "Blütenwiesen" (R2-M2.1, step M2.1.0, E-297 to E-302).

Usage: python3 tools/design/chapter1_drafts.py && cargo xtask svg-preview \
        docs/release-2/design/chapter1-drafts.svg docs/release-2/design/chapter1-drafts.png 1600

Style like the figures (adventure_figures.py) and buildings (tauwinkel_buildings.py):
drop creatures, dark outline, flat colours; enemies round and colourful, looking grumpy, no blood.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from adventure_figures import DIM, OUT, TEXT, angry_eye, drop, g, star, text  # noqa: E402

HONEY = '#f2b33a'
HONEY_DARK = '#c98a1e'
STRIPE = '#3a3030'
WING = '#dff1fb'
PINK = '#ef7fb0'
LEAF = '#6cbf4a'
STEM = '#4f9a3a'


def st(w=3.5):
    return f'stroke="{OUT}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'


# ── Beekeeper Wabe ──────────────────────────────────────────────────────

def wabe():
    """Beekeeper: amber, beekeeper hat with turned-up veil, smoker, honey jar."""
    body, feet = 'e8a84a', 'b8742a'
    # smoker held at the back
    s = '<path d="M -44,-40 L -60,-30" stroke="#2b2b2b" stroke-width="4"/>'
    s += f'<rect x="-84" y="-44" width="26" height="36" rx="6" fill="#c8ced6" {st()}/>'
    s += f'<path d="M -82,-44 Q -71,-62 -60,-44 Z" fill="#a8b0ba" {st(3)}/>'
    s += f'<path d="M -74,-62 q -6,-10 2,-16 q 8,-6 2,-16" fill="none" stroke="#b8b2aa" stroke-width="5" stroke-linecap="round"/>'
    s += drop(body, feet)
    # bib apron with honeycomb pattern
    s += f'<path d="M -38,-34 Q 4,-22 46,-34 L 44,-12 Q 4,-2 -36,-12 Z" fill="#f4ead4" {st(3.5)}/>'
    for (x, y) in ((-16, -22), (0, -19), (16, -22), (32, -24), (-8, -12), (8, -11), (24, -13)):
        pts = ' '.join(f'{x + 5 * math.cos(math.radians(a)):.1f},{y + 5 * math.sin(math.radians(a)):.1f}' for a in range(0, 360, 60))
        s += f'<polygon points="{pts}" fill="{HONEY}" stroke="{HONEY_DARK}" stroke-width="1.4"/>'
    # beekeeper hat with veil
    s += f'<ellipse cx="10" cy="-104" rx="52" ry="10" fill="#f0ece4" {st(4)}/>'
    s += f'<path d="M -18,-108 Q -16,-142 12,-144 Q 40,-142 40,-108 Z" fill="#f0ece4" {st(4)}/>'
    s += '<path d="M -42,-100 Q -50,-80 -44,-70 M 62,-100 Q 70,-80 62,-70" fill="none" stroke="#9aa6b0" stroke-width="3" stroke-dasharray="4 3"/>'
    s += '<path d="M -17,-116 Q 12,-110 39,-116" fill="none" stroke="' + HONEY_DARK + '" stroke-width="5"/>'
    # honey jar held in front
    s += f'<rect x="44" y="-52" width="24" height="28" rx="5" fill="{HONEY}" fill-opacity="0.85" {st(3)}/>'
    s += f'<rect x="42" y="-58" width="28" height="8" rx="2" fill="#e8685a" {st(2.5)}/>'
    s += '<path d="M 50,-44 q 4,-2 6,2" fill="none" stroke="#fff6d8" stroke-width="2.5" stroke-linecap="round"/>'
    # a bee circles around the hat
    s += bee(scale=0.38, x=58, y=-140)
    return s


# ── Bees ────────────────────────────────────────────────────────────────

def bee(scale=1.0, x=0, y=0, angry=False, flip=False):
    """Small bee: striped, two wings; `angry` = confused enemy (spiral eyes)."""
    s = f'<ellipse cx="-6" cy="-30" rx="14" ry="20" fill="{WING}" fill-opacity="0.9" {st(3)} transform="rotate(-25 -6 -30)"/>'
    s += f'<ellipse cx="10" cy="-32" rx="12" ry="18" fill="{WING}" fill-opacity="0.9" {st(3)} transform="rotate(20 10 -32)"/>'
    s += f'<ellipse cx="0" cy="-6" rx="26" ry="20" fill="{HONEY}" {st(4)}/>'
    s += f'<path d="M -10,-24 Q -14,-6 -10,12 M 4,-25 Q 0,-6 4,13" fill="none" stroke="{STRIPE}" stroke-width="7"/>'
    s += f'<path d="M -26,-6 L -36,-4 L -26,0" fill="{STRIPE}" {st(2)}/>'
    if angry:
        for ex in (14, 24):
            s += f'<circle cx="{ex}" cy="-10" r="5.5" fill="#ffffff" {st(2)}/>'
            s += f'<path d="M {ex},{-10} m -3,0 a 3,3 0 1 1 3,3 a 1.6,1.6 0 1 1 -1.6,-1.6" fill="none" stroke="{OUT}" stroke-width="1.6"/>'
        s += f'<path d="M 12,0 q 4,-3 8,0 q 4,3 8,0" fill="none" stroke="{OUT}" stroke-width="2.4"/>'
    else:
        s += f'<circle cx="18" cy="-10" r="4" fill="{OUT}"/><circle cx="17" cy="-11.5" r="1.4" fill="#ffffff"/>'
        s += f'<path d="M 14,0 Q 19,4 24,-1" fill="none" stroke="{OUT}" stroke-width="2.4"/>'
        s += '<circle cx="24" cy="-2" r="3" fill="#e8685a" opacity="0.4"/>'
    s += f'<path d="M 12,-24 Q 14,-38 22,-40 M 18,-22 Q 24,-34 32,-34" fill="none" stroke="{OUT}" stroke-width="2.4"/>'
    s += f'<circle cx="22" cy="-40" r="2.6" fill="{OUT}"/><circle cx="32" cy="-34" r="2.6" fill="{OUT}"/>'
    return g(x, y, scale, s, flip)


# ── Grumble bumblebee ───────────────────────────────────────────────────

def bumblebee(pose='flug'):
    """Warden of the blossom spring: large, furry bumblebee. Poses: flug, sturz, benommen, ruhig."""
    calm = pose == 'ruhig'
    stunned = pose == 'benommen'
    s = ''
    if pose == 'sturz':
        for k, (dx, l) in enumerate(((-70, 60), (-50, 80), (-30, 60))):
            s += f'<path d="M {dx},-150 l {-20},{-l}" stroke="#ffffff" stroke-width="{6 - k}" stroke-linecap="round" opacity="0.8"/>'
    wings = not stunned
    if wings:
        flap = -10 if pose == 'flug' else 25
        s += f'<ellipse cx="-30" cy="-118" rx="40" ry="62" fill="{WING}" fill-opacity="0.85" {st(4)} transform="rotate({-35 + flap} -30 -118)"/>'
        s += f'<ellipse cx="12" cy="-124" rx="34" ry="54" fill="{WING}" fill-opacity="0.85" {st(4)} transform="rotate({20 - flap} 12 -124)"/>'
        s += '<path d="M -40,-150 q 10,20 4,40 M 6,-152 q 6,18 0,36" fill="none" stroke="#b6d4e4" stroke-width="2.5"/>'
    # body: furry (jagged edge)
    cx, cy, rx, ry = 0, -70 if not stunned else -52, 82, 64 if not stunned else 52
    pts = []
    for i in range(48):
        a = 2 * math.pi * i / 48
        r = 1 + (0.06 if i % 2 else 0)
        pts.append(f'{cx + rx * r * math.cos(a):.1f},{cy + ry * r * math.sin(a):.1f}')
    s += f'<polygon points="{" ".join(pts)}" fill="{HONEY}" {st(5)}/>'
    for dx in (-36, 0, 34):
        top, bot = cy - ry * 0.92, cy + ry * 0.92
        s += f'<path d="M {dx - 6},{top + 6} Q {dx - 16},{cy} {dx - 6},{bot - 6} L {dx + 12},{bot - 4} Q {dx + 2},{cy} {dx + 12},{top + 4} Z" fill="{STRIPE}"/>'
    s += f'<ellipse cx="{cx - 30}" cy="{cy - ry * 0.55}" rx="22" ry="9" fill="#ffffff" opacity="0.25"/>'
    # stinger (blunt, child-friendly)
    s += f'<path d="M {cx - rx},{cy + 6} L {cx - rx - 22},{cy + 14} L {cx - rx + 2},{cy + 22} Z" fill="{STRIPE}" {st(3)}/>'
    # head
    hx, hy = cx + 70, cy - 10
    s += f'<circle cx="{hx}" cy="{hy}" r="40" fill="{STRIPE}" {st(5)}/>'
    s += f'<path d="M {hx - 4},{hy - 36} Q {hx - 10},{hy - 76} {hx + 10},{hy - 84} M {hx + 16},{hy - 34} Q {hx + 24},{hy - 70} {hx + 44},{hy - 72}" fill="none" stroke="{OUT}" stroke-width="5" stroke-linecap="round"/>'
    s += f'<circle cx="{hx + 10}" cy="{hy - 84}" r="7" fill="{HONEY}" {st(3)}/><circle cx="{hx + 44}" cy="{hy - 72}" r="7" fill="{HONEY}" {st(3)}/>'
    if stunned:
        for ex in (hx + 4, hx + 26):
            s += f'<circle cx="{ex}" cy="{hy - 4}" r="11" fill="#ffffff" {st(3)}/>'
            s += f'<path d="M {ex - 5},{hy - 9} L {ex + 5},{hy + 1} M {ex + 5},{hy - 9} L {ex - 5},{hy + 1}" stroke="{OUT}" stroke-width="3.5" stroke-linecap="round"/>'
        s += f'<path d="M {hx + 2},{hy + 20} q 6,-6 12,0 q 6,6 12,0" fill="none" stroke="#ffffff" stroke-width="3"/>'
        for k in range(3):
            a = math.radians(k * 120 + 20)
            s += star(hx + 14 + 38 * math.cos(a), hy - 52 + 10 * math.sin(a), 9, '#f2c14e')
    elif calm:
        for ex in (hx + 4, hx + 26):
            s += f'<path d="M {ex - 9},{hy - 2} Q {ex},{hy - 12} {ex + 9},{hy - 2}" fill="none" stroke="#ffffff" stroke-width="4" stroke-linecap="round"/>'
        s += f'<path d="M {hx + 4},{hy + 14} Q {hx + 15},{hy + 24} {hx + 26},{hy + 14}" fill="none" stroke="#ffffff" stroke-width="3.5" stroke-linecap="round"/>'
        s += f'<circle cx="{hx - 8}" cy="{hy + 8}" r="6" fill="#e8685a" opacity="0.5"/>'
    else:
        s += angry_eye(hx + 4, hy - 4, 11, 1) + angry_eye(hx + 28, hy - 4, 11, 1)
        s += f'<path d="M {hx + 2},{hy + 20} Q {hx + 16},{hy + 12} {hx + 30},{hy + 20}" fill="none" stroke="#ffffff" stroke-width="3.5" stroke-linecap="round"/>'
    # little legs
    if stunned:
        for x in (-30, 0, 30):
            s += f'<path d="M {x},{cy + ry - 4} q 4,10 10,8" fill="none" stroke="{OUT}" stroke-width="6" stroke-linecap="round"/>'
    else:
        for x in (-24, 4, 32):
            s += f'<path d="M {x},{cy + ry - 6} q -4,22 6,30" fill="none" stroke="{OUT}" stroke-width="6" stroke-linecap="round"/>'
    rot = {'sturz': 35, 'benommen': 0}.get(pose, 0)
    return f'<g transform="rotate({rot})">{s}</g>'


def pollen(x, y, r=9):
    s = f'<circle cx="{x}" cy="{y}" r="{r + 5}" fill="#f2c14e" opacity="0.35"/><circle cx="{x}" cy="{y}" r="{r}" fill="#f8dd6e" stroke="{OUT}" stroke-width="2.5"/>'
    return s


# ── Arena and chapter items ─────────────────────────────────────────────

def hook_blossom(fresh=True):
    """Hook point in the air: large blossom on a tendril from above, centre as a ring for hooking."""
    petal = PINK if fresh else '#c8b6bc'
    core = '#f2c14e' if fresh else '#b8ac90'
    s = f'<path d="M 0,-130 Q 12,-90 0,-44" fill="none" stroke="{OUT}" stroke-width="9"/>'
    s += f'<path d="M 0,-130 Q 12,-90 0,-44" fill="none" stroke="{STEM if fresh else "#8a9a6a"}" stroke-width="4.5"/>'
    s += f'<path d="M 6,-96 Q 34,-104 34,-84 Q 18,-80 6,-96 Z" fill="{LEAF if fresh else "#a8b088"}" {st(3)}/>'
    for i in range(6):
        a = math.radians(i * 60 + (0 if fresh else 15))
        droop = 0 if fresh else 10
        px, py = 30 * math.cos(a), -20 + 30 * math.sin(a) + droop
        s += f'<ellipse cx="{px:.1f}" cy="{py:.1f}" rx="20" ry="12" transform="rotate({i * 60} {px:.1f} {py:.1f})" fill="{petal}" {st(3.5)}/>'
    s += f'<circle cx="0" cy="-20" r="20" fill="{core}" {st(4)}/>'
    s += f'<circle cx="0" cy="-20" r="9" fill="none" stroke="{OUT}" stroke-width="4"/>'
    if fresh:
        s += f'<circle cx="0" cy="-20" r="30" fill="none" stroke="#ffffff" stroke-width="2" stroke-dasharray="4 6" opacity="0.8"/>'
    return s


def spring_spark():
    s = '<circle cx="0" cy="-30" r="34" fill="#ffd6ea" opacity="0.45"/>'
    s += f'<path d="M 0,-64 C 18,-40 22,-24 0,-6 C -22,-24 -18,-40 0,-64 Z" fill="{PINK}" {st(4)}/>'
    s += '<path d="M -6,-40 Q -8,-28 -2,-20" fill="none" stroke="#ffffff" stroke-width="4" stroke-linecap="round"/>'
    for (x, y, r) in ((24, -56, 7), (-26, -46, 5), (20, -12, 5)):
        s += star(x, y, r, '#fff3a8')
    return s


def honeycomb_hat():
    s = f'<ellipse cx="0" cy="-10" rx="44" ry="10" fill="#f0ece4" {st(4)}/>'
    s += f'<path d="M -24,-14 Q -22,-48 0,-50 Q 22,-48 24,-14 Z" fill="#f0ece4" {st(4)}/>'
    s += f'<path d="M -23,-22 Q 0,-16 23,-22" fill="none" stroke="{HONEY_DARK}" stroke-width="6"/>'
    for (x, y) in ((-8, -34), (4, -36), (14, -30)):
        pts = ' '.join(f'{x + 5 * math.cos(math.radians(a)):.1f},{y + 5 * math.sin(math.radians(a)):.1f}' for a in range(0, 360, 60))
        s += f'<polygon points="{pts}" fill="{HONEY}" stroke="{HONEY_DARK}" stroke-width="1.4"/>'
    s += '<path d="M -40,-6 Q -42,8 -36,16 M 40,-6 Q 42,8 36,16" fill="none" stroke="#9aa6b0" stroke-width="2.5" stroke-dasharray="4 3"/>'
    return s


def beehive():
    """Straw skep (beehive) on a wooden trestle."""
    s = f'<rect x="-34" y="-20" width="68" height="8" fill="#a87a52" {st(3)}/><path d="M -28,-12 V 0 M 28,-12 V 0" stroke="{OUT}" stroke-width="5"/>'
    s += f'<path d="M -30,-20 Q -34,-74 0,-80 Q 34,-74 30,-20 Z" fill="#d9b860" {st(4)}/>'
    for k in range(1, 6):
        y = -20 - k * 11
        w = 30 - k * 3.5
        s += f'<path d="M {-w},{y} Q 0,{y + 4} {w},{y}" fill="none" stroke="#b08f3c" stroke-width="2.4"/>'
    s += f'<ellipse cx="0" cy="-26" rx="7" ry="5" fill="{OUT}"/>'
    return s


def hive_stack():
    """Wooden hives (boxes) stacked, with lid."""
    s = ''
    for k, c in enumerate(('#e8c070', '#d9a85a', '#e8c070')):
        y = -24 - k * 24
        s += f'<rect x="-30" y="{y}" width="60" height="24" fill="{c}" {st(3.5)}/>'
        s += f'<path d="M -22,{y + 12} H 22" stroke="#b8863c" stroke-width="2"/>'
    s += f'<path d="M -36,-72 H 36 L 30,-84 H -30 Z" fill="#8a6040" {st(3.5)}/>'
    s += f'<rect x="-10" y="-8" width="20" height="5" rx="2" fill="{OUT}"/>'
    return s


def honey_stand():
    s = f'<rect x="-50" y="-44" width="100" height="44" fill="#c79a66" {st()}/>'
    s += f'<path d="M -58,-44 H 58 L 50,-56 H -50 Z" fill="#a87a52" {st()}/>'
    for k, x in enumerate((-34, -12, 10, 32)):
        s += f'<rect x="{x - 8}" y="-76" width="16" height="20" rx="4" fill="{HONEY}" {st(2.5)}/><rect x="{x - 9}" y="-80" width="18" height="6" rx="2" fill="{["#e8685a", "#5aaee8", "#e8685a", "#7fd99a"][k]}" {st(2)}/>'
    s += f'<path d="M -56,-56 V -110 M 56,-56 V -110" stroke="{OUT}" stroke-width="5"/>'
    s += f'<path d="M -64,-108 Q -32,-122 0,-108 Q 32,-122 64,-108 L 64,-96 Q 32,-110 0,-96 Q -32,-110 -64,-96 Z" fill="#f2c14e" {st()}/>'
    s += f'<path d="M -32,-114 V -100 M 32,-114 V -100" stroke="#ffffff" stroke-width="5"/>'
    return s


def giant_flower(color=PINK):
    """Decoration in wiese-2: flower as tall as a house, leaves as steps."""
    s = f'<path d="M 0,0 Q -14,-120 6,-250" fill="none" stroke="{OUT}" stroke-width="16"/>'
    s += f'<path d="M 0,0 Q -14,-120 6,-250" fill="none" stroke="{STEM}" stroke-width="9"/>'
    s += f'<path d="M -6,-70 Q -70,-90 -80,-60 Q -40,-50 -6,-70 Z" fill="{LEAF}" {st()}/>'
    s += f'<path d="M -2,-150 Q 66,-176 78,-144 Q 40,-134 -2,-150 Z" fill="{LEAF}" {st()}/>'
    for i in range(8):
        a = math.radians(i * 45)
        px, py = 6 + 44 * math.cos(a), -270 + 30 * math.sin(a)
        s += f'<ellipse cx="{px:.1f}" cy="{py:.1f}" rx="30" ry="16" transform="rotate({i * 45} {px:.1f} {py:.1f})" fill="{color}" {st()}/>'
    s += f'<ellipse cx="6" cy="-270" rx="30" ry="22" fill="#f2c14e" {st(4)}/>'
    return s


def garland():
    s = f'<path d="M -110,-150 Q 0,-110 110,-150" fill="none" stroke="#7a7266" stroke-width="2.5"/>'
    colors = ('#e8685a', '#f2c14e', '#5aaee8', '#7fd99a', '#a77be0', PINK)
    for k in range(9):
        x = -96 + k * 24
        y = -150 + 40 * (1 - ((x / 110) ** 2)) * 0.95
        s += f'<path d="M {x - 9},{y} H {x + 9} L {x},{y + 20} Z" fill="{colors[k % 6]}" {st(2.2)}/>'
    s += f'<path d="M -110,-150 V 0 M 110,-150 V 0" stroke="{OUT}" stroke-width="6"/><path d="M -110,-150 V 0 M 110,-150 V 0" stroke="#a87a52" stroke-width="3"/>'
    return s


def festival_lantern(color='#e8685a'):
    s = f'<path d="M 0,-120 V -100" stroke="{OUT}" stroke-width="2.5"/>'
    s += f'<circle cx="0" cy="-78" r="34" fill="#ffd27a" opacity="0.3"/>'
    s += f'<ellipse cx="0" cy="-78" rx="20" ry="24" fill="{color}" {st(3)}/>'
    s += f'<path d="M -12,-98 Q -20,-78 -12,-58 M 12,-98 Q 20,-78 12,-58 M 0,-102 V -54" fill="none" stroke="#2b2b2b" stroke-width="1.6" opacity="0.6"/>'
    s += f'<rect x="-10" y="-104" width="20" height="6" rx="2" fill="#4a4a48" {st(2)}/><rect x="-8" y="-56" width="16" height="5" rx="2" fill="#4a4a48" {st(2)}/>'
    s += f'<path d="M 0,-51 V -40" stroke="{color}" stroke-width="3"/>'
    return s


def blossom_spring(freed=True):
    """Spring at the end of the Blütenwiesen: stone basin, water, blossoms; withered = grey and dry."""
    s = ''
    if freed:
        s += '<ellipse cx="0" cy="-60" rx="140" ry="90" fill="#ffd6ea" opacity="0.35"/>'
    stone = '#aca69c'
    s += f'<path d="M -110,0 Q -116,-40 -90,-44 H 90 Q 116,-40 110,0 Z" fill="{stone}" {st()}/>'
    for x in range(-96, 100, 32):
        s += f'<path d="M {x},-44 V 0" stroke="#857f76" stroke-width="1.6"/>'
    s += f'<ellipse cx="0" cy="-44" rx="92" ry="14" fill="{"#7fc8e8" if freed else "#b8ac90"}" {st(3.5)}/>'
    if freed:
        s += '<path d="M -40,-46 q 10,-4 20,0 M 20,-42 q 10,-4 20,0" fill="none" stroke="#ffffff" stroke-width="3" stroke-linecap="round"/>'
        s += f'<path d="M 0,-44 Q -6,-110 0,-150" fill="none" stroke="#bfe6f5" stroke-width="10" stroke-linecap="round" opacity="0.8"/>'
        for k in range(5):
            a = math.radians(-90 + (k - 2) * 30)
            s += f'<path d="M 0,-150 q {50 * math.cos(a):.0f},{-20 + 30 * math.sin(a):.0f} {70 * math.cos(a):.0f},{40}" fill="none" stroke="#bfe6f5" stroke-width="5" stroke-linecap="round" opacity="0.8"/>'
        s += star(0, -156, 12, '#ffffff')
    else:
        s += '<path d="M -30,-44 l 10,6 l 8,-6 l 12,5 M 20,-46 l 8,6 l 10,-4" fill="none" stroke="#8a7a5a" stroke-width="2"/>'
    for (x, c) in ((-120, PINK), (-96, '#f2c14e'), (100, '#a77be0'), (124, PINK)):
        col = c if freed else '#c8c2b4'
        h = 40 if freed else 26
        s += f'<path d="M {x},0 V {-h}" stroke="{OUT}" stroke-width="5"/><path d="M {x},0 V {-h}" stroke="{STEM if freed else "#9a9a80"}" stroke-width="2.5"/>'
        s += f'<circle cx="{x}" cy="{-h - 6}" r="9" fill="{col}" {st(2.5)}/>'
    return s


# ── Sheet ───────────────────────────────────────────────────────────────

def card(x, y, w, h, title):
    return (f'<rect x="{x + 3}" y="{y + 5}" width="{w}" height="{h}" rx="18" fill="#1e2a36" fill-opacity="0.10"/>'
            f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="18" fill="#fffaf0" stroke="#e7dcc8" stroke-width="1.5"/>'
            + text(x + 22, y + 32, title, 19, TEXT, 'start', 'bold'))


def ground(x, y, w):
    return f'<rect x="{x}" y="{y}" width="{w}" height="6" rx="3" fill="#8fbf7a"/>'


def elora(x, y, s=0.36):
    return g(x, y, s, drop('f2c14e', 'd9a43a'))


def sheet():
    W, H = 1600, 1640
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
         '<defs><linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#a9cde8"/>'
         '<stop offset="1" stop-color="#eef4f8"/></linearGradient></defs>',
         f'<rect width="{W}" height="{H}" fill="url(#sky)"/>',
         text(40, 50, 'Kapitel 1 „Blütenwiesen“ – Entwürfe (R2-M2.1, M2.1.0)', 28, TEXT, 'start', 'bold'),
         text(40, 78, 'Neue Figur, Hüter, Gegner, Arena, Gegenstände und Deko im Stil der bisherigen Figuren und Gebäude. '
              'Elora klein daneben als Maßstab.', 15, TEXT, 'start')]

    # Wabe and bees
    o.append(card(30, 100, 760, 390, 'Imkerin Wabe und die Bienen'))
    o.append(ground(60, 400, 260) + g(190, 400, 1.0, wabe()) + elora(320, 400))
    o.append(text(190, 436, 'Imkerin Wabe', 17, TEXT, weight='bold'))
    o.append(text(190, 458, 'Imkerhut mit Schleier, Schürze mit Waben, Smoker, Honigglas', 12, DIM, style='italic'))
    o.append(bee(1.3, 470, 340) + bee(1.3, 650, 340, angry=True))
    o.append(text(470, 436, 'Biene (Sammelstück)', 15, TEXT, weight='bold'))
    o.append(text(470, 456, 'freundlich, schwebt, 5 Stück', 12, DIM))
    o.append(text(650, 436, 'verwirrte Biene (Gegner)', 15, TEXT, weight='bold'))
    o.append(text(650, 456, 'Phase 3, umschwirrt Elora', 12, DIM))

    # items
    o.append(card(810, 100, 760, 390, 'Gegenstände'))
    o.append(ground(860, 400, 180) + g(950, 400, 1.6, spring_spark()))
    o.append(text(950, 436, 'Quellfunke', 17, TEXT, weight='bold'))
    o.append(text(950, 458, 'Beute des Hüters, für Tüftel', 12, DIM))
    o.append(ground(1120, 400, 180) + g(1210, 400, 1.6, honeycomb_hat()))
    o.append(text(1210, 436, 'Wabenhut', 17, TEXT, weight='bold'))
    o.append(text(1210, 458, 'Belohnung für die Bienen (E-300)', 12, DIM))
    o.append(g(1440, 330, 1.2, bee(1.0, 0, 0)) + text(1440, 436, 'Bienensymbol', 15, TEXT, weight='bold'))
    o.append(text(1440, 458, 'Aufgabenbuch, Zähler 0/5', 12, DIM))

    # bumblebee
    o.append(card(30, 510, 1540, 470, 'Hüter: Brummbär-Hummel (E-298, E-299)'))
    poses = [
        (240, 860, 'flug', 'Kreisen', 'fliegt hoch, lässt Pollen fallen'),
        (600, 820, 'sturz', 'Sturzflug', 'nach Warnung: Brummen, Schatten am Boden'),
        (960, 900, 'benommen', 'benommen am Boden', 'nur jetzt verwundbar'),
        (1320, 900, 'ruhig', 'nach dem Kampf', 'nur verwirrt, nicht böse'),
    ]
    for x, y, pose, name, note in poses:
        o.append(ground(x - 160, 900, 320))
        o.append(g(x - 20, y, 1.0, bumblebee(pose)))
        o.append(text(x, 934, name, 17, TEXT, weight='bold'))
        o.append(text(x, 956, note, 12, DIM))
    o.append(pollen(330, 880) + pollen(300, 840, 7))
    o.append('<ellipse cx="600" cy="902" rx="70" ry="8" fill="#2b2b2b" opacity="0.25"/>')
    o.append(elora(1480, 900))

    # arena
    o.append(card(30, 1000, 760, 360, 'Arena und Blütenquelle'))
    o.append(g(120, 1250, 0.9, hook_blossom(True)) + g(250, 1250, 0.9, hook_blossom(False)))
    o.append(text(185, 1316, 'Hook-Blüte frisch / welk', 15, TEXT, weight='bold'))
    o.append(text(185, 1336, 'Hookpunkt in der Luft (neues Tile)', 12, DIM))
    o.append(ground(330, 1290, 440) + g(430, 1290, 0.75, blossom_spring(False)) + g(670, 1290, 0.75, blossom_spring(True)))
    o.append(text(550, 1316, 'Blütenquelle verdorrt / befreit', 15, TEXT, weight='bold'))
    o.append(text(550, 1336, 'Mitte der Arena, Schluss des Kapitels', 12, DIM))

    # apiary and decoration
    o.append(card(810, 1000, 760, 360, 'Imkerei und Deko (wiese-2)'))
    o.append(ground(840, 1290, 700))
    o.append(g(890, 1290, 1.0, beehive()) + g(980, 1290, 1.0, hive_stack()) + g(1100, 1290, 0.9, honey_stand()))
    o.append(g(1260, 1290, 0.85, giant_flower()) + g(1400, 1290, 0.7, giant_flower('#a77be0')) + elora(1490, 1290))
    o.append(text(935, 1316, 'Bienenkorb, Beuten', 15, TEXT, weight='bold'))
    o.append(text(1100, 1316, 'Wabes Honigstand', 15, TEXT, weight='bold'))
    o.append(text(1330, 1316, 'Riesenblumen', 15, TEXT, weight='bold'))
    o.append(text(1330, 1336, 'Blätter als Stufen, Blüte als Plattform', 12, DIM))

    # festival
    o.append(card(30, 1380, 1540, 230, 'Fest in Tauwinkel nach Kapitel 1 (E-301)'))
    o.append(ground(60, 1570, 1480))
    o.append(g(260, 1570, 1.0, garland()) + g(560, 1570, 1.0, garland()))
    for k, (x, c) in enumerate(((800, '#e8685a'), (880, '#f2c14e'), (960, '#5aaee8'), (1040, '#7fd99a'))):
        o.append(g(x, 1570 - (k % 2) * 14, 1.0, festival_lantern(c)))
    o.append(text(410, 1600, 'Girlanden zwischen den Häusern', 15, TEXT, weight='bold'))
    o.append(text(920, 1600, 'Festlaternen (leuchten am Abend)', 15, TEXT, weight='bold'))
    o.append(g(1250, 1570, 0.8, wabe()) + elora(1350, 1570) + g(1430, 1490, 0.7, bumblebee('ruhig')))
    o.append(text(1340, 1600, 'Wabe, Elora und die Hummel feiern mit', 13, DIM))
    o.append('</svg>')
    return '\n'.join(o)


def export():
    """Game graphics from the accepted drafts (E-303): hook blossom as a tile."""
    parts = ''.join(f'<g id="{k}"><g transform="translate(16,24) scale(0.4)">{hook_blossom(v)}</g></g>'
                    for k, v in (('fresh', True), ('wilted', False)))
    with open('assets/map/tiles/hookpoint.svg', 'w') as f:
        f.write('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">\n'
                '  <!-- Hookpunkt „Hook-Blüte“ (R2-M2.1, aus tools/design/kapitel1_entwuerfe.py): frisch und welk. -->\n'
                f'  {parts}\n</svg>\n')


# Decoration for the chapter 1 maps (E-303): name: (drawing, viewBox), origin at the bottom centre.
# `-fest` only hangs during the festival, `-verdorrt` becomes `-befreit` after `befreit.<name>` (session).
DECOR = {
    'beehive': (beehive(), '-40 -84 80 86'),
    'hive_stack': (hive_stack(), '-40 -88 80 90'),
    'honey_stand': (honey_stand(), '-70 -126 140 128'),
    'giant-flower-pink': (giant_flower(PINK), '-110 -310 220 312'),
    'giant-flower-yellow': (giant_flower('#f2c14e'), '-110 -310 220 312'),
    'giant-flower-lilac': (giant_flower('#a77be0'), '-110 -310 220 312'),
    'garland-party': (garland(), '-116 -156 232 158'),
    'party-lantern-party': (festival_lantern(), '-36 -124 72 86'),
    'party-lantern-yellow-party': (festival_lantern('#f2c14e'), '-36 -124 72 86'),
    'blossom-spring-withered': (blossom_spring(False), '-150 -180 300 182'),
    'blossom-spring-freed': (blossom_spring(True), '-150 -180 300 182'),
}


def export_decor():
    for name, (art, vb) in DECOR.items():
        with open(f'assets/map/decor/{name}.svg', 'w') as f:
            f.write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}">\n'
                    f'  <!-- Kapitel 1: {name} (R2-M2.1, aus tools/design/kapitel1_entwuerfe.py). Ursprung unten in der Mitte. -->\n'
                    f'  {art}\n</svg>\n')


if __name__ == '__main__':
    os.makedirs('docs/release-2/design', exist_ok=True)
    open('docs/release-2/design/chapter1-drafts.svg', 'w').write(sheet())
    export()
    export_decor()

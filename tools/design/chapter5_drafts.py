"""Drafts for chapter 5 "Sternschlucht" (R2-M2.5, step M2.5.0, E-362 to E-368).

Usage: python3 tools/design/chapter5_drafts.py && cargo xtask svg-preview \
        docs/release-2/design/chapter5-drafts.svg docs/release-2/design/chapter5-drafts.png 1600

Style like chapters 1 to 4: dark outline, flat colours; here night blue, violet and
starlight. Two variants each for Luma and the Withered One (your choice), one draft for
everything else, plus a small night scene showing darkness and light (M2.5.1).
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from adventure_figures import DIM, OUT, TEXT, angry_eye, drop, g, star, text  # noqa: E402
from chapter1_drafts import card, elora  # noqa: E402
from chapter2_drafts import blob  # noqa: E402

NIGHT = '#1f2452'
NIGHT_MID = '#2f3778'
NIGHT_LIGHT = '#4a55a8'
VIOLET = '#a77be0'
VIOLET_LIGHT = '#d9c6f7'
VIOLET_DARK = '#6e4aa8'
STARLIGHT = '#fff3b0'
GOLD = '#f2c14e'
STONE = '#77728f'
STONE_LIGHT = '#9893b0'
STONE_DARK = '#4f4a68'
GLOW = '#7fe8d6'
GREY = '#a3a3ab'
GREY_LIGHT = '#c9c9cf'
GREY_DARK = '#6c6c76'
MOSS = '#5f7a8a'


def st(w=3.5):
    return f'stroke="{OUT}" stroke-width="{w}" stroke-linejoin="round" stroke-linecap="round"'


def ground(x, y, w):
    return f'<rect x="{x}" y="{y}" width="{w}" height="6" rx="3" fill="{STONE_LIGHT}"/>'


def sparkle(cx, cy, r, color=STARLIGHT):
    return star(cx, cy, r, color, OUT, 1.2)


def note(cx, cy, s=1.0, color=STARLIGHT):
    """A music note (song, note stones)."""
    return (f'<g transform="translate({cx},{cy}) scale({s})">'
            f'<ellipse cx="0" cy="0" rx="7" ry="5.5" fill="{color}" {st(2)} transform="rotate(-20)"/>'
            f'<path d="M 6,-2 V -24 Q 14,-20 16,-12" fill="none" stroke="{OUT}" stroke-width="2.5"/></g>')


def glow_circle(cx, cy, r, color=STARLIGHT, opacity=0.35):
    return f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="{color}" opacity="{opacity}"/>'


# ── Luma ────────────────────────────────────────────────────────────────

def luma(variant='A', pose='schwach'):
    """Luma the star weaver, last guardian of the Star Spring, grown weak.
    A: violet drop with a veil of light threads and a spindle with a star.
    B: older, small moth wings of light, feeler antennae and a woven shawl.
    Poses: schwach (dim, drooping), webt (weaves the star cape), singt, erleichtert."""
    dim = pose == 'schwach'
    body = '8a6ac0' if dim else 'a77be0'
    feet = '5e4690' if dim else '7a52c0'
    s = ''
    if pose in ('singt', 'erleichtert'):
        s += glow_circle(0, -80, 92, STARLIGHT, 0.25)
    if variant == 'B':
        # moth wings of light behind the body
        for side in (-1, 1):
            s += (f'<path d="M {side * 20},-90 Q {side * 110},-170 {side * 96},-70 Q {side * 80},-40 {side * 20},-60 Z" '
                  f'fill="{VIOLET_LIGHT}" fill-opacity="{0.45 if dim else 0.85}" {st(3)}/>')
            s += f'<circle cx="{side * 70}" cy="-100" r="9" fill="{STARLIGHT}" {st(2)}/>'
    s += drop(body, feet, mouth=pose not in ('schwach', 'singt'))
    # closed eyes: cover the drawn ones, then tired (schwach) or happy (singt) lids
    if pose in ('schwach', 'singt'):
        for x in (2, 26):
            s += f'<ellipse cx="{x}" cy="-63" rx="9" ry="13" fill="#{body}"/>'
        bend = 6 if dim else -7
        s += (f'<path d="M -6,-62 q 8,{bend} 16,0 M 18,-62 q 8,{bend} 16,0" fill="none" stroke="{OUT}" '
              f'stroke-width="3.5" stroke-linecap="round"/>')
        if dim:
            s += f'<path d="M 8,-40 q 6,-4 12,0" fill="none" stroke="{OUT}" stroke-width="3" stroke-linecap="round"/>'
        else:
            s += f'<ellipse cx="14" cy="-40" rx="6" ry="7" fill="{OUT}"/>'
    if variant == 'A':
        # veil of light threads over the head
        s += (f'<path d="M -40,-112 Q 0,-150 44,-108 Q 60,-60 50,-20 L 40,-20 Q 46,-64 32,-100 Q 0,-124 -30,-100 '
              f'Q -46,-60 -40,-20 L -50,-20 Q -60,-70 -40,-112 Z" fill="{VIOLET_LIGHT}" fill-opacity="0.75" {st(3)}/>')
        for x in (-30, -12, 8, 26):
            s += f'<path d="M {x},-120 q 4,30 0,60" fill="none" stroke="{STARLIGHT}" stroke-width="2" opacity="0.9"/>'
        s += sparkle(4, -138, 11 if not dim else 7, STARLIGHT if not dim else GREY_LIGHT)
        # spindle with a star
        if pose != 'erleichtert':
            s += f'<path d="M -54,-80 L -70,-10" stroke="{OUT}" stroke-width="6" stroke-linecap="round"/>'
            s += f'<path d="M -54,-80 L -70,-10" stroke="#c9955c" stroke-width="3" stroke-linecap="round"/>'
            s += sparkle(-52, -86, 9 if not dim else 6, STARLIGHT if not dim else GREY_LIGHT)
    else:
        # feelers and woven shawl
        for side in (-1, 1):
            s += f'<path d="M {side * 10},-126 Q {side * 30},-170 {side * 48},-168" fill="none" stroke="{OUT}" stroke-width="3"/>'
            s += f'<circle cx="{side * 50}" cy="-168" r="6" fill="{STARLIGHT if not dim else GREY_LIGHT}" {st(2)}/>'
        s += f'<path d="M -46,-58 Q 0,-34 46,-58 L 40,-40 Q 0,-20 -40,-40 Z" fill="{NIGHT_LIGHT}" {st(3)}/>'
        for x in (-28, -10, 8, 26):
            s += sparkle(x, -46, 4, STARLIGHT)
    if pose == 'webt':
        # threads from her hands into a little star cape
        s += f'<path d="M 46,-70 Q 90,-110 120,-60" fill="none" stroke="{STARLIGHT}" stroke-width="2.5"/>'
        s += f'<path d="M 46,-60 Q 86,-90 116,-50" fill="none" stroke="{STARLIGHT}" stroke-width="2"/>'
        s += g(126, -40, 0.5, star_cape())
    if pose == 'singt':
        for (x, y, k) in ((60, -120, 1.0), (84, -150, 0.8), (40, -160, 0.7)):
            s += note(x, y, k)
    if pose == 'erleichtert':
        s += '<ellipse cx="-4" cy="-48" rx="8" ry="4" fill="#ef9fc0" opacity="0.7"/><ellipse cx="40" cy="-48" rx="8" ry="4" fill="#ef9fc0" opacity="0.7"/>'
        for (x, y) in ((-70, -120), (72, -140), (90, -70)):
            s += sparkle(x, y, 8)
    return s


# ── The Withered One ────────────────────────────────────────────────────

def withered_one(variant='A', pose='saugt'):
    """The Withered One, first appearance: grey, thin, sad – not wicked.
    A: tall thin grey drop with cracks, long arms like dry twigs, hollow sad eyes.
    B: grey wanderer in a hooded cloak, a dry flower in his hand.
    Poses: saugt (drains colour), blickt (looks at Elora), flieht (turns away)."""
    s = ''
    if pose == 'saugt':
        # a stream of colour flows into him
        for k, c in enumerate(('#7fd99a', '#f2c14e', '#8fcde8', VIOLET)):
            s += (f'<path d="M {170 - k * 6},{-40 - k * 18} Q 110,{-120 - k * 10} 50,{-120 + k * 8}" fill="none" '
                  f'stroke="{c}" stroke-width="7" stroke-linecap="round" opacity="0.85"/>')
    lean = {'saugt': 'rotate(4 0 0)', 'blickt': '', 'flieht': 'rotate(-10 0 0)'}[pose]
    o = f'<g transform="{lean}">'
    if variant == 'A':
        o += f'<path d="M 0,-250 Q 56,-160 56,-110 Q 56,-60 0,-60 Q -56,-60 -56,-110 Q -56,-160 0,-250 Z" fill="{GREY}" {st(5)}/>'
        o += f'<path d="M -10,-200 l 12,24 l -10,18 M 24,-150 l -10,20 l 10,14 M -30,-120 l 14,10" fill="none" stroke="{GREY_DARK}" stroke-width="3"/>'
        # thin legs
        o += f'<path d="M -18,-62 L -24,0 M 18,-62 L 26,0" stroke="{OUT}" stroke-width="9" stroke-linecap="round"/>'
        o += f'<path d="M -18,-62 L -24,0 M 18,-62 L 26,0" stroke="{GREY_DARK}" stroke-width="5" stroke-linecap="round"/>'
        # twig arms
        arm = {'saugt': 'M 46,-130 L 96,-150 L 120,-140 M 96,-150 L 110,-176',
               'blickt': 'M 46,-120 L 70,-80 L 66,-56 M 70,-80 L 90,-74',
               'flieht': 'M 46,-130 L 90,-160 L 120,-158 M 90,-160 L 92,-190'}[pose]
        arm2 = 'M -46,-120 L -74,-84 L -70,-60 M -74,-84 L -94,-78'
        for d in (arm, arm2):
            o += f'<path d="{d}" fill="none" stroke="{OUT}" stroke-width="9" stroke-linecap="round" stroke-linejoin="round"/>'
            o += f'<path d="{d}" fill="none" stroke="#8a7a66" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>'
        face_y = -150
    else:
        # hooded cloak
        o += f'<path d="M -70,0 Q -60,-120 -40,-180 Q 0,-240 40,-180 Q 60,-120 70,0 Z" fill="{GREY_DARK}" {st(5)}/>'
        o += f'<path d="M -40,-180 Q 0,-250 40,-180 Q 30,-140 0,-136 Q -30,-140 -40,-180 Z" fill="{GREY}" {st(4)}/>'
        o += f'<path d="M -50,-60 Q -48,-20 -54,0 M 50,-60 Q 48,-20 54,0" fill="none" stroke="{GREY}" stroke-width="3"/>'
        # dry flower in his hand
        o += f'<path d="M 50,-90 Q 70,-110 78,-150" fill="none" stroke="#8a7a66" stroke-width="4"/>'
        for k in range(5):
            a = math.radians(-90 + k * 72)
            o += f'<ellipse cx="{78 + 9 * math.cos(a):.1f}" cy="{-158 + 9 * math.sin(a):.1f}" rx="6" ry="4" fill="{GREY_LIGHT}" {st(1.5)}/>'
        face_y = -168
    # hollow sad eyes
    if pose == 'blickt':
        o += f'<ellipse cx="-14" cy="{face_y}" rx="7" ry="10" fill="#2b2b2b"/><ellipse cx="14" cy="{face_y}" rx="7" ry="10" fill="#2b2b2b"/>'
        o += f'<circle cx="-12" cy="{face_y - 4}" r="2.5" fill="#ffffff"/><circle cx="16" cy="{face_y - 4}" r="2.5" fill="#ffffff"/>'
        o += f'<path d="M -10,{face_y + 30} q 10,-6 20,0" fill="none" stroke="{OUT}" stroke-width="3" stroke-linecap="round"/>'
        o += f'<path d="M -22,{face_y - 12} l 12,-6 M 22,{face_y - 12} l -12,-6" stroke="{OUT}" stroke-width="3" stroke-linecap="round"/>'
    else:
        o += f'<ellipse cx="-14" cy="{face_y}" rx="7" ry="5" fill="#2b2b2b"/><ellipse cx="14" cy="{face_y}" rx="7" ry="5" fill="#2b2b2b"/>'
    o += '</g>'
    s += o
    if pose == 'flieht':
        for y in (-200, -150, -100):
            s += f'<path d="M -90,{y} h -40" stroke="{GREY_DARK}" stroke-width="4" stroke-linecap="round" opacity="0.6"/>'
    return s


def grey_shadow():
    """Grey shadow of the Withered One in the climax: drifts towards Luma, light dissolves it."""
    s = f'<path d="M -30,0 Q -40,-50 -10,-70 Q 20,-84 34,-50 Q 44,-20 30,0 Q 20,-10 10,0 Q 0,-12 -10,0 Q -20,-10 -30,0 Z" fill="{GREY_DARK}" fill-opacity="0.85" {st(3)}/>'
    s += f'<ellipse cx="-6" cy="-46" rx="4" ry="6" fill="{GREY_LIGHT}"/><ellipse cx="12" cy="-46" rx="4" ry="6" fill="{GREY_LIGHT}"/>'
    return s


# ── Enemies ─────────────────────────────────────────────────────────────

def shadow_moth(flap=0):
    """Shadow moth: dark, small, glowing eyes; avoids light, comes in swarms."""
    up = -26 if flap == 0 else -12
    s = ''
    for side in (-1, 1):
        s += f'<path d="M 0,-30 Q {side * 40},{up - 30} {side * 46},{up} Q {side * 30},-14 0,-26 Z" fill="{NIGHT_MID}" {st(3)}/>'
        s += f'<circle cx="{side * 26}" cy="{up - 6}" r="5" fill="{VIOLET}" opacity="0.8"/>'
    s += f'<ellipse cx="0" cy="-26" rx="9" ry="16" fill="{NIGHT}" {st(3)}/>'
    s += f'<circle cx="-4" cy="-34" r="3" fill="{STARLIGHT}"/><circle cx="4" cy="-34" r="3" fill="{STARLIGHT}"/>'
    s += f'<path d="M -3,-42 q -8,-12 -14,-12 M 3,-42 q 8,-12 14,-12" fill="none" stroke="{OUT}" stroke-width="2"/>'
    return s


def stone_goblin(pose='fels'):
    """Stone goblin: looks like a rock (fels), wakes up (wach), rolls at Elora (rollt)."""
    if pose == 'rollt':
        s = f'<circle cx="0" cy="-30" r="30" fill="{STONE}" {st(4)}/>'
        s += f'<path d="M -20,-40 Q 0,-10 20,-44 M -10,-56 Q 10,-34 26,-20" fill="none" stroke="{STONE_DARK}" stroke-width="3"/>'
        s += angry_eye(-8, -36, 6) + angry_eye(10, -32, 6, brow=-1)
        for x in (-44, -56):
            s += f'<path d="M {x},-30 h -14" stroke="{STONE_LIGHT}" stroke-width="4" stroke-linecap="round"/>'
        return s
    s = blob(0, -28, 44, 30, STONE, 18, 0.08, 4)
    s += f'<path d="M -26,-30 l 12,-8 M 14,-44 l 14,6 M -6,-16 l 18,2" stroke="{STONE_DARK}" stroke-width="3" stroke-linecap="round"/>'
    s += f'<ellipse cx="-24" cy="-50" rx="10" ry="5" fill="{MOSS}" {st(2)}/>'
    if pose == 'wach':
        s += angry_eye(-12, -32, 7) + angry_eye(12, -32, 7, brow=-1)
        s += f'<path d="M -10,-14 q 10,-6 20,0" fill="none" stroke="{OUT}" stroke-width="3"/>'
        for x in (-30, 30):
            s += f'<path d="M {x},-6 v 8" stroke="{OUT}" stroke-width="7" stroke-linecap="round"/>'
    return s


def echo():
    """Echo: translucent violet copy of Elora that repeats her path with a delay."""
    inner = drop('b9a0e8', '8a70c8', eyes='4a3a7a')
    trail = ''
    for k, op in enumerate((0.25, 0.15, 0.08)):
        trail += f'<g transform="translate({-28 * (k + 1)},{6 * (k + 1)})" opacity="{op}">{inner}</g>'
    return trail + f'<g opacity="0.75">{inner}</g>'


# ── Items and objects ───────────────────────────────────────────────────

def star_cape():
    """Star cape from Luma = gliding."""
    s = f'<path d="M -40,-80 Q 0,-96 40,-80 L 56,0 Q 0,-14 -56,0 Z" fill="{NIGHT_LIGHT}" {st(4)}/>'
    s += f'<path d="M -40,-80 Q 0,-96 40,-80 L 34,-68 Q 0,-80 -34,-68 Z" fill="{VIOLET}" {st(3)}/>'
    for (x, y, r) in ((-24, -40, 6), (10, -54, 5), (26, -20, 6), (-6, -16, 4), (-34, -12, 4)):
        s += sparkle(x, y, r)
    return s


def star_fragment():
    """Star fragment (side quest): a glowing shard."""
    return glow_circle(0, -20, 26, STARLIGHT, 0.3) + (
        f'<path d="M 0,-44 L 12,-22 L 4,-2 L -10,-16 Z" fill="{STARLIGHT}" {st(3)}/>'
        f'<path d="M 0,-44 L 4,-2" stroke="{GOLD}" stroke-width="2"/>')


def note_stone(lit=False):
    """Note stone: dark, or lit after touching it (plays a note, lights a star bridge)."""
    s = glow_circle(0, -40, 46, STARLIGHT, 0.35) if lit else ''
    s += f'<path d="M -26,0 L -30,-60 Q 0,-84 30,-60 L 26,0 Z" fill="{STONE_LIGHT if lit else STONE}" {st(4)}/>'
    s += note(-2, -36, 1.4, STARLIGHT if lit else STONE_DARK)
    return s


def star_bridge():
    """Star bridge: glowing translucent planks, only for a few seconds."""
    s = glow_circle(0, -8, 0, STARLIGHT)
    s += f'<rect x="-120" y="-16" width="240" height="14" rx="7" fill="{STARLIGHT}" fill-opacity="0.55" {st(2.5)}/>'
    for x in range(-100, 101, 40):
        s += sparkle(x, -9, 6)
    return s


def star_lantern(lit=True):
    """Star lantern: lights a circle that dissolves grey shadows (climax)."""
    s = glow_circle(0, -100, 70, STARLIGHT, 0.3) if lit else ''
    s += f'<path d="M 0,0 V -80" stroke="{OUT}" stroke-width="8"/><path d="M 0,0 V -80" stroke="{STONE_DARK}" stroke-width="4"/>'
    s += f'<path d="M -20,-80 L 20,-80 L 14,-124 L -14,-124 Z" fill="{STARLIGHT if lit else GREY_DARK}" {st(3.5)}/>'
    s += f'<path d="M -16,-124 L 0,-140 L 16,-124 Z" fill="{STONE}" {st(3)}/>'
    if lit:
        s += sparkle(0, -102, 9, '#ffffff')
    return s


def updraft():
    """Updraft column: lifts a gliding Elora."""
    s = f'<rect x="-40" y="-260" width="80" height="260" rx="30" fill="{VIOLET_LIGHT}" fill-opacity="0.25"/>'
    for y in range(-40, -260, -50):
        s += f'<path d="M -24,{y} q 24,-24 48,0" fill="none" stroke="{VIOLET}" stroke-width="4" stroke-linecap="round" opacity="0.8"/>'
    s += f'<ellipse cx="0" cy="0" rx="46" ry="8" fill="{STONE}" {st(3)}/>'
    return s


def star_island(sinking=False):
    """Floating star island: rock with grass, glowing underside; some sink under Elora."""
    s = ''
    if sinking:
        for x in (-30, 0, 30):
            s += f'<path d="M {x},40 v 18" stroke="{STONE_LIGHT}" stroke-width="4" stroke-linecap="round" opacity="0.7"/>'
    s += glow_circle(0, 10, 50, GLOW, 0.25)
    s += f'<path d="M -90,0 Q -80,-12 -60,-14 L 60,-14 Q 84,-12 90,0 Q 60,30 20,40 Q 0,60 -20,40 Q -60,30 -90,0 Z" fill="{STONE}" {st(4)}/>'
    s += f'<path d="M -86,-4 Q 0,-26 86,-4" fill="none" stroke="#6fbf8a" stroke-width="8" stroke-linecap="round"/>'
    for (x, y) in ((-40, 18), (10, 30), (40, 10)):
        s += f'<circle cx="{x}" cy="{y}" r="4" fill="{GLOW}"/>'
    return s


def crystal(cx, cy, h, w, color=STARLIGHT):
    """A glowing star crystal pointing down (hangs under the islands)."""
    return (f'<path d="M {cx - w},{cy} L {cx},{cy - h * 0.25} L {cx + w},{cy} L {cx},{cy + h} Z" '
            f'fill="{color}" {st(2.5)}/>'
            f'<path d="M {cx - w * 0.4},{cy + h * 0.05} L {cx},{cy + h * 0.6}" stroke="#ffffff" stroke-width="2" '
            f'stroke-linecap="round" opacity="0.9"/>')


def island_tile(sinking=False):
    """Star island for the game (M2.5.2): origin in the middle of the walkable surface, 180 wide;
    the client stretches it to the width of the island zone. The light comes from the star
    crystals that grew into the rock and hang below it; a sinking island has a cracked crystal
    that only flickers."""
    core = GOLD if sinking else STARLIGHT
    s = f'<g transform="translate(0,14)">'
    # halo around the crystals (the reason for the glow)
    s += f'<ellipse cx="0" cy="52" rx="56" ry="34" fill="{core}" opacity="{0.16 if sinking else 0.28}"/>'
    s += f'<ellipse cx="0" cy="56" rx="30" ry="20" fill="{core}" opacity="{0.2 if sinking else 0.35}"/>'
    s += f'<path d="M -90,0 Q -80,-12 -60,-14 L 60,-14 Q 84,-12 90,0 Q 60,30 20,40 Q 0,60 -20,40 Q -60,30 -90,0 Z" fill="{STONE}" {st(4)}/>'
    s += f'<path d="M -86,-6 Q 0,-20 86,-6" fill="none" stroke="#6fbf8a" stroke-width="7" stroke-linecap="round"/>'
    # crystals hanging below
    s += crystal(-24, 36, 22, 7, core) + crystal(22, 34, 20, 6, core) + crystal(0, 46, 34, 10, core)
    # small crystal tips in the rock
    for (x, y) in ((-52, 14), (48, 12)):
        s += f'<path d="M {x - 4},{y + 3} L {x},{y - 6} L {x + 4},{y + 3} Z" fill="{core}" {st(1.5)}/>'
    if sinking:
        # cracks in the rock and in the big crystal
        s += f'<path d="M -30,-4 L -24,8 L -32,18 M 34,-2 L 28,10" fill="none" stroke="{OUT}" stroke-width="2.5" stroke-linecap="round"/>'
        s += f'<path d="M -4,52 L 3,60 L -2,68" fill="none" stroke="{OUT}" stroke-width="2"/>'
    return s + '</g>'


def export_tiles():
    with open('assets/map/tiles/note-stone.svg', 'w') as f:
        f.write('<svg xmlns="http://www.w3.org/2000/svg" viewBox="-50 -90 100 92">\n'
                '  <!-- Note stone (R2-M2.5, M2.5.3, from tools/design/chapter5_drafts.py): dark and lit '
                '(the glow is drawn by the game). Origin bottom centre. -->\n'
                f'  <g id="dark">{note_stone(False)}</g>\n'
                f'  <g id="lit">{note_stone(True).replace(glow_circle(0, -40, 46, STARLIGHT, 0.35), "")}</g>\n</svg>\n')
    with open('assets/map/tiles/star-island.svg', 'w') as f:
        f.write('<svg xmlns="http://www.w3.org/2000/svg" viewBox="-96 -10 192 104">\n'
                '  <!-- Star island (R2-M2.5, M2.5.2, from tools/design/chapter5_drafts.py): drifting and '
                'sinking. Origin in the middle of the walkable surface. -->\n'
                f'  <g id="drift">{island_tile(False)}</g>\n  <g id="sink">{island_tile(True)}</g>\n</svg>\n')


# ── Spring and decoration ───────────────────────────────────────────────

def star_spring(spark=False):
    """Ruins of the Star Spring: dark basin, or with the saved star spark."""
    s = f'<path d="M -110,0 L -100,-40 Q 0,-56 100,-40 L 110,0 Z" fill="{STONE}" {st(4)}/>'
    s += f'<ellipse cx="0" cy="-42" rx="90" ry="12" fill="{NIGHT if not spark else VIOLET_DARK}" {st(3)}/>'
    for x in (-130, 130):
        s += f'<path d="M {x - 14},0 V -110 L {x},-126 L {x + 14},-110 V 0 Z" fill="{STONE_LIGHT}" {st(3.5)}/>'
    s += f'<path d="M -130,-120 Q 0,-200 130,-120" fill="none" stroke="{OUT}" stroke-width="16" stroke-dasharray="70 30"/>'
    s += f'<path d="M -130,-120 Q 0,-200 130,-120" fill="none" stroke="{STONE_LIGHT}" stroke-width="10" stroke-dasharray="70 30"/>'
    if spark:
        s += glow_circle(0, -80, 56, STARLIGHT, 0.35) + sparkle(0, -80, 18, STARLIGHT)
        for (x, y) in ((-40, -110), (44, -100), (0, -130)):
            s += sparkle(x, y, 6, VIOLET_LIGHT)
    return s


def crystal_star():
    """Crystal star on a rock (decoration, light source)."""
    return (glow_circle(0, -70, 40, STARLIGHT, 0.25)
            + blob(0, -24, 40, 26, STONE, 16, 0.08, 4)
            + f'<path d="M 0,-40 L 0,-90" stroke="{OUT}" stroke-width="3"/>' + sparkle(0, -96, 16))


def glow_mushrooms():
    """Glowing mushrooms in night colours (light source)."""
    s = ''
    for (x, h, r, c) in ((-30, 34, 18, GLOW), (0, 50, 24, VIOLET), (30, 28, 14, GLOW)):
        s += glow_circle(x, -h, r + 16, c, 0.25)
        s += f'<path d="M {x},0 V {-h}" stroke="{OUT}" stroke-width="9"/><path d="M {x},0 V {-h}" stroke="#e8e0f0" stroke-width="5"/>'
        s += f'<path d="M {x - r},{-h} Q {x},{-h - r * 1.4} {x + r},{-h} Z" fill="{c}" {st(3)}/>'
    return s


def ruin_arch():
    """Broken ruin arch of the star folk."""
    s = f'<path d="M -70,0 V -120 Q -70,-170 -20,-176 L -10,-160 Q -50,-150 -50,-116 V 0 Z" fill="{STONE_LIGHT}" {st(4)}/>'
    s += f'<path d="M 50,0 V -100 L 70,-110 V 0 Z" fill="{STONE_LIGHT}" {st(4)}/>'
    s += sparkle(-40, -130, 7, VIOLET_LIGHT)
    return s


def night_rock():
    """Dark gorge rock with veins of light."""
    s = f'<path d="M -80,0 L -60,-90 L -20,-130 L 30,-110 L 70,-60 L 80,0 Z" fill="{STONE_DARK}" {st(4)}/>'
    s += f'<path d="M -40,-80 L -10,-60 L 20,-90 M 30,-50 L 50,-30" fill="none" stroke="{GLOW}" stroke-width="3" opacity="0.8"/>'
    return s


def hanging_threads():
    """Threads of light hanging from above (Luma's hideout)."""
    s = ''
    for k, x in enumerate(range(-60, 61, 30)):
        y = 120 + (k % 3) * 30
        s += f'<path d="M {x},0 V {y}" stroke="{STARLIGHT}" stroke-width="2" opacity="0.8"/>' + sparkle(x, y, 6)
    return s


# ── Night scene (darkness and light, M2.5.1) ───────────────────────────

def night_scene(x0, y0, w, h):
    """A small part of a gorge map: night, Elora's light circle and light sources."""
    o = [f'<clipPath id="nightclip"><rect x="{x0}" y="{y0}" width="{w}" height="{h}" rx="12"/></clipPath>',
         '<radialGradient id="hole"><stop offset="0" stop-color="#000" stop-opacity="1"/>'
         '<stop offset="0.6" stop-color="#000" stop-opacity="0.85"/><stop offset="1" stop-color="#000" stop-opacity="0"/></radialGradient>',
         '<mask id="dark">', f'<rect x="{x0}" y="{y0}" width="{w}" height="{h}" fill="#ffffff"/>']
    lights = ((x0 + 300, y0 + h - 90, 150), (x0 + 120, y0 + h - 60, 80), (x0 + 560, y0 + h - 80, 90),
              (x0 + 700, y0 + 120, 70))
    for (cx, cy, r) in lights:
        o.append(f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="url(#hole)"/>')
    o.append('</mask>')
    o.append(f'<g clip-path="url(#nightclip)">')
    o.append(f'<rect x="{x0}" y="{y0}" width="{w}" height="{h}" fill="{NIGHT_MID}"/>')
    for (sx, sy) in ((80, 40), (200, 70), (420, 30), (520, 90), (640, 50), (760, 60)):
        o.append(sparkle(x0 + sx, y0 + sy, 4, '#ffffff'))
    o.append(f'<rect x="{x0}" y="{y0 + h - 40}" width="{w}" height="40" fill="{STONE_DARK}"/>')
    o.append(g(x0 + 120, y0 + h - 40, 0.8, glow_mushrooms()))
    o.append(g(x0 + 560, y0 + h - 40, 0.6, star_lantern(True)))
    o.append(g(x0 + 700, y0 + 150, 0.6, star_island()))
    o.append(g(x0 + 440, y0 + h - 40, 0.8, stone_goblin('fels')))
    o.append(g(x0 + 640, y0 + h - 120, 0.7, shadow_moth()))
    o.append(g(x0 + 300, y0 + h - 40, 0.36, drop('f2c14e', 'd9a43a')))
    o.append(f'<rect x="{x0}" y="{y0}" width="{w}" height="{h}" fill="#0b0e26" fill-opacity="0.82" mask="url(#dark)"/>')
    o.append('</g>')
    return ''.join(o)


# ── Sheet ───────────────────────────────────────────────────────────────

def sheet():
    W, H = 1600, 2400
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" font-family="Inter">',
         '<defs><linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#3a4290"/>'
         '<stop offset="1" stop-color="#b9b4e0"/></linearGradient></defs>',
         f'<rect width="{W}" height="{H}" fill="url(#sky)"/>',
         text(40, 50, 'Kapitel 5 „Sternschlucht“ – Entwürfe (R2-M2.5, M2.5.0)', 28, '#ffffff', 'start', 'bold'),
         text(40, 78, 'Luma und der Dürre je in zwei Varianten zur Auswahl; Gegner, Gelände, Quelle und Deko. '
              'Elora klein daneben als Maßstab.', 15, '#ffffff', 'start')]

    # Luma
    o.append(card(30, 100, 1540, 430, 'Luma, die Sternenweberin – Variante A (Lichtschleier, Spindel) oder B (Mottenflügel, Fühler)'))
    o.append(ground(60, 440, 1480))
    for x, var, pose, name in ((150, 'A', 'schwach', 'A: geschwächt'), (360, 'A', 'webt', 'A: webt den Umhang'),
                               (600, 'A', 'singt', 'A: singt'), (790, 'A', 'erleichtert', 'A: erleichtert'),
                               (1030, 'B', 'schwach', 'B: geschwächt'), (1240, 'B', 'singt', 'B: singt'),
                               (1460, 'B', 'erleichtert', 'B: erleichtert')):
        o.append(g(x, 440, 1.0, luma(var, pose)))
        o.append(text(x, 476, name, 15, TEXT, weight='bold'))
    o.append(elora(920, 440))
    o.append(text(780, 506, 'zierlich, freundlich, müde; Licht kehrt zurück, wenn sie singt', 12, DIM, style='italic'))

    # Withered One
    o.append(card(30, 550, 1540, 500, 'Der Dürre – erste Begegnung (E-362): Variante A (rissiger Tropfen, Zweigarme) oder B (Wanderer im Kapuzenmantel)'))
    o.append(ground(60, 960, 1480))
    for x, var, pose, name in ((200, 'A', 'saugt', 'A: saugt die Farbe'), (480, 'A', 'blickt', 'A: blickt Elora an'),
                               (680, 'A', 'flieht', 'A: flieht'), (960, 'B', 'saugt', 'B: saugt die Farbe'),
                               (1220, 'B', 'blickt', 'B: blickt Elora an'), (1430, 'B', 'flieht', 'B: flieht')):
        o.append(g(x, 960, 0.85, withered_one(var, pose)))
        o.append(text(x, 996, name, 15, TEXT, weight='bold'))
    o.append(elora(580, 960))
    o.append(text(800, 1024, 'grau, dünn, traurig – nicht böse: „Er sucht etwas, das er verloren hat.“', 12, DIM, style='italic'))

    # enemies
    o.append(card(30, 1070, 1000, 380, 'Gegner (E-365)'))
    o.append(ground(60, 1370, 940))
    o.append(g(120, 1250, 1.0, shadow_moth(0)) + g(180, 1290, 0.8, shadow_moth(1)) + g(230, 1240, 0.9, shadow_moth(0)))
    o.append(text(175, 1406, 'Schattenmotten', 16, TEXT, weight='bold'))
    o.append(text(175, 1426, 'Schwarm, meiden Licht', 12, DIM))
    o.append(g(380, 1370, 1.0, stone_goblin('fels')) + g(500, 1370, 1.0, stone_goblin('wach')) + g(620, 1370, 1.0, stone_goblin('rollt')))
    o.append(text(500, 1406, 'Steinkobold: Fels · wach · rollt', 16, TEXT, weight='bold'))
    o.append(text(500, 1426, 'Stampfen oder Hammer stoppt ihn', 12, DIM))
    o.append(g(900, 1370, 0.6, echo()))
    o.append(text(860, 1406, 'Echo', 16, TEXT, weight='bold'))
    o.append(text(860, 1426, 'wiederholt Eloras Weg, Licht lässt es verblassen', 12, DIM))

    # items
    o.append(card(1050, 1070, 520, 380, 'Gegenstände'))
    o.append(ground(1080, 1370, 460))
    o.append(g(1150, 1370, 1.0, star_cape()))
    o.append(text(1150, 1406, 'Sternenumhang', 15, TEXT, weight='bold'))
    o.append(text(1150, 1426, '= Gleiten (E-363)', 12, DIM))
    o.append(g(1300, 1370, 1.4, star_fragment()))
    o.append(text(1300, 1406, 'Sternensplitter', 15, TEXT, weight='bold'))
    o.append(g(1450, 1370, 0.9, grey_shadow()))
    o.append(text(1450, 1406, 'graue Schatten', 15, TEXT, weight='bold'))
    o.append(text(1450, 1426, 'Höhepunkt: treiben zu Luma', 12, DIM))

    # terrain
    o.append(card(30, 1470, 1540, 420, 'Gelände (E-364): Sterneninseln, Aufwinde, Notensteine und Sternenbrücken, Sternenlaternen'))
    o.append(ground(60, 1810, 1480))
    o.append(g(170, 1640, 0.9, star_island()) + g(360, 1700, 0.8, star_island(True)))
    o.append(text(270, 1846, 'Sterneninseln: treibt · sinkt', 15, TEXT, weight='bold'))
    o.append(g(560, 1810, 1.0, updraft()) + elora(560, 1620))
    o.append(text(560, 1846, 'Aufwind', 15, TEXT, weight='bold'))
    o.append(text(560, 1866, 'hebt beim Gleiten', 12, DIM))
    o.append(g(740, 1810, 1.0, note_stone(False)) + g(860, 1810, 1.0, note_stone(True)))
    o.append(g(1040, 1720, 0.8, star_bridge()))
    o.append(text(900, 1846, 'Notenstein: dunkel · leuchtet → Sternenbrücke für ein paar Sekunden', 15, TEXT, weight='bold'))
    o.append(g(1290, 1810, 0.9, star_lantern(False)) + g(1430, 1810, 0.9, star_lantern(True)))
    o.append(text(1360, 1846, 'Sternenlaterne: aus · an', 15, TEXT, weight='bold'))
    o.append(text(1360, 1866, 'Lichtkreis löst Schatten auf', 12, DIM))

    # spring, decoration, night scene
    o.append(card(30, 1910, 760, 460, 'Sternenquelle und Deko'))
    o.append(ground(60, 2280, 700))
    o.append(g(170, 2280, 0.6, star_spring(False)) + g(380, 2280, 0.6, star_spring(True)))
    o.append(text(275, 2316, 'Ruine der Sternenquelle: dunkel · Funke', 14, TEXT, weight='bold'))
    o.append(g(540, 2280, 0.7, crystal_star()) + g(640, 2280, 0.7, glow_mushrooms()) + g(730, 2280, 0.7, ruin_arch()))
    o.append(g(110, 2010, 0.6, hanging_threads()) + g(470, 2280, 0.45, night_rock()))
    o.append(text(470, 2316, 'Nachtfels', 13, TEXT, weight='bold'))
    for x, t in ((540, 'Kristallstern'), (640, 'Leuchtpilze'), (725, 'Ruinenbogen')):
        o.append(text(x, 2340, t, 13, TEXT, weight='bold'))
    o.append(card(810, 1910, 760, 460, 'Nachtbild (M2.5.1): dunkel, Licht um Elora und an Lichtquellen'))
    o.append(night_scene(830, 1960, 720, 380))
    o.append('</svg>')
    return '\n'.join(o)


if __name__ == '__main__':
    os.makedirs('docs/release-2/design', exist_ok=True)
    open('docs/release-2/design/chapter5-drafts.svg', 'w').write(sheet())
    export_tiles()

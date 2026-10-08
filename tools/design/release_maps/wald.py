from kit import Grid, check
def build():
    W, H = 150, 48
    g = Grid(W, H)
    # --- left half (red), then mirrored ---
    g.rect(1, 42, 74, 46)                                    # ground
    # base: fortress at the wall
    g.rect(1, 30, 22, 31)                                    # ceiling of the base
    g.rect(20, 32, 22, 37)                                   # inner wall with passage at the bottom
    g.rect(1, 38, 6, 41)                                     # flag pedestal
    g.set(4, 37, 'r')
    g.rect(1, 19, 10, 21)                                    # lookout above the base
    g.plat(12, 25, 8)
    g.put(24, 42, '/')                                       # jump pad in front of the base, upwards
    # field in front of the base
    g.rect(28, 40, 36, 41); g.rect(31, 38, 34, 39)           # hill
    g.rect(44, 36, 52, 37, '%')                              # rock (unhookable)
    g.plat(26, 33, 7); g.plat(38, 28, 8)
    # upper route: tree house platforms
    g.plat(24, 21, 9); g.plat(40, 17, 8); g.plat(54, 21, 9)
    g.rect(58, 33, 66, 35)                                   # island in front of the centre
    g.rect(14, 10, 22, 11)
    g.mirror()
    # --- centre ---
    g.rect(68, 40, 81, 41)                                   # flat hill
    g.plat(70, 27, 10)
    g.rect(66, 13, 83, 14, '%')                              # rock at the top in the centre
    # pickups (red on the left, mirrored for blue)
    for x, y, c in [(3, 18, 'L'), (40, 27, 'G'), (31, 37, 'h'), (17, 9, 'a'), (61, 32, 'h'), (47, 16, 'a'), (25, 41, 'h')]:
        g.set(x, y, c); g.set(W - 1 - x, y, c)
    g.set(74, 26, 'G'); g.set(75, 26, 'G')
    g.set(74, 39, 'h'); g.set(75, 39, 'h')
    # spawns red (blue mirrored)
    for x, y in [(8, 41), (11, 41), (14, 41), (17, 41), (5, 29), (9, 29), (3, 18), (7, 18)]:
        g.set(x, y, 'R'); g.set(W - 1 - x, y, 'B')
    # laser in the lookout: replaces the spawn there? better put the laser next to it
    g.set(3, 18, 'L'); g.set(W - 4, 18, 'L')
    g.set(2, 18, 'R'); g.set(W - 3, 18, 'B')
    return g
if __name__ == '__main__':
    g = build()
    e, u = check(g, ctf=True, name='Wald')
    print(u[:30])
    print('\n'.join(g.rows()))

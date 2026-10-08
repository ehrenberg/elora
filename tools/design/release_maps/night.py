from kit import Grid, check
def build():
    W, H = 190, 64
    g = Grid(W, H)
    # --- left half (red) ---
    g.rect(1, 58, 94, 62)                                     # ground
    # lower tunnel: ceiling above the lower route, conveyor towards the centre
    g.rect(30, 50, 94, 51)                                    # tunnel ceiling
    g.put(40, 58, '>' * 30)                                   # conveyor in the tunnel
    # castle: stone outside (unhookable), earth inside
    g.rect(1, 36, 26, 37, '%')                                # castle ceiling
    g.rect(24, 38, 26, 47, '%')                               # outer wall with gate at the bottom
    g.rect(1, 50, 8, 57)                                      # flag pedestal
    g.set(4, 49, 'r')
    g.plat(10, 50, 8); g.plat(14, 43, 8)
    g.put(18, 50, '!')                                        # jump pad in the castle → through the hatch onto the roof
    g.rect(18, 36, 21, 37, '.')                               # hatch in the castle ceiling
    # towers
    g.rect(1, 22, 12, 24, '%'); g.rect(1, 25, 4, 35, '%')
    g.plat(6, 30, 6)
    # field (middle route)
    g.rect(34, 44, 44, 49); g.rect(48, 40, 56, 41)
    g.plat(36, 34, 9); g.plat(52, 30, 8)
    g.rect(62, 38, 74, 40)
    g.rect(80, 44, 94, 45)
    # upper route: bridges
    g.rect(16, 14, 30, 15); g.plat(36, 18, 10); g.plat(52, 13, 10); g.plat(68, 18, 10)
    g.rect(84, 24, 94, 25, '%')
    g.mirror()
    # centre
    g.plat(82, 15, 26)                                         # central bridge at the top
    # pickups
    for x, y, c in [(2, 21, 'L'), (45, 33, 'G'), (40, 43, 'h'), (22, 13, 'a'), (68, 37, 'h'),
                    (57, 12, 'a'), (52, 57, 'h'), (17, 42, 'a'), (72, 17, 'G')]:
        g.set(x, y, c); g.set(W - 1 - x, y, c)
    g.set(94, 14, 'L'); g.set(95, 14, 'L')
    g.set(94, 43, 'h'); g.set(95, 43, 'h')
    # spawns red (blue mirrored)
    for x, y in [(10, 57), (13, 57), (16, 57), (12, 49), (15, 49), (16, 42), (19, 42), (8, 29),
                 (6, 21), (9, 21), (6, 35), (14, 35)]:
        g.set(x, y, 'R'); g.set(W - 1 - x, y, 'B')
    return g
if __name__ == '__main__':
    g = build()
    e, u = check(g, ctf=True, name='Night')
    print(u[:30])
    print('\n'.join(g.rows()))

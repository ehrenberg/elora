from kit import Grid, check
def build():
    W, H = 64, 36
    g = Grid(W, H)
    # ground with gentle hills
    g.rect(1, 31, 62, 34)
    g.rect(1, 29, 9, 30); g.rect(54, 29, 62, 30)          # elevations at the sides
    g.rect(24, 30, 39, 30)                                # flat hump in the centre
    g.rect(1, 28, 4, 28); g.rect(59, 28, 62, 28)
    # jump pads bottom left/right → up to the side ledges
    g.put(12, 31, '\\\\'); g.put(50, 31, '//')   # diagonally to the side ledges
    # side ledges on the walls (middle level)
    g.rect(1, 20, 8, 21); g.rect(55, 20, 62, 21)
    # central island with passage
    g.rect(22, 21, 41, 23); g.rect(30, 21, 33, 23, '.')   # hole to fall through
    # platforms between the levels
    g.plat(13, 25, 6); g.plat(45, 25, 6)
    g.plat(12, 15, 7); g.plat(45, 15, 7)
    # upper islands left/right and central bridge
    g.rect(3, 9, 13, 11); g.rect(50, 9, 60, 11)
    g.plat(24, 12, 16)
    g.rect(29, 6, 34, 7, '%')                             # cap above the centre (unhookable)
    # pickups
    g.set(31, 11, 'L')                                    # laser on the central bridge
    g.set(32, 29, 'G')                                    # grenade at the bottom in the centre
    g.set(27, 20, 'h'); g.set(36, 20, 'h')                # hearts on the central island
    g.set(4, 19, 'a'); g.set(59, 19, 'a')                 # armour on the ledges
    g.set(8, 8, 'h'); g.set(55, 8, 'h')
    # spawns
    for x, y in [(3, 27), (60, 27), (18, 30), (45, 30), (6, 8), (57, 8), (25, 20), (38, 20)]:
        g.set(x, y, 'S')
    return g
if __name__ == '__main__':
    g = build()
    check(g, name='Meadow')
    print('\n'.join(g.rows()))

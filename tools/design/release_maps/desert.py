from kit import Grid, check
def build():
    W, H = 96, 48
    g = Grid(W, H)
    # build the left half, then mirror
    g.rect(1, 42, 47, 46)                                  # ground
    g.rect(1, 36, 7, 41); g.rect(8, 39, 12, 41)            # steps at the wall
    g.rect(18, 41, 26, 41)                                 # dune
    g.rect(38, 37, 47, 46, '.')                            # pit in the centre (left)
    g.rect(36, 37, 37, 41)                                 # pit rim
    g.rect(38, 45, 47, 45, '^')                            # spikes at the bottom
    g.rect(38, 46, 47, 46)
    g.rect(1, 25, 9, 27)                                   # ledge left
    g.plat(14, 34, 8); g.plat(24, 29, 7)
    g.rect(14, 20, 24, 22)                                 # floating dune
    g.plat(30, 15, 8)
    g.rect(2, 11, 9, 12)                                   # upper corner
    g.put(5, 36, '!')                                      # jump pad on the step → ledge
    g.mirror(swap=False)
    # centre: two conveyors above the pit, running in opposite directions
    g.put(36, 36, '>' * 24)                                # lower one to the right
    g.put(38, 25, '<' * 20)                                # upper one to the left
    g.rect(40, 10, 55, 12)                                 # central island at the top
    g.rect(44, 6, 51, 7, '%')
    # pickups
    g.set(47, 9, 'L'); g.set(48, 9, 'G')
    g.set(47, 35, 'h'); g.set(48, 35, 'h')
    g.set(19, 19, 'a'); g.set(76, 19, 'a')
    g.set(5, 10, 'h'); g.set(90, 10, 'h')
    g.set(22, 40, 'G'); g.set(73, 40, 'L')
    g.set(4, 24, 'a'); g.set(91, 24, 'a')
    # spawns
    for x, y in [(2, 35), (93, 35), (16, 41), (79, 41), (30, 41), (65, 41), (17, 19), (78, 19),
                 (6, 24), (89, 24), (42, 9), (53, 9)]:
        g.set(x, y, 'S')
    return g
if __name__ == '__main__':
    g = build()
    check(g, name='Desert')
    print('\n'.join(g.rows()))

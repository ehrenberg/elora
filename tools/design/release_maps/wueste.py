from kit import Grid, check
def build():
    W, H = 96, 48
    g = Grid(W, H)
    # linke Hälfte bauen, dann spiegeln
    g.rect(1, 42, 47, 46)                                  # Boden
    g.rect(1, 36, 7, 41); g.rect(8, 39, 12, 41)            # Stufen an der Wand
    g.rect(18, 41, 26, 41)                                 # Düne
    g.rect(38, 37, 47, 46, '.')                            # Grube in der Mitte (links)
    g.rect(36, 37, 37, 41)                                 # Grubenrand
    g.rect(38, 45, 47, 45, '^')                            # Stacheln am Grund
    g.rect(38, 46, 47, 46)
    g.rect(1, 25, 9, 27)                                   # Sims links
    g.plat(14, 34, 8); g.plat(24, 29, 7)
    g.rect(14, 20, 24, 22)                                 # schwebende Düne
    g.plat(30, 15, 8)
    g.rect(2, 11, 9, 12)                                   # obere Ecke
    g.put(5, 36, '!')                                      # Sprungfeld auf der Stufe → Sims
    g.mirror(swap=False)
    # Mitte: zwei Laufbänder über der Grube, gegenläufig
    g.put(36, 36, '>' * 24)                                # unten nach rechts
    g.put(38, 25, '<' * 20)                                # oben nach links
    g.rect(40, 10, 55, 12)                                 # Mittelinsel oben
    g.rect(44, 6, 51, 7, '%')
    # Pickups
    g.set(47, 9, 'L'); g.set(48, 9, 'G')
    g.set(47, 35, 'h'); g.set(48, 35, 'h')
    g.set(19, 19, 'a'); g.set(76, 19, 'a')
    g.set(5, 10, 'h'); g.set(90, 10, 'h')
    g.set(22, 40, 'G'); g.set(73, 40, 'L')
    g.set(4, 24, 'a'); g.set(91, 24, 'a')
    # Spawns
    for x, y in [(2, 35), (93, 35), (16, 41), (79, 41), (30, 41), (65, 41), (17, 19), (78, 19),
                 (6, 24), (89, 24), (42, 9), (53, 9)]:
        g.set(x, y, 'S')
    return g
if __name__ == '__main__':
    g = build()
    check(g, name='Wüste')
    print('\n'.join(g.rows()))

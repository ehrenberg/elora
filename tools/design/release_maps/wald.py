from kit import Grid, check
def build():
    W, H = 150, 48
    g = Grid(W, H)
    # --- linke Hälfte (Rot), dann gespiegelt ---
    g.rect(1, 42, 74, 46)                                    # Boden
    # Basis: Festung an der Wand
    g.rect(1, 30, 22, 31)                                    # Decke der Basis
    g.rect(20, 32, 22, 37)                                   # Innenwand mit Durchgang unten
    g.rect(1, 38, 6, 41)                                     # Sockel für die Flagge
    g.set(4, 37, 'r')
    g.rect(1, 19, 10, 21)                                    # Ausguck über der Basis
    g.plat(12, 25, 8)
    g.put(24, 42, '/')                                       # Sprungfeld vor der Basis hinauf
    # Feld vor der Basis
    g.rect(28, 40, 36, 41); g.rect(31, 38, 34, 39)           # Hügel
    g.rect(44, 36, 52, 37, '%')                              # Fels (nicht hookbar)
    g.plat(26, 33, 7); g.plat(38, 28, 8)
    # obere Route: Baumhaus-Plattformen
    g.plat(24, 21, 9); g.plat(40, 17, 8); g.plat(54, 21, 9)
    g.rect(58, 33, 66, 35)                                   # Insel vor der Mitte
    g.rect(14, 10, 22, 11)
    g.mirror()
    # --- Mitte ---
    g.rect(68, 40, 81, 41)                                   # flacher Hügel
    g.plat(70, 27, 10)
    g.rect(66, 13, 83, 14, '%')                              # Fels oben in der Mitte
    # Pickups (Rot links, gespiegelt für Blau)
    for x, y, c in [(3, 18, 'L'), (40, 27, 'G'), (31, 37, 'h'), (17, 9, 'a'), (61, 32, 'h'), (47, 16, 'a'), (25, 41, 'h')]:
        g.set(x, y, c); g.set(W - 1 - x, y, c)
    g.set(74, 26, 'G'); g.set(75, 26, 'G')
    g.set(74, 39, 'h'); g.set(75, 39, 'h')
    # Spawns Rot (Blau gespiegelt)
    for x, y in [(8, 41), (11, 41), (14, 41), (17, 41), (5, 29), (9, 29), (3, 18), (7, 18)]:
        g.set(x, y, 'R'); g.set(W - 1 - x, y, 'B')
    # Laser im Ausguck: Spawn dort ersetzt? Laser lieber daneben
    g.set(3, 18, 'L'); g.set(W - 4, 18, 'L')
    g.set(2, 18, 'R'); g.set(W - 3, 18, 'B')
    return g
if __name__ == '__main__':
    g = build()
    e, u = check(g, ctf=True, name='Wald')
    print(u[:30])
    print('\n'.join(g.rows()))

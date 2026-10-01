from kit import Grid, check
def build():
    W, H = 64, 36
    g = Grid(W, H)
    # Boden mit sanften Hügeln
    g.rect(1, 31, 62, 34)
    g.rect(1, 29, 9, 30); g.rect(54, 29, 62, 30)          # Erhöhungen an den Seiten
    g.rect(24, 30, 39, 30)                                # flacher Buckel in der Mitte
    g.rect(1, 28, 4, 28); g.rect(59, 28, 62, 28)
    # Sprungfelder unten links/rechts → hinauf zu den Seitensimsen
    g.put(12, 31, '\\\\'); g.put(50, 31, '//')   # schräg zu den Seitensimsen
    # Seitensimse an den Wänden (mittlere Ebene)
    g.rect(1, 20, 8, 21); g.rect(55, 20, 62, 21)
    # Mittelinsel mit Durchgang
    g.rect(22, 21, 41, 23); g.rect(30, 21, 33, 23, '.')   # Loch zum Durchfallen
    # Plattformen zwischen den Ebenen
    g.plat(13, 25, 6); g.plat(45, 25, 6)
    g.plat(12, 15, 7); g.plat(45, 15, 7)
    # obere Inseln links/rechts und Mittelbrücke
    g.rect(3, 9, 13, 11); g.rect(50, 9, 60, 11)
    g.plat(24, 12, 16)
    g.rect(29, 6, 34, 7, '%')                             # Kappe über der Mitte (nicht hookbar)
    # Pickups
    g.set(31, 11, 'L')                                    # Laser auf der Mittelbrücke
    g.set(32, 29, 'G')                                    # Granate unten in der Mitte
    g.set(27, 20, 'h'); g.set(36, 20, 'h')                # Herzen auf der Mittelinsel
    g.set(4, 19, 'a'); g.set(59, 19, 'a')                 # Rüstung auf den Simsen
    g.set(8, 8, 'h'); g.set(55, 8, 'h')
    # Spawns
    for x, y in [(3, 27), (60, 27), (18, 30), (45, 30), (6, 8), (57, 8), (25, 20), (38, 20)]:
        g.set(x, y, 'S')
    return g
if __name__ == '__main__':
    g = build()
    check(g, name='Wiese')
    print('\n'.join(g.rows()))

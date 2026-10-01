from kit import Grid, check
def build():
    W, H = 128, 64
    g = Grid(W, H)
    # --- linke Hälfte ---
    g.rect(1, 58, 63, 62)                                   # Boden
    g.rect(22, 57, 40, 57, '~'); g.rect(22, 58, 40, 58, '~') # Eissee (rutschig)
    g.rect(1, 50, 10, 57); g.rect(11, 54, 15, 57)           # Hang an der Wand
    g.rect(44, 55, 52, 57)                                  # Hügel vor der Mitte
    # Ebene 2 (≈ 45)
    g.rect(1, 40, 14, 42)
    g.plat(18, 49, 8); g.plat(30, 47, 9)
    g.rect(20, 40, 36, 42); g.rect(26, 40, 30, 42, '.')     # Brücke mit Loch
    g.rect(40, 43, 46, 44, '%')                              # Steinblock (nicht hookbar)
    # Ebene 3 (≈ 30)
    g.plat(6, 34, 8); g.plat(40, 35, 7)
    g.rect(12, 27, 30, 29)
    g.rect(1, 20, 7, 22)
    g.plat(34, 24, 10)
    # Ebene 4 (oben)
    g.rect(14, 13, 26, 15); g.plat(30, 17, 8)
    g.rect(40, 9, 52, 10, '%')
    g.mirror(swap=False)
    # --- Mitte: Schacht mit Sprungfeldern (je ≈ 12 Tiles, Eingänge seitlich auf jeder Stufe) ---
    g.rect(58, 50, 69, 57, '.')
    g.rect(58, 57, 69, 57)
    g.put(62, 57, '!!!!')                                   # Stufe 1 (unten)
    g.plat(58, 45, 12); g.put(62, 45, '!!!!')               # Stufe 2
    g.plat(58, 33, 12); g.put(62, 33, '!!!!')               # Stufe 3
    for y0, y1 in [(24, 29), (36, 41), (48, 52)]:            # Wände mit Lücken über jeder Stufe
        g.rect(56, y0, 57, y1); g.rect(70, y0, 71, y1)
    g.rect(54, 20, 73, 21); g.rect(60, 20, 67, 21, '.')     # obere Mittelbrücke mit Öffnung
    # Pickups
    g.set(57, 19, 'L'); g.set(70, 19, 'G')
    g.set(59, 56, 'h'); g.set(68, 56, 'a')
    g.set(33, 39, 'G'); g.set(94, 39, 'L')
    g.set(4, 49, 'a'); g.set(123, 49, 'a')
    g.set(20, 12, 'h'); g.set(107, 12, 'h')
    g.set(31, 56, 'h'); g.set(96, 56, 'h')
    g.set(3, 19, 'L'); g.set(124, 19, 'G')
    g.set(20, 26, 'a'); g.set(107, 26, 'a')
    # Spawns
    for x, y in [(3, 49), (124, 49), (18, 57), (109, 57), (48, 54), (79, 54), (8, 39), (119, 39),
                 (24, 39), (103, 39), (15, 26), (112, 26), (5, 19), (122, 19), (17, 12), (110, 12)]:
        g.set(x, y, 'S')
    return g
if __name__ == '__main__':
    g = build()
    e, u = check(g, name='Winter')
    print(u[:30])
    print('\n'.join(g.rows()))

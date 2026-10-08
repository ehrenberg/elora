"""Construction kit for the release maps: grid with basic shapes, mirroring, checks."""
SOLID = set('#%~!\\/<>')     # solid (incl. ice, jump pads, boosters)
GROUND = SOLID | {'='}       # can be stood on
ENTS = set('SRBrbhaLGDWJX')

class Grid:
    def __init__(s, w, h):
        s.w, s.h = w, h
        s.g = [['.'] * w for _ in range(h)]
        for x in range(w):
            s.g[0][x] = s.g[h-1][x] = '#'
        for y in range(h):
            s.g[y][0] = s.g[y][w-1] = '#'
    def ok(s, x, y): return 0 <= x < s.w and 0 <= y < s.h
    def at(s, x, y): return s.g[y][x] if s.ok(x, y) else '#'
    def set(s, x, y, c):
        if s.ok(x, y): s.g[y][x] = c
    def rect(s, x0, y0, x1, y1, c='#'):
        for y in range(min(y0,y1), max(y0,y1)+1):
            for x in range(min(x0,x1), max(x0,x1)+1):
                s.set(x, y, c)
    def plat(s, x, y, n): s.rect(x, y, x+n-1, y, '=')
    def put(s, x, y, text):
        for i, c in enumerate(text): s.set(x+i, y, c)
    def mirror(s, swap=True):
        """Right half = mirrored left half (red ↔ blue, directions swapped)."""
        sw = {'R':'B','B':'R','r':'b','b':'r','\\':'/','/':'\\','<':'>','>':'<'} if swap else {'\\':'/','/':'\\','<':'>','>':'<'}
        for y in range(s.h):
            for x in range(s.w // 2):
                c = s.g[y][x]
                s.g[y][s.w-1-x] = sw.get(c, c)
    def rows(s): return [''.join(r) for r in s.g]

def check(grid, ctf=False, name=''):
    g, w, h = grid, grid.w, grid.h
    errs = []
    stand = set()
    for y in range(h):
        for x in range(w):
            c = g.at(x, y)
            if c in ENTS:
                if g.at(x, y+1) not in GROUND and c in 'SRBrb':
                    errs.append(f'{c} bei {x},{y} steht nicht auf Boden')
                if c in 'SRB' and g.at(x, y-1) not in '.' + ''.join(ENTS):
                    errs.append(f'{c} bei {x},{y}: kein Platz über dem Spawn')
            if (c == '.' or c in ENTS) and g.at(x, y+1) in GROUND and g.at(x, y-1) not in SOLID:
                stand.add((x, y))
    # Reachability: from standing spot to standing spot if the path (straight line) is clear and
    # at most 9 tiles high (jump + double jump) and 9 wide; any distance downwards.
    def free(a, b):
        (x0, y0), (x1, y1) = a, b
        n = max(abs(x1-x0), abs(y1-y0), 1)
        for i in range(n+1):
            x = round(x0 + (x1-x0)*i/n); y = round(y0 + (y1-y0)*i/n)
            if g.at(x, y) in SOLID: return False
            # head: the figure is about one tile high
        return True
    nodes = sorted(stand)
    import collections
    byx = collections.defaultdict(list)
    for n in nodes: byx[n[0]].append(n)
    def nbrs(n):
        x, y = n
        for dx in range(-9, 10):
            for m in byx.get(x+dx, []):
                dy = m[1] - y
                if m != n and dy >= -9 and (dy >= 0 or abs(dx) <= 9) and free(n, m):
                    # jump pads throw further upwards
                    yield m
        # jump pad below n: up to 14 tiles high
        if g.at(x, y+1) in '!\\/':
            for dx in range(-8, 9):
                for m in byx.get(x+dx, []):
                    if m[1] < y and y - m[1] <= 14 and free(n, m): yield m
    start = next(((x,y) for (x,y) in nodes if g.at(x,y) in 'SR'), None)
    seen = {start}; todo = [start]
    while todo:
        n = todo.pop()
        for m in nbrs(n):
            if m not in seen: seen.add(m); todo.append(m)
    for y in range(h):
        for x in range(w):
            c = g.at(x, y)
            if c in ENTS:
                # entity must be within reach of a reachable standing spot
                near = any(abs(x-a) <= 2 and 0 <= b - y <= 3 or (a, b) == (x, y) for (a, b) in seen)
                if not near: errs.append(f'{c} bei {x},{y} nicht erreichbar')
    unreach = [n for n in nodes if n not in seen]
    if ctf:
        for y in range(h):
            for x in range(w):
                c, d = g.at(x, y), g.at(w-1-x, y)
                sw = {'R':'B','B':'R','r':'b','b':'r','\\':'/','/':'\\','<':'>','>':'<'}
                if sw.get(c, c) != d: errs.append(f'nicht gespiegelt bei {x},{y}'); break
    counts = {k: sum(r.count(k) for r in grid.rows()) for k in 'SRBrbhaLG'}
    print(f'== {name} {w}×{h}: Standplätze {len(nodes)}, erreichbar {len(seen)}, unerreichbar {len(unreach)}; {counts}')
    for e in errs[:20]: print('  FEHLER', e)
    return errs, unreach

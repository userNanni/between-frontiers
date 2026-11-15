"""Small raster helpers in plain numpy (the tools run under `python3 -I`, without scipy)."""

import numpy as np


def fill_nearest(grid, steps):
    """Fills zero cells with a non-zero 8-neighbour, growing `steps` cells out from the data."""
    g = grid.copy()
    for _ in range(steps):
        empty = g == 0
        if not empty.any():
            break
        for dy, dx in ((0, 1), (0, -1), (1, 0), (-1, 0), (1, 1), (1, -1), (-1, 1), (-1, -1)):
            src = np.roll(np.roll(g, dy, 0), dx, 1)
            take = empty & (g == 0) & (src != 0)
            g[take] = src[take]
    return g


def _find(parent, a):
    root = a
    while parent[root] != root:
        root = parent[root]
    while parent[a] != root:
        parent[a], a = root, parent[a]
    return root


def label_runs(mask):
    """4-connected components of a boolean image whose columns wrap around (longitude).

    Returns (labels, n) with labels 0 for False and 1..n for components.
    """
    h, w = mask.shape
    run_rows, run_starts, run_ends = [], [], []
    for y in range(h):
        row = mask[y].astype(np.int8)
        d = np.diff(np.concatenate(([0], row, [0])))
        starts = np.nonzero(d == 1)[0]
        ends = np.nonzero(d == -1)[0]  # exclusive
        run_rows.append(np.full(len(starts), y))
        run_starts.append(starts)
        run_ends.append(ends)
    offsets = np.cumsum([0] + [len(s) for s in run_starts])
    nruns = int(offsets[-1])
    parent = list(range(nruns))

    def union(a, b):
        ra, rb = _find(parent, a), _find(parent, b)
        if ra != rb:
            parent[max(ra, rb)] = min(ra, rb)

    for y in range(h):
        s, e, o = run_starts[y], run_ends[y], offsets[y]
        if len(s) > 1 and s[0] == 0 and e[-1] == w:
            union(o, o + len(s) - 1)
        if y == 0 or len(s) == 0:
            continue
        ps, pe, po = run_starts[y - 1], run_ends[y - 1], offsets[y - 1]
        if len(ps) == 0:
            continue
        # Runs of the row above that overlap each run: ps < e and pe > s.
        lo = np.searchsorted(pe, s, side="right")
        hi = np.searchsorted(ps, e, side="left")
        for i in np.nonzero(hi > lo)[0]:
            for j in range(lo[i], hi[i]):
                union(o + i, po + j)
    roots = np.array([_find(parent, i) for i in range(nruns)], dtype=np.int64)
    uniq, comp = np.unique(roots, return_inverse=True)
    labels = np.zeros((h, w), dtype=np.int32)
    for y in range(h):
        o = offsets[y]
        for i, (a, b) in enumerate(zip(run_starts[y], run_ends[y])):
            labels[y, a:b] = comp[o + i] + 1
    return labels, len(uniq)


def nearest_true(mask, xs, ys, max_radius=400):
    """For each (x, y), the nearest True pixel of `mask`, searching growing square windows."""
    h, w = mask.shape
    out_y = np.zeros(len(xs), dtype=np.int64)
    out_x = np.zeros(len(xs), dtype=np.int64)
    for k, (x, y) in enumerate(zip(xs, ys)):
        r = 2
        while True:
            y0, y1 = max(0, y - r), min(h, y + r + 1)
            cols = np.arange(x - r, x + r + 1) % w
            win = mask[y0:y1][:, cols]
            yy, xx = np.nonzero(win)
            if len(yy) or r >= max_radius:
                break
            r *= 2
        if not len(yy):
            raise ValueError(f"no land near pixel {x},{y}")
        d = (yy + y0 - y) ** 2 + (xx - r) ** 2
        i = int(np.argmin(d))
        out_y[k], out_x[k] = yy[i] + y0, cols[xx[i]]
    return out_y, out_x

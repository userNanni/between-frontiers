"""Sums HYDE 3.3 population and urban population of 1925 onto the 870 states of the map.

Run from the repository root:

    python3 -I tools/initial/aggregate.py

Inputs (in data/raw/, which git ignores; see data/scenario/1925/INITIAL.md):
- data/raw/hyde33/asc/{popc,urbc}_{1920,1930}AD.asc (HYDE 3.3 baseline, 1920AD_pop.zip and
  1930AD_pop.zip; people per 5' cell)
- assets/map/provinces.png and provinces.ron (the game map)

Output: tools/initial/state_hyde.csv, one row per state (people in 1925 by HYDE, total and urban),
consumed by tools/initial/population.py.

Method: 1925 is the mean of the 1920 and 1930 grids (linear interpolation). Each HYDE cell's
people are spread over the map's land pixels inside the cell in proportion to their true area, so
the world total is kept; cells with people but no land pixel in our map (coastline differences)
go whole to the state of the nearest land pixel. Same geometry as tools/land/aggregate.py.
"""

import csv
import math
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "land"))
from aggregate import EARTH_RADIUS_KM, parse_map  # noqa: E402
from hyde import read_asc  # noqa: E402
from raster import nearest_true  # noqa: E402

ROOT = os.getcwd()
ASC = os.path.join(ROOT, "data", "raw", "hyde33", "asc")
OUT = os.path.join(HERE, "state_hyde.csv")
CHUNK = 256


def main():
    Image.MAX_IMAGE_PIXELS = None
    frame, provinces, states = parse_map(os.path.join(ROOT, "assets/map/provinces.ron"))
    assert len(states) == 870, len(states)
    ids_img = np.array(Image.open(os.path.join(ROOT, "assets/map/provinces.png"))).astype(np.int32)
    w, h = frame.width, frame.height
    nst = len(states)
    sidx = {s["id"]: i for i, s in enumerate(states)}
    state_of_prov = np.full(max(provinces) + 1, -1, dtype=np.int32)
    for pid, p in provinces.items():
        state_of_prov[pid] = sidx[p["state"]]

    lon_c = frame.lon_of_col(np.arange(w) + 0.5)
    lat_c = frame.lat_of_row(np.arange(h) + 0.5)
    lat_e = frame.lat_of_row(np.arange(h + 1))
    row_area = EARTH_RADIUS_KM**2 * (2 * math.pi / w) * (
        np.sin(np.radians(lat_e[:-1])) - np.sin(np.radians(lat_e[1:]))
    )
    hy_col = np.clip(((lon_c + 180) / 360 * 4320).astype(np.int64), 0, 4319)
    hy_row = np.clip(((90 - lat_c) / 180 * 2160).astype(np.int64), 0, 2159)

    grids = {}
    for v in ("popc", "urbc"):
        a = read_asc(os.path.join(ASC, f"{v}_1920AD.asc")).astype(np.float64)
        b = read_asc(os.path.join(ASC, f"{v}_1930AD.asc")).astype(np.float64)
        grids[v] = (0.5 * (a + b)).reshape(-1)
    grids["urbc"] = np.minimum(grids["urbc"], grids["popc"])

    cell_land = np.zeros(4320 * 2160)
    for y0 in range(0, h, CHUNK):
        y1 = min(h, y0 + CHUNK)
        ids = ids_img[y0:y1]
        land = ids > 0
        area = np.broadcast_to(row_area[y0:y1, None], ids.shape)[land]
        cell = (hy_row[y0:y1, None] * 4320 + hy_col[None, :])[land]
        cell_land += np.bincount(cell, weights=area, minlength=4320 * 2160)

    out = {k: np.zeros(nst) for k in grids}
    for y0 in range(0, h, CHUNK):
        y1 = min(h, y0 + CHUNK)
        ids = ids_img[y0:y1]
        land = ids > 0
        st = state_of_prov[ids[land]]
        area = np.broadcast_to(row_area[y0:y1, None], ids.shape)[land]
        cell = (hy_row[y0:y1, None] * 4320 + hy_col[None, :])[land]
        frac = area / cell_land[cell]
        for k, g in grids.items():
            out[k] += np.bincount(st, weights=frac * g[cell], minlength=nst)

    orphan = np.nonzero((cell_land == 0) & (grids["popc"] > 0))[0]
    orow, ocol = orphan // 4320, orphan % 4320
    olon = -180 + (ocol + 0.5) * (360 / 4320)
    olat = 90 - (orow + 0.5) * (180 / 2160)
    ox, oy = frame.to_pixel(olon, olat)
    ny, nx = nearest_true(ids_img > 0, ox.astype(np.int64) % w, np.clip(oy, 0, h - 1).astype(np.int64))
    ost = state_of_prov[ids_img[ny, nx]]
    for k, g in grids.items():
        out[k] += np.bincount(ost, weights=g[orphan], minlength=nst)

    world = grids["popc"].sum()
    print(f"HYDE 1925: world {world / 1e6:.1f} M, urban {grids['urbc'].sum() / 1e6:.1f} M, "
          f"states {out['popc'].sum() / 1e6:.1f} M, via nearest land {grids['popc'][orphan].sum() / 1e6:.2f} M")
    with open(OUT, "w", newline="", encoding="utf-8") as f:
        wr = csv.writer(f)
        wr.writerow(["state", "name", "owner", "hyde_pop", "hyde_urban"])
        for i, s in enumerate(states):
            wr.writerow([s["key"], s["name"], s["owner"], f"{out['popc'][i]:.0f}", f"{out['urbc'][i]:.0f}"])
    print("wrote", OUT)


if __name__ == "__main__":
    main()

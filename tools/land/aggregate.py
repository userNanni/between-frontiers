"""Aggregates climate and land use onto the 870 states of the 1925 map.

Run from the repository root:

    python3 -I tools/land/aggregate.py

Inputs (downloaded into data/raw/, which git ignores; see data/scenario/1925/LAND.md):
- data/raw/koppen/Beck_KG_V1_present_0p0083.tif and _0p083.tif (Beck et al. 2018, CC BY 4.0)
- data/raw/hyde33/asc/*.asc (HYDE 3.3 baseline, 1920AD_lu.zip, 1930AD_lu.zip, general_files.zip)
- assets/map/provinces.png and provinces.ron (the game map)

Output: tools/land/state_stats.csv, one row per state, consumed by tools/land/farms.py.

Method: every land pixel of provinces.png (Miller projection) is turned back into lon/lat and
weighted by its true area on the sphere. Climate: the 1 km Köppen class at the pixel centre.
Land use: each HYDE 5' cell's km² are spread over the map's land pixels inside that cell, in
proportion to their area, so world totals are kept; cells with no land pixel in our map (coastline
differences) go whole to the state of the nearest coast pixel.
"""

import csv
import math
import os
import re
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hyde import read_asc  # noqa: E402
from raster import fill_nearest, label_runs, nearest_true  # noqa: E402

ROOT = os.getcwd()
RAW = os.path.join(ROOT, "data", "raw")
OUT = os.path.join(ROOT, "tools", "land", "state_stats.csv")
EARTH_RADIUS_KM = 6371.0088
CHUNK = 256
# A water body counts as sea when it is this large (the Caspian: 371 000 km²) or within
# STRAIT_PX pixels of one (straits narrower than the map's pixels close up in the raster).
SEA_MIN_KM2 = 250_000.0
STRAIT_PX = 3
KOPPEN = [
    "Af", "Am", "Aw", "BWh", "BWk", "BSh", "BSk", "Csa", "Csb", "Csc", "Cwa", "Cwb", "Cwc",
    "Cfa", "Cfb", "Cfc", "Dsa", "Dsb", "Dsc", "Dsd", "Dwa", "Dwb", "Dwc", "Dwd", "Dfa", "Dfb",
    "Dfc", "Dfd", "ET", "EF",
]
# HYDE layers kept, averaged between 1920 and 1930.
HYDE_VARS = [
    "cropland", "grazing", "pasture", "rangeland", "conv_rangeland", "tot_irri", "tot_rice",
    "ir_rice",
]


def miller_y(lat):
    return 1.25 * np.log(np.tan(math.pi / 4 + 0.4 * np.radians(lat)))


def miller_lat(y):
    return np.degrees(2.5 * np.arctan(np.exp(0.8 * y)) - 0.625 * math.pi)


class Frame:
    """Same frame as crates/bf-mapgen/src/projection.rs."""

    def __init__(self, width, height, lat_top):
        self.width, self.height = width, height
        self.scale = width / (2 * math.pi)
        self.y_top = float(miller_y(lat_top))

    def lat_of_row(self, y):
        return miller_lat(self.y_top - np.asarray(y, dtype=np.float64) / self.scale)

    def lon_of_col(self, x):
        return np.degrees(np.asarray(x, dtype=np.float64) / self.scale - math.pi)

    def to_pixel(self, lon, lat):
        x = (np.radians(lon) + math.pi) * self.scale
        y = (self.y_top - miller_y(lat)) * self.scale
        return x, y


def parse_map(path):
    text = open(path, encoding="utf-8").read()
    meta = re.search(r"width: (\d+),\s*height: (\d+),\s*projection: \"miller\",\s*lat_top: ([-\d.]+)", text)
    width, height, lat_top = int(meta.group(1)), int(meta.group(2)), float(meta.group(3))
    prov_part, state_part = text.split("\n    states: [")
    provinces = {}
    for m in re.finditer(
        r"\(id: (\d+), key: \"[^\"]*\", name: \"(?:[^\"\\]|\\.)*\", owner: \"[^\"]*\", "
        r"center: \(([-\d.]+), ([-\d.]+)\), area_km2: ([\d.]+), pixels: (\d+), colony: \w+, state: (\d+)\)",
        prov_part,
    ):
        provinces[int(m.group(1))] = dict(
            cx=float(m.group(2)), cy=float(m.group(3)), area=float(m.group(4)),
            pixels=int(m.group(5)), state=int(m.group(6)),
        )
    states = []
    for m in re.finditer(
        r"\(id: (\d+), key: \"([^\"]+)\", name: \"((?:[^\"\\]|\\.)*)\", owner: \"([^\"]+)\", colony: (\w+), "
        r"center: \(([-\d.]+), ([-\d.]+)\), area_km2: ([\d.]+), provinces: \[([\d, ]*)\]\)",
        state_part,
    ):
        states.append(dict(
            id=int(m.group(1)), key=m.group(2), name=m.group(3).replace("\\'", "'"),
            owner=m.group(4), cx=float(m.group(6)), cy=float(m.group(7)),
            area=float(m.group(8)), provinces=[int(p) for p in m.group(9).split(",") if p.strip()],
        ))
    return Frame(width, height, lat_top), provinces, states


def koppen_filled(path):
    """10 km Köppen map with the sea filled from the nearest land class (fallback for coasts)."""
    return fill_nearest(np.array(Image.open(path)), steps=30)


def main():
    Image.MAX_IMAGE_PIXELS = None
    frame, provinces, states = parse_map(os.path.join(ROOT, "assets/map/provinces.ron"))
    assert len(states) == 870, len(states)
    ids_img = np.array(Image.open(os.path.join(ROOT, "assets/map/provinces.png")))
    assert ids_img.shape == (frame.height, frame.width), ids_img.shape
    ids_img = ids_img.astype(np.int32)
    w, h = frame.width, frame.height
    nst = len(states)
    sidx = {s["id"]: i for i, s in enumerate(states)}
    max_pid = max(provinces) + 1
    state_of_prov = np.full(max_pid, -1, dtype=np.int32)
    for pid, p in provinces.items():
        state_of_prov[pid] = sidx[p["state"]]

    # Geometry: lon of each column, lat and true pixel area of each row.
    lon_c = frame.lon_of_col(np.arange(w) + 0.5)
    lat_c = frame.lat_of_row(np.arange(h) + 0.5)
    lat_e = frame.lat_of_row(np.arange(h + 1))
    row_area = EARTH_RADIUS_KM**2 * (2 * math.pi / w) * (
        np.sin(np.radians(lat_e[:-1])) - np.sin(np.radians(lat_e[1:]))
    )

    print("Köppen…", flush=True)
    k1 = np.array(Image.open(os.path.join(RAW, "koppen/Beck_KG_V1_present_0p0083.tif")))
    k10 = koppen_filled(os.path.join(RAW, "koppen/Beck_KG_V1_present_0p083.tif"))
    k1_col = np.clip(((lon_c + 180) / 360 * k1.shape[1]).astype(np.int64), 0, k1.shape[1] - 1)
    k1_row = np.clip(((90 - lat_c) / 180 * k1.shape[0]).astype(np.int64), 0, k1.shape[0] - 1)
    k10_col = np.clip(((lon_c + 180) / 360 * k10.shape[1]).astype(np.int64), 0, k10.shape[1] - 1)
    k10_row = np.clip(((90 - lat_c) / 180 * k10.shape[0]).astype(np.int64), 0, k10.shape[0] - 1)
    hy_col = np.clip(((lon_c + 180) / 360 * 4320).astype(np.int64), 0, 4319)
    hy_row = np.clip(((90 - lat_c) / 180 * 2160).astype(np.int64), 0, 2159)

    climate = np.zeros((nst, 31))
    raster_area = np.zeros(nst)
    cell_land = np.zeros(4320 * 2160)
    for y0 in range(0, h, CHUNK):
        y1 = min(h, y0 + CHUNK)
        ids = ids_img[y0:y1]
        land = ids > 0
        st = state_of_prov[ids[land]]
        area = np.broadcast_to(row_area[y0:y1, None], ids.shape)[land]
        k = k1[k1_row[y0:y1, None], k1_col[None, :]][land]
        fb = k10[k10_row[y0:y1, None], k10_col[None, :]][land]
        k = np.where(k == 0, fb, k).astype(np.int64)
        climate += np.bincount(st * 31 + k, weights=area, minlength=nst * 31).reshape(nst, 31)
        raster_area += np.bincount(st, weights=area, minlength=nst)
        cell = (hy_row[y0:y1, None] * 4320 + hy_col[None, :])[land]
        cell_land += np.bincount(cell, weights=area, minlength=4320 * 2160)
    del k1

    # States too small for a pixel: the climate at each province's centre.
    for i, s in enumerate(states):
        if raster_area[i] > 0:
            continue
        for pid in s["provinces"]:
            p = provinces[pid]
            r = int(np.clip(k10_row[min(h - 1, int(p["cy"]))], 0, k10.shape[0] - 1))
            c = int(k10_col[min(w - 1, int(p["cx"]))])
            climate[i, k10[r, c]] += max(p["area"], 1e-3)

    print("Sea and coast…", flush=True)
    water = ids_img == 0
    labels, n = label_runs(water)  # 4-connected, wrapping across the antimeridian
    comp_area = np.bincount(labels.reshape(-1), weights=np.broadcast_to(row_area[:, None], water.shape).reshape(-1),
                            minlength=n + 1)
    big = comp_area >= SEA_MIN_KM2
    big[0] = False
    # Reopen straits narrower than a few pixels: water bodies that come that close to a big one.
    sea = big[labels]
    near = sea.copy()
    for _ in range(STRAIT_PX):
        grown = near.copy()
        grown |= np.roll(near, 1, 1) | np.roll(near, -1, 1)
        grown[1:] |= near[:-1]
        grown[:-1] |= near[1:]
        near = grown
    is_sea_comp = np.zeros(n + 1, dtype=bool)
    is_sea_comp[np.unique(labels[near & water])] = True
    is_sea_comp[0] = False
    sea = is_sea_comp[labels]
    print(f"  water bodies {n}, seas {int(is_sea_comp.sum())}, lakes {n - int(is_sea_comp.sum())}")
    del labels, near
    land_all = ~water
    coast_px = np.zeros_like(land_all)
    for shifted in (np.roll(sea, 1, 1), np.roll(sea, -1, 1)):
        coast_px |= land_all & shifted
    coast_px[1:] |= land_all[1:] & sea[:-1]
    coast_px[:-1] |= land_all[:-1] & sea[1:]
    coast = np.zeros(nst, dtype=bool)
    cy, cx = np.nonzero(coast_px)
    coast[np.unique(state_of_prov[ids_img[cy, cx]])] = True
    # Coastline length: one pixel side per coastal pixel (a rough measure, for fisheries).
    coast_km = np.bincount(state_of_prov[ids_img[cy, cx]], weights=np.sqrt(row_area[cy]), minlength=nst)
    # Provinces with no pixel: coastal when their centre falls on the sea.
    for i, s in enumerate(states):
        for pid in s["provinces"]:
            p = provinces[pid]
            if p["pixels"] == 0:
                yy, xx = min(h - 1, int(p["cy"])), min(w - 1, int(p["cx"]))
                if sea[yy, xx]:
                    coast[i] = True
    del sea, coast_px, water, land_all

    print("HYDE…", flush=True)
    asc = os.path.join(RAW, "hyde33", "asc")
    grids = {}
    for v in HYDE_VARS:
        grids[v] = 0.5 * (read_asc(os.path.join(asc, f"{v}1920AD.asc")) + read_asc(os.path.join(asc, f"{v}1930AD.asc")))
    maxln = read_asc(os.path.join(asc, "maxln_cr.asc"))
    potf_flag = read_asc(os.path.join(asc, "forest_wwf_cr.asc")) > 0.5
    potforest = np.where(potf_flag, maxln, 0.0)
    # Forest in 1925: potential forest minus the land cleared for crops and grazing on it.
    # Cropland and intensive pasture are taken from forest and non-forest land in proportion
    # to the potential forest share of the cell; converted rangeland is by definition former
    # forest. Natural rangeland (grassland, savanna, shrub) is not forest land.
    # What is left is then scaled by the unused share of the cell, (1 - u): in a farmed landscape
    # the land outside fields and pastures is mostly villages, roads, fallow, heath and waste, not
    # woodland. Without this factor England kept 27 % forest and Berlin 78 %.
    fshare = np.where(maxln > 0, potforest / np.maximum(maxln, 1e-9), 0.0)
    used = grids["cropland"] + grids["pasture"] + grids["conv_rangeland"]
    cleared = (grids["cropland"] + grids["pasture"]) * fshare + grids["conv_rangeland"] * potf_flag
    unused = 1.0 - np.clip(used / np.maximum(maxln, 1e-9), 0.0, 1.0)
    grids["potforest"] = potforest
    grids["forest"] = np.clip(potforest - cleared, 0.0, None) * unused
    grids["maxln"] = maxln
    names = list(grids)
    flat = {k: g.reshape(-1).astype(np.float64) for k, g in grids.items()}

    out = {k: np.zeros(nst) for k in names}
    for y0 in range(0, h, CHUNK):
        y1 = min(h, y0 + CHUNK)
        ids = ids_img[y0:y1]
        land = ids > 0
        st = state_of_prov[ids[land]]
        area = np.broadcast_to(row_area[y0:y1, None], ids.shape)[land]
        cell = (hy_row[y0:y1, None] * 4320 + hy_col[None, :])[land]
        frac = area / cell_land[cell]
        for k in names:
            out[k] += np.bincount(st, weights=frac * flat[k][cell], minlength=nst)

    # Cells with land use but no land pixel in our map.
    orphan = np.nonzero((cell_land == 0) & ((flat["cropland"] + flat["grazing"] + flat["forest"]) > 0))[0]
    orow, ocol = orphan // 4320, orphan % 4320
    olon = -180 + (ocol + 0.5) * (360 / 4320)
    olat = 90 - (orow + 0.5) * (180 / 2160)
    ox, oy = frame.to_pixel(olon, olat)
    ny, nx = nearest_true(ids_img > 0, ox.astype(np.int64) % w, np.clip(oy, 0, h - 1).astype(np.int64))
    ost = state_of_prov[ids_img[ny, nx]]
    orphan_tot = {}
    for k in names:
        add = np.bincount(ost, weights=flat[k][orphan], minlength=nst)
        out[k] += add
        orphan_tot[k] = add.sum()

    for k in ("cropland", "grazing", "forest", "potforest", "tot_irri", "tot_rice"):
        print(f"  {k}: world {flat[k].sum() / 1e6:.3f} Mkm², states {out[k].sum() / 1e6:.3f}, "
              f"via nearest coast {orphan_tot[k] / 1e6:.4f}")

    with open(OUT, "w", newline="", encoding="utf-8") as f:
        wr = csv.writer(f)
        wr.writerow(["state", "name", "owner", "lat", "lon", "area_km2", "raster_km2", "coast", "coast_km"]
                    + KOPPEN + [f"{k}_km2" for k in names])
        for i, s in enumerate(states):
            tot = climate[i, 1:].sum()
            shares = climate[i, 1:] / tot if tot > 0 else np.zeros(30)
            lon = float(frame.lon_of_col(s["cx"] + 0.5))
            lat = float(frame.lat_of_row(s["cy"] + 0.5))
            wr.writerow([s["key"], s["name"], s["owner"], f"{lat:.2f}", f"{lon:.2f}", f"{s['area']:.1f}",
                         f"{raster_area[i]:.1f}", int(coast[i]), f"{coast_km[i]:.0f}"]
                        + [f"{x:.4f}" for x in shares] + [f"{out[k][i]:.1f}" for k in names])
    print("wrote", OUT, "states", nst, "coastal", int(coast.sum()),
          "without pixels", int((raster_area == 0).sum()))


if __name__ == "__main__":
    main()

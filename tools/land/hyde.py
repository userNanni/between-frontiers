"""Reads HYDE 3.3 ESRI ASCII grids (5 arc-minute, 4320 x 2160) into numpy, cached as .npy."""

import os

import numpy as np

NCOLS, NROWS = 4320, 2160


def read_asc(path):
    cache = path + ".npy"
    if os.path.exists(cache) and os.path.getmtime(cache) >= os.path.getmtime(path):
        return np.load(cache)
    with open(path, "rb") as f:
        header = {}
        for _ in range(6):
            k, v = f.readline().split()
            header[k.decode().lower()] = float(v)
        a = np.array(f.read().split(), dtype=np.float32)
    assert int(header["ncols"]) == NCOLS and int(header["nrows"]) == NROWS, path
    a = a.reshape(NROWS, NCOLS)
    a[a == header["nodata_value"]] = 0.0
    np.save(cache, a)
    return a

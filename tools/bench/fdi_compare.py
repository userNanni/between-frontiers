#!/usr/bin/env python3
"""Foreign direct investment in a run against `data/history/fdi.csv`.

    python3 tools/bench/fdi_compare.py history.json [--years 1975,1985,1995,2005,2015]

From the history the bench writes (`--history-out`), per country and year: the inflow of the
year (growth of the foreign-owned stock, `foreign_stock`) over GDP, beside the World Bank's net
inflow over GDP (`fdi_inflow_gdp`) for the same country and year; and the world's inflow over the
world's GDP, beside the GDP-weighted mean of the reference. The stock is valued as the bench values
it (a level's capital at the host's productivity), so the inflow is gross of depreciation.
"""

import csv
import json
import sys
from pathlib import Path

# Game tag -> ISO3 of the reference, where they differ or matter.
ISO = {
    "USA": "USA", "GBR": "GBR", "FRA": "FRA", "GER": "DEU", "ITA": "ITA", "JAP": "JPN",
    "CHN": "CHN", "RAJ": "IND", "BRA": "BRA", "MEX": "MEX", "ARG": "ARG", "SOV": "RUS",
    "KOR": "KOR", "NLD": "NLD", "BEL": "BEL", "ESP": "ESP", "POR": "PRT", "CAN": "CAN",
    "AUS": "AUS", "TUR": "TUR", "EGY": "EGY", "INS": "IDN", "POL": "POL",
}


def reference(root: Path) -> dict:
    ref = {}
    with open(root / "data/history/fdi.csv") as f:
        rows = csv.reader(line for line in f if not line.startswith("#"))
        for r in rows:
            if r[0] == "iso3" or r[2] != "fdi_inflow_gdp":
                continue
            try:
                ref[(r[0], int(r[1]))] = float(r[3])
            except ValueError:
                pass
    return ref


def main() -> None:
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        sys.exit(2)
    years = [1935, 1945, 1955, 1965, 1975, 1985, 1995, 2005, 2015]
    if "--years" in args:
        years = [int(y) for y in args[args.index("--years") + 1].split(",")]
    root = Path(__file__).resolve().parents[2]
    h = json.load(open(args[0]))
    first = h["first_year"]
    ref = reference(root)
    countries = {c["tag"]: c for c in h["countries"]}
    years = [y for y in years if 0 < y - first < len(h["countries"][0]["gdp"])]
    print("inflow / GDP, % (game / World Bank)")
    print("tag  " + " ".join(f"{y:>11}" for y in years) + "   stock/GDP at end")
    for tag, iso in ISO.items():
        c = countries.get(tag)
        if not c:
            continue
        st, g = c["foreign_stock"], c["gdp"]
        cells = []
        for y in years:
            k = y - first
            sim = 100 * (st[k] - st[k - 1]) / g[k] if g[k] else 0.0
            rv = ref.get((iso, y))
            cells.append(f"{sim:5.1f}/{rv:4.1f}" if rv is not None else f"{sim:5.1f}/  --")
        end = 100 * st[-1] / g[-1] if g[-1] else 0.0
        print(f"{tag:4} " + " ".join(f"{x:>11}" for x in cells) + f"   {end:5.0f}%")
    # The world: the game's sum over every country, the reference weighted by the game's GDP.
    print("\nworld inflow / world GDP, %")
    for y in years:
        k = y - first
        inflow = sum(c["foreign_stock"][k] - c["foreign_stock"][k - 1] for c in h["countries"])
        gdp = sum(c["gdp"][k] for c in h["countries"])
        num = den = 0.0
        for tag, iso in ISO.items():
            c = countries.get(tag)
            rv = ref.get((iso, y))
            if c and rv is not None:
                num += rv * c["gdp"][k]
                den += c["gdp"][k]
        refw = f"{num / den:4.1f}" if den else "  --"
        print(f"  {y}: game {100 * inflow / gdp if gdp else 0:5.2f}   reference {refw}")


if __name__ == "__main__":
    main()

"""Foreign direct investment of a game run against `data/history/fdi.csv`.

The run's history (`bf-bench --game --history-out`) has, per country and year, the foreign stock
it hosts and owns at the year's end (stakes and works paid from abroad, at what a level costs
there) and its GDP. The yearly inflow is the change of the hosted stock plus its depreciation
(`econ::yearly::DEPRECIATION`, 1.5 % a year); plants closed at a loss are not counted, so the
inflow is a floor. The reference is the World Bank's inflow over GDP (`fdi_inflow_gdp`).

    python3 tools/history/fdi.py HISTORY.json

Prints the world's inflow over GDP by decade (game and reference, the reference weighted by
GDP as `inflow_usd / (inflow_gdp / 100)`), the largest hosts and owners by decade, and the
direction cases of `docs/late-industrialization.pt-BR.md` (part 2, section 7).
"""

import csv
import json
import os
import sys
from collections import defaultdict

ROOT = os.path.normpath(os.path.join(os.path.dirname(__file__), "..", ".."))
DEPRECIATION = 0.015
# Game tags whose iso3 differs, for the cases below (tools/history/mapping.csv maps units).
ISO = {"RAJ": "IND"}


def reference():
    inflow_gdp = defaultdict(dict)
    inflow_usd = defaultdict(dict)
    with open(os.path.join(ROOT, "data", "history", "fdi.csv"), encoding="utf-8") as f:
        rows = (r for r in csv.reader(f) if r and not r[0].startswith("#"))
        next(rows)
        for iso, year, series, value in rows:
            if series == "fdi_inflow_gdp":
                inflow_gdp[iso][int(year)] = float(value)
            elif series == "fdi_inflow_usd":
                inflow_usd[iso][int(year)] = float(value)
    return inflow_gdp, inflow_usd


def game(path):
    with open(path, encoding="utf-8") as f:
        h = json.load(f)
    years = h["years"]
    out = {}
    for c in h["countries"]:
        stock = c.get("foreign_stock") or []
        owned = c.get("foreign_owned") or []
        gdp = c.get("gdp") or []
        inflow = {}
        for i in range(1, min(len(stock), len(gdp))):
            inflow[years[i]] = (stock[i] - stock[i - 1] * (1 - DEPRECIATION), gdp[i])
        out[c["tag"]] = {"inflow": inflow, "stock": dict(zip(years, stock)),
                         "owned": dict(zip(years, owned)), "gdp": dict(zip(years, gdp))}
    return years, out


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    years, g = game(sys.argv[1])
    ref_gdp, ref_usd = reference()
    print("world inflow over GDP, % (decade mean): game | reference")
    for d0 in range(1930, max(years) + 1, 10):
        span = [y for y in years if d0 <= y < d0 + 10]
        if not span:
            continue
        inflow = sum(c["inflow"].get(y, (0, 0))[0] for c in g.values() for y in span)
        gdp = sum(c["inflow"].get(y, (0, 0))[1] for c in g.values() for y in span)
        ru = rg = 0.0
        for iso, by in ref_usd.items():
            for y in span:
                if y in by and ref_gdp[iso].get(y, 0) > 0:
                    ru += by[y]
                    rg += by[y] / (ref_gdp[iso][y] / 100)
        ref = f"{100 * ru / rg:.2f}" if rg else "-"
        print(f"  {d0}s: {100 * inflow / max(gdp, 1):.2f} | {ref}")
    print("largest hosts and owners at the decade's end (stock, % of GDP)")
    for y in [y for y in years if y % 10 == 5]:
        hosts = sorted(g.items(), key=lambda kv: -kv[1]["stock"].get(y, 0))[:6]
        owners = sorted(g.items(), key=lambda kv: -kv[1]["owned"].get(y, 0))[:6]
        fmt = lambda kv, k: f"{kv[0]} {100 * kv[1][k].get(y, 0) / max(kv[1]['gdp'].get(y, 1), 1):.1f}"
        print(f"  {y}: hosts {', '.join(fmt(kv, 'stock') for kv in hosts)}")
        print(f"        owners {', '.join(fmt(kv, 'owned') for kv in owners)}")
    print("direction cases, inflow over GDP % (game | reference), mean of the years")
    cases = [("CHN", 1970, 1978), ("CHN", 1985, 1994), ("CHN", 1995, 2010), ("RAJ", 1975, 1990),
             ("RAJ", 1992, 2010), ("KOR", 1965, 1990), ("TWN", 1965, 1990), ("USA", 1970, 2010),
             ("GBR", 1970, 2010), ("BRA", 1970, 2010), ("MEX", 1970, 2010)]
    for tag, a, b in cases:
        if tag not in g:
            continue
        span = range(a, b + 1)
        gi = [100 * g[tag]["inflow"][y][0] / max(g[tag]["inflow"][y][1], 1)
              for y in span if y in g[tag]["inflow"]]
        iso = ISO.get(tag, tag)
        ri = [ref_gdp[iso][y] for y in span if y in ref_gdp.get(iso, {})]
        gm = f"{sum(gi) / len(gi):.2f}" if gi else "-"
        rm = f"{sum(ri) / len(ri):.2f}" if ri else "-"
        print(f"  {tag} {a}-{b}: {gm} | {rm}")


if __name__ == "__main__":
    main()

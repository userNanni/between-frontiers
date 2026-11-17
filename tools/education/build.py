#!/usr/bin/env python3
"""Schooling data from Lee and Lee (2016), "Human capital in the long run", Journal of
Development Economics 122: 147-169, dataset LeeLee_v1 (MIT licence, https://github.com/barrolee/
BarroLeeDataSet; site http://barrolee.com). See docs/population.pt-BR.md, "Ensino por coorte".

    python3 tools/education/build.py --fetch    # download the raw files into data/raw/history/leelee/
    python3 tools/education/build.py            # write the two outputs (needs pandas to read .dta)

Outputs:
- data/history/education.csv: by modern country (iso3) and year (1870-2010, five-yearly):
  enrolment ratios of primary, secondary and tertiary (adjusted, %), the shares of those 15-64
  by highest level attained (none, primary, secondary, tertiary, %) and their mean years of
  schooling. The scorecard's reference.
- data/scenario/1925/education.ron: by unit of 1925 (tag, metropole or colonies), the enrolment
  of each level every five years from 1865 to 1925 and the attainment of those 15-64 in 1925,
  averaged over the modern countries the unit held (tools/history/mapping.csv), weighted by
  their population of 1925 (data/history/population.csv). The simulation grows each state's
  schooling of 1925 from this.
"""

import csv
import os
import sys
import urllib.request

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
RAW = os.path.join(ROOT, "data", "raw", "history", "leelee")
BASE = "https://raw.githubusercontent.com/barrolee/BarroLeeDataSet/master/"
FILES = {
    "LeeLee_v1.dta": "LeeLee/LeeLee_v1.dta",
    "BL2013_MF1599_v2.2.csv": "BLData/BL2013_MF1599_v2.2.csv",
}
FIRST, LAST = 1870, 2010
PATH_YEARS = list(range(1865, 1926, 5))


def fetch():
    os.makedirs(RAW, exist_ok=True)
    for name, path in FILES.items():
        out = os.path.join(RAW, name)
        if os.path.exists(out):
            continue
        req = urllib.request.Request(BASE + path, headers={"User-Agent": "between-frontiers-data/1.0"})
        with urllib.request.urlopen(req, timeout=300) as r, open(out, "wb") as f:
            f.write(r.read())
        print("downloaded", name)


def find(name):
    for d, _, files in os.walk(RAW):
        if name in files:
            return os.path.join(d, name)
    sys.exit(f"{name} missing: run with --fetch")


def build():
    import pandas as pd  # only needed here

    ll = pd.read_stata(find("LeeLee_v1.dta"))
    ll = ll[ll["sex"].astype(str).str.upper().isin(["MF", "TOTAL", "T"])] if "sex" in ll else ll
    bl = pd.read_csv(find("BL2013_MF1599_v2.2.csv"))
    iso = dict(zip(bl["BLcode"], bl["WBcode"]))
    ll = ll[ll["BLcode"].isin(iso.keys())].copy()
    ll["iso3"] = ll["BLcode"].map(iso)
    ll["year"] = ll["year"].astype(int)

    def num(v, d=2):
        return "" if pd.isna(v) else round(float(v), d)

    rows = []
    for _, r in ll[(ll.year >= FIRST) & (ll.year <= LAST)].iterrows():
        rows.append((r.iso3, r.year, num(r.pri), num(r.sec), num(r.ter), num(r.lu), num(r.lp),
                     num(r.ls), num(r.lh), num(r.tyr)))
    rows.sort()
    out = os.path.join(ROOT, "data", "history", "education.csv")
    with open(out, "w", newline="", encoding="utf-8") as f:
        f.write("# Lee and Lee (2016), Human capital in the long run, JDE 122; dataset LeeLee_v1 "
                "(MIT licence, github.com/barrolee/BarroLeeDataSet). Enrolment: adjusted ratio, %. "
                "Attainment: share of those 15-64 by highest level attained (complete or not), %. "
                "See SOURCES.md.\n")
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["iso3", "year", "primary_enrolment", "secondary_enrolment", "tertiary_enrolment",
                    "no_school_15_64", "primary_15_64", "secondary_15_64", "tertiary_15_64",
                    "mean_years_15_64"])
        w.writerows(rows)
    print("wrote", os.path.relpath(out, ROOT), len(rows), "rows")

    # Units of 1925.
    pop = {}
    with open(os.path.join(ROOT, "data", "history", "population.csv"), encoding="utf-8") as f:
        for r in csv.DictReader(line for line in f if not line.startswith("#")):
            if r["year"] == "1925" and r["population"]:
                pop[r["iso3"]] = float(r["population"])
    units = {}
    region_of = {}
    with open(os.path.join(ROOT, "tools", "history", "mapping.csv"), encoding="utf-8") as f:
        for r in csv.DictReader(f):
            if r["tag"]:
                units.setdefault((r["tag"], r["part"]), []).append(r["iso3"])
                region_of[r["iso3"]] = r["region"]
    by = {(r.iso3, r.year): r for _, r in ll.iterrows()}
    lines = []
    # A unit without data takes the mean of the countries with data of the region (of the
    # scorecard) its largest country is in.
    region_members = {}
    for i in pop:
        if (i, 1925) in by and i in region_of:
            region_members.setdefault(region_of[i], []).append(i)
    for (tag, part), members in sorted(units.items()):
        total = sum(pop.get(i, 0.0) for i in members)
        have = [i for i in members if (i, 1925) in by and pop.get(i)]
        covered = sum(pop[i] for i in have)
        estimate = ""
        if not have and total > 0:
            largest = max(members, key=lambda i: pop.get(i, 0.0))
            region = region_of.get(largest, "")
            have = region_members.get(region, [])
            covered = 0.0
            estimate = f", estimate: Some(\"mean of {region}\")"
        if not have or total <= 0:
            continue
        def mean(field, year):
            vals = [(float(getattr(by[(i, year)], field)), pop[i]) for i in have
                    if (i, year) in by and not pd.isna(getattr(by[(i, year)], field))]
            w = sum(p for _, p in vals)
            return round(sum(v * p for v, p in vals) / w, 2) if w else 0.0
        path = [[mean(k, y) for k in ("pri", "sec", "ter")] for y in PATH_YEARS]
        attain = [mean(k, 1925) for k in ("lu", "lp", "ls", "lh")]
        lines.append(
            f'        (tag: "{tag}", colonies: {"true" if part == "colonies" else "false"}, '
            f"coverage: {round(covered / total, 3)}, "
            f"enrolment: [{', '.join('(' + ', '.join(str(v) for v in p) + ')' for p in path)}], "
            f"attainment_15_64: ({', '.join(str(v) for v in attain)}), "
            f'countries: "{" ".join(sorted(have)) if not estimate else ""}"{estimate}),'
        )
    out = os.path.join(ROOT, "data", "scenario", "1925", "education.ron")
    with open(out, "w", encoding="utf-8") as f:
        f.write(
            "// Between Frontiers: schooling of 1925 by unit. Generated by tools/education/build.py;\n"
            "// do not edit. Source: Lee and Lee (2016), Human capital in the long run, JDE 122,\n"
            "// dataset LeeLee_v1 (MIT licence, github.com/barrolee/BarroLeeDataSet), averaged over\n"
            "// the modern countries each unit held (tools/history/mapping.csv), weighted by their\n"
            "// population of 1925. `enrolment`: (primary, secondary, tertiary) adjusted enrolment\n"
            "// ratios, %, every five years from 1865 to 1925. `attainment_15_64`: shares of those\n"
            "// 15-64 with no schooling, primary, secondary and tertiary as the highest level\n"
            "// attained, %, in 1925. `coverage`: share of the unit's 1925 population in countries\n"
            "// with data. Units missing here have no data: docs/population.pt-BR.md says what the\n"
            "// simulation does for them.\n"
            "(\n    first_year: 1865,\n    step: 5,\n    units: [\n"
        )
        f.write("\n".join(lines) + "\n    ],\n)\n")
    print("wrote", os.path.relpath(out, ROOT), len(lines), "units")


if __name__ == "__main__":
    if "--fetch" in sys.argv:
        fetch()
    else:
        build()

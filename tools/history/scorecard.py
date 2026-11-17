"""The historical scorecard: a 100-year run of the simulation against the world of 1925-2025.

    python3 tools/history/scorecard.py --sim target/history/history.json [--out target/history]

Reads the yearly series that `bf-bench --game --history-out` writes, the reference series in
data/history/ and the border map tools/history/mapping.csv, and writes scorecard.json,
scorecard.md and PNG charts (charts need matplotlib; without it they are skipped).

Units of comparison are the 1925 tags, metropole and colonies apart, because the simulation keeps
the 1925 borders: every modern country is added to the tag (or tags) that held it in 1925.
How to read the result: docs/history.pt-BR.md.
"""

import argparse
import csv
import json
import math
import os
from collections import defaultdict

ROOT = os.path.normpath(os.path.join(os.path.dirname(__file__), "..", ".."))
DATA = os.path.join(ROOT, "data", "history")
MAPPING = os.path.join(ROOT, "tools", "history", "mapping.csv")

DECADES = list(range(1925, 2026, 10))
PERIODS = [
    (1925, 1929, "boom of the twenties"),
    (1929, 1933, "Great Depression"),
    (1933, 1939, "recovery and rearmament"),
    (1939, 1945, "Second World War"),
    (1945, 1973, "post-war golden age"),
    (1973, 1991, "oil shocks to Soviet collapse"),
    (1991, 2008, "globalisation"),
    (2008, 2025, "financial crisis to today"),
]
# Below this share of the unit's population covered by a series, the unit has no reference.
MIN_COVERAGE = 0.8
# Longest gap (years) bridged by interpolation between two observations.
MAX_GAP = {"gdppc": 25, "urban": 20, "literacy": 60, "life_expectancy": 25}
# Vital rates in the reference (vital.csv) and the simulation's names for them.
VITAL = ("births", "deaths", "life_expectancy", "fertility")
# Units whose net migration by decade the scorecard shows.
MIGRATION_UNITS = [
    ("USA", "metropole"), ("CAN", "metropole"), ("AUS", "metropole"), ("ARG", "metropole"),
    ("BRA", "metropole"), ("GBR", "metropole"), ("FRA", "metropole"), ("GER", "metropole"),
    ("ITA", "metropole"), ("ESP", "metropole"), ("MEX", "metropole"), ("SOV", "metropole"),
]
# Maddison's former states and the modern countries they cover.
AGGREGATES = {
    "SUN": "RUS UKR BLR KAZ UZB TKM KGZ TJK GEO ARM AZE EST LVA LTU MDA".split(),
    "CSK": ["CZE", "SVK"],
    "YUG": "SVN HRV BIH SRB MNE MKD".split(),
    "SDN_FORMER": ["SDN", "SSD"],
}
# Natural Earth adm0_a3 codes (state keys) that differ from ISO 3166 alpha-3.
NE_TO_ISO = {
    "SDS": "SSD", "KOS": "SRB", "SOL": "SOM", "CYN": "CYP", "SAH": "ESH", "PSX": "PSE",
    "KAB": "KAZ", "CNM": "CYP", "ESB": "CYP", "WSB": "CYP", "USG": "CUB", "BJN": "COL",
    "SER": "COL", "SCR": "CHN", "SPI": "CHN", "CLP": "PYF", "ATC": "AUS",
    "IOA": "AUS", "BRI": "BRA", "KAS": "IND", "ALD": "FIN", "GSA": "GBR", "IOT": "MUS",
    # Islands with no series of their own, for the regional split only: the nearest neighbour
    # with the same 1925 owner.
    "CSI": "AUS", "HMD": "AUS", "PCN": "FJI", "UMI": "GUM", "ATF": "REU", "BVT": "NOR",
    "SGS": "FLK", "ATA": "AUS", "CPT": "PYF", "PGA": "CHN",
}


PROVINCES = os.path.join(ROOT, "assets", "map", "provinces.ron")


def state_countries():
    """Modern country (ISO alpha-3) of every map state: the Natural Earth country holding most of
    its area, read from the provinces' keys."""
    import re

    prov = {}
    states = {}
    section = None
    for line in open(PROVINCES, encoding="utf-8"):
        t = line.strip()
        if t.startswith("provinces: ["):
            section = "p"
        elif t.startswith("states: ["):
            section = "s"
        m = re.match(r'\(id: (\d+), key: "([^"]+)"', t)
        if not m:
            continue
        if section == "p":
            area = re.search(r"area_km2: ([0-9.]+)", t)
            key = m.group(2)
            adm0 = key[5:8] if key.startswith("ADM0-") else key[:3]
            prov[int(m.group(1))] = (adm0, float(area.group(1)) if area else 0.0)
        elif section == "s":
            ids = re.search(r"provinces: \[([0-9, ]*)\]", t)
            states[m.group(2)] = [int(x) for x in ids.group(1).split(",") if x.strip()] if ids else []
    out = {}
    for key, ids in states.items():
        area = defaultdict(float)
        for i in ids:
            if i in prov:
                a0, km2 = prov[i]
                area[NE_TO_ISO.get(a0, a0)] += km2 + 1e-6
        if area:
            out[key] = max(area, key=area.get)
    return out


def read_csv(path):
    with open(path, newline="", encoding="utf-8") as f:
        lines = [l for l in f if not l.startswith("#")]
    return list(csv.DictReader(lines))


def interpolate(points, first, last, max_gap, log=False):
    """Yearly values from sparse points: linear (or geometric) between points at most
    `max_gap` years apart, never extrapolated."""
    ys = sorted(points)
    out = {}
    for a, b in zip(ys, ys[1:]):
        if b - a > max_gap:
            continue
        va, vb = points[a], points[b]
        for y in range(a, b + 1):
            t = (y - a) / (b - a)
            if log and va > 0 and vb > 0:
                out[y] = va * (vb / va) ** t
            else:
                out[y] = va + (vb - va) * t
    for y in ys:
        out[y] = points[y]
    return {y: v for y, v in out.items() if first <= y <= last}


class Reference:
    """Reference series by modern country (iso3) and year."""

    def __init__(self, first=1920, last=2025):
        self.pop = defaultdict(dict)
        for r in read_csv(os.path.join(DATA, "population.csv")):
            self.pop[r["iso3"]][int(r["year"])] = float(r["population"])
        mpd_gdppc, mpd_pop = defaultdict(dict), defaultdict(dict)
        for r in read_csv(os.path.join(DATA, "maddison.csv")):
            y = int(r["year"])
            if r["gdppc_2011usd"]:
                mpd_gdppc[r["code"]][y] = float(r["gdppc_2011usd"])
            if r["population"]:
                mpd_pop[r["code"]][y] = float(r["population"])
        self.gdppc = {
            c: interpolate(p, first, last, MAX_GAP["gdppc"], log=True) for c, p in mpd_gdppc.items()
        }
        self.gdppc_observed = {c: set(p) for c, p in mpd_gdppc.items()}
        # Former states: what the members lack is the aggregate minus the members that have data.
        for agg, members in AGGREGATES.items():
            for y, g in list(self.gdppc.get(agg, {}).items()):
                p = mpd_pop[agg].get(y) or sum(self.pop[m].get(y, 0) for m in members)
                known = [m for m in members if y in self.gdppc.get(m, {})]
                missing = [m for m in members if m not in known and self.pop[m].get(y)]
                if not missing:
                    continue
                rest = g * p - sum(
                    self.gdppc[m][y] * (mpd_pop[m].get(y) or self.pop[m].get(y, 0)) for m in known
                )
                rest_pop = sum(self.pop[m][y] for m in missing)
                if rest > 0 and rest_pop > 0:
                    for m in missing:
                        self.gdppc.setdefault(m, {})[y] = rest / rest_pop
        self.production = defaultdict(lambda: defaultdict(dict))  # good -> iso -> year -> value
        self.units = {}
        # production_local.csv: series that may be used here but not redistributed (fetch.py).
        for path in (
            os.path.join(DATA, "production.csv"),
            os.path.join(ROOT, "data", "raw", "history", "production_local.csv"),
        ):
            if not os.path.exists(path):
                continue
            for r in read_csv(path):
                self.production[r["good"]][r["iso3"]][int(r["year"])] = float(r["value"])
                self.units[r["good"]] = r["unit"]
        self.wdi = defaultdict(lambda: defaultdict(dict))  # series -> iso -> year -> percent
        path = os.path.join(DATA, "wdi.csv")
        if os.path.exists(path):
            for r in read_csv(path):
                self.wdi[r["series"]][r["iso3"]][int(r["year"])] = float(r["value"])
        # Informal employment (ILOSTAT, SDG 8.3.1): one recent year per country, which stands for
        # the last simulated years (the series is short and moves slowly).
        path = os.path.join(DATA, "informal.csv")
        if os.path.exists(path):
            for r in read_csv(path):
                for y in range(2000, 2031):
                    self.wdi["employment_informal"][r["iso3"]][y] = float(r["informal_share_percent"])
        # Births and deaths per thousand (from 1950) and life expectancy (long run, gaps filled).
        self.vital = {}
        path = os.path.join(DATA, "vital.csv")
        pts = {k: defaultdict(dict) for k in VITAL}
        # Children per woman (UN WPP, from 1950) in a file of its own.
        for path in (path, os.path.join(DATA, "fertility.csv")):
            if not os.path.exists(path):
                continue
            for r in read_csv(path):
                for k in VITAL:
                    if r.get(k):
                        pts[k][r["iso3"]][int(r["year"])] = float(r[k])
        for k in VITAL:
            self.vital[k] = {
                c: interpolate(p, first, last, MAX_GAP["life_expectancy"]) if k == "life_expectancy" else p
                for c, p in pts[k].items()
            }
        self.share = {}
        for name, file in (("urban", "urban.csv"), ("literacy", "literacy.csv")):
            pts = defaultdict(dict)
            for r in read_csv(os.path.join(DATA, file)):
                pts[r["iso3"]][int(r["year"])] = float(r["share"])
            self.share[name] = {
                c: interpolate(p, first, last, MAX_GAP[name]) for c, p in pts.items()
            }


class Units:
    """Every 1925 unit (tag, part) as weighted modern countries."""

    def __init__(self, ref):
        rows = read_csv(MAPPING)
        self.region_of = {}
        self.members = defaultdict(list)  # (tag, part) -> [(iso, weight)]
        carved = defaultdict(float)
        main = {}
        for r in rows:
            iso = r["iso3"]
            self.region_of.setdefault(iso, r["region"])
            key = (r["tag"], r["part"])
            if r["pop1925_k"]:
                p1925 = ref.pop[iso].get(1925)
                w = float(r["pop1925_k"]) * 1000 / p1925
                carved[iso] += w
                self.members[key].append((iso, w))
            else:
                main[iso] = key
        for iso, key in main.items():
            self.members[key].append((iso, 1.0 - carved[iso]))
        self.region_units = defaultdict(list)


def aggregate(ref, members, year):
    """Population, GDP, GDP per head, urban share and literacy of a set of weighted countries,
    each None when the series covers less than MIN_COVERAGE of the population."""
    pop = sum(ref.pop[i].get(year, 0) * w for i, w in members)
    out = {"population": pop or None}
    if not pop:
        return {k: None for k in ("population", "gdp", "gdp_per_capita", "urban", "literacy", "non_farm") + VITAL}
    covered = gdp = 0.0
    for i, w in members:
        g = ref.gdppc.get(i, {}).get(year)
        p = ref.pop[i].get(year, 0) * w
        if g is not None and p:
            covered += p
            gdp += g * p
    ok = covered >= MIN_COVERAGE * pop
    out["gdp"] = gdp / covered * pop if ok else None
    out["gdp_per_capita"] = gdp / covered if ok else None
    out["gdp_coverage"] = covered / pop
    for name in ("urban", "literacy"):
        cov = val = 0.0
        for i, w in members:
            v = ref.share[name].get(i, {}).get(year)
            p = ref.pop[i].get(year, 0) * w
            if v is not None and p:
                cov += p
                val += v * p
        out[name] = val / cov if cov >= MIN_COVERAGE * pop else None
    # Employment outside agriculture (ILO modelled, from 1991), the reference for the
    # simulation's share of people outside farming.
    cov = val = 0.0
    for i, w in members:
        v = ref.wdi["employment_agriculture"].get(i, {}).get(year)
        p = ref.pop[i].get(year, 0) * w
        if v is not None and p:
            cov += p
            val += (1 - v / 100) * p
    out["non_farm"] = val / cov if cov >= MIN_COVERAGE * pop else None
    for name in VITAL:
        cov = val = 0.0
        for i, w in members:
            v = ref.vital.get(name, {}).get(i, {}).get(year)
            p = ref.pop[i].get(year, 0) * w
            if v is not None and p:
                cov += p
                val += v * p
        out[name] = val / cov if cov >= MIN_COVERAGE * pop else None
    return out


# Simulated units to the reference's: tonnes of coal, oil and 1,000 m3 of gas to TWh of primary
# energy (approximate heat contents: coal 7.0 MWh/t, between hard coal and lignite; oil 1 toe =
# 11.63 MWh/t; gas 10 MWh per 1,000 m3), MWh to TWh. Steel and cereals are tonnes on both sides
# (the reference counts rice as paddy, the simulation as milled rice).
TO_REFERENCE = {
    "coal": 7.0e-6,
    "crude_oil": 11.63e-6,
    "natural_gas": 10.0e-6,
    "electricity": 1e-6,
    "steel": 1.0,
    "cereals": 1.0,
}


def ref_production(ref, members, good, year):
    vals = [ref.production[good].get(i, {}).get(year) for i, _ in members]
    if all(v is None for v in vals):
        return None
    return sum((v or 0) * w for v, (_, w) in zip(vals, members))


def ref_wdi(ref, members, series, years):
    """Population-weighted mean of a WDI series over the members, averaged over `years`; None
    below MIN_COVERAGE."""
    out = []
    for y in years:
        pop = sum(ref.pop[i].get(y, 0) * w for i, w in members)
        cov = val = 0.0
        for i, w in members:
            v = ref.wdi[series].get(i, {}).get(y)
            p = ref.pop[i].get(y, 0) * w
            if v is not None and p:
                cov += p
                val += v * p
        if pop and cov >= MIN_COVERAGE * pop:
            out.append(val / cov)
    return sum(out) / len(out) if out else None


class Sim:
    def __init__(self, path):
        with open(path, encoding="utf-8") as f:
            h = json.load(f)
        self.years = h["years"]
        self.goods = h["goods"]
        self.sectors = h["sectors"]
        self.idx = {y: i for i, y in enumerate(self.years)}
        self.units = {(t["tag"], t["part"]): t for t in h["territories"]}
        self.countries = {c["tag"]: c for c in h["countries"]}
        self.raw_units = h["territories"]
        self.states = h["states"]

    def unit(self, key, year):
        t = self.units.get(key)
        i = self.idx.get(year)
        if t is None or i is None or not t["population"][i]:
            return None
        pop = t["population"][i]
        emp = t["employment"][i]
        return {
            "population": pop,
            "gdp": t["gdp"][i],
            "gdp_per_capita": t["gdp"][i] / pop,
            "non_farm": t["non_farm"][i],
            "literacy": t["literacy"][i],
            # Older history files have no vital rates.
            **{k: t[k][i] if t.get(k) else None for k in VITAL + ("migration",)},
            "employment": emp,
            "production": dict(zip(self.goods, t["production"][i])),
        }


def sum_sim(rows):
    rows = [r for r in rows if r]
    if not rows:
        return None
    pop = sum(r["population"] for r in rows)
    out = {"population": pop, "gdp": sum(r["gdp"] for r in rows)}
    out["gdp_per_capita"] = out["gdp"] / pop
    out["non_farm"] = sum(r["non_farm"] * r["population"] for r in rows) / pop
    out["literacy"] = sum(r["literacy"] * r["population"] for r in rows) / pop
    return out


def ratio(a, b):
    return a / b if a is not None and b else None


def growth(series, a, b):
    va, vb = series.get(a), series.get(b)
    if not va or not vb or va <= 0 or vb <= 0:
        return None
    return ((vb / va) ** (1 / (b - a)) - 1) * 100


def spearman(xs, ys):
    def ranks(v):
        order = sorted(range(len(v)), key=lambda i: v[i])
        r = [0.0] * len(v)
        for k, i in enumerate(order):
            r[i] = k
        return r

    n = len(xs)
    if n < 3:
        return None
    rx, ry = ranks(xs), ranks(ys)
    d = sum((a - b) ** 2 for a, b in zip(rx, ry))
    return 1 - 6 * d / (n * (n * n - 1))


def build(sim_path):
    ref = Reference()
    units = Units(ref)
    sim = Sim(sim_path)
    years = [y for y in sim.years if y >= 1925]
    last_ref_gdp = min(max(y for c in ("USA", "GBR", "CHN") for y in ref.gdppc[c]), years[-1])
    last_pop = min(2025, years[-1])

    # Reference and simulation by unit and year.
    keys = sorted(units.members)
    table = {}
    for key in keys:
        table[key] = {
            y: {"ref": aggregate(ref, units.members[key], y), "sim": sim.unit(key, y)} for y in years
        }
    missing_units = sorted(k for k in sim.units if k not in units.members and sim.unit(k, years[0]))

    # Regions: simulated states by the modern country of their key; reference by country.
    region_states = defaultdict(list)
    unknown_states = []
    country_of_state = state_countries()
    for s in sim.states:
        iso = country_of_state.get(s["key"], s["key"][:3])
        region = units.region_of.get(iso)
        if region is None:
            unknown_states.append(s["key"])
            continue
        region_states[region].append(s)
    region_isos = defaultdict(list)
    for iso, region in units.region_of.items():
        region_isos[region].append((iso, 1.0))
    regions = sorted(region_isos)

    def sim_region(region, y):
        i = sim.idx.get(y)
        if i is None:
            return None
        pop = sum(s["population"][i] for s in region_states[region])
        gdp = sum(s["gdp"][i] for s in region_states[region])
        out = {"population": pop, "gdp": gdp, "gdp_per_capita": gdp / pop if pop else None}
        # Vital rates weighted by population (older history files have none).
        for k in VITAL + ("migration",):
            rows = [(st[k][i], st["population"][i]) for st in region_states[region] if st.get(k)]
            n = sum(p for _, p in rows)
            out[k] = sum(v * p for v, p in rows) / n if n else None
        return out

    all_isos = [(iso, 1.0) for iso in units.region_of]
    world = {
        y: {
            "ref": aggregate(ref, all_isos, y),
            "sim": sum_sim([sim.unit(k, y) for k in sim.units]),
        }
        for y in years
    }
    region_rows = {
        r: {y: {"ref": aggregate(ref, region_isos[r], y), "sim": sim_region(r, y)} for y in years}
        for r in regions
    }

    # Gross world product by summing the regions (a region misses when its coverage is low).
    def gdp_year(y):
        return y if y <= last_ref_gdp else last_ref_gdp

    decade_rows = []
    for y in DECADES:
        if y not in sim.idx:
            continue
        gy = gdp_year(y)
        w = world[y]
        wg = world[gy]
        row = {
            "year": y,
            "gdp_year": gy,
            "population": [w["sim"]["population"], w["ref"]["population"]],
            "gdp": [wg["sim"]["gdp"], wg["ref"]["gdp"]],
            "gdp_per_capita": [wg["sim"]["gdp_per_capita"], wg["ref"]["gdp_per_capita"]],
            "non_farm": [w["sim"]["non_farm"], w["ref"]["non_farm"]],
            # The model has no urban population of its own: HYDE's urban share is context only.
            "urban_reference": w["ref"]["urban"],
            "literacy": [w["sim"]["literacy"], w["ref"]["literacy"]],
        }
        # Across units with a reference: mean |log error| and rank correlation of GDP per head.
        errs = {"population": [], "gdp": [], "gdp_per_capita": []}
        sims, refs = [], []
        for key in keys:
            c = table[key].get(gy)
            p = table[key].get(y)
            if p and p["sim"] and p["ref"]["population"]:
                errs["population"].append(abs(math.log(p["sim"]["population"] / p["ref"]["population"])))
            if c and c["sim"] and c["ref"]["gdp"] and c["sim"]["gdp"] > 0:
                errs["gdp"].append(abs(math.log(c["sim"]["gdp"] / c["ref"]["gdp"])))
                errs["gdp_per_capita"].append(
                    abs(math.log(c["sim"]["gdp_per_capita"] / c["ref"]["gdp_per_capita"]))
                )
                sims.append(c["sim"]["gdp_per_capita"])
                refs.append(c["ref"]["gdp_per_capita"])
        row["mean_abs_log_error"] = {
            k: (sum(v) / len(v) if v else None) for k, v in errs.items()
        }
        row["units_with_gdp"] = len(sims)
        row["rank_correlation_gdp_per_capita"] = spearman(sims, refs)
        # Shares of world GDP by region.
        shares = {}
        sim_total = sum((region_rows[r][gy]["sim"] or {}).get("gdp") or 0 for r in regions)
        ref_vals = {r: region_rows[r][gy]["ref"]["gdp"] for r in regions}
        ref_total = wg["ref"]["gdp"]
        for r in regions:
            s = (region_rows[r][gy]["sim"] or {}).get("gdp")
            shares[r] = [
                s / sim_total if s and sim_total else None,
                ref_vals[r] / ref_total if ref_total and ref_vals[r] else None,
            ]
        row["region_gdp_share"] = shares
        decade_rows.append(row)

    # Growth by historical period and region.
    period_rows = []
    for a, b, label in PERIODS:
        if b > last_pop:
            continue
        b_gdp = min(b, last_ref_gdp)
        for name, series in [("World", world)] + [(r, region_rows[r]) for r in regions]:
            row = {"period": f"{a}-{b}", "label": label, "region": name, "gdp_end": b_gdp}
            for metric, end in (("population", b), ("gdp", b_gdp), ("gdp_per_capita", b_gdp)):
                for side in ("sim", "ref"):
                    vals = {y: (series[y][side] or {}).get(metric) for y in (a, end) if y in series}
                    row[f"{metric}_{side}"] = growth(vals, a, end)
            period_rows.append(row)

    # The largest units of 1925.
    by_size = sorted(
        (k for k in keys if table[k][1925]["sim"]),
        key=lambda k: -table[k][1925]["sim"]["population"],
    )
    largest = by_size[:20]
    unit_rows = []
    for key in keys:
        r = {"tag": key[0], "part": key[1], "decades": []}
        for y in DECADES:
            if y not in sim.idx:
                continue
            gy = gdp_year(y)
            s, f = table[key][y]["sim"], table[key][y]["ref"]
            sg, fg = table[key][gy]["sim"], table[key][gy]["ref"]
            if not s:
                continue
            r["decades"].append(
                {
                    "year": y,
                    "population": ratio(s["population"], f["population"]),
                    "gdp": ratio(sg["gdp"], fg["gdp"]) if sg else None,
                    "gdp_per_capita": ratio(sg["gdp_per_capita"], fg["gdp_per_capita"]) if sg else None,
                    "non_farm": [s["non_farm"], f["non_farm"]],
                    "urban_reference": f["urban"],
                    "literacy": [s["literacy"], f["literacy"]],
                    **{k: [s.get(k), f.get(k)] for k in VITAL},
                    "migration": s.get("migration"),
                }
            )
        unit_rows.append(r)

    # Physical production, world, by decade.
    production_rows = []
    goods = [g for g in TO_REFERENCE if g in sim.goods]
    for y in DECADES:
        if y not in sim.idx:
            continue
        i = sim.idx[y]
        row = {"year": y}
        for g in goods:
            # Only units with a reference, on both sides: a series that covers a few countries
            # is compared with those countries.
            k = sim.goods.index(g)
            sv = rv = 0.0
            n = 0
            for key in keys:
                r = ref_production(ref, units.members[key], g, y)
                t = sim.units.get(key)
                if r is None or t is None:
                    continue
                sv += t["production"][i][k] * TO_REFERENCE[g]
                rv += r
                n += 1
            total = sum(t["production"][i][k] for t in sim.raw_units) * TO_REFERENCE[g]
            row[g] = [sv, rv if n else None, n, sv / total if total else None, total]
        production_rows.append(row)

    # Employment by sector, public accounts and inflation of the largest units, last decade.
    recent = [y for y in range(last_pop - 10, last_pop) if y in sim.idx]
    other_rows = []
    for key in largest:
        t = sim.units[key]
        members = units.members[key]
        i = sim.idx[last_pop]
        emp = t["employment"][i]
        tot = sum(emp) or 1.0
        row = {"unit": f"{key[0]}/{key[1]}", "year": last_pop}
        for k, sector in enumerate(sim.sectors):
            ref_v = ref_wdi(ref, members, f"employment_{sector}", [last_pop - 1, last_pop])
            row[f"employment_{sector}"] = [emp[k] / tot * 100, ref_v]
        if t.get("informal"):
            ref_v = ref_wdi(ref, members, "employment_informal", [last_pop - 1, last_pop])
            row["employment_informal"] = [t["informal"][i] / tot * 100, ref_v]
        c = sim.countries.get(key[0])
        if key[1] == "metropole" and c:
            def mean(xs):
                xs = [x for x in xs if x is not None]
                return sum(xs) / len(xs) if xs else None
            idx = [sim.idx[y] for y in recent]
            row["taxes"] = [mean([c["taxes"][j] * 100 for j in idx]), ref_wdi(ref, members, "revenue", recent)]
            row["spending"] = [mean([c["spending"][j] * 100 for j in idx]), ref_wdi(ref, members, "expense", recent)]
            row["debt"] = [c["debt"][i] * 100, ref_wdi(ref, members, "debt", [last_pop - 2, last_pop - 1])]
            row["inflation"] = [mean([c["inflation"][j] for j in idx]), ref_wdi(ref, members, "inflation", recent)]
        other_rows.append(row)

    # Births, deaths and life expectancy by region and year (decades from 1950, plus 1925-45 for
    # life expectancy); net migration of the reference is what population change leaves after
    # births and deaths.
    def ref_migration(members, y):
        a, b = aggregate(ref, members, y), aggregate(ref, members, y + 1)
        if not (a["population"] and b["population"]) or a["births"] is None or a["deaths"] is None:
            return None
        return 1000 * (b["population"] - a["population"]) / a["population"] - (a["births"] - a["deaths"])

    vital_years = [1925, 1935, 1945] + list(range(1950, 2021, 10))
    vital_rows = []
    for r in ["World"] + regions:
        row = {"region": r, "years": {}}
        for y in vital_years:
            if y not in sim.idx:
                continue
            if r == "World":
                rows = [region_rows[g][y]["sim"] for g in regions if region_rows[g][y]["sim"]]
                n = sum(x["population"] for x in rows)
                sv = {k: (sum((x[k] or 0) * x["population"] for x in rows) / n if n and all(x[k] is not None for x in rows) else None)
                      for k in VITAL + ("migration",)}
                fv = aggregate(ref, all_isos, y)
                fm = None
            else:
                sv = region_rows[r][y]["sim"]
                fv = region_rows[r][y]["ref"]
                fm = ref_migration(region_isos[r], y)
            row["years"][y] = {k: [sv.get(k), fv.get(k)] for k in VITAL}
            row["years"][y]["migration"] = [sv.get("migration"), fm]
        vital_rows.append(row)

    # Net migration by decade, people a year, for the units where it mattered most.
    migration_rows = []
    for key in [k for k in MIGRATION_UNITS if k in units.members]:
        t = sim.units.get(key)
        if t is None or not t.get("migration"):
            continue
        row = {"unit": f"{key[0]}/{key[1]}", "decades": {}}
        for d0 in range(1925, 2025, 10):
            ys = [y for y in range(d0, d0 + 10) if y in sim.idx]
            if not ys:
                continue
            sim_n = sum(t["migration"][sim.idx[y]] * t["population"][sim.idx[y]] / 1000 for y in ys) / len(ys)
            refs = [ref_migration(units.members[key], y) for y in ys]
            pops = [aggregate(ref, units.members[key], y)["population"] for y in ys]
            ok = [(m, p) for m, p in zip(refs, pops) if m is not None and p]
            ref_n = sum(m * p / 1000 for m, p in ok) / len(ok) if len(ok) == len(ys) else None
            row["decades"][d0] = [sim_n, ref_n]
        migration_rows.append(row)

    arrival_rows = arrivals_table(sim_path, sim)

    deviations = find_deviations(table, largest, world, region_rows, regions, period_rows, last_ref_gdp, last_pop)
    return {
        "sim_years": [sim.years[0], sim.years[-1]],
        "last_reference_gdp_year": last_ref_gdp,
        "min_coverage": MIN_COVERAGE,
        "world": decade_rows,
        "periods": period_rows,
        "production": production_rows,
        "production_units": {g: ref.units.get(g, "") for g in goods},
        "largest_other": other_rows,
        "recent_years": [recent[0], recent[-1]] if recent else None,
        "units": unit_rows,
        "vital_regions": vital_rows,
        "migration": migration_rows,
        "arrivals": arrival_rows,
        "waves": json.load(open(sim_path, encoding="utf-8")).get("waves") or [],
        "largest": [f"{t}/{p}" for t, p in largest],
        "deviations": deviations,
        "deviation_groups": group_deviations(deviations),
        "unmatched_states": unknown_states,
        "units_without_reference": [f"{t}/{p}" for t, p in missing_units],
        "_series": {
            "years": years,
            "world": world,
            "regions": region_rows,
            "units": {f"{t}/{p}": table[(t, p)] for t, p in largest},
        },
    }


# Arrivals compared with immigration records: (row, destination unit, origin units in the
# simulation, destination and origin in immigration.csv). Origin None = everyone.
ARRIVALS = [
    ("Brasil ← Itália", "BRA/metropole", ["ITA/metropole"], "BRA", "ITA"),
    ("Brasil ← Portugal", "BRA/metropole", ["POR/metropole"], "BRA", "PRT"),
    ("Brasil ← Espanha", "BRA/metropole", ["ESP/metropole"], "BRA", "ESP"),
    ("Brasil ← Alemanha", "BRA/metropole", ["GER/metropole"], "BRA", "DEU"),
    ("Brasil ← Japão", "BRA/metropole", ["JAP/metropole", "JAP/colonies"], "BRA", "JPN"),
    ("Brasil ← URSS", "BRA/metropole", ["SOV/metropole"], "BRA", "RUS"),
    ("Brasil ← todos", "BRA/metropole", None, "BRA", None),
    ("EUA ← todos", "USA/metropole", None, "USA", None),
    ("Argentina ← todos", "ARG/metropole", None, None, None),
    ("Canadá ← todos", "CAN/metropole", None, None, None),
    ("Austrália ← todos", "AUS/metropole", None, None, None),
]


def arrivals_table(sim_path, sim):
    """Arrivals a year by decade, simulated (the run's roads) against the records."""
    with open(sim_path, encoding="utf-8") as f:
        pairs = json.load(f).get("migration_pairs") or []
    if not pairs:
        return []
    records = defaultdict(lambda: defaultdict(float))  # (dest, origin) -> year -> people
    path = os.path.join(DATA, "immigration.csv")
    if os.path.exists(path):
        for r in read_csv(path):
            if r["origin"].startswith("stock:"):
                continue  # stocks of the 1940 census, not arrivals
            records[(r["destination"], r["origin"])][int(r["year"])] = float(r["arrivals"])
            records[(r["destination"], None)][int(r["year"])] += float(r["arrivals"])
    out = []
    for label, dest, origins, rdest, rorigin in ARRIVALS:
        row = {"label": label, "decades": {}}
        for d0 in range(1925, 2025, 10):
            ys = [y for y in range(d0, d0 + 10) if y in sim.idx]
            if not ys:
                continue
            simv = 0.0
            for p in pairs:
                if p["dest"] == dest and (origins is None or p["origin"] in origins):
                    simv += sum(p["arrivals"][sim.idx[y]] for y in ys)
            ref = records.get((rdest, rorigin), {}) if rdest else {}
            have = [ref[y] for y in ys if y in ref]
            row["decades"][d0] = [simv / len(ys), sum(have) / len(have) if have else None, len(have)]
        out.append(row)
    return out


def find_deviations(table, largest, world, region_rows, regions, period_rows, last_gdp, last_pop):
    """Candidate gaps between the run and history, each with a size in log points weighted by
    how much of the world it concerns, largest first."""
    out = []
    end = last_gdp
    wgdp = world[end]["ref"]["gdp"] or 1.0

    def add(kind, subject, metric, sim_v, ref_v, weight, detail):
        if sim_v and ref_v and sim_v > 0 and ref_v > 0:
            gap = math.log(sim_v / ref_v)
            out.append(
                {
                    "kind": kind,
                    "subject": subject,
                    "metric": metric,
                    "sim": sim_v,
                    "ref": ref_v,
                    "log_gap": gap,
                    "weight": weight,
                    "score": abs(gap) * math.sqrt(weight),
                    "detail": detail,
                }
            )

    w = world
    add("world", "World", "population", w[last_pop]["sim"]["population"], w[last_pop]["ref"]["population"], 1.0, str(last_pop))
    add("world", "World", "gdp_per_capita", w[end]["sim"]["gdp_per_capita"], w[end]["ref"]["gdp_per_capita"], 1.0, str(end))
    first = min(w)
    add("world", "World", "gdp first year", w[first]["sim"]["gdp"], w[first]["ref"]["gdp"], 1.0,
        f"{first}: the first simulated year against Maddison")
    add("world", "World", "non_farm", w[last_pop]["sim"]["non_farm"], w[last_pop]["ref"]["non_farm"], 1.0,
        f"{last_pop}: people outside farming vs employment outside agriculture")
    for key in largest:
        t = table[key]
        s0, r0 = t[1925]["sim"], t[1925]["ref"]
        s1, r1 = t[end]["sim"], t[end]["ref"]
        name = f"{key[0]}/{key[1]}"
        # GDP per head is compared as a level at the end: per head, the border mismatch of
        # 1925 matters little, and the first simulated year starts far below its calibration.
        if s1 and r1["gdp_per_capita"]:
            add(
                "unit", name, f"gdp_per_capita {end}",
                s1["gdp_per_capita"], r1["gdp_per_capita"],
                (r1["gdp"] or 0) / wgdp,
                f"{s1['gdp_per_capita']:,.0f} vs {r1['gdp_per_capita']:,.0f} $".replace(",", " "),
            )
        # Population since 1925, so that a border mismatch present from the start does not count.
        p1s, p1r = t[last_pop]["sim"], t[last_pop]["ref"]
        if s0 and r0["population"] and p1s and p1r["population"]:
            add(
                "unit", name, f"population growth 1925-{last_pop}",
                p1s["population"] / s0["population"], p1r["population"] / r0["population"],
                p1r["population"] / w[last_pop]["ref"]["population"],
                f"x{p1s['population'] / s0['population']:.2f} vs x{p1r['population'] / r0['population']:.2f}",
            )
    for row in period_rows:
        if row["region"] == "World":
            continue
        a, b = (int(x) for x in row["period"].split("-"))
        b = row["gdp_end"]
        if row["gdp_per_capita_sim"] is None or row["gdp_per_capita_ref"] is None:
            continue
        years = b - a
        sim_f = (1 + row["gdp_per_capita_sim"] / 100) ** years
        ref_f = (1 + row["gdp_per_capita_ref"] / 100) ** years
        share = (region_rows[row["region"]][a]["ref"]["gdp"] or 0) / (world[a]["ref"]["gdp"] or 1)
        add("period", row["region"], f"gdp_per_capita {row['period']}", sim_f, ref_f, share,
            f"{row['gdp_per_capita_sim']:+.1f}%/y vs {row['gdp_per_capita_ref']:+.1f}%/y")
    out.sort(key=lambda d: -d["score"])
    return out


# What the model lacks, by the deviations it explains. The first rule that matches wins; the
# list is a reading aid, kept short on purpose: a new kind of gap gets a rule here.
CAUSES = [
    ("small_start", "Começo pequeno: o PIB do primeiro ano simulado e a recuperação dele",
     lambda d: d["metric"] == "gdp first year" or d["metric"].endswith("1925-1929")),
    ("no_cycle_or_war", "Sem ciclo nem guerra: Depressão, guerra e reconstrução",
     lambda d: any(d["metric"].endswith(p) for p in ("1929-1933", "1933-1939", "1939-1945"))
     or (d["metric"].endswith("1945-1973") and d["subject"] in ("Western Europe", "Western offshoots"))),
    ("no_regime", "Sem efeito de regime: estagnação e colapso do bloco soviético",
     lambda d: d["subject"] in ("Former USSR", "Eastern Europe") and d["kind"] == "period"),
    ("demography", "Demografia: transição adiantada, população baixa",
     lambda d: d["metric"].startswith("population")),
    ("non_farm", "Emprego e urbanização: a parcela fora da agropecuária",
     lambda d: d["metric"] == "non_farm"),
    ("convergence", "Convergência mecânica: produtividade que fecha a distância ao líder",
     lambda d: d["metric"].startswith("gdp_per_capita")),
]


def group_deviations(devs):
    groups = {}
    for d in devs:
        key, label = next(((k, l) for k, l, rule in CAUSES if rule(d)), ("other", "Outros"))
        d["cause"] = key
        g = groups.setdefault(key, {"cause": key, "label": label, "score": 0.0, "members": []})
        g["members"].append(d)
        g["score"] = math.sqrt(g["score"] ** 2 + d["score"] ** 2)
    out = sorted(groups.values(), key=lambda g: -g["score"])
    for g in out:
        g["members"].sort(key=lambda d: -d["score"])
    return out


# ---------------------------------------------------------------------------- output

def fmt(v, kind):
    if v is None:
        return "–"
    if kind == "pop":
        return f"{v / 1e9:.2f} bi" if v >= 1e9 else f"{v / 1e6:.0f} M"
    if kind == "gdp":
        return f"{v / 1e12:.1f} tri"
    if kind == "pc":
        return f"{v:,.0f}".replace(",", " ")
    if kind == "share":
        return f"{v * 100:.0f} %"
    if kind == "ratio":
        return f"{v:.2f}"
    if kind == "pct":
        return f"{v:+.1f}"
    return str(v)


def markdown(card):
    L = []
    L.append("# Placar histórico\n")
    L.append(
        f"Simulação {card['sim_years'][0]}–{card['sim_years'][-1]} contra a referência em `data/history/`. "
        f"PIB em dólares internacionais de 2011 (Maddison); a referência de PIB vai até {card['last_reference_gdp_year']}, "
        f"e as linhas de PIB de 2025 comparam {card['last_reference_gdp_year']}. "
        "O modelo não tem população urbana própria: \"fora da agropecuária\" (profissões não rurais) é comparado com o "
        "emprego não agrícola da OIT (desde 1991); a urbanização HYDE fica só como contexto. "
        "Como ler: `docs/history.pt-BR.md`.\n"
    )
    L.append("## Mundo por década (simulado / real)\n")
    L.append("| Ano | População | PIB | PIB por hab. | Fora da agropecuária | Urbana (HYDE) | Alfabetização | Erro log. médio pop / PIB / PIB pc | Correlação de ranking PIB pc (n) |")
    L.append("|---|---|---|---|---|---|---|---|---|")
    for r in card["world"]:
        e = r["mean_abs_log_error"]
        rc = r["rank_correlation_gdp_per_capita"]
        L.append(
            f"| {r['year']}{'' if r['gdp_year'] == r['year'] else ' (PIB ' + str(r['gdp_year']) + ')'} "
            f"| {fmt(r['population'][0], 'pop')} / {fmt(r['population'][1], 'pop')} "
            f"| {fmt(r['gdp'][0], 'gdp')} / {fmt(r['gdp'][1], 'gdp')} "
            f"| {fmt(r['gdp_per_capita'][0], 'pc')} / {fmt(r['gdp_per_capita'][1], 'pc')} "
            f"| {fmt(r['non_farm'][0], 'share')} / {fmt(r['non_farm'][1], 'share')} "
            f"| {fmt(r['urban_reference'], 'share')} "
            f"| {fmt(r['literacy'][0], 'share')} / {fmt(r['literacy'][1], 'share')} "
            f"| {' / '.join('–' if e[k] is None else f'{e[k]:.2f}' for k in ('population', 'gdp', 'gdp_per_capita'))} "
            f"| {'–' if rc is None else f'{rc:.2f}'} ({r['units_with_gdp']}) |"
        )
    L.append("\n## Participação das regiões no PIB mundial (simulado / real)\n")
    regions = list(card["world"][0]["region_gdp_share"])
    L.append("| Região | " + " | ".join(str(r["year"]) for r in card["world"]) + " |")
    L.append("|---|" + "---|" * len(card["world"]))
    for reg in regions:
        cells = []
        for r in card["world"]:
            s, f = r["region_gdp_share"][reg]
            cells.append(f"{fmt(s, 'share')} / {fmt(f, 'share')}")
        L.append(f"| {reg} | " + " | ".join(cells) + " |")
    L.append("\n## Crescimento por período histórico, % ao ano (simulado / real)\n")
    L.append("PIB por habitante; entre parênteses, população.\n")
    names = []
    for p in card["periods"]:
        if p["region"] not in names:
            names.append(p["region"])
    periods = []
    for p in card["periods"]:
        if p["period"] not in periods:
            periods.append(p["period"])
    L.append("| Região | " + " | ".join(periods) + " |")
    L.append("|---|" + "---|" * len(periods))
    for n in names:
        cells = []
        for per in periods:
            p = next(x for x in card["periods"] if x["region"] == n and x["period"] == per)
            cells.append(
                f"{fmt(p['gdp_per_capita_sim'], 'pct')} / {fmt(p['gdp_per_capita_ref'], 'pct')} "
                f"({fmt(p['population_sim'], 'pct')} / {fmt(p['population_ref'], 'pct')})"
            )
        L.append(f"| {n} | " + " | ".join(cells) + " |")
    L.append("\n## Os 20 maiores de 1925: razão simulado / real\n")
    L.append("População · PIB por habitante, por década. `–` = sem referência (cobertura abaixo de "
             f"{int(card['min_coverage'] * 100)} % da população).\n")
    years = [r["year"] for r in card["world"]]
    L.append("| Unidade | " + " | ".join(str(y) for y in years) + " |")
    L.append("|---|" + "---|" * len(years))
    units = {f"{u['tag']}/{u['part']}": u for u in card["units"]}
    for name in card["largest"]:
        u = units[name]
        cells = []
        for y in years:
            d = next((d for d in u["decades"] if d["year"] == y), None)
            cells.append("–" if d is None else f"{fmt(d['population'], 'ratio')} · {fmt(d['gdp_per_capita'], 'ratio')}")
        L.append(f"| {name} | " + " | ".join(cells) + " |")
    if any(d.get("births", [None])[0] is not None for u in card["units"] for d in u["decades"]):
        L.append("\n## Demografia dos 20 maiores (simulado / real)\n")
        L.append("Natalidade e mortalidade por mil habitantes ao ano (ONU, de 1950) · expectativa de vida ao "
                 "nascer (anos) · migração líquida por mil (só simulada) · filhos por mulher (ONU, de 1950). "
                 "`–` = sem referência.\n")
        vy = [y for y in years if y in (1925, 1955, 1975, 1995, 2015)]
        L.append("| Unidade | " + " | ".join(str(y) for y in vy) + " |")
        L.append("|---|" + "---|" * len(vy))
        units = {f"{u['tag']}/{u['part']}": u for u in card["units"]}

        def pair(v, digits=0):
            a, b = v
            sa = "–" if a is None else f"{a:.{digits}f}"
            sb = "–" if b is None else f"{b:.{digits}f}"
            return f"{sa}/{sb}"

        for name in card["largest"]:
            u = units[name]
            cells = []
            for y in vy:
                d = next((d for d in u["decades"] if d["year"] == y), None)
                if d is None or d.get("births") is None:
                    cells.append("–")
                    continue
                mig = "–" if d.get("migration") is None else f"{d['migration']:+.1f}"
                tfr = pair(d.get("fertility") or [None, None], 1)
                cells.append(f"{pair(d['births'])} · {pair(d['deaths'])} · {pair(d['life_expectancy'])} · {mig} · {tfr}")
            L.append(f"| {name} | " + " | ".join(cells) + " |")
    if card.get("vital_regions"):
        L.append("\n## Natalidade, mortalidade e migração por região (simulado / real)\n")
        L.append("Por mil habitantes ao ano: natalidade · mortalidade · migração líquida (a real é o que sobra da "
                 "variação da população depois de nascimentos e mortes, ONU de 1950). Até 1945, só a "
                 "expectativa de vida (anos) tem referência.\n")
        vys = [y for y in (1925, 1935, 1945) if any(y in r["years"] for r in card["vital_regions"])]
        if vys:
            L.append("| Região | " + " | ".join(f"e0 {y}" for y in vys) + " |")
            L.append("|---|" + "---|" * len(vys))
            for r in card["vital_regions"]:
                cells = []
                for y in vys:
                    v = r["years"].get(y)
                    a, b = (v or {}).get("life_expectancy", [None, None])
                    cells.append(f"{'–' if a is None else f'{a:.0f}'}/{'–' if b is None else f'{b:.0f}'}")
                L.append(f"| {r['region']} | " + " | ".join(cells) + " |")
            L.append("")
        dys = [y for y in range(1950, 2021, 10) if any(y in r["years"] for r in card["vital_regions"])]
        L.append("| Região | " + " | ".join(str(y) for y in dys) + " |")
        L.append("|---|" + "---|" * len(dys))

        def pr(v, plus=False):
            a, b = v
            f = (lambda x: f"{x:+.1f}") if plus else (lambda x: f"{x:.0f}")
            return f"{'–' if a is None else f(a)}/{'–' if b is None else f(b)}"

        for r in card["vital_regions"]:
            cells = []
            for y in dys:
                v = r["years"].get(y)
                cells.append("–" if not v else f"{pr(v['births'])} · {pr(v['deaths'])} · {pr(v['migration'], True)}")
            L.append(f"| {r['region']} | " + " | ".join(cells) + " |")
        if any(v.get("fertility", [None, None])[1] is not None for r in card["vital_regions"] for v in r["years"].values()):
            L.append("\n### Filhos por mulher por região (simulado / real)\n")
            L.append("TFR médio ponderado pela população dos dois lados (ONU WPP 2024, de 1950). Entre "
                     "colchetes, simulado menos real quando passa de ±0,5.\n")
            L.append("| Região | " + " | ".join(str(y) for y in dys) + " |")
            L.append("|---|" + "---|" * len(dys))
            for r in card["vital_regions"]:
                cells = []
                for y in dys:
                    a, b = (r["years"].get(y) or {}).get("fertility", [None, None])
                    c = f"{'–' if a is None else f'{a:.1f}'}/{'–' if b is None else f'{b:.1f}'}"
                    if a is not None and b is not None and abs(a - b) > 0.5:
                        c += f" [{a - b:+.1f}]"
                    cells.append(c)
                L.append(f"| {r['region']} | " + " | ".join(cells) + " |")
    if card.get("migration"):
        L.append("\n## Migração líquida por década (simulado / real)\n")
        L.append("Milhares de pessoas por ano, média da década; positivo entra. A real é a variação da "
                 "população menos nascimentos e mortes (só de 1950, quando a ONU tem as taxas).\n")
        ds = sorted({d for r in card["migration"] for d in r["decades"]}, key=int)
        L.append("| Unidade | " + " | ".join(f"{d}s" for d in ds) + " |")
        L.append("|---|" + "---|" * len(ds))
        for r in card["migration"]:
            cells = []
            for d in ds:
                a, b = r["decades"].get(d, [None, None])
                cells.append(f"{'–' if a is None else f'{a / 1000:+.0f}'}/{'–' if b is None else f'{b / 1000:+.0f}'}")
            L.append(f"| {r['unit']} | " + " | ".join(cells) + " |")
    if card.get("arrivals"):
        L.append("\n## Chegadas de imigrantes por década (simulado / registrado)\n")
        L.append("Milhares por ano, média da década. Brasil: IBGE/MTIC, *Estatística Migratória de 1952*, "
                 "tabela 8 (só os anos cujas colunas somam o total; entre colchetes, quantos anos da década). "
                 "EUA: DHS, residentes permanentes. A simulação conta quem chegou pela estrada origem → "
                 "destino, sem os filhos.\n")
        ds = sorted({int(d) for r in card["arrivals"] for d in r["decades"]})
        L.append("| Estrada | " + " | ".join(f"{d}s" for d in ds) + " |")
        L.append("|---|" + "---|" * len(ds))
        for r in card["arrivals"]:
            cells = []
            for d in ds:
                a, b, n = r["decades"].get(d, r["decades"].get(str(d), [None, None, 0]))
                sa = "–" if a is None else f"{a / 1000:.1f}"
                sb = "–" if b is None else f"{b / 1000:.1f}" + (f" [{n}]" if n < 10 else "")
                cells.append(f"{sa} / {sb}")
            L.append(f"| {r['label']} | " + " | ".join(cells) + " |")
        if card.get("waves"):
            L.append("\n### Ondas de migração\n")
            L.append("| Origem → destino | Abriu | Fechou | Por quê abriu | Por quê fechou | Pessoas (mil) |")
            L.append("|---|---|---|---|---|---:|")
            for w in sorted(card["waves"], key=lambda w: -w["people"])[:30]:
                until = "aberta" if not w["until"] else f"{w['until'][0]}-{w['until'][1]:02d}"
                L.append(f"| {w['origin']} → {w['dest']} | {w['from'][0]}-{w['from'][1]:02d} | {until} | "
                         f"{w['cause']} | {w['end'] or '–'} | {w['people'] / 1000:.0f} |")
    if card.get("production"):
        L.append("\n## Produção mundial (simulado / real)\n")
        units = card["production_units"]
        goods = [g for g in card["production"][0] if g != "year"]
        L.append("Carvão, petróleo e gás em TWh de energia primária (simulação convertida de toneladas e m³ por "
                 "poder calorífico médio); eletricidade em TWh; aço e cereais em milhões de toneladas. "
                 "Os dois lados somam só as unidades com série de referência no ano; entre colchetes, quantas "
                 "unidades e que parte da produção simulada do mundo elas cobrem. `–` = sem série no ano. "
                 "Aço: série usada só localmente (ver `data/history/SOURCES.md`).\n")
        L.append("| Ano | " + " | ".join(f"{g} ({'Mt' if units.get(g) == 't' else units.get(g)})" for g in goods) + " |")
        L.append("|---|" + "---|" * len(goods))
        for r in card["production"]:
            cells = []
            for g in goods:
                sv, rv, n, cov, _total = r[g]
                sc = 1e-6 if units.get(g) == "t" else 1.0
                if rv is None:
                    cells.append("–")
                else:
                    cells.append(f"{sv * sc:,.0f} / {rv * sc:,.0f} [{n}; {cov * 100:.0f} %]".replace(",", " "))
            L.append(f"| {r['year']} | " + " | ".join(cells) + " |")
    if card.get("largest_other"):
        y0, y1 = card["recent_years"] or (None, None)
        L.append(f"\n## Emprego, contas públicas e inflação dos 20 maiores (simulado / real)\n")
        L.append(f"Emprego: % do total em {card['largest_other'][0]['year']} (referência: OIT modelada, WDI, "
                 f"média dos dois últimos anos). Impostos, gasto e inflação: médias de {y0}–{y1}, % do PIB e % ao ano; "
                 "dívida: % do PIB no fim. A simulação conta o país inteiro (metrópole e colônias) nas contas públicas; "
                 "a referência, os países modernos da metrópole. Receita do WDI exclui doações; gasto do WDI é despesa "
                 "do governo central.\n")
        cols = ["employment_agriculture", "employment_industry", "employment_services", "employment_informal",
                "taxes", "spending", "debt", "inflation"]
        heads = ["Agro", "Indústria", "Serviços", "Informal", "Impostos", "Gasto", "Dívida", "Inflação"]
        L.append("| Unidade | " + " | ".join(heads) + " |")
        L.append("|---|" + "---|" * len(cols))
        for r in card["largest_other"]:
            cells = []
            for c in cols:
                if c not in r:
                    cells.append("")
                    continue
                sv, rv = r[c]
                cells.append(f"{'–' if sv is None else f'{sv:.0f}'} / {'–' if rv is None else f'{rv:.0f}'}")
            L.append(f"| {r['unit']} | " + " | ".join(cells) + " |")
    L.append("\n## Maiores desvios, por causa\n")
    L.append("Cada desvio é |log(simulado/real)| × √(peso no mundo); os desvios com a mesma causa no modelo "
             "ficam juntos, e a causa pesa a raiz da soma dos quadrados dos seus desvios. Até cinco desvios "
             "por causa. Leitura das causas: `docs/history.pt-BR.md`.\n")
    L.append("| # | Causa | Peso | Desvios (simulado vs real) |")
    L.append("|---|---|---|---|")
    for i, g in enumerate(card["deviation_groups"][:10], 1):
        items = "; ".join(
            f"{d['subject']} {d['metric']}: {d['detail']}" for d in g["members"][:5]
        )
        more = len(g["members"]) - 5
        if more > 0:
            items += f"; e mais {more}"
        L.append(f"| {i} | {g['label']} | {g['score']:.2f} | {items} |")
    L.append("\n### Os 15 maiores, um a um\n")
    L.append("| # | Assunto | Métrica | Simulado | Real | Detalhe | Causa |")
    L.append("|---|---|---|---|---|---|---|")
    for i, d in enumerate(card["deviations"][:15], 1):
        L.append(
            f"| {i} | {d['subject']} | {d['metric']} | {d['sim']:.3g} | {d['ref']:.3g} | {d['detail']} | {d['cause']} |"
        )
    if card["unmatched_states"]:
        L.append(f"\nEstados sem país moderno reconhecido (fora das regiões): {', '.join(card['unmatched_states'])}\n")
    return "\n".join(L) + "\n"


def charts(card, out):
    try:
        import matplotlib

        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
    except ImportError:
        print("matplotlib not found: charts skipped")
        return []
    s = card["_series"]
    years = s["years"]
    files = []
    sim_c, ref_c = "#c0392b", "#2c3e50"

    def line(ax, series, metric, scale=1.0, title=""):
        xs = [y for y in years if series[y]["sim"] and series[y]["sim"].get(metric) is not None]
        ax.plot(xs, [series[y]["sim"][metric] * scale for y in xs], color=sim_c, lw=1.6, label="simulado")
        # Gaps in the reference stay gaps (NaN) instead of straight lines across them.
        yr = [
            series[y]["ref"][metric] * scale
            if series[y]["ref"] and series[y]["ref"].get(metric) is not None
            else float("nan")
            for y in years
        ]
        ax.plot(years, yr, color=ref_c, lw=1.6, ls="--", label="real")
        ax.set_title(title, fontsize=9)
        ax.grid(alpha=0.25)
        ax.tick_params(labelsize=7)

    fig, axs = plt.subplots(2, 3, figsize=(12, 6.5))
    w = s["world"]
    line(axs[0][0], w, "population", 1e-9, "População (bilhões)")
    line(axs[0][1], w, "gdp", 1e-12, "PIB (trilhões de US$ 2011)")
    line(axs[0][2], w, "gdp_per_capita", 1, "PIB por habitante (US$ 2011)")
    line(axs[1][0], w, "non_farm", 100, "Fora da agropecuária % (real: emprego, OIT)")
    hyde = [w[y]["ref"]["urban"] * 100 if w[y]["ref"].get("urban") is not None else float("nan") for y in years]
    axs[1][0].plot(years, hyde, color="#7f8c8d", lw=1.2, ls=":", label="urbana (HYDE), sem par no modelo")
    axs[1][0].legend(fontsize=7)
    line(axs[1][1], w, "literacy", 100, "Alfabetização %")
    axs[1][2].axis("off")
    axs[0][0].legend(fontsize=8)
    fig.suptitle("Mundo: simulação × história", fontsize=11)
    fig.tight_layout()
    p = os.path.join(out, "world.png")
    fig.savefig(p, dpi=110)
    plt.close(fig)
    files.append(p)

    regs = s["regions"]
    names = sorted(regs)
    for metric, scale, title, fname in (
        ("gdp_per_capita", 1, "PIB por habitante (US$ 2011), log", "regions_gdp_per_capita.png"),
        ("population", 1e-6, "População (milhões)", "regions_population.png"),
    ):
        n = len(names)
        cols = 4
        rows = (n + cols - 1) // cols
        fig, axs = plt.subplots(rows, cols, figsize=(13, 2.8 * rows))
        for i, name in enumerate(names):
            ax = axs[i // cols][i % cols]
            line(ax, regs[name], metric, scale, name)
            if metric == "gdp_per_capita":
                ax.set_yscale("log")
        for j in range(n, rows * cols):
            axs[j // cols][j % cols].axis("off")
        axs[0][0].legend(fontsize=7)
        fig.suptitle(f"Regiões: {title}", fontsize=11)
        fig.tight_layout()
        p = os.path.join(out, fname)
        fig.savefig(p, dpi=100)
        plt.close(fig)
        files.append(p)

    if card.get("production"):
        rows = card["production"]
        goods = [g for g in rows[0] if g != "year"]
        fig, axs = plt.subplots(2, 3, figsize=(12, 6.5))
        for k, g in enumerate(goods[:6]):
            ax = axs[k // 3][k % 3]
            sc = 1e-6 if card["production_units"].get(g) == "t" else 1.0
            ys = [r["year"] for r in rows]
            ax.plot(ys, [r[g][4] * sc for r in rows], color=sim_c, marker="o", ms=3, label="simulado, mundo")
            pts = [(r["year"], r[g][1] * sc) for r in rows if r[g][1] is not None]
            if pts:
                ax.plot(*zip(*pts), color=ref_c, ls="--", marker="o", ms=3, label="real")
                cov = [(r["year"], r[g][0] * sc) for r in rows if r[g][1] is not None]
                ax.plot(*zip(*cov), color=sim_c, ls=":", lw=1, label="simulado, mesmas unidades")
            unit = "Mt" if sc != 1.0 else card["production_units"].get(g, "")
            ax.set_title(f"{g} ({unit})", fontsize=9)
            ax.grid(alpha=0.25)
            ax.tick_params(labelsize=7)
        axs[0][0].legend(fontsize=8)
        fig.suptitle("Produção mundial por década", fontsize=11)
        fig.tight_layout()
        p = os.path.join(out, "production.png")
        fig.savefig(p, dpi=100)
        plt.close(fig)
        files.append(p)

    units = s["units"]
    for metric, scale, title, fname, log in (
        ("gdp_per_capita", 1, "PIB por habitante (US$ 2011), log", "countries_gdp_per_capita.png", True),
        ("population", 1e-6, "População (milhões)", "countries_population.png", False),
    ):
        fig, axs = plt.subplots(5, 4, figsize=(13, 14))
        for i, name in enumerate(card["largest"]):
            ax = axs[i // 4][i % 4]
            series = {int(y): v for y, v in units[name].items()}
            line(ax, series, metric, scale, name)
            if log:
                ax.set_yscale("log")
        axs[0][0].legend(fontsize=7)
        fig.suptitle(f"Os 20 maiores de 1925: {title}", fontsize=11)
        fig.tight_layout()
        p = os.path.join(out, fname)
        fig.savefig(p, dpi=90)
        plt.close(fig)
        files.append(p)
    return files


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--sim", required=True, help="JSON written by bf-bench --history-out")
    ap.add_argument("--out", default=os.path.join(ROOT, "target", "history"))
    args = ap.parse_args()
    os.makedirs(args.out, exist_ok=True)
    card = build(args.sim)
    files = charts(card, args.out)
    series = card.pop("_series")
    with open(os.path.join(args.out, "scorecard.json"), "w", encoding="utf-8") as f:
        json.dump(card, f, indent=1, default=lambda o: None)
    with open(os.path.join(args.out, "scorecard.md"), "w", encoding="utf-8") as f:
        f.write(markdown(card))
    del series
    for p in [os.path.join(args.out, "scorecard.json"), os.path.join(args.out, "scorecard.md")] + files:
        print("wrote", p)


if __name__ == "__main__":
    main()

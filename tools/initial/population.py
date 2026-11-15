"""Builds data/scenario/1925/population.ron.

Run from the repository root (after tools/initial/aggregate.py, whose output is versioned):

    python3 -I tools/initial/population.py

- States: each country's metropole population (`population` in countries.ron) is spread over its
  metropole states, and its colonial population (`colonies.population`) over its colonial states,
  in proportion to the HYDE 3.3 population of 1925 (tools/initial/state_hyde.csv), rounded by
  largest remainder so the sums are exact. Where a 1920s census of the country's subdivisions
  exists (tools/initial/census/*.csv), the census groups take the census shares of the people
  (after the states outside every group take their HYDE share) and HYDE only splits each group
  among its states. The urban share of each state is HYDE's urban /
  total population, multiplied by one factor per block (capped at 1) so that the
  population-weighted mean equals the block's `urbanization` in countries.ron; a few city-states
  (CITY_FLOORS) have a floor.
- Workforce: tools/initial/workforce.csv (sources and estimate flags in INITIAL.md).

Also writes tools/initial/population_check.md: HYDE against countries.ron per block, and the
state distribution against 1920s censuses (tools/initial/census/*.csv).
"""

import csv
import glob
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from common import ROOT, read_countries, read_states  # noqa: E402

OUT = os.path.join(ROOT, "data/scenario/1925/population.ron")
CHECK = os.path.join(HERE, "population_check.md")


def largest_remainder(total, weights):
    s = sum(weights)
    if s <= 0:
        weights = [1.0] * len(weights)
        s = float(len(weights))
    raw = [total * w / s for w in weights]
    base = [int(r) for r in raw]
    left = total - sum(base)
    order = sorted(range(len(raw)), key=lambda i: (-(raw[i] - base[i]), i))
    for i in order[:left]:
        base[i] += 1
    return base


# States that are essentially one city, where HYDE's 5' grid spreads the urban people over the
# surrounding cells: a floor on the urban share (census of the city against the state).
CITY_FLOORS = {
    "DEU-BE": 0.95,  # Groß-Berlin 1925
    "DEU-HH": 0.90,  # Hamburg state: the city and a few Vierlande villages
    "DEU-HB": 0.85,  # Bremen and Bremerhaven
    "AUT-9": 0.95,  # Vienna
    "USA-DC": 1.0,  # District of Columbia, wholly urban in the 1920 census
    "MEX-DIF": 0.85,  # Distrito Federal 1921: Mexico City 0.62 M of 0.91 M plus towns
    "IND-DELHI": 0.6,  # Delhi province 1921: city 0.30 M of 0.49 M
}


def fit_urban(pops, shares, target, floors):
    """One factor k with sum(p * max(floor, min(1, k * u))) = target * sum(p), by bisection."""
    tot = sum(pops)
    if tot == 0:
        return [0.0] * len(pops)
    if target is None:
        return [max(f, u) for u, f in zip(shares, floors)]
    if sum(p * u for p, u in zip(pops, shares)) <= 0:
        return [max(f, target) for f in floors]

    def at(k):
        return [max(f, min(1.0, k * u)) for u, f in zip(shares, floors)]

    lo, hi = 0.0, 1e4
    for _ in range(200):
        k = 0.5 * (lo + hi)
        v = sum(p * u for p, u in zip(pops, at(k))) / tot
        if v < target:
            lo = k
        else:
            hi = k
    return at(hi)


def load_workforce():
    rows = []
    with open(os.path.join(HERE, "workforce.csv"), encoding="utf-8") as f:
        for r in csv.DictReader(row for row in f if not row.startswith("#")):
            rows.append(r)
    return rows


def main():
    states = read_states()
    countries = read_countries()
    hyde = {}
    with open(os.path.join(HERE, "state_hyde.csv"), encoding="utf-8") as f:
        for r in csv.DictReader(f):
            hyde[r["state"]] = (float(r["hyde_pop"]), float(r["hyde_urban"]))
    blocks = {}
    for s in states:
        blocks.setdefault((s["owner"], s["colony"]), []).append(s["key"])

    census = load_census()
    pop, urb, hyde_only = {}, {}, {}
    report = ["# População de 1925: conferência", "",
              "Gerado por `tools/initial/population.py`. Não editar à mão.", "",
              "## HYDE 3.3 contra `countries.ron`, por bloco", "",
              "Razão = população do bloco em `countries.ron` / soma do HYDE nos estados do bloco. "
              "Fator urbano = multiplicador aplicado à parcela urbana do HYDE para bater a urbanização do bloco.", "",
              "| Bloco | Estados | countries.ron (mil) | HYDE (mil) | Razão | Urbano alvo | Urbano HYDE | Fator |",
              "|---|---|---|---|---|---|---|---|"]
    for (owner, colony), keys in sorted(blocks.items()):
        c = countries[owner]
        if colony:
            assert c["colonies"], owner
            total, target = c["colonies"]["population"], c["colonies"]["urbanization"]
        else:
            total, target = c["population"], c["urbanization"]
        hw = [hyde[k][0] for k in keys]
        w = hw
        for k, p in zip(keys, largest_remainder(total, w)):
            hyde_only[k] = p
        if not colony and owner in census:
            w = census_weights(keys, hyde, census[owner][1])
        alloc = largest_remainder(total, w)
        hsum = sum(hw)
        hurb = sum(hyde[k][1] for k in keys) / hsum if hsum else 0
        # A state without people in HYDE (an islet) takes the block's HYDE urban share.
        shares = [hyde[k][1] / hyde[k][0] if hyde[k][0] > 0 else hurb for k in keys]
        urban = fit_urban(alloc, shares, target, [CITY_FLOORS.get(k, 0.0) for k in keys])
        for k, p, u in zip(keys, alloc, urban):
            pop[k], urb[k] = p, u
        mean_u = sum(p * u for p, u in zip(alloc, urban)) / total if total else 0
        fac = (mean_u / hurb) if hurb else float("nan")
        report.append(
            f"| {owner}{' colônias' if colony else ''} | {len(keys)} | {total / 1e3:,.0f} | {hsum / 1e3:,.0f} | "
            f"{total / hsum if hsum else float('nan'):.2f} | {'' if target is None else f'{target:.3f}'} | {hurb:.3f} | {fac:.2f} |"
        )
    # Every country with population has states.
    for tag, c in countries.items():
        assert (tag, False) in blocks, f"{tag}: no metropole states"
        if c["colonies"]:
            assert (tag, True) in blocks, f"{tag}: no colonial states"

    report += census_checks(hyde_only, pop, states, census)
    wf = load_workforce()
    lines = [
        "// Between Frontiers: population of each state on 1925-01-01 and labour force per country.",
        "// Format: bf_sim::initial::PopulationFile. Generated by tools/initial/population.py from",
        "// HYDE 3.3 (CC BY 4.0) and countries.ron; method, sources and doubts in INITIAL.md.",
        "// Licence: CC BY-SA 4.0.",
        "(",
        "    states: [",
    ]
    for s in states:
        k = s["key"]
        lines.append(f'        (state: "{k}", population: {pop[k]}, urban: {round(urb[k], 3)}),')
    lines += ["    ],", "    workforce: ["]
    seen = set()
    for r in wf:
        tag, col = r["country"], r["colonies"] == "true"
        assert tag in countries, tag
        assert (tag, col) not in seen, (tag, col)
        seen.add((tag, col))
        a, i, sv = float(r["agriculture"]), float(r["industry"]), float(r["services"])
        assert abs(a + i + sv - 1) < 0.0015, r
        lines.append(
            f'        (country: "{tag}", colonies: {r["colonies"]}, participation: {float(r["participation"])}, '
            f"agriculture: {a}, industry: {i}, services: {sv}),"
        )
    missing = [(t, False) for t in countries if (t, False) not in seen]
    missing += [(t, True) for t, c in countries.items() if c["colonies"] and (t, True) not in seen]
    assert not missing, f"workforce missing: {missing}"
    lines += ["    ],", ")", ""]
    open(OUT, "w", encoding="utf-8").write("\n".join(lines))
    open(CHECK, "w", encoding="utf-8").write("\n".join(report) + "\n")
    print("wrote", OUT, "and", CHECK)


def load_census():
    """tag -> (meta, rows). Each census CSV has `# country: TAG`, `# title:`, `# source:` headers
    and rows `group,population,states`, `states` being `;`-separated state keys."""
    out = {}
    for path in sorted(glob.glob(os.path.join(HERE, "census", "*.csv"))):
        meta, rows = {}, []
        with open(path, encoding="utf-8") as f:
            for line in f:
                if line.startswith("#"):
                    k, _, v = line[1:].partition(":")
                    meta.setdefault(k.strip(), v.strip())
        with open(path, encoding="utf-8") as f:
            for r in csv.DictReader(line for line in f if not line.startswith("#")):
                r["keys"] = r["states"].split(";")
                rows.append(r)
        keys = [k for r in rows for k in r["keys"]]
        assert len(keys) == len(set(keys)), path
        out[meta["country"]] = (meta, rows)
    return out


def census_weights(keys, hyde, rows):
    """Weights per state: states outside every census group keep their HYDE share; the groups
    split the rest by their census shares, and each group's states by HYDE."""
    own = set(keys)
    grouped = {k for r in rows for k in r["keys"]}
    assert grouped <= own, grouped - own
    h_tot = sum(hyde[k][0] for k in keys)
    free = sum(hyde[k][0] for k in keys if k not in grouped) / h_tot
    c_tot = sum(float(r["population"]) for r in rows)
    w = {k: hyde[k][0] / h_tot for k in keys if k not in grouped}
    for r in rows:
        share = (1 - free) * float(r["population"]) / c_tot
        h = sum(hyde[k][0] for k in r["keys"])
        for k in r["keys"]:
            w[k] = share * (hyde[k][0] / h if h > 0 else 1 / len(r["keys"]))
    return [w[k] for k in keys]


def census_checks(hyde_only, pop, states, census):
    """Compares the HYDE-only state shares with the census shares, group by group."""
    out = ["", "## Distribuição entre estados contra censos da década de 1920", "",
           "Compara a parcela de cada grupo de estados no total do país pela repartição só com o HYDE "
           "com a parcela no censo. Desvio = parcela HYDE / parcela no censo − 1; marcados com "
           "**>20 %** os desvios acima de 20 %. A última coluna é a população final do arquivo, "
           "que segue a parcela do censo.", ""]
    for tag, (meta, rows) in sorted(census.items()):
        cen_tot = sum(float(r["population"]) for r in rows)
        keys = [k for r in rows for k in r["keys"]]
        own = [s["key"] for s in states if s["owner"] == tag and not s["colony"]]
        h_tot = sum(hyde_only[k] for k in keys)
        out += [f"### {tag}: {meta.get('title', '')}", "",
                f"Fonte: {meta['source']}. Estados fora da comparação: "
                f"{', '.join(sorted(set(own) - set(keys))) or 'nenhum'}.", "",
                "| Grupo | Censo (mil) | Parcela censo | Só HYDE (mil) | Parcela HYDE | Desvio | Final (mil) |",
                "|---|---|---|---|---|---|---|"]
        big = 0
        for r in rows:
            c = float(r["population"])
            g = sum(hyde_only[k] for k in r["keys"])
            fin = sum(pop[k] for k in r["keys"])
            dev = (g / h_tot) / (c / cen_tot) - 1
            flag = " **>20 %**" if abs(dev) > 0.2 else ""
            big += bool(flag)
            out.append(f"| {r['group']} | {c / 1e3:,.0f} | {c / cen_tot:.2%} | {g / 1e3:,.0f} | "
                       f"{g / h_tot:.2%} | {dev:+.0%}{flag} | {fin / 1e3:,.0f} |")
        out += ["", f"Grupos com desvio acima de 20 %: {big} de {len(rows)}.", ""]
    return out


if __name__ == "__main__":
    main()

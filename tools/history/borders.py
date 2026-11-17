# SPDX-License-Identifier: AGPL-3.0-or-later
"""What the border operations change over a century: a run with the reference changes of
`data/history/borders.ron` applied on their dates (`bf-bench --game --borders-history`) against
a run without them, both scored against history by the units of 1925 (`docs/borders.pt-BR.md`).

    python3 -I tools/history/borders.py BASE_HISTORY.json BORDERS_HISTORY.json [BORDERS_RUN.json]

Prints a Markdown report:
- colonies and colonial metropoles by decade, simulated over real GDP per head and population,
  without and with the changes (the 1925 units are fixed sets of states, so the two runs compare
  the same land);
- the countries the run created, by their states each year, against the modern country of the
  same ISO tag (GDP per head in 2022 and population in 2025);
- what the run did with the reference: submitted, applied, refused, not conserved.

The run with the changes is not the reference run of the scorecard: history happens by date only
to exercise the operations (phase 7 makes it happen by conditions).
"""

import importlib.util
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("scorecard", os.path.join(HERE, "scorecard.py"))
sc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sc)

POWERS = ["GBR", "FRA", "NLD", "BEL", "POR", "ESP", "ITA", "USA", "JAP"]
DECADES = list(range(1925, 2026, 10))


def ratio(sim, ref):
    if sim is None or ref in (None, 0):
        return None
    return sim / ref


def cell(a, b):
    f = lambda v: "–" if v is None else f"{v:.2f}"
    return f"{f(a)} → {f(b)}"


def gdp_year(y):
    return min(y, 2022)


def unit_rows(base, run, ref, units, part, metric):
    rows = []
    for tag in POWERS:
        key = (tag, part)
        members = units.members.get(key)
        if not members or key not in base.units:
            continue
        cells = []
        for y in DECADES:
            yy = gdp_year(y) if metric == "gdp_per_capita" else y
            r = sc.aggregate(ref, members, yy).get(metric)
            a = (base.unit(key, yy) or {}).get(metric)
            b = (run.unit(key, yy) or {}).get(metric)
            cells.append(cell(ratio(a, r), ratio(b, r)))
        rows.append(f"| {tag}/{part} | " + " | ".join(cells) + " |")
    return rows


def countries_by_states(h, year):
    """(population, GDP) of every owner in `year`, from the state series."""
    i = h["years"].index(year) if year in h["years"] else None
    out = {}
    if i is None:
        return out
    for s in h["states"]:
        owners = s.get("owner") or []
        if i >= len(owners):
            continue
        p, g = out.get(owners[i], (0.0, 0.0))
        out[owners[i]] = (p + s["population"][i], g + s["gdp"][i])
    return out


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    base, run = sc.Sim(sys.argv[1]), sc.Sim(sys.argv[2])
    with open(sys.argv[2], encoding="utf-8") as f:
        raw = json.load(f)
    ref = sc.Reference()
    units = sc.Units(ref)
    head = "| Unidade | " + " | ".join(str(y) for y in DECADES) + " |"
    sep = "|---|" + "---|" * len(DECADES)
    print("# Fronteiras aplicadas por data: o que muda\n")
    print("Razão simulado/real, sem as mudanças → com as mudanças. Unidades de 1925 fixas por "
          "estados; PIB de 2025 é o de 2022.\n")
    for part, title in (("colonies", "Colônias"), ("metropole", "Metrópoles coloniais")):
        for metric, name in (("gdp_per_capita", "PIB por habitante"), ("population", "População")):
            print(f"## {title}: {name}\n")
            print(head)
            print(sep)
            for r in unit_rows(base, run, ref, units, part, metric):
                print(r)
            print()

    old = {c["tag"] for c in json.load(open(sys.argv[1], encoding="utf-8"))["countries"]}
    by = countries_by_states(raw, 2022)
    by25 = countries_by_states(raw, 2025)
    print("## Países criados pela rodada, contra o país moderno da mesma etiqueta\n")
    print("| País | População 2025 sim. | real | razão | PIB por hab. 2022 sim. | real | razão |")
    print("|---|---|---|---|---|---|---|")
    for tag in sorted(set(by25) - old):
        p25 = by25[tag][0]
        rp = ref.pop.get(tag, {}).get(2025)
        p22, g22 = by.get(tag, (0.0, 0.0))
        gpc = g22 / p22 if p22 else None
        rg = ref.gdppc.get(tag, {}).get(2022)
        f = lambda v, d=0: "–" if v is None else f"{v:,.{d}f}"
        print(f"| {tag} | {f(p25 / 1e6, 1)} M | {f(rp / 1e6 if rp else None, 1)} M | "
              f"{f(ratio(p25, rp), 2)} | {f(gpc)} | {f(rg)} | {f(ratio(gpc, rg), 2)} |")
    print()
    if len(sys.argv) > 3:
        with open(sys.argv[3], encoding="utf-8") as f:
            b = json.load(f).get("borders") or {}
        print("## A rodada e a referência\n")
        print(f"- mudanças submetidas: {b.get('submitted')}, aplicadas: {b.get('applied')}, "
              f"recusadas: {len(b.get('refused', []))}, sem conservar: {len(b.get('not_conserved', []))}")
        for r in b.get("refused", [])[:30]:
            print(f"  - recusada {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Noise band of the scorecard, and comparisons that flag only what leaves it.

The model is sensitive: a change that does not touch a country still moves its trajectory. Runs
that differ only by a perturbation with no economic weight (`BF_PERTURB=K`, K dollars more in
every investment pool in 1925) measure how far that sensitivity alone moves each number.

    noise.py band --runs p0/scorecard.json p1/scorecard.json p2/scorecard.json --out band.json
    noise.py compare --band band.json --base base/scorecard.json --new branch/scorecard.json

`band` writes the spread (sample standard deviation across runs) of every cell: unit x decade x
{population, GDP, GDP per head} as the simulated / real ratio, and the aggregates (world,
ranking, regions' share of world GDP, world production). It prints the distribution by decade and
the least stable units. `compare` lists the cells and aggregates where the branch differs from
the base by more than `--sigmas` (2) standard deviations, and whether that is closer to history
or farther (docs/history.pt-BR.md, "Faixa de ruído").
"""

import argparse
import json
import math
import statistics
import sys

METRICS = ("population", "gdp", "gdp_per_capita")


def cells(score):
    """Every comparable number of a scorecard, keyed by a readable path."""
    out = {}
    for u in score["units"]:
        unit = f"{u['tag']}/{u['part']}"
        for d in u["decades"]:
            for m in METRICS:
                v = d.get(m)
                if v is not None and v > 0:
                    out[f"unit|{unit}|{d['year']}|{m}"] = v
    for w in score["world"]:
        y = w["year"]
        for m in METRICS:
            sim, ref = w[m]
            if sim and ref:
                out[f"world|World|{y}|{m}"] = sim / ref
        if w.get("rank_correlation_gdp_per_capita") is not None:
            out[f"rank|World|{y}|gdp_per_capita"] = w["rank_correlation_gdp_per_capita"]
        for region, (sim, ref) in (w.get("region_gdp_share") or {}).items():
            if sim is not None:
                out[f"share|{region}|{y}|gdp"] = sim
                if ref:
                    out[f"share_ratio|{region}|{y}|gdp"] = sim / ref
    for p in score["production"]:
        y = p["year"]
        for good, row in p.items():
            if good == "year" or not isinstance(row, list):
                continue
            sim, ref, world = row[0], row[1], row[4]
            out[f"production|{good}|{y}|world"] = world
            if sim and ref:
                out[f"production_ratio|{good}|{y}|ref"] = sim / ref
    return out


def quantile(xs, q):
    xs = sorted(xs)
    if not xs:
        return float("nan")
    i = min(len(xs) - 1, max(0, round(q * (len(xs) - 1))))
    return xs[i]


def band(args):
    runs = [cells(json.load(open(p))) for p in args.runs]
    if len(runs) < 2:
        sys.exit("band needs at least two runs")
    largest = set(json.load(open(args.runs[0]))["largest"])
    keys = set(runs[0]).intersection(*runs[1:])
    sigma = {}
    for k in sorted(keys):
        vals = [r[k] for r in runs]
        sigma[k] = {"sd": statistics.stdev(vals), "mean": statistics.fmean(vals)}
    json.dump({"runs": args.runs, "cells": sigma}, open(args.out, "w"), indent=0)

    print(f"Faixa de ruído: {len(args.runs)} rodadas, {len(sigma)} números\n")
    print("Desvio-padrão da razão simulado/real por década (unidades com referência):\n")
    print("| Década | PIB pc: mediana | PIB pc: p90 | PIB pc dos 20 maiores: p90 | PIB: p90 | População: p90 |")
    print("|---|---|---|---|---|---|")
    years = sorted({int(k.split("|")[2]) for k in sigma if k.startswith("unit|")})
    for y in years:
        def sds(metric, only=None):
            return [
                v["sd"]
                for k, v in sigma.items()
                if k.startswith("unit|")
                and k.endswith(f"|{y}|{metric}")
                and (only is None or k.split("|")[1] in only)
            ]
        pc = sds("gdp_per_capita")
        print(
            f"| {y} | {quantile(pc, 0.5):.3f} | {quantile(pc, 0.9):.3f} | "
            f"{quantile(sds('gdp_per_capita', largest), 0.9):.3f} | "
            f"{quantile(sds('gdp'), 0.9):.3f} | {quantile(sds('population'), 0.9):.3f} |"
        )
    print("\nAgregados (desvio-padrão; média entre parênteses):\n")
    for prefix in ("world", "rank", "production_ratio"):
        rows = sorted(
            (k, v) for k, v in sigma.items() if k.startswith(prefix + "|")
        )
        for k, v in rows:
            _, name, y, m = k.split("|")
            if int(y) in (1955, 1985, 2025):
                print(f"- {prefix} {name} {m} {y}: {v['sd']:.3f} ({v['mean']:.3f})")
    print("\nUnidades menos estáveis (maior desvio do PIB pc em qualquer década; primeira década acima de 0,10):\n")
    worst = {}
    first = {}
    for k, v in sigma.items():
        if not (k.startswith("unit|") and k.endswith("gdp_per_capita")):
            continue
        _, unit, y, _ = k.split("|")
        worst[unit] = max(worst.get(unit, 0.0), v["sd"])
        if v["sd"] > 0.10:
            first[unit] = min(first.get(unit, 9999), int(y))
    for unit, sd in sorted(worst.items(), key=lambda x: -x[1])[: args.top]:
        mark = " (20 maiores)" if unit in largest else ""
        print(f"- {unit}{mark}: {sd:.2f}, desde {first.get(unit, '–')}")


def compare(args):
    sig = json.load(open(args.band))["cells"]
    base = cells(json.load(open(args.base)))
    new = cells(json.load(open(args.new)))
    largest = set(json.load(open(args.new))["largest"])
    flagged = []
    for k, s in sig.items():
        if k not in base or k not in new:
            continue
        d = new[k] - base[k]
        # Before a perturbation this small spreads (the first decades) the spread is zero: the
        # floor keeps a rule change from reading as millions of deviations there.
        sd = max(s["sd"], args.floor)
        if abs(d) <= args.sigmas * sd:
            continue
        kind = k.split("|")[0]
        if kind in ("unit", "world", "share_ratio", "production_ratio"):
            closer = abs(math.log(new[k])) < abs(math.log(base[k]))
            verdict = "melhor" if closer else "pior"
        elif kind == "rank":
            verdict = "melhor" if new[k] > base[k] else "pior"
        else:
            verdict = "–"
        flagged.append((kind, k, base[k], new[k], d / sd, verdict))
    print(f"Fora de {args.sigmas} desvios: {len(flagged)} de {len(sig)} números\n")
    order = {"world": 0, "rank": 1, "production_ratio": 2, "production": 3, "share_ratio": 4, "share": 5, "unit": 6}
    flagged.sort(key=lambda f: (order.get(f[0], 9), -abs(f[4])))
    print("| Número | Base | Branch | Desvios | Contra o real |")
    print("|---|---|---|---|---|")
    for kind, k, b, n, z, verdict in flagged:
        _, name, y, m = k.split("|")
        if kind == "unit" and args.largest_only and name not in largest:
            continue
        print(f"| {kind} {name} {y} {m} | {b:.3f} | {n:.3f} | {z:+.1f} | {verdict} |")
    worse = [
        f for f in flagged if f[0] == "unit" and f[1].split("|")[1] in largest and f[5] == "pior"
    ]
    print(f"\nCélulas dos 20 maiores fora da faixa para pior: {len(worse)}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    b = sub.add_parser("band")
    b.add_argument("--runs", nargs="+", required=True)
    b.add_argument("--out", required=True)
    b.add_argument("--top", type=int, default=15)
    c = sub.add_parser("compare")
    c.add_argument("--band", required=True)
    c.add_argument("--base", required=True)
    c.add_argument("--new", required=True)
    c.add_argument("--sigmas", type=float, default=2.0)
    c.add_argument("--floor", type=float, default=0.01, help="smallest spread used, in ratio units")
    c.add_argument("--largest-only", action="store_true", help="list only the 20 largest units' cells")
    args = ap.parse_args()
    band(args) if args.cmd == "band" else compare(args)


if __name__ == "__main__":
    main()

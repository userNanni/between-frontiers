#!/usr/bin/env python3
"""Compara uma rodada de 101 anos com a faixa de reembaralhamento da base, país a país e por
década.

    band.py BASE_DIR NEW_HISTORY.json [--keys gdp,population,...] [--k 2] [--years 1935,...]
            [--tags ITA,GBR] [--all]

BASE_DIR é `target/baseline/<sha>/` (de `baseline.sh` e `band.sh`): a base e as amostras de
`band/p*/history.json`. Para cada território (`history.json`, `territories`: país e parte), cada
série e cada ano de década, a faixa é a média ± k desvios-padrão da base com as amostras
(k = 2 por padrão). Mostra os territórios fora da faixa, com o desvio em σ, e um resumo por
série e por década. Com menos de oito amostras o desvio-padrão erra demais (±25 % com oito,
±50 % com três): o script avisa."""
import argparse, glob, json, os, statistics, sys

DEFAULT_KEYS = 'population,gdp,gdp_per_capita,literacy,non_farm'


def load(path):
    h = json.load(open(path))
    years = h['years']
    out = {}
    for t in h['territories']:
        out[(t['tag'], t.get('part', ''))] = {k: v for k, v in t.items() if isinstance(v, list)}
    return years, out


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument('base')
    p.add_argument('new')
    p.add_argument('--keys', default=DEFAULT_KEYS)
    p.add_argument('--k', type=float, default=2.0)
    p.add_argument('--years', default='')
    p.add_argument('--tags', default='')
    p.add_argument('--all', action='store_true', help='mostra também os que ficam dentro')
    a = p.parse_args()
    samples = [os.path.join(a.base, 'history.json')] + sorted(glob.glob(os.path.join(a.base, 'band', 'p*', 'history.json')))
    samples = [s for s in samples if os.path.exists(s)]
    if len(samples) < 9:
        print(f'aviso: {len(samples)} rodadas na faixa (base + amostras); o guia pede base + 8', file=sys.stderr)
    if len(samples) < 3:
        sys.exit('faixa sem amostras: rode tools/bench/band.sh')
    years, new = load(a.new)
    runs = [load(s)[1] for s in samples]
    keys = a.keys.split(',')
    want = [int(y) for y in a.years.split(',') if y] or [y for y in years if y % 10 == 5 or y == years[-1]]
    tags = set(a.tags.split(',')) if a.tags else None
    idx = {y: i for i, y in enumerate(years)}
    out_count = {}
    rows = []
    for terr, series in sorted(new.items()):
        if tags and terr[0] not in tags:
            continue
        for k in keys:
            if k not in series:
                continue
            for y in want:
                i = idx.get(y)
                if i is None:
                    continue
                vals = [r[terr][k][i] for r in runs if terr in r and k in r[terr] and i < len(r[terr][k])]
                if len(vals) < 3 or not isinstance(series[k][i], (int, float)):
                    continue
                m, sd = statistics.mean(vals), statistics.stdev(vals)
                v = series[k][i]
                z = (v - m) / sd if sd > 0 else (0.0 if v == m else float('inf'))
                outside = abs(z) > a.k
                out_count.setdefault((k, y), [0, 0])
                out_count[(k, y)][0] += outside
                out_count[(k, y)][1] += 1
                if outside or a.all:
                    rows.append((terr, k, y, v, m, sd, z))
    name = lambda t: t[0] + (f'/{t[1]}' if t[1] else '')
    print(f'{len(samples)} rodadas na faixa; fora = |desvio| > {a.k:g} σ')
    print(f"{'território':14} {'série':15} {'ano':>5} {'valor':>14} {'média':>14} {'σ':>12} {'desvio':>8}")
    for terr, k, y, v, m, sd, z in sorted(rows, key=lambda r: -abs(r[6]))[:200]:
        print(f'{name(terr):14} {k:15} {y:>5} {v:>14.4g} {m:>14.4g} {sd:>12.3g} {z:>+7.1f}σ')
    print('\nfora da faixa por série e década (territórios fora / comparados; por acaso, ~5 % a 2σ):')
    for k in keys:
        cells = [f'{y}: {out_count[(k, y)][0]}/{out_count[(k, y)][1]}' for y in want if (k, y) in out_count]
        if cells:
            print(f'  {k:15} ' + '  '.join(cells))


if __name__ == '__main__':
    main()

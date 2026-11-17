#!/usr/bin/env python3
"""Livro de rodadas longas: uma linha JSON por rodada em
~/Projects/between-frontiers/target/runs/ledger.jsonl.

    ledger.py append --out DIR --commit SHA --branch B --load L0 L1 --exit N --args "..."
        lê DIR/game.json e DIR/cpu.txt (de run-long.sh) e acrescenta a linha
    ledger.py show [--branch B] [--last N]
        CPU por rodada (total e sistemas mais caros) e, por década, a dispersão do PIB e da
        população mundiais entre as rodadas de mesmo hash e entre todas

Cada linha: data, commit, ramo, hash final, anos, threads, CPU do usuário e do sistema,
parede, tempo de simulação, CPU por sistema, população e PIB mundiais por década, carga de
1 min no início e no fim, código de saída e argumentos."""
import argparse, datetime, json, os, socket, statistics, sys

LEDGER = os.environ.get('BF_LEDGER') or os.path.expanduser('~/Projects/between-frontiers/target/runs/ledger.jsonl')


def append(a):
    rep = json.load(open(os.path.join(a.out, 'game.json')))
    user, sys_, wall = (float(x) for x in open(os.path.join(a.out, 'cpu.txt')).read().split()[-3:])
    row = {
        'date': datetime.datetime.now().isoformat(timespec='seconds'),
        'host': socket.gethostname(),
        'commit': a.commit, 'branch': a.branch, 'args': a.args, 'exit': a.exit,
        'hash': rep['final_hash'], 'years': rep['years'], 'threads': rep['threads'],
        'cpu_user_s': user, 'cpu_sys_s': sys_, 'wall_s': wall,
        'sim_s': round(rep['total_ms'] / 1000, 1),
        'load': [float(x) for x in a.load],
        'systems': {s['name']: round(s['total_ms'] / 1000, 2) for s in rep['systems']},
        'decades': [{'year': d['year'], 'population': d['population'], 'gdp': d['gdp']}
                    for d in rep['decades']],
        'out': os.path.abspath(a.out),
    }
    os.makedirs(os.path.dirname(LEDGER), exist_ok=True)
    with open(LEDGER, 'a') as f:
        f.write(json.dumps(row) + '\n')


def spread(vals):
    if len(vals) < 2:
        return '—'
    m = statistics.mean(vals)
    return f'±{(max(vals) - min(vals)) / 2 / m * 100:.1f} %' if m else '—'


def show(a):
    rows = [json.loads(l) for l in open(LEDGER) if l.strip()]
    if a.branch:
        rows = [r for r in rows if r['branch'] == a.branch]
    rows = rows[-a.last:]
    print(f"{'data':19} {'commit':8} {'ramo':28} {'hash':8} {'anos':>4} {'thr':>3} "
          f"{'CPU s':>7} {'sim s':>6} {'carga':>5}  mais caros")
    for r in rows:
        top = sorted(r['systems'].items(), key=lambda kv: -kv[1])[:3]
        print(f"{r['date']:19} {r['commit'][:8]:8} {r['branch'][:28]:28} {r['hash'][:8]:8} "
              f"{r['years']:>4} {r['threads']:>3} {r['cpu_user_s']:>7.0f} {r['sim_s']:>6.0f} "
              f"{r['load'][0]:>5.1f}  " + ', '.join(f'{k} {v:.0f}' for k, v in top))
    # CPU trend: median per commit, for the same years and threads.
    groups = {}
    for r in rows:
        groups.setdefault((r['commit'][:8], r['years'], r['threads']), []).append(r)
    if len(groups) > 1 or any(len(v) > 1 for v in groups.values()):
        print('\nCPU por commit (mediana; anos, threads):')
        for (c, y, t), rs in groups.items():
            cpu = [r['cpu_user_s'] for r in rs]
            print(f"  {c} {y:>4} a {t} thr: {statistics.median(cpu):7.0f} s em {len(rs)} rodada(s), "
                  f"hashes {len({r['hash'] for r in rs})}")
    # Dispersion of the world aggregates by decade, across the runs shown.
    # Census decades only (1925, the years ending in 5, the last), also from BF_VERBOSE runs.
    years = sorted({d['year'] for r in rows for d in r['decades']
                    if d['year'] % 10 == 5 or d['year'] == r['decades'][-1]['year']})
    if rows and years:
        print('\nPIB e população mundiais por década entre as rodadas acima (meia amplitude / média):')
        print(f"{'ano':>5} {'n':>3} {'PIB (bn)':>10} {'disp.':>8} {'pop (M)':>9} {'disp.':>8}")
        for y in years:
            g = [d['gdp'] for r in rows for d in r['decades'] if d['year'] == y]
            p = [d['population'] for r in rows for d in r['decades'] if d['year'] == y]
            print(f"{y:>5} {len(g):>3} {statistics.mean(g) / 1e9:>10.0f} {spread(g):>8} "
                  f"{statistics.mean(p) / 1e6:>9.0f} {spread(p):>8}")


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest='cmd', required=True)
    ap = sub.add_parser('append')
    ap.add_argument('--out', required=True)
    ap.add_argument('--commit', default='')
    ap.add_argument('--branch', default='')
    ap.add_argument('--load', nargs=2, default=['0', '0'])
    ap.add_argument('--exit', type=int, default=0)
    ap.add_argument('--args', default='')
    sp = sub.add_parser('show')
    sp.add_argument('--branch')
    sp.add_argument('--last', type=int, default=30)
    a = p.parse_args()
    append(a) if a.cmd == 'append' else show(a)


if __name__ == '__main__':
    sys.exit(main())

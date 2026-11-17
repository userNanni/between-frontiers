#!/usr/bin/env python3
"""Agrega as pilhas de `sample.sh`: custo próprio por função e por linha, e a linha do jogo
mais interna de cada amostra (onde, no código de `bf-sim`, o tempo foi gasto).

    python3 tools/bench/profile.py OUTDIR BIN [filtro] [linhas]

`filtro` guarda só as amostras com uma função cujo nome o contém (ex.: `week::`)."""
import collections, glob, re, subprocess, sys

d, binp = sys.argv[1], sys.argv[2]
filt = sys.argv[3] if len(sys.argv) > 3 and sys.argv[3] else None
K = int(sys.argv[4]) if len(sys.argv) > 4 else 30
base = int(open(d + '/maps.txt').read().split('-')[0], 16)
IDLE = ('syscall', 'futex', 'sched_yield', 'lll', 'pthread_cond', 'nanosleep', 'epoll')

raw = []
for f in glob.glob(d + '/s*.txt'):
    cur = None
    for line in open(f):
        if line.startswith('TID'):
            if cur:
                raw.append(cur)
            cur = []
        else:
            m = re.match(r'#(\d+)\s+0x([0-9a-f]+)\s+(\S+)', line)
            if m and cur is not None:
                cur.append((int(m.group(2), 16), m.group(3)))
    if cur:
        raw.append(cur)
busy = [s for s in raw if s and not any(k in s[0][1] for k in IDLE)]

# Return addresses point after the call: step back one byte to land on the call's line.
def off(i, a):
    return a - base - (1 if i else 0)

addrs = sorted({off(i, a) for s in busy for i, (a, _) in enumerate(s) if base <= a < base + 0x10000000})
out = subprocess.run(['addr2line', '-e', binp, '-i', '-f', '-C', '-a'] + [hex(a) for a in addrs],
                     capture_output=True, text=True).stdout.split('\n')

def clean(n):
    n = re.sub(r'::h[0-9a-f]{16}', '', n)
    n = re.sub(r'<([\w:]+?)(?:<[^>]*>)?\s+as\s+[^>]*>', lambda m: '<' + '::'.join(m.group(1).split('::')[-2:]) + '>', n)
    n = re.sub(r'::\{closure[^}]*\}', '::{c}', n)
    return n[:120]

res, cur, i = {}, None, 0
while i < len(out):
    l = out[i]
    if l.startswith('0x'):
        cur = int(l, 16); res[cur] = []; i += 1
    elif cur is not None and l:
        path = out[i + 1].split(' ')[0] if i + 1 < len(out) else ''
        # Game code: our crates, except the fixed-point arithmetic itself.
        game = '/crates/' in path and not path.rsplit(':', 1)[0].endswith(('fixed.rs', 'accum.rs'))
        res[cur].append((clean(l), path.split('/')[-1], game)); i += 2
    else:
        i += 1

stacks = []
for s in busy:
    fr = []
    for i, (a, sym) in enumerate(s):
        fr += res.get(off(i, a), [(sym[:60], '', False)])
    stacks.append(fr)
if filt:
    stacks = [s for s in stacks if any(filt in f for f, _, _ in s)]
N = max(len(stacks), 1)
print(f'amostras ocupadas {len(stacks)} de {len(raw)} pilhas')

own, own_line, game_line, game_fn = (collections.Counter() for _ in range(4))
for s in stacks:
    own[s[0][0]] += 1
    own_line[f'{s[0][0]} @{s[0][1]}'] += 1
    hits = [(f, l) for f, l, game in s if game]
    if hits:
        game_line[hits[0][1]] += 1
        game_fn[hits[0][0]] += 1

for title, c in (('custo próprio', own), ('custo próprio por linha', own_line),
                 ('linha do jogo mais interna', game_line), ('função do jogo mais interna', game_fn)):
    print(f'--- {title}')
    for k, n in c.most_common(K):
        print(f'{n * 100 / N:5.1f}% {k}')

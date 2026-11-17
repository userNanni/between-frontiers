#!/bin/bash
# Rodada longa do bench com registro no livro de rodadas.
#
#   tools/bench/run-long.sh OUTDIR BIN [argumentos do bf-bench]
#   (padrão: --game --years 101 --threads 2)
#
# Grava em OUTDIR: run.txt (saída), game.json, history.json, cpu.txt e status ("ok" ou
# "fail <código>"), e acrescenta uma linha a ~/Projects/between-frontiers/target/runs/ledger.jsonl.
# Rode por `bf-bench-slot` e numa unidade `systemd-run --user`; a tarefa em segundo plano de um
# agente morre em 30 min. COMMIT e BRANCH no ambiente dizem o que o binário é; sem eles, vêm do
# git do diretório atual.
set -u
export PATH=$HOME/.cargo/bin:$HOME/.local/bin:$PATH
out=$1; bin=$2; shift 2
[ $# -eq 0 ] && set -- --game --years 101 --threads 2
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$out"; rm -f "$out/status"
load0=$(cut -d' ' -f1 /proc/loadavg)
/usr/bin/time -f '%U %S %e' -o "$out/cpu.txt" "$bin" "$@" --out "$out/game.json" \
  --history-out "$out/history.json" > "$out/run.txt" 2>&1
code=$?
load1=$(cut -d' ' -f1 /proc/loadavg)
commit=${COMMIT:-$(git rev-parse HEAD 2>/dev/null)}
branch=${BRANCH:-$(git rev-parse --abbrev-ref HEAD 2>/dev/null)}
python3 "$here/ledger.py" append --out "$out" --commit "$commit" --branch "$branch" \
  --load "$load0" "$load1" --exit "$code" --args "$*" || true
if [ "$code" -eq 0 ]; then echo ok > "$out/status"; else echo "fail $code" > "$out/status"; fi
exit "$code"

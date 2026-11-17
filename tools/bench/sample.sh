#!/bin/bash
# Amostrador de pilha para quando `perf` não roda (perf_event_open bloqueado): tira a pilha de
# todas as threads com `eu-stack` a cada GAP segundos. Resolva e agregue com `profile.py`.
#
#   SKIP=6 GAP=0.05 tools/bench/sample.sh OUTDIR target/release/bf-bench --game --years 2 --threads 1
#   python3 tools/bench/profile.py OUTDIR target/release/bf-bench [filtro] [linhas]
#
# Precisa de `eu-stack` (elfutils), `addr2line` (binutils), `kernel.yama.ptrace_scope` = 0 e o
# binário com `debug = "line-tables-only"` (o perfil release já tem). SKIP pula a carga do mundo.
out=$1; shift
rm -rf "$out"; mkdir -p "$out"
"$@" > "$out/run.txt" 2>&1 &
pid=$!
sleep "${SKIP:-6}"
grep -m1 "$(readlink -f "$1")" "/proc/$pid/maps" > "$out/maps.txt"
i=0
while kill -0 "$pid" 2>/dev/null; do
  eu-stack -r -p "$pid" -n 60 > "$out/s$i.txt" 2>/dev/null
  i=$((i+1)); sleep "${GAP:-0.05}"
done
wait "$pid"
echo "samples $i"

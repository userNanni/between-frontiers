#!/bin/bash
# Faixa de reembaralhamento da `main` num commit: oito rodadas de 101 anos com um empurrão
# mínimo (BF_PERTURB_PERMILLE = ±1..±4, cada fundo de investimento N por mil maior ou menor),
# guardadas ao lado da base: ~/Projects/between-frontiers/target/baseline/<sha>/band/p<N>/.
#
#   tools/bench/band.sh [COMMIT]     (padrão: origin/main)
#
# Garante a base primeiro (baseline.sh). Cada amostra roda uma vez só por sha (flock), cada uma
# no seu pedido ao `bf-bench-slot`, e entra no livro de rodadas com o ramo `band`. Leva umas 3 h
# de slot: chame de dentro de uma unidade `systemd-run --user`; o fim é o arquivo band/status.
# Compare uma rodada com a faixa com `tools/bench/band.py`.
set -eu
export PATH=$HOME/.cargo/bin:$HOME/.local/bin:$PATH
here=$(cd "$(dirname "$0")" && pwd)
dir=$("$here/baseline.sh" "${1:-origin/main}" | tail -1)
sha=$(basename "$dir")
mkdir -p "$dir/band"
slot=$HOME/.local/bin/bf-bench-slot; [ -x "$slot" ] || slot=
fail=0
for n in -1 1 -2 2 -3 3 -4 4; do
  out=$dir/band/p$n
  (
    flock 9
    [ "$(cat "$out/status" 2>/dev/null)" = ok ] && exit 0
    cd "$dir/src"
    BF_PERTURB_PERMILLE=$n COMMIT=$sha BRANCH=band $slot "$dir/tools/run-long.sh" "$out" \
      "$dir/bf-bench" --game --years 101 --threads 2
  ) 9>"$dir/band/p$n.lock" || fail=1
done
if [ $fail -eq 0 ]; then echo ok > "$dir/band/status"; else echo fail > "$dir/band/status"; fi
echo "$dir/band"

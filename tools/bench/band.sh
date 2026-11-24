#!/bin/bash
# Faixa de reembaralhamento da `main` num commit: catorze rodadas de 101 anos, cada uma com um
# empurrão pequeno de um tipo, guardadas ao lado da base:
# ~/Projects/between-frontiers/target/baseline/<sha>/band/p<nome>/.
#   p±2, p±4   BF_PERTURB_PERMILLE: todo fundo de investimento N por mil maior ou menor;
#   ps1, ps2   BF_PERTURB_SIGNED: cada fundo ±3 por mil, o sinal sorteado por país;
#   pr±20, pr±50  BF_PERTURB_RESEARCH: toda tecnologia N por mil mais barata (ou mais cara);
#   pt1..pt4   BF_PERTURB_TECH: as datas históricas de cada tecnologia um ano antes ou depois,
#              sorteado por tecnologia.
# Uma faixa só de investimento era estreita para mudanças de meio de século (três mudanças
# diferentes moveram 1965 em ~3σ cada): a pesquisa e as datas mexem nessa época.
#
#   tools/bench/band.sh [COMMIT]     (padrão: origin/main)
#
# Garante a base primeiro (baseline.sh). Cada amostra roda uma vez só por sha (flock), cada uma
# no seu pedido ao `bf-bench-slot`, e entra no livro de rodadas com o ramo `band`. Leva umas 2 h
# de slot: chame de dentro de uma unidade `systemd-run --user`; o fim é o arquivo band/status.
# Compare uma rodada com a faixa com `tools/bench/band.py`.
set -eu
export PATH=$HOME/.cargo/bin:$HOME/.local/bin:$PATH
here=$(cd "$(dirname "$0")" && pwd)
dir=$("$here/baseline.sh" "${1:-origin/main}" | tail -1)
sha=$(basename "$dir")
mkdir -p "$dir/band"
slot=$HOME/.local/bin/bf-bench-slot; [ -x "$slot" ] || slot=
# BF_BAND_BENCH: outro bf-bench para as amostras (um build de um ramo que só muda a ferramenta,
# sobre o mesmo commit, quando o da base ainda não conhece uma perturbação).
bench=${BF_BAND_BENCH:-$dir/bf-bench}
fail=0
for spec in -2:BF_PERTURB_PERMILLE=-2 2:BF_PERTURB_PERMILLE=2 -4:BF_PERTURB_PERMILLE=-4 \
  4:BF_PERTURB_PERMILLE=4 s1:BF_PERTURB_SIGNED=1 s2:BF_PERTURB_SIGNED=2 \
  r20:BF_PERTURB_RESEARCH=20 r-20:BF_PERTURB_RESEARCH=-20 r50:BF_PERTURB_RESEARCH=50 \
  r-50:BF_PERTURB_RESEARCH=-50 t1:BF_PERTURB_TECH=1 t2:BF_PERTURB_TECH=2 t3:BF_PERTURB_TECH=3 \
  t4:BF_PERTURB_TECH=4; do
  n=${spec%%:*}; var=${spec#*:}
  out=$dir/band/p$n
  (
    flock 9
    [ "$(cat "$out/status" 2>/dev/null)" = ok ] && exit 0
    cd "$dir/src"
    env "$var" BF_DATA_ROOT="$dir/src" COMMIT=$sha BRANCH=band $slot "$dir/tools/run-long.sh" "$out" \
      "$bench" --game --years 101 --threads 2
  ) 9>"$dir/band/p$n.lock" || fail=1
done
if [ $fail -eq 0 ]; then echo ok > "$dir/band/status"; else echo fail > "$dir/band/status"; fi
echo "$dir/band"

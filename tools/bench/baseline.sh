#!/bin/bash
# Rodada de 101 anos da `main` num commit, guardada para todos os threads.
#
#   tools/bench/baseline.sh [COMMIT]     (padrão: origin/main)
#
# Devolve (imprime) o diretório ~/Projects/between-frontiers/target/baseline/<sha>/, com
# bf-bench, game.json, history.json, hash.txt, cpu.txt, systems.txt e status. Se a rodada já
# existe, só imprime o caminho. Se não, compila o commit (fonte por `git archive`, sem mexer em
# nenhum checkout), roda por `bf-bench-slot` e registra no livro de rodadas. Dois pedidos do
# mesmo sha esperam um pelo outro (`flock` por sha). Leva ~25 min: chame de dentro de uma
# unidade `systemd-run --user` e espere o arquivo `status`.
set -eu
export PATH=$HOME/.cargo/bin:$HOME/.local/bin:$PATH
here=$(cd "$(dirname "$0")" && pwd)
repo=$HOME/Projects/between-frontiers
git -C "$repo" fetch -q origin
sha=$(git -C "$repo" rev-parse --verify "${1:-origin/main}^{commit}")
base=$repo/target/baseline
dir=$base/$sha
mkdir -p "$base"
exec {lock}>"$base/$sha.lock"
flock "$lock"
if [ "$(cat "$dir/status" 2>/dev/null)" = ok ]; then echo "$dir"; exit 0; fi
rm -rf "$dir"; mkdir -p "$dir/src" "$dir/tools"
# The helpers are copied first: the run takes ~25 min and the caller's checkout may change.
cp "$here/run-long.sh" "$here/ledger.py" "$here/compare.py" "$dir/tools/"
git -C "$repo" archive "$sha" | tar -x -C "$dir/src"
# Os builds de base dividem um target (só dependências se reaproveitam) e vão um de cada vez.
cargo=cargo; [ -x "$HOME/.local/bin/bf-cargo" ] && cargo=$HOME/.local/bin/bf-cargo
(
  flock 9
  cd "$dir/src"
  CARGO_TARGET_DIR=$base/.target nice -n 10 "$cargo" build --release -j 2 -p bf-bench \
    > "$dir/build.log" 2>&1
  cp "$base/.target/release/bf-bench" "$dir/bf-bench"
) 9>"$base/.build.lock"
# O binário lê os dados de BF_DATA_ROOT, explícito: a fonte desta base, nunca o caminho em que o
# alvo compartilhado o compilou (a base de 61a1918 leu a cópia de outro thread). src/ fica, sem
# target.
slot=$HOME/.local/bin/bf-bench-slot; [ -x "$slot" ] || slot=
cd "$dir/src"
export BF_DATA_ROOT="$dir/src"
COMMIT=$sha BRANCH=main $slot "$dir/tools/run-long.sh" "$dir" "$dir/bf-bench" \
  --game --years 101 --threads 2 || { echo "rodada falhou: $dir/run.txt" >&2; exit 1; }
grep -o 'final hash [0-9a-f]*' "$dir/run.txt" | cut -d' ' -f3 > "$dir/hash.txt"
python3 "$dir/tools/compare.py" "$dir/game.json" > "$dir/systems.txt" || true
echo "$dir"

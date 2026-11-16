#!/usr/bin/env bash
# The historical scorecard in one command: runs the campaign from 1925 to 2026-01-01 with
# `--history-out`, then compares it with data/history/ and writes target/history/scorecard.md,
# scorecard.json and the PNG charts. See docs/history.pt-BR.md.
#
#   tools/history/run.sh                 # build, run 101 years, score
#   tools/history/run.sh --score-only    # score target/history/history.json again
#
# Environment: BF_YEARS (default 101), BF_THREADS (default 0 = all cores), BF_LOCK (a lock file
# for `flock`, so that only one long run uses the machine at a time; empty = no lock).
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
out="$root/target/history"
mkdir -p "$out"
cd "$root"

if [[ "${1:-}" != "--score-only" ]]; then
  cargo build -p bf-bench --release
  # A copy, so that a rebuild during the run does not swap the binary underneath it.
  cp target/release/bf-bench "$out/bf-bench"
  cmd=("$out/bf-bench" --game --years "${BF_YEARS:-101}" --threads "${BF_THREADS:-0}"
       --out "$out/game.json" --history-out "$out/history.json")
  if [[ -n "${BF_LOCK:-}" ]]; then
    cmd=(flock "$BF_LOCK" "${cmd[@]}")
  fi
  "${cmd[@]}" | tee "$out/run.log"
fi

if [[ ! -f data/history/population.csv ]]; then
  python3 tools/history/fetch.py
fi

# Charts need matplotlib: use it if installed, otherwise a private virtualenv under target/.
python=python3
if ! python3 -c 'import matplotlib' 2>/dev/null; then
  if [[ ! -x "$out/.venv/bin/python" ]]; then
    python3 -m venv "$out/.venv"
    "$out/.venv/bin/pip" install --quiet matplotlib
  fi
  python="$out/.venv/bin/python"
fi
"$python" tools/history/scorecard.py --sim "$out/history.json" --out "$out"

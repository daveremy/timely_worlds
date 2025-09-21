#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="${SCRIPT_DIR}/.."
cd "$ROOT"

OUTPUT_DIR="${1:-metrics/mfg}"
shift || true
mkdir -p "$OUTPUT_DIR"

BEAM_WIDTHS=(12 16 24)
DEPTHS=(3 4)
BACKLOG_THRESHOLDS=(6 8)
PROB_THRESHOLDS=(0.25 0.35)

for beam in "${BEAM_WIDTHS[@]}"; do
  for depth in "${DEPTHS[@]}"; do
    for backlog in "${BACKLOG_THRESHOLDS[@]}"; do
      for prob in "${PROB_THRESHOLDS[@]}"; do
        outfile="$OUTPUT_DIR/mfg_beam${beam}_depth${depth}_back${backlog}_prob${prob}.jsonl"
        echo "Running mfg demo: beam_width=${beam}, max_depth=${depth}, backlog_threshold=${backlog}, prob_threshold=${prob}" >&2
        scripts/run_mfg_demo.sh \
          --beam-width "$beam" \
          --max-depth "$depth" \
          --backlog-threshold "$backlog" \
          --prob-threshold "$prob" \
          "$@" \
          > "$outfile"
      done
    done
  done
 done

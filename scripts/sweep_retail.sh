#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="${SCRIPT_DIR}/.."
cd "$ROOT"

OUTPUT_DIR="${1:-metrics/retail}"
shift || true
mkdir -p "$OUTPUT_DIR"

TOP_K_VALUES=(5)
BEAM_WIDTHS=(16 32 64)
DEPTHS=(5 8)
BRANCH_PROBS=(0.35 0.5)

for topk in "${TOP_K_VALUES[@]}"; do
  for beam in "${BEAM_WIDTHS[@]}"; do
    for depth in "${DEPTHS[@]}"; do
      for branch in "${BRANCH_PROBS[@]}"; do
        outfile="$OUTPUT_DIR/retail_k${topk}_beam${beam}_depth${depth}_branch${branch}.jsonl"
        echo "Running retail demo: top_k=${topk}, beam_width=${beam}, max_depth=${depth}, branch_prob=${branch}" >&2
        scripts/run_retail_demo.sh \
          --top-k "$topk" \
          --beam-width "$beam" \
          --max-depth "$depth" \
          --branch-prob "$branch" \
          "$@" \
          > "$outfile"
      done
    done
  done
 done

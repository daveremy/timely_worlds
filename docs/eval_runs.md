Planned Evaluation Sweeps
=========================

Retail
------
- Script: `scripts/sweep_retail.sh <output_dir> [extra demo flags]`
- Defaults (override inside script):
  - `TOP_K_VALUES=(5)`
  - `BEAM_WIDTHS=(16 32 64)`
  - `DEPTHS=(5 8)`
  - `BRANCH_PROBS=(0.35 0.5)`
- Produces JSONL metrics per run; summarize with `scripts/summarize_metrics.py metrics/retail/*.jsonl --pretty`.

Manufacturing
-------------
- Script: `scripts/sweep_mfg.sh <output_dir> [extra demo flags]`
- Defaults:
  - `BEAM_WIDTHS=(12 16 24)`
  - `DEPTHS=(3 4)`
  - `BACKLOG_THRESHOLDS=(6 8)`
  - `PROB_THRESHOLDS=(0.25 0.35)`
- Summaries: `scripts/summarize_metrics.py metrics/mfg/*.jsonl --pretty`.

Result Logging Template
-----------------------
Create `docs/eval_results/` with subfolders per sweep. For each run log:

| Domain | Top-K | Beam Width | Max Depth | Branch/Prob | Base Events | Predicted Events | Avg Epoch (ms) | Alerts | Active Peak |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| retail | 5 | 16 | 5 | 0.35 | | | | | |
| retail | 5 | 32 | 8 | 0.50 | | | | | |
| mfg | - | 16 | 3 | backlog=6, p=0.25 | | | | | |

Fill in values from `summarize_metrics.py` output and raw JSONL files.

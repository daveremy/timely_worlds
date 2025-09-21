Timely Worlds

This repo explores depth-bounded predictive branching over incremental views using Timely Dataflow and Differential Dataflow. The core idea: maintain a base world view W from streaming (or sequential) events, branch predicted futures W'..W^D as lightweight scenario overlays, and let users subscribe to whether a condition becomes true in any scenario within a bounded depth and probability threshold.

Status
- Docs and workspace scaffolding are in place.
- Two demo domains planned: Retail and Manufacturing.
- Next: implement the base world (retail) with core views.

Docs
- See `docs/README.md` for the index and navigation.

Running Demos
- Retail: `scripts/run_retail_demo.sh --batches 5 --batch-size 150 --beam-width 64 --max-depth 8 > retail_metrics.jsonl`
- Manufacturing: `scripts/run_mfg_demo.sh --ops-per-batch 150 --beam-width 24 --backlog-threshold 8 > mfg_metrics.jsonl`
- All demos accept `--help` for full parameter lists; logs emit JSON (`metrics_json`) per epoch and a final summary for evaluation.
- Summaries: `scripts/summarize_metrics.py retail_metrics.jsonl --pretty`

Workspace Layout
- `crates/core`: core types and traits (events, diffs, ids)
- `crates/runtime`: timely/differential wiring and runtime helpers
- `crates/views`: reusable view builders (top-K, windows, joins, graphs)
- `crates/predictors`: predictor trait and adapters (rules/ML/LLM stubs)
- `crates/scenarios`: scenario overlays and beam/pruning manager
- `crates/examples`: binaries: `retail_demo`, `mfg_demo`

Naming
- The name is a placeholder; alternatives include Differential Futures, Manyfold, and Foresight Flow.

License
- To be decided.

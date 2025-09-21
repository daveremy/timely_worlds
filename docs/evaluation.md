Evaluation Plan

Metrics
- Latency: per‑event end‑to‑end update time (p50/p95).
- Throughput: events/sec and predictions/sec processed.
- Memory: arrangement sizes, per‑scenario overlay footprint.
- Accuracy: alert precision/recall or calibration vs. synthetic truth.
- Efficiency: effect of sharing (with vs. without shared arrangements).

Datasets
- Synthetic Retail: configurable order rates, SKU popularity (Zipf), inventory policies, return behavior.
- Synthetic Manufacturing: job graphs (routing), machine capacities, failure/repair distributions, material delays.

Experiments
1) Scalability vs K and D
   - Vary beam width (K ∈ {8, 32, 128}) and depth (D ∈ {3, 5, 10}).
   - Record latency/throughput/memory and alert counts.
2) Sharing Benefit
   - Compare overlays with shared arrangements vs duplicated state.
   - Measure memory and CPU savings.
3) Approximate vs Exact Views
   - Replace exact top‑K with sketches; compare speed/accuracy tradeoffs.
4) Predictor Latency Sensitivity
   - Inject delays; observe backpressure and staleness impacts.
5) Subscription Load
   - Increase number/complexity of subscriptions; measure incremental cost.

Reporting
- JSONL metrics emitted per epoch (label `retail_epoch`, `mfg_epoch`) and final summary lines (`retail_final`, `mfg_final`).
- Parse fields: `base_events`, `predicted_events`, `scenario_created`, `scenario_retired`, `scenario_alerts`, `scenario_active_peak`, `elapsed_ms`.
- Aggregate into plots of latency/throughput vs K, D; memory instrumentation TBD.
- Alert precision/recall over runs (requires synthetic truth logs).

Collection Workflow
1. Run the demos with desired parameters (`scripts/run_retail_demo.sh --help`, `scripts/run_mfg_demo.sh --help`) and redirect stdout to JSONL.
2. Summarize metrics with `scripts/summarize_metrics.py retail_metrics.jsonl --pretty` (supports multiple files and stdin).
3. Use `jq`/Python notebooks for deeper analysis: filter by `label` (`retail_epoch`, `mfg_epoch`, etc.) and compute aggregates such as throughput, alert counts, and scenario churn.
4. Sweep beam width (K), max depth (D), branch probability, and workload size using CLI flags; chart latency/throughput vs. configuration.

Targets (MVP)
- Sub‑second p95 per update at moderate rates (O(1–5K)/s) on a single machine.
- Depth D up to 10 with K up to 100 under GB‑scale memory.

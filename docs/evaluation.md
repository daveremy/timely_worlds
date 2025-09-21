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
- Plots of latency/throughput vs K, D.
- Memory vs active scenarios and time.
- Alert precision/recall over runs.

Targets (MVP)
- Sub‑second p95 per update at moderate rates (O(1–5K)/s) on a single machine.
- Depth D up to 10 with K up to 100 under GB‑scale memory.


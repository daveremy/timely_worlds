Timely Worlds: Branching Futures over Incremental Views (Concept)

Problem
- We want to predict not just the next event in a stream/sequence, but depth‑bounded sets of plausible future worlds while maintaining efficient, shared state.
- Users ask: "Does condition C become true within depth ≤ D with probability ≥ p?" and receive alerts when satisfied by any scenario branch.

Core Idea
- Maintain a materialized base world view W as a set of incremental relational/graph views over events.
- Branch futures by layering per‑scenario overlays (streams of deltas) on top of W, without duplicating full state.
- Expand scenarios using predictors (rules/ML/LLM), prune using beam/utility, and evaluate subscriptions incrementally.

Key Concepts
- Event: typed fact/change with metadata (time, source, domain).
- Diff: a weighted incremental update (+1/−1 counts, numeric deltas) applied to relations/arrangements.
- World View (W): shared arrangements (indexes) maintained by Differential Dataflow.
- Scenario Overlay (S_i): additional diffs keyed by `scenario_id`, composed with W at query time.
- Depth (d): branching step index (0 = base world). Branching proceeds in waves d = 1..D.
- Probability/Weight: score attached to a scenario path; used for pruning and subscription thresholds.

Why Timely/Differential
- Incremental maintenance: views update with low latency as new events arrive.
- Shared state: arrangements are reused across operators and queries, critical for many scenarios.
- Coordinated progress via frontiers enables backpressure and consistent branching waves.

Subscriptions (Declarative)
- Exists-within-depth: "∃ scenario s with depth ≤ D such that predicate P(view_state_s) is true and weight(s) ≥ p_min".
- Examples:
  - Retail: Customer X enters top‑5 spenders within 10 steps with probability ≥ 0.2.
  - Manufacturing: Any scenario where line 3 becomes the bottleneck in ≤ 2 steps.

Managing Combinatorics
- Beam search: keep top‑K scenarios per depth.
- Utility/pruning: drop dominated or low‑value branches; merge near‑duplicates.
- Approximate summaries (heavy hitters, sketches) where exactness isn’t critical.

Domains
1) Retail
   - Events: OrderPlaced, OrderFulfilled, InventoryAdjusted, Payment, Return.
   - Views: top spenders, SKU velocity, co‑occurrence graphs, stockout risk, sliding metrics.
   - Predictions: next‑order likelihoods, returns, stockout risk; promo impact.
2) Manufacturing
   - Events: JobCreated, OperationStart/Complete, MachineStateChange, Failure, Maintenance, MaterialArrival.
   - Views: WIP by operation, machine utilization/OEE, queue lengths, bottleneck graph, job ETA.
   - Predictions: failure risk, queue buildup, SLA misses.

Feasibility Targets
- Sub‑second incremental updates at moderate event rates (100s‑10Ks/s) on a single node.
- Depth D up to 10 with beam K up to 100, tunable per resource budget.
- Notification latency bounded by branching wave completion (frontier advances).

Non‑Goals (Initial)
- Full probabilistic databases semantics; we use scenario weights heuristically.
- Exact global optimal planning; we use greedy/beam heuristics.

Deliverables
- Reference Rust implementation on Timely/Differential with two domain demos.
- Paper: model and semantics, architecture, algorithms, evaluation.


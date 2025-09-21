Paper Outline (Draft)

Title
- Timely Worlds: Depth‑Bounded Predictive Branching over Incremental Views

Abstract
- We propose a model and system that maintains a shared base world of incremental views over streaming events and expands depth‑bounded predictive scenarios as lightweight overlays. A scenario manager performs beam‑style pruning, while a declarative subscription language alerts when conditions become true in any scenario within depth and probability thresholds. A Rust implementation on Timely/Differential demonstrates feasibility on retail and manufacturing domains.

1. Introduction
- Motivation: from next‑event prediction to branching futures.
- Challenges: combinatorial explosion, latency constraints, incremental maintenance.
- Contributions: overlay model, integrated branching/pruning, declarative subscriptions, evaluation.

2. Background
- Timely Dataflow and Differential Dataflow basics.
- Incremental view maintenance and arrangements.

3. Model and Semantics
- Events, diffs, world views, scenario overlays.
- Depth and probability; subscription semantics (exists‑within‑depth).

4. Architecture
- Ingest, views registry, predictors, scenario manager, subscription engine, runtime.
- Sharing arrangements across scenarios; product timestamps and frontiers.

5. Algorithms
- Beam/pruning strategy; deduplication/merging; approximate summaries.
- Predicate compilation for subscriptions; incremental evaluation.

6. Implementation
- Rust crates, operator designs, async predictors, metrics.

7. Evaluation
- Setup, datasets, metrics, experiments; results across K and D; ablation studies.

8. Related Work
- Probabilistic databases, what‑if analysis, materialized views, streaming analytics, planning/branching search.

9. Discussion
- Tradeoffs, limitations, extensions (distributional summaries, cluster deployment).

10. Conclusion
- Summary and future directions.


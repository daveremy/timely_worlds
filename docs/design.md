Design: Model, Semantics, and Architecture

1) Data Model
- Event: domain‑typed payload + metadata
  - Fields: `domain`, `kind`, `payload` (serde), `epoch` (u64), `source` (string), optional `key`.
- Diffs: signed integer counts or numeric deltas; stored as (+/- value) entries.
- Identifiers: `ScenarioId(u64)`, `Depth(u32)`, `Epoch(u64)`.
- Probability: `f64` weight per scenario path; used for ranking/pruning and subscription thresholds.

2) Semantics
- Base world W: relations maintained from ingested events.
- Overlays: scenario‑scoped diffs applied in addition to W for queries over scenario s.
- Branching time: depth is a logical step separate from event time; we may optionally use product timestamps (epoch, depth) for coordination, but keep `scenario_id` in the data to maximize arrangement sharing.
- Subscriptions:
  - Form: exists depth ≤ D and scenario s with weight ≥ p where P(view_s) holds.
  - Evaluation: incremental; a subscription triggers when P becomes true in any active branch.

3) Views Registry (Shared Arrangements)
- Reusable view builders: top‑K per key, sliding windows, incremental joins, sketches, graph projections.
- Internal form: typed streams (Timely) + arrangements (Differential) keyed for sharing.
- Composition: base W + overlay diffs keyed by scenario when required.

4) Predictors
- Trait: consume selected view changes → emit candidate future events labeled with (scenario_id parent, depth+1, probability, payload diffs).
- Implementations: rules/Markov (initial), ML/LLM adapters (async) later.
- Backpressure: async operators with bounded buffers; predictions join the dataflow as new diffs.

5) Scenario Manager
- Maintains active branches: (scenario_id, depth, weight, lineage/parent, metadata).
- Expansion policy: beam width K, max depth D, utility function, dedup/merge near‑equivalent overlays.
- Lifecycle: create children, assign weights, prune, and retire branches as frontiers advance.

6) Subscription Engine
- DSL to register predicates over views with (D, p_min) and optional per‑domain constraints.
- Compiles to incremental queries; publishes alerts with provenance (scenario ids, depth, explanation).

7) Runtime Architecture
- Ingest normalization → base world views update → predictors observe deltas → scenario manager emits overlays → subscription engine evaluates conditions.
- Frontiers coordinate depth waves and enable consistent notification points.
- Metrics: operator latencies, arrangement sizes, queue depths, alert counts.

8) Repository Layout
- `crates/core`: data types, traits, errors, ids, serde types.
- `crates/runtime`: runtime bootstrap, Timely/Differential wiring, operator utilities, metrics.
- `crates/views`: reusable view builders and helpers.
- `crates/predictors`: predictor trait and baseline implementations.
- `crates/scenarios`: overlay representation and beam/pruning manager.
- `crates/examples`: binaries for retail and manufacturing demos.

9) Defaults (Initial)
- Beam width K = 32, Max depth D = 5, p_min = 0.1.
- Exact top‑K (per key) and sliding windows; approximate modes added later.
- Single‑process Timely runtime; cluster deployment out of scope for MVP.


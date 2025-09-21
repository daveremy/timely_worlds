use anyhow::Result;
use clap::Parser;
use tracing::info;
use tw_runtime::metrics::{EpochTimer, MetricsRegistry};
use tw_runtime::{init_tracing, start_runtime};

use differential_dataflow::input::InputSession;
use differential_dataflow::operators::reduce::Reduce;
use timely::dataflow::operators::probe::{Handle as ProbeHandle, Probe};
use timely::dataflow::operators::{Inspect, Map};

use std::sync::Arc;

use tw_core::retail::{OrderLine, OrderPlaced};
use tw_core::{EventEnvelope, EventMeta};
use tw_predictors::SpendGrowthPredictor;
use tw_scenarios::retail::{RetailBeamConfig, RetailScenarioManager};

#[derive(Parser, Debug)]
#[command(name = "retail_demo", about = "Retail branching futures demo with configurable parameters")]
struct RetailOpts {
    #[arg(long, default_value_t = 5)]
    top_k: usize,
    #[arg(long, default_value_t = 50)]
    customers: u64,
    #[arg(long, default_value_t = 10)]
    batches: u64,
    #[arg(long, default_value_t = 200)]
    batch_size: u64,
    #[arg(long, default_value_t = 5)]
    max_depth: u32,
    #[arg(long, default_value_t = 32)]
    beam_width: usize,
    #[arg(long, default_value_t = 0.1)]
    min_prob: f64,
    #[arg(long, default_value_t = 0.5)]
    branch_prob: f64,
    #[arg(long, default_value_t = 0.3)]
    delta_multiplier: f64,
    #[arg(long, default_value_t = 3_000)]
    min_delta_cents: i64,
    #[arg(long, default_value_t = 7)]
    target_customer: u64,
    #[arg(long, default_value_t = 0.2)]
    prob_threshold: f64,
}

fn main() -> Result<()> {
    init_tracing();
    info!("retail_demo starting");
    let opts = RetailOpts::parse();
    info!(?opts, "retail opts");
    let beam_cfg = RetailBeamConfig {
        max_depth: opts.max_depth,
        beam_width: opts.beam_width,
        min_prob: opts.min_prob,
        branch_prob: opts.branch_prob,
        delta_multiplier: opts.delta_multiplier,
        min_delta_cents: opts.min_delta_cents,
    };
    start_runtime(1, move |_index, worker| {
        info!("retail_demo worker running");

        // Input for typed OrderPlaced events (base world)
        let mut input: InputSession<_, EventEnvelope<OrderPlaced>, isize> = InputSession::new();
        // Predicted overlay input: (scenario_id, customer_id, delta_amount_cents)
        let mut pred_input: InputSession<_, (u64, u64, i64), isize> = InputSession::new();
        // Scenario weights: (scenario_id, probability)
        let mut scen_weight_input: InputSession<_, (u64, f64), isize> = InputSession::new();
        let mut probe = ProbeHandle::new();

        let predictor = Arc::new(SpendGrowthPredictor::default());
        let mut scenario_manager = RetailScenarioManager::new(beam_cfg.clone(), predictor);
        let metrics = MetricsRegistry::default();

        // Build dataflow: per-customer totals and global top-K
        let top_k = opts.top_k;
        let prob_threshold = opts.prob_threshold;
        let target_customer = opts.target_customer;
        let metrics_for_dataflow = metrics.clone();
        worker.dataflow::<u64, _, _>(move |scope| {
            let orders = input.to_collection(scope);

            // Map typed orders to (customer_id, amount_cents)
            let spends = orders.map(|env| {
                let cust = env.payload.customer_id;
                let amt = env.payload.total_cents();
                (cust, amt)
            });

            // Per-customer running totals
            let totals = spends.reduce(|_cust, inputs, output| {
                let mut sum: i64 = 0;
                for (amt, cnt) in inputs.iter() {
                    sum += *amt * (*cnt as i64);
                }
                output.push((sum, 1));
            });

            // Global top-K customers by spend (base world)
            let topk = totals
                .map(|(cust, sum)| ((), (sum, cust)))
                .reduce(move |_unit, inputs, output| {
                    let mut vals: Vec<((i64, u64), isize)> =
                        inputs.iter().map(|(v, c)| (*v, *c)).collect();
                    // Sort by sum descending
                    vals.sort_by(|a, b| b.0 .0.cmp(&a.0 .0));
                    for i in 0..top_k.min(vals.len()) {
                        output.push((vals[i].0, 1));
                    }
                });

            topk.inspect(|x| info!(?x, "topk update"))
                .probe_with(&mut probe);

            // === Scenario overlays and scenario top-K ===
            // Predicted overlay deltas per (scenario, customer)
            let pred = pred_input
                .to_collection(scope)
                .map(|(sid, cust, delta)| (sid, cust, delta));

            // Sum predicted deltas per (scenario, customer)
            let pred_totals = pred
                .map(|(sid, cust, delta)| ((sid, cust), delta))
                .reduce(|_key, inputs, output| {
                    let mut sum: i64 = 0;
                    for (delta, cnt) in inputs.iter() {
                        sum += *delta * (*cnt as i64);
                    }
                    output.push((sum, 1));
                })
                .map(|((sid, cust), delta)| (sid, cust, delta));

            // Scenario weights (sid -> max prob)
            let scen_weights = scen_weight_input
                .to_collection(scope)
                .map(|(sid, prob)| (sid, prob))
                .reduce(|_sid, inputs, output| {
                    let maxp = inputs.iter().map(|(p, _)| *p).fold(0.0_f64, f64::max);
                    output.push((maxp, 1));
                });

            // Active scenarios from weights
            let scenarios_by_unit = scen_weights.map(|(sid, _p)| ((), sid));

            // Broadcast base top-K to each scenario
            let base_topk = topk.map(|(_unit, (sum, cust))| (cust, sum));
            let base_topk_by_unit = base_topk.map(|(cust, sum)| ((), (cust, sum)));
            let base_topk_broadcast = scenarios_by_unit
                .join(&base_topk_by_unit)
                .map(|(_unit, (sid, (cust, sum)))| (sid, (sum, cust)));

            // For predicted deltas, compute base_total + delta per scenario
            let base_totals_by_cust = totals.map(|(cust, sum)| (cust, sum));
            let pred_totals_by_cust = pred_totals.map(|(sid, cust, delta)| (cust, (sid, delta)));
            let scenario_changed = pred_totals_by_cust
                .join(&base_totals_by_cust)
                .map(|(cust, ((sid, delta), base_sum))| (sid, (base_sum + delta, cust)));

            // Candidates are broadcast base top-K plus changed customers per scenario
            let candidates = base_topk_broadcast.concat(&scenario_changed);

            // Compute top-K per scenario from candidates
            let scenario_topk = candidates
                .map(|(sid, pair)| (sid, pair))
                .reduce(move |_sid, inputs, output| {
                    let mut vals: Vec<((i64, u64), isize)> =
                        inputs.iter().map(|(v, c)| (*v, *c)).collect();
                    vals.sort_by(|a, b| b.0 .0.cmp(&a.0 .0));
                    for i in 0..top_k.min(vals.len()) {
                        output.push((vals[i].0, 1));
                    }
                });

            scenario_topk.inspect(|x| info!(?x, "scenario_topk update"));

            // Subscription: target customer enters top-K in any scenario with prob >= p_min
            let alerts = scenario_topk
                .map(|(sid, (sum, cust))| (sid, (cust, sum)))
                .filter(move |(_sid, (cust, _sum))| *cust == target_customer)
                .join(&scen_weights)
                .filter(move |(_sid, ((_cust, _sum), prob))| *prob >= prob_threshold)
                .map(|(sid, ((cust, sum), prob))| (sid, cust, sum, prob));

            let metrics_alerts = metrics_for_dataflow.clone();
            alerts
                .inspect(move |a| {
                    metrics_alerts.inc_scenario_alerts(1);
                    info!(?a, "ALERT: target customer in top-K within scenario");
                })
                .probe_with(&mut probe);
        });

        // Synthetic generator
        let mut epoch: u64 = 0;
        let customers = opts.customers;
        for batch in 0..opts.batches {
            let epoch_timer = EpochTimer::start();
            let completed_epoch = epoch;
            for i in 0..opts.batch_size {
                // Spread spend across customers with some skew
                let cust = (batch * 13 + i * 7) % customers;
                let base: i64 = 1000 + ((i % 10) as i64) * 250; // cents
                let bonus: i64 = if cust % 7 == 0 { 2000 } else { 0 };
                let amount = base + bonus;
                let order = OrderPlaced {
                    order_id: batch * 10_000 + i,
                    customer_id: cust,
                    lines: vec![OrderLine { sku_id: (i % 100) as u64, qty: 1, price_cents: amount }],
                    ts_ms: epoch * 1000,
                };
                let outcome = scenario_manager.expand_order(&order);
                metrics.inc_scenario_created(outcome.created.len() as u64);
                metrics.inc_scenario_retired(outcome.retired.len() as u64);
                let overlay_changes = outcome.overlays_added.len() + outcome.overlays_removed.len();
                if overlay_changes > 0 {
                    metrics.inc_predicted_events(overlay_changes as u64);
                }
                metrics.record_active_peak(scenario_manager.active_len() as u64);
                let env = EventEnvelope {
                    meta: EventMeta {
                        domain: "retail".to_string(),
                        kind: "OrderPlaced".to_string(),
                        epoch,
                        source: "synthetic".to_string(),
                        key: None,
                    },
                    payload: order,
                };
                input.insert(env);
                metrics.inc_base_events(1);

                for meta in &outcome.created {
                    scen_weight_input.insert((meta.id, meta.weight.0));
                }

                for delta in &outcome.overlays_added {
                    pred_input.insert((delta.scenario_id, delta.customer_id, delta.delta_cents));
                }

                for delta in &outcome.overlays_removed {
                    pred_input.remove((delta.scenario_id, delta.customer_id, delta.delta_cents));
                }

                for meta in &outcome.retired {
                    scen_weight_input.remove((meta.id, meta.weight.0));
                }
            }
            epoch += 1;
            input.advance_to(epoch);
            pred_input.advance_to(epoch);
            scen_weight_input.advance_to(epoch);
            input.flush();
            pred_input.flush();
            scen_weight_input.flush();
            // Drive the dataflow until this epoch completes
            while probe.less_than(input.time()) {
                worker.step();
            }
            let elapsed = epoch_timer.elapsed();
            let snapshot = metrics.snapshot();
            let json = snapshot.to_json_line("retail_epoch", Some(elapsed));
            info!(epoch = completed_epoch, %json, "epoch complete");
        }
        let final_snapshot = metrics.snapshot();
        let json = final_snapshot.to_json_line("retail_final", None);
        info!(%json, "final metrics summary");
    })
}

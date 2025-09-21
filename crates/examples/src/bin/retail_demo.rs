use anyhow::Result;
use tracing::info;
use tw_runtime::{init_tracing, start_runtime};

use differential_dataflow::input::InputSession;
use differential_dataflow::operators::reduce::Reduce;
use timely::dataflow::operators::probe::{Handle as ProbeHandle, Probe};
use timely::dataflow::operators::{Inspect, Map};

use std::sync::Arc;

use tw_core::retail::{OrderLine, OrderPlaced};
use tw_core::{EventEnvelope, EventMeta};
use tw_predictors::SpendGrowthPredictor;
use tw_scenarios::{BeamConfig, ScenarioManager};

fn main() -> Result<()> {
    init_tracing();
    info!("retail_demo starting");
    start_runtime(1, |_index, worker| {
        info!("retail_demo worker running");

        // Input for typed OrderPlaced events (base world)
        let mut input: InputSession<_, EventEnvelope<OrderPlaced>, isize> = InputSession::new();
        // Predicted overlay input: (scenario_id, customer_id, delta_amount_cents)
        let mut pred_input: InputSession<_, (u64, u64, i64), isize> = InputSession::new();
        // Scenario weights: (scenario_id, probability)
        let mut scen_weight_input: InputSession<_, (u64, f64), isize> = InputSession::new();
        let mut probe = ProbeHandle::new();

        let predictor = Arc::new(SpendGrowthPredictor::default());
        let mut scenario_manager = ScenarioManager::new(BeamConfig::default(), predictor);

        // Build dataflow: per-customer totals and global top-K
        const TOP_K: usize = 5;
        worker.dataflow::<u64, _, _>(|scope| {
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
                    for i in 0..TOP_K.min(vals.len()) {
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
                    for i in 0..TOP_K.min(vals.len()) {
                        output.push((vals[i].0, 1));
                    }
                });

            scenario_topk.inspect(|x| info!(?x, "scenario_topk update"));

            // Subscription: target customer enters top-K in any scenario with prob >= p_min
            let p_min = 0.2f64;
            let target_customer: u64 = 7;
            let alerts = scenario_topk
                .map(|(sid, (sum, cust))| (sid, (cust, sum)))
                .filter(move |(_sid, (cust, _sum))| *cust == target_customer)
                .join(&scen_weights)
                .filter(move |(_sid, ((_cust, _sum), prob))| *prob >= p_min)
                .map(|(sid, ((cust, sum), prob))| (sid, cust, sum, prob));

            alerts
                .inspect(|a| info!(?a, "ALERT: target customer in top-K within scenario"))
                .probe_with(&mut probe);
        });

        // Synthetic generator: 50 customers, deterministic pattern (typed events)
        let mut epoch: u64 = 0;
        let customers = 50u64;
        for batch in 0..10u64 {
            for i in 0..200u64 {
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
        }
    })
}

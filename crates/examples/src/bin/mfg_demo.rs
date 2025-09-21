use anyhow::Result;
use tracing::info;
use tw_runtime::{init_tracing, start_runtime};
use tw_runtime::metrics::{EpochTimer, MetricsRegistry};

use differential_dataflow::input::InputSession;
use differential_dataflow::operators::reduce::Reduce;
use timely::dataflow::operators::probe::{Handle as ProbeHandle, Probe};
use timely::dataflow::operators::{Inspect, Map};

use std::sync::Arc;

use tw_core::manufacturing::{ManufacturingEvent, OperationComplete, OperationStart};
use tw_core::{EventEnvelope, EventMeta};
use tw_predictors::QueueGrowthPredictor;
use tw_scenarios::manufacturing::{
    ManufacturingBeamConfig, ManufacturingScenarioDelta, ManufacturingScenarioManager,
};

#[derive(Debug, Clone)]
struct ActiveJob {
    job_id: u64,
    machine_id: u64,
    ready_epoch: u64,
}

fn main() -> Result<()> {
    init_tracing();
    info!("mfg_demo starting");
    start_runtime(1, |_index, worker| {
        info!("mfg_demo worker running");

        let mut input: InputSession<_, EventEnvelope<ManufacturingEvent>, isize> = InputSession::new();
        let mut pred_input: InputSession<_, (u64, u64, i64), isize> = InputSession::new();
        let mut scen_weight_input: InputSession<_, (u64, f64), isize> = InputSession::new();
        let mut probe = ProbeHandle::new();

        let predictor = Arc::new(QueueGrowthPredictor::default());
        let mut scenario_manager = ManufacturingScenarioManager::new(
            ManufacturingBeamConfig::default(),
            predictor,
        );
        let metrics = MetricsRegistry::default();

        const TOP_K: usize = 3;
        let metrics_for_dataflow = metrics.clone();
        worker.dataflow::<u64, _, _>(|scope| {
            let events = input.to_collection(scope);

            // Map events to machine backlog deltas
            let machine_deltas = events.flat_map(|env| match env.payload {
                ManufacturingEvent::OperationStart(ref op) => vec![(op.machine_id, 1i64)],
                ManufacturingEvent::OperationComplete(ref op) => vec![(op.machine_id, -1i64)],
                ManufacturingEvent::MachineStateChange(_) => Vec::new(),
            });

            let wip = machine_deltas
                .map(|(machine, delta)| (machine, delta))
                .reduce(|_machine, inputs, output| {
                    let mut sum: i64 = 0;
                    for (delta, count) in inputs.iter() {
                        sum += *delta * (*count as i64);
                    }
                    output.push((sum, 1));
                });

            let topk = wip
                .map(|(machine, sum)| ((), (sum, machine)))
                .reduce(move |_unit, inputs, output| {
                    let mut vals: Vec<((i64, u64), isize)> =
                        inputs.iter().map(|(val, cnt)| (*val, *cnt)).collect();
                    vals.sort_by(|a, b| b.0 .0.cmp(&a.0 .0));
                    for i in 0..TOP_K.min(vals.len()) {
                        output.push((vals[i].0, 1));
                    }
                });

            topk.inspect(|x| info!(?x, "base top machines"))
                .probe_with(&mut probe);

            // Scenario overlays
            let pred = pred_input
                .to_collection(scope)
                .map(|(sid, machine, delta)| (sid, machine, delta));

            let pred_totals = pred
                .map(|(sid, machine, delta)| ((sid, machine), delta))
                .reduce(|_key, inputs, output| {
                    let mut sum: i64 = 0;
                    for (delta, cnt) in inputs.iter() {
                        sum += *delta * (*cnt as i64);
                    }
                    output.push((sum, 1));
                })
                .map(|((sid, machine), delta)| (sid, machine, delta));

            let scen_weights = scen_weight_input
                .to_collection(scope)
                .map(|(sid, prob)| (sid, prob))
                .reduce(|_sid, inputs, output| {
                    let maxp = inputs.iter().map(|(p, _)| *p).fold(0.0_f64, f64::max);
                    output.push((maxp, 1));
                });

            let scenarios_by_unit = scen_weights.map(|(sid, _)| ((), sid));

            let base_topk_by_unit = topk.map(|(_unit, (sum, machine))| ((), (machine, sum)));
            let base_topk_broadcast = scenarios_by_unit
                .join(&base_topk_by_unit)
                .map(|(_unit, (sid, (machine, sum)))| (sid, (sum, machine)));

            let base_wip_by_machine = wip.map(|(machine, sum)| (machine, sum));
            let pred_totals_by_machine = pred_totals.map(|(sid, machine, delta)| (machine, (sid, delta)));
            let scenario_changes = pred_totals_by_machine
                .join(&base_wip_by_machine)
                .map(|(machine, ((sid, delta), base_sum))| (sid, (base_sum + delta, machine)));

            let candidates = base_topk_broadcast.concat(&scenario_changes);

            let scenario_topk = candidates
                .map(|(sid, pair)| (sid, pair))
                .reduce(move |_sid, inputs, output| {
                    let mut vals: Vec<((i64, u64), isize)> =
                        inputs.iter().map(|(val, cnt)| (*val, *cnt)).collect();
                    vals.sort_by(|a, b| b.0 .0.cmp(&a.0 .0));
                    for i in 0..TOP_K.min(vals.len()) {
                        output.push((vals[i].0, 1));
                    }
                });

            scenario_topk.inspect(|x| info!(?x, "scenario top machines"));

            let target_threshold = 6i64;
            let alerts = scenario_topk
                .map(|(sid, (sum, machine))| (sid, (machine, sum)))
                .join(&scen_weights)
                .filter(move |(_sid, ((machine, sum), prob))| *sum >= target_threshold && *prob >= 0.3)
                .map(|(sid, ((machine, sum), prob))| (sid, machine, sum, prob));

            let metrics_alerts = metrics_for_dataflow.clone();
            alerts
                .inspect(move |alert| {
                    metrics_alerts.inc_scenario_alerts(1);
                    info!(?alert, "ALERT: machine backlog risk");
                })
                .probe_with(&mut probe);
        });

        // Synthetic generator
        let mut epoch: u64 = 0;
        let machines = 8u64;
        let mut job_counter: u64 = 0;
        let mut active_jobs: Vec<ActiveJob> = Vec::new();

        for batch in 0..12u64 {
            let epoch_timer = EpochTimer::start();
            let completed_epoch = epoch;
            for i in 0..120u64 {
                job_counter += 1;
                let machine = (batch * 5 + i * 11) % machines;
                let duration_ms = 3_000 + (machine * 250) + ((i % 5) as u64) * 500;
                let op = OperationStart {
                    job_id: job_counter,
                    operation_id: (i % 4) as u32,
                    machine_id: machine,
                    ts_ms: epoch * 1_000,
                    expected_duration_ms: duration_ms,
                };

                let outcome = scenario_manager.expand_operation(&op);
                metrics.inc_scenario_created(outcome.created.len() as u64);
                metrics.inc_scenario_retired(outcome.retired.len() as u64);
                let overlay_changes = outcome.overlays_added.len() + outcome.overlays_removed.len();
                if overlay_changes > 0 {
                    metrics.inc_predicted_events(overlay_changes as u64);
                }
                metrics.record_active_peak(scenario_manager.active_len() as u64);
                for meta in &outcome.created {
                    scen_weight_input.insert((meta.id, meta.weight.0));
                }
                for ManufacturingScenarioDelta { scenario_id, machine_id, delta_wip } in
                    &outcome.overlays_added
                {
                    pred_input.insert((*scenario_id, *machine_id, *delta_wip));
                }
                for ManufacturingScenarioDelta { scenario_id, machine_id, delta_wip } in
                    &outcome.overlays_removed
                {
                    pred_input.remove((*scenario_id, *machine_id, *delta_wip));
                }
                for meta in &outcome.retired {
                    scen_weight_input.remove((meta.id, meta.weight.0));
                }

                let env = EventEnvelope {
                    meta: EventMeta {
                        domain: "manufacturing".to_string(),
                        kind: "OperationStart".to_string(),
                        epoch,
                        source: "synthetic".to_string(),
                        key: Some(format!("job:{}", op.job_id)),
                    },
                    payload: ManufacturingEvent::OperationStart(op.clone()),
                };
                input.insert(env);
                metrics.inc_base_events(1);

                let ready_epoch = epoch + 1 + (machine % 3);
                active_jobs.push(ActiveJob {
                    job_id: op.job_id,
                    machine_id: op.machine_id,
                    ready_epoch,
                });
            }

            // Emit completions that are ready this epoch
            let mut completed: Vec<ActiveJob> = Vec::new();
            active_jobs.retain(|job| {
                if job.ready_epoch <= epoch {
                    completed.push(job.clone());
                    false
                } else {
                    true
                }
            });

            for job in completed {
                let complete = OperationComplete {
                    job_id: job.job_id,
                    operation_id: 0,
                    machine_id: job.machine_id,
                    ts_ms: epoch * 1_000 + 500,
                };
                let env = EventEnvelope {
                    meta: EventMeta {
                        domain: "manufacturing".to_string(),
                        kind: "OperationComplete".to_string(),
                        epoch,
                        source: "synthetic".to_string(),
                        key: Some(format!("job:{}", job.job_id)),
                    },
                    payload: ManufacturingEvent::OperationComplete(complete),
                };
                input.insert(env);
                metrics.inc_base_events(1);
            }

            epoch += 1;
            input.advance_to(epoch);
            pred_input.advance_to(epoch);
            scen_weight_input.advance_to(epoch);
            input.flush();
            pred_input.flush();
            scen_weight_input.flush();

            while probe.less_than(input.time()) {
                worker.step();
            }
            let elapsed = epoch_timer.elapsed();
            let snapshot = metrics.snapshot();
            info!(
                epoch = completed_epoch,
                duration_ms = elapsed.as_millis(),
                base_events = snapshot.base_events,
                predicted_events = snapshot.predicted_events,
                scenarios_created = snapshot.scenario_created,
                scenarios_retired = snapshot.scenario_retired,
                alerts = snapshot.scenario_alerts,
                active_peak = snapshot.scenario_active_peak,
                "epoch complete"
            );
        }
        let final_snapshot = metrics.snapshot();
        info!(?final_snapshot, "final metrics summary");
    })
}

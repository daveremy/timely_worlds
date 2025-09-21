use anyhow::Result;
use tracing::info;
use tw_runtime::{init_tracing, start_runtime};

use differential_dataflow::input::InputSession;
use differential_dataflow::operators::reduce::Reduce;
use timely::dataflow::operators::probe::{Handle as ProbeHandle, Probe};

fn main() -> Result<()> {
    init_tracing();
    info!("retail_demo starting");
    start_runtime(1, |_index, worker| {
        info!("retail_demo worker running");

        // Input for (customer_id, amount_cents) tuples
        let mut input: InputSession<_, (u64, i64), isize> = InputSession::new();
        let mut probe = ProbeHandle::new();

        // Build dataflow: per-customer totals and global top-K
        const TOP_K: usize = 5;
        worker.dataflow::<u64, _, _>(|scope| {
            let orders = input.to_collection(scope);

            // Per-customer running totals
            let totals = orders.reduce(|_cust, inputs, output| {
                let mut sum: i64 = 0;
                for (amt, cnt) in inputs.iter() {
                    sum += *amt * (*cnt as i64);
                }
                output.push((sum, 1));
            });

            // Global top-K customers by spend
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
        });

        // Synthetic generator: 50 customers, deterministic pattern
        let mut epoch: u64 = 0;
        let customers = 50u64;
        for batch in 0..10u64 {
            for i in 0..200u64 {
                // Spread spend across customers with some skew
                let cust = (batch * 13 + i * 7) % customers;
                let base: i64 = 1000 + ((i % 10) as i64) * 250; // cents
                let bonus: i64 = if cust % 7 == 0 { 2000 } else { 0 };
                let amount = base + bonus;
                input.insert((cust, amount));
            }
            epoch += 1;
            input.advance_to(epoch);
            input.flush();
            // Drive the dataflow until this epoch completes
            while probe.less_than(input.time()) {
                worker.step();
            }
        }
    })
}

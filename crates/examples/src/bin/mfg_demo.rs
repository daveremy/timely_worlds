use anyhow::Result;
use tracing::info;
use tw_runtime::{init_tracing, start_runtime};

fn main() -> Result<()> {
    init_tracing();
    info!("mfg_demo starting");
    start_runtime(1, |_index, _worker| {
        // TODO: wire ingest → base views → noop predictor → scenario manager → subscriptions
        info!("mfg_demo worker running");
    })
}


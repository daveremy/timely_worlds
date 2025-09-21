//! Runtime bootstrap for Timely/Differential dataflows.

use anyhow::Result;
use tracing::{info, Level};

pub mod metrics;

pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_max_level(Level::INFO)
        .try_init();
}

/// Start a single-process timely runtime and execute the provided closure once per worker.
pub fn start_runtime<F>(workers: usize, f: F) -> Result<()>
where
    F: Fn(usize, &mut timely::worker::Worker<timely::communication::allocator::Generic>) + Clone + Send + 'static,
{
    info!(%workers, "starting timely runtime");
    timely::execute_from_args(std::env::args(), move |worker| {
        let index = worker.index();
        f(index, worker);
    })?;
    Ok(())
}

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone, Default)]
pub struct MetricsRegistry {
    inner: Arc<MetricsInner>,
}

#[derive(Default)]
struct MetricsInner {
    base_events: AtomicU64,
    predicted_events: AtomicU64,
    scenario_alerts: AtomicU64,
    scenario_created: AtomicU64,
    scenario_retired: AtomicU64,
    scenario_active_peak: AtomicU64,
}

impl MetricsRegistry {
    pub fn inc_base_events(&self, delta: u64) {
        self.inner.base_events.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn inc_predicted_events(&self, delta: u64) {
        self.inner.predicted_events.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn inc_scenario_alerts(&self, delta: u64) {
        self.inner.scenario_alerts.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn inc_scenario_created(&self, delta: u64) {
        self.inner.scenario_created.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn inc_scenario_retired(&self, delta: u64) {
        self.inner.scenario_retired.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn record_active_peak(&self, active: u64) {
        self.inner
            .scenario_active_peak
            .fetch_max(active, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            base_events: self.inner.base_events.load(Ordering::Relaxed),
            predicted_events: self.inner.predicted_events.load(Ordering::Relaxed),
            scenario_alerts: self.inner.scenario_alerts.load(Ordering::Relaxed),
            scenario_created: self.inner.scenario_created.load(Ordering::Relaxed),
            scenario_retired: self.inner.scenario_retired.load(Ordering::Relaxed),
            scenario_active_peak: self.inner.scenario_active_peak.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MetricsSnapshot {
    pub base_events: u64,
    pub predicted_events: u64,
    pub scenario_alerts: u64,
    pub scenario_created: u64,
    pub scenario_retired: u64,
    pub scenario_active_peak: u64,
}

pub struct EpochTimer {
    start: Instant,
}

impl EpochTimer {
    pub fn start() -> Self {
        Self { start: Instant::now() }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}


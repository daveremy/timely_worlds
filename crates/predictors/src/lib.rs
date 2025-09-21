//! Predictor trait and baseline stubs.

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use tw-core::{Depth, EventEnvelope, Predicted, Prob, ScenarioId};

use tw_core::retail::OrderPlaced;

/// A predictor consumes view changes and produces candidate future events for expansion.
pub trait Predictor<T>
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    fn predict(
        &self,
        parent: ScenarioId,
        next_depth: Depth,
        context: &EventEnvelope<T>,
    ) -> Result<Vec<Predicted<T>>>;
}

/// A simple pass-through predictor stub for initial wiring.
pub struct NoopPredictor;

impl<T> Predictor<T> for NoopPredictor
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    fn predict(
        &self,
        parent: ScenarioId,
        next_depth: Depth,
        context: &EventEnvelope<T>,
    ) -> Result<Vec<Predicted<T>>> {
        let child = parent.wrapping_add(1);
        let pred = Predicted {
            parent_scenario: parent,
            child_scenario: child,
            depth: next_depth,
            prob: Prob(1.0),
            event: context.clone(),
        };
        Ok(vec![pred])
    }
}

pub trait SpendDeltaPredictor: Send + Sync + 'static {
    fn predict_delta(&self, order: &OrderPlaced) -> i64;
}

pub struct SpendGrowthPredictor {
    pub uplift_ratio: f64,
    pub min_delta_cents: i64,
}

impl Default for SpendGrowthPredictor {
    fn default() -> Self {
        Self { uplift_ratio: 0.3, min_delta_cents: 3_000 }
    }
}

impl SpendDeltaPredictor for SpendGrowthPredictor {
    fn predict_delta(&self, order: &OrderPlaced) -> i64 {
        let base = order.total_cents().max(1);
        let uplift = ((base as f64) * self.uplift_ratio).round() as i64;
        std::cmp::max(uplift, self.min_delta_cents)
    }
}

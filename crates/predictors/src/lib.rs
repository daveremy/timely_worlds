//! Predictor trait and baseline stubs.

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use tw-core::{Depth, EventEnvelope, Predicted, Prob, ScenarioId};

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


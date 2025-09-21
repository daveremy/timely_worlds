//! Scenario overlays and a simple beam/pruning manager stub.

use serde::{Deserialize, Serialize};
use tw-core::{Depth, Prob, ScenarioId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioMeta {
    pub id: ScenarioId,
    pub parent: Option<ScenarioId>,
    pub depth: Depth,
    pub weight: Prob,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamConfig {
    pub max_depth: Depth,
    pub beam_width: usize,
    pub min_prob: f64,
}

impl Default for BeamConfig {
    fn default() -> Self {
        Self { max_depth: 5, beam_width: 32, min_prob: 0.1 }
    }
}

pub struct ScenarioManager {
    pub cfg: BeamConfig,
}

impl ScenarioManager {
    pub fn new(cfg: BeamConfig) -> Self { Self { cfg } }
}


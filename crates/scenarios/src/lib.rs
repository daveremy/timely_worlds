use serde::{Deserialize, Serialize};
use tw_core::{Depth, Prob, ScenarioId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioMeta {
    pub id: ScenarioId,
    pub parent: Option<ScenarioId>,
    pub depth: Depth,
    pub weight: Prob,
}

pub mod retail;
pub mod manufacturing;

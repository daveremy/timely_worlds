use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use tw_core::manufacturing::OperationStart;
use tw_core::Prob;
use tw_predictors::MachineBacklogPredictor;

use crate::ScenarioMeta;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingScenarioDelta {
    pub scenario_id: u64,
    pub machine_id: u64,
    pub delta_wip: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingBeamConfig {
    pub max_depth: u32,
    pub beam_width: usize,
    pub min_prob: f64,
    pub branch_prob: f64,
    pub delta_multiplier: f64,
    pub min_delta_units: i64,
}

impl Default for ManufacturingBeamConfig {
    fn default() -> Self {
        Self {
            max_depth: 4,
            beam_width: 16,
            min_prob: 0.1,
            branch_prob: 0.45,
            delta_multiplier: 0.5,
            min_delta_units: 2,
        }
    }
}

#[derive(Debug, Default)]
pub struct ManufacturingExpansionOutcome {
    pub created: Vec<ScenarioMeta>,
    pub retired: Vec<ScenarioMeta>,
    pub overlays_added: Vec<ManufacturingScenarioDelta>,
    pub overlays_removed: Vec<ManufacturingScenarioDelta>,
}

pub struct ManufacturingScenarioManager {
    cfg: ManufacturingBeamConfig,
    predictor: Arc<dyn MachineBacklogPredictor>,
    next_id: u64,
    active: Vec<ScenarioMeta>,
    overlays: HashMap<u64, ManufacturingScenarioDelta>,
}

impl ManufacturingScenarioManager {
    pub fn new(cfg: ManufacturingBeamConfig, predictor: Arc<dyn MachineBacklogPredictor>) -> Self {
        Self {
            cfg,
            predictor,
            next_id: 1,
            active: Vec::new(),
            overlays: HashMap::new(),
        }
    }

    pub fn expand_operation(&mut self, op: &OperationStart) -> ManufacturingExpansionOutcome {
        let mut outcome = ManufacturingExpansionOutcome::default();

        let mut survivors = Vec::new();
        let mut retired = Vec::new();

        for meta in self.active.drain(..) {
            if meta.weight.0 < self.cfg.min_prob || meta.depth >= self.cfg.max_depth {
                retired.push(meta);
            } else {
                survivors.push(meta);
            }
        }

        let mut candidates = survivors.clone();

        let parents_iter = std::iter::once(ScenarioMeta {
            id: 0,
            parent: None,
            depth: 0,
            weight: Prob(1.0),
        })
        .chain(survivors.into_iter());

        let predicted_delta = self.predict_delta(op);

        for parent in parents_iter {
            if parent.depth >= self.cfg.max_depth {
                continue;
            }
            let parent_weight = if parent.id == 0 { 1.0 } else { parent.weight.0 };
            let child_weight = parent_weight * self.cfg.branch_prob;
            if child_weight < self.cfg.min_prob {
                continue;
            }
            let child_id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1);

            let meta = ScenarioMeta {
                id: child_id,
                parent: if parent.id == 0 { None } else { Some(parent.id) },
                depth: parent.depth + 1,
                weight: Prob(child_weight),
            };

            let delta = ManufacturingScenarioDelta {
                scenario_id: child_id,
                machine_id: op.machine_id,
                delta_wip: predicted_delta,
            };

            self.overlays.insert(child_id, delta.clone());

            outcome.created.push(meta.clone());
            outcome.overlays_added.push(delta);
            candidates.push(meta);
        }

        let mut seen: HashSet<u64> = HashSet::new();
        let mut sorted = candidates;
        sorted.sort_by(|a, b| b.weight.0.partial_cmp(&a.weight.0).unwrap_or(Ordering::Equal));

        let mut retained = Vec::new();
        for meta in sorted {
            if seen.contains(&meta.id) {
                continue;
            }
            seen.insert(meta.id);
            if retained.len() < self.cfg.beam_width {
                retained.push(meta);
            } else {
                retired.push(meta);
            }
        }

        for meta in retired.iter() {
            if let Some(delta) = self.overlays.remove(&meta.id) {
                outcome.overlays_removed.push(delta);
            }
        }

        outcome.retired.extend(retired);
        self.active = retained;

        outcome
    }

    pub fn active_weights(&self) -> Vec<(u64, f64)> {
        self.active
            .iter()
            .map(|meta| (meta.id, meta.weight.0))
            .collect()
    }

    pub fn active_len(&self) -> usize {
        self.active.len()
    }

    fn predict_delta(&self, op: &OperationStart) -> i64 {
        let mut delta = self.predictor.predict_backlog(op);
        if (self.cfg.delta_multiplier - 1.0).abs() > f64::EPSILON {
            delta = ((delta as f64) * self.cfg.delta_multiplier).round() as i64;
        }
        if delta < self.cfg.min_delta_units {
            self.cfg.min_delta_units
        } else {
            delta
        }
    }
}

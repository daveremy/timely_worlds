//! Core types and traits for Timely Worlds.

use serde::{Deserialize, Serialize};

pub type Epoch = u64;
pub type Depth = u32;
pub type ScenarioId = u64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Prob(pub f64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Diff {
    Int(i64),
    Float(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventMeta {
    pub domain: String,
    pub kind: String,
    pub epoch: Epoch,
    pub source: String,
    pub key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope<T> {
    pub meta: EventMeta,
    pub payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Predicted<T> {
    pub parent_scenario: ScenarioId,
    pub child_scenario: ScenarioId,
    pub depth: Depth,
    pub prob: Prob,
    pub event: EventEnvelope<T>,
}

#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub mod retail;
pub mod manufacturing;

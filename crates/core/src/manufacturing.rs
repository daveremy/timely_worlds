use serde::{Deserialize, Serialize};

pub type JobId = u64;
pub type OperationId = u32;
pub type MachineId = u64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationStart {
    pub job_id: JobId,
    pub operation_id: OperationId,
    pub machine_id: MachineId,
    pub ts_ms: u64,
    pub expected_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationComplete {
    pub job_id: JobId,
    pub operation_id: OperationId,
    pub machine_id: MachineId,
    pub ts_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MachineStatus {
    Running,
    Idle,
    Maintenance,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineStateChange {
    pub machine_id: MachineId,
    pub status: MachineStatus,
    pub ts_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ManufacturingEvent {
    OperationStart(OperationStart),
    OperationComplete(OperationComplete),
    MachineStateChange(MachineStateChange),
}

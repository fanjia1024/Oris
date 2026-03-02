//! Proposal-only runtime contract for external agents.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgentCapabilityLevel {
    A0,
    A1,
    A2,
    A3,
    A4,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProposalTarget {
    WorkspaceRoot,
    Paths(Vec<String>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MutationProposal {
    pub intent: String,
    pub files: Vec<String>,
    pub expected_effect: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionFeedback {
    pub accepted: bool,
    pub asset_state: Option<String>,
    pub summary: String,
}

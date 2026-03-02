//! Policy-only governor contracts for Oris EvoKernel.

use serde::{Deserialize, Serialize};

use oris_evolution::{AssetState, BlastRadius, CandidateSource};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernorConfig {
    pub promote_after_successes: u64,
    pub max_files_changed: usize,
    pub max_lines_changed: usize,
    pub cooldown_secs: u64,
    pub revoke_after_replay_failures: u64,
}

impl Default for GovernorConfig {
    fn default() -> Self {
        Self {
            promote_after_successes: 3,
            max_files_changed: 5,
            max_lines_changed: 300,
            cooldown_secs: 30 * 60,
            revoke_after_replay_failures: 2,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoolingWindow {
    pub cooldown_secs: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RevocationReason {
    ReplayRegression,
    ValidationFailure,
    Manual(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernorInput {
    pub candidate_source: CandidateSource,
    pub success_count: u64,
    pub blast_radius: BlastRadius,
    pub replay_failures: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernorDecision {
    pub target_state: AssetState,
    pub reason: String,
    pub cooling_window: Option<CoolingWindow>,
    pub revocation_reason: Option<RevocationReason>,
}

pub trait Governor: Send + Sync {
    fn evaluate(&self, input: GovernorInput) -> GovernorDecision;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultGovernor {
    config: GovernorConfig,
}

impl DefaultGovernor {
    pub fn new(config: GovernorConfig) -> Self {
        Self { config }
    }
}

impl Governor for DefaultGovernor {
    fn evaluate(&self, input: GovernorInput) -> GovernorDecision {
        if input.replay_failures >= self.config.revoke_after_replay_failures {
            return GovernorDecision {
                target_state: AssetState::Revoked,
                reason: "replay validation failures exceeded threshold".into(),
                cooling_window: None,
                revocation_reason: Some(RevocationReason::ReplayRegression),
            };
        }

        if input.blast_radius.files_changed > self.config.max_files_changed
            || input.blast_radius.lines_changed > self.config.max_lines_changed
        {
            return GovernorDecision {
                target_state: AssetState::Candidate,
                reason: "blast radius exceeds promotion threshold".into(),
                cooling_window: None,
                revocation_reason: None,
            };
        }

        if input.success_count >= self.config.promote_after_successes {
            return GovernorDecision {
                target_state: AssetState::Promoted,
                reason: "success threshold reached".into(),
                cooling_window: Some(CoolingWindow {
                    cooldown_secs: self.config.cooldown_secs,
                }),
                revocation_reason: None,
            };
        }

        GovernorDecision {
            target_state: AssetState::Candidate,
            reason: "collecting more successful executions".into(),
            cooling_window: None,
            revocation_reason: None,
        }
    }
}

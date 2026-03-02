//! Non-financial EVU accounting for local publish and validation incentives.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EvuAccount {
    pub node_id: String,
    pub balance: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ReputationRecord {
    pub node_id: String,
    pub publish_success_rate: f32,
    pub validator_accuracy: f32,
    pub reuse_impact: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StakePolicy {
    pub publish_cost: i64,
}

impl Default for StakePolicy {
    fn default() -> Self {
        Self { publish_cost: 1 }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationSettlement {
    pub publisher_delta: i64,
    pub validator_delta: i64,
    pub reason: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EvuLedger {
    pub accounts: Vec<EvuAccount>,
    pub reputations: Vec<ReputationRecord>,
}

impl EvuLedger {
    pub fn can_publish(&self, node_id: &str, policy: &StakePolicy) -> bool {
        self.accounts
            .iter()
            .find(|account| account.node_id == node_id)
            .map(|account| account.balance >= policy.publish_cost)
            .unwrap_or(false)
    }
}

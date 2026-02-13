//! Policy: governance layer (authorize, retry, budget).
//!
//! Must exist even as a minimal implementation so Oris is not a "run any tool" demo.

use crate::kernel::action::Action;
use crate::kernel::identity::RunId;
use crate::kernel::KernelError;

/// Context passed to policy (e.g. caller identity, run metadata).
#[derive(Clone, Debug, Default)]
pub struct PolicyCtx {
    pub user_id: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Decision after an action failure (retry, backoff, or fail).
#[derive(Clone, Debug)]
pub enum RetryDecision {
    Retry,
    RetryAfterMs(u64),
    Fail,
}

/// Optional budget rules (cost, token limits, etc.).
#[derive(Clone, Debug, Default)]
pub struct BudgetRules {
    pub max_tool_calls: Option<u64>,
    pub max_llm_tokens: Option<u64>,
}

/// Policy: authorize actions, decide retries, optional budget.
pub trait Policy: Send + Sync {
    /// Whether the action is allowed for this run and context.
    fn authorize(
        &self,
        run_id: &RunId,
        action: &Action,
        ctx: &PolicyCtx,
    ) -> Result<(), KernelError>;

    /// Whether to retry after an error (and optionally after a delay).
    fn retry_strategy(
        &self,
        err: &dyn std::fmt::Display,
        _action: &Action,
    ) -> RetryDecision {
        let _ = err;
        RetryDecision::Fail
    }

    /// Optional budget; default is no limits.
    fn budget(&self) -> BudgetRules {
        BudgetRules::default()
    }
}

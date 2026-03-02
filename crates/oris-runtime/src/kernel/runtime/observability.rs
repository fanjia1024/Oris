//! Backpressure and kernel observability types for scheduling and telemetry.
//!
//! **RejectionReason**: why a dispatch or API request was rejected (e.g. tenant limit),
//! for safe backpressure and clear API responses.
//!
//! **KernelObservability**: placeholder structure for kernel telemetry (reasoning timeline,
//! lease graph, replay cost, interrupt latency). Implementations can fill these for
//! metrics and tracing; no built-in collection in this crate.

use serde::{Deserialize, Serialize};

/// Reason for rejecting a dispatch or API request (e.g. rate limit, tenant cap).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RejectionReason {
    /// Tenant-level limit exceeded; optional description of the limit.
    TenantLimit(Option<String>),
    /// Worker or capacity limit.
    CapacityLimit(Option<String>),
    /// Other rejections (policy, invalid request, etc.).
    Other(String),
}

impl RejectionReason {
    pub fn tenant_limit(description: impl Into<String>) -> Self {
        RejectionReason::TenantLimit(Some(description.into()))
    }

    pub fn capacity_limit(description: impl Into<String>) -> Self {
        RejectionReason::CapacityLimit(Some(description.into()))
    }
}

/// Placeholder structure for kernel observability / telemetry.
///
/// Fields can be populated by the runtime for metrics and tracing.
/// No built-in collection or export; types exist for API stability.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KernelObservability {
    /// Optional reasoning or decision timeline (e.g. scheduler steps).
    pub reasoning_timeline: Option<Vec<String>>,
    /// Optional lease/ownership snapshot (e.g. attempt → worker).
    pub lease_graph: Option<Vec<(String, String)>>,
    /// Optional replay cost hint (e.g. event count or duration).
    pub replay_cost: Option<u64>,
    /// Optional interrupt handling latency (e.g. ms).
    pub interrupt_latency_ms: Option<u64>,
}

impl KernelObservability {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_reasoning_timeline(mut self, entries: Vec<String>) -> Self {
        self.reasoning_timeline = Some(entries);
        self
    }

    pub fn with_lease_graph(mut self, edges: Vec<(String, String)>) -> Self {
        self.lease_graph = Some(edges);
        self
    }

    pub fn with_replay_cost(mut self, cost: u64) -> Self {
        self.replay_cost = Some(cost);
        self
    }

    pub fn with_interrupt_latency_ms(mut self, ms: u64) -> Self {
        self.interrupt_latency_ms = Some(ms);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejection_reason_tenant_limit() {
        let r = RejectionReason::tenant_limit("max concurrent runs");
        assert!(
            matches!(r, RejectionReason::TenantLimit(Some(ref s)) if s == "max concurrent runs")
        );
    }

    #[test]
    fn kernel_observability_builder() {
        let o = KernelObservability::new()
            .with_reasoning_timeline(vec!["step1".into()])
            .with_replay_cost(42)
            .with_interrupt_latency_ms(10);
        assert_eq!(o.reasoning_timeline, Some(vec!["step1".into()]));
        assert_eq!(o.replay_cost, Some(42));
        assert_eq!(o.interrupt_latency_ms, Some(10));
    }
}

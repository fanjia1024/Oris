//! Crash recovery pipeline: lease expiry → checkpoint reload → replay → re-dispatch.
//!
//! When a worker crashes, the lease expires (or is detected stale). This module
//! defines the pipeline steps and a minimal context type so the kernel can
//! drive: expire (via [crate::kernel::runtime::LeaseManager::tick] /
//! [crate::kernel::runtime::RuntimeRepository::expire_leases_and_requeue]),
//! then checkpoint reload and replay for the affected attempt, then allow
//! re-dispatch to a new worker.

use crate::kernel::identity::RunId;

/// Steps of the zero-data-loss failure recovery pipeline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStep {
    /// Lease has expired or worker is gone; attempt is requeued.
    LeaseExpired,
    /// Checkpoint for the run is (to be) reloaded.
    CheckpointReload,
    /// Execution is (to be) replayed from checkpoint.
    Replay,
    /// Attempt is ready to be dispatched to a new worker.
    ReadyForDispatch,
}

impl RecoveryStep {
    /// Ordered sequence of steps for the pipeline.
    pub fn pipeline() -> &'static [RecoveryStep] {
        &[
            RecoveryStep::LeaseExpired,
            RecoveryStep::CheckpointReload,
            RecoveryStep::Replay,
            RecoveryStep::ReadyForDispatch,
        ]
    }
}

/// Context produced after checkpoint reload for replay. Pass `run_id` to replay logic.
#[derive(Clone, Debug, Default)]
pub struct RecoveryContext {
    pub attempt_id: String,
    pub run_id: RunId,
}

impl RecoveryContext {
    pub fn new(attempt_id: impl Into<String>, run_id: impl Into<RunId>) -> Self {
        Self {
            attempt_id: attempt_id.into(),
            run_id: run_id.into(),
        }
    }
}

/// Crash recovery pipeline: documents and drives the steps from lease expiry to re-dispatch.
///
/// Use with [RecoveryStep::pipeline]. After [expire_leases_and_requeue](crate::kernel::runtime::RuntimeRepository::expire_leases_and_requeue),
/// the caller loads checkpoint for each requeued attempt, builds a [RecoveryContext] with
/// attempt_id and run_id, then replays; the attempt becomes dispatchable again.
#[derive(Clone, Debug)]
pub struct CrashRecoveryPipeline {
    pub attempt_id: String,
    pub run_id: RunId,
}

impl CrashRecoveryPipeline {
    pub fn new(attempt_id: impl Into<String>, run_id: impl Into<RunId>) -> Self {
        Self {
            attempt_id: attempt_id.into(),
            run_id: run_id.into(),
        }
    }

    /// Returns the ordered recovery steps.
    pub fn steps() -> &'static [RecoveryStep] {
        RecoveryStep::pipeline()
    }

    /// Builds a context for the replay phase (checkpoint reload done; run_id available for replay).
    pub fn replay_context(&self) -> RecoveryContext {
        RecoveryContext::new(&self.attempt_id, &self.run_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_step_pipeline_order() {
        let steps = RecoveryStep::pipeline();
        assert_eq!(steps.len(), 4);
        assert_eq!(steps[0], RecoveryStep::LeaseExpired);
        assert_eq!(steps[1], RecoveryStep::CheckpointReload);
        assert_eq!(steps[2], RecoveryStep::Replay);
        assert_eq!(steps[3], RecoveryStep::ReadyForDispatch);
    }

    #[test]
    fn crash_recovery_pipeline_replay_context() {
        let p = CrashRecoveryPipeline::new("attempt-1", "run-1");
        let ctx = p.replay_context();
        assert_eq!(ctx.attempt_id, "attempt-1");
        assert_eq!(ctx.run_id, "run-1");
    }
}

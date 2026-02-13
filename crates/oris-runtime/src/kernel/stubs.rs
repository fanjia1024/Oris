//! Stub implementations for building a Kernel (e.g. for replay-only tests).
//! Replay does not call ActionExecutor or StepFn; policy is used only when running.

use crate::kernel::action::{Action, ActionExecutor, ActionResult};
use crate::kernel::identity::RunId;
use crate::kernel::policy::{Policy, PolicyCtx};
use crate::kernel::state::KernelState;
use crate::kernel::step::{Next, StepFn};
use crate::kernel::KernelError;

/// Action executor that never runs (for replay-only Kernel).
pub struct NoopActionExecutor;

impl ActionExecutor for NoopActionExecutor {
    fn execute(&self, _run_id: &RunId, _action: &Action) -> Result<ActionResult, KernelError> {
        Err(KernelError::Driver(
            "action execution not used in replay".into(),
        ))
    }
}

/// Step function that always completes immediately (for replay-only Kernel).
pub struct NoopStepFn;

impl<S: KernelState> StepFn<S> for NoopStepFn {
    fn next(&self, _state: &S) -> Result<Next, KernelError> {
        Ok(Next::Complete)
    }
}

/// Policy that allows all actions (for replay-only Kernel).
pub struct AllowAllPolicy;

impl Policy for AllowAllPolicy {
    fn authorize(
        &self,
        _run_id: &RunId,
        _action: &Action,
        _ctx: &PolicyCtx,
    ) -> Result<(), KernelError> {
        Ok(())
    }
}

//! Step function: given state, decide next (emit events, do action, interrupt, or complete).
//!
//! Graph/Agent compile down to a StepFn.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::kernel::action::Action;
use crate::kernel::event::Event;
use crate::kernel::state::KernelState;
use crate::kernel::KernelError;

/// Information attached to an interrupt (e.g. for human-in-the-loop).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InterruptInfo {
    pub value: Value,
}

/// What to do next after a step.
#[derive(Clone, Debug)]
pub enum Next {
    /// Emit internal events only (no external action).
    Emit(Vec<Event>),
    /// Request one external action (policy + executor; result becomes events).
    Do(Action),
    /// Pause for interrupt (e.g. human approval).
    Interrupt(InterruptInfo),
    /// Run is complete.
    Complete,
}

/// Step function: given current state, returns the next action (emit / do / interrupt / complete).
/// Graph and Agent are compiled to this interface.
pub trait StepFn<S: KernelState>: Send + Sync {
    fn next(&self, state: &S) -> Result<Next, KernelError>;
}

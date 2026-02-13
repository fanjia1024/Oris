//! Execution trace events for observability and audit.
//!
//! When using `invoke_with_config_interrupt`, the returned `InvokeResult` includes
//! a `trace` of events: steps completed, interrupts reached, and resume values received.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single event in an execution trace.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TraceEvent {
    /// A node completed successfully.
    StepCompleted { node: String },
    /// Execution hit an interrupt (human-in-the-loop or policy).
    InterruptReached { value: Value },
    /// Execution was resumed with a value (after an interrupt).
    ResumeReceived { value: Value },
}

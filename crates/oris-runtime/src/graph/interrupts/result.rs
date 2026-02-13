use serde_json::Value;

use crate::graph::state::State;
use crate::graph::trace::TraceEvent;

use super::types::Interrupt;

/// Result of graph invocation that may contain interrupt information and execution trace.
///
/// When an interrupt occurs, the result includes the `__interrupt__` field
/// containing information about the interrupt. The `trace` field exposes
/// step, interrupt, and resume events for debugging and audit.
#[derive(Debug, Clone)]
pub struct InvokeResult<S: State> {
    /// The final state (or state at interrupt point)
    pub state: S,
    /// Interrupt information (if an interrupt occurred)
    pub interrupt: Option<Vec<Interrupt>>,
    /// Execution trace: steps completed, interrupts reached, resume values received.
    pub trace: Vec<TraceEvent>,
}

impl<S: State> InvokeResult<S> {
    /// Create a new InvokeResult with state only (empty trace).
    pub fn new(state: S) -> Self {
        Self {
            state,
            interrupt: None,
            trace: Vec::new(),
        }
    }

    /// Create a new InvokeResult with interrupt information (empty trace).
    pub fn with_interrupt(state: S, interrupt: Vec<Interrupt>) -> Self {
        Self {
            state,
            interrupt: Some(interrupt),
            trace: Vec::new(),
        }
    }

    /// Create with state and trace (no interrupt).
    pub fn new_with_trace(state: S, trace: Vec<TraceEvent>) -> Self {
        Self {
            state,
            interrupt: None,
            trace,
        }
    }

    /// Create with state, interrupt, and trace.
    pub fn with_interrupt_and_trace(
        state: S,
        interrupt: Vec<Interrupt>,
        trace: Vec<TraceEvent>,
    ) -> Self {
        Self {
            state,
            interrupt: Some(interrupt),
            trace,
        }
    }

    /// Convert to JSON format (similar to Python's result format)
    ///
    /// The result will have the state as the main object, with
    /// `__interrupt__` field if interrupts occurred.
    pub fn to_json(&self) -> Result<Value, crate::graph::error::GraphError> {
        let mut result = serde_json::to_value(&self.state)
            .map_err(crate::graph::error::GraphError::SerializationError)?;

        if let Some(ref interrupts) = self.interrupt {
            result["__interrupt__"] = serde_json::to_value(interrupts)
                .map_err(crate::graph::error::GraphError::SerializationError)?;
        }

        Ok(result)
    }

    /// Check if this result contains an interrupt
    pub fn has_interrupt(&self) -> bool {
        self.interrupt.is_some()
    }

    /// Get the interrupt information
    pub fn interrupt(&self) -> Option<&Vec<Interrupt>> {
        self.interrupt.as_ref()
    }
}

impl<S: State> From<S> for InvokeResult<S> {
    fn from(state: S) -> Self {
        Self::new(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::state::MessagesState;

    #[test]
    fn test_invoke_result() {
        let state = MessagesState::new();
        let result = InvokeResult::new(state);
        assert!(!result.has_interrupt());
    }

    #[test]
    fn test_invoke_result_with_interrupt() {
        let state = MessagesState::new();
        let interrupt = Interrupt::new(serde_json::json!("test"));
        let result = InvokeResult::with_interrupt(state, vec![interrupt]);
        assert!(result.has_interrupt());
    }
}

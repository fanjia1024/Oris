//! Reducer: projects events onto state (pure functional semantics).
//!
//! Axiom: state is the projection of the event log. Reducer must be deterministic for replay.

use serde::de::DeserializeOwned;

use crate::kernel::event::{Event, SequencedEvent};
use crate::kernel::state::KernelState;
use crate::kernel::KernelError;

/// Reducer applies events to state. Must be pure (deterministic) for replay.
pub trait Reducer<S: KernelState>: Send + Sync {
    /// Applies a single sequenced event to the state (in place).
    fn apply(&self, state: &mut S, event: &SequencedEvent) -> Result<(), KernelError>;
}

/// Reducer that applies only StateUpdated (deserialize payload → replace state); other events no-op.
/// Use for replay when the event log was produced by the graph (StateUpdated, Interrupted, Resumed, Completed).
pub struct StateUpdatedOnlyReducer;

impl<S> Reducer<S> for StateUpdatedOnlyReducer
where
    S: KernelState + DeserializeOwned,
{
    fn apply(&self, state: &mut S, event: &SequencedEvent) -> Result<(), KernelError> {
        if let Event::StateUpdated { payload, .. } = &event.event {
            let s: S = serde_json::from_value(payload.clone())
                .map_err(|e| KernelError::EventStore(e.to_string()))?;
            *state = s;
        }
        Ok(())
    }
}

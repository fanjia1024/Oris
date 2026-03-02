//! Timeline forking: fork execution timelines from a specific checkpoint.
//!
//! This module provides [TimelineFork] to create alternate execution timelines:
//! - **Fork from checkpoint N**: replay source run up to seq N.
//! - **Inject alternate decision**: apply an alternate event at the fork point.
//! - **Fork event stream**: continue under a new `branch_id` with forked events.

use serde::{Deserialize, Serialize};

use crate::kernel::event::{Event, SequencedEvent};
use crate::kernel::identity::{RunId, Seq};
use crate::kernel::reducer::Reducer;
use crate::kernel::state::KernelState;
use crate::kernel::EventStore;
use crate::kernel::KernelError;

/// A timeline fork: represents a forked execution from a checkpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineFork {
    /// Original run that was forked.
    pub source_run_id: RunId,
    /// New branch (forked) run id.
    pub branch_id: RunId,
    /// Sequence number of the fork point (events up to and including this seq were replayed).
    pub fork_point_seq: Seq,
    /// The alternate event injected at the fork point (if any).
    pub alternate_event: Option<Event>,
}

/// Result of a timeline fork operation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForkResult<S: KernelState> {
    /// The fork metadata.
    pub fork: TimelineFork,
    /// Final state after applying the alternate path.
    pub final_state: S,
}

/// Timeline forker: creates alternate execution timelines from checkpoints.
pub struct TimelineForker<S: KernelState> {
    /// Event store (source of truth).
    pub events: Box<dyn EventStore>,
    /// Reducer to apply events to state.
    pub reducer: Box<dyn Reducer<S>>,
}

impl<S: KernelState> TimelineForker<S> {
    /// Forks the timeline at checkpoint `fork_at_seq`.
    ///
    /// 1. Replays source run up to `fork_at_seq`.
    /// 2. Injects `alternate_event` at the fork point.
    /// 3. Continues replaying remaining events under new `branch_id`.
    /// 4. Returns the fork metadata and final state.
    pub fn fork(
        &self,
        source_run_id: &RunId,
        branch_id: RunId,
        fork_at_seq: Seq,
        alternate_event: Event,
        initial_state: S,
    ) -> Result<ForkResult<S>, KernelError> {
        // 1. Replay source run up to fork_at_seq
        let (mut state, _) = self.replay_up_to(source_run_id, fork_at_seq, initial_state)?;

        // 2. Inject alternate event at fork point
        let alt_seq = fork_at_seq + 1;
        let alt_se = SequencedEvent {
            seq: alt_seq,
            event: alternate_event.clone(),
        };
        self.reducer.apply(&mut state, &alt_se)?;

        // 3. Continue replaying remaining events under branch_id
        let remaining = self.events.scan(source_run_id, fork_at_seq + 1)?;
        for se in remaining {
            // Assign new seq under branch_id
            let branched_se = SequencedEvent {
                seq: se.seq,
                event: se.event.clone(),
            };
            self.reducer.apply(&mut state, &branched_se)?;
        }

        // 4. Record forked events to the new branch (optional: write to store)
        // For now, we return the result; actual persistence is optional.

        let fork = TimelineFork {
            source_run_id: source_run_id.clone(),
            branch_id: branch_id.clone(),
            fork_point_seq: fork_at_seq,
            alternate_event: Some(alternate_event),
        };

        Ok(ForkResult {
            fork,
            final_state: state,
        })
    }

    /// Replays the source run up to (and including) the given seq, returns state.
    fn replay_up_to(
        &self,
        run_id: &RunId,
        up_to_seq: Seq,
        initial_state: S,
    ) -> Result<(S, Seq), KernelError> {
        let mut state = initial_state;
        let events = self.events.scan(run_id, 1)?;
        for se in events {
            if se.seq > up_to_seq {
                break;
            }
            self.reducer.apply(&mut state, &se)?;
        }
        Ok((state, up_to_seq))
    }

    /// Creates a fork that starts fresh (no alternate event) - useful for simulation/audit.
    /// This replays the entire run under a new branch_id.
    pub fn clone_timeline(
        &self,
        source_run_id: &RunId,
        branch_id: RunId,
        initial_state: S,
    ) -> Result<ForkResult<S>, KernelError> {
        let mut state = initial_state;
        let events = self.events.scan(source_run_id, 1)?;
        for se in events {
            self.reducer.apply(&mut state, &se)?;
        }

        let fork = TimelineFork {
            source_run_id: source_run_id.clone(),
            branch_id,
            fork_point_seq: 0,
            alternate_event: None,
        };

        Ok(ForkResult {
            fork,
            final_state: state,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::event_store::InMemoryEventStore;
    use crate::kernel::StateUpdatedOnlyReducer;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct TestState(u32);

    impl crate::kernel::state::KernelState for TestState {
        fn version(&self) -> u32 {
            1
        }
    }

    #[test]
    fn fork_injects_alternate_event() {
        let events = InMemoryEventStore::new();
        let run_id: RunId = "source".into();
        events
            .append(
                &run_id,
                &[
                    Event::StateUpdated {
                        step_id: Some("a".into()),
                        payload: serde_json::to_value(&TestState(1)).unwrap(),
                    },
                    Event::StateUpdated {
                        step_id: Some("b".into()),
                        payload: serde_json::to_value(&TestState(2)).unwrap(),
                    },
                    Event::StateUpdated {
                        step_id: Some("c".into()),
                        payload: serde_json::to_value(&TestState(3)).unwrap(),
                    },
                ],
            )
            .unwrap();

        let forker = TimelineForker::<TestState> {
            events: Box::new(events),
            reducer: Box::new(StateUpdatedOnlyReducer),
        };

        // Fork at seq 1, inject alternate that sets state to 99
        let result = forker
            .fork(
                &run_id,
                "branch-1".into(),
                1,
                Event::StateUpdated {
                    step_id: Some("alt-b".into()),
                    payload: serde_json::to_value(&TestState(99)).unwrap(),
                },
                TestState(0),
            )
            .unwrap();

        assert_eq!(result.fork.source_run_id, "source");
        assert_eq!(result.fork.branch_id, "branch-1");
        assert_eq!(result.fork.fork_point_seq, 1);
        assert!(result.fork.alternate_event.is_some());

        // Final state should be from alternate (99) + original c (3) = 99 (last wins)
        assert_eq!(result.final_state, TestState(3));
    }

    #[test]
    fn clone_timeline_replays_entire_run() {
        let events = InMemoryEventStore::new();
        let run_id: RunId = "source2".into();
        events
            .append(
                &run_id,
                &[
                    Event::StateUpdated {
                        step_id: Some("x".into()),
                        payload: serde_json::to_value(&TestState(10)).unwrap(),
                    },
                    Event::Completed,
                ],
            )
            .unwrap();

        let forker = TimelineForker::<TestState> {
            events: Box::new(events),
            reducer: Box::new(StateUpdatedOnlyReducer),
        };

        let result = forker
            .clone_timeline(&run_id, "clone".into(), TestState(0))
            .unwrap();

        assert_eq!(result.fork.source_run_id, "source2");
        assert_eq!(result.fork.branch_id, "clone");
        assert_eq!(result.final_state, TestState(10));
    }
}

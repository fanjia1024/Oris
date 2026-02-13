//! Kernel driver: run_until_blocked, resume, replay.

use crate::kernel::action::{ActionExecutor, ActionResult};
use crate::kernel::event::{Event, EventStore};
use crate::kernel::identity::{RunId, Seq};
use crate::kernel::policy::{Policy, PolicyCtx};
use crate::kernel::reducer::Reducer;
use crate::kernel::snapshot::SnapshotStore;
use crate::kernel::state::KernelState;
use crate::kernel::step::{InterruptInfo, Next, StepFn};
use crate::kernel::KernelError;

/// Status of a run after run_until_blocked or resume.
#[derive(Clone, Debug)]
pub enum RunStatus {
    /// Run completed.
    Completed,
    /// Run is blocked (e.g. on interrupt or WaitSignal).
    Blocked(BlockedInfo),
}

#[derive(Clone, Debug)]
pub struct BlockedInfo {
    pub interrupt: Option<InterruptInfo>,
    pub wait_signal: Option<String>,
}

/// Signal to resume a blocked run (e.g. human approval, external event).
#[derive(Clone, Debug)]
pub enum Signal {
    Resume(serde_json::Value),
    Signal { name: String, value: serde_json::Value },
}

/// Kernel: event store, optional snapshot store, reducer, executor, step fn, policy.
pub struct Kernel<S: KernelState> {
    pub events: Box<dyn EventStore>,
    pub snaps: Option<Box<dyn SnapshotStore<S>>>,
    pub reducer: Box<dyn Reducer<S>>,
    pub exec: Box<dyn ActionExecutor>,
    pub step: Box<dyn StepFn<S>>,
    pub policy: Box<dyn Policy>,
}

impl<S: KernelState> Kernel<S> {
    /// Runs until the step returns Complete or Interrupt/WaitSignal. Returns run status.
    /// State is obtained by replaying the event log (or initial_state if the run has no events yet).
    pub fn run_until_blocked(
        &self,
        run_id: &RunId,
        initial_state: S,
    ) -> Result<RunStatus, KernelError> {
        self.run_loop(run_id, initial_state)
    }

    /// Resumes a blocked run with a signal (e.g. resume value or external signal).
    /// Appends Resumed (or Signal) event, then runs until blocked or complete.
    pub fn resume(
        &self,
        run_id: &RunId,
        initial_state: S,
        signal: Signal,
    ) -> Result<RunStatus, KernelError> {
        let value = match &signal {
            Signal::Resume(v) => v.clone(),
            Signal::Signal { value, .. } => value.clone(),
        };
        self.events
            .append(run_id, &[Event::Resumed { value }])?;
        self.run_loop(run_id, initial_state)
    }

    /// Inner loop: replay to get state, then step until Complete or Blocked.
    fn run_loop(&self, run_id: &RunId, initial_state: S) -> Result<RunStatus, KernelError> {
        const FROM_SEQ: Seq = 1;
        let mut state = initial_state;
        let sequenced = self.events.scan(run_id, FROM_SEQ)?;
        for se in sequenced {
            self.reducer.apply(&mut state, &se)?;
        }

        loop {
            let next = self.step.next(&state)?;
            match next {
                Next::Emit(evs) => {
                    if !evs.is_empty() {
                        let before = self.events.head(run_id)?;
                        self.events.append(run_id, &evs)?;
                        let new_events = self.events.scan(run_id, before + 1)?;
                        for se in new_events {
                            self.reducer.apply(&mut state, &se)?;
                        }
                    }
                }
                Next::Do(action) => {
                    self.policy.authorize(run_id, &action, &PolicyCtx::default())?;
                    let before = self.events.head(run_id)?;
                    let action_id = format!("{}-{}", run_id, before + 1);
                    let payload = serde_json::to_value(&action)
                        .map_err(|e| KernelError::Driver(e.to_string()))?;
                    self.events.append(
                        run_id,
                        &[Event::ActionRequested {
                            action_id: action_id.clone(),
                            payload,
                        }],
                    )?;
                    let result = self.exec.execute(run_id, &action);
                    match result {
                        Ok(ActionResult::Success(output)) => {
                            self.events.append(
                                run_id,
                                &[Event::ActionSucceeded {
                                    action_id: action_id.clone(),
                                    output,
                                }],
                            )?;
                        }
                        Ok(ActionResult::Failure(error)) => {
                            self.events.append(
                                run_id,
                                &[Event::ActionFailed {
                                    action_id,
                                    error,
                                }],
                            )?;
                        }
                        Err(e) => return Err(e),
                    }
                    let new_events = self.events.scan(run_id, before + 1)?;
                    for se in new_events {
                        self.reducer.apply(&mut state, &se)?;
                    }
                }
                Next::Interrupt(info) => {
                    self.events.append(
                        run_id,
                        &[Event::Interrupted {
                            value: info.value.clone(),
                        }],
                    )?;
                    return Ok(RunStatus::Blocked(BlockedInfo {
                        interrupt: Some(info),
                        wait_signal: None,
                    }));
                }
                Next::Complete => {
                    self.events.append(run_id, &[Event::Completed])?;
                    return Ok(RunStatus::Completed);
                }
            }
        }
    }

    /// Replays the run from the event log without executing external actions; returns final state.
    ///
    /// Scans all events for the run (from seq 1), applies each with the Reducer in order.
    /// Does not call ActionExecutor; any ActionRequested is satisfied by the following
    /// ActionSucceeded/ActionFailed already stored in the log (reducer applies them).
    pub fn replay(&self, run_id: &RunId, initial_state: S) -> Result<S, KernelError> {
        const FROM_SEQ: Seq = 1;
        let sequenced = self.events.scan(run_id, FROM_SEQ)?;
        let mut state = initial_state;
        for se in sequenced {
            self.reducer.apply(&mut state, &se)?;
        }
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::event_store::{InMemoryEventStore, SharedEventStore};
    use crate::kernel::stubs::{AllowAllPolicy, NoopActionExecutor, NoopStepFn};
    use crate::kernel::StateUpdatedOnlyReducer;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    struct TestState(u32);
    impl KernelState for TestState {
        fn version(&self) -> u32 {
            1
        }
    }

    #[test]
    fn run_until_blocked_complete() {
        let k = Kernel::<TestState> {
            events: Box::new(InMemoryEventStore::new()),
            snaps: None,
            reducer: Box::new(StateUpdatedOnlyReducer),
            exec: Box::new(NoopActionExecutor),
            step: Box::new(NoopStepFn),
            policy: Box::new(AllowAllPolicy),
        };
        let run_id = "run-complete".to_string();
        let status = k.run_until_blocked(&run_id, TestState(0)).unwrap();
        assert!(matches!(status, RunStatus::Completed));
    }

    struct InterruptOnceStep(bool);
    impl StepFn<TestState> for InterruptOnceStep {
        fn next(&self, _state: &TestState) -> Result<Next, KernelError> {
            if !self.0 {
                Ok(Next::Interrupt(InterruptInfo {
                    value: serde_json::json!("pause"),
                }))
            } else {
                Ok(Next::Complete)
            }
        }
    }

    #[test]
    fn run_until_blocked_then_resume() {
        let inner = Arc::new(InMemoryEventStore::new());
        let events = Box::new(SharedEventStore(inner.clone()));
        let run_id = "run-interrupt-resume".to_string();
        let k = Kernel::<TestState> {
            events: events,
            snaps: None,
            reducer: Box::new(StateUpdatedOnlyReducer),
            exec: Box::new(NoopActionExecutor),
            step: Box::new(InterruptOnceStep(false)),
            policy: Box::new(AllowAllPolicy),
        };
        let status = k.run_until_blocked(&run_id, TestState(0)).unwrap();
        assert!(matches!(status, RunStatus::Blocked(_)));

        let k2 = Kernel::<TestState> {
            events: Box::new(SharedEventStore(inner)),
            snaps: None,
            reducer: Box::new(StateUpdatedOnlyReducer),
            exec: Box::new(NoopActionExecutor),
            step: Box::new(InterruptOnceStep(true)),
            policy: Box::new(AllowAllPolicy),
        };
        let status2 = k2.resume(&run_id, TestState(0), Signal::Resume(serde_json::json!(1))).unwrap();
        assert!(matches!(status2, RunStatus::Completed));
    }
}

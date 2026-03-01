//! Agent checkpointer for human-in-the-loop: persist state at interrupt and resume with decisions.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::prompt::PromptArgs;
use crate::schemas::agent::AgentAction;

/// Serializable agent state saved at interrupt.
///
/// Used to resume execution after human provides decisions for pending tool calls.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentCheckpointState {
    /// Steps (action, observation) so far.
    pub steps: Vec<(AgentAction, String)>,
    /// Last plan input (chat_history, input, etc.).
    pub input_variables: PromptArgs,
    /// Tool calls that were pending when we interrupted (same order as in interrupt payload).
    pub pending_actions: Vec<AgentAction>,
}

/// Trait for persisting and loading agent checkpoint state (e.g. for HILP resume).
#[async_trait]
pub trait AgentCheckpointer: Send + Sync {
    /// Save state for the given thread. Overwrites any existing state for that thread.
    fn put(&self, thread_id: &str, state: &AgentCheckpointState);

    /// Load state for the given thread, if any.
    fn get(&self, thread_id: &str) -> Option<AgentCheckpointState>;

    /// Async-compatible save API for runtimes that should not block the async reactor.
    ///
    /// The default implementation preserves backward compatibility by delegating
    /// to the existing synchronous method.
    async fn put_async(&self, thread_id: &str, state: &AgentCheckpointState) {
        self.put(thread_id, state);
    }

    /// Async-compatible load API for runtimes that should not block the async reactor.
    ///
    /// The default implementation preserves backward compatibility by delegating
    /// to the existing synchronous method.
    async fn get_async(&self, thread_id: &str) -> Option<AgentCheckpointState> {
        self.get(thread_id)
    }
}

/// In-memory checkpointer (one state per thread_id).
#[derive(Default)]
pub struct InMemoryAgentSaver {
    state: std::sync::RwLock<std::collections::HashMap<String, AgentCheckpointState>>,
}

impl InMemoryAgentSaver {
    pub fn new() -> Self {
        Self {
            state: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait]
impl AgentCheckpointer for InMemoryAgentSaver {
    fn put(&self, thread_id: &str, state: &AgentCheckpointState) {
        let mut g = self.state.write().expect("lock");
        g.insert(thread_id.to_string(), state.clone());
    }

    fn get(&self, thread_id: &str) -> Option<AgentCheckpointState> {
        let g = self.state.read().expect("lock");
        g.get(thread_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use super::*;

    fn sample_state() -> AgentCheckpointState {
        AgentCheckpointState {
            steps: vec![(
                AgentAction {
                    tool: "search".to_string(),
                    tool_input: "{\"q\":\"oris\"}".to_string(),
                    log: "planned".to_string(),
                },
                "ok".to_string(),
            )],
            input_variables: PromptArgs::new(),
            pending_actions: vec![AgentAction {
                tool: "write_file".to_string(),
                tool_input: "{\"path\":\"README.md\"}".to_string(),
                log: "pending".to_string(),
            }],
        }
    }

    #[tokio::test]
    async fn in_memory_agent_saver_async_round_trip_uses_compatibility_wrapper() {
        let saver = InMemoryAgentSaver::new();
        let state = sample_state();

        saver.put_async("thread-1", &state).await;
        let loaded = saver.get_async("thread-1").await.expect("checkpoint");

        assert_eq!(loaded.steps.len(), 1);
        assert_eq!(loaded.pending_actions.len(), 1);
        assert_eq!(loaded.steps[0].0.tool, "search");
        assert_eq!(loaded.pending_actions[0].tool, "write_file");
    }

    struct AsyncOnlyProbeSaver {
        sync_put_calls: AtomicUsize,
        sync_get_calls: AtomicUsize,
        async_put_calls: AtomicUsize,
        async_get_calls: AtomicUsize,
        state: std::sync::RwLock<Option<AgentCheckpointState>>,
    }

    impl AsyncOnlyProbeSaver {
        fn new() -> Self {
            Self {
                sync_put_calls: AtomicUsize::new(0),
                sync_get_calls: AtomicUsize::new(0),
                async_put_calls: AtomicUsize::new(0),
                async_get_calls: AtomicUsize::new(0),
                state: std::sync::RwLock::new(None),
            }
        }
    }

    #[async_trait]
    impl AgentCheckpointer for AsyncOnlyProbeSaver {
        fn put(&self, _thread_id: &str, _state: &AgentCheckpointState) {
            self.sync_put_calls.fetch_add(1, Ordering::SeqCst);
        }

        fn get(&self, _thread_id: &str) -> Option<AgentCheckpointState> {
            self.sync_get_calls.fetch_add(1, Ordering::SeqCst);
            None
        }

        async fn put_async(&self, _thread_id: &str, state: &AgentCheckpointState) {
            self.async_put_calls.fetch_add(1, Ordering::SeqCst);
            let mut guard = self.state.write().expect("lock");
            *guard = Some(state.clone());
        }

        async fn get_async(&self, _thread_id: &str) -> Option<AgentCheckpointState> {
            self.async_get_calls.fetch_add(1, Ordering::SeqCst);
            self.state.read().expect("lock").clone()
        }
    }

    #[tokio::test]
    async fn async_agent_checkpointer_can_override_async_methods_without_sync_fallback() {
        let saver = Arc::new(AsyncOnlyProbeSaver::new());
        let state = sample_state();

        saver.put_async("thread-2", &state).await;
        let loaded = saver.get_async("thread-2").await.expect("checkpoint");

        assert_eq!(loaded.pending_actions.len(), 1);
        assert_eq!(saver.async_put_calls.load(Ordering::SeqCst), 1);
        assert_eq!(saver.async_get_calls.load(Ordering::SeqCst), 1);
        assert_eq!(saver.sync_put_calls.load(Ordering::SeqCst), 0);
        assert_eq!(saver.sync_get_calls.load(Ordering::SeqCst), 0);
    }
}

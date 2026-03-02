use std::sync::Arc;

use async_trait::async_trait;
use oris_execution_runtime::{
    ExecutionCheckpointView, ExecutionGraphBridge, ExecutionGraphBridgeError, ExecutionInvokeView,
    ExecutionStateView,
};
use serde_json::Value;

use crate::graph::{Command, CompiledGraph, MessagesState, RunnableConfig, StateOrCommand};
use crate::schemas::messages::Message;

pub(crate) struct CompiledGraphExecutionBridge {
    compiled: Arc<CompiledGraph<MessagesState>>,
}

impl CompiledGraphExecutionBridge {
    pub(crate) fn new(compiled: Arc<CompiledGraph<MessagesState>>) -> Self {
        Self { compiled }
    }
}

#[async_trait]
impl ExecutionGraphBridge for CompiledGraphExecutionBridge {
    async fn run(
        &self,
        thread_id: &str,
        input: &str,
    ) -> Result<ExecutionInvokeView, ExecutionGraphBridgeError> {
        let initial = MessagesState::with_messages(vec![Message::new_human_message(input)]);
        let config = RunnableConfig::with_thread_id(thread_id);
        let result = self
            .compiled
            .invoke_with_config_interrupt(StateOrCommand::State(initial), &config)
            .await
            .map_err(|e| ExecutionGraphBridgeError::internal(e.to_string()))?;
        Ok(ExecutionInvokeView {
            interrupts: result
                .interrupt
                .unwrap_or_default()
                .into_iter()
                .map(|interrupt| interrupt.value)
                .collect(),
        })
    }

    async fn resume(
        &self,
        thread_id: &str,
        checkpoint_id: Option<&str>,
        value: Value,
    ) -> Result<ExecutionInvokeView, ExecutionGraphBridgeError> {
        let config = checkpoint_config(thread_id, checkpoint_id);
        let result = self
            .compiled
            .invoke_with_config_interrupt(StateOrCommand::Command(Command::resume(value)), &config)
            .await
            .map_err(|e| ExecutionGraphBridgeError::internal(e.to_string()))?;
        Ok(ExecutionInvokeView {
            interrupts: result
                .interrupt
                .unwrap_or_default()
                .into_iter()
                .map(|interrupt| interrupt.value)
                .collect(),
        })
    }

    async fn replay(
        &self,
        thread_id: &str,
        checkpoint_id: Option<&str>,
    ) -> Result<(), ExecutionGraphBridgeError> {
        let config = checkpoint_config(thread_id, checkpoint_id);
        self.compiled
            .invoke_with_config(None, &config)
            .await
            .map_err(|e| ExecutionGraphBridgeError::internal(e.to_string()))?;
        Ok(())
    }

    async fn snapshot(
        &self,
        thread_id: &str,
        checkpoint_id: Option<&str>,
    ) -> Result<ExecutionStateView, ExecutionGraphBridgeError> {
        let config = checkpoint_config(thread_id, checkpoint_id);
        let snapshot = self
            .compiled
            .get_state(&config)
            .await
            .map_err(map_snapshot_error)?;
        let values = serde_json::to_value(&snapshot.values).map_err(|e| {
            ExecutionGraphBridgeError::internal(format!("serialize state failed: {}", e))
        })?;
        Ok(ExecutionStateView {
            checkpoint_id: snapshot.checkpoint_id().cloned(),
            created_at: snapshot.created_at,
            values,
        })
    }

    async fn history(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ExecutionCheckpointView>, ExecutionGraphBridgeError> {
        let config = RunnableConfig::with_thread_id(thread_id);
        let history = self
            .compiled
            .get_state_history(&config)
            .await
            .map_err(|e| ExecutionGraphBridgeError::internal(e.to_string()))?;
        Ok(history
            .into_iter()
            .map(|snapshot| ExecutionCheckpointView {
                checkpoint_id: snapshot.checkpoint_id().cloned(),
                created_at: snapshot.created_at,
            })
            .collect())
    }
}

fn checkpoint_config(thread_id: &str, checkpoint_id: Option<&str>) -> RunnableConfig {
    if let Some(checkpoint_id) = checkpoint_id {
        RunnableConfig::with_checkpoint(thread_id, checkpoint_id)
    } else {
        RunnableConfig::with_thread_id(thread_id)
    }
}

fn map_snapshot_error(error: crate::graph::GraphError) -> ExecutionGraphBridgeError {
    let message = error.to_string();
    if message.contains("No state found") || message.contains("Checkpoint not found") {
        ExecutionGraphBridgeError::not_found(message)
    } else {
        ExecutionGraphBridgeError::internal(message)
    }
}

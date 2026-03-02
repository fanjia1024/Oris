//! Graph-facing bridge types for the execution server.

#![cfg(feature = "execution-server")]

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionGraphBridgeErrorKind {
    NotFound,
    Internal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionGraphBridgeError {
    pub kind: ExecutionGraphBridgeErrorKind,
    pub message: String,
}

impl ExecutionGraphBridgeError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            kind: ExecutionGraphBridgeErrorKind::NotFound,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            kind: ExecutionGraphBridgeErrorKind::Internal,
            message: message.into(),
        }
    }
}

impl fmt::Display for ExecutionGraphBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ExecutionGraphBridgeError {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionInvokeView {
    pub interrupts: Vec<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionStateView {
    pub checkpoint_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub values: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionCheckpointView {
    pub checkpoint_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait ExecutionGraphBridge: Send + Sync {
    async fn run(
        &self,
        thread_id: &str,
        input: &str,
    ) -> Result<ExecutionInvokeView, ExecutionGraphBridgeError>;

    async fn resume(
        &self,
        thread_id: &str,
        checkpoint_id: Option<&str>,
        value: Value,
    ) -> Result<ExecutionInvokeView, ExecutionGraphBridgeError>;

    async fn replay(
        &self,
        thread_id: &str,
        checkpoint_id: Option<&str>,
    ) -> Result<(), ExecutionGraphBridgeError>;

    async fn snapshot(
        &self,
        thread_id: &str,
        checkpoint_id: Option<&str>,
    ) -> Result<ExecutionStateView, ExecutionGraphBridgeError>;

    async fn history(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ExecutionCheckpointView>, ExecutionGraphBridgeError>;
}

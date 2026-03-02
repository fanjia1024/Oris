//! Interrupt: standardized representation and persistence of kernel interrupts.
//!
//! This module defines the [Interrupt] struct and [InterruptStore] trait for
//! persisting interrupts alongside execution checkpoints.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::kernel::identity::RunId;

/// Unique identifier for an interrupt.
pub type InterruptId = String;

/// Kind of interrupt.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum InterruptKind {
    /// Human-in-the-loop: waiting for user input.
    HumanInTheLoop,
    /// Approval required: waiting for external approval.
    ApprovalRequired,
    /// Tool call waiting: waiting for a blocking tool to complete.
    ToolCallWaiting,
    /// Custom interrupt kind.
    Custom(String),
}

/// A kernel interrupt: represents a pause in execution waiting for external input.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Interrupt {
    /// Unique interrupt identifier.
    pub id: InterruptId,
    /// Run (thread) this interrupt belongs to.
    pub thread_id: RunId,
    /// Kind of interrupt.
    pub kind: InterruptKind,
    /// JSON schema describing the expected payload for resolution.
    pub payload_schema: serde_json::Value,
    /// When the interrupt was created.
    pub created_at: DateTime<Utc>,
    /// Optional step/node that triggered the interrupt.
    pub step_id: Option<String>,
}

impl Interrupt {
    /// Creates a new interrupt.
    pub fn new(
        id: InterruptId,
        thread_id: RunId,
        kind: InterruptKind,
        payload_schema: serde_json::Value,
    ) -> Self {
        Self {
            id,
            thread_id,
            kind,
            payload_schema,
            created_at: Utc::now(),
            step_id: None,
        }
    }

    /// Creates a new interrupt with a step id.
    pub fn with_step(
        id: InterruptId,
        thread_id: RunId,
        kind: InterruptKind,
        payload_schema: serde_json::Value,
        step_id: String,
    ) -> Self {
        Self {
            id,
            thread_id,
            kind,
            payload_schema,
            created_at: Utc::now(),
            step_id: Some(step_id),
        }
    }
}

/// Store for persisting interrupts.
pub trait InterruptStore: Send + Sync {
    /// Saves an interrupt.
    fn save(&self, interrupt: &Interrupt) -> Result<(), InterruptError>;

    /// Loads an interrupt by id.
    fn load(&self, id: &InterruptId) -> Result<Option<Interrupt>, InterruptError>;

    /// Loads all interrupts for a run.
    fn load_for_run(&self, thread_id: &RunId) -> Result<Vec<Interrupt>, InterruptError>;

    /// Deletes an interrupt (e.g. after resolution).
    fn delete(&self, id: &InterruptId) -> Result<(), InterruptError>;
}

/// Errors for interrupt operations.
#[derive(Debug, thiserror::Error)]
pub enum InterruptError {
    #[error("Interrupt store error: {0}")]
    Store(String),
    #[error("Interrupt not found: {0}")]
    NotFound(InterruptId),
}

/// In-memory interrupt store: one interrupt per id.
#[derive(Debug, Default)]
pub struct InMemoryInterruptStore {
    by_id: std::sync::RwLock<std::collections::HashMap<InterruptId, Interrupt>>,
}

impl InMemoryInterruptStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl InterruptStore for InMemoryInterruptStore {
    fn save(&self, interrupt: &Interrupt) -> Result<(), InterruptError> {
        let mut guard = self
            .by_id
            .write()
            .map_err(|e| InterruptError::Store(e.to_string()))?;
        guard.insert(interrupt.id.clone(), interrupt.clone());
        Ok(())
    }

    fn load(&self, id: &InterruptId) -> Result<Option<Interrupt>, InterruptError> {
        let guard = self
            .by_id
            .read()
            .map_err(|e| InterruptError::Store(e.to_string()))?;
        Ok(guard.get(id).cloned())
    }

    fn load_for_run(&self, thread_id: &RunId) -> Result<Vec<Interrupt>, InterruptError> {
        let guard = self
            .by_id
            .read()
            .map_err(|e| InterruptError::Store(e.to_string()))?;
        Ok(guard
            .values()
            .filter(|i| i.thread_id == *thread_id)
            .cloned()
            .collect())
    }

    fn delete(&self, id: &InterruptId) -> Result<(), InterruptError> {
        let mut guard = self
            .by_id
            .write()
            .map_err(|e| InterruptError::Store(e.to_string()))?;
        guard.remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_interrupt() {
        let store = InMemoryInterruptStore::new();
        let interrupt = Interrupt::new(
            "intr-1".into(),
            "run-1".into(),
            InterruptKind::HumanInTheLoop,
            serde_json::json!({"type": "string"}),
        );
        store.save(&interrupt).unwrap();

        let loaded = store.load(&"intr-1".into()).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, "intr-1");
    }

    #[test]
    fn load_for_run_filters() {
        let store = InMemoryInterruptStore::new();
        store
            .save(&Interrupt::new(
                "i1".into(),
                "run-a".into(),
                InterruptKind::ApprovalRequired,
                serde_json::json!({}),
            ))
            .unwrap();
        store
            .save(&Interrupt::new(
                "i2".into(),
                "run-b".into(),
                InterruptKind::HumanInTheLoop,
                serde_json::json!({}),
            ))
            .unwrap();
        store
            .save(&Interrupt::new(
                "i3".into(),
                "run-a".into(),
                InterruptKind::ToolCallWaiting,
                serde_json::json!({}),
            ))
            .unwrap();

        let run_a = store.load_for_run(&"run-a".into()).unwrap();
        assert_eq!(run_a.len(), 2);
    }

    #[test]
    fn delete_removes_interrupt() {
        let store = InMemoryInterruptStore::new();
        store
            .save(&Interrupt::new(
                "i1".into(),
                "run-1".into(),
                InterruptKind::Custom("test".into()),
                serde_json::json!({}),
            ))
            .unwrap();
        store.delete(&"i1".into()).unwrap();
        let loaded = store.load(&"i1".into()).unwrap();
        assert!(loaded.is_none());
    }
}

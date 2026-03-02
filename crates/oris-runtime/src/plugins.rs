//! Plugin categories and interfaces for the Oris kernel.
//!
//! Standardizes the types of plugins the kernel will recognize and load:
//! - **Node**: graph node factories ([`crate::graph::NodePlugin`])
//! - **Tool**: tool factories
//! - **Memory**: memory backends
//! - **LLMAdapter**: LLM implementations
//! - **Scheduler**: scheduler implementations
//!
//! All plugins must declare [PluginMetadata] (determinism, side effects, replay safety)
//! so the kernel can enforce replay substitution and sandbox routing.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::language_models::llm::LLM;
use crate::schemas::memory::BaseMemory;
use crate::tools::Tool;

/// Error returned when plugin creation or lookup fails.
#[derive(Error, Debug)]
#[error("Plugin error: {0}")]
pub struct PluginError(pub String);

/// Declared behavioral boundaries of a plugin for kernel enforcement.
///
/// Every plugin must provide this so the kernel can decide replay substitution
/// and sandbox routing. See [allow_in_replay] and [requires_sandbox].
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin output is deterministic for the same inputs (no clock, randomness, or external I/O).
    #[serde(default)]
    pub deterministic: bool,
    /// Plugin may perform side effects (network, disk, state changes).
    #[serde(default = "default_true")]
    pub side_effects: bool,
    /// Plugin is safe to use in Replay/Verify mode (deterministic and no nondeterministic traps).
    #[serde(default)]
    pub replay_safe: bool,
}

fn default_true() -> bool {
    true
}

impl PluginMetadata {
    /// Conservative defaults: not deterministic, has side effects, not replay-safe.
    pub fn conservative() -> Self {
        Self {
            deterministic: false,
            side_effects: true,
            replay_safe: false,
        }
    }

    /// All deterministic, no side effects, replay-safe (e.g. pure compute).
    pub fn pure() -> Self {
        Self {
            deterministic: true,
            side_effects: false,
            replay_safe: true,
        }
    }
}

/// Trait for plugins that declare [PluginMetadata]. Required by all plugin interfaces.
pub trait HasPluginMetadata: Send + Sync {
    /// Behavioral boundaries for kernel enforcement (replay, sandbox).
    fn plugin_metadata(&self) -> PluginMetadata {
        PluginMetadata::conservative()
    }
}

/// Returns true if the plugin is allowed in Replay/Verify mode (kernel enforcement).
#[inline]
pub fn allow_in_replay(meta: &PluginMetadata) -> bool {
    meta.replay_safe
}

/// Returns true if the plugin should be routed through a sandbox (kernel enforcement).
#[inline]
pub fn requires_sandbox(meta: &PluginMetadata) -> bool {
    meta.side_effects
}

/// Standard categories of plugins the kernel recognizes and can load.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PluginCategory {
    /// Graph node plugin: creates nodes from config (see [`crate::graph::NodePlugin`]).
    Node,
    /// Tool plugin: creates tools from config.
    Tool,
    /// Memory plugin: creates memory backends from config.
    Memory,
    /// LLM adapter: creates LLM instances from config.
    LLMAdapter,
    /// Scheduler plugin: creates scheduler instances from config.
    Scheduler,
}

impl PluginCategory {
    /// Stable string identifier for serialization and registry keys.
    pub fn as_str(self) -> &'static str {
        match self {
            PluginCategory::Node => "node",
            PluginCategory::Tool => "tool",
            PluginCategory::Memory => "memory",
            PluginCategory::LLMAdapter => "llm_adapter",
            PluginCategory::Scheduler => "scheduler",
        }
    }
}

/// Plugin interface for creating tools from runtime configuration.
///
/// Implement this trait to register custom tool types that the kernel can
/// instantiate from a config payload (e.g. JSON). Must declare [PluginMetadata].
pub trait ToolPlugin: HasPluginMetadata {
    /// Stable plugin type identifier used for registration and lookup.
    fn plugin_type(&self) -> &str;

    /// Create a tool instance from the given configuration.
    fn create_tool(&self, config: &Value) -> Result<Arc<dyn Tool>, PluginError>;
}

/// Plugin interface for creating memory backends from runtime configuration.
///
/// Implement this trait to register custom memory types that the kernel can
/// instantiate from a config payload. Must declare [PluginMetadata].
pub trait MemoryPlugin: HasPluginMetadata {
    /// Stable plugin type identifier used for registration and lookup.
    fn plugin_type(&self) -> &str;

    /// Create a memory instance from the given configuration.
    fn create_memory(&self, config: &Value) -> Result<Arc<Mutex<dyn BaseMemory>>, PluginError>;
}

/// Plugin interface for creating LLM adapters from runtime configuration.
///
/// Implement this trait to register custom LLM backends that the kernel can
/// instantiate from a config payload (e.g. API keys, model names). Must declare [PluginMetadata].
pub trait LLMAdapter: HasPluginMetadata {
    /// Stable plugin type identifier used for registration and lookup.
    fn plugin_type(&self) -> &str;

    /// Create an LLM instance from the given configuration.
    fn create_llm(&self, config: &Value) -> Result<Arc<dyn LLM>, PluginError>;
}

/// Plugin interface for scheduler implementations the kernel can load.
///
/// Implement this trait to register custom schedulers. The kernel uses
/// the plugin type to resolve and instantiate the scheduler from config. Must declare [PluginMetadata].
pub trait SchedulerPlugin: HasPluginMetadata {
    /// Stable plugin type identifier used for registration and lookup.
    fn plugin_type(&self) -> &str;

    /// Optional human-readable description for discovery and docs.
    fn description(&self) -> &str {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_category_as_str() {
        assert_eq!(PluginCategory::Node.as_str(), "node");
        assert_eq!(PluginCategory::Tool.as_str(), "tool");
        assert_eq!(PluginCategory::Memory.as_str(), "memory");
        assert_eq!(PluginCategory::LLMAdapter.as_str(), "llm_adapter");
        assert_eq!(PluginCategory::Scheduler.as_str(), "scheduler");
    }

    #[test]
    fn plugin_category_equality() {
        assert_eq!(PluginCategory::Node, PluginCategory::Node);
        assert_ne!(PluginCategory::Tool, PluginCategory::Memory);
    }

    #[test]
    fn plugin_metadata_conservative() {
        let m = PluginMetadata::conservative();
        assert!(!m.deterministic);
        assert!(m.side_effects);
        assert!(!m.replay_safe);
    }

    #[test]
    fn plugin_metadata_pure() {
        let m = PluginMetadata::pure();
        assert!(m.deterministic);
        assert!(!m.side_effects);
        assert!(m.replay_safe);
    }

    #[test]
    fn allow_in_replay_uses_replay_safe() {
        assert!(allow_in_replay(&PluginMetadata::pure()));
        assert!(!allow_in_replay(&PluginMetadata::conservative()));
    }

    #[test]
    fn requires_sandbox_uses_side_effects() {
        assert!(requires_sandbox(&PluginMetadata::conservative()));
        assert!(!requires_sandbox(&PluginMetadata::pure()));
    }
}

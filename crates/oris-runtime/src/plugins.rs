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
//! so the kernel can enforce replay substitution, sandbox routing, and execution mode
//! ([PluginExecutionMode]: InProcess, IsolatedProcess, Remote).

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

/// Compatibility metadata for plugin load validation. Check with [validate_plugin_compatibility].
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginCompatibility {
    /// Plugin API version (e.g. \"0.2\").
    #[serde(default)]
    pub plugin_api_version: String,
    /// Kernel compatibility requirement (e.g. \">=0.2.0\" or exact \"0.2.7\").
    #[serde(default)]
    pub kernel_compat: String,
    /// Optional schema hash for strict schema compatibility.
    #[serde(default)]
    pub schema_hash: Option<String>,
}

/// Validates plugin compatibility against the current kernel version.
///
/// Returns `Ok(())` if the plugin is compatible, or [PluginError] if validation fails.
/// Use before registering a plugin to prevent runtime corruption.
pub fn validate_plugin_compatibility(
    compat: &PluginCompatibility,
    kernel_version: &str,
) -> Result<(), PluginError> {
    if kernel_version.is_empty() {
        return Ok(());
    }
    if compat.kernel_compat.is_empty() {
        return Ok(());
    }
    let req = compat.kernel_compat.trim();
    if req.starts_with(">=") {
        let min = req.trim_start_matches(">=").trim();
        if kernel_version < min {
            return Err(PluginError(format!(
                "kernel version {} does not meet plugin requirement {}",
                kernel_version, compat.kernel_compat
            )));
        }
    } else if req != kernel_version {
        return Err(PluginError(format!(
            "kernel version {} does not match plugin kernel_compat {}",
            kernel_version, compat.kernel_compat
        )));
    }
    Ok(())
}

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

/// Execution mode for plugin isolation. Used by the kernel to route plugin execution.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PluginExecutionMode {
    /// Run in the same process; no isolation (e.g. when plugin is pure or trusted).
    InProcess,
    /// Run in an isolated process (subprocess / sandbox) for safety.
    IsolatedProcess,
    /// Run on a remote worker or service.
    Remote,
}

impl PluginExecutionMode {
    /// Stable string identifier for serialization and config.
    pub fn as_str(self) -> &'static str {
        match self {
            PluginExecutionMode::InProcess => "in_process",
            PluginExecutionMode::IsolatedProcess => "isolated_process",
            PluginExecutionMode::Remote => "remote",
        }
    }
}

/// Selects execution mode for a plugin based on its metadata (kernel enforcement).
///
/// Plugins with side effects are routed to [PluginExecutionMode::IsolatedProcess] by default;
/// pure plugins use [PluginExecutionMode::InProcess]. Callers can override for policy (e.g. force Remote).
#[inline]
pub fn route_to_execution_mode(meta: &PluginMetadata) -> PluginExecutionMode {
    if requires_sandbox(meta) {
        PluginExecutionMode::IsolatedProcess
    } else {
        PluginExecutionMode::InProcess
    }
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

    #[test]
    fn plugin_execution_mode_as_str() {
        assert_eq!(PluginExecutionMode::InProcess.as_str(), "in_process");
        assert_eq!(
            PluginExecutionMode::IsolatedProcess.as_str(),
            "isolated_process"
        );
        assert_eq!(PluginExecutionMode::Remote.as_str(), "remote");
    }

    #[test]
    fn route_to_execution_mode_pure_in_process() {
        assert_eq!(
            route_to_execution_mode(&PluginMetadata::pure()),
            PluginExecutionMode::InProcess
        );
    }

    #[test]
    fn route_to_execution_mode_side_effects_isolated() {
        assert_eq!(
            route_to_execution_mode(&PluginMetadata::conservative()),
            PluginExecutionMode::IsolatedProcess
        );
    }

    #[test]
    fn validate_plugin_compatibility_empty_ok() {
        let c = PluginCompatibility::default();
        assert!(validate_plugin_compatibility(&c, "0.2.7").is_ok());
        assert!(validate_plugin_compatibility(&c, "").is_ok());
    }

    #[test]
    fn validate_plugin_compatibility_exact_match() {
        let c = PluginCompatibility {
            plugin_api_version: "0.2".into(),
            kernel_compat: "0.2.7".into(),
            schema_hash: None,
        };
        assert!(validate_plugin_compatibility(&c, "0.2.7").is_ok());
        assert!(validate_plugin_compatibility(&c, "0.2.8").is_err());
    }

    #[test]
    fn validate_plugin_compatibility_gte() {
        let c = PluginCompatibility {
            plugin_api_version: "0.2".into(),
            kernel_compat: ">=0.2.0".into(),
            schema_hash: None,
        };
        assert!(validate_plugin_compatibility(&c, "0.2.7").is_ok());
        assert!(validate_plugin_compatibility(&c, "0.2.0").is_ok());
        assert!(validate_plugin_compatibility(&c, "0.1.9").is_err());
    }
}

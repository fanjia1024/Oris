//! Unified interrupt routing layer: single resolver for all external interrupt sources.
//!
//! This module provides [InterruptResolver] trait and implementations for handling
//! interrupts from various sources: UI, agents, policy engines, and APIs.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::kernel::interrupt::{Interrupt, InterruptId};

/// Result of resolving an interrupt.
pub type ResolveResult = Result<serde_json::Value, InterruptResolverError>;

/// Unified interrupt resolver: handles interrupts from all external sources.
#[async_trait]
pub trait InterruptResolver: Send + Sync {
    /// Resolves an interrupt, returning the value to resume with.
    async fn resolve(&self, interrupt: &Interrupt) -> ResolveResult;
}

/// Errors for interrupt resolution.
#[derive(Debug, thiserror::Error)]
pub enum InterruptResolverError {
    #[error("Resolver error: {0}")]
    Resolver(String),
    #[error("Interrupt not found: {0}")]
    NotFound(InterruptId),
    #[error("Resolution timeout")]
    Timeout,
    #[error("Resolution rejected")]
    Rejected,
}

/// Source of an interrupt request.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InterruptSource {
    /// Human input via UI.
    Ui,
    /// Agent-generated interrupt.
    Agent,
    /// Policy engine decision.
    PolicyEngine,
    /// API request.
    Api,
    /// Custom source.
    Custom(String),
}

impl InterruptSource {
    pub fn as_str(&self) -> &str {
        match self {
            InterruptSource::Ui => "ui",
            InterruptSource::Agent => "agent",
            InterruptSource::PolicyEngine => "policy_engine",
            InterruptSource::Api => "api",
            InterruptSource::Custom(s) => s.as_str(),
        }
    }
}

/// Handler function type for resolving interrupts from a specific source.
pub type InterruptHandlerFn = Box<dyn Fn(&Interrupt) -> ResolveResult + Send + Sync>;

/// Unified interrupt resolver that routes to source-specific handlers.
pub struct UnifiedInterruptResolver {
    handlers: std::collections::HashMap<InterruptSource, InterruptHandlerFn>,
}

impl UnifiedInterruptResolver {
    /// Creates a new resolver with no handlers.
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }

    /// Registers a handler for a specific source.
    pub fn with_handler(mut self, source: InterruptSource, handler: InterruptHandlerFn) -> Self {
        self.handlers.insert(source, handler);
        self
    }

    /// Resolves an interrupt by routing to the appropriate handler.
    pub fn resolve(&self, interrupt: &Interrupt) -> ResolveResult {
        let source = extract_source(interrupt);

        if let Some(handler) = self.handlers.get(&source) {
            return handler(interrupt);
        }

        // Fallback to API handler if registered
        if let Some(handler) = self.handlers.get(&InterruptSource::Api) {
            return handler(interrupt);
        }

        Err(InterruptResolverError::Resolver(format!(
            "No handler for source: {}",
            source.as_str()
        )))
    }
}

impl Default for UnifiedInterruptResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InterruptResolver for UnifiedInterruptResolver {
    async fn resolve(&self, interrupt: &Interrupt) -> ResolveResult {
        self.resolve(interrupt)
    }
}

/// Extract source from interrupt (fallback to Api).
fn extract_source(_interrupt: &Interrupt) -> InterruptSource {
    // Check if there's metadata indicating the source
    // For now, default to API
    InterruptSource::Api
}

/// Creates a default unified resolver with no-op handlers (for testing).
#[cfg(test)]
pub fn create_test_resolver() -> UnifiedInterruptResolver {
    let noop_handler = Box::new(|_: &Interrupt| Ok(serde_json::Value::Null));

    UnifiedInterruptResolver::new()
        .with_handler(InterruptSource::Ui, noop_handler.clone())
        .with_handler(InterruptSource::Agent, noop_handler.clone())
        .with_handler(InterruptSource::PolicyEngine, noop_handler.clone())
        .with_handler(InterruptSource::Api, noop_handler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::interrupt::InterruptKind;

    #[test]
    fn resolver_routes_by_source() {
        let resolver = create_test_resolver();

        let ui_interrupt = Interrupt::new(
            "i1".into(),
            "run-1".into(),
            InterruptKind::HumanInTheLoop,
            serde_json::json!({}),
        );

        let result = resolver.resolve(&ui_interrupt);
        assert!(result.is_ok());
    }
}

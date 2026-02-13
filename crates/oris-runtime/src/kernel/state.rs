//! State trait for the Oris kernel.
//!
//! State must be serializable and versioned for schema evolution (2.0 migration).

/// Kernel state: cloneable, send, sync, and with a schema version for migrations.
///
/// Existing graph::State can implement this by adding `fn version(&self) -> u32` (e.g. returning 1).
pub trait KernelState: Clone + Send + Sync + 'static {
    /// Schema version for state migration (e.g. 1, 2, ...).
    fn version(&self) -> u32;
}

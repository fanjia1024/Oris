//! Run identity types for the Oris kernel.
//!
//! RunId identifies a long-running run; StepId identifies a step (e.g. node + attempt);
//! Seq is the monotonically increasing event sequence number per run.

/// Identifies a long-running run (maps to current `thread_id` in graph API).
pub type RunId = String;

/// Identifies a single step (e.g. node name + attempt).
pub type StepId = String;

/// Monotonically increasing event sequence number per run.
pub type Seq = u64;

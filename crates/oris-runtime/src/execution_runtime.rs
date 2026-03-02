//! Execution runtime control plane, scheduler, repositories, and compatibility re-exports.
//!
//! Graph-aware HTTP helpers are now owned by [`crate::execution_server`]. Only
//! the legacy compatibility re-exports for that graph-aware surface are
//! deprecated here.

pub use oris_execution_runtime::*;

#[cfg(feature = "execution-server")]
#[deprecated(
    since = "0.2.12",
    note = "use `oris_runtime::execution_server::{build_router, ApiRole, ExecutionApiState}`"
)]
pub use crate::execution_server::{build_router, ApiRole, ExecutionApiState};

#[cfg(all(feature = "execution-server", feature = "sqlite-persistence"))]
#[deprecated(
    since = "0.2.12",
    note = "use `oris_runtime::execution_server::*` for runtime benchmark helpers"
)]
pub use crate::execution_server::{
    canonical_runtime_benchmark_baseline_path, run_runtime_benchmark_suite,
    runtime_benchmark_suite_pretty_json, write_runtime_benchmark_suite,
    RuntimeBenchmarkEnvironment, RuntimeBenchmarkMetric, RuntimeBenchmarkSuiteReport,
    RUNTIME_BENCHMARK_BASELINE_DOC_PATH,
};

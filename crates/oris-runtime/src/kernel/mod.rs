//! Compatibility shim for the extracted `oris-kernel` crate.
//!
//! Graph-dependent runtime helpers now live under [`crate::execution_runtime`]
//! and [`crate::execution_server`].
//! `crate::kernel::runtime` remains as a compatibility re-export.

pub use oris_kernel::*;

pub mod runtime {
    pub use oris_execution_runtime::*;

    #[cfg(feature = "execution-server")]
    #[deprecated(
        since = "0.2.12",
        note = "use `oris_runtime::execution_server::{build_router, ApiRole, ExecutionApiState}` instead"
    )]
    pub use crate::execution_server::{build_router, ApiRole, ExecutionApiState};

    #[cfg(all(feature = "execution-server", feature = "sqlite-persistence"))]
    #[deprecated(
        since = "0.2.12",
        note = "use `oris_runtime::execution_server::*` instead of `oris_runtime::kernel::runtime::*` for benchmark helpers"
    )]
    pub use crate::execution_server::{
        canonical_runtime_benchmark_baseline_path, run_runtime_benchmark_suite,
        runtime_benchmark_suite_pretty_json, write_runtime_benchmark_suite,
        RuntimeBenchmarkEnvironment, RuntimeBenchmarkMetric, RuntimeBenchmarkSuiteReport,
        RUNTIME_BENCHMARK_BASELINE_DOC_PATH,
    };
}

#[cfg(feature = "execution-server")]
#[deprecated(
    since = "0.2.12",
    note = "use `oris_runtime::execution_server::{build_router, ApiRole, ExecutionApiState}`"
)]
pub use crate::execution_server::{build_router, ApiRole, ExecutionApiState};
#[cfg(all(feature = "execution-server", feature = "sqlite-persistence"))]
#[deprecated(
    since = "0.2.12",
    note = "use `oris_runtime::execution_server::*` for graph-aware benchmark helpers"
)]
pub use crate::execution_server::{
    canonical_runtime_benchmark_baseline_path, run_runtime_benchmark_suite,
    runtime_benchmark_suite_pretty_json, write_runtime_benchmark_suite,
    RuntimeBenchmarkEnvironment, RuntimeBenchmarkMetric, RuntimeBenchmarkSuiteReport,
    RUNTIME_BENCHMARK_BASELINE_DOC_PATH,
};
#[cfg(feature = "kernel-postgres")]
pub use oris_execution_runtime::PostgresRuntimeRepository;
#[cfg(feature = "execution-server")]
pub use oris_execution_runtime::{
    canonical_runtime_api_contract_path, generate_runtime_api_contract,
    runtime_api_contract_pretty_json, write_runtime_api_contract, ApiEnvelope, ApiError, ApiMeta,
    AttemptRetryHistoryItem, AttemptRetryHistoryResponse, AuditLogItem, AuditLogListResponse,
    CancelJobRequest, CancelJobResponse, CheckpointInspectResponse, DeadLetterItem,
    DeadLetterListResponse, DeadLetterReplayResponse, InterruptDetailResponse,
    InterruptListResponse, JobDetailResponse, JobHistoryItem, JobHistoryResponse, JobStateResponse,
    JobTimelineItem, JobTimelineResponse, ListAuditLogsQuery, ListDeadLettersQuery,
    ListInterruptsQuery, ListJobsQuery, ListJobsResponse, RejectInterruptRequest, ReplayJobRequest,
    ResumeInterruptRequest, ResumeJobRequest, RetryPolicyRequest, RunJobRequest, RunJobResponse,
    TimelineExportResponse, TimeoutPolicyRequest, TraceContextResponse, WorkerAckRequest,
    WorkerAckResponse, WorkerExtendLeaseRequest, WorkerHeartbeatRequest, WorkerLeaseResponse,
    WorkerPollRequest, WorkerPollResponse, WorkerReportStepRequest, RUNTIME_API_CONTRACT_DOC_PATH,
};
pub use oris_execution_runtime::{
    AttemptDispatchRecord, AttemptExecutionStatus, InterruptRecord, LeaseConfig, LeaseManager,
    LeaseRecord, LeaseTickResult, RepositoryLeaseManager, RunRecord, RunRuntimeStatus,
    RuntimeRepository, SchedulerDecision, SkeletonScheduler,
};
#[cfg(all(feature = "execution-server", feature = "sqlite-persistence"))]
pub use oris_execution_runtime::{
    IdempotencyRecord, SqliteIdempotencyStore, SqliteRuntimeRepository,
};
#[cfg(feature = "sqlite-persistence")]
pub use oris_execution_runtime::{RuntimeStorageBackend, RuntimeStorageConfig};

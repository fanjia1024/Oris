#![cfg(feature = "execution-server")]

//! Compatibility smoke tests for legacy execution server paths.

fn assert_type<T>() {}

#[test]
#[allow(deprecated)]
fn deprecated_execution_server_paths_still_resolve() {
    let _ = oris_runtime::execution_server::build_router;
    let _ = oris_runtime::execution_runtime::build_router;
    let _ = oris_runtime::kernel::build_router;
    let _ = oris_runtime::kernel::runtime::build_router;

    assert_type::<oris_runtime::execution_server::ExecutionApiState>();
    assert_type::<oris_runtime::execution_runtime::ExecutionApiState>();
    assert_type::<oris_runtime::kernel::ExecutionApiState>();
    assert_type::<oris_runtime::kernel::runtime::ExecutionApiState>();

    let _ = oris_runtime::execution_server::ApiRole::Admin;
    let _ = oris_runtime::execution_runtime::ApiRole::Operator;
    let _ = oris_runtime::kernel::ApiRole::Worker;
    let _ = oris_runtime::kernel::runtime::ApiRole::Admin;
}

#[cfg(feature = "sqlite-persistence")]
#[test]
#[allow(deprecated)]
fn deprecated_benchmark_paths_still_resolve() {
    let _ = oris_runtime::execution_server::run_runtime_benchmark_suite;
    let _ = oris_runtime::execution_runtime::run_runtime_benchmark_suite;
    let _ = oris_runtime::kernel::run_runtime_benchmark_suite;
    let _ = oris_runtime::kernel::runtime::run_runtime_benchmark_suite;

    let _ = oris_runtime::execution_server::RUNTIME_BENCHMARK_BASELINE_DOC_PATH;
    let _ = oris_runtime::execution_runtime::RUNTIME_BENCHMARK_BASELINE_DOC_PATH;
    let _ = oris_runtime::kernel::RUNTIME_BENCHMARK_BASELINE_DOC_PATH;
    let _ = oris_runtime::kernel::runtime::RUNTIME_BENCHMARK_BASELINE_DOC_PATH;

    assert_type::<oris_runtime::execution_server::RuntimeBenchmarkSuiteReport>();
    assert_type::<oris_runtime::execution_runtime::RuntimeBenchmarkSuiteReport>();
    assert_type::<oris_runtime::kernel::RuntimeBenchmarkSuiteReport>();
    assert_type::<oris_runtime::kernel::runtime::RuntimeBenchmarkSuiteReport>();
}

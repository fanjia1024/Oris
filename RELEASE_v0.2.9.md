# v0.2.9 - Finalize Lease-Based Execution (K5)

Minor release for `oris-runtime` that formalizes single-owner execution via WorkerLease.

## What's in this release

### K5 - Distributed Execution
- `#59`: **WorkerLease**: wraps [LeaseRecord] to enforce single-owner execution; [WorkerLease::verify_owner], [WorkerLease::is_expired], [WorkerLease::check_execution_allowed] for pre-execution checks. Lease expiry and recovery remain in [LeaseManager::tick] (expire + requeue); replay-restart is re-dispatch after requeue.

## Validation

- cargo fmt --all -- --check
- cargo test -p oris-runtime --lib
- cargo build -p oris-runtime --release
- cargo publish -p oris-runtime --all-features --dry-run --registry crates-io

## Links

- Crate: https://crates.io/crates/oris-runtime
- Docs: https://docs.rs/oris-runtime
- Repo: https://github.com/Colin4k1024/Oris

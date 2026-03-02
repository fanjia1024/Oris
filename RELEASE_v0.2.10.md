# v0.2.10 - Zero-Data-Loss Failure Recovery Loop (K5)

Minor release adding crash recovery pipeline types and helpers.

## What's in this release

### K5 - Crash recovery
- `#60`: **CrashRecoveryPipeline** and **RecoveryStep**: explicit pipeline steps (LeaseExpired → CheckpointReload → Replay → ReadyForDispatch). **RecoveryContext** holds `attempt_id` and `run_id` for replay. Integrates with existing [LeaseManager::tick] / [RuntimeRepository::expire_leases_and_requeue]; no schema changes.

## Validation

- cargo fmt --all
- cargo test -p oris-runtime --lib
- cargo build -p oris-runtime --release
- cargo publish -p oris-runtime --all-features --dry-run --registry crates-io

## Links

- Crate: https://crates.io/crates/oris-runtime
- Docs: https://docs.rs/oris-runtime
- Repo: https://github.com/Colin4k1024/Oris

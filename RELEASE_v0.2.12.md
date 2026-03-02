# v0.2.12 - Safe Backpressure & Kernel Observability (K5)

Minor release adding rejection reasons and observability types for scheduling and telemetry.

## What's in this release

### K5 - Backpressure & observability
- `#62`: **RejectionReason** enum: `TenantLimit`, `CapacityLimit`, `Other` for clear rejections in scheduler/API. **KernelObservability** struct: optional `reasoning_timeline`, `lease_graph`, `replay_cost`, `interrupt_latency_ms` for future metrics/tracing; no built-in collection.

## Validation

- cargo fmt --all
- cargo test -p oris-runtime --lib
- cargo build -p oris-runtime --release
- cargo publish -p oris-runtime --all-features --dry-run --registry crates-io

## Links

- Crate: https://crates.io/crates/oris-runtime
- Docs: https://docs.rs/oris-runtime
- Repo: https://github.com/Colin4k1024/Oris

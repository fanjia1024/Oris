# v0.2.3 - Oris 2.0 Kernel: Replay-Based Resume Semantics

Patch release for `oris-runtime` that implements replay-based resume semantics.

## What's in this release

### K3 - Interrupt Kernel
- `#53`: ReplayResume - enforces Replay + Inject Decision semantics for idempotent resumes. ResumeDecision struct and ResumeResult with events_replayed and idempotent flag.

## Validation

- cargo fmt --all -- --check
- cargo test -p oris-runtime --lib kernel::
- cargo build -p oris-runtime --release
- cargo publish -p oris-runtime --all-features --dry-run --registry crates-io

## Links

- Crate: https://crates.io/crates/oris-runtime
- Docs: https://docs.rs/oris-runtime
- Repo: https://github.com/Colin4k1024/Oris

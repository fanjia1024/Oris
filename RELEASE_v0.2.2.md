# v0.2.2 - Oris 2.0 Kernel: Execution Suspension State Machine

Patch release for `oris-runtime` that implements the execution suspension state machine.

## What's in this release

### K3 - Interrupt Kernel
- `#52`: ExecutionSuspensionState - state transitions Running -> Suspended -> WaitingInput with safe worker teardown semantics.

## Validation

- cargo fmt --all -- --check
- cargo test -p oris-runtime --lib kernel::
- cargo build -p oris-runtime --release
- cargo publish -p oris-runtime --all-features --dry-run --registry crates-io

## Links

- Crate: https://crates.io/crates/oris-runtime
- Docs: https://docs.rs/oris-runtime
- Repo: https://github.com/Colin4k1024/Oris

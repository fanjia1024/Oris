# v0.2.4 - Oris 2.0 Kernel: Unified Interrupt Routing

Patch release for `oris-runtime` that implements the unified interrupt routing layer.

## What's in this release

### K3 - Interrupt Kernel
- `#54`: InterruptResolver trait with async resolve(interrupt) -> Value. UnifiedInterruptResolver routes UI, agents, policy engines, and API interrupts through source-specific handlers.

## Validation

- cargo fmt --all -- --check
- cargo test -p oris-runtime --lib kernel ::
- cargo build -p oris-runtime --release
- cargo publish -p oris-runtime --all-features --dry-run --registry crates-io

## Links

- Crate: https://crates.io/crates/oris-runtime
- Docs: https://docs.rs/oris-runtime
- Repo: https://github.com/Colin4k1024/Oris

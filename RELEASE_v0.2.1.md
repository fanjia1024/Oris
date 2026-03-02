# v0.2.1 - Oris 2.0 Kernel: Interrupt Kernel

Patch release for `oris-runtime` that implements the first issue of the K3 Interrupt Kernel epic.

## What's in this release

### K3 - Interrupt Kernel
- `#51`: Interrupt struct - standardized representation (`id`, `thread_id`, `kind`, `payload_schema`, `created_at`, `step_id`) with `InterruptStore` trait and `InMemoryInterruptStore` implementation.

## Validation

- cargo fmt --all -- --check
- cargo test -p oris-runtime --lib kernel::
- cargo build -p oris-runtime --release
- cargo publish -p oris-runtime --all-features --dry-run --registry crates-io

## Links

- Crate: https://crates.io/crates/oris-runtime
- Docs: https://docs.rs/oris-runtime
- Repo: https://github.com/Colin4k1024/Oris

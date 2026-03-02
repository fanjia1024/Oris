# v0.2.11 - Context-Aware Scheduler Kernel (K5)

Minor release adding dispatch context for tenant/priority/plugin/worker routing.

## What's in this release

### K5 - Scheduler context
- `#61`: **DispatchContext**: optional `tenant_id`, `priority`, `plugin_requirements`, `worker_capabilities`. **SkeletonScheduler::dispatch_one_with_context(worker_id, context)** added; context is passed through for future filtering/sorting; [RuntimeRepository] unchanged.

## Validation

- cargo fmt --all
- cargo test -p oris-runtime --lib
- cargo build -p oris-runtime --release
- cargo publish -p oris-runtime --all-features --dry-run --registry crates-io

## Links

- Crate: https://crates.io/crates/oris-runtime
- Docs: https://docs.rs/oris-runtime
- Repo: https://github.com/Colin4k1024/Oris

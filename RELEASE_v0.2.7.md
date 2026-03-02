# v0.2.7 - Plugin Execution Sandbox (K4)

Minor release for `oris-runtime` that adds execution mode routing for plugin isolation.

## What's in this release

### K4 - Plugin Runtime
- `#57`: **PluginExecutionMode** enum: `InProcess`, `IsolatedProcess`, `Remote` for kernel routing.
- **route_to_execution_mode(meta)**: selects mode from [PluginMetadata]; plugins with side effects route to `IsolatedProcess`, pure to `InProcess`. WASM mode left for future work.

## Validation

- cargo fmt --all -- --check
- cargo test -p oris-runtime --lib
- cargo build -p oris-runtime --release
- cargo publish -p oris-runtime --all-features --dry-run --registry crates-io

## Links

- Crate: https://crates.io/crates/oris-runtime
- Docs: https://docs.rs/oris-runtime
- Repo: https://github.com/Colin4k1024/Oris

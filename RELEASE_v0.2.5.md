# v0.2.5 - Plugin Categories and Interfaces (K4)

Minor release for `oris-runtime` that defines and enforces the plugin categories the kernel recognizes and can load.

## What's in this release

### K4 - Plugin Runtime
- `#55`: **Plugin categories**: `PluginCategory` enum (Node, Tool, Memory, LLMAdapter, Scheduler) for kernel plugin discovery and dispatch.
- **Plugin interfaces**: `ToolPlugin`, `MemoryPlugin`, `LLMAdapter`, and `SchedulerPlugin` traits with `plugin_type()` and config-based factory methods where applicable. `NodePlugin` (existing in `graph::plugin`) is documented as the Node category.
- New `plugins` module with `PluginError` and unit tests for `PluginCategory`.

## Validation

- cargo fmt --all -- --check
- cargo test -p oris-runtime --lib
- cargo build -p oris-runtime --release
- cargo publish -p oris-runtime --all-features --dry-run

## Links

- Crate: https://crates.io/crates/oris-runtime
- Docs: https://docs.rs/oris-runtime
- Repo: https://github.com/Colin4k1024/Oris

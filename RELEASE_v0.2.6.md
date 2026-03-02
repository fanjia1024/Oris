# v0.2.6 - Plugin Determinism Declarations (K4)

Minor release for `oris-runtime` that requires plugins to declare behavioral boundaries for kernel enforcement.

## What's in this release

### K4 - Plugin Runtime
- `#56`: **PluginMetadata**: struct with `deterministic`, `side_effects`, `replay_safe` (all bools); `PluginMetadata::conservative()` and `PluginMetadata::pure()` constructors; serde support.
- **HasPluginMetadata** trait with default `plugin_metadata() -> PluginMetadata`. All plugin interfaces (ToolPlugin, MemoryPlugin, LLMAdapter, SchedulerPlugin) now require [HasPluginMetadata]; NodePlugin in graph has `plugin_metadata()` with default.
- **Kernel enforcement helpers**: `allow_in_replay(meta)` and `requires_sandbox(meta)` for replay substitution and sandbox routing decisions.

## Validation

- cargo fmt --all -- --check
- cargo test -p oris-runtime --lib
- cargo build -p oris-runtime --release
- cargo publish -p oris-runtime --all-features --dry-run --registry crates-io --allow-dirty

## Links

- Crate: https://crates.io/crates/oris-runtime
- Docs: https://docs.rs/oris-runtime
- Repo: https://github.com/Colin4k1024/Oris

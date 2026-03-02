# v0.2.8 - Plugin Version Negotiation & Dynamic Registry (K4)

Minor release for `oris-runtime` that adds plugin load validation and dynamic registry support.

## What's in this release

### K4 - Plugin Runtime
- `#58`: **PluginCompatibility**: `plugin_api_version`, `kernel_compat`, `schema_hash` with serde; **validate_plugin_compatibility(compat, kernel_version)** for strict validation on load (exact or `>=` kernel_compat).
- **NodePluginRegistry::unregister_plugin(plugin_type)**: dynamic unloading; returns true if removed. Enables hot-loading workflow (register/unregister at runtime).

## Validation

- cargo fmt --all -- --check
- cargo test -p oris-runtime --lib
- cargo build -p oris-runtime --release
- cargo publish -p oris-runtime --all-features --dry-run --registry crates-io

## Links

- Crate: https://crates.io/crates/oris-runtime
- Docs: https://docs.rs/oris-runtime
- Repo: https://github.com/Colin4k1024/Oris

# EvoKernel Design Mirrors

Local mirrors of the Notion design pages under:
https://www.notion.so/317e8a70eec5809c85e1f52aa03870e4

Last synced: March 2, 2026

Files:

- `architecture.md`
- `evolution.md`
- `governor.md`
- `network.md`
- `economics.md`
- `kernel.md`
- `implementation-roadmap.md`
- `bootstrap.md`
- `agent.md`
- `devloop.md`
- `spec.md`
- `vision.md`
- `founding-paper.md`

The top-level overview remains in `../evokernel-v0.1.md`.

## Implementation Status Matrix

| Layer | Local crate/module | Status | Gate |
| --- | --- | --- | --- |
| Kernel | `crates/oris-kernel` | implemented baseline | default |
| Evolution | `crates/oris-evolution` | implemented baseline, lifecycle extended | `evolution-experimental` |
| Sandbox | `crates/oris-sandbox` | implemented baseline, blast radius helper added | `evolution-experimental` |
| EvoKernel | `crates/oris-evokernel` | implemented baseline, governor-aware capture added | `evolution-experimental` |
| Governor | `crates/oris-governor` | experimental scaffold with default policy | `governor-experimental` |
| Evolution Network | `crates/oris-evolution-network` | experimental protocol scaffold | `evolution-network-experimental` |
| Economics | `crates/oris-economics` | experimental ledger scaffold | `economics-experimental` |
| Spec | `crates/oris-spec` | experimental YAML compiler scaffold | `spec-experimental` |
| Agent Contract | `crates/oris-agent-contract` | experimental proposal contract scaffold | `agent-contract-experimental` |
| Full stack | `crates/oris-runtime` re-exports | experimental aggregate | `full-evolution-experimental` |

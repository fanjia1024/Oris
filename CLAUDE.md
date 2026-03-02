# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Oris is a **durable execution runtime for reasoning-driven software systems** written in Rust. It provides stateful graphs, agents, tools, and multi-step execution capabilities—similar to Temporal or Ray but designed for AI/ML workloads.

The main crate is `oris-runtime` in `crates/oris-runtime/`. This is a Cargo workspace with multiple examples in `examples/`.

## Common Commands

### Building
```bash
# Build the workspace
cargo build

# Build with all features (required for full validation)
cargo build --all --release --all-features
```

### Testing
```bash
# Run all tests
cargo test --release --all-features

# Run targeted tests for a specific module
cargo test -p oris-runtime <test_name_or_module>

# Format check
cargo fmt --all -- --check
```

### Running Examples
```bash
# Run the execution server (HTTP API for jobs)
cargo run -p oris-runtime --example execution_server --features "sqlite-persistence,execution-server"

# Run other examples (list available in crates/oris-runtime/examples/)
cargo run -p oris-runtime --example <example_name> --features "..."
```

### Linting
```bash
cargo fmt --all
```

## Architecture

### Core Modules (in `crates/oris-runtime/src/`)

| Directory | Purpose |
|-----------|---------|
| **graph/** | State graphs, execution engine, persistence/checkpointing, interrupts, streaming |
| **agent/** | Agent loop (conversational, unified, Deep agent), tools, middleware, multi-agent patterns |
| **kernel/** | Kernel API (2.0) - event-first execution, actions, replay, determinism verification |
| **tools/** | Tool trait and built-in tools (command, search, SQL, scraper, browser-use, etc.) |
| **llm/** | LLM implementations (OpenAI, Claude, Ollama, Mistral, Gemini, Bedrock) |
| **memory/** | Memory implementations (simple, conversational, long-term) |
| **vectorstore/** | Vector stores (pgvector, Qdrant, SQLite, SurrealDB, etc.) |
| **document_loaders/** | PDF, HTML, CSV, Git, S3 loaders |
| **rag/** | RAG implementations (agentic, hybrid, two-step) |

### Key Abstractions

**StateGraph** (`graph/graph.rs`): Builder for creating stateful graphs with nodes, edges, and conditional routing.

**CompiledGraph** (`graph/compiled.rs`): Executable representation of a compiled graph with `invoke()`, `stream()`, `step_once()` methods.

**Checkpointer** (`graph/persistence/checkpointer.rs`): Trait for checkpointing state. Implementations: `InMemorySaver`, `SqliteCheckpointer`.

**Agent** (`agent/agent.rs`): Trait for building agents with `plan()` and `get_tools()` methods.

**Tool** (`tools/tool.rs`): Trait for implementing tools that agents can call.

### Stable API (0.1.x)

The public stable surface for building on Oris:
- `oris_runtime::graph` — State graphs, execution, persistence, interrupts, trace
- `oris_runtime::agent` — Agent loop, tools, Deep Agent
- `oris_runtime::tools` — Tool trait and built-in tools

## Development Workflow

This repository follows an **issue-driven GitHub workflow**. When working on maintenance or features:

1. **Preflight**: Check `git status --short --branch`, verify `gh auth status`, list open issues with `gh issue list --state open --limit 20`

2. **Issue Selection**: Use `gh issue view <number>` to read issue details. Apply selection order from `skills/oris-maintainer/references/issue-selection.md`

3. **Implementation**: Make narrow, issue-scoped changes. Avoid opportunistic refactors.

4. **Validation**: Run in order:
   - `cargo fmt --all`
   - `cargo test -p oris-runtime <targeted_test>`
   - Full validation: `cargo fmt --all -- --check && cargo build --all --release --all-features && cargo test --release --all-features`

5. **Release**: Use `cargo publish -p oris-runtime --all-features --dry-run` before real publish

See `skills/oris-maintainer/SKILL.md` and `skills/oris-maintainer/references/command-checklist.md` for the full workflow.

## Feature Flags

Key features for common use cases:
- `sqlite-persistence` — SQLite checkpointing for durable execution
- `postgres` / `kernel-postgres` — PostgreSQL backend
- `ollama` — Local LLM support
- `execution-server` — HTTP API server
- `surrealdb`, `qdrant`, `chroma` — Vector stores

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `OPENAI_API_KEY` | OpenAI API key |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `OLLAMA_HOST` | Ollama host (default `http://localhost:11434`) |
| `ORIS_SERVER_ADDR` | Execution server address (default `127.0.0.1:8080`) |
| `ORIS_SQLITE_DB` | SQLite database path |
| `ORIS_RUNTIME_BACKEND` | Runtime backend (`sqlite` or `postgres`) |

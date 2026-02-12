# Oris

**A programmable execution runtime for AI agents.**

[![Latest Version](https://img.shields.io/crates/v/oris.svg)](https://crates.io/crates/oris)
[![docs.rs](https://img.shields.io/docsrs/oris)](https://docs.rs/oris)

Oris is not a prompt framework.

It is a runtime layer that lets software systems *execute reasoning*, not just generate text.

Modern LLM applications are no longer single requests.
They are long-running processes: planning, tool use, memory updates, retries, and human approval.

Today this logic lives in ad-hoc code, background jobs, and fragile queues.

Oris turns that into a **first-class execution system**.

---

## What Oris actually provides

Oris is closer to **Temporal / Ray** than to a chat SDK.

It provides a persistent execution environment for agentic workloads:

* Stateful execution graphs
* Durable checkpoints
* Interruptible runs (human-in-the-loop)
* Tool calling as system actions
* Multi-step planning loops
* Deterministic replay
* Recovery after crash or deploy

Instead of writing:

> "call LLM → parse → call tool → retry → store memory → schedule task"

You define an execution graph, and the runtime runs it.

---

## Why this exists

LLMs changed backend architecture.

We are moving from:

request → response

to:

goal → process → decisions → actions → memory → continuation

This is not an API problem anymore.

It is an **execution problem**.

Oris is an attempt to build the execution layer for software that *thinks before it acts*.

---

## Mental model

If databases manage data
and message queues manage communication

**Oris manages reasoning processes.**

---

## What you can build with it

* autonomous coding agents
* long-running research agents
* human-approval workflows
* retrieval-augmented systems
* operational copilots
* AI operations pipelines

---

## Status

Early but functional.
The runtime, graph execution, and agent loop are implemented and usable today.

---

## Quick start (30 seconds)

Add the crate and set your API key:

```bash
cargo add oris
export OPENAI_API_KEY="your-key"
```

Minimal LLM call:

```rust
use oris::{language_models::llm::LLM, llm::openai::OpenAI};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let llm = OpenAI::default();
    let response = llm.invoke("What is Rust?").await?;
    println!("{}", response);
    Ok(())
}
```

Hello-world state graph (no API key needed):

```rust
use oris::graph::{function_node, MessagesState, StateGraph, END, START};
use oris::schemas::messages::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mock_llm = function_node("mock_llm", |_state: &MessagesState| async move {
        use std::collections::HashMap;
        let mut update = HashMap::new();
        update.insert(
            "messages".to_string(),
            serde_json::to_value(vec![Message::new_ai_message("hello world")])?,
        );
        Ok(update)
    });

    let mut graph = StateGraph::<MessagesState>::new();
    graph.add_node("mock_llm", mock_llm)?;
    graph.add_edge(START, "mock_llm");
    graph.add_edge("mock_llm", END);

    let compiled = graph.compile()?;
    let initial_state = MessagesState::with_messages(vec![Message::new_human_message("hi!")]);
    let _final_state = compiled.invoke(initial_state).await?;
    Ok(())
}
```

## Architecture

```mermaid
flowchart TB
  User[User Request]
  Runtime[Runtime: Graph or Agent]
  Tools[Tools]
  LLM[LLM Provider]
  Memory[Memory or State]
  User --> Runtime
  Runtime --> Tools
  Runtime --> LLM
  Runtime --> Memory
  Tools --> Runtime
  LLM --> Runtime
  Memory --> Runtime
```

## Key concepts

- **State graphs** — Define workflows as directed graphs; run, stream, and optionally persist state (e.g. SQLite or in-memory).
- **Agents and tools** — Give agents tools (search, filesystem, custom); use multi-agent routers and subagents.
- **Persistence and interrupts** — Checkpoint state, resume runs, and pause for human approval or review.

See the [examples](examples/) directory for runnable code.

## Install and config

```bash
cargo add oris
# With a vector store (e.g. PostgreSQL):
cargo add oris --features postgres
# With Ollama (local):
cargo add oris --features ollama
```

Common environment variables:

| Provider   | Variable           |
|-----------|--------------------|
| OpenAI    | `OPENAI_API_KEY`   |
| Anthropic | `ANTHROPIC_API_KEY` |
| Ollama    | `OLLAMA_HOST` (optional, default `http://localhost:11434`) |

## Examples and docs

- [Hello World graph](examples/graph_hello_world.rs)
- [Agent with tools](examples/agent.rs)
- [Streaming](examples/graph_streaming.rs)
- [Persistence](examples/graph_persistence_basic.rs)
- [Deep agent (planning + filesystem)](examples/deep_agent_basic.rs)

[API documentation](https://docs.rs/oris) · [Examples directory](examples/)

## License and attribution

MIT. This project includes code derived from [langchain-rust](https://github.com/langchain-ai/langchain-rust); see [LICENSE](LICENSE).

## Links

- [Crates.io](https://crates.io/crates/oris)
- [GitHub](https://github.com/fanjia1024/oris)
- [docs.rs](https://docs.rs/oris)

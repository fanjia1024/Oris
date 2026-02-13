# Durable execution: crash recovery and replay

Oris supports persisting graph execution state and resuming after a process restart or from a specific checkpoint (replay).

## Crash recovery (resume from latest checkpoint)

After a process crash (or restart), you can continue a run using only the same `thread_id`. No `checkpoint_id` is required: the runtime loads the **latest** checkpoint for that thread and continues from there.

- Use **`invoke_with_config_and_mode`** with `initial_state: None` and config containing only `thread_id`:

  ```rust
  use oris_runtime::graph::{RunnableConfig, DurabilityMode};

  let config = RunnableConfig::with_thread_id("my-job-123");
  let final_state = compiled
      .invoke_with_config_and_mode(None, &config, DurabilityMode::Sync)
      .await?;
  ```

- Or use **`invoke_with_config`** the same way (it delegates to the interrupt path and also supports "resume from latest"):

  ```rust
  let final_state = compiled.invoke_with_config(None, &config).await?;
  ```

If no checkpoint exists for that `thread_id`, you get a clear error: `"No checkpoint found for thread_id: <id>"`.

## Replay from a specific checkpoint (time-travel)

To re-run from a **specific** checkpoint (e.g. for debugging or audit), pass that checkpoint’s id in the config:

- **`RunnableConfig::with_checkpoint(thread_id, checkpoint_id)`**
- Then call **`invoke_with_config(None, &config)`** or **`invoke_with_config_and_mode(None, &config, mode)`**.

Example:

```rust
let config = RunnableConfig::with_checkpoint("my-job-123", "checkpoint-abc");
let state = compiled
    .invoke_with_config_and_mode(None, &config, DurabilityMode::Sync)
    .await?;
```

To obtain checkpoint ids, use **`get_state_history(&config)`** on the compiled graph; it returns all checkpoints for the thread in chronological order.

## Operator API summary

| Action | API |
|--------|-----|
| Start or continue a run | `invoke_with_config(Some(initial_state), &config)` or `invoke_with_config_and_mode(...)` |
| Resume after crash | `invoke_with_config(None, &RunnableConfig::with_thread_id(id))` or `invoke_with_config_and_mode(None, &config, mode)` |
| Replay from checkpoint | `invoke_with_config(None, &RunnableConfig::with_checkpoint(thread_id, checkpoint_id))` |
| List checkpoints | `compiled.get_state_history(&config).await?` |
| Get latest state | `compiled.get_state(&config).await?` |

## Minimal CLI example

The **`cli_durable_job`** example (run with `--features sqlite-persistence`) provides subcommands that wrap the operator API:

- **`run --thread-id <id>`** — Start a run (research → approval interrupt) with SQLite persistence.
- **`list --thread-id <id>`** — List all checkpoints for the thread.
- **`resume --thread-id <id> [--checkpoint-id <id>]`** — Resume from latest or from a specific checkpoint.

Run: `cargo run -p oris-runtime --example cli_durable_job --features sqlite-persistence -- run --thread-id my-job`

## Execution trace

When you use **`invoke_with_config_interrupt`**, the returned **`InvokeResult`** includes a **`trace`** field: a sequence of `TraceEvent` values for debugging and audit:

- **`StepCompleted { node }`** — A node finished successfully.
- **`InterruptReached { value }`** — Execution paused at an interrupt (e.g. human-in-the-loop).
- **`ResumeReceived { value }`** — Execution was resumed with a value after an interrupt.

Example: inspect `result.trace` after a run to see the order of steps and interrupt/resume events.

See [Oris 2.0 Strategy & Evolution Blueprint](ORIS_2.0_STRATEGY.md) for Phase 1 acceptance criteria and roadmap.

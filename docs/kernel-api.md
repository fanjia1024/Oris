# Oris Kernel API (2.0)

This document defines the **minimal complete set** of interfaces that constitute the Oris execution kernel. It aligns with the axioms in [ORIS_2.0_STRATEGY.md](ORIS_2.0_STRATEGY.md) and ensures the runtime satisfies four properties:

1. **Run** — Long-running execution with identity
2. **Event Log** — Source of truth (auditable)
3. **Replay** — Replayable and recoverable
4. **Action** — Single channel for tools and external world (governable)

Graph, Agent, and Planner sit **above** the kernel; they compile down to the kernel’s step and action model.

---

## Axioms (A1–A9) and kernel mapping

- **A1–A2** (reasoning state, interrupt semantics) → Run identity, Event log (Interrupted/Resumed), StepFn returning Next::Interrupt.
- **A3** (replay, effect boundaries) → EventStore as source of truth; Reducer projects events to state; replay uses only stored events (no external action execution).
- **A4** (unit of execution = step) → StepFn and Next (Emit / Do(Action) / Interrupt / Complete).
- **A5** (state = graph state + messages, versioned) → State with `version()`; Snapshot carries `at_seq`.
- **A6** (tool/LLM as system action) → Action enum and ActionExecutor; results only as events (ActionSucceeded/ActionFailed).
- **A9** (governance) → Policy trait (authorize, retry_strategy, budget).

---

## 1. Run identity

- **RunId** — Identifies a long-running run (maps to current `thread_id`).
- **StepId** — Identifies a step (node + attempt).
- **Seq** — Monotonically increasing event sequence number per run. EventStore assigns or requires seq for ordering.

---

## 2. EventStore (source of truth)

**Constraints (must be enforced in implementation and tests):**

- **append**: Either all events in the batch succeed or none do (atomicity).
- Every event has a **seq** (assigned by the store or by the caller in a defined way).
- **scan(run_id, from)** returns events in **ascending seq order**.

Events are the only source of truth; state is derived by reduction.

---

## 3. SnapshotStore (optimization layer)

- Snapshots are an **optimization**, not the source of truth.
- Every **Snapshot** must include **at_seq: Seq** — the seq up to which state has been projected. This is required for correct recovery and replay (load snapshot, then replay events after at_seq).
- **Implementations**: `kernel::InMemorySnapshotStore<S>` stores one snapshot per run. Graph `StateSnapshot` has optional `at_seq`; when the graph uses an event store, checkpoints saved at interrupt carry `at_seq` from the store head.

---

## 4. Action results only as events

- **ActionExecutor** must not mutate state directly.
- All external effects (tool call, LLM call, etc.) are recorded as:
  - **ActionRequested** (before execution)
  - **ActionSucceeded { output }** or **ActionFailed { error }** (after execution)
- Replay uses these stored results and does **not** call ActionExecutor.

---

## 5. Policy (governance)

- The kernel must have a **Policy** layer (even if a minimal implementation).
- **authorize(run_id, action, ctx)** — Decide whether the action is allowed.
- **retry_strategy(error, action)** — Decide retry/backoff/fail.
- **budget()** (optional) — Cost/usage limits.

Without this, Oris remains a demo framework; with it, enterprise deployment is possible.

---

## 6. Module layout (in-repo)

- **`oris_runtime::kernel`** — All kernel types and traits (identity, event, snapshot, state, reducer, action, step, policy, driver).
- **graph / agent / tools** — Unchanged stable API; later, Graph/Agent compile to StepFn, tools implement ActionExecutor.

---

## 7. Relation to current APIs

- **Checkpoint / thread_id** — Today’s checkpointer is snapshot-only; kernel adds EventStore and Snapshot with `at_seq`. Existing `thread_id` ↔ RunId.
- **Interrupt / resume** — Map to events Interrupted / Resumed; StepFn returns Next::Interrupt; driver exposes resume(run_id, signal).
- **RunStatus** — Standardized status: `Completed`, `Blocked(BlockedInfo)` (interrupt or WaitSignal), `Running` (optional), `Failed { recoverable: bool }` (optional).
- **Trace (TraceEvent)** — Current trace events (StepCompleted, InterruptReached, ResumeReceived) are a subset of kernel Event types; kernel Event covers also StateUpdated, ActionRequested/Succeeded/Failed, Completed.

---

## 8. Event-first pilot path (Phase 2)

One execution path satisfies “write event → then execute → then write result” (A2/A3):

- **Path**: Graph execution with interrupts: `invoke_with_config_interrupt` when the graph is compiled with `.with_event_store(store)`.
- **RunId**: `thread_id` from checkpoint config.
- **Events written**: Before execution, any resume values produce `Resumed` events. During execution, per node: `ActionRequested` (action_id = run_id-node, payload = state) → node runs → `ActionSucceeded` (output = state update) or `ActionFailed` (on error or on interrupt); then `StateUpdated`; on interrupt, `ActionFailed { error: "Interrupt" }` then `Interrupted { value }` so every ActionRequested is paired; on normal end, `Completed`.
- **EventStore**: Use `kernel::InMemoryEventStore` (or any `EventStore` implementation). When event_store is set, checkpoints saved at interrupt include `at_seq` (from EventStore head).

---

## 9. Replay-only mode (Phase 3)

- **`Kernel::replay(run_id, initial_state)`**: Scans all events for the run (from seq 1), applies each with the Reducer in order. Does **not** call ActionExecutor; any ActionRequested is satisfied by the following ActionSucceeded/ActionFailed already in the log (reducer applies them).
- **`Kernel::replay_from_snapshot(run_id, initial_state)`**: If a snapshot exists for the run, starts from `snap.state` and applies only events with seq > snap.at_seq; otherwise same as replay. Rebuild semantics: state = snap + events(from=at_seq+1).
- **Use case**: Reproducible state from history, audit, and recovery without re-executing external actions (A3).
- **Tests**: `test_replay_reproduces_state` (graph + replay state match); `replay_no_side_effects` (executor 0 calls); `replay_state_equivalence` (same log → same state); `replay_from_snapshot_applies_tail_only`.

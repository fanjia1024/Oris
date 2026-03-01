# Oris + Rust Ecosystem Integration Guide

This guide explains how to integrate Oris into common Rust stacks and how downstream users can adopt it safely.

## Integration principles

- Keep Oris execution contracts explicit (`thread_id`, idempotency keys, resume semantics).
- Treat persistence and replay as first-class concerns, not optional afterthoughts.
- Separate control-plane APIs (operators) from business APIs.
- Use structured logging and tracing from day one.

## Recommended integration patterns

## 1) Embedded service (Axum today)

- Use Oris runtime APIs behind authenticated service endpoints.
- Expose a health endpoint and keep Oris routes under a versioned prefix.
- Start from `examples/oris_starter_axum` when your application owns the HTTP layer.

## 2) Standalone worker (pure Tokio)

- Use `examples/oris_worker_tokio` when an execution server already exists and this process should
  only poll, heartbeat, and ack.
- Avoid blocking calls in the async worker loop.
- Use graceful shutdown so active attempts can heartbeat or fail over cleanly.

## 3) Operator control plane (CLI)

- Use `examples/oris_operator_cli` when operators need direct access to `run/list/inspect/resume/replay/cancel`.
- Keep the CLI mapped one-to-one with execution server contracts to reduce incident-time ambiguity.
- Add auth headers and output modes before production rollout.

## 4) Persistence

- Local/dev:
  - `sqlite-persistence` for quick durable workflows.
- Production:
  - Use stable persistence strategy and plan migration/backup from the start.
  - Validate replay and recovery in CI.

## 5) Observability (tracing ecosystem)

- Use `tracing` and `tracing-subscriber` with request/run correlation IDs.
- Emit structured fields for:
  - `thread_id`/`run_id`
  - `attempt_id`
  - lease and interrupt identifiers
- Add metrics and traces before scaling worker concurrency.

## 6) Testing strategy

- Unit tests for reducers/state transitions.
- API tests for idempotency and conflict semantics.
- Crash/failover tests for checkpoint recovery and lease expiry.

## Adoption path for external users

1. Start with a single service and SQLite persistence.
2. Implement one business workflow graph end-to-end.
3. Add a standalone worker or operator client if you need a split deployment model.
4. Add interrupt + resume flow with operator visibility.
5. Add replay and failover validation in CI.
6. Harden auth/security and observability before production rollout.

## Reference entry points

- Runtime examples: `crates/oris-runtime/examples/`
- Starter project: `examples/oris_starter_axum`
- Standalone worker: `examples/oris_worker_tokio`
- Operator CLI: `examples/oris_operator_cli`
- Template matrix: `examples/templates/` (`cargo generate --path ... --name ...`)
- Production ops: `docs/production-operations-guide.md`
- Incident runbook: `docs/incident-response-runbook.md`
- Durable execution docs: `docs/durable-execution.md`
- Kernel contract docs: `docs/kernel-api.md`

## Common mistakes to avoid

- Treating `thread_id` as optional/non-stable.
- Missing idempotency for run and step reporting.
- Skipping replay/recovery tests until late stages.
- Coupling business API errors with operator control-plane behavior.

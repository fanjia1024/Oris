# Oris Worker (Pure Tokio)

Use this example when the execution server is already hosted elsewhere and you only need a
standalone worker process.

## Choose this path when

- Your service team already runs Oris control-plane APIs in another process.
- You want to scale workers independently from the HTTP/API surface.
- You need the smallest possible deployment unit for polling, heartbeats, and ack.

## What this example demonstrates

- Poll loop against `/v1/workers/poll`
- Lease heartbeat updates
- Attempt acknowledgement (`completed` by default in the sample)
- Backpressure/noop handling in a plain Tokio process

## Run

Start an execution server first (for example the Axum starter):

```bash
cargo run -p oris_starter_axum
```

In a second shell, run the worker:

```bash
cargo run -p oris_worker_tokio
```

Optional environment variables:

- `ORIS_SERVER_URL` (default: `http://127.0.0.1:8080`)
- `ORIS_WORKER_ID` (default: `tokio-worker-example-1`)
- `ORIS_MAX_ACTIVE_LEASES` (default: `1`)
- `ORIS_POLL_INTERVAL_SECONDS` (default: `2`)

## Next edits

1. Replace `execute_attempt` with real business execution.
2. Add `report-step` events for multi-stage tasks.
3. Add retry and failure classification before marking an attempt `failed`.

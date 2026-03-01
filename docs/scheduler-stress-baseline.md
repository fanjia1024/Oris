# Scheduler Stress Baseline

This document records the current baseline for the scheduler failover/conflict stress suite introduced for `SCH-06`.

## Scope

The suite injects three failure patterns against the SQLite-backed execution API:

- Lease ownership conflict via heartbeat from the wrong worker.
- Forced lease expiry followed by failover redispatch.
- Repeated dispatch/ack loops to capture steady-state throughput.

## Run command

```bash
./scripts/run_scheduler_stress_suite.sh
```

The script runs:

```bash
cargo test -p oris-runtime --features "sqlite-persistence,execution-server" \
  kernel::runtime::api_handlers::tests::scheduler_stress_ \
  -- --nocapture --test-threads=1
```

## Current baseline

Measured on March 1, 2026 from a local run of the suite:

| Metric | Baseline |
|--------|----------|
| Conflict rate | `100.00%` |
| Average recovery latency | `0.245 ms` |
| Max recovery latency | `0.288 ms` |
| Throughput | `2729.63 dispatches/sec` |
| Failover recovery success | `8/8` |

These values are a regression baseline, not a production SLA. CI assertions remain deliberately loose (`>= 99%` conflict detection, all injected failovers recovered, non-zero throughput) so the suite stays stable across different runners while still catching behavioral regressions.

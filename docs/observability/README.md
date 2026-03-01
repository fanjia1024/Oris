# Observability Assets

This folder contains the default observability assets for the runtime metrics exposed at `GET /metrics`.

## Included files

- `runtime-dashboard.json`: Grafana dashboard covering queue depth, backpressure, dispatch latency, recovery latency, terminal error rate, and lease conflict rate.
- `prometheus-alert-rules.yml`: Prometheus alert thresholds for elevated terminal error rate, high recovery latency, and sustained backpressure.
- `sample-runtime-workload.prom`: A sample scrape from the regression workload used to validate that the dashboard and alert rules reference real exported metrics.

## Validation

The repository regression `observability_assets_reference_metrics_present_in_sample_workload` checks that:

- every metric used by the dashboard exists in the sample workload scrape
- every metric used by the alert rules exists in the sample workload scrape

The runtime metrics endpoint regression `metrics_endpoint_is_scrape_ready_and_exposes_runtime_metrics` verifies that the live `/metrics` endpoint exports the same metric family names in Prometheus text format.

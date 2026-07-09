# Observability

## Logging

The app uses `tracing` with a non-blocking JSON writer:

```bash
RUST_LOG=debug cargo run -p liquidity-hub-app
```

Logs are written to `logs/liquidity-hub.jsonl`.

For production latency work:

- avoid logging every tick
- emit counters and histograms instead
- sample quote decisions
- log anomalous slow paths
- keep audit/event persistence separate from diagnostic logging

## OpenTelemetry

Recommended metrics:

- `liquidity_ticks_total`
- `liquidity_decisions_total`
- `liquidity_holds_total{reason}`
- `liquidity_decision_latency_ns`
- `chronicle_read_latency_ns`
- `chronicle_write_latency_ns`
- `risk_snapshot_version`
- `analytics_signal_version`

Recommended traces:

- service startup
- config reload
- Chronicle reconnect/reopen
- risk snapshot refresh
- slow-path decision above threshold

Do not create a trace span for every market tick in production. At high tick rates, tracing every event will distort the system you are trying to measure.

## Collector

For Kubernetes, run an OpenTelemetry Collector as a sidecar or daemonset and export to your backend of choice.

Local collector example:

```bash
docker run --rm -p 4317:4317 -p 4318:4318 \
  -v "$PWD/deploy/otel-collector.yaml:/etc/otelcol/config.yaml" \
  otel/opentelemetry-collector-contrib:latest
```


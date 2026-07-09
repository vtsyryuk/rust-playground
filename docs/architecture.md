# Architecture Notes

## Recommended Service Shape

Use a modular monolith inside one Rust process before splitting into many microservices. A few-microsecond latency budget is usually harmed by unnecessary network hops.

Recommended internal modules:

- `domain`: instruments, prices, quantities, risk, ticks, analytics, decisions
- `engine`: deterministic decision logic
- `risk_adapter`: subscribes to central risk book snapshots and deltas
- `market_data_adapter`: normalizes venue/feed ticks
- `analytics_adapter`: reads kdb/fair-value signals
- `transport`: Chronicle Queue input/output
- `observability`: logs, metrics, traces, health checks
- `replay`: deterministic event replay for testing incidents

## Event Model

Everything entering the engine should be an immutable event:

- `RiskAppetite`
- `AnalyticsSignal`
- `MarketTick`

Everything leaving should be explicit:

- `LiquidityDecision::Quote`
- `LiquidityDecision::Hold`

This makes testing and replay simple. You can persist input events, replay them into the same engine version, and compare decisions.

## Threading Model

For learning and early development:

- one async app process
- mock queues
- one engine instance

For low latency:

- one dedicated hot thread per shard or instrument group
- no async in the hot thread
- single-writer state ownership
- lock-free or Chronicle-backed ingress/egress
- async sidecar tasks for metrics, health checks, config, and logs

## Observability

Use three levels:

- counters: ticks processed, decisions emitted, holds by reason
- histograms: decision latency, queue read latency, queue write latency
- sampled traces: lifecycle, config reloads, slow-path anomalies

Do not trace every tick in production. Use sampling or anomaly-triggered trace capture.

## Deployment

For cloud development:

- container image
- Kubernetes Deployment
- OpenTelemetry Collector sidecar or daemonset
- logs to stdout or async file shipper

For production low latency:

- benchmark cloud and bare-metal separately
- prefer host networking where appropriate
- pin CPUs
- reserve cores for the process
- disable noisy neighbors
- test p99.9 and p99.99, not just averages


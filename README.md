# Rust Liquidity Hub Playground

This repository is a learning project for building a low-latency Rust microservice for an equities cash trading platform.

The service acts as a liquidity hub:

- receives risk appetite from a central risk book
- consumes market data ticks
- consumes analytics/fair-value signals from kdb or a mocked analytics feed
- makes quote/hold decisions
- publishes decisions through a Chronicle-style low-latency transport
- exposes an operational path for build, test, deploy, logging, and OpenTelemetry

The current code is intentionally small. The hot path is in `liquidity-hub-core`, with no async runtime, no network code, and no logging.

## Architecture

```mermaid
flowchart LR
    Risk["Central Risk Book"] --> RiskAdapter["Risk Adapter"]
    Kdb["kdb+/Analytics"] --> AnalyticsAdapter["Analytics Adapter"]
    Md["Market Data Feed"] --> MdAdapter["Market Data Adapter"]

    RiskAdapter --> Ingress["Chronicle Queue / Mock Queue"]
    AnalyticsAdapter --> Ingress
    MdAdapter --> Ingress

    Ingress --> Hub["Liquidity Hub Core"]
    Hub --> Decisions["Decision Queue"]
    Decisions --> Oms["OMS / Smart Order Router"]

    Hub --> Logs["Async JSON Logs"]
    Hub --> Otel["OpenTelemetry Metrics/Traces"]
```

## Design Principles

For a target of a few microseconds per decision, the architecture should separate the hot path from the operational shell.

The hot path should:

- use fixed-size integer prices and quantities, not floating point
- avoid allocation while processing each tick
- avoid logging per tick unless sampled or written to an async queue
- avoid `.await` inside the decision function
- keep state local to the thread where possible
- use structs and enums for explicit domain modeling
- use Chronicle Queue, shared memory, Aeron, or kernel-bypass networking for real low-latency boundaries

The operational shell can use:

- Tokio for lifecycle, admin tasks, signal handling, and slower IO
- OpenTelemetry for metrics and traces
- structured JSON logs with non-blocking writers
- Kubernetes for normal cloud deployment
- dedicated bare-metal or tuned compute for true microsecond production latency

Cloud Kubernetes is useful for development, integration, replay, and control-plane services. For the actual few-microsecond trading path, expect CPU pinning, tuned Linux, careful NIC/interrupt placement, and usually colocated bare metal.

## Workspace

- `crates/liquidity-hub-core`: pure decision engine and Rust basics playground
- `crates/chronicle-transport`: mock Chronicle-style queue traits and future Chronicle Enterprise adapter boundary
- `crates/liquidity-hub-app`: runnable mock service
- `docs/architecture.md`: deeper design notes
- `docs/rust-learning-path.md`: Rust concepts taught through this project
- `docs/observability.md`: logging and OpenTelemetry guidance
- `docs/cicd.md`: CI/CD publishing and deployment notes
- `deploy/k8s`: Kubernetes deployment skeleton
- `.github/workflows/ci.yml`: build and test pipeline

## Commands

Install Rust from [rustup.rs](https://rustup.rs), then run:

```bash
cargo fmt
cargo test
cargo run -p liquidity-hub-app -- --ticks 100
```

Useful release checks:

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release --workspace
```

## CI/CD

GitHub Actions are configured with:

- `.github/workflows/ci.yml`: format, clippy, tests, release build, and Docker build check
- `.github/workflows/cd.yml`: publish image to GitHub Container Registry on `main`, `master`, and `v*` tags
- optional manual Kubernetes deploy using repository secret `KUBE_CONFIG`

See `docs/cicd.md` for operating notes.

Published images use:

```text
ghcr.io/<owner>/<repo>/liquidity-hub:<git-sha>
```

## Chronicle Integration

Chronicle Queue Enterprise currently advertises native Rust support, and Chronicle publishes Rust API docs for queue builders, appenders, and tailers. This repo keeps Chronicle behind `EventSink` and `EventSource` traits so local learning and tests work without a commercial dependency.

Production integration should implement:

- `EventSink` using a Chronicle `ExcerptAppender`
- `EventSource` using a Chronicle `ExcerptTailer`
- binary serialization compatible with the rest of your platform
- one queue per traffic class: risk, analytics, market data, decisions, audit

## Latency Roadmap

1. Start with correctness: domain types, deterministic tests, replayable events.
2. Remove avoidable allocations from the tick path.
3. Add criterion benchmarks around `LiquidityEngine::on_tick`.
4. Replace JSON mock serialization with a fixed binary layout.
5. Add Chronicle Enterprise adapter.
6. Pin hub thread to a CPU and isolate it from async/admin work.
7. Add percentile latency histograms and replay tests.
8. Tune OS, CPU governor, IRQ affinity, NUMA placement, and container settings.

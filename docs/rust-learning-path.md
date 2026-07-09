# Rust Learning Path Through This Project

## 1. Values and Ownership

Rust values have one owner by default. Passing a value into a function moves it unless the type implements `Copy` or you borrow it.

In this repo:

- `MarketTick` is `Copy`, so the engine can receive it by value cheaply.
- `LiquidityEngine` owns its maps of latest risk and analytics state.
- `update_risk` and `update_analytics` move snapshots into the engine.

## 2. Structs

Structs group named fields:

```rust
pub struct MarketTick {
    pub instrument: InstrumentId,
    pub bid: PriceMicros,
    pub ask: PriceMicros,
    pub bid_size: Quantity,
    pub ask_size: Quantity,
    pub ts: TimestampNanos,
}
```

Trading systems should prefer domain-specific structs over raw `i64` and `u64` everywhere. `PriceMicros(10_000_000)` is harder to misuse than a naked integer.

## 3. Enums

Enums model finite choices:

```rust
pub enum LiquidityDecision {
    Quote(OrderIntent),
    Hold(HoldReason),
}
```

This forces callers to handle both paths. That is one of Rust's best habits: make invalid or forgotten states hard to express.

## 4. Pattern Matching

The engine uses `let Some(value) = ... else { ... }` to handle missing state:

```rust
let Some(risk) = self.risk.get(&tick.instrument) else {
    return LiquidityDecision::Hold(HoldReason::NoRisk);
};
```

This is common Rust: unwrap the good case, return early for the bad case.

## 5. Borrowing

`on_tick(&self, tick: MarketTick)` borrows the engine immutably. It can read risk and analytics, but it cannot mutate them.

`update_risk(&mut self, risk: RiskAppetite)` borrows the engine mutably. Rust guarantees only one mutable borrow at a time, which prevents data races.

## 6. Traits

Traits define behavior:

```rust
pub trait EventSink {
    fn publish(&mut self, event: HubEvent) -> Result<(), TransportError>;
}
```

The app can publish to a mock queue today and a Chronicle Queue tomorrow without changing the engine.

## 7. Result and Error Handling

Rust does not use exceptions for normal error handling. Fallible functions return `Result<T, E>`.

```rust
fn seed_mock_inputs(queue: &mut MockChronicleQueue, ticks: u64) -> Result<()>
```

The `?` operator returns early if an error occurs.

## 8. Tests

The core crate has unit tests beside the code:

```bash
cargo test -p liquidity-hub-core
```

Good trading-engine tests should include:

- missing risk
- disabled risk
- missing analytics
- crossed or wide markets
- buy edge
- sell edge
- quantity capped by risk
- quantity capped by market size

## 9. Next Rust Topics

After this first version, learn:

- lifetimes, once borrowing feels natural
- `Arc`, channels, and atomics
- `Pin` and async only when needed
- benchmarking with Criterion
- binary serialization and memory layout
- FFI if Chronicle or feed handlers require native bindings


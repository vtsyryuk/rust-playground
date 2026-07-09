use liquidity_hub_core::{AnalyticsSignal, LiquidityDecision, MarketTick, RiskAppetite};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HubEvent {
    Risk(RiskAppetite),
    Analytics(AnalyticsSignal),
    Tick(MarketTick),
    Decision(LiquidityDecision),
}

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("transport is empty")]
    Empty,
    #[error("serialization failed: {0}")]
    Serialization(String),
}

pub trait EventSink {
    fn publish(&mut self, event: HubEvent) -> Result<(), TransportError>;
}

pub trait EventSource {
    fn try_next(&mut self) -> Result<Option<HubEvent>, TransportError>;
}

#[derive(Debug, Default)]
pub struct MockChronicleQueue {
    events: VecDeque<Vec<u8>>,
}

impl EventSink for MockChronicleQueue {
    fn publish(&mut self, event: HubEvent) -> Result<(), TransportError> {
        let bytes = serde_json::to_vec(&event)
            .map_err(|err| TransportError::Serialization(err.to_string()))?;
        self.events.push_back(bytes);
        Ok(())
    }
}

impl EventSource for MockChronicleQueue {
    fn try_next(&mut self) -> Result<Option<HubEvent>, TransportError> {
        let Some(bytes) = self.events.pop_front() else {
            return Ok(None);
        };
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|err| TransportError::Serialization(err.to_string()))
    }
}

#[cfg(feature = "chronicle-enterprise")]
pub mod enterprise {
    //! Placeholder for Chronicle Queue Enterprise integration.
    //!
    //! Chronicle's Rust API is a commercial dependency. Keep production code behind this
    //! feature and implement the same `EventSink` / `EventSource` traits with Chronicle
    //! appender and tailer types. The rest of the liquidity hub should not know whether
    //! it is backed by Chronicle, an in-memory queue, or a test fixture.
}

#[cfg(test)]
mod tests {
    use super::*;
    use liquidity_hub_core::{InstrumentId, PriceMicros, Quantity, TimestampNanos};

    #[test]
    fn mock_transport_round_trips_events() {
        let mut queue = MockChronicleQueue::default();
        let event = HubEvent::Tick(MarketTick {
            instrument: InstrumentId(1),
            bid: PriceMicros(10),
            ask: PriceMicros(11),
            bid_size: Quantity(100),
            ask_size: Quantity(200),
            ts: TimestampNanos(42),
        });

        queue.publish(event.clone()).unwrap();

        assert_eq!(queue.try_next().unwrap(), Some(event));
        assert_eq!(queue.try_next().unwrap(), None);
    }
}

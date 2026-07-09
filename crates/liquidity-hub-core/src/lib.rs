use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstrumentId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quantity(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PriceMicros(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimestampNanos(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketTick {
    pub instrument: InstrumentId,
    pub bid: PriceMicros,
    pub ask: PriceMicros,
    pub bid_size: Quantity,
    pub ask_size: Quantity,
    pub ts: TimestampNanos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskAppetite {
    pub instrument: InstrumentId,
    pub max_buy_qty: Quantity,
    pub max_sell_qty: Quantity,
    pub enabled: bool,
    pub version: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalyticsSignal {
    pub instrument: InstrumentId,
    pub fair_value: PriceMicros,
    pub confidence_bps: u16,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiquidityDecision {
    Quote(OrderIntent),
    Hold(HoldReason),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderIntent {
    pub instrument: InstrumentId,
    pub side: Side,
    pub price: PriceMicros,
    pub qty: Quantity,
    pub reason: DecisionReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionReason {
    BuyBelowFairValue,
    SellAboveFairValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HoldReason {
    NoRisk,
    RiskDisabled,
    NoAnalytics,
    SpreadTooWide,
    NoEdge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineConfig {
    pub max_spread_micros: i64,
    pub min_edge_micros: i64,
    pub default_quote_qty: Quantity,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_spread_micros: 250,
            min_edge_micros: 50,
            default_quote_qty: Quantity(100),
        }
    }
}

#[derive(Debug, Default)]
pub struct LiquidityEngine {
    config: EngineConfig,
    risk: HashMap<InstrumentId, RiskAppetite>,
    analytics: HashMap<InstrumentId, AnalyticsSignal>,
}

impl LiquidityEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config,
            risk: HashMap::new(),
            analytics: HashMap::new(),
        }
    }

    pub fn update_risk(&mut self, risk: RiskAppetite) {
        self.risk.insert(risk.instrument, risk);
    }

    pub fn update_analytics(&mut self, signal: AnalyticsSignal) {
        self.analytics.insert(signal.instrument, signal);
    }

    pub fn on_tick(&self, tick: MarketTick) -> LiquidityDecision {
        let Some(risk) = self.risk.get(&tick.instrument) else {
            return LiquidityDecision::Hold(HoldReason::NoRisk);
        };
        if !risk.enabled {
            return LiquidityDecision::Hold(HoldReason::RiskDisabled);
        }

        let Some(signal) = self.analytics.get(&tick.instrument) else {
            return LiquidityDecision::Hold(HoldReason::NoAnalytics);
        };

        let spread = tick.ask.0 - tick.bid.0;
        if spread <= 0 || spread > self.config.max_spread_micros {
            return LiquidityDecision::Hold(HoldReason::SpreadTooWide);
        }

        if signal.fair_value.0 - tick.ask.0 >= self.config.min_edge_micros {
            return LiquidityDecision::Quote(OrderIntent {
                instrument: tick.instrument,
                side: Side::Buy,
                price: tick.ask,
                qty: min_qty(
                    self.config.default_quote_qty,
                    risk.max_buy_qty,
                    tick.ask_size,
                ),
                reason: DecisionReason::BuyBelowFairValue,
            });
        }

        if tick.bid.0 - signal.fair_value.0 >= self.config.min_edge_micros {
            return LiquidityDecision::Quote(OrderIntent {
                instrument: tick.instrument,
                side: Side::Sell,
                price: tick.bid,
                qty: min_qty(
                    self.config.default_quote_qty,
                    risk.max_sell_qty,
                    tick.bid_size,
                ),
                reason: DecisionReason::SellAboveFairValue,
            });
        }

        LiquidityDecision::Hold(HoldReason::NoEdge)
    }
}

fn min_qty(a: Quantity, b: Quantity, c: Quantity) -> Quantity {
    Quantity(a.0.min(b.0).min(c.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> LiquidityEngine {
        let mut engine = LiquidityEngine::new(EngineConfig::default());
        engine.update_risk(RiskAppetite {
            instrument: InstrumentId(101),
            max_buy_qty: Quantity(50),
            max_sell_qty: Quantity(75),
            enabled: true,
            version: 1,
        });
        engine.update_analytics(AnalyticsSignal {
            instrument: InstrumentId(101),
            fair_value: PriceMicros(10_100_000),
            confidence_bps: 9_000,
            version: 1,
        });
        engine
    }

    #[test]
    fn quotes_buy_when_ask_is_below_fair_value() {
        let decision = engine().on_tick(MarketTick {
            instrument: InstrumentId(101),
            bid: PriceMicros(10_000_000),
            ask: PriceMicros(10_000_100),
            bid_size: Quantity(1_000),
            ask_size: Quantity(1_000),
            ts: TimestampNanos(1),
        });

        assert_eq!(
            decision,
            LiquidityDecision::Quote(OrderIntent {
                instrument: InstrumentId(101),
                side: Side::Buy,
                price: PriceMicros(10_000_100),
                qty: Quantity(50),
                reason: DecisionReason::BuyBelowFairValue,
            })
        );
    }

    #[test]
    fn holds_when_risk_is_missing() {
        let engine = LiquidityEngine::new(EngineConfig::default());
        let decision = engine.on_tick(MarketTick {
            instrument: InstrumentId(101),
            bid: PriceMicros(10_000_000),
            ask: PriceMicros(10_000_100),
            bid_size: Quantity(1_000),
            ask_size: Quantity(1_000),
            ts: TimestampNanos(1),
        });

        assert_eq!(decision, LiquidityDecision::Hold(HoldReason::NoRisk));
    }
}

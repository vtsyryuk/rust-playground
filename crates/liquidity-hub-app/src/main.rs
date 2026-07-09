use anyhow::Result;
use chronicle_transport::{EventSink, EventSource, HubEvent, MockChronicleQueue};
use clap::Parser;
use liquidity_hub_core::{
    AnalyticsSignal, EngineConfig, InstrumentId, LiquidityDecision, LiquidityEngine, MarketTick,
    PriceMicros, Quantity, RiskAppetite, TimestampNanos,
};
use tracing::{debug, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, env = "LH_TICKS", default_value_t = 10)]
    ticks: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _log_guard = init_logging()?;
    let args = Args::parse();

    let mut input = MockChronicleQueue::default();
    let mut output = MockChronicleQueue::default();
    seed_mock_inputs(&mut input, args.ticks)?;

    let mut engine = LiquidityEngine::new(EngineConfig::default());
    let mut processed = 0_u64;

    while let Some(event) = input.try_next()? {
        match event {
            HubEvent::Risk(risk) => engine.update_risk(risk),
            HubEvent::Analytics(signal) => engine.update_analytics(signal),
            HubEvent::Tick(tick) => {
                let decision = engine.on_tick(tick);
                if matches!(decision, LiquidityDecision::Quote(_)) {
                    debug!(?decision, "quote decision");
                }
                output.publish(HubEvent::Decision(decision))?;
                processed += 1;
            }
            HubEvent::Decision(_) => {}
        }
    }

    info!(processed, "liquidity hub completed mock run");
    Ok(())
}

fn init_logging() -> Result<tracing_appender::non_blocking::WorkerGuard> {
    let file_appender = tracing_appender::rolling::never("logs", "liquidity-hub.jsonl");
    let (writer, guard) = tracing_appender::non_blocking(file_appender);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let stdout_layer = fmt::layer().compact().with_writer(std::io::stdout);
    let file_layer = fmt::layer().json().with_writer(writer);
    let subscriber = Registry::default()
        .with(filter)
        .with(stdout_layer)
        .with(file_layer);

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(guard)
}

fn seed_mock_inputs(queue: &mut MockChronicleQueue, ticks: u64) -> Result<()> {
    queue.publish(HubEvent::Risk(RiskAppetite {
        instrument: InstrumentId(101),
        max_buy_qty: Quantity(50),
        max_sell_qty: Quantity(75),
        enabled: true,
        version: 1,
    }))?;

    queue.publish(HubEvent::Analytics(AnalyticsSignal {
        instrument: InstrumentId(101),
        fair_value: PriceMicros(10_100_000),
        confidence_bps: 9_000,
        version: 1,
    }))?;

    for i in 0..ticks {
        queue.publish(HubEvent::Tick(MarketTick {
            instrument: InstrumentId(101),
            bid: PriceMicros(10_000_000 + i as i64),
            ask: PriceMicros(10_000_100 + i as i64),
            bid_size: Quantity(1_000),
            ask_size: Quantity(1_000),
            ts: TimestampNanos(i),
        }))?;
    }

    Ok(())
}

//! Performance Benchmark for HFT Trading System
//! 
//! This example benchmarks the HFT trading system to demonstrate its ultra-low latency capabilities.

use hft_trading_system::{
    Engine,
    data::{MarketEvent, MarketDataKind, PublicTrade, InstrumentId, ExchangeId, Side},
    execution::MockExecutionClient,
    strategy::DefaultStrategy,
    risk::DefaultRiskManager,
    SystemEvent,
    engine::EngineConfig,
};
use chrono::Utc;
use rust_decimal::Decimal;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!("HFT TRADING SYSTEM - PERFORMANCE BENCHMARK");
    println!("This benchmark demonstrates the ultra-low latency capabilities of the system");
    println!("{}", "=".repeat(80));
    println!();

    // Create trading components
    let strategy = DefaultStrategy::new("benchmark_hft".to_string());
    let risk_manager = DefaultRiskManager::default();
    let execution_client = MockExecutionClient::new();
    let engine_config = EngineConfig::default();
    
    // Create the trading engine
    let mut engine = Engine::new(strategy, risk_manager, execution_client, engine_config);
    
    // Create sample instrument
    let instrument = InstrumentId {
        base: "BTC".to_string(),
        quote: "USDT".to_string(),
        exchange_symbol: "BTCUSDT".to_string(),
    };
    
    println!("🔧 Trading engine initialized for benchmarking");
    println!();
    
    // Benchmark different scenarios
    benchmark_single_event(&mut engine, &instrument)?;
    benchmark_batch_processing(&mut engine, &instrument)?;
    benchmark_high_frequency(&mut engine, &instrument)?;
    
    // Final results
    println!("{}", "=".repeat(80));
    println!("🏆 BENCHMARK RESULTS SUMMARY");
    println!("{}", "=".repeat(80));
    println!("   Total Events Processed: {}", engine.meta.events_processed);
    println!("   Overall Average Latency: {}μs", engine.metrics.avg_latency_micros);
    println!("   Minimum Latency Recorded: {}μs", engine.metrics.min_latency_micros);
    println!("   Maximum Latency Recorded: {}μs", engine.metrics.max_latency_micros);
    println!("   Total Orders Generated: {}", engine.metrics.orders_sent);
    println!();
    println!("🎉 Performance benchmark completed!");
    
    Ok(())
}

/// Benchmark single event processing
fn benchmark_single_event(
    engine: &mut hft_trading_system::engine::Engine<DefaultStrategy, DefaultRiskManager, MockExecutionClient>,
    instrument: &InstrumentId,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ SINGLE EVENT PROCESSING BENCHMARK");
    println!("   Testing latency for individual market event processing");
    
    let market_event = MarketEvent {
        exchange: ExchangeId::Binance,
        instrument: instrument.clone(),
        kind: MarketDataKind::Trade(PublicTrade {
            id: "benchmark_trade".to_string(),
            price: Decimal::from_str_exact("50000.00")?,
            quantity: Decimal::from_str_exact("0.1")?,
            side: Side::Buy,
            timestamp: Utc::now(),
        }),
        exchange_time: Utc::now(),
        receipt_time: Utc::now(),
    };
    
    let start_time = Instant::now();
    let output = engine.process_event(SystemEvent::Market(market_event));
    let duration = start_time.elapsed();
    
    println!("   ├── Processing Time: {}ns ({}μs)", duration.as_nanos(), duration.as_micros());
    println!("   ├── Strategy Output: {:?}", output.strategy_output.is_some());
    println!("   └── Risk Check Output: {:?}", output.risk_output.is_some());
    println!();
    
    Ok(())
}

/// Benchmark batch processing
fn benchmark_batch_processing(
    engine: &mut hft_trading_system::engine::Engine<DefaultStrategy, DefaultRiskManager, MockExecutionClient>,
    instrument: &InstrumentId,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 BATCH PROCESSING BENCHMARK");
    println!("   Testing throughput for processing multiple events");
    
    // Create batch of market events
    let batch_size = 1000;
    let mut market_events = Vec::new();
    
    for i in 0..batch_size {
        let price = 50000.0 + (i as f64 * 0.01);
        let quantity = 0.1 + (i as f64 * 0.001);
        
        market_events.push(MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: instrument.clone(),
            kind: MarketDataKind::Trade(PublicTrade {
                id: format!("batch_trade_{}", i),
                price: Decimal::from_str_exact(&price.to_string())?,
                quantity: Decimal::from_str_exact(&quantity.to_string())?,
                side: if i % 2 == 0 { Side::Buy } else { Side::Sell },
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        });
    }
    
    let start_time = Instant::now();
    
    for event in market_events {
        engine.process_event(SystemEvent::Market(event));
    }
    
    let duration = start_time.elapsed();
    let avg_latency = duration.as_micros() as f64 / batch_size as f64;
    
    println!("   ├── Batch Size: {} events", batch_size);
    println!("   ├── Total Time: {}ms", duration.as_millis());
    println!("   ├── Average Latency: {:.2}μs per event", avg_latency);
    println!("   ├── Events Processed: {}", engine.meta.events_processed);
    println!("   └── Throughput: {:.0} events/second", (batch_size as f64 / duration.as_secs_f64()) as i64);
    println!();
    
    Ok(())
}

/// Benchmark high frequency processing
fn benchmark_high_frequency(
    engine: &mut hft_trading_system::engine::Engine<DefaultStrategy, DefaultRiskManager, MockExecutionClient>,
    instrument: &InstrumentId,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 HIGH FREQUENCY PROCESSING BENCHMARK");
    println!("   Testing system under high-frequency market conditions");
    
    let events_per_second = 10000;
    let duration_seconds = 5;
    let total_events = events_per_second * duration_seconds;
    
    println!("   ├── Target Rate: {} events/second", events_per_second);
    println!("   ├── Duration: {} seconds", duration_seconds);
    println!("   └── Total Events: {}", total_events);
    println!();
    
    let start_time = Instant::now();
    let mut events_processed = 0;
    
    // Process events at high frequency
    for i in 0..total_events {
        let price = 50000.0 + ((i % 1000) as f64 * 0.01);
        let quantity = 0.1 + ((i % 100) as f64 * 0.001);
        
        let market_event = MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: instrument.clone(),
            kind: MarketDataKind::Trade(PublicTrade {
                id: format!("hf_trade_{}", i),
                price: Decimal::from_str_exact(&price.to_string())?,
                quantity: Decimal::from_str_exact(&quantity.to_string())?,
                side: if i % 2 == 0 { Side::Buy } else { Side::Sell },
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        };
        
        engine.process_event(SystemEvent::Market(market_event));
        events_processed += 1;
        
        // Small delay to simulate realistic timing
        if i % 1000 == 0 {
            println!("   Processed {} events...", events_processed);
        }
    }
    
    let total_duration = start_time.elapsed();
    let avg_latency = total_duration.as_micros() as f64 / events_processed as f64;
    let actual_rate = (events_processed as f64 / total_duration.as_secs_f64()) as i64;
    
    println!();
    println!("   RESULTS:");
    println!("   ├── Actual Events Processed: {}", events_processed);
    println!("   ├── Total Time: {:.2}ms", total_duration.as_millis());
    println!("   ├── Average Latency: {:.2}μs per event", avg_latency);
    println!("   ├── Actual Rate: {} events/second", actual_rate);
    println!("   └── Efficiency: {:.1}%", (actual_rate as f64 / events_per_second as f64) * 100.0);
    println!();
    
    Ok(())
}
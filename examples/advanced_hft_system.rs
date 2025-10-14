//! Advanced HFT system example
//! 
//! This example demonstrates a more complete HFT trading system with multiple components
//! working together, including market data streaming, order execution, and risk management.

use hft_trading_system::{
    Engine,
    data::{MarketEvent, MarketDataKind, PublicTrade, OrderBookL1, InstrumentId, ExchangeId, Side},
    execution::{MockExecutionClient, ExecutionClient},
    strategy::DefaultStrategy,
    risk::{DefaultRiskManager, RiskLimits},
    config::{SystemConfig, InstrumentConfig, ExecutionConfig, DataConfig},
    SystemEvent,
    engine::{EngineConfig, EngineState},
};
use chrono::Utc;
use rust_decimal::Decimal;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Advanced HFT Trading System Example");
    
    // Create system configuration
    let _config = create_system_config();
    
    // Create trading components
    let strategy = DefaultStrategy::new("advanced_mean_reversion".to_string());
    let risk_manager = DefaultRiskManager::default();
    let execution_client = MockExecutionClient::new();
    let engine_config = EngineConfig::default();
    
    // Create the trading engine
    let mut engine = Engine::new(strategy, risk_manager, execution_client, engine_config);
    
    // Create sample instruments
    let instruments = create_sample_instruments();
    
    // Create sample market data stream
    let market_events = create_sample_market_data(&instruments);
    
    // Process market events in a loop to simulate real-time trading
    println!("Processing market events...");
    
    for (i, event) in market_events.into_iter().enumerate() {
        if engine.state == EngineState::Shutdown {
            break;
        }
        
        let output = engine.process_event(SystemEvent::Market(event));
        
        // Print metrics every 5 events
        if i % 5 == 0 {
            println!("Events processed: {}, Avg latency: {}μs", 
                engine.meta.events_processed, 
                engine.metrics.avg_latency_micros);
        }
        
        // Send any generated orders
        if let Some(strategy_output) = output.strategy_output {
            for order in strategy_output.orders {
                match engine.execution_client.send_order(order) {
                    Ok(report) => {
                        println!("Order sent: {} (status: {:?})", 
                            report.client_order_id, report.status);
                        engine.metrics.record_order_sent();
                    },
                    Err(e) => println!("Failed to send order: {:?}", e),
                }
            }
        }
        
        // Simulate some delay between events
        sleep(Duration::from_millis(10)).await;
    }
    
    // Print final performance metrics
    println!("\n=== Final Performance Metrics ===");
    println!("Total events processed: {}", engine.meta.events_processed);
    println!("Average latency: {}μs", engine.metrics.avg_latency_micros);
    println!("Min latency: {}μs", engine.metrics.min_latency_micros);
    println!("Max latency: {}μs", engine.metrics.max_latency_micros);
    println!("Orders sent: {}", engine.metrics.orders_sent);
    
    println!("\nAdvanced HFT Trading System Example completed");
    Ok(())
}

/// Create system configuration
fn create_system_config() -> SystemConfig {
    SystemConfig {
        risk_limits: RiskLimits::default(),
        instruments: vec![
            InstrumentConfig {
                instrument: InstrumentId {
                    base: "BTC".to_string(),
                    quote: "USDT".to_string(),
                    exchange_symbol: "BTCUSDT".to_string(),
                },
                enabled: true,
                base_currency: "BTC".to_string(),
                quote_currency: "USDT".to_string(),
                min_order_size: Decimal::from_str_exact("0.001").unwrap(),
                tick_size: Decimal::from_str_exact("0.1").unwrap(),
                max_position_size: Decimal::from_str_exact("10").unwrap(),
            },
            InstrumentConfig {
                instrument: InstrumentId {
                    base: "ETH".to_string(),
                    quote: "USDT".to_string(),
                    exchange_symbol: "ETHUSDT".to_string(),
                },
                enabled: true,
                base_currency: "ETH".to_string(),
                quote_currency: "USDT".to_string(),
                min_order_size: Decimal::from_str_exact("0.01").unwrap(),
                tick_size: Decimal::from_str_exact("0.01").unwrap(),
                max_position_size: Decimal::from_str_exact("100").unwrap(),
            },
        ],
        execution: ExecutionConfig {
            default_order_type: hft_trading_system::execution::OrderType::Limit,
            default_time_in_force: "GTC".to_string(),
            enable_order_aggregation: true,
            order_aggregation_timeout_ms: 10,
        },
        data: DataConfig {
            enable_market_data: true,
            market_data_types: vec!["trades".to_string(), "orderbook_l1".to_string()],
            update_frequency_ms: 100,
            enable_historical_data: false,
        },
    }
}

/// Create sample instruments
fn create_sample_instruments() -> Vec<InstrumentId> {
    vec![
        InstrumentId {
            base: "BTC".to_string(),
            quote: "USDT".to_string(),
            exchange_symbol: "BTCUSDT".to_string(),
        },
        InstrumentId {
            base: "ETH".to_string(),
            quote: "USDT".to_string(),
            exchange_symbol: "ETHUSDT".to_string(),
        },
    ]
}

/// Create sample market data
fn create_sample_market_data(instruments: &[InstrumentId]) -> Vec<MarketEvent> {
    let mut events = Vec::new();
    
    // Create trade events
    for (i, instrument) in instruments.iter().enumerate() {
        events.push(MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: instrument.clone(),
            kind: MarketDataKind::Trade(PublicTrade {
                id: format!("trade_{}", i),
                price: Decimal::from_str_exact(&(50000.0 + (i as f64 * 1000.0)).to_string()).unwrap(),
                quantity: Decimal::from_str_exact("0.1").unwrap(),
                side: if i % 2 == 0 { Side::Buy } else { Side::Sell },
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        });
    }
    
    // Create order book events
    for (i, instrument) in instruments.iter().enumerate() {
        events.push(MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: instrument.clone(),
            kind: MarketDataKind::OrderBookL1(OrderBookL1 {
                bid_price: Decimal::from_str_exact(&(49990.0 + (i as f64 * 1000.0)).to_string()).unwrap(),
                bid_quantity: Decimal::from_str_exact("1.0").unwrap(),
                ask_price: Decimal::from_str_exact(&(50010.0 + (i as f64 * 1000.0)).to_string()).unwrap(),
                ask_quantity: Decimal::from_str_exact("1.0").unwrap(),
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        });
    }
    
    events
}
//! Terminal Demo for HFT Trading System
//! 
//! This example demonstrates the HFT trading system in action with real-time terminal output
//! showing market data processing, strategy execution, risk management, and order execution.

use hft_trading_system::{
    Engine,
    data::{MarketEvent, MarketDataKind, PublicTrade, OrderBookL1, InstrumentId, ExchangeId, Side},
    execution::{MockExecutionClient, ExecutionClient},
    strategy::DefaultStrategy,
    risk::{DefaultRiskManager, RiskLimits},
    config::{SystemConfig, InstrumentConfig, ExecutionConfig, DataConfig},
    statistic::PerformanceMetrics,
    SystemEvent,
    engine::EngineConfig,
};
use chrono::Utc;
use rust_decimal::Decimal;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!("HFT TRADING SYSTEM TERMINAL DEMO");
    println!("This demo shows a high-frequency trading system in action");
    println!("{}", "=".repeat(80));
    println!();

    // Initialize the system
    println!("🔧 Initializing HFT Trading System...");
    sleep(Duration::from_millis(500)).await;
    
    // Create system configuration
    create_system_config();
    println!("   ├── System configuration loaded");
    
    // Create trading components
    let strategy = DefaultStrategy::new("mean_reversion_hft".to_string());
    println!("   ├── Strategy module initialized");
    
    let risk_manager = DefaultRiskManager::default();
    println!("   ├── Risk management module initialized");
    
    let execution_client = MockExecutionClient::new();
    println!("   ├── Execution client initialized");
    
    let engine_config = EngineConfig::default();
    println!("   └── Engine configuration set");
    
    sleep(Duration::from_millis(500)).await;
    
    // Create the trading engine
    let mut engine = Engine::new(strategy, risk_manager, execution_client, engine_config);
    println!("🚀 Trading engine started successfully!");
    println!("   ├── Engine state: {:?}", engine.state);
    println!("   ├── Sequence ID: {}", engine.meta.sequence.value());
    println!("   └── Start time: {}", engine.meta.start_time);
    println!();
    
    sleep(Duration::from_millis(1000)).await;
    
    // Create sample instruments
    let instruments = create_sample_instruments();
    println!("📈 Market instruments loaded:");
    for (i, instrument) in instruments.iter().enumerate() {
        println!("   {}. {}/{} ({})", i+1, instrument.base, instrument.quote, instrument.exchange_symbol);
    }
    println!();
    
    sleep(Duration::from_millis(1000)).await;
    
    // Simulate real-time market data processing
    println!("📡 Starting market data simulation...");
    println!();
    
    // Process a series of market events to demonstrate the system
    let market_events = create_demonstration_market_data(&instruments);
    
    let mut total_orders_sent = 0;
    let mut total_orders_filled = 0;
    
    for (i, event) in market_events.into_iter().enumerate() {
        // Print market event
        print_market_event(&event, i + 1);
        
        // Process the event through the engine
        let output = engine.process_event(SystemEvent::Market(event));
        
        // Show processing results
        if let Some(strategy_output) = &output.strategy_output {
            if !strategy_output.orders.is_empty() {
                println!("   📊 Strategy generated {} order(s)", strategy_output.orders.len());
                
                // Send orders through risk management
                if let Some(risk_output) = &output.risk_output {
                    for (j, risk_check) in risk_output.iter().enumerate() {
                        if risk_check.approved {
                            println!("   ✅ Order {} passed risk checks", j + 1);
                        } else {
                            println!("   ❌ Order {} failed risk checks: {:?}", j + 1, risk_check.reason);
                        }
                    }
                    
                    // Send approved orders to execution
                    for order in &strategy_output.orders {
                        match engine.execution_client.send_order(order.clone()) {
                            Ok(report) => {
                                println!("   📤 Order sent: {}", report.client_order_id);
                                total_orders_sent += 1;
                                
                                // Simulate order fill
                                if total_orders_sent % 3 == 0 {
                                    println!("   💰 Order filled: {}", report.client_order_id);
                                    total_orders_filled += 1;
                                }
                            },
                            Err(e) => println!("   ⚠️  Failed to send order: {:?}", e),
                        }
                    }
                }
            } else {
                println!("   📊 Strategy generated no orders for this event");
            }
        }
        
        // Show performance metrics every few events
        if (i + 1) % 3 == 0 {
            show_performance_metrics(&engine.metrics, engine.meta.events_processed);
        }
        
        println!();
        sleep(Duration::from_millis(800)).await;
    }
    
    // Final system status
    println!("{}", "=".repeat(80));
    println!("🏁 TRADING SESSION COMPLETED");
    println!("{}", "=".repeat(80));
    
    show_final_summary(&engine, total_orders_sent, total_orders_filled);
    
    // Demonstrate engine control features
    println!("\n⚙️  Demonstrating engine control features:");
    sleep(Duration::from_millis(500)).await;
    
    println!("   Pausing engine...");
    engine.pause();
    println!("   ├── Engine state: {:?}", engine.state);
    
    sleep(Duration::from_millis(500)).await;
    
    println!("   Resuming engine...");
    engine.resume();
    println!("   ├── Engine state: {:?}", engine.state);
    
    sleep(Duration::from_millis(500)).await;
    
    println!("   Shutting down engine...");
    engine.shutdown();
    println!("   ├── Engine state: {:?}", engine.state);
    println!("   └── Shutdown complete");
    
    println!("\n🎉 HFT Trading System Demo Completed Successfully!");
    println!("   Thank you for watching the demonstration.");
    
    Ok(())
}

/// Create system configuration
fn create_system_config() -> SystemConfig {
    SystemConfig {
        risk_limits: RiskLimits {
            max_position_size: Decimal::from_str_exact("50").unwrap(),
            max_notional_exposure: Decimal::from_str_exact("50000").unwrap(),
            max_orders_per_second: 200,
            max_order_size: Decimal::from_str_exact("5").unwrap(),
            enable_circuit_breaker: true,
            max_drawdown_percent: Decimal::from_str_exact("3").unwrap(),
        },
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
                max_position_size: Decimal::from_str_exact("5").unwrap(),
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
                max_position_size: Decimal::from_str_exact("50").unwrap(),
            },
        ],
        execution: ExecutionConfig {
            default_order_type: hft_trading_system::execution::OrderType::Limit,
            default_time_in_force: "GTC".to_string(),
            enable_order_aggregation: true,
            order_aggregation_timeout_ms: 5,
        },
        data: DataConfig {
            enable_market_data: true,
            market_data_types: vec!["trades".to_string(), "orderbook_l1".to_string()],
            update_frequency_ms: 50,
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

/// Create demonstration market data
fn create_demonstration_market_data(instruments: &[InstrumentId]) -> Vec<MarketEvent> {
    let mut events = Vec::new();
    
    // Create trade events
    events.push(MarketEvent {
        exchange: ExchangeId::Binance,
        instrument: instruments[0].clone(),
        kind: MarketDataKind::Trade(PublicTrade {
            id: "trade_001".to_string(),
            price: Decimal::from_str_exact("50125.50").unwrap(),
            quantity: Decimal::from_str_exact("0.15").unwrap(),
            side: Side::Buy,
            timestamp: Utc::now(),
        }),
        exchange_time: Utc::now(),
        receipt_time: Utc::now(),
    });
    
    events.push(MarketEvent {
        exchange: ExchangeId::Binance,
        instrument: instruments[1].clone(),
        kind: MarketDataKind::Trade(PublicTrade {
            id: "trade_002".to_string(),
            price: Decimal::from_str_exact("2850.75").unwrap(),
            quantity: Decimal::from_str_exact("2.3").unwrap(),
            side: Side::Sell,
            timestamp: Utc::now(),
        }),
        exchange_time: Utc::now(),
        receipt_time: Utc::now(),
    });
    
    // Create order book events
    events.push(MarketEvent {
        exchange: ExchangeId::Binance,
        instrument: instruments[0].clone(),
        kind: MarketDataKind::OrderBookL1(OrderBookL1 {
            bid_price: Decimal::from_str_exact("50124.20").unwrap(),
            bid_quantity: Decimal::from_str_exact("1.25").unwrap(),
            ask_price: Decimal::from_str_exact("50126.80").unwrap(),
            ask_quantity: Decimal::from_str_exact("0.80").unwrap(),
            timestamp: Utc::now(),
        }),
        exchange_time: Utc::now(),
        receipt_time: Utc::now(),
    });
    
    events.push(MarketEvent {
        exchange: ExchangeId::Binance,
        instrument: instruments[1].clone(),
        kind: MarketDataKind::OrderBookL1(OrderBookL1 {
            bid_price: Decimal::from_str_exact("2849.50").unwrap(),
            bid_quantity: Decimal::from_str_exact("15.7").unwrap(),
            ask_price: Decimal::from_str_exact("2851.25").unwrap(),
            ask_quantity: Decimal::from_str_exact("8.3").unwrap(),
            timestamp: Utc::now(),
        }),
        exchange_time: Utc::now(),
        receipt_time: Utc::now(),
    });
    
    // More trade events
    events.push(MarketEvent {
        exchange: ExchangeId::Binance,
        instrument: instruments[0].clone(),
        kind: MarketDataKind::Trade(PublicTrade {
            id: "trade_003".to_string(),
            price: Decimal::from_str_exact("50127.10").unwrap(),
            quantity: Decimal::from_str_exact("0.08").unwrap(),
            side: Side::Sell,
            timestamp: Utc::now(),
        }),
        exchange_time: Utc::now(),
        receipt_time: Utc::now(),
    });
    
    events.push(MarketEvent {
        exchange: ExchangeId::Binance,
        instrument: instruments[1].clone(),
        kind: MarketDataKind::Trade(PublicTrade {
            id: "trade_004".to_string(),
            price: Decimal::from_str_exact("2848.90").unwrap(),
            quantity: Decimal::from_str_exact("1.8").unwrap(),
            side: Side::Buy,
            timestamp: Utc::now(),
        }),
        exchange_time: Utc::now(),
        receipt_time: Utc::now(),
    });
    
    events
}

/// Print market event details
fn print_market_event(event: &MarketEvent, event_number: usize) {
    println!("📨 Market Event #{}", event_number);
    println!("   ├── Exchange: {:?}", event.exchange);
    println!("   ├── Instrument: {}/{}", event.instrument.base, event.instrument.quote);
    
    match &event.kind {
        MarketDataKind::Trade(trade) => {
            println!("   ├── Type: Trade");
            println!("   ├── Price: ${}", trade.price);
            println!("   ├── Quantity: {}", trade.quantity);
            println!("   ├── Side: {:?}", trade.side);
            println!("   └── Trade ID: {}", trade.id);
        },
        MarketDataKind::OrderBookL1(book) => {
            println!("   ├── Type: Order Book (L1)");
            println!("   ├── Bid: ${} ({} qty)", book.bid_price, book.bid_quantity);
            println!("   ├── Ask: ${} ({} qty)", book.ask_price, book.ask_quantity);
            println!("   └── Spread: ${}", book.ask_price - book.bid_price);
        },
        _ => {
            println!("   └── Type: Other");
        }
    }
}

/// Show performance metrics
fn show_performance_metrics(metrics: &PerformanceMetrics, events_processed: u64) {
    println!("   📈 Performance Metrics Update:");
    println!("   ├── Events Processed: {}", events_processed);
    println!("   ├── Avg Latency: {}μs", metrics.avg_latency_micros);
    println!("   ├── Min Latency: {}μs", metrics.min_latency_micros);
    println!("   ├── Max Latency: {}μs", metrics.max_latency_micros);
    println!("   ├── Orders Sent: {}", metrics.orders_sent);
    println!("   └── Orders Filled: {}", metrics.orders_filled);
}

/// Show final summary
fn show_final_summary(engine: &hft_trading_system::engine::Engine<DefaultStrategy, DefaultRiskManager, MockExecutionClient>, 
                     total_orders_sent: u64, total_orders_filled: u64) {
    println!("📊 FINAL TRADING SESSION SUMMARY");
    println!("   ├── Total Events Processed: {}", engine.meta.events_processed);
    println!("   ├── Total Processing Time: {}ms", 
             (Utc::now() - engine.meta.start_time).num_milliseconds());
    println!("   ├── Average Processing Latency: {}μs", engine.metrics.avg_latency_micros);
    println!("   ├── Total Orders Sent: {}", total_orders_sent);
    println!("   ├── Total Orders Filled: {}", total_orders_filled);
    println!("   ├── Sequence Numbers Processed: {}", engine.meta.sequence.value());
    println!("   └── Engine Final State: {:?}", engine.state);
}
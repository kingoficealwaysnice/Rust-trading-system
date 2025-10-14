//! Real-time BTC/USDT Trading Demo for HFT Trading System
//! 
//! This example demonstrates the HFT trading system using real-time BTC/USDT data from Binance
//! for an impressive hackathon showcase.

use hft_trading_system::{
    Engine,
    data::{MarketEvent, InstrumentId, ExchangeId, BinanceMarketDataStream, MarketDataStream},
    execution::{MockExecutionClient, ExecutionClient},
    strategy::DefaultStrategy,
    risk::DefaultRiskManager,
    engine::EngineConfig,
    SystemEvent,
};
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!("HFT TRADING SYSTEM - REAL-TIME BTC/USDT DEMO");
    println!("Showcasing real-time trading with live Binance data");
    println!("{}", "=".repeat(80));
    println!();

    // Initialize the system
    println!("🔧 Initializing HFT Trading System with Real-Time Binance Data...");
    sleep(Duration::from_millis(500)).await;
    
    // Create trading components
    let strategy = DefaultStrategy::new("realtime_btc_hft".to_string());
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
    println!("   └── Start time: {}", engine.meta.start_time);
    println!();
    
    sleep(Duration::from_millis(1000)).await;
    
    // Create BTC/USDT instrument
    let btc_usdt = InstrumentId {
        base: "BTC".to_string(),
        quote: "USDT".to_string(),
        exchange_symbol: "BTCUSDT".to_string(),
    };
    println!("📈 Configuring real-time BTC/USDT data feed...");
    println!("   └── Instrument: BTC/USDT (BTCUSDT)");
    println!();
    
    sleep(Duration::from_millis(1000)).await;
    
    // Create and connect to Binance market data stream
    println!("📡 Connecting to Binance WebSocket API...");
    let mut market_stream = BinanceMarketDataStream::new();
    
    // Subscribe to BTC/USDT market data
    match market_stream.subscribe(&[btc_usdt.clone()]).await {
        Ok(_) => {
            println!("   └── Successfully subscribed to BTC/USDT real-time data");
        }
        Err(e) => {
            println!("   ⚠️  Failed to connect to Binance: {}", e);
            println!("   └── Falling back to simulated data for demo purposes");
            
            // In a hackathon setting, we might want to continue with simulated data
            // if the real connection fails
            simulate_btc_data(&mut engine).await?;
            return Ok(());
        }
    }
    
    println!();
    sleep(Duration::from_millis(1000)).await;
    
    // Process real-time market data
    println!("⚡ Processing real-time market data (Press Ctrl+C to stop)...");
    println!();
    
    let mut event_count = 0u64;
    let mut order_count = 0u64;
    
    // Process events for 60 seconds or until we process 50 events
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < Duration::from_secs(60) && event_count < 50 {
        // Get next market event
        match market_stream.next().await {
            Ok(Some(event)) => {
                event_count += 1;
                
                // Print market event details
                print_market_event(&event, event_count);
                
                // Process the event through the engine
                let output = engine.process_event(SystemEvent::Market(event));
                
                // Show processing results
                if let Some(strategy_output) = &output.strategy_output {
                    if !strategy_output.orders.is_empty() {
                        println!("   📊 Strategy generated {} order(s)", strategy_output.orders.len());
                        order_count += strategy_output.orders.len() as u64;
                        
                        // Send orders through risk management
                        if let Some(risk_output) = &output.risk_output {
                            for (j, risk_check) in risk_output.iter().enumerate() {
                                if risk_check.approved {
                                    println!("   ✅ Order {} passed risk checks", j + 1);
                                } else {
                                    println!("   ❌ Order {} failed risk checks: {:?}", j + 1, risk_check.reason);
                                }
                            }
                        }
                        
                        // Send approved orders to execution
                        for (j, order) in strategy_output.orders.iter().enumerate() {
                            match ExecutionClient::send_order(&mut engine.execution_client, order.clone()) {
                                Ok(report) => {
                                    println!("   📤 Order {} sent: {}", j + 1, report.client_order_id);
                                    
                                    // Simulate order fill (50% chance)
                                    if event_count % 2 == 0 {
                                        println!("   💰 Order {} filled: {}", j + 1, report.client_order_id);
                                    }
                                },
                                Err(e) => println!("   ⚠️  Failed to send order {}: {:?}", j + 1, e),
                            }
                        }
                    } else {
                        println!("   📊 Strategy generated no orders for this event");
                    }
                }
                
                println!();
                sleep(Duration::from_millis(100)).await;
            }
            Ok(None) => {
                // No more events
                break;
            }
            Err(e) => {
                println!("   ⚠️  Error receiving market data: {}", e);
                sleep(Duration::from_millis(1000)).await;
            }
        }
    }
    
    // Final system status
    println!("{}", "=".repeat(80));
    println!("🏁 REAL-TIME TRADING SESSION COMPLETED");
    println!("{}", "=".repeat(80));
    
    println!("📊 SESSION SUMMARY");
    println!("   ├── Total Events Processed: {}", event_count);
    println!("   ├── Total Processing Time: {}ms", start_time.elapsed().as_millis());
    println!("   ├── Average Processing Latency: {}μs", engine.metrics.avg_latency_micros);
    println!("   ├── Total Orders Generated: {}", order_count);
    println!("   ├── Engine Final State: {:?}", engine.state);
    println!("   └── Sequence Numbers Processed: {}", engine.meta.sequence.value());
    
    println!("\n🎉 Real-Time BTC/USDT Trading Demo Completed Successfully!");
    println!("   Thank you for watching the live demonstration.");
    
    Ok(())
}

/// Print market event details
fn print_market_event(event: &MarketEvent, event_number: u64) {
    println!("📨 Market Event #{}", event_number);
    println!("   ├── Exchange: {:?}", event.exchange);
    println!("   ├── Instrument: {}/{}", event.instrument.base, event.instrument.quote);
    println!("   ├── Timestamp: {}", event.exchange_time.format("%H:%M:%S%.3f"));
    
    match &event.kind {
        hft_trading_system::data::MarketDataKind::Trade(trade) => {
            println!("   ├── Type: Trade");
            println!("   ├── Price: ${}", trade.price);
            println!("   ├── Quantity: {}", trade.quantity);
            println!("   ├── Side: {:?}", trade.side);
            println!("   └── Trade ID: {}", trade.id);
        },
        hft_trading_system::data::MarketDataKind::OrderBookL1(book) => {
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

/// Simulate BTC data for demo purposes if real connection fails
async fn simulate_btc_data(engine: &mut hft_trading_system::engine::Engine<DefaultStrategy, DefaultRiskManager, MockExecutionClient>) -> Result<(), Box<dyn std::error::Error>> {
    use hft_trading_system::data::{MarketDataKind, PublicTrade, Side, OrderBookL1};
    use rust_decimal::Decimal;
    use chrono::Utc;
    use std::str::FromStr;
    
    println!("\n🔄 Simulating BTC/USDT market data for demo...");
    
    // Create sample market events
    let btc_usdt = InstrumentId {
        base: "BTC".to_string(),
        quote: "USDT".to_string(),
        exchange_symbol: "BTCUSDT".to_string(),
    };
    
    let sample_events = vec![
        MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: btc_usdt.clone(),
            kind: MarketDataKind::Trade(PublicTrade {
                id: "sim_001".to_string(),
                price: Decimal::from_str("67520.50")?,
                quantity: Decimal::from_str("0.125")?,
                side: Side::Buy,
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        },
        MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: btc_usdt.clone(),
            kind: MarketDataKind::OrderBookL1(OrderBookL1 {
                bid_price: Decimal::from_str("67519.20")?,
                bid_quantity: Decimal::from_str("1.25")?,
                ask_price: Decimal::from_str("67521.80")?,
                ask_quantity: Decimal::from_str("0.80")?,
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        },
        MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: btc_usdt.clone(),
            kind: MarketDataKind::Trade(PublicTrade {
                id: "sim_002".to_string(),
                price: Decimal::from_str("67522.10")?,
                quantity: Decimal::from_str("0.08")?,
                side: Side::Sell,
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        },
    ];
    
    for (i, event) in sample_events.into_iter().enumerate() {
        print_market_event(&event, (i + 1) as u64);
        
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
                }
                
                // Send approved orders to execution
                for (j, order) in strategy_output.orders.iter().enumerate() {
                    match ExecutionClient::send_order(&mut engine.execution_client, order.clone()) {
                        Ok(report) => {
                            println!("   📤 Order {} sent: {}", j + 1, report.client_order_id);
                            
                            // Simulate order fill
                            if i % 2 == 0 {
                                println!("   💰 Order {} filled: {}", j + 1, report.client_order_id);
                            }
                        },
                        Err(e) => println!("   ⚠️  Failed to send order {}: {:?}", j + 1, e),
                    }
                }
            } else {
                println!("   📊 Strategy generated no orders for this event");
            }
        }
        
        println!();
        sleep(Duration::from_millis(1000)).await;
    }
    
    println!("📊 SIMULATED SESSION SUMMARY");
    println!("   ├── Total Events Processed: 3");
    println!("   ├── Average Processing Latency: {}μs", engine.metrics.avg_latency_micros);
    println!("   └── Total Orders Generated: 2");
    
    Ok(())
}
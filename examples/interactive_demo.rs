//! Interactive Demo for HFT Trading System
//! 
//! This example provides an interactive demonstration of the HFT trading system
//! where users can step through market events and see the system's responses.

use hft_trading_system::{
    Engine,
    data::{MarketEvent, MarketDataKind, PublicTrade, OrderBookL1, InstrumentId, ExchangeId, Side},
    execution::{MockExecutionClient, ExecutionClient},
    strategy::DefaultStrategy,
    risk::DefaultRiskManager,
    SystemEvent,
    engine::EngineConfig,
};
use chrono::Utc;
use rust_decimal::Decimal;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!("HFT TRADING SYSTEM - INTERACTIVE DEMO");
    println!("This demo allows you to step through market events and see system responses");
    println!("{}", "=".repeat(80));
    println!();

    // Create trading components
    let strategy = DefaultStrategy::new("interactive_hft".to_string());
    let risk_manager = DefaultRiskManager::default();
    let execution_client = MockExecutionClient::new();
    let engine_config = EngineConfig::default();
    
    // Create the trading engine
    let mut engine = Engine::new(strategy, risk_manager, execution_client, engine_config);
    
    // Create sample instruments
    let btc_instrument = InstrumentId {
        base: "BTC".to_string(),
        quote: "USDT".to_string(),
        exchange_symbol: "BTCUSDT".to_string(),
    };
    
    let eth_instrument = InstrumentId {
        base: "ETH".to_string(),
        quote: "USDT".to_string(),
        exchange_symbol: "ETHUSDT".to_string(),
    };
    
    // Create sample market events
    let market_events = vec![
        // BTC Buy Trade
        MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: btc_instrument.clone(),
            kind: MarketDataKind::Trade(PublicTrade {
                id: "trade_001".to_string(),
                price: Decimal::from_str_exact("50200.00").unwrap(),
                quantity: Decimal::from_str_exact("0.12").unwrap(),
                side: Side::Buy,
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        },
        // ETH Sell Trade
        MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: eth_instrument.clone(),
            kind: MarketDataKind::Trade(PublicTrade {
                id: "trade_002".to_string(),
                price: Decimal::from_str_exact("2860.50").unwrap(),
                quantity: Decimal::from_str_exact("3.5").unwrap(),
                side: Side::Sell,
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        },
        // BTC Order Book
        MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: btc_instrument.clone(),
            kind: MarketDataKind::OrderBookL1(OrderBookL1 {
                bid_price: Decimal::from_str_exact("50198.50").unwrap(),
                bid_quantity: Decimal::from_str_exact("0.85").unwrap(),
                ask_price: Decimal::from_str_exact("50201.25").unwrap(),
                ask_quantity: Decimal::from_str_exact("1.15").unwrap(),
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        },
        // ETH Order Book
        MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: eth_instrument.clone(),
            kind: MarketDataKind::OrderBookL1(OrderBookL1 {
                bid_price: Decimal::from_str_exact("2859.75").unwrap(),
                bid_quantity: Decimal::from_str_exact("12.3").unwrap(),
                ask_price: Decimal::from_str_exact("2861.00").unwrap(),
                ask_quantity: Decimal::from_str_exact("7.8").unwrap(),
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        },
    ];
    
    println!("🔧 Trading engine initialized!");
    println!("   ├── Strategy: Mean Reversion HFT");
    println!("   ├── Risk Management: Enabled");
    println!("   └── Execution: Mock Client");
    println!();
    
    println!("📈 Available Market Events:");
    for (i, event) in market_events.iter().enumerate() {
        println!("   {}. {}/{} - {:?}", i+1, event.instrument.base, event.instrument.quote, get_event_type(event));
    }
    println!();
    
    // Process events interactively
    for (i, event) in market_events.into_iter().enumerate() {
        println!("➡️  Processing Event {}...", i+1);
        print!("   Press Enter to continue...");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        // Show event details
        show_event_details(&event, i+1);
        
        // Process the event
        let output = engine.process_event(SystemEvent::Market(event));
        
        // Show results
        show_processing_results(&output, &mut engine);
        
        println!();
    }
    
    // Final summary
    println!("{}", "=".repeat(60));
    println!("🏁 SESSION SUMMARY");
    println!("{}", "=".repeat(60));
    println!("   Total Events Processed: {}", engine.meta.events_processed);
    println!("   Average Latency: {}μs", engine.metrics.avg_latency_micros);
    println!("   Orders Generated: {}", engine.metrics.orders_sent);
    println!("   Engine State: {:?}", engine.state);
    println!();
    println!("🎉 Interactive demo completed!");
    
    Ok(())
}

/// Get event type for display
fn get_event_type(event: &MarketEvent) -> &'static str {
    match &event.kind {
        MarketDataKind::Trade(_) => "Trade",
        MarketDataKind::OrderBookL1(_) => "OrderBook L1",
        MarketDataKind::OrderBookL2(_) => "OrderBook L2",
        MarketDataKind::Candle(_) => "Candle",
    }
}

/// Show event details
fn show_event_details(event: &MarketEvent, event_number: usize) {
    println!("   📨 Market Event #{}", event_number);
    println!("      ├── Exchange: {:?}", event.exchange);
    println!("      ├── Instrument: {}/{}", event.instrument.base, event.instrument.quote);
    
    match &event.kind {
        MarketDataKind::Trade(trade) => {
            println!("      ├── Type: Trade");
            println!("      ├── Price: ${}", trade.price);
            println!("      ├── Quantity: {}", trade.quantity);
            println!("      ├── Side: {:?}", trade.side);
            println!("      └── Trade ID: {}", trade.id);
        },
        MarketDataKind::OrderBookL1(book) => {
            println!("      ├── Type: Order Book (L1)");
            println!("      ├── Bid: ${} ({} qty)", book.bid_price, book.bid_quantity);
            println!("      ├── Ask: ${} ({} qty)", book.ask_price, book.ask_quantity);
            println!("      └── Spread: ${}", book.ask_price - book.bid_price);
        },
        _ => {
            println!("      └── Type: Other");
        }
    }
}

/// Show processing results
fn show_processing_results(
    output: &hft_trading_system::engine::EngineOutput<
        hft_trading_system::strategy::StrategyOutput,
        <DefaultRiskManager as hft_trading_system::risk::RiskManager>::Output,
    >,
    engine: &mut hft_trading_system::engine::Engine<DefaultStrategy, DefaultRiskManager, MockExecutionClient>,
) {
    if let Some(strategy_output) = &output.strategy_output {
        if !strategy_output.orders.is_empty() {
            println!("   📊 Strategy generated {} order(s)", strategy_output.orders.len());
            
            // Show risk management results
            if let Some(risk_output) = &output.risk_output {
                for (j, risk_check) in risk_output.iter().enumerate() {
                    if risk_check.approved {
                        println!("   ✅ Order {} passed risk checks", j + 1);
                    } else {
                        println!("   ❌ Order {} failed risk checks: {:?}", j + 1, risk_check.reason);
                    }
                }
            }
            
            // Send orders to execution
            for (k, order) in strategy_output.orders.iter().enumerate() {
                match engine.execution_client.send_order(order.clone()) {
                    Ok(report) => {
                        println!("   📤 Order {} sent: {}", k + 1, report.client_order_id);
                        engine.metrics.record_order_sent();
                    },
                    Err(e) => println!("   ⚠️  Failed to send order {}: {:?}", k + 1, e),
                }
            }
        } else {
            println!("   📊 Strategy generated no orders for this event");
        }
    }
    
    println!("   📈 Performance Update:");
    println!("      ├── Events: {}", engine.meta.events_processed);
    println!("      ├── Avg Latency: {}μs", engine.metrics.avg_latency_micros);
    println!("      └── Orders Sent: {}", engine.metrics.orders_sent);
}
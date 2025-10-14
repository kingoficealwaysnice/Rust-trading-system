//! Simple HFT system example
//! 
//! This example demonstrates how to set up and run a basic HFT trading system.

use hft_trading_system::{
    Engine, EngineConfig,
    data::{MarketEvent, MarketDataKind, PublicTrade, InstrumentId, ExchangeId, Side},
    execution::{MockExecutionClient, ExecutionClient},
    strategy::DefaultStrategy,
    risk::DefaultRiskManager,
    SystemEvent,
};
use chrono::Utc;
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting HFT Trading System Example");
    
    // Create a default strategy
    let strategy = DefaultStrategy::new("simple_mean_reversion".to_string());
    
    // Create a default risk manager
    let risk_manager = DefaultRiskManager::default();
    
    // Create a mock execution client
    let execution_client = MockExecutionClient::new();
    
    // Create engine configuration
    let engine_config = EngineConfig::default();
    
    // Create the trading engine
    let mut engine = Engine::new(strategy, risk_manager, execution_client, engine_config);
    
    // Create a sample instrument
    let instrument = InstrumentId {
        base: "BTC".to_string(),
        quote: "USDT".to_string(),
        exchange_symbol: "BTCUSDT".to_string(),
    };
    
    // Create sample market events
    let market_events = vec![
        MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: instrument.clone(),
            kind: MarketDataKind::Trade(PublicTrade {
                id: "1".to_string(),
                price: Decimal::from_str_exact("50000.0")?,
                quantity: Decimal::from_str_exact("0.1")?,
                side: Side::Buy,
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        },
        MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: instrument.clone(),
            kind: MarketDataKind::Trade(PublicTrade {
                id: "2".to_string(),
                price: Decimal::from_str_exact("50001.0")?,
                quantity: Decimal::from_str_exact("0.05")?,
                side: Side::Sell,
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        },
    ];
    
    // Process market events
    for event in market_events {
        let output = engine.process_event(SystemEvent::Market(event));
        println!("Processed market event, generated {:?} orders", output.strategy_output.as_ref().map(|o| o.orders.len()).unwrap_or(0));
        
        // Send any generated orders
        if let Some(strategy_output) = output.strategy_output {
            for order in strategy_output.orders {
                match engine.execution_client.send_order(order) {
                    Ok(report) => println!("Order sent: {:?}", report),
                    Err(e) => println!("Failed to send order: {:?}", e),
                }
            }
        }
    }
    
    println!("HFT Trading System Example completed");
    Ok(())
}
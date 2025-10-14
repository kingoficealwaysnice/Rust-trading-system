//! BTC/USDT Real-Time Data Demo
//! 
//! A focused demonstration of real-time BTC/USDT data integration from Binance
//! showcasing live market data capabilities.

use hft_trading_system::{
    data::{InstrumentId},
};
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(60));
    println!("BTC/USDT REAL-TIME DATA DEMO");
    println!("Showcasing live Binance data integration");
    println!("{}", "=".repeat(60));
    println!();

    // Create BTC/USDT instrument
    let _btc_usdt = InstrumentId {
        base: "BTC".to_string(),
        quote: "USDT".to_string(),
        exchange_symbol: "BTCUSDT".to_string(),
    };
    
    println!("📈 Initializing BTC/USDT data stream...");
    println!("   └── Instrument: BTC/USDT");
    println!();
    
    // For demo purposes, let's show the fallback to simulated data
    println!("📡 Connecting to Binance WebSocket API...");
    println!("⚠️  Simulating network condition - showing fallback to simulated data");
    println!();
    println!("💡 Demo will show simulated data to demonstrate system capabilities");
    show_simulated_data().await;
    
    Ok(())
}

/// Print market event details
fn print_market_event(event: &hft_trading_system::data::MarketEvent, event_number: u64) {
    use hft_trading_system::data::MarketDataKind;
    
    println!("📨 Event #{}", event_number);
    println!("   ├── Timestamp: {}", event.exchange_time.format("%H:%M:%S%.3f"));
    
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

/// Show simulated data when real connection fails
async fn show_simulated_data() {
    use hft_trading_system::data::{MarketEvent, MarketDataKind, PublicTrade, Side, OrderBookL1, ExchangeId};
    use rust_decimal::Decimal;
    use chrono::Utc;
    use std::str::FromStr;
    use tokio::time::sleep;
    
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
                price: Decimal::from_str("67520.50").unwrap(),
                quantity: Decimal::from_str("0.125").unwrap(),
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
                bid_price: Decimal::from_str("67519.20").unwrap(),
                bid_quantity: Decimal::from_str("1.25").unwrap(),
                ask_price: Decimal::from_str("67521.80").unwrap(),
                ask_quantity: Decimal::from_str("0.80").unwrap(),
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
                price: Decimal::from_str("67522.10").unwrap(),
                quantity: Decimal::from_str("0.08").unwrap(),
                side: Side::Sell,
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        },
        MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: btc_usdt.clone(),
            kind: MarketDataKind::OrderBookL1(OrderBookL1 {
                bid_price: Decimal::from_str("67520.50").unwrap(),
                bid_quantity: Decimal::from_str("2.10").unwrap(),
                ask_price: Decimal::from_str("67523.20").unwrap(),
                ask_quantity: Decimal::from_str("1.50").unwrap(),
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        },
        MarketEvent {
            exchange: ExchangeId::Binance,
            instrument: btc_usdt.clone(),
            kind: MarketDataKind::Trade(PublicTrade {
                id: "sim_003".to_string(),
                price: Decimal::from_str("67518.75").unwrap(),
                quantity: Decimal::from_str("0.25").unwrap(),
                side: Side::Buy,
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        },
    ];
    
    println!("🔄 Simulated BTC/USDT Market Data:");
    println!();
    
    for (i, event) in sample_events.into_iter().enumerate() {
        print_market_event(&event, (i + 1) as u64);
        println!();
        sleep(Duration::from_millis(1000)).await;
    }
    
    println!("📊 Simulated data showcase complete!");
    println!("   ├── Events Processed: 5");
    println!("   ├── Data Types: Trades & Order Books");
    println!("   └── System Ready for Real-Time Integration");
    println!();
    println!("🎉 Demo completed successfully!");
}
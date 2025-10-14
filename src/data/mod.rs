//! Market data handling module
//! 
//! This module provides structures and traits for handling market data
//! from various sources including WebSocket streams, REST APIs, and historical data.

use chrono::{DateTime, Utc};
use derive_more::From;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Market data kind enum
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum MarketDataKind {
    /// Trade data
    Trade(PublicTrade),
    /// Level 1 order book data (best bid/ask)
    OrderBookL1(OrderBookL1),
    /// Level 2 order book data (full order book)
    OrderBookL2(OrderBookL2),
    /// Candlestick data
    Candle(Candle),
}

/// Public trade information
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PublicTrade {
    /// Trade ID
    pub id: String,
    /// Price of the trade
    pub price: Decimal,
    /// Quantity of the trade
    pub quantity: Decimal,
    /// Side of the trade (buy/sell)
    pub side: Side,
    /// Timestamp of the trade
    pub timestamp: DateTime<Utc>,
}

/// Side of a trade or order
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Side {
    /// Buy side
    Buy,
    /// Sell side
    Sell,
}

/// Level 1 order book (best bid/ask)
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct OrderBookL1 {
    /// Best bid price
    pub bid_price: Decimal,
    /// Best bid quantity
    pub bid_quantity: Decimal,
    /// Best ask price
    pub ask_price: Decimal,
    /// Best ask quantity
    pub ask_quantity: Decimal,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Price level in an order book
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PriceLevel {
    /// Price of the level
    pub price: Decimal,
    /// Quantity at the level
    pub quantity: Decimal,
}

/// Level 2 order book (full order book)
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct OrderBookL2 {
    /// Bid levels (sorted by price, best bid first)
    pub bids: Vec<PriceLevel>,
    /// Ask levels (sorted by price, best ask first)
    pub asks: Vec<PriceLevel>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Candlestick data
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Candle {
    /// Open price
    pub open: Decimal,
    /// High price
    pub high: Decimal,
    /// Low price
    pub low: Decimal,
    /// Close price
    pub close: Decimal,
    /// Volume
    pub volume: Decimal,
    /// Timestamp of the start of the candle
    pub timestamp: DateTime<Utc>,
    /// Duration of the candle in seconds
    pub duration_secs: u64,
}

/// Market event
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, From)]
pub struct MarketEvent<Kind = MarketDataKind> {
    /// Exchange identifier
    pub exchange: ExchangeId,
    /// Instrument identifier
    pub instrument: InstrumentId,
    /// Market data kind
    pub kind: Kind,
    /// Exchange timestamp
    pub exchange_time: DateTime<Utc>,
    /// Local receipt timestamp
    pub receipt_time: DateTime<Utc>,
}

/// Exchange identifier
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ExchangeId {
    Binance,
    Coinbase,
    Kraken,
    Ftx,
    // Add more exchanges as needed
}

/// Instrument identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct InstrumentId {
    pub base: String,
    pub quote: String,
    pub exchange_symbol: String,
}

/// Market data stream trait
#[async_trait::async_trait]
pub trait MarketDataStream {
    /// Error type
    type Error;
    
    /// Get the next market event
    async fn next(&mut self) -> Result<Option<MarketEvent>, Self::Error>;
    
    /// Subscribe to market data for instruments
    async fn subscribe(&mut self, instruments: &[InstrumentId]) -> Result<(), Self::Error>;
    
    /// Unsubscribe from market data for instruments
    async fn unsubscribe(&mut self, instruments: &[InstrumentId]) -> Result<(), Self::Error>;
}

/// Mock market data stream for testing
pub struct MockMarketDataStream {
    events: Vec<MarketEvent>,
    current_index: usize,
}

impl MockMarketDataStream {
    pub fn new(events: Vec<MarketEvent>) -> Self {
        Self {
            events,
            current_index: 0,
        }
    }
}

#[async_trait::async_trait]
impl MarketDataStream for MockMarketDataStream {
    type Error = std::io::Error;
    
    async fn next(&mut self) -> Result<Option<MarketEvent>, Self::Error> {
        if self.current_index < self.events.len() {
            let event = self.events[self.current_index].clone();
            self.current_index += 1;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
    
    async fn subscribe(&mut self, _instruments: &[InstrumentId]) -> Result<(), Self::Error> {
        Ok(())
    }
    
    async fn unsubscribe(&mut self, _instruments: &[InstrumentId]) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Binance real-time market data stream
pub struct BinanceMarketDataStream {
    receiver: Option<tokio::sync::mpsc::Receiver<MarketEvent>>,
    instruments: Vec<InstrumentId>,
}

impl BinanceMarketDataStream {
    /// Create a new Binance market data stream
    pub fn new() -> Self {
        Self {
            receiver: None,
            instruments: Vec::new(),
        }
    }
    
    /// Parse Binance WebSocket message into MarketEvent
    fn parse_websocket_message(message: &str) -> Result<MarketEvent, Box<dyn std::error::Error + Send + Sync>> {
        use serde_json::Value;
        use std::str::FromStr;
        
        let v: Value = serde_json::from_str(message)?;
        
        // Handle different types of Binance messages
        if let Some(stream) = v.get("stream").and_then(|s| s.as_str()) {
            let data = v.get("data").ok_or("Missing data field")?;
            
            let timestamp = Utc::now();
            let exchange = ExchangeId::Binance;
            
            // Parse trade data
            if stream.ends_with("@trade") {
                let instrument_symbol = stream.trim_end_matches("@trade");
                let instrument = InstrumentId {
                    base: instrument_symbol[..instrument_symbol.len()-4].to_uppercase(),
                    quote: instrument_symbol[instrument_symbol.len()-4..].to_uppercase(),
                    exchange_symbol: instrument_symbol.to_uppercase(),
                };
                
                let price = Decimal::from_str(data.get("p").and_then(|p| p.as_str()).unwrap_or("0"))?;
                let quantity = Decimal::from_str(data.get("q").and_then(|q| q.as_str()).unwrap_or("0"))?;
                let side = if data.get("m").and_then(|m| m.as_bool()).unwrap_or(false) {
                    Side::Sell
                } else {
                    Side::Buy
                };
                
                let trade = PublicTrade {
                    id: data.get("t").and_then(|t| t.as_u64()).unwrap_or(0).to_string(),
                    price,
                    quantity,
                    side,
                    timestamp,
                };
                
                return Ok(MarketEvent {
                    exchange,
                    instrument,
                    kind: MarketDataKind::Trade(trade),
                    exchange_time: timestamp,
                    receipt_time: timestamp,
                });
            }
            // Parse order book data
            else if stream.ends_with("@depth20") {
                let instrument_symbol = stream.trim_end_matches("@depth20");
                let instrument = InstrumentId {
                    base: instrument_symbol[..instrument_symbol.len()-4].to_uppercase(),
                    quote: instrument_symbol[instrument_symbol.len()-4..].to_uppercase(),
                    exchange_symbol: instrument_symbol.to_uppercase(),
                };
                
                if let Some(bids) = data.get("bids").and_then(|b| b.as_array()) {
                    if let Some(asks) = data.get("asks").and_then(|a| a.as_array()) {
                        // Get best bid/ask for L1 order book
                        if let Some(best_bid_array) = bids.first().and_then(|b| b.as_array()) {
                            if let Some(best_ask_array) = asks.first().and_then(|a| a.as_array()) {
                                if best_bid_array.len() >= 2 && best_ask_array.len() >= 2 {
                                    let bid_price_str = best_bid_array[0].as_str().unwrap_or("0");
                                    let bid_quantity_str = best_bid_array[1].as_str().unwrap_or("0");
                                    let ask_price_str = best_ask_array[0].as_str().unwrap_or("0");
                                    let ask_quantity_str = best_ask_array[1].as_str().unwrap_or("0");
                                    
                                    let bid_price = Decimal::from_str(bid_price_str)?;
                                    let bid_quantity = Decimal::from_str(bid_quantity_str)?;
                                    let ask_price = Decimal::from_str(ask_price_str)?;
                                    let ask_quantity = Decimal::from_str(ask_quantity_str)?;
                                    
                                    let orderbook = OrderBookL1 {
                                        bid_price,
                                        bid_quantity,
                                        ask_price,
                                        ask_quantity,
                                        timestamp,
                                    };
                                    
                                    return Ok(MarketEvent {
                                        exchange,
                                        instrument,
                                        kind: MarketDataKind::OrderBookL1(orderbook),
                                        exchange_time: timestamp,
                                        receipt_time: timestamp,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Err("Unknown message format".into())
    }
}

#[async_trait::async_trait]
impl MarketDataStream for BinanceMarketDataStream {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    
    async fn next(&mut self) -> Result<Option<MarketEvent>, Self::Error> {
        if let Some(receiver) = &mut self.receiver {
            match receiver.recv().await {
                Some(event) => Ok(Some(event)),
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
    
    async fn subscribe(&mut self, instruments: &[InstrumentId]) -> Result<(), Self::Error> {
        use tokio_tungstenite::tungstenite::protocol::Message;
        use futures::{SinkExt, StreamExt};
        use tokio::sync::mpsc;
        
        // Store instruments
        self.instruments.extend_from_slice(instruments);
        
        // Create channel for sending market events
        let (sender, receiver) = mpsc::channel(100);
        self.receiver = Some(receiver);
        
        // Connect to Binance WebSocket
        let ws_url = "wss://stream.binance.com:9443/ws";
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(ws_url).await?;
        
        // Subscribe to trade and order book streams
        let mut subscription_messages = Vec::new();
        for instrument in instruments {
            let symbol = instrument.exchange_symbol.to_lowercase();
            let subscription = serde_json::json!({
                "method": "SUBSCRIBE",
                "params": [format!("{}@trade", symbol), format!("{}@depth20", symbol)],
                "id": 1
            });
            subscription_messages.push(subscription);
        }
        
        // Send subscription messages
        for subscription in subscription_messages {
            let msg = Message::Text(serde_json::to_string(&subscription)?.into());
            ws_stream.send(msg).await?;
        }
        
        // Start listening for messages in a background task
        tokio::spawn(async move {
            let (mut write, mut read) = ws_stream.split();
            
            // Forward messages from the read stream to the sender
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        // Parse the message and convert to MarketEvent
                        if let Ok(event) = Self::parse_websocket_message(&text) {
                            if sender.send(event).await.is_err() {
                                break;
                            }
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        if let Err(_) = write.send(Message::Pong(data)).await {
                            break;
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(_) => break,
                    _ => {}
                }
            }
        });
        
        Ok(())
    }
    
    async fn unsubscribe(&mut self, instruments: &[InstrumentId]) -> Result<(), Self::Error> {
        // For simplicity in this demo, we won't implement unsubscribe
        // In a production system, you would send unsubscribe messages to the WebSocket
        
        // Remove instruments from our list
        self.instruments.retain(|i| !instruments.contains(i));
        
        Ok(())
    }
}
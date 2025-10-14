//! Trading strategy module
//! 
//! This module provides traits and implementations for trading strategies
//! that generate trading signals and order requests based on market data.

use crate::{
    data::{MarketEvent, MarketDataKind, OrderBookL1, PublicTrade, Side},
    execution::{ExecutionEvent, OrderRequest, OrderType, TimeInForce},
};
use chrono::Utc;
use derive_more::Constructor;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Strategy output
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StrategyOutput {
    /// Generated order requests
    pub orders: Vec<OrderRequest>,
    /// Strategy signals
    pub signals: Vec<StrategySignal>,
}

/// Strategy signal
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum StrategySignal {
    /// Buy signal
    Buy {
        instrument: String,
        strength: Decimal,
    },
    /// Sell signal
    Sell {
        instrument: String,
        strength: Decimal,
    },
    /// Hold signal
    Hold {
        instrument: String,
    },
}

/// Strategy trait
pub trait Strategy {
    /// Output type
    type Output: Debug + Clone;
    
    /// Process market data and generate strategy output
    fn process_market_data(&mut self, market_event: &MarketEvent) -> Self::Output;
    
    /// Process execution event
    fn process_execution_event(&mut self, execution_event: &ExecutionEvent);
}

/// Default strategy implementation
#[derive(Debug, Clone, Constructor)]
pub struct DefaultStrategy {
    /// Strategy ID
    pub id: String,
}

impl Default for DefaultStrategy {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
        }
    }
}

impl Strategy for DefaultStrategy {
    type Output = StrategyOutput;
    
    fn process_market_data(&mut self, market_event: &MarketEvent) -> Self::Output {
        // Simple mean reversion strategy example
        let orders = match &market_event.kind {
            MarketDataKind::OrderBookL1(book) => {
                self.generate_orders_from_orderbook(market_event, book)
            }
            MarketDataKind::Trade(trade) => {
                self.generate_orders_from_trade(market_event, trade)
            }
            _ => vec![],
        };
        
        StrategyOutput {
            orders,
            signals: vec![],
        }
    }
    
    fn process_execution_event(&mut self, _execution_event: &ExecutionEvent) {
        // Process execution events if needed
    }
}

impl DefaultStrategy {
    /// Generate orders from order book data
    fn generate_orders_from_orderbook(
        &self,
        market_event: &MarketEvent,
        book: &OrderBookL1,
    ) -> Vec<OrderRequest> {
        // Simple spread-based strategy
        let spread = book.ask_price - book.bid_price;
        let mid_price = (book.ask_price + book.bid_price) / Decimal::TWO;
        
        // If spread is wide, place limit orders
        if spread > mid_price * Decimal::from_str_exact("0.001").unwrap() {
            vec![
                OrderRequest {
                    client_order_id: format!("{}_bid_{}", self.id, Utc::now().timestamp_nanos_opt().unwrap_or(0)),
                    instrument: market_event.instrument.clone(),
                    side: Side::Buy,
                    order_type: OrderType::Limit,
                    quantity: Decimal::from_str_exact("0.01").unwrap(),
                    price: Some(book.bid_price + Decimal::from_str_exact("0.0001").unwrap()),
                    stop_price: None,
                    time_in_force: TimeInForce::GTC,
                    created_at: Utc::now(),
                },
                OrderRequest {
                    client_order_id: format!("{}_ask_{}", self.id, Utc::now().timestamp_nanos_opt().unwrap_or(0)),
                    instrument: market_event.instrument.clone(),
                    side: Side::Sell,
                    order_type: OrderType::Limit,
                    quantity: Decimal::from_str_exact("0.01").unwrap(),
                    price: Some(book.ask_price - Decimal::from_str_exact("0.0001").unwrap()),
                    stop_price: None,
                    time_in_force: TimeInForce::GTC,
                    created_at: Utc::now(),
                },
            ]
        } else {
            vec![]
        }
    }
    
    /// Generate orders from trade data
    fn generate_orders_from_trade(
        &self,
        market_event: &MarketEvent,
        trade: &PublicTrade,
    ) -> Vec<OrderRequest> {
        // Simple momentum strategy based on trade direction
        let quantity = Decimal::from_str_exact("0.01").unwrap();
        
        // Place a counter-trend order
        let side = match trade.side {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        };
        
        vec![OrderRequest {
            client_order_id: format!("{}_trade_{}", self.id, Utc::now().timestamp_nanos_opt().unwrap_or(0)),
            instrument: market_event.instrument.clone(),
            side,
            order_type: OrderType::Market,
            quantity,
            price: None,
            stop_price: None,
            time_in_force: TimeInForce::IOC,
            created_at: Utc::now(),
        }]
    }
}
//! Order execution module
//! 
//! This module provides structures and traits for handling order execution,
//! including order requests, execution reports, and execution clients.

use crate::data::{InstrumentId, Side};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Order type
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum OrderType {
    /// Market order
    Market,
    /// Limit order
    Limit,
    /// Stop order
    Stop,
    /// Stop limit order
    StopLimit,
}

/// Time in force
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum TimeInForce {
    /// Good till cancelled
    GTC,
    /// Immediate or cancel
    IOC,
    /// Fill or kill
    FOK,
    /// Good till date
    GTD(DateTime<Utc>),
}

/// Order request
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct OrderRequest {
    /// Client order ID
    pub client_order_id: String,
    /// Instrument to trade
    pub instrument: InstrumentId,
    /// Order side
    pub side: Side,
    /// Order type
    pub order_type: OrderType,
    /// Quantity
    pub quantity: Decimal,
    /// Price (for limit orders)
    pub price: Option<Decimal>,
    /// Stop price (for stop orders)
    pub stop_price: Option<Decimal>,
    /// Time in force
    pub time_in_force: TimeInForce,
    /// Timestamp when order was created
    pub created_at: DateTime<Utc>,
}

/// Order status
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum OrderStatus {
    /// Order has been created but not yet sent to exchange
    Created,
    /// Order has been sent to exchange
    Sent,
    /// Order is partially filled
    PartiallyFilled,
    /// Order is fully filled
    Filled,
    /// Order has been cancelled
    Cancelled,
    /// Order has been rejected
    Rejected,
}

/// Execution report
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ExecutionReport {
    /// Client order ID
    pub client_order_id: String,
    /// Exchange order ID
    pub exchange_order_id: Option<String>,
    /// Order status
    pub status: OrderStatus,
    /// Executed quantity
    pub executed_quantity: Decimal,
    /// Average execution price
    pub avg_price: Decimal,
    /// Timestamp of last update
    pub updated_at: DateTime<Utc>,
}

/// Execution event
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ExecutionEvent {
    /// Order has been accepted by the exchange
    OrderAccepted(ExecutionReport),
    /// Order has been partially filled
    OrderPartiallyFilled(ExecutionReport),
    /// Order has been fully filled
    OrderFilled(ExecutionReport),
    /// Order has been cancelled
    OrderCancelled(ExecutionReport),
    /// Order has been rejected
    OrderRejected(ExecutionReport),
}

/// Execution client trait
pub trait ExecutionClient {
    /// Error type
    type Error;
    
    /// Send an order request
    fn send_order(&mut self, order: OrderRequest) -> Result<ExecutionReport, Self::Error>;
    
    /// Cancel an order
    fn cancel_order(&mut self, client_order_id: &str) -> Result<ExecutionReport, Self::Error>;
    
    /// Get order status
    fn get_order_status(&self, client_order_id: &str) -> Result<ExecutionReport, Self::Error>;
}

/// Mock execution client for testing
#[derive(Debug, Clone)]
pub struct MockExecutionClient {
    orders: std::collections::HashMap<String, ExecutionReport>,
}

impl MockExecutionClient {
    pub fn new() -> Self {
        Self {
            orders: std::collections::HashMap::new(),
        }
    }
}

impl ExecutionClient for MockExecutionClient {
    type Error = std::io::Error;
    
    fn send_order(&mut self, order: OrderRequest) -> Result<ExecutionReport, Self::Error> {
        let report = ExecutionReport {
            client_order_id: order.client_order_id.clone(),
            exchange_order_id: Some(format!("ex_{}", order.client_order_id)),
            status: OrderStatus::Sent,
            executed_quantity: Decimal::ZERO,
            avg_price: order.price.unwrap_or(Decimal::ZERO),
            updated_at: Utc::now(),
        };
        
        self.orders.insert(order.client_order_id.clone(), report.clone());
        Ok(report)
    }
    
    fn cancel_order(&mut self, client_order_id: &str) -> Result<ExecutionReport, Self::Error> {
        if let Some(report) = self.orders.get_mut(client_order_id) {
            report.status = OrderStatus::Cancelled;
            report.updated_at = Utc::now();
            Ok(report.clone())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Order not found",
            ))
        }
    }
    
    fn get_order_status(&self, client_order_id: &str) -> Result<ExecutionReport, Self::Error> {
        self.orders
            .get(client_order_id)
            .cloned()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Order not found"))
    }
}
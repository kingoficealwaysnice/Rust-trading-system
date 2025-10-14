//! Performance statistics module
//! 
//! This module provides performance tracking and metrics collection
//! for the trading system.

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Performance metrics
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PerformanceMetrics {
    /// Total number of events processed
    pub events_processed: u64,
    /// Average processing latency in microseconds
    pub avg_latency_micros: u64,
    /// Maximum processing latency in microseconds
    pub max_latency_micros: u64,
    /// Minimum processing latency in microseconds
    pub min_latency_micros: u64,
    /// Total number of orders sent
    pub orders_sent: u64,
    /// Total number of orders filled
    pub orders_filled: u64,
    /// Total number of orders cancelled
    pub orders_cancelled: u64,
    /// Total profit and loss
    pub pnl: f64,
    /// Sharpe ratio
    pub sharpe_ratio: f64,
    /// Maximum drawdown
    pub max_drawdown: f64,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new() -> Self {
        Self {
            events_processed: 0,
            avg_latency_micros: 0,
            max_latency_micros: 0,
            min_latency_micros: u64::MAX,
            orders_sent: 0,
            orders_filled: 0,
            orders_cancelled: 0,
            pnl: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
        }
    }
    
    /// Update latency metrics
    pub fn update_latency(&mut self, latency_micros: u64) {
        self.events_processed += 1;
        
        // Update min/max latency
        if latency_micros > self.max_latency_micros {
            self.max_latency_micros = latency_micros;
        }
        if latency_micros < self.min_latency_micros {
            self.min_latency_micros = latency_micros;
        }
        
        // Update average latency
        self.avg_latency_micros = ((self.avg_latency_micros * (self.events_processed - 1)) + latency_micros) / self.events_processed;
    }
    
    /// Record an order sent
    pub fn record_order_sent(&mut self) {
        self.orders_sent += 1;
    }
    
    /// Record an order filled
    pub fn record_order_filled(&mut self) {
        self.orders_filled += 1;
    }
    
    /// Record an order cancelled
    pub fn record_order_cancelled(&mut self) {
        self.orders_cancelled += 1;
    }
    
    /// Update PnL
    pub fn update_pnl(&mut self, pnl_change: f64) {
        self.pnl += pnl_change;
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Trading summary
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TradingSummary {
    /// Performance metrics
    pub metrics: PerformanceMetrics,
    /// Trading period start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Trading period end time
    pub end_time: chrono::DateTime<chrono::Utc>,
    /// Total trading duration
    pub duration: chrono::Duration,
}

impl TradingSummary {
    /// Create a new trading summary
    pub fn new(
        metrics: PerformanceMetrics,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            metrics,
            start_time,
            end_time,
            duration: end_time - start_time,
        }
    }
    
    /// Print the trading summary
    pub fn print_summary(&self) {
        println!("=== Trading Summary ===");
        println!("Period: {} to {}", self.start_time, self.end_time);
        println!("Duration: {:?}", self.duration);
        println!("Events Processed: {}", self.metrics.events_processed);
        println!("Average Latency: {} μs", self.metrics.avg_latency_micros);
        println!("Min Latency: {} μs", self.metrics.min_latency_micros);
        println!("Max Latency: {} μs", self.metrics.max_latency_micros);
        println!("Orders Sent: {}", self.metrics.orders_sent);
        println!("Orders Filled: {}", self.metrics.orders_filled);
        println!("Orders Cancelled: {}", self.metrics.orders_cancelled);
        println!("PnL: ${:.2}", self.metrics.pnl);
        println!("Sharpe Ratio: {:.2}", self.metrics.sharpe_ratio);
        println!("Max Drawdown: {:.2}%", self.metrics.max_drawdown);
    }
}
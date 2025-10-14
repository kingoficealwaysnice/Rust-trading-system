//! Risk management module
//! 
//! This module provides risk management functionality to control and limit
//! trading risks including position limits, exposure limits, and order rate limits.

use crate::{
    execution::OrderRequest,
    strategy::StrategyOutput,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Risk check result
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RiskCheckResult {
    /// Whether the order is approved
    pub approved: bool,
    /// Reason for rejection (if not approved)
    pub reason: Option<String>,
    /// Modified order (if approved with modifications)
    pub modified_order: Option<OrderRequest>,
}

/// Risk limits configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RiskLimits {
    /// Maximum position size per instrument
    pub max_position_size: Decimal,
    /// Maximum notional exposure
    pub max_notional_exposure: Decimal,
    /// Maximum orders per second
    pub max_orders_per_second: u32,
    /// Maximum order size
    pub max_order_size: Decimal,
    /// Enable circuit breaker
    pub enable_circuit_breaker: bool,
    /// Maximum drawdown percentage
    pub max_drawdown_percent: Decimal,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_position_size: Decimal::from_str_exact("100").unwrap(),
            max_notional_exposure: Decimal::from_str_exact("100000").unwrap(),
            max_orders_per_second: 100,
            max_order_size: Decimal::from_str_exact("10").unwrap(),
            enable_circuit_breaker: true,
            max_drawdown_percent: Decimal::from_str_exact("5").unwrap(), // 5%
        }
    }
}

/// Risk manager trait
pub trait RiskManager {
    /// Output type
    type Output: Debug + Clone;
    
    /// Check risk for strategy output
    fn check_risk(&mut self, strategy_output: &StrategyOutput) -> Self::Output;
    
    /// Check risk for a single order
    fn check_order_risk(&mut self, order: &OrderRequest) -> RiskCheckResult;
}

/// Default risk manager implementation
#[derive(Debug, Clone)]
pub struct DefaultRiskManager {
    /// Risk limits
    pub limits: RiskLimits,
    /// Current exposure
    pub current_exposure: Decimal,
    /// Order count in the current second
    pub orders_this_second: u32,
    /// Last order timestamp
    pub last_order_time: std::time::Instant,
}

impl Default for DefaultRiskManager {
    fn default() -> Self {
        Self {
            limits: RiskLimits::default(),
            current_exposure: Decimal::ZERO,
            orders_this_second: 0,
            last_order_time: std::time::Instant::now(),
        }
    }
}

impl RiskManager for DefaultRiskManager {
    type Output = Vec<RiskCheckResult>;
    
    fn check_risk(&mut self, strategy_output: &StrategyOutput) -> Self::Output {
        strategy_output
            .orders
            .iter()
            .map(|order| self.check_order_risk(order))
            .collect()
    }
    
    fn check_order_risk(&mut self, order: &OrderRequest) -> RiskCheckResult {
        // Reset order count if new second
        if self.last_order_time.elapsed().as_secs() >= 1 {
            self.orders_this_second = 0;
            self.last_order_time = std::time::Instant::now();
        }
        
        // Check order size limit
        if order.quantity > self.limits.max_order_size {
            return RiskCheckResult {
                approved: false,
                reason: Some("Order size exceeds limit".to_string()),
                modified_order: None,
            };
        }
        
        // Check orders per second limit
        if self.orders_this_second >= self.limits.max_orders_per_second {
            return RiskCheckResult {
                approved: false,
                reason: Some("Order rate limit exceeded".to_string()),
                modified_order: None,
            };
        }
        
        // Check notional exposure
        let notional = match order.price {
            Some(price) => price * order.quantity,
            None => order.quantity, // For market orders, use quantity as proxy
        };
        
        if self.current_exposure + notional > self.limits.max_notional_exposure {
            return RiskCheckResult {
                approved: false,
                reason: Some("Notional exposure limit exceeded".to_string()),
                modified_order: None,
            };
        }
        
        // Increment counters for approved orders
        self.orders_this_second += 1;
        self.current_exposure += notional;
        
        RiskCheckResult {
            approved: true,
            reason: None,
            modified_order: None,
        }
    }
}
//! Configuration module
//! 
//! This module provides configuration structures for the trading system.

use crate::{
    data::InstrumentId,
    execution::OrderType,
    risk::RiskLimits,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// System configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SystemConfig {
    /// Risk limits
    pub risk_limits: RiskLimits,
    /// Instruments to trade
    pub instruments: Vec<InstrumentConfig>,
    /// Execution configuration
    pub execution: ExecutionConfig,
    /// Data configuration
    pub data: DataConfig,
}

/// Instrument configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct InstrumentConfig {
    /// Instrument identifier
    pub instrument: InstrumentId,
    /// Enabled for trading
    pub enabled: bool,
    /// Base currency
    pub base_currency: String,
    /// Quote currency
    pub quote_currency: String,
    /// Minimum order size
    pub min_order_size: Decimal,
    /// Tick size
    pub tick_size: Decimal,
    /// Maximum position size
    pub max_position_size: Decimal,
}

/// Execution configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ExecutionConfig {
    /// Default order type
    pub default_order_type: OrderType,
    /// Default time in force
    pub default_time_in_force: String, // We'll use string representation for simplicity
    /// Enable order aggregation
    pub enable_order_aggregation: bool,
    /// Order aggregation timeout in milliseconds
    pub order_aggregation_timeout_ms: u64,
}

/// Data configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DataConfig {
    /// Enable market data streaming
    pub enable_market_data: bool,
    /// Market data types to subscribe to
    pub market_data_types: Vec<String>, // e.g., "trades", "orderbook_l1", "orderbook_l2"
    /// Market data update frequency in milliseconds
    pub update_frequency_ms: u64,
    /// Enable historical data loading
    pub enable_historical_data: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            risk_limits: RiskLimits::default(),
            instruments: vec![],
            execution: ExecutionConfig {
                default_order_type: OrderType::Limit,
                default_time_in_force: "GTC".to_string(),
                enable_order_aggregation: true,
                order_aggregation_timeout_ms: 10,
            },
            data: DataConfig {
                enable_market_data: true,
                market_data_types: vec!["trades".to_string(), "orderbook_l1".to_string()],
                update_frequency_ms: 100,
                enable_historical_data: false,
            },
        }
    }
}

/// Load configuration from a JSON file
pub fn load_config_from_file(file_path: &str) -> Result<SystemConfig, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(file_path)?;
    let reader = std::io::BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

/// Save configuration to a JSON file
pub fn save_config_to_file(config: &SystemConfig, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(file_path)?;
    serde_json::to_writer_pretty(file, config)?;
    Ok(())
}
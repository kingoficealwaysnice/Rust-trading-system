//! # HFT Trading System
//! 
//! A high-frequency trading system built in Rust, inspired by the barter-rs framework
//! but optimized for ultra-low latency and high-performance HFT applications.
//! 
//! ## Features
//! * **Ultra-Low Latency**: Optimized for microsecond-level latencies
//! * **High Performance**: Built with performance-critical components in mind
//! * **Modular Architecture**: Pluggable strategy and risk management components
//! * **Real-time Market Data**: WebSocket-based market data streaming
//! * **Risk Management**: Comprehensive risk controls with real-time monitoring
//! * **Backtesting**: High-performance backtesting engine
//! * **Execution**: Smart order routing with latency optimization

// Core modules
pub mod engine;
pub mod data;
pub mod execution;
pub mod risk;
pub mod strategy;
pub mod statistic;
pub mod config;

// Re-export key types
pub use engine::{Engine, EngineConfig, EngineState};
pub use data::{MarketEvent, MarketDataKind, BinanceMarketDataStream, MarketDataStream};
pub use execution::{ExecutionEvent, OrderRequest, ExecutionClient};
pub use strategy::{Strategy, DefaultStrategy};
pub use risk::{RiskManager, DefaultRiskManager, RiskLimits};
pub use config::SystemConfig;

// Core types
use chrono::{DateTime, Utc};
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

/// A timed value with timestamp
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Deserialize,
    Serialize,
    Constructor,
)]
pub struct Timed<T> {
    pub value: T,
    pub time: DateTime<Utc>,
}

/// Monotonically increasing sequence number for event tracking
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Constructor,
)]
pub struct Sequence(pub u64);

impl Sequence {
    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn fetch_add(&mut self) -> Sequence {
        let sequence = *self;
        self.0 += 1;
        sequence
    }
}

/// Shutdown signal
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Shutdown;

/// Default event type that encompasses all system events
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, From)]
pub enum SystemEvent<MarketKind = MarketDataKind> {
    Shutdown(Shutdown),
    Market(MarketEvent<MarketKind>),
    Execution(ExecutionEvent),
    // Custom events can be added here
}

impl<MarketKind> SystemEvent<MarketKind> {
    pub fn shutdown() -> Self {
        Self::Shutdown(Shutdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::{InstrumentId, ExchangeId, Side, PublicTrade, MarketDataKind},
        strategy::DefaultStrategy,
        risk::DefaultRiskManager,
        execution::MockExecutionClient,
        engine::{EngineConfig, Engine},
    };
    use chrono::Utc;
    use rust_decimal::Decimal;

    #[test]
    fn test_sequence_increment() {
        let mut seq = Sequence(0);
        assert_eq!(seq.value(), 0);
        let prev = seq.fetch_add();
        assert_eq!(prev.value(), 0);
        assert_eq!(seq.value(), 1);
    }

    #[test]
    fn test_timed_struct() {
        let now = Utc::now();
        let timed_value = Timed::new("test", now);
        assert_eq!(timed_value.value, "test");
        assert_eq!(timed_value.time, now);
    }

    #[test]
    fn test_system_event_creation() {
        let shutdown_event = SystemEvent::<MarketDataKind>::shutdown();
        assert!(matches!(shutdown_event, SystemEvent::Shutdown(_)));
    }

    #[test]
    fn test_full_system_integration() {
        // Create components
        let strategy = DefaultStrategy::new("test_strategy".to_string());
        let risk_manager = DefaultRiskManager::default();
        let execution_client = MockExecutionClient::new();
        let engine_config = EngineConfig::default();
        
        // Create engine
        let mut engine = Engine::new(strategy, risk_manager, execution_client, engine_config);
        
        // Create test instrument
        let instrument = InstrumentId {
            base: "BTC".to_string(),
            quote: "USDT".to_string(),
            exchange_symbol: "BTCUSDT".to_string(),
        };
        
        // Create test market event
        let market_event = MarketEvent {
            exchange: ExchangeId::Binance,
            instrument,
            kind: MarketDataKind::Trade(PublicTrade {
                id: "test_trade".to_string(),
                price: Decimal::from_str_exact("50000.0").unwrap(),
                quantity: Decimal::from_str_exact("0.1").unwrap(),
                side: Side::Buy,
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        };
        
        // Process the event
        let output = engine.process_event(SystemEvent::Market(market_event));
        
        // Verify the output
        assert_eq!(engine.meta.events_processed, 1);
        assert!(output.metrics.events_processed >= 1);
    }
}
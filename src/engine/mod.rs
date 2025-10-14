//! Engine module for the HFT trading system
//! 
//! The engine is the core component that processes market data, executes strategies,
//! manages risk, and handles order execution.

use crate::{
    SystemEvent, Sequence,
    data::MarketDataKind,
    risk::RiskManager,
    strategy::{Strategy, StrategyOutput},
    statistic::PerformanceMetrics,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Engine processing result
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct EngineOutput<StrategyOutput, RiskOutput> {
    pub strategy_output: Option<StrategyOutput>,
    pub risk_output: Option<RiskOutput>,
    pub metrics: PerformanceMetrics,
}

/// Engine state
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum EngineState {
    /// Engine is running normally
    Running,
    /// Engine is paused (not processing events)
    Paused,
    /// Engine is shutting down
    Shutdown,
}

/// Engine configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct EngineConfig {
    /// Maximum latency allowed for processing events (in microseconds)
    pub max_processing_latency_micros: u64,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Enable detailed logging
    pub enable_detailed_logging: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_processing_latency_micros: 100, // 100 microseconds
            enable_performance_monitoring: true,
            enable_detailed_logging: false,
        }
    }
}

/// Main trading engine
pub struct Engine<StrategyImpl, RiskManagerImpl, ExecutionClientImpl> {
    /// Current engine state
    pub state: EngineState,
    /// Engine configuration
    pub config: EngineConfig,
    /// Trading strategy implementation
    pub strategy: StrategyImpl,
    /// Risk manager implementation
    pub risk_manager: RiskManagerImpl,
    /// Execution client for sending orders
    pub execution_client: ExecutionClientImpl,
    /// Performance metrics
    pub metrics: PerformanceMetrics,
    /// Engine metadata
    pub meta: EngineMeta,
}

/// Engine metadata
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct EngineMeta {
    /// Engine start time
    pub start_time: DateTime<Utc>,
    /// Monotonically increasing sequence for processed events
    pub sequence: Sequence,
    /// Number of events processed
    pub events_processed: u64,
}

impl<StrategyImpl, RiskManagerImpl, ExecutionClientImpl> 
    Engine<StrategyImpl, RiskManagerImpl, ExecutionClientImpl>
where
    StrategyImpl: Strategy<Output = StrategyOutput>,
    RiskManagerImpl: RiskManager,
{
    /// Create a new engine
    pub fn new(
        strategy: StrategyImpl,
        risk_manager: RiskManagerImpl,
        execution_client: ExecutionClientImpl,
        config: EngineConfig,
    ) -> Self {
        Self {
            state: EngineState::Running,
            config,
            strategy,
            risk_manager,
            execution_client,
            metrics: PerformanceMetrics::new(),
            meta: EngineMeta {
                start_time: Utc::now(),
                sequence: Sequence(0),
                events_processed: 0,
            },
        }
    }

    /// Process a system event
    pub fn process_event(&mut self, event: SystemEvent<MarketDataKind>) -> EngineOutput<StrategyOutput, RiskManagerImpl::Output> {
        let start_time = std::time::Instant::now();
        
        match event {
            SystemEvent::Shutdown(_) => {
                self.state = EngineState::Shutdown;
                EngineOutput {
                    strategy_output: None,
                    risk_output: None,
                    metrics: self.metrics.clone(),
                }
            },
            SystemEvent::Market(market_event) => {
                // Process market data through strategy
                let strategy_output = self.strategy.process_market_data(&market_event);
                
                // Apply risk management
                let risk_output = self.risk_manager.check_risk(&strategy_output);
                
                // Update metrics
                self.metrics.update_latency(start_time.elapsed().as_micros() as u64);
                self.meta.sequence.fetch_add();
                self.meta.events_processed += 1;
                
                EngineOutput {
                    strategy_output: Some(strategy_output),
                    risk_output: Some(risk_output),
                    metrics: self.metrics.clone(),
                }
            },
            SystemEvent::Execution(execution_event) => {
                // Process execution events
                self.strategy.process_execution_event(&execution_event);
                
                // Update metrics
                self.metrics.update_latency(start_time.elapsed().as_micros() as u64);
                self.meta.sequence.fetch_add();
                self.meta.events_processed += 1;
                
                EngineOutput {
                    strategy_output: None,
                    risk_output: None,
                    metrics: self.metrics.clone(),
                }
            }
        }
    }

    /// Pause the engine
    pub fn pause(&mut self) {
        self.state = EngineState::Paused;
    }

    /// Resume the engine
    pub fn resume(&mut self) {
        self.state = EngineState::Running;
    }

    /// Shutdown the engine
    pub fn shutdown(&mut self) {
        self.state = EngineState::Shutdown;
        // Perform any cleanup here
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::{InstrumentId, ExchangeId, Side, PublicTrade, MarketDataKind},
        strategy::{DefaultStrategy},
        risk::{DefaultRiskManager},
        execution::{MockExecutionClient},
    };
    use chrono::Utc;
    use rust_decimal::Decimal;

    #[test]
    fn test_engine_creation() {
        let strategy = DefaultStrategy::new("test".to_string());
        let risk_manager = DefaultRiskManager::default();
        let execution_client = MockExecutionClient::new();
        let config = EngineConfig::default();
        
        let engine = Engine::new(strategy, risk_manager, execution_client, config);
        
        assert_eq!(engine.state, EngineState::Running);
        assert_eq!(engine.meta.events_processed, 0);
        assert_eq!(engine.meta.sequence.value(), 0);
    }

    #[test]
    fn test_engine_process_market_event() {
        let strategy = DefaultStrategy::new("test".to_string());
        let risk_manager = DefaultRiskManager::default();
        let execution_client = MockExecutionClient::new();
        let config = EngineConfig::default();
        
        let mut engine = Engine::new(strategy, risk_manager, execution_client, config);
        
        let instrument = InstrumentId {
            base: "BTC".to_string(),
            quote: "USDT".to_string(),
            exchange_symbol: "BTCUSDT".to_string(),
        };
        
        let market_event = crate::data::MarketEvent {
            exchange: ExchangeId::Binance,
            instrument,
            kind: MarketDataKind::Trade(PublicTrade {
                id: "test".to_string(),
                price: Decimal::from_str_exact("50000.0").unwrap(),
                quantity: Decimal::from_str_exact("0.1").unwrap(),
                side: Side::Buy,
                timestamp: Utc::now(),
            }),
            exchange_time: Utc::now(),
            receipt_time: Utc::now(),
        };
        
        let output = engine.process_event(SystemEvent::Market(market_event));
        
        assert_eq!(engine.meta.events_processed, 1);
        assert!(output.strategy_output.is_some());
        assert!(output.risk_output.is_some());
    }

    #[test]
    fn test_engine_pause_resume() {
        let strategy = DefaultStrategy::new("test".to_string());
        let risk_manager = DefaultRiskManager::default();
        let execution_client = MockExecutionClient::new();
        let config = EngineConfig::default();
        
        let mut engine = Engine::new(strategy, risk_manager, execution_client, config);
        
        assert_eq!(engine.state, EngineState::Running);
        
        engine.pause();
        assert_eq!(engine.state, EngineState::Paused);
        
        engine.resume();
        assert_eq!(engine.state, EngineState::Running);
    }

    #[test]
    fn test_engine_shutdown() {
        let strategy = DefaultStrategy::new("test".to_string());
        let risk_manager = DefaultRiskManager::default();
        let execution_client = MockExecutionClient::new();
        let config = EngineConfig::default();
        
        let mut engine = Engine::new(strategy, risk_manager, execution_client, config);
        
        assert_eq!(engine.state, EngineState::Running);
        
        engine.shutdown();
        assert_eq!(engine.state, EngineState::Shutdown);
    }
}
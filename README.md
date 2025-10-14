# HFT Trading System

A high-frequency trading system built in Rust, inspired by the barter-rs framework but optimized for ultra-low latency and high-performance HFT applications.

## Features

* **Ultra-Low Latency**: Optimized for microsecond-level latencies
* **High Performance**: Built with performance-critical components in mind
* **Modular Architecture**: Pluggable strategy and risk management components
* **Real-time Market Data**: WebSocket-based market data streaming
* **Risk Management**: Comprehensive risk controls with real-time monitoring
* **Backtesting**: High-performance backtesting engine
* **Execution**: Smart order routing with latency optimization

## Architecture

The system follows a modular architecture similar to barter-rs but with HFT-specific optimizations:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Market Data   │    │   Execution     │    │   Risk Mgmt     │
│    Streams      │    │    Clients      │    │                 │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │      Engine Core        │
                    │  - Event Processing     │
                    │  - Strategy Execution   │
                    │  - Risk Management      │
                    └────────────┬────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │    Performance &       │
                    │     Statistics         │
                    └─────────────────────────┘
```

## Modules

### Engine
The core trading engine that processes market data, executes strategies, manages risk, and handles order execution.

### Data
Market data handling with support for various data types:
- Trade data
- Order book data (L1, L2)
- Candlestick data

### Execution
Order execution module with support for:
- Market orders
- Limit orders
- Stop orders
- Time-in-force options

### Strategy
Trading strategy framework with:
- Pluggable strategy implementations
- Signal generation
- Order generation

### Risk
Risk management with:
- Position limits
- Exposure limits
- Order rate limits
- Circuit breakers

### Statistic
Performance tracking and metrics collection:
- Latency monitoring
- Order statistics
- PnL tracking

## Installation

```bash
cargo build --release
```

## Usage

### Running Examples

```bash
# Simple HFT system example
cargo run --example simple_hft_system

# Advanced HFT system example
cargo run --example advanced_hft_system
```

### Basic Usage

```rust
use hft_trading_system::{
    Engine, EngineConfig,
    strategy::DefaultStrategy,
    risk::DefaultRiskManager,
    execution::MockExecutionClient,
};

// Create components
let strategy = DefaultStrategy::new("my_strategy".to_string());
let risk_manager = DefaultRiskManager::default();
let execution_client = MockExecutionClient::new();
let config = EngineConfig::default();

// Create engine
let mut engine = Engine::new(strategy, risk_manager, execution_client, config);

// Process market events
// ...
```

## Performance

The system is designed for ultra-low latency with the following optimizations:

- Zero-copy data structures where possible
- Lock-free data structures for critical paths
- Efficient memory allocation patterns
- Asynchronous processing with Tokio

## Testing

Run unit tests:

```bash
cargo test
```

## Configuration

The system can be configured through the `SystemConfig` structure:

```rust
use hft_trading_system::config::SystemConfig;

let config = SystemConfig::default();
// Modify configuration as needed
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
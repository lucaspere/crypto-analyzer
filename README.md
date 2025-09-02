# Analytics Technical Indicators

A command-line interface client for the real-time cryptocurrency trade analytics system. This client allows you to request various financial indicators (VWAP, SMA, MACD) and subscribe to live trade data streams.

## Architecture Overview

The system follows a modern data pipeline architecture with both real-time (hot path) and batch (cold path) processing:

<img width="5414" height="3025" alt="image" src="https://github.com/user-attachments/assets/4587be34-425d-4d00-bafc-d588847c6997" />


### Data Flow

1. **Data Ingestion**: Trade data flows from cryptocurrency exchanges (Binance, Coinbase) through WebSocket connections
2. **Real-time Processing**: NATS messaging system handles immediate data distribution
3. **Persistent Storage**: Kafka + ClickHouse for historical data storage and analytics
4. **Analytics Engine**: Rust-based gRPC server using DataFusion for calculations
5. **Client Interface**: This CLI client for user interaction

## Technology Stack

- **Language**: Rust (performance, safety, concurrency)
- **Messaging**: NATS (real-time), Kafka (persistence)
- **Database**: ClickHouse (columnar, analytical queries)
- **Analytics Engine**: DataFusion (Apache Arrow-based)
- **API**: gRPC with Tonic framework
- **Data Format**: Protocol Buffers
- **CLI Framework**: Clap

## Quick Start

### Installation

```bash
cargo build --package analytics-cli-client
```

### Usage Examples

```bash
# Get VWAP for BTCUSDT (last 5 minutes)
just vwap-now BTCUSDT

# Get SMA with custom window size
just sma BTCUSDT 1756648635 1756 20

# Get MACD analysis
just macd-now BTCUSDT

# Subscribe to live trades
just subscribe BTCUSDT
```

## Available Commands

### Core Analytics
- `vwap` - Volume Weighted Average Price
- `sma` - Simple Moving Average
- `macd` - Moving Average Convergence Divergence
- `subscribe` - Real-time trade subscription

## Related Components

- **analytics-server**: gRPC server providing analytics services
- **feed-handler**: Data ingestion from exchanges
- **clickhouse-sink**: Data persistence to ClickHouse
- **nats-to-kafka-bridge**: Message routing between systems

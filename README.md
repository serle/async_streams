# async_streams

**Async stock market technical analysis tool with real-time streaming of S&P 500 indicators**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](Cargo.toml)

---

## Overview

**async_streams** is a high-performance async Rust application that fetches real-time stock market data from Yahoo Finance and calculates technical indicators for S&P 500 companies. The tool streams analysis data continuously, making it ideal for monitoring market trends and building trading strategies.

### Key Features

- 📊 **Real-time Data Streaming** - Continuous updates every 30 seconds for S&P 500 stocks
- 📈 **Technical Indicators** - SMA (Simple Moving Average), Min/Max prices, price differences
- ⚡ **Async Performance** - Built with Tokio for concurrent API calls and efficient streaming
- 💾 **CSV Output** - Structured data export for analysis and visualization
- 🔄 **Automatic Retry** - Resilient error handling with exponential backoff
- 🎯 **Flexible Configuration** - Command-line interface for custom symbol lists and date ranges

---

## Installation

### Prerequisites

- Rust 2021 edition or later
- Internet connection for Yahoo Finance API access

### Build from Source

```bash
git clone https://github.com/serle/async_streams.git
cd async_streams
cargo build --release
```

---

## Usage

### Real-time Streaming Mode

Stream technical indicators for S&P 500 stocks (requires `sp500.txt` file with symbols):

```bash
cargo run --release
```

Output appears both on console and in `data.csv`:
```
period start,symbol,price,change %,min,max,30d avg
2025-01-22T10:30:00-05:00,AAPL,$150.25,2.15%,$145.00,$152.50,$148.33
```

### Single Analysis Mode

Analyze specific symbols over a date range:

```bash
# Analyze Apple stock for the last 2 weeks (default)
cargo run --release -- --symbols AAPL

# Multiple symbols
cargo run --release -- --symbols "AAPL,MSFT,GOOG,TSLA"

# Custom date range
cargo run --release -- --symbols "AAPL,MSFT" \
  --from "2025-01-01 00:00:00 UTC" \
  --to "2025-01-31 23:59:59 UTC"
```

---

## Technical Indicators

The tool calculates the following metrics for each stock:

| Indicator | Description |
|-----------|-------------|
| **Price** | Current/latest closing price |
| **Change %** | Percentage change from first to last price in period |
| **Period Min** | Lowest price in the analysis period |
| **Period Max** | Highest price in the analysis period |
| **30d SMA** | 30-day Simple Moving Average |

### Custom Indicators

The system uses a trait-based architecture for easy extensibility:

```rust
#[async_trait]
pub trait AsyncStockSignal {
    type SignalType;

    async fn calculate(&self, data: &[f64]) -> Option<Self::SignalType>;
}
```

Available implementations:
- `MinPrice` - Find minimum price in series
- `MaxPrice` - Find maximum price in series
- `PriceDifference` - Calculate absolute and relative price changes
- `WindowedSMA` - Calculate simple moving average over window

---

## Architecture

```
┌─────────────────┐
│ Yahoo Finance   │
│      API        │
└────────┬────────┘
         │
    ┌────▼────────────────┐
    │  Data Fetcher       │
    │  (async/await)      │
    └────┬────────────────┘
         │
    ┌────▼────────────────┐
    │ Signal Calculators  │
    │  - MinPrice         │
    │  - MaxPrice         │
    │  - PriceDifference  │
    │  - WindowedSMA      │
    └────┬────────────────┘
         │
    ┌────▼────────────────┐
    │  CSV Writer         │
    │  (data.csv)         │
    └─────────────────────┘
```

### Async Streaming Flow

1. Load S&P 500 symbol list from `sp500.txt`
2. Every 30 seconds, spawn async tasks for each symbol
3. Fetch closing prices from Yahoo Finance API
4. Calculate technical indicators concurrently
5. Stream results to console and CSV file
6. Automatic retry on transient failures (up to 5 attempts)

---

## Configuration

### S&P 500 Symbol List

Create a `sp500.txt` file with comma-separated stock symbols:

```
AAPL, MSFT, GOOGL, AMZN, META, TSLA, NVDA, JPM, V, WMT
```

### Command-line Options

```bash
Options:
  -s, --symbols <SYMBOLS>    Comma-separated stock symbols (default: AAPL,MSFT,UBER,GOOG)
  -f, --from <FROM>          Start date (default: 2 weeks ago)
  -t, --to <TO>              End date (default: now)
  -h, --help                 Print help
  -V, --version              Print version
```

---

## Testing

Run the comprehensive test suite:

```bash
# Run all tests
cargo test

# Test with Yahoo Finance API (requires internet)
cargo test -- --ignored

# Specific test
cargo test test_windowed_sma_calculate
```

Test coverage includes:
- ✅ Technical indicator calculations
- ✅ Yahoo Finance API integration
- ✅ Date range queries
- ✅ Error handling and edge cases
- ✅ Streaming signal generation

---

## Dependencies

- **tokio** - Async runtime for concurrent operations
- **tokio-stream** - Stream utilities for interval-based updates
- **yahoo_finance_api** - Yahoo Finance data provider
- **chrono** - Date/time handling
- **clap** - Command-line argument parsing
- **async-trait** - Async trait support
- **async-recursion** - Recursive async functions for retry logic

---

## Performance

- **Concurrent API Calls** - Fetches data for multiple symbols in parallel
- **Efficient Streaming** - Minimal memory footprint with async iterators
- **Rate Limiting** - 30-second intervals to respect API limits
- **Resilient Retry** - Exponential backoff for failed requests

---

## Output Format

### CSV Structure

```csv
period start,symbol,price,change %,min,max,30d avg
2025-01-22T10:30:00-05:00,AAPL,$150.25,2.15%,$145.00,$152.50,$148.33
2025-01-22T10:30:00-05:00,MSFT,$380.75,1.85%,$375.20,$385.00,$379.50
```

### Console Output

Real-time updates with colored formatting showing symbol, price movements, and technical indicators.

---

## Use Cases

- 📊 **Algorithmic Trading** - Feed data into trading algorithms
- 📈 **Market Monitoring** - Track S&P 500 performance in real-time
- 🔍 **Technical Analysis** - Identify trends using moving averages
- 📉 **Risk Management** - Monitor price volatility and ranges
- 📱 **Alert Systems** - Build notification systems for price movements

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Author

Serle Shuman - [serle.shuman@gmail.com](mailto:serle.shuman@gmail.com)

---

## Acknowledgments

- Built with Rust 🦀 for performance and reliability
- Powered by [Yahoo Finance API](https://github.com/xemwebe/yahoo_finance_api)
- Async runtime provided by [Tokio](https://tokio.rs)

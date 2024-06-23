mod signals;

use std::cmp::Ordering;
//--------------------------------------------------------------------------------------------------
use std::io::{BufWriter, Error, ErrorKind, Write};
use std::time::{Duration, UNIX_EPOCH};
use std::fs;
use clap::Parser;
use chrono::prelude::*;
use chrono::TimeDelta;
use yahoo_finance_api as yahoo;
use yahoo::time::macros::datetime;
use yahoo::YahooError;
use time::OffsetDateTime;
use signals::AsyncStockSignal;
use signals::{
    PriceDifference,
    WindowedSMA,
    MaxPrice,
    MinPrice
};
//--------------------------------------------------------------------------------------------------
#[derive(Parser, Debug)]
#[clap(
    version = "2.0",
    author = "Serle Shuman",
    about = "Async Rust project"
)]
struct Opts {
    #[clap(short, long)]
    symbols: Option<String>,
    #[clap(short, long)]
    from: Option<String>,
    #[clap(short, long)]
    to: Option<String>,
}
//--------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Params {
    symbols: Vec<String>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl Default for Params {
    fn default() -> Self {
        let opts = Opts::parse();
        let default_symbols: Vec<String> = vec!["AAPL".to_string(), "MSFT".to_string(), "UBER".to_string(), "GOOG".to_string()];
        let symbols: Vec<String> = match opts.symbols {
            Some(symbols) => symbols.split(",").map(|v| v.trim().to_string()).collect(),
            None => default_symbols.into_iter().map(|v| v.to_string()).collect(),
        };
        let default_start: DateTime<Utc> = Utc::now() - TimeDelta::weeks(2);
        let start: DateTime<Utc> = match opts.from {
            Some(from) => from.parse().unwrap_or(default_start),
            None => default_start
        };
        let default_end: DateTime<Utc> = Utc::now();
        let end = match opts.to {
            Some(to) => to.parse().unwrap_or(default_end),
            None => default_end,
        };

        match start.cmp(&end) {
            Ordering::Greater => {
                Self {
                    symbols,
                    start: end,
                    end: start
                }
            },
            _ => {
                Self {
                    symbols,
                    start,
                    end
                }
            },
        }
    }
}

///
/// Retrieve data from a data source and extract the closing prices. Errors during download are mapped onto io::Errors as InvalidData.
///
async fn fetch_closing_data(
    symbol: &str,
    start: &DateTime<Utc>,
    end: &DateTime<Utc>,
) -> std::io::Result<Vec<f64>> {
    let provider = yahoo::YahooConnector::new()
        .map_err(|_| Error::from(ErrorKind::ConnectionRefused))?;
    // incompatibility between chron and time crates
    let start = OffsetDateTime::from_unix_timestamp(start.timestamp()).unwrap();
    let end = OffsetDateTime::from_unix_timestamp(end.timestamp()).unwrap();
    let resp = provider.get_quote_history(symbol, start, end).await
        .map_err(|_| Error::from(ErrorKind::InvalidData))?;
    let mut quotes = resp.quotes()
        .map_err(|_| Error::from(ErrorKind::InvalidData))?;
    println!("{} quotes in January: {:?}", symbol, quotes);

    if !quotes.is_empty() {
        quotes.sort_by_cached_key(|k| k.timestamp);
        Ok(quotes.iter().map(|q| q.adjclose as f64).collect())
    } else {
        Ok(vec![])
    }
}

async fn calculate_signals(symbol: &str, start: &DateTime<Utc>, closes: &Vec<f64>) -> (String, String, f64, f64, f64, f64, f64) {
    let signal = MaxPrice {};
    let period_max = signal.calculate(closes).await.unwrap_or(0.0);
    let signal = MinPrice {};
    let period_min = signal.calculate(closes).await.unwrap_or(0.0);
    let signal = WindowedSMA::new(3);
    let sma = signal.calculate(closes).await.unwrap_or(vec![]);
    let signal = PriceDifference {};
    let price_diff = signal.calculate(closes).await.unwrap_or((0.0, 0.0));
    let pct_change = price_diff.1 * 100.0;
    let last_price = *closes.last().unwrap_or(&0.0);
    let last_sma = *sma.last().unwrap_or(&0.0);
    let date = start.to_rfc3339();

    let result = (date,
                  symbol.to_string(),
                  last_price,
                  pct_change,
                  period_min,
                  period_max,
                  last_sma);

    result
}

async fn stream_signals(symbols: &Vec<String>, start: &DateTime<Utc>, end: &DateTime<Utc>) -> std::io::Result<()> {
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open("data.csv")?;
    let mut stream = BufWriter::new(file);
    let header = "period start,symbol,price,change %,min,max,30d avg\n";
    println!("{}", &header);
    stream.write(header.as_bytes())?;
    for symbol in symbols.iter() {
        let closes = fetch_closing_data(&symbol, &start, &end).await?;
        if !closes.is_empty() {
            let data = calculate_signals(symbol, &start, &closes).await;
            let row = format!("{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}\n", data.0, data.1, data.2, data.3, data.4, data.5, data.6);
            println!("{}", &row);
            stream.write(row.as_bytes())?;
        }
    }
    stream.flush()?;
    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let params = Params::default();
    stream_signals(&params.symbols, &params.start, &params.end).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[ignore]
    #[test]
    fn it_loads_params() {
        let params = Params::default();
        //println!("{:?}", &params);
        assert_eq!(params.symbols.len(), 5usize);
    }

    #[tokio::test]
    async fn it_gets_closing_data() -> Result<(),Error>{
        let symbol = "AAPL";
        let start: DateTime<Utc> = DateTime::from_str("2020-1-1 0:00:00.00 UTC").unwrap();
        let end: DateTime<Utc> = DateTime::from_str("2020-1-31 23:59:59.99 UTC").unwrap();
        let data = fetch_closing_data(symbol, &start, &end).await?;
        println!("{:?}", &data);
        Ok(())
    }

    #[tokio::test]
    async fn it_streams_signals() -> Result<(),Error>{
        let symbols = vec!["AAPL".to_string()];
        let start: DateTime<Utc> = DateTime::from_str("2020-1-1 0:00:00.00 UTC").unwrap();
        let end: DateTime<Utc> = DateTime::from_str("2020-1-31 23:59:59.99 UTC").unwrap();
        let data = stream_signals(&symbols, &start, &end).await?;
        println!("{:?}", &data);
        Ok(())
    }


    #[tokio::test]
    async fn it_gets_latest_quote() -> Result<(),YahooError>{
        let provider = yahoo::YahooConnector::new().unwrap();
        // get the latest quotes in 1 minute intervals
        let response = provider.get_latest_quotes("AAPL", "1d").await?;
        // extract just the latest valid quote summery
        // including timestamp,open,close,high,low,volume
        let quote = response.last_quote().unwrap();
        let time: OffsetDateTime =
            OffsetDateTime::from(UNIX_EPOCH + Duration::from_secs(quote.timestamp));
        println!("At {} quote price of Apple was {}", time, quote.close);
        Ok(())
    }

    #[tokio::test]
    async fn it_gets_a_quote_range() -> Result<(),YahooError>{
        let provider = yahoo::YahooConnector::new().unwrap();
        let start = datetime!(2020-1-1 0:00:00.00 UTC);
        let end = datetime!(2020-1-31 23:59:59.99 UTC);
        // returns historic quotes with daily interval
        let resp = provider.get_quote_history("AAPL", start, end).await?;
        let quotes = resp.quotes().unwrap();
        println!("Apple's quotes in January: {:?}", quotes);

        Ok(())
    }

    #[tokio::test]
    async fn it_retrieves_daily_quotes_for_the_last_month() -> Result<(),YahooError>{
        let provider = yahoo::YahooConnector::new().unwrap();
        let response = provider.get_quote_range("AAPL", "1d", "1mo").await?;
        let quotes = response.quotes().unwrap();
        println!("Apple's quotes of the last month: {:?}", quotes);

        Ok(())
    }

    #[tokio::test]
    async fn it_finds_all_matching_tickers() -> Result<(),YahooError>{
        let provider = yahoo::YahooConnector::new().unwrap();
        let resp = provider.search_ticker("Apple").await?;

        println!("All tickers found while searching for 'Apple':");
        let items = resp.quotes.iter();
        items.for_each(|item| println!("{}", item.symbol));

        Ok(())
    }

    #[tokio::test]
    async fn test_price_difference_calculate() {
        use signals::PriceDifference;

        let signal = PriceDifference {};
        assert_eq!(signal.calculate(&[]).await, None);
        assert_eq!(signal.calculate(&[1.0]).await, Some((0.0, 0.0)));
        assert_eq!(signal.calculate(&[1.0, 0.0]).await, Some((-1.0, -1.0)));
        assert_eq!(
            signal.calculate(&[2.0, 3.0, 5.0, 6.0, 1.0, 2.0, 10.0]).await,
            Some((8.0, 4.0))
        );
        assert_eq!(
            signal.calculate(&[0.0, 3.0, 5.0, 6.0, 1.0, 2.0, 1.0]).await,
            Some((1.0, 1.0))
        );
    }

    #[tokio::test]
    async fn test_min_price_calculate() {
        use signals::MinPrice;

        let signal = MinPrice {};
        assert_eq!(signal.calculate(&[]).await, None);
        assert_eq!(signal.calculate(&[1.0]).await, Some(1.0));
        assert_eq!(signal.calculate(&[1.0, 0.0]).await, Some(0.0));
        assert_eq!(
            signal.calculate(&[2.0, 3.0, 5.0, 6.0, 1.0, 2.0, 10.0]).await,
            Some(1.0)
        );
        assert_eq!(
            signal.calculate(&[0.0, 3.0, 5.0, 6.0, 1.0, 2.0, 1.0]).await,
            Some(0.0)
        );
    }

    #[tokio::test]
    async fn test_max_price_calculate() {
        use signals::MaxPrice;

        let signal = MaxPrice {};
        assert_eq!(signal.calculate(&[]).await, None);
        assert_eq!(signal.calculate(&[1.0]).await, Some(1.0));
        assert_eq!(signal.calculate(&[1.0, 0.0]).await, Some(1.0));
        assert_eq!(
            signal.calculate(&[2.0, 3.0, 5.0, 6.0, 1.0, 2.0, 10.0]).await,
            Some(10.0)
        );
        assert_eq!(
            signal.calculate(&[0.0, 3.0, 5.0, 6.0, 1.0, 2.0, 1.0]).await,
            Some(6.0)
        );
    }

    #[tokio::test]
    async fn test_windowed_sma_calculate() {
        use signals::WindowedSMA;

        let series = vec![2.0, 4.5, 5.3, 6.5, 4.7];

        let signal = WindowedSMA::new(3);
        assert_eq!(
            signal.calculate(&series).await,
            Some(vec![3.9333333333333336, 5.433333333333334, 5.5])
        );

        let signal = WindowedSMA::new( 5 );
        assert_eq!(signal.calculate(&series).await, Some(vec![4.6]));

        let signal = WindowedSMA::new( 10 );
        assert_eq!(signal.calculate(&series).await, Some(vec![]));
    }
}

use chrono::DateTime;
use clap::Parser;
use clap::Subcommand;
use prost_types::Timestamp;

pub mod analytics {
    tonic::include_proto!("analytics");
}

pub mod data {
    tonic::include_proto!("data");
}
use crate::analytics::{
    GetMacdRequest, GetMovingAverageRequest, GetTradeAnalyticsRequest, SubscribeToTradesRequest,
};
use analytics::analytics_service_client::AnalyticsServiceClient;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Vwap {
        #[arg(short, long)]
        symbol: String,
        #[arg(long)]
        start_timestamp: u64,
        #[arg(long)]
        end_timestamp: u64,
    },
    Sma {
        #[arg(short, long)]
        symbol: String,
        #[arg(long)]
        start_timestamp: u64,
        #[arg(long)]
        end_timestamp: u64,
        #[arg(short, long, default_value_t = 20)]
        window_size: u32,
    },
    Subscribe {
        #[arg(short, long)]
        symbol: String,
    },
    Macd {
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        start_timestamp: u64,
        #[arg(long)]
        end_timestamp: u64,
        #[arg(short, long, default_value_t = 12)]
        fast_period: u32,
        #[arg(long, default_value_t = 26)]
        slow_period: u32,
        #[arg(long, default_value_t = 9)]
        signal_period: u32,
    },
}

fn format_timestamp_us(us: u64) -> String {
    let seconds = (us / 1_000_000) as i64;
    let nanoseconds = (us % 1_000_000 * 1000) as u32;
    let dt = DateTime::from_timestamp(seconds, nanoseconds).unwrap_or_default();
    dt.format("%Y-%m-%d %H:%M:%S.%3f").to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("Connecting to analytics server...");
    let mut client = AnalyticsServiceClient::connect("http://[::1]:50051").await?;
    println!("Connected!");
    match cli.command {
        Commands::Vwap {
            symbol,
            start_timestamp,
            end_timestamp,
        } => {
            let request = tonic::Request::new(GetTradeAnalyticsRequest {
                symbol: symbol.clone(),
                start_timestamp: Some(Timestamp {
                    seconds: start_timestamp as i64,
                    nanos: 0,
                }),
                end_timestamp: Some(Timestamp {
                    seconds: end_timestamp as i64,
                    nanos: 0,
                }),
            });

            println!("\nSending request for symbol '{}'...", symbol);
            let response = client.get_trade_analytics(request).await?;
            let data = response.into_inner();

            println!("\n✅ Analysis Complete!");
            println!("------------------------------------");
            println!("Symbol:           {}", symbol);
            println!("Trade Count:      {}", data.trades_count);
            println!(
                "Total Volume:     {:.2} (in quotes)",
                data.total_volume_in_quotes
            );
            println!("VWAP:             {:.2}", data.vwap);
            println!("------------------------------------");

            println!("------------------------------------");
            println!("SMA Request");
            println!("------------------------------------");
        }
        Commands::Sma {
            symbol,
            start_timestamp,
            end_timestamp,
            window_size,
        } => {
            let request = tonic::Request::new(GetMovingAverageRequest {
                symbol: symbol.clone(),
                start_timestamp: Some(Timestamp {
                    seconds: start_timestamp as i64,
                    nanos: 0,
                }),
                end_timestamp: Some(Timestamp {
                    seconds: end_timestamp as i64,
                    nanos: 0,
                }),
                window_size: window_size as u32,
            });
            let response = client.get_moving_average(request).await?;
            let data = response.into_inner();
            println!("\n✅ SMA Analysis Complete! (showing first 10 points)");
            println!("------------------------------------");
            println!("{:<28} | {:<15}", "Timestamp", "SMA Value");
            println!("------------------------------------");
            for point in data.points.iter().skip(500).take(10) {
                println!(
                    "{:<28} | {:<15.2}",
                    format_timestamp_us(point.timestamp),
                    point.value
                );
            }
            println!("------------------------------------");
            println!("Total points calculated: {}", data.points.len());
        }

        Commands::Subscribe { symbol } => {
            let request = tonic::Request::new(SubscribeToTradesRequest {
                symbol: symbol.clone(),
            });
            println!("Subscribing to trades for symbol '{}'...", symbol);
            let response = client.subscribe_to_trades(request).await?;
            let mut data = response.into_inner();
            while let Ok(Some(trade)) = data.message().await {
                println!("Received trade: {:?}", trade);
            }
            println!("Stream closed!");
        }
        Commands::Macd {
            symbol,
            start_timestamp,
            end_timestamp,
            fast_period,
            slow_period,
            signal_period,
        } => {
            let request = tonic::Request::new(GetMacdRequest {
                start_timestamp: Some(Timestamp {
                    seconds: start_timestamp as i64,
                    nanos: 0,
                }),
                end_timestamp: Some(Timestamp {
                    seconds: end_timestamp as i64,
                    nanos: 0,
                }),
                fast_period: fast_period as u32,
                slow_period: slow_period as u32,
                signal_period: signal_period as u32,
                symbol: symbol.clone(),
            });
            let response = client.get_macd(request).await?;
            let data = response.into_inner();
            println!("\n✅ MACD Analysis Complete! (showing first 10 points)");
            println!("------------------------------------");
            println!(
                "{:<28} | {:<15} | {:<15} | {:<15}",
                "Timestamp", "MACD Line", "Signal Line", "Histogram"
            );
            println!("------------------------------------");
            for point in data.points.iter().skip(500).take(10) {
                println!(
                    "{:<28} | {:<15.2} | {:<15.2} | {:<15.2}",
                    format_timestamp_us(point.timestamp),
                    point.macd_line,
                    point.signal_line,
                    point.histogram
                );
            }
            println!("------------------------------------");
            println!("Total points calculated: {}", data.points.len());
        }
    }
    Ok(())
}

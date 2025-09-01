use clap::Parser;
use clap::Subcommand;
use prost_types::Timestamp;

pub mod analytics {
    tonic::include_proto!("analytics");
}

use crate::analytics::{GetMovingAverageRequest, GetTradeAnalyticsRequest};
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
    /// Calcula a Média Móvel Simples (SMA) para um ativo
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
                println!("{:<28} | {:<15.2}", point.timestamp, point.value);
            }
            println!("------------------------------------");
            println!("Total points calculated: {}", data.points.len());
        }
    }
    Ok(())
}

use clap::Parser;
use prost_types::Timestamp;

pub mod analytics {
    tonic::include_proto!("analytics");
}

use crate::analytics::GetTradeAnalyticsRequest;
use analytics::analytics_service_client::AnalyticsServiceClient;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    symbol: String,

    #[arg(long)]
    start_timestamp: u64,

    #[arg(long)]
    end_timestamp: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("Connecting to analytics server...");
    let mut client = AnalyticsServiceClient::connect("http://[::1]:50051").await?;
    println!("Connected!");
    dbg!(&cli);
    let request = tonic::Request::new(GetTradeAnalyticsRequest {
        symbol: cli.symbol.clone(),
        start_timestamp: Some(Timestamp {
            seconds: cli.start_timestamp as i64,
            nanos: 0,
        }),
        end_timestamp: Some(Timestamp {
            seconds: cli.end_timestamp as i64,
            nanos: 0,
        }),
    });

    println!("\nSending request for symbol '{}'...", cli.symbol);
    let response = client.get_trade_analytics(request).await?;
    let data = response.into_inner();

    println!("\n✅ Analysis Complete!");
    println!("------------------------------------");
    println!("Symbol:           {}", cli.symbol);
    println!("Trade Count:      {}", data.trades_count);
    println!(
        "Total Volume:     {:.2} (in quotes)",
        data.total_volume_in_quotes
    );
    println!("VWAP:             {:.2}", data.vwap);
    println!("------------------------------------");

    Ok(())
}

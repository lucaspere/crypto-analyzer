use std::fmt::Display;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Trade {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "p")]
    price: String,
    #[serde(rename = "q")]
    quantity: String,
    #[serde(rename = "T")]
    timestamp: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nats_url = "nats://localhost:4222";
    let nats_client = async_nats::connect(nats_url).await?;
    let nats_subject = "trades.binance.btcusdt";

    let mut subscriptions = nats_client.subscribe(nats_subject.to_string()).await?;

    while let Some(msg) = subscriptions.next().await {
        if let Ok(trade) = serde_json::from_slice::<Trade>(&msg.payload) {
            println!("Received Trade: {:?}", trade);
        } else
        {
            eprintln!("Failed to deserialized message: {:?}", msg.payload);
        }
    }

    println!("Hello, world!");

    Ok(())
}

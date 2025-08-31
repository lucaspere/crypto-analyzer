use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

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

impl Display for Trade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Trade: {} {} @ {}",
            self.symbol, self.quantity, self.price
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nats_url = "nats://localhost:4222";
    let nats_client = async_nats::connect(nats_url).await?;
    let nats_subject = "trades.binance.btcusdt";

    let url = "wss://fstream.binance.com/ws/btcusdt@trade"
        .into_client_request()
        .expect("Failed to create request");
    let (ws_stream, _) = tokio_tungstenite::connect_async(url)
        .await
        .expect("Successfully connected!");

    println!("Connected!");

    let mut stream = ws_stream;

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(Message::Text(text)) => match serde_json::from_str::<Trade>(&text) {
                Ok(trade) => {
                    let payload = serde_json::to_vec(&trade)?;
                    nats_client
                        .publish(nats_subject.to_string(), payload.into())
                        .await?;

                    println!(
                        "Published trade for {}: Price: {}",
                        trade.symbol, trade.price
                    )
                }
                Err(e) => eprintln!("Error: {}", e),
            },
            Ok(msg) => {
                println!("Got message: {:?}", msg);
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}

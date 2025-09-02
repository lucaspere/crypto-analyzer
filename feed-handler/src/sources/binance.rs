use super::FeedSource;
use async_nats::Client as NatsClient;
use async_trait::async_trait;
use chrono::Utc;
use futures_util::stream::StreamExt;
use prost::Message;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

#[derive(Debug, Deserialize, Serialize)]
struct BinanceTrade {
    #[serde(rename(deserialize = "s"))]
    symbol: String,
    #[serde(rename(deserialize = "p"))]
    price: String,
    #[serde(rename(deserialize = "q"))]
    quantity: String,
    #[serde(rename(deserialize = "T"))]
    timestamp: u64,
}

impl From<BinanceTrade> for crate::data::Trade {
    fn from(value: BinanceTrade) -> Self {
        let now = Utc::now();
        let seconds = now.timestamp();
        let nanos = now.timestamp_subsec_nanos();
        Self {
            symbol: value.symbol,
            price: value.price.parse().unwrap_or_default(),
            quantity: value.quantity.parse().unwrap_or_default(),
            exchange_timestamp: value.timestamp,
            ingestion_timestamp: Some(prost_types::Timestamp {
                seconds,
                nanos: nanos as i32,
            }),
        }
    }
}

impl Display for BinanceTrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Trade: {} {} @ {}",
            self.symbol, self.quantity, self.price
        )
    }
}

pub struct BinanceSource;

#[async_trait]
impl FeedSource for BinanceSource {
    fn name(&self) -> &'static str {
        "Binance"
    }

    async fn connect_and_stream(
        &self,
        nats_client: NatsClient,
        symbol: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let symbol = symbol.to_lowercase();
        let nats_subject = format!("trades.{}.{symbol}", self.name().to_lowercase());
        let url = format!("wss://fstream.binance.com/ws/{symbol}@trade")
            .into_client_request()
            .expect("Failed to create request");
        let (ws_stream, _) = tokio_tungstenite::connect_async(url)
            .await
            .expect("Successfully connected!");

        println!("Connected!");

        let mut stream = ws_stream;

        while let Some(msg) = stream.next().await {
            match msg {
                Ok(tungstenite::Message::Text(text)) => {
                    match serde_json::from_str::<BinanceTrade>(&text) {
                        Ok(trade) => {
                            let payload: crate::data::Trade = trade.into();
                            let payload = payload.encode_to_vec();
                            nats_client
                                .publish(nats_subject.to_string(), payload.into())
                                .await?;

                            println!("Published trade for Symbol: {symbol}")
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                Ok(msg) => {
                    println!("Got message: {:?}", msg);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }

        Ok(())
    }
}

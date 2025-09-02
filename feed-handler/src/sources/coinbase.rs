use async_nats::Client as NatsClient;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use prost::Message;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::Message as WsMessage},
};

use crate::{data, sources::FeedSource};

#[derive(Serialize)]
struct CoinbaseSubscription {
    #[serde(rename = "type")]
    msg_type: &'static str,
    product_ids: Vec<String>,
    channels: Vec<&'static str>,
}

#[derive(Deserialize, Debug)]
struct CoinbaseMatch {
    #[serde(rename = "type")]
    msg_type: String,
    product_id: String,
    price: String,
    size: String,
    time: DateTime<Utc>,
}

impl From<CoinbaseMatch> for crate::data::Trade {
    fn from(value: CoinbaseMatch) -> Self {
        Self {
            symbol: value.product_id,
            price: value.price.parse::<f64>().unwrap_or_default(),
            quantity: value.size.parse::<f64>().unwrap_or_default(),
            exchange_timestamp: value.time.timestamp_micros() as u64,
            exchange: data::trade::Exchange::Coinbase.into(),
            ingestion_timestamp: Some(prost_types::Timestamp {
                seconds: value.time.timestamp(),
                nanos: value.time.timestamp_subsec_nanos() as i32,
            }),
        }
    }
}
pub struct CoinbaseSource;

#[async_trait]
impl FeedSource for CoinbaseSource {
    fn name(&self) -> &'static str {
        "Coinbase"
    }

    async fn connect_and_stream(
        &self,
        nats_client: NatsClient,
        symbol: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let nats_subject = format!(
            "trades.{}.{}",
            self.name().to_lowercase(),
            symbol.to_lowercase().replace("-", "")
        );
        let ws_url = "wss://ws-feed.exchange.coinbase.com".into_client_request()?;

        let (ws_stream, _) = connect_async(ws_url).await?;

        let (mut write, mut read) = ws_stream.split();

        let subscription_msg = CoinbaseSubscription {
            msg_type: "subscribe",
            product_ids: vec![symbol.to_string()],
            channels: vec!["matches"],
        };
        let json_msg = serde_json::to_string(&subscription_msg)?;
        write.send(WsMessage::Text(json_msg.into())).await?;
        println!("[{}] Subscribed to trades for {}", self.name(), symbol);

        while let Some(msg) = read.next().await {
            if let Ok(WsMessage::Text(text)) = msg {
                if let Ok(coinbase_match) = serde_json::from_str::<CoinbaseMatch>(&text) {
                    if coinbase_match.msg_type != "match" {
                        continue;
                    }

                    let payload: data::Trade = coinbase_match.into();
                    let payload = payload.encode_to_vec();

                    nats_client
                        .publish(nats_subject.clone(), payload.into())
                        .await?;
                }
            }
        }
        Ok(())
    }
}

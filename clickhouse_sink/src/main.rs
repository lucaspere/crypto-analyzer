use clickhouse::Row;
use clickhouse::insert::Insert;
use prost::Message as ProstMessage;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{
    BaseConsumer, CommitMode, Consumer, ConsumerContext, Rebalance, StreamConsumer,
};
use rdkafka::{ClientConfig, ClientContext, Message};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::Instant;

mod data {
    include!(concat!(env!("OUT_DIR"), "/data.rs"));
}

#[derive(Debug, Clone, Deserialize, Serialize)]
enum Exchange {
    Unknown = 0,
    Binance = 1,
    Coinbase = 2,
}

impl From<data::trade::Exchange> for Exchange {
    fn from(value: data::trade::Exchange) -> Self {
        match value {
            data::trade::Exchange::Unknown => Self::Unknown,
            data::trade::Exchange::Binance => Self::Binance,
            data::trade::Exchange::Coinbase => Self::Coinbase,
        }
    }
}

impl From<i32> for Exchange {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Unknown,
            1 => Self::Binance,
            2 => Self::Coinbase,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Row)]
struct Trade {
    symbol: String,
    price: f64,
    quantity: f64,
    exchange_timestamp: u64,
    /// nanoseconds
    ingestion_timestamp: Option<u64>,
    exchange: Exchange,
}

impl From<data::Trade> for Trade {
    fn from(value: data::Trade) -> Self {
        Self {
            symbol: value.symbol,
            price: value.price as f64,
            quantity: value.quantity as f64,
            exchange_timestamp: value.exchange_timestamp,
            ingestion_timestamp: value.ingestion_timestamp.map(|t| {
                let seconds_as_nanos = t.seconds as u64 * 1_000_000_000;

                let nanos = t.nanos as u64;

                seconds_as_nanos + nanos
            }),
            exchange: value.exchange.into(),
        }
    }
}

impl From<Trade> for data::Trade {
    fn from(value: Trade) -> Self {
        Self {
            symbol: value.symbol,
            price: value.price,
            quantity: value.quantity,
            exchange_timestamp: value.exchange_timestamp,
            exchange: value.exchange as i32,
            ingestion_timestamp: value.ingestion_timestamp.map(|t| prost_types::Timestamp {
                seconds: t as i64 / 1_000_000_000,
                nanos: (t % 1_000_000_000) as i32,
            }),
        }
    }
}

struct CustomContext;
impl ClientContext for CustomContext {}
impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, _base_consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        println!("Pre rebalance {rebalance:?}");
    }
    fn post_rebalance(&self, _base_consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        println!("Post rebalance {rebalance:?}");
    }
}

type LoggingConsumer = StreamConsumer<CustomContext>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = clickhouse::Client::default()
        .with_url("http://localhost:8123")
        .with_database("default");

    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", "clickhouse_sink_group")
        .set("bootstrap.servers", "localhost:9092")
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(CustomContext)?;

    consumer.subscribe(&["trades"])?;
    println!("Subscribed to topic");

    let mut buffer = Vec::with_capacity(100);
    let mut last_flush = Instant::now();
    let flush_interval = Duration::from_secs(5);

    loop {
        match consumer.recv().await {
            Ok(msg) => {
                if let Some(payload) = msg.payload() {
                    if let Ok(trade) = data::Trade::decode(payload) {
                        buffer.push(trade);
                    }
                }
            }
            Err(e) => {
                eprintln!("Consumer error: {:?}", e);
            }
        }

        if !buffer.is_empty() && (buffer.len() >= 100 || last_flush.elapsed() >= flush_interval) {
            println!("Flushing {} records to Clickhouse...", buffer.len());
            let mut inserter: Insert<Trade> = client.insert("trades")?;
            for trade in buffer.drain(..) {
                inserter.write(&trade.into()).await?;
            }

            inserter.end().await?;
            println!("Flush complete");
            last_flush = Instant::now();

            consumer.commit_message(&consumer.recv().await?, CommitMode::Async)?;
        }
    }
}

use std::time::Duration;
use serde::{Deserialize, Serialize};
use clickhouse::Row;
use rdkafka::{ClientConfig, ClientContext, Message};
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{BaseConsumer, CommitMode, Consumer, ConsumerContext, Rebalance, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::message::BorrowedMessage;
use tokio::time::Instant;

#[derive(Debug, Clone, Deserialize, Serialize, Row)]
struct Trade {
    symbol: String,
    price: String,
    quantity: String,
    timestamp: u64
}

struct CustomContext;
impl ClientContext for CustomContext {}
impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, base_consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        println!("Pre rebalance {rebalance:?}");
    }
    fn post_rebalance(&self, base_consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
       println!("Post rebalance {rebalance:?}");
    }
}

type LoggingConsumer = StreamConsumer<CustomContext>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let client = clickhouse::Client::default().with_url("http://localhost:8123").with_database("default");

    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", "clickhouse_sink_group")
        .set("bootstrap.servers", "localhost:9092")
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(CustomContext)?;

    consumer.subscribe(&[ "trades" ])?;
    println!("Subscribed to topic");

    let mut buffer: Vec<Trade> = Vec::with_capacity(100);
    let mut last_flush = Instant::now();
    let flush_interval = Duration::from_secs(5);

    loop {
        match consumer.recv().await {
            Ok(msg) => {
                if let Some(payload) = msg.payload() {
                    if let Ok(trade) = serde_json::from_slice::<Trade>(payload) {
                        buffer.push(trade);
                    }
                }
            },
            Err(e) => {
                eprintln!("Consumer error: {:?}", e);
            }
        }

        if !buffer.is_empty() && (buffer.len() >= 100 || last_flush.elapsed() >= flush_interval) {
            println!("Flushing {} records to Clickhouse...", buffer.len());
            let mut inserter = client.insert("trades")?;
            for trade in buffer.drain(..) {
                inserter.write(&trade).await?;
            }

            inserter.end().await?;
            println!("Flush complete");
            last_flush = Instant::now();


            consumer.commit_message(&consumer.recv().await?, CommitMode::Async)?;
        }
    }

}

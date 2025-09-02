use futures_util::StreamExt;
use rdkafka::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kafka_brokers = "localhost:9092";
    let kafka_topic = "trades";

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", kafka_brokers)
        .set("message.timeout.ms", "5000")
        .create()?;

    println!("Connected to Kafka brokers at {}", kafka_brokers);

    let nats_url = "nats://localhost:4222";
    let nats_subject = "trades.*.*";
    let nats_client = async_nats::connect(nats_url).await?;

    let mut subscription = nats_client.subscribe(nats_subject.to_string()).await?;

    while let Some(msg) = subscription.next().await {
        let record = FutureRecord::to(kafka_topic)
            .payload(&msg.payload[..])
            .key("trade");

        match producer.send(record, Duration::from_secs(0)).await {
            Ok(_) => println!("Message forwarded from NATS to Kafka topic: '{kafka_topic}'"),
            Err((e, _)) => eprintln!("Error sending message: {e}"),
        }
    }
    println!("Producer created:");

    Ok(())
}

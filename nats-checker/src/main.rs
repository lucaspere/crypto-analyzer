use futures_util::StreamExt;
use prost::Message;

mod data {
    include!(concat!(env!("OUT_DIR"), "/data.rs"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nats_url = "nats://localhost:4222";
    let nats_client = async_nats::connect(nats_url).await?;
    let nats_subject = "trades.*";

    let mut subscriptions = nats_client.subscribe(nats_subject.to_string()).await?;

    while let Some(msg) = subscriptions.next().await {
        let bytes = msg.payload.to_vec();
        if let Ok(trade) = data::Trade::decode(&bytes[..]) {
            println!("Received Trade: {:?}", trade);
        } else {
            eprintln!("Failed to deserialized message: {:?}", msg.payload);
        }
    }

    println!("Hello, world!");

    Ok(())
}

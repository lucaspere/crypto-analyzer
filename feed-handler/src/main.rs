mod sources;
use sources::{binance::BinanceSource, FeedSource};

mod data {
    include!(concat!(env!("OUT_DIR"), "/data.rs"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nats_url = "nats://localhost:4222";
    let nats_client = async_nats::connect(nats_url).await?;
    let nats_subject = "trades.binance.btcusdt";

    let binance_handler = BinanceSource;

    binance_handler
        .connect_and_stream(nats_client, nats_subject)
        .await?;

    Ok(())
}

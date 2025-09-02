mod sources;

use sources::{binance::BinanceSource, coinbase::CoinbaseSource, FeedSource};
use tokio::task::JoinHandle;

mod data {
    include!(concat!(env!("OUT_DIR"), "/data.rs"));
}

type FeedHandler = Box<dyn FeedSource + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nats_client = async_nats::connect("nats://localhost:4222").await?;
    println!("[Main] Connected to NATS");

    let handlers: Vec<(FeedHandler, &str)> = vec![
        (Box::new(BinanceSource), "BTCUSDT"),
        (Box::new(CoinbaseSource), "BTC-USD"),
    ];

    let mut tasks: Vec<JoinHandle<()>> = Vec::new();

    for (handler, symbol) in handlers {
        let nats_clone = nats_client.clone();
        let symbol_str = symbol.to_string();

        let task = tokio::spawn(async move {
            println!(
                "[Main] Launching {} feed handler for symbol {}...",
                handler.name(),
                symbol_str
            );
            if let Err(e) = handler.connect_and_stream(nats_clone, &symbol_str).await {
                eprintln!("[{}] Handler failed: {}", handler.name(), e);
            }
        });
        tasks.push(task);
    }

    futures::future::join_all(tasks).await;

    Ok(())
}

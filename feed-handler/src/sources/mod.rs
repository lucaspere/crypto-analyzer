pub mod binance;

use async_nats::Client as NatsClient;
use async_trait::async_trait;

#[async_trait]
pub trait FeedSource {
    fn name(&self) -> &'static str;

    async fn connect_and_stream(
        &self,
        nats_client: NatsClient,
        symbol: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
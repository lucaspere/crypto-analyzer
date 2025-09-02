use std::sync::Arc;

use async_nats::Client as AsyncNatsClient;
use clickhouse::Client;
use clickhouse::query::RowCursor;
use datafusion::arrow::array::{
    Array, Float64Array, Int64Array, RecordBatch, TimestampNanosecondArray,
};
use datafusion::prelude::SessionContext;
use futures::StreamExt;
use futures::stream::BoxStream;
use prost::Message;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

mod analytics {
    tonic::include_proto!("analytics");
}
mod data {
    tonic::include_proto!("data");
}

use analytics::analytics_service_server::AnalyticsService;
use analytics::{
    GetMacdRequest, GetMacdResponse, GetMovingAverageRequest, GetMovingAverageResponse,
    GetTradeAnalyticsRequest, GetTradeAnalyticsResponse, SubscribeToTradesRequest,
};

use crate::analytics::analytics_service_server::AnalyticsServiceServer;
use crate::analytics::{MacdDataPoint, MovingAverageDataPoint};

pub struct AnalyticsServiceHandler {
    clickhouse_client: Client,
    nats_client: AsyncNatsClient,
}

impl AnalyticsServiceHandler {
    async fn fetch_trades(
        &self,
        req: GetTradeAnalyticsRequest,
    ) -> Result<(Vec<u64>, Vec<f64>, Vec<f64>), Status> {
        let query = "SELECT exchange_timestamp, price, quantity
             FROM default.trades
             WHERE symbol = ? AND exchange_timestamp >= ? AND exchange_timestamp <= ?"
            .to_string();
        let start_timestamp_millis = req.start_timestamp.as_ref().map(|t| t.seconds * 1000);
        let end_timestamp_millis = req.end_timestamp.as_ref().map(|t| t.seconds * 1000);
        #[derive(Debug, serde::Deserialize, clickhouse::Row)]
        struct Row {
            exchange_timestamp: u64,
            price: f64,
            quantity: f64,
        }
        let mut cursor: RowCursor<Row> = self
            .clickhouse_client
            .query(&query)
            .bind(req.symbol)
            .bind(start_timestamp_millis)
            .bind(end_timestamp_millis)
            .fetch()
            .map_err(|e| Status::internal(format!("Error fetching data: {}", e)))?;
        let mut timestamps = Vec::new();
        let mut prices = Vec::new();
        let mut quantities = Vec::new();
        while let Some(row) = cursor
            .next()
            .await
            .map_err(|e| Status::internal(format!("Error fetching row: {}", e)))?
        {
            let timestamp_nanos: u64 = row.exchange_timestamp * 1_000_000;
            timestamps.push(timestamp_nanos);
            prices.push(row.price);
            quantities.push(row.quantity);
        }
        if timestamps.is_empty() {
            return Err(Status::not_found(
                "No data found for the given symbol and timestamp range",
            ));
        }
        Ok((timestamps, prices, quantities))
    }

    fn build_record_batch(
        timestamps: Vec<u64>,
        prices: Vec<f64>,
        quantities: Vec<f64>,
    ) -> Result<RecordBatch, Status> {
        RecordBatch::try_from_iter(vec![
            (
                "timestamp",
                Arc::new(TimestampNanosecondArray::from_iter_values(
                    timestamps.iter().map(|t| *t as i64),
                )) as _,
            ),
            ("price", Arc::new(Float64Array::from(prices)) as _),
            ("quantity", Arc::new(Float64Array::from(quantities)) as _),
        ])
        .map_err(|_e| Status::internal("Error creating batch"))
    }

    async fn compute_analytics(batch: RecordBatch) -> Result<(f64, f64, u64), Status> {
        let ctx = SessionContext::new();
        ctx.register_batch("trades_mem", batch)
            .map_err(|e| Status::internal(format!("Error registering batch: {}", e)))?;
        let df = ctx
            .sql(
                "SELECT
                SUM(price * quantity) AS total_volume_in_quotes,
                SUM(price * quantity) / SUM(quantity) AS vwap,
                COUNT(*) AS trades_count
             FROM trades_mem",
            )
            .await
            .map_err(|e| Status::internal(format!("Error executing query: {}", e)))?;
        let results = df
            .collect()
            .await
            .map_err(|e| Status::internal(format!("Error collecting results: {}", e)))?;
        let result_batch = results
            .first()
            .ok_or_else(|| Status::internal("Analysis returned no rows"))?;
        let total_volume_in_quotes = result_batch
            .column_by_name("total_volume_in_quotes")
            .unwrap()
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap()
            .value(0);
        let vwap = result_batch
            .column_by_name("vwap")
            .expect("Not found vmap column in result")
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("Failed to cast vwap column to Float64Array")
            .value(0);
        let trades_count = result_batch
            .column_by_name("trades_count")
            .expect("Not found trades_count column in result")
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0) as u64;
        Ok((total_volume_in_quotes, vwap, trades_count))
    }

    async fn fetch_macd_data(
        &self,
        request: GetMacdRequest,
    ) -> Result<(Vec<u64>, Vec<f64>), Status> {
        let start_timestamp_millis = request.start_timestamp.as_ref().map(|t| t.seconds * 1000);
        let end_timestamp_millis = request.end_timestamp.as_ref().map(|t| t.seconds * 1000);
        let query = format!(
            "SELECT exchange_timestamp, price
             FROM default.trades
             WHERE symbol = ? AND exchange_timestamp >= ? AND exchange_timestamp <= ?
             ORDER BY exchange_timestamp",
        );
        #[derive(Debug, serde::Deserialize, clickhouse::Row)]
        struct Row {
            exchange_timestamp: u64,
            price: f64,
        }
        let mut cursor: RowCursor<Row> = self
            .clickhouse_client
            .query(&query)
            .bind(request.symbol)
            .bind(start_timestamp_millis)
            .bind(end_timestamp_millis)
            .fetch()
            .map_err(|e| Status::internal(format!("Error fetching data: {}", e)))?;
        let mut timestamps = Vec::<u64>::new();
        let mut prices = Vec::<f64>::new();
        while let Some(row) = cursor
            .next()
            .await
            .map_err(|e| Status::internal(format!("Error fetching row: {}", e)))?
        {
            let timestamp_nano = row.exchange_timestamp * 1_000_000;
            timestamps.push(timestamp_nano);
            prices.push(row.price);
        }
        Ok((timestamps, prices))
    }

    fn calculate_ema(&self, prices: &[f64], window_size: usize) -> Vec<f64> {
        let mut ema = Vec::new();
        let multiplier = 2.0 / (window_size as f64 + 1.0);
        let first_sma = prices.iter().take(window_size).sum::<f64>() / window_size as f64;
        ema.push(first_sma);
        for price in prices.iter().skip(window_size) {
            let last_ema = ema.last().unwrap();
            let new_ema = (price * multiplier) + (last_ema * (1.0 - multiplier));
            ema.push(new_ema);
        }
        ema
    }
}

#[tonic::async_trait]
impl AnalyticsService for AnalyticsServiceHandler {
    async fn get_trade_analytics(
        &self,
        request: Request<GetTradeAnalyticsRequest>,
    ) -> Result<Response<GetTradeAnalyticsResponse>, Status> {
        let req = request.into_inner();
        println!("Received request for symbol {}", req.symbol);
        let (timestamps, prices, quantities) = self.fetch_trades(req).await?;
        let batch = AnalyticsServiceHandler::build_record_batch(timestamps, prices, quantities)?;
        let (total_volume_in_quotes, vwap, trades_count) =
            AnalyticsServiceHandler::compute_analytics(batch).await?;
        Ok(Response::new(GetTradeAnalyticsResponse {
            total_volume_in_quotes,
            vwap,
            trades_count,
        }))
    }

    async fn get_moving_average(
        &self,
        request: Request<GetMovingAverageRequest>,
    ) -> Result<Response<GetMovingAverageResponse>, Status> {
        let request = request.into_inner();
        println!(
            "Received SMA request for symbol {} with window size {}",
            request.symbol, request.window_size
        );

        let start_timestamp_millis = request.start_timestamp.as_ref().map(|t| {
            let seconds = t.seconds;
            let nanos = t.nanos;
            let millis = seconds * 1000 + nanos as i64 / 1_000_000;
            millis
        });
        let end_timestamp_millis = request.end_timestamp.map(|t| {
            let seconds = t.seconds;
            let nanos = t.nanos;
            let millis = seconds * 1000 + nanos as i64 / 1_000_000;
            millis
        });
        let query = format!(
            "SELECT exchange_timestamp, price
             FROM default.trades
             WHERE symbol = ? AND exchange_timestamp >= ? AND exchange_timestamp <= ?
             ORDER BY exchange_timestamp",
        );

        #[derive(Debug, serde::Deserialize, clickhouse::Row)]
        struct Row {
            exchange_timestamp: u64,
            price: f64,
        }
        let mut cursor: RowCursor<Row> = self
            .clickhouse_client
            .query(&query)
            .bind(request.symbol)
            .bind(start_timestamp_millis)
            .bind(end_timestamp_millis)
            .fetch()
            .map_err(|e| Status::internal(format!("Error fetching data: {}", e)))?;
        let mut timestamps = Vec::<u64>::new();
        let mut prices = Vec::<f64>::new();
        while let Some(row) = cursor
            .next()
            .await
            .map_err(|e| Status::internal(format!("Error fetching row: {}", e)))?
        {
            let timestamp_nano = row.exchange_timestamp * 1_000_000;
            timestamps.push(timestamp_nano);
            prices.push(row.price);
        }

        if timestamps.is_empty() {
            return Err(Status::not_found(
                "No data found for the given symbol and timestamp range",
            ));
        }

        let batch = RecordBatch::try_from_iter(vec![
            (
                "timestamp",
                Arc::new(TimestampNanosecondArray::from_iter_values(
                    timestamps.iter().map(|t| *t as i64),
                )) as _,
            ),
            ("price", Arc::new(Float64Array::from(prices)) as _),
        ])
        .map_err(|_e| Status::internal("Error creating batch"))?;

        let ctx = SessionContext::new();
        ctx.register_batch("trades_mem", batch)
            .map_err(|e| Status::internal(format!("Error registering batch: {}", e)))?;

        let sql_query = format!(
            "SELECT
             timestamp,
             AVG(price) OVER (
                ORDER BY timestamp
                ROWS BETWEEN {} PRECEDING AND CURRENT ROW
             ) AS sma
             FROM trades_mem",
            request.window_size - 1
        );

        let df = ctx
            .sql(&sql_query)
            .await
            .map_err(|e| Status::internal(format!("Error executing query: {}", e)))?;

        let results = df
            .collect()
            .await
            .map_err(|e| Status::internal(format!("Error collecting results: {}", e)))?;

        let mut points = Vec::new();
        for result in results {
            let timestamp = result
                .column_by_name("timestamp")
                .unwrap()
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
                .unwrap();
            let smas = result
                .column_by_name("sma")
                .unwrap()
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap();

            for i in 0..result.num_rows() {
                if smas.is_valid(i) {
                    points.push(MovingAverageDataPoint {
                        timestamp: timestamp.value(i) as u64,
                        value: smas.value(i),
                    });
                }
            }
        }

        Ok(Response::new(GetMovingAverageResponse { points }))
    }

    async fn get_ema(
        &self,
        request: Request<GetMovingAverageRequest>,
    ) -> Result<Response<GetMovingAverageResponse>, Status> {
        let request = request.into_inner();
        println!(
            "Received EMA request for symbol {} with window size {}",
            request.symbol, request.window_size
        );

        let window_size = request.window_size as usize;
        let start_timestamp_millis = request.start_timestamp.as_ref().map(|t| {
            let seconds = t.seconds;
            let nanos = t.nanos;
            let millis = seconds * 1000 + nanos as i64 / 1_000_000;
            millis
        });
        let end_timestamp_millis = request.end_timestamp.map(|t| {
            let seconds = t.seconds;
            let nanos = t.nanos;
            let millis = seconds * 1000 + nanos as i64 / 1_000_000;
            millis
        });
        let query = format!(
            "SELECT exchange_timestamp, price
             FROM default.trades
             WHERE symbol = ? AND exchange_timestamp >= ? AND exchange_timestamp <= ?
             ORDER BY exchange_timestamp",
        );
        #[derive(Debug, serde::Deserialize, clickhouse::Row)]
        struct Row {
            exchange_timestamp: u64,
            price: f64,
        }
        let mut cursor: RowCursor<Row> = self
            .clickhouse_client
            .query(&query)
            .bind(request.symbol)
            .bind(start_timestamp_millis)
            .bind(end_timestamp_millis)
            .fetch()
            .map_err(|e| Status::internal(format!("Error fetching data: {}", e)))?;
        let mut timestamps = Vec::<u64>::new();
        let mut prices = Vec::<f64>::new();
        while let Some(row) = cursor
            .next()
            .await
            .map_err(|e| Status::internal(format!("Error fetching row: {}", e)))?
        {
            let timestamp_nano = row.exchange_timestamp * 1_000_000;
            timestamps.push(timestamp_nano);
            prices.push(row.price);
        }

        if prices.len() < window_size {
            return Err(Status::invalid_argument("Not enough data to compute EMA"));
        }

        let mut ema = Vec::new();
        let multiplier = 2.0 / (window_size as f64 + 1.0);

        let first_sma = prices.iter().take(window_size).sum::<f64>() / window_size as f64;

        ema.push(first_sma);

        for price in prices.iter().skip(window_size) {
            let last_ema = ema.last().unwrap();
            let new_ema = (price * multiplier) + (last_ema * (1.0 - multiplier));
            ema.push(new_ema);
        }

        let points = timestamps
            .iter()
            .zip(ema.iter())
            .map(|(timestamp, value)| MovingAverageDataPoint {
                timestamp: *timestamp * 1000 as u64,
                value: *value,
            })
            .collect();

        Ok(Response::new(GetMovingAverageResponse { points }))
    }

    type SubscribeToTradesStream = BoxStream<'static, Result<data::Trade, Status>>;
    async fn subscribe_to_trades(
        &self,
        request: Request<SubscribeToTradesRequest>,
    ) -> Result<Response<BoxStream<'static, Result<data::Trade, Status>>>, Status> {
        let request = request.into_inner();

        let subject = format!("trades.*.{}", request.symbol.to_lowercase());

        let subscription = self
            .nats_client
            .subscribe(subject)
            .await
            .map_err(|e| Status::internal(format!("Error subscribing to subject: {}", e)))?;
        let trade_stream = subscription.map(|msg| {
            data::Trade::decode(msg.payload)
                .map_err(|e| Status::internal(format!("Failed to decode trade data: {}", e)))
        });

        Ok(Response::new(Box::pin(trade_stream)))
    }

    async fn get_macd(
        &self,
        request: Request<GetMacdRequest>,
    ) -> Result<Response<GetMacdResponse>, Status> {
        let request = request.into_inner();
        let signal_period = request.signal_period as usize;
        let slow_period = request.slow_period as usize;
        let fast_period = request.fast_period as usize;
        println!(
            "Received MACD request for symbol {} with fast period {}, slow period {}, and signal period {}",
            request.symbol, request.fast_period, request.slow_period, request.signal_period
        );

        let (timestamps, prices) = self.fetch_macd_data(request).await?;
        if prices.len() < slow_period as usize {
            return Err(Status::invalid_argument("Not enough data to compute MACD"));
        }

        let ema_fast = self.calculate_ema(&prices, fast_period);
        let ema_slow = self.calculate_ema(&prices, slow_period);
        let aligned_ema_fast = &ema_fast[fast_period - 1..];

        let macd_line: Vec<f64> = aligned_ema_fast
            .iter()
            .zip(ema_slow.iter())
            .map(|(fast, slow)| fast - slow)
            .collect();

        let signal_line = self.calculate_ema(&macd_line, signal_period);
        let aligned_macd_line = &macd_line[signal_period - 1..];
        let histogram: Vec<f64> = aligned_macd_line
            .iter()
            .zip(signal_line.iter())
            .map(|(signal, macd)| signal - macd)
            .collect();

        let points = timestamps
            .into_iter()
            .skip((slow_period + signal_period - 2) as usize)
            .zip(aligned_macd_line.iter())
            .zip(signal_line.iter())
            .zip(histogram.iter())
            .map(|(((timestamp, macd), signal), histogram)| MacdDataPoint {
                timestamp: timestamp * 1000 as u64,
                macd_line: *macd,
                signal_line: *signal,
                histogram: *histogram,
            })
            .collect();
        Ok(Response::new(GetMacdResponse { points }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let nats_url = "nats://localhost:4222";
    let nats_client = async_nats::connect(nats_url).await?;

    let client = Client::default()
        .with_url("http://localhost:8123")
        .with_database("default");

    let analytics_service = AnalyticsServiceHandler {
        clickhouse_client: client,
        nats_client: nats_client,
    };

    let svc = AnalyticsServiceServer::new(analytics_service);

    println!("AnalyticsServer listening on {}", addr);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}

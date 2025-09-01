use std::sync::Arc;

use clickhouse::Client;
use clickhouse::query::RowCursor;
use datafusion::arrow::array::{
    Array, Float64Array, Int64Array, RecordBatch, Time64MicrosecondArray, TimestampNanosecondArray,
};
use datafusion::prelude::SessionContext;
use tonic::{Request, Response, Status, transport::Server};

mod analytics {
    tonic::include_proto!("analytics");
}

use analytics::analytics_service_server::{AnalyticsService, AnalyticsServiceServer};
use analytics::{
    GetMovingAverageRequest, GetMovingAverageResponse, GetTradeAnalyticsRequest,
    GetTradeAnalyticsResponse,
};

use crate::analytics::MovingAverageDataPoint;

pub struct AnalyticsServiceHandler {
    clickhouse_client: Client,
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

        dbg!(&timestamps);
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

        dbg!(&results);
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;

    // Criamos o nosso cliente ClickHouse que será partilhado entre as requisições
    let client = Client::default()
        .with_url("http://localhost:8123")
        .with_database("default");

    let analytics_service = AnalyticsServiceHandler {
        clickhouse_client: client,
    };

    let svc = AnalyticsServiceServer::new(analytics_service);

    println!("AnalyticsServer listening on {}", addr);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}

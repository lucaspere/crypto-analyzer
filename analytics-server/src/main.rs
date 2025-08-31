use std::sync::Arc;

use clickhouse::Client;
use clickhouse::query::RowCursor;
use datafusion::arrow::array::{
    Array, Datum, Float64Array, Int64Array, RecordBatch, TimestampNanosecondArray, UInt64Array,
};
use datafusion::prelude::SessionContext;
use tonic::{Request, Response, Status, transport::Server};

mod analytics {
    tonic::include_proto!("analytics");
}

use analytics::analytics_service_server::{AnalyticsService, AnalyticsServiceServer};
use analytics::{GetTradeAnalyticsRequest, GetTradeAnalyticsResponse};

pub struct AnalyticsServiceHandler {
    clickhouse_client: Client,
}

#[tonic::async_trait]
impl AnalyticsService for AnalyticsServiceHandler {
    async fn get_trade_analytics(
        &self,
        request: Request<GetTradeAnalyticsRequest>,
    ) -> Result<Response<GetTradeAnalyticsResponse>, Status> {
        let req = request.into_inner();

        println!("Received request for symbol {}", req.symbol);

        let query = "SELECT timestamp, CAST(price, 'Float64'), CAST(quantity, 'Float64')
             FROM default.trades
             WHERE symbol = ? AND timestamp >= ? AND timestamp <= ?"
            .to_string();

        let start_timestamp_millis = req.start_timestamp.as_ref().map(|t| t.seconds * 1000);
        let end_timestamp_millis = req.end_timestamp.as_ref().map(|t| t.seconds * 1000);

        #[derive(Debug, serde::Deserialize, clickhouse::Row)]
        struct Row {
            timestamp: u64,
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
            let timestamp_nanos: u64 = row.timestamp * 1_000_000;
            timestamps.push(timestamp_nanos);
            prices.push(row.price);
            quantities.push(row.quantity);
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
            ("quantity", Arc::new(Float64Array::from(quantities)) as _),
        ])
        .map_err(|e| Status::internal("Error creating batch"))?;

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

        Ok(Response::new(GetTradeAnalyticsResponse {
            total_volume_in_quotes,
            vwap,
            trades_count,
        }))
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

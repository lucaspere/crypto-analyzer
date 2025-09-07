use crate::types::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use prost::Message;
use reqwest::Client;
// use tonic_web_wasm_client::Client;

mod analytics {
    include!(concat!(env!("OUT_DIR"), "/analytics.rs"));
}
mod data {
    include!(concat!(env!("OUT_DIR"), "/data.rs"));
}

use analytics::{
    GetMacdRequest, GetMovingAverageRequest, GetTradeAnalyticsRequest, SubscribeToTradesRequest,
};
const ANALYTICS_SERVER_URL: &str = "http://localhost:50051";

type Error = Box<dyn std::error::Error>;

pub struct AnalyticsService;

impl AnalyticsService {
    // fn get_client() -> AnalyticsServiceClient<
    //     GrpcWebClientService<Client<HttpConnector, GrpcWebCall<tonic::body::Body>>>,
    // > {
    //     let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build_http();
    //     let svc = tower::ServiceBuilder::new()
    //         .layer(GrpcWebClientLayer::new())
    //         .service(client);
    //     let client =
    //         AnalyticsServiceClient::with_origin(svc, ANALYTICS_SERVER_URL.try_into().unwrap());

    //     client
    // }

    // pub async fn get_trade_analytics(
    //     symbol: &str,
    //     start_timestamp: DateTime<Utc>,
    //     end_timestamp: DateTime<Utc>,
    // ) -> Result<TradeAnalytics> {
    //     let mut client = Self::get_client();

    //     let request = Request::new(GetTradeAnalyticsRequest {
    //         symbol: symbol.to_string(),
    //         start_timestamp: Some(prost_types::Timestamp {
    //             seconds: start_timestamp.timestamp(),
    //             nanos: start_timestamp.timestamp_subsec_nanos() as i32,
    //         }),
    //         end_timestamp: Some(prost_types::Timestamp {
    //             seconds: end_timestamp.timestamp(),
    //             nanos: end_timestamp.timestamp_subsec_nanos() as i32,
    //         }),
    //     });

    //     let response = client.get_trade_analytics(request).await?;
    //     let data = response.into_inner();

    //     Ok(TradeAnalytics {
    //         symbol: symbol.to_string(),
    //         total_volume_in_quotes: data.total_volume_in_quotes,
    //         vwap: data.vwap,
    //         trades_count: data.trades_count,
    //     })
    // }

    // Nossa função de cliente manual
    pub async fn get_trade_analytics(
        symbol: &str,
        start_timestamp: DateTime<Utc>,
        end_timestamp: DateTime<Utc>,
    ) -> Result<analytics::GetTradeAnalyticsResponse, Error> {
        let url = format!(
            "{}/analytics.AnalyticsService/GetTradeAnalytics",
            ANALYTICS_SERVER_URL
        );

        let client = Client::new();
        let request_bytes = analytics::GetTradeAnalyticsRequest {
            symbol: symbol.to_string(),
            start_timestamp: Some(prost_types::Timestamp {
                seconds: start_timestamp.timestamp(),
                nanos: start_timestamp.timestamp_subsec_nanos() as i32,
            }),
            end_timestamp: Some(prost_types::Timestamp {
                seconds: end_timestamp.timestamp(),
                nanos: end_timestamp.timestamp_subsec_nanos() as i32,
            }),
        }
        .encode_to_vec();

        let mut body = Vec::with_capacity(5 + request_bytes.len());
        body.push(0);
        body.extend_from_slice(&(request_bytes.len() as u32).to_be_bytes());
        body.extend_from_slice(&request_bytes);

        let response = client
            .post(url)
            .header("Content-Type", "application/grpc-web")
            .header("Accept", "application/grpc-web")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Erro na requisição: {}", response.status()).into());
        }

        let response_body = response.bytes().await?;

        if response_body.len() < 5 {
            return Err("Resposta inválida do servidor".into());
        }
        let response_message =
            analytics::GetTradeAnalyticsResponse::decode(&response_body[5..]).unwrap();

        Ok(response_message)
    }

    pub async fn get_moving_average(
        symbol: &str,
        start_timestamp: DateTime<Utc>,
        end_timestamp: DateTime<Utc>,
        window_size: u32,
    ) -> Result<Vec<MovingAveragePoint>, Error> {
        let url = format!(
            "{}/analytics.AnalyticsService/GetMovingAverage",
            ANALYTICS_SERVER_URL
        );
        let client = Client::new();

        let request = analytics::GetMovingAverageRequest {
            symbol: symbol.to_string(),
            start_timestamp: Some(prost_types::Timestamp {
                seconds: start_timestamp.timestamp(),
                nanos: start_timestamp.timestamp_subsec_nanos() as i32,
            }),
            end_timestamp: Some(prost_types::Timestamp {
                seconds: end_timestamp.timestamp(),
                nanos: end_timestamp.timestamp_subsec_nanos() as i32,
            }),
            window_size,
        };

        let response = client
            .post(url)
            .header("Content-Type", "application/grpc-web")
            .header("Accept", "application/grpc-web")
            .body(request.encode_to_vec())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Erro na requisição: {}", response.status()).into());
        }

        let response_body = response.bytes().await?;

        if response_body.len() < 5 {
            return Err("Resposta inválida do servidor".into());
        }
        let response_message =
            analytics::GetMovingAverageResponse::decode(&response_body[5..]).unwrap();

        let data = response_message;

        let points = data
            .points
            .into_iter()
            .map(|point| MovingAveragePoint {
                timestamp: point.timestamp,
                value: point.value,
            })
            .collect();

        Ok(points)
    }

    pub async fn get_macd(
        symbol: &str,
        start_timestamp: DateTime<Utc>,
        end_timestamp: DateTime<Utc>,
        fast_period: u32,
        slow_period: u32,
        signal_period: u32,
    ) -> Result<Vec<MacdPoint>, Error> {
        let url = format!(
            "{}/analytics.AnalyticsService/GetMacd",
            ANALYTICS_SERVER_URL
        );
        let client = Client::new();

        let request = analytics::GetMacdRequest {
            symbol: symbol.to_string(),
            start_timestamp: Some(prost_types::Timestamp {
                seconds: start_timestamp.timestamp(),
                nanos: start_timestamp.timestamp_subsec_nanos() as i32,
            }),
            end_timestamp: Some(prost_types::Timestamp {
                seconds: end_timestamp.timestamp(),
                nanos: end_timestamp.timestamp_subsec_nanos() as i32,
            }),
            fast_period,
            slow_period,
            signal_period,
        };
        let response = client
            .post(url)
            .header("Content-Type", "application/grpc-web")
            .header("Accept", "application/grpc-web")
            .body(request.encode_to_vec())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Erro na requisição: {}", response.status()).into());
        }

        let response_body = response.bytes().await?;

        if response_body.len() < 5 {
            return Err("Resposta inválida do servidor".into());
        }
        let response_message = analytics::GetMacdResponse::decode(&response_body[5..]).unwrap();
        let data = response_message;

        let points = data
            .points
            .into_iter()
            .map(|point| MacdPoint {
                timestamp: point.timestamp,
                macd_line: point.macd_line,
                signal_line: point.signal_line,
                histogram: point.histogram,
            })
            .collect();

        Ok(points)
    }

    pub async fn subscribe_to_trades(
        symbol: &str,
        on_trade: impl Fn(Trade) + 'static,
    ) -> Result<()> {
        log::info!("Subscribing to live trades for symbol: {}", symbol);

        // let url = format!(
        //     "{}/analytics.AnalyticsService/SubscribeToTrades",
        //     ANALYTICS_SERVER_URL
        // );
        // let client = Client::new();

        // let request = analytics::SubscribeToTradesRequest {
        //     symbol: symbol.to_string(),
        // };

        // let response = client
        //     .post(url)
        //     .header("Content-Type", "application/grpc-web")
        //     .header("Accept", "application/grpc-web")
        //     .body(request.encode_to_vec())
        //     .send()
        //     .await?;

        // if !response.status().is_success() {
        //     return Err(format!("Erro na requisição: {}", response.status()).into());
        // }

        // let response_body = response.bytes().await?;

        // // Process the stream
        // while let Some(trade_result) = response_body.().await {
        //     match trade_result {
        //         Ok(trade) => {
        //             let frontend_trade = Trade {
        //                 symbol: trade.symbol,
        //                 price: trade.price,
        //                 quantity: trade.quantity,
        //                 exchange_timestamp: trade.exchange_timestamp,
        //                 exchange: format!("{:?}", trade.exchange),
        //             };
        //             on_trade(frontend_trade);
        //         }
        //         Err(e) => {
        //             log::error!("Error in trade stream: {}", e);
        //             break;
        //         }
        //     }
        // }

        Ok(())
    }
}

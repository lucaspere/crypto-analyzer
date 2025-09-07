use crate::components::{chart::*, form::*};
use crate::services::AnalyticsService;
use crate::types::*;
use yew::prelude::*;

#[function_component(Analytics)]
pub fn analytics() -> Html {
    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);
    let vwap_data = use_state(|| None::<TradeAnalytics>);
    let sma_data = use_state(|| None::<Vec<MovingAveragePoint>>);
    let macd_data = use_state(|| None::<Vec<MacdPoint>>);

    let on_submit = {
        let loading = loading.clone();
        let error = error.clone();
        let vwap_data = vwap_data.clone();
        let sma_data = sma_data.clone();
        let macd_data = macd_data.clone();

        Callback::from(move |request: AnalyticsRequest| {
            let loading = loading.clone();
            let error = error.clone();
            let vwap_data = vwap_data.clone();
            let sma_data = sma_data.clone();
            let macd_data = macd_data.clone();

            loading.set(true);
            error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                log::info!("Making analytics request for: {:?}", request.analytics_type);

                let result = match request.analytics_type {
                    AnalyticsType::Vwap => {
                        log::info!("Requesting VWAP for symbol: {}", request.symbol);
                        match AnalyticsService::get_trade_analytics(
                            &request.symbol,
                            request.time_range.start,
                            request.time_range.end,
                        )
                        .await
                        {
                            Ok(data) => {
                                log::info!("VWAP request successful: {:?}", data);
                                vwap_data.set(Some(TradeAnalytics {
                                    symbol: request.symbol,
                                    total_volume_in_quotes: data.total_volume_in_quotes,
                                    vwap: data.vwap,
                                    trades_count: data.trades_count,
                                }));
                                Ok(())
                            }
                            Err(e) => {
                                log::error!("VWAP request failed: {}", e);
                                Err(e.to_string())
                            }
                        }
                    }
                    AnalyticsType::Sma => {
                        log::info!(
                            "Requesting SMA for symbol: {} with window: {}",
                            request.symbol,
                            request.window_size
                        );
                        match AnalyticsService::get_moving_average(
                            &request.symbol,
                            request.time_range.start,
                            request.time_range.end,
                            request.window_size,
                        )
                        .await
                        {
                            Ok(data) => {
                                log::info!("SMA request successful, got {} points", data.len());
                                sma_data.set(Some(data));
                                Ok(())
                            }
                            Err(e) => {
                                log::error!("SMA request failed: {}", e);
                                Err(e.to_string())
                            }
                        }
                    }
                    AnalyticsType::Macd => {
                        log::info!(
                            "Requesting MACD for symbol: {} with periods: {}/{}/{}",
                            request.symbol,
                            request.fast_period,
                            request.slow_period,
                            request.signal_period
                        );
                        match AnalyticsService::get_macd(
                            &request.symbol,
                            request.time_range.start,
                            request.time_range.end,
                            request.fast_period,
                            request.slow_period,
                            request.signal_period,
                        )
                        .await
                        {
                            Ok(data) => {
                                log::info!("MACD request successful, got {} points", data.len());
                                macd_data.set(Some(data));
                                Ok(())
                            }
                            Err(e) => {
                                log::error!("MACD request failed: {}", e);
                                Err(e.to_string())
                            }
                        }
                    }
                };

                loading.set(false);
                if let Err(err) = result {
                    error.set(Some(err));
                }
            });
        })
    };

    html! {
        <div class="space-y-6">
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <AnalyticsForm {on_submit} />

                <div class="space-y-4">
                    if *loading {
                        <div class="bg-white shadow rounded-lg p-6">
                            <div class="flex items-center justify-center">
                                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                                <span class="ml-2 text-gray-600">{"Loading..."}</span>
                            </div>
                        </div>
                    }

                    if let Some(err) = (*error).as_ref() {
                        <div class="bg-red-50 border border-red-200 rounded-lg p-4">
                            <div class="text-red-800">
                                <strong>{"Error: "}</strong>{err}
                            </div>
                        </div>
                    }

                    if let Some(data) = (*vwap_data).as_ref() {
                        <div class="bg-white shadow rounded-lg p-6">
                            <h3 class="text-lg font-medium text-gray-900 mb-4">{"VWAP Results"}</h3>
                            <div class="space-y-2">
                                <div class="flex justify-between">
                                    <span class="text-gray-600">{"Symbol:"}</span>
                                    <span class="font-medium">{&data.symbol}</span>
                                </div>
                                <div class="flex justify-between">
                                    <span class="text-gray-600">{"VWAP:"}</span>
                                    <span class="font-medium">{format!("{:.2}", data.vwap)}</span>
                                </div>
                                <div class="flex justify-between">
                                    <span class="text-gray-600">{"Total Volume:"}</span>
                                    <span class="font-medium">{format!("{:.2}", data.total_volume_in_quotes)}</span>
                                </div>
                                <div class="flex justify-between">
                                    <span class="text-gray-600">{"Trade Count:"}</span>
                                    <span class="font-medium">{data.trades_count}</span>
                                </div>
                            </div>
                        </div>
                    }
                </div>
            </div>

            if let Some(data) = (*sma_data).as_ref() {
                <Chart
                    data={data.clone()}
                    title={"Simple Moving Average"}
                />
            }

            if let Some(data) = (*macd_data).as_ref() {
                <MacdChart
                    data={data.clone()}
                    title={"MACD Analysis"}
                />
            }
        </div>
    }
}

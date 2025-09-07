use crate::services::AnalyticsService;
use crate::types::*;
use gloo_timers::callback::Interval;
use yew::prelude::*;

#[function_component(Trades)]
pub fn trades() -> Html {
    let trades = use_state(|| Vec::<Trade>::new());
    let symbol = use_state(|| "BTCUSDT".to_string());
    let connected = use_state(|| false);

    let on_symbol_change = {
        let symbol = symbol.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                symbol.set(input.value());
            }
        })
    };

    let on_connect = {
        let trades = trades.clone();
        let symbol = symbol.clone();
        let connected = connected.clone();

        Callback::from(move |_| {
            if *connected {
                // Disconnect
                connected.set(false);
                trades.set(Vec::new());
                log::info!("Disconnected from live trades");
            } else {
                // Connect
                connected.set(true);
                let symbol_clone = (*symbol).clone();
                let trades_clone = trades.clone();

                log::info!("Connecting to live trades for symbol: {}", symbol_clone);

                // Use the real gRPC streaming service
                wasm_bindgen_futures::spawn_local(async move {
                    let trades_for_callback = trades_clone.clone();

                    match AnalyticsService::subscribe_to_trades(&symbol_clone, move |trade| {
                        log::info!("Received real trade: {:?}", trade);

                        // Add to trades list (keep last 50)
                        let mut current_trades = (*trades_for_callback).clone();
                        current_trades.push(trade);
                        if current_trades.len() > 50 {
                            current_trades.remove(0);
                        }
                        trades_for_callback.set(current_trades);
                    })
                    .await
                    {
                        Ok(_) => {
                            log::info!("Successfully subscribed to trade stream");
                        }
                        Err(e) => {
                            log::error!("Failed to subscribe to trades: {}", e);
                            // Fall back to polling if streaming fails
                            log::info!("Falling back to polling for trade data");

                            let symbol = symbol_clone.clone();
                            let trades = trades_clone.clone();

                            let interval = Interval::new(2000, move || {
                                let symbol = symbol.clone();
                                let trades = trades.clone();

                                wasm_bindgen_futures::spawn_local(async move {
                                    let now = chrono::Utc::now();
                                    let one_minute_ago = now - chrono::Duration::minutes(1);

                                    match AnalyticsService::get_trade_analytics(
                                        &symbol,
                                        one_minute_ago,
                                        now,
                                    )
                                    .await
                                    {
                                        Ok(analytics) => {
                                            let mock_trade = Trade {
                                                symbol: symbol.clone(),
                                                price: analytics.vwap,
                                                quantity: analytics.total_volume_in_quotes
                                                    / analytics.vwap,
                                                exchange_timestamp: now.timestamp_micros() as u64,
                                                exchange: "MOCK".to_string(),
                                            };

                                            let mut current_trades = (*trades).clone();
                                            current_trades.push(mock_trade);
                                            if current_trades.len() > 50 {
                                                current_trades.remove(0);
                                            }
                                            trades.set(current_trades);
                                        }
                                        Err(e) => {
                                            log::error!("Failed to fetch trade data: {}", e);
                                        }
                                    }
                                });
                            });

                            std::mem::forget(interval);
                        }
                    }
                });
            }
        })
    };

    let symbol_for_effect = symbol.clone();
    let trades_for_effect = trades.clone();

    use_effect_with((), move |_| {
        if !(*symbol_for_effect).is_empty() {
            let symbol_clone = (*symbol_for_effect).clone();
            let trades_clone = trades_for_effect.clone();

            log::info!("Starting trade polling for symbol: {}", symbol_clone);

            // Poll for recent trades every 2 seconds.
            let interval = Interval::new(2000, move || {
                let symbol = symbol_clone.clone();
                let trades = trades_clone.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    // Get recent trades by requesting analytics for the last minute.
                    let now = chrono::Utc::now();
                    let one_minute_ago = now - chrono::Duration::minutes(1);

                    match AnalyticsService::get_trade_analytics(&symbol, one_minute_ago, now).await
                    {
                        Ok(analytics) => {
                            if analytics.trades_count > 0 && analytics.vwap > 0.0 {
                                // Create a mock trade from the analytics data.
                                let mock_trade = Trade {
                                    symbol: symbol.clone(),
                                    price: analytics.vwap,
                                    quantity: analytics.total_volume_in_quotes / analytics.vwap,
                                    exchange_timestamp: now.timestamp_micros() as u64,
                                    exchange: "MOCK".to_string(),
                                };

                                // Add to trades list (keep last 50).
                                let mut current_trades = (*trades).clone();
                                current_trades.push(mock_trade);
                                if current_trades.len() > 50 {
                                    current_trades.remove(0);
                                }
                                trades.set(current_trades);
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to fetch trade analytics for polling: {}", e);
                        }
                    }
                });
            });

            // When the component is destroyed, the interval will be dropped.
            Box::new(move || drop(interval)) as Box<dyn FnOnce()>
        } else {
            // Return a no-op cleanup function when symbol is empty
            Box::new(move || {}) as Box<dyn FnOnce()>
        }
    });

    html! {
        <div class="space-y-6">
            <div class="bg-white shadow rounded-lg p-6">
                <h2 class="text-xl font-semibold text-gray-900 mb-4">{"Live Trade Stream"}</h2>

                <div class="flex space-x-4 mb-4">
                    <input
                        type="text"
                        value={(*symbol).clone()}
                        oninput={on_symbol_change}
                        class="flex-1 border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500"
                        placeholder="Enter symbol (e.g., BTCUSDT)"
                    />
                    <button
                        onclick={on_connect}
                        class={if *connected {
                            "px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700"
                        } else {
                            "px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700"
                        }}
                    >
                        {if *connected { "Disconnect" } else { "Connect" }}
                    </button>
                </div>

                <div class="flex items-center space-x-2 mb-4">
                    <div class={if *connected {
                        "w-3 h-3 bg-green-500 rounded-full animate-pulse"
                    } else {
                        "w-3 h-3 bg-gray-400 rounded-full"
                    }}></div>
                    <span class="text-sm text-gray-600">
                        {if *connected { "Connected" } else { "Disconnected" }}
                    </span>
                </div>
            </div>

            if *connected {
                <div class="bg-white shadow rounded-lg p-6">
                    <h3 class="text-lg font-medium text-gray-900 mb-4">{"Recent Trades"}</h3>

                    if trades.is_empty() {
                        <div class="text-center py-8 text-gray-500">
                            {"Waiting for trade data..."}
                        </div>
                    } else {
                        <div class="overflow-x-auto">
                            <table class="min-w-full divide-y divide-gray-200">
                                <thead class="bg-gray-50">
                                    <tr>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                            {"Timestamp"}
                                        </th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                            {"Price"}
                                        </th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                            {"Quantity"}
                                        </th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                            {"Exchange"}
                                        </th>
                                    </tr>
                                </thead>
                                <tbody class="bg-white divide-y divide-gray-200">
                                    {trades.iter().rev().take(20).map(|trade| {
                                        html! {
                                            <tr key={trade.exchange_timestamp}>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                    {format_timestamp_us(trade.exchange_timestamp)}
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                    {format!("{:.2}", trade.price)}
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                    {format!("{:.6}", trade.quantity)}
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                    {&trade.exchange}
                                                </td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        </div>
                    }
                </div>
            } else {
                <div class="bg-gray-50 rounded-lg p-8 text-center">
                    <div class="text-gray-500">
                        {"Enter a symbol and click Connect to start receiving live trade data"}
                    </div>
                </div>
            }
        </div>
    }
}

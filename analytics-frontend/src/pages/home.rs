use yew::prelude::*;

#[function_component(Home)]
pub fn home() -> Html {
    html! {
        <div class="space-y-6">
            <div class="bg-white shadow rounded-lg p-6">
                <h1 class="text-3xl font-bold text-gray-900 mb-4">
                    {"Crypto Analytics Dashboard"}
                </h1>
                <p class="text-gray-600 mb-6">
                    {"Real-time cryptocurrency trade analytics powered by Rust and WebAssembly.
                    Get VWAP, SMA, MACD indicators and live trade data from major exchanges."}
                </p>

                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div class="bg-blue-50 p-4 rounded-lg">
                        <h3 class="text-lg font-semibold text-blue-900 mb-2">{"VWAP"}</h3>
                        <p class="text-blue-700">
                            {"Volume Weighted Average Price - the average price weighted by volume over a time period."}
                        </p>
                    </div>

                    <div class="bg-green-50 p-4 rounded-lg">
                        <h3 class="text-lg font-semibold text-green-900 mb-2">{"SMA"}</h3>
                        <p class="text-green-700">
                            {"Simple Moving Average - the average price over a specified number of periods."}
                        </p>
                    </div>

                    <div class="bg-purple-50 p-4 rounded-lg">
                        <h3 class="text-lg font-semibold text-purple-900 mb-2">{"MACD"}</h3>
                        <p class="text-purple-700">
                            {"Moving Average Convergence Divergence - a trend-following momentum indicator."}
                        </p>
                    </div>
                </div>
            </div>

            <div class="bg-white shadow rounded-lg p-6">
                <h2 class="text-xl font-semibold text-gray-900 mb-4">{"System Architecture"}</h2>
                <div class="text-gray-600 space-y-2">
                    <p>{"• Real-time data ingestion from Binance and Coinbase exchanges"}</p>
                    <p>{"• NATS messaging for high-performance data distribution"}</p>
                    <p>{"• ClickHouse for analytical data storage and queries"}</p>
                    <p>{"• Rust gRPC server with DataFusion for calculations"}</p>
                    <p>{"• Yew frontend with WebAssembly for optimal performance"}</p>
                </div>
            </div>

            <div class="bg-white shadow rounded-lg p-6">
                <h2 class="text-xl font-semibold text-gray-900 mb-4">{"Quick Start"}</h2>
                <div class="space-y-4">
                    <div>
                        <h3 class="font-medium text-gray-900">{"1. Analytics"}</h3>
                        <p class="text-gray-600">{"Navigate to the Analytics page to request VWAP, SMA, or MACD calculations for any trading pair."}</p>
                    </div>
                    <div>
                        <h3 class="font-medium text-gray-900">{"2. Live Trades"}</h3>
                        <p class="text-gray-600">{"View real-time trade data streaming from cryptocurrency exchanges."}</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

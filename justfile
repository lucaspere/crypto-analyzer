vwap symbol start_timestamp end_timestamp:
    cargo run --package analytics-cli-client -- vwap --symbol {{symbol}} --start-timestamp {{start_timestamp}} --end-timestamp {{end_timestamp}}

sma symbol start_timestamp end_timestamp window_size="20":
    cargo run --package analytics-cli-client -- sma --symbol {{symbol}} --start-timestamp {{start_timestamp}} --end-timestamp {{end_timestamp}} --window-size {{window_size}}

subscribe symbol:
    cargo run --package analytics-cli-client -- subscribe --symbol {{symbol}}

macd symbol start_timestamp end_timestamp fast_period="12" slow_period="26" signal_period="9":
    cargo run --package analytics-cli-client -- macd --symbol {{symbol}} --start-timestamp {{start_timestamp}} --end-timestamp {{end_timestamp}} --fast-period {{fast_period}} --slow-period {{slow_period}} --signal-period {{signal_period}}

end_ts       := `date +%s`
start_ts_5m  := `date -v-5M +%s`
start_ts_10m := `date -v-10M +%s`


vwap symbol start_timestamp end_timestamp:
    cargo run --package analytics-cli-client -- vwap --symbol {{symbol}} --start-timestamp {{start_timestamp}} --end-timestamp {{end_timestamp}}

sma symbol start_timestamp end_timestamp window_size="20":
    cargo run --package analytics-cli-client -- sma --symbol {{symbol}} --start-timestamp {{start_timestamp}} --end-timestamp {{end_timestamp}} --window-size {{window_size}}

macd symbol start_timestamp end_timestamp fast_period="12" slow_period="26" signal_period="9":
    cargo run --package analytics-cli-client -- macd --symbol {{symbol}} --start-timestamp {{start_timestamp}} --end-timestamp {{end_timestamp}} --fast-period {{fast_period}} --slow-period {{slow_period}} --signal-period {{signal_period}}

subscribe symbol:
    cargo run --package analytics-cli-client -- subscribe --symbol {{symbol}}


vwap-now symbol:
    @just vwap {{symbol}} {{start_ts_5m}} {{end_ts}}

sma-now symbol:
    @just sma {{symbol}} {{start_ts_5m}} {{end_ts}}

macd-now symbol:
    @just macd {{symbol}} {{start_ts_5m}} {{end_ts}}



vwap-10min symbol:
    @just vwap {{symbol}} {{start_ts_10m}} {{end_ts}}

sma-10min symbol:
    @just sma {{symbol}} {{start_ts_10m}} {{end_ts}}

macd-10min symbol:
    @just macd {{symbol}} {{start_ts_10m}} {{end_ts}}
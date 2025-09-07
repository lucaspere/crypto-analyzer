use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAnalytics {
    pub symbol: String,
    pub total_volume_in_quotes: f64,
    pub vwap: f64,
    pub trades_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MovingAveragePoint {
    pub timestamp: u64,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacdPoint {
    pub timestamp: u64,
    pub macd_line: f64,
    pub signal_line: f64,
    pub histogram: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub exchange_timestamp: u64,
    pub exchange: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnalyticsType {
    Vwap,
    Sma,
    Macd,
}

impl AnalyticsType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnalyticsType::Vwap => "VWAP",
            AnalyticsType::Sma => "SMA",
            AnalyticsType::Macd => "MACD",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl Default for TimeRange {
    fn default() -> Self {
        let now = Utc::now();
        let five_minutes_ago = now - chrono::Duration::minutes(5);
        Self {
            start: five_minutes_ago,
            end: now,
        }
    }
}

pub fn format_timestamp_us(us: u64) -> String {
    let seconds = (us / 1_000_000) as i64;
    let nanoseconds = (us % 1_000_000 * 1000) as u32;
    let dt = DateTime::from_timestamp(seconds, nanoseconds).unwrap_or_default();
    dt.format("%Y-%m-%d %H:%M:%S.%3f").to_string()
}

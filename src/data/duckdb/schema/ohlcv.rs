use serde::{Deserialize, Serialize};

use crate::data::duckdb::types::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ohlcv {
    pub symbol: String,
    pub timestamp: Timestamp,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

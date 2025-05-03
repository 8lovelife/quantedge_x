use serde::{Deserialize, Serialize};

use crate::data::duckdb::types::Timestamp;

#[derive(Debug, Serialize, Deserialize)]
pub struct BacktestRunTrade {
    pub id: Option<i64>,
    pub strategy_id: i64,
    pub run_id: i64,
    pub date: String,
    pub trade_type: String, // Allowed values: "buy" or "sell"
    pub result: String,     // Allowed values: "win" or "loss"
    pub profit: f64,
    pub created_at: Timestamp,
}

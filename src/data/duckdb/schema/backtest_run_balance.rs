use serde::{Deserialize, Serialize};

use crate::data::duckdb::types::Timestamp;

#[derive(Debug, Serialize, Deserialize)]
pub struct BacktestRunBalance {
    pub id: Option<i64>,
    pub strategy_id: i64,
    pub run_id: i64,
    pub date: String,
    pub capital: f64,
    pub trades: u32,
    pub created_at: Timestamp,
}

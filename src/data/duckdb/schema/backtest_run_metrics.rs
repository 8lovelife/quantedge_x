use serde::{Deserialize, Serialize};

use crate::data::duckdb::types::Timestamp;

#[derive(Debug, Serialize, Deserialize)]
pub struct BacktestRunMetrics {
    pub id: Option<i64>,
    pub strategy_id: i64,
    pub run_id: i64,
    pub date: String,
    pub strategy_return: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub sharpe_ratio: f64,
    pub created_at: Timestamp,
}

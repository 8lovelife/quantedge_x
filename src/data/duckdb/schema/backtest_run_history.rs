use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::data::duckdb::types::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacktestRunHistory {
    pub id: Option<i64>,
    pub strategy_id: i64,
    pub parameters: Value,
    pub market_details: Value,
    pub status: String,
    pub start_time: Timestamp,
    pub performance: Option<Value>,
    pub end_time: Option<Timestamp>,
    pub created_at: Option<Timestamp>,
}

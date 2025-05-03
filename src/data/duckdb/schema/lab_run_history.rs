use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::data::duckdb::types::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabRunHistory {
    pub id: Option<i64>,
    pub template_id: i64,
    pub parameters: Value,
    pub market_details: Value,
    pub status: String,
    pub start_time: Timestamp,
    pub performance: Option<Value>,
    pub end_time: Option<Timestamp>,
    pub created_at: Option<Timestamp>,
}

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::data::duckdb::types::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyTemplate {
    pub id: Option<i64>,
    pub name: String,
    pub r#type: String, // type is a reserved keyword
    pub description: String,
    pub info: String,
    pub parameters: Option<StrategyParameterConfig>,
    pub risk: Option<Value>,
    pub execution: Option<Value>,
    pub performance: Option<Value>,
    pub likes: Option<i64>,
    pub usage: Option<i64>,
    pub author: Option<String>,
    pub latest_lab_backtest_version: Option<i64>,

    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyParameterConfig {
    pub options: Option<Value>,
    pub default: Option<Value>,
}

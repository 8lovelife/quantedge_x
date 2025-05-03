use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::data::duckdb::types::Timestamp;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StrategyBuilds {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub algorithm_type: String,
    pub algorithm_sub_type: Option<String>,
    pub lifecycle: String,
    pub progress: String,
    pub assets: Option<Value>,
    pub parameters: Option<Value>,
    #[serde(rename = "marketDetails")]
    pub market_details: Option<Value>,
    #[serde(rename = "applyBacktestVersion")]
    pub apply_backtest_version: Option<i64>,
    pub risk: Option<Value>,
    #[serde(rename = "latestBacktestVersion")]
    pub latest_backtest_version: Option<i64>,
    pub backtest_performance: Option<Value>,
    pub paper_performance: Option<Value>,
    pub live_performance: Option<Value>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressInType {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub algorithm_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressInAssets {
    pub id: i64,
    pub symbol: String,
    pub weight: u32,
    pub direction: String,
}

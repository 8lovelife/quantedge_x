use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeStrategy {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub algorithm: String,
    pub risk: String,
    pub allocation: u32,
    pub timeframe: String,
    pub assets: String, // Storing JSON string for parameters
    pub status: String,
    pub parameters: String, // Storing JSON string for parameters
    #[serde(rename = "latestVersion")]
    pub latest_backtest_version: Option<i64>,
}

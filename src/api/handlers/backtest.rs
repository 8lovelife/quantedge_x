use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{
    api::AppState,
    data::{duckdb::schema::backtest_run_history::BacktestRunHistory, sleddb::ChartDB},
    engine::{
        backtest_result::{Balance, Trade},
        parameters::StrategyRunParameters,
    },
    indicators::calculator::{DistributionData, MonthlyReturnData},
    service::backtest_service::{MarketDetails, RunBacktestData, RunBacktestReq},
};

use super::StrategyBacktestRunRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridResult {
    pub version: u64,
    pub generated_at: String,
    pub meta: GridMeta,
    // pub entries: Vec<GridEntry>,
    // pub leaderboard: Vec<GridEntry>,
    // pub pareto_front: Vec<GridEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridMeta {
    pub template_id: i64,
    pub pair: String,
    pub timeframe: String,
    pub initial_capital: f64,
    pub position_type: String,
    pub grid_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GridParams {
    pub fast_period: Option<Vec<u32>>,
    pub slow_period: Option<Vec<u32>>,
    pub entry_threshold: Option<Vec<f64>>,
    pub exit_threshold: Option<Vec<f64>>,
    pub stop_loss: Option<Vec<f64>>,
    pub take_profit: Option<Vec<f64>>,
    pub position_size: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabRunGridSearchRequest {
    pub template_id: u64,
    #[serde(rename = "type")]
    pub r#type: String,
    pub sub_type: Option<String>,
    pub grid_params: GridParams,
    pub pairs: String,
    pub timeframe: String,
    pub initial_capital: f64,
    pub position_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LabBacktestRunRequest {
    #[serde(rename = "templateId")]
    pub template_id: i64,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "subType")]
    pub sub_type: Option<String>,
    pub pairs: String,
    pub timeframe: String,
    #[serde(rename = "initialCapital")]
    pub initial_capital: f64,
    pub params: StrategyRunParameters,
    #[serde(rename = "positionType")]
    #[serde(default = "default_position_type")]
    pub position_type: String,
}

#[derive(Debug, Serialize)]
pub struct StrategyBacktestRunResponse {
    pub params: Option<StrategyRunParameters>,
    pub trades: Vec<Trade>,
    pub balances: Vec<Balance>,
    #[serde(rename = "monthlyReturns")]
    pub monthly_returns: Vec<MonthlyReturnData>,
    #[serde(rename = "returnDistribution")]
    pub return_distribution: Vec<DistributionData>,
    pub metrics: Metrics,
    pub version: Option<i64>,
    pub date: Option<String>,
}

fn default_position_type() -> String {
    "long".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestParameters {
    pub params: StrategyParams,
    pub timeframe: String,
    #[serde(rename = "strategyId")]
    pub strategy_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestHistoryDataReq {
    #[serde(rename = "strategyId")]
    pub strategy_id: i64,
    pub version: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct StrategyRunHistoryRes {
    pub historys: Vec<BacktestRunHistory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyParams {
    #[serde(rename = "smaFast")]
    pub sma_fast: f64,
    #[serde(rename = "smaSlow")]
    pub sma_slow: f64,
    #[serde(rename = "riskLevel")]
    pub risk_level: String,
    #[serde(rename = "stopLoss")]
    pub stop_loss: f64,
    #[serde(rename = "takeProfit")]
    pub take_profit: f64,
    #[serde(rename = "useTrailingStop")]
    pub use_trailing_stop: bool,
    #[serde(rename = "trailingStopDistance")]
    pub trailing_stop_distance: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunBacktestHistoryReq {
    #[serde(rename = "strategyId")]
    pub strategy_id: i64,
    #[serde(default = "default_top")]
    pub top: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunBacktestHistoryRes {
    pub id: i64,
    #[serde(rename = "startDate")]
    pub start_date: String,
    #[serde(rename = "endDate")]
    pub end_date: String,
    pub status: String,
}

fn default_top() -> u32 {
    5
}

#[derive(Debug, Serialize)]
pub struct Metrics {
    #[serde(rename = "strategyReturn")]
    pub strategy_return: f64,
    #[serde(rename = "maxDrawdown")]
    pub max_drawdown: f64,
    #[serde(rename = "winRate")]
    pub win_rate: f64,
    #[serde(rename = "sharpeRatio")]
    pub sharpe_ratio: f64,
    #[serde(rename = "totalTrades")]
    pub total_trades: u64,
    #[serde(rename = "profitFactor")]
    pub profit_factor: f64,
}

#[derive(Debug, Serialize)]
pub struct RunBacktestDataRes {
    pub params: Option<StrategyParams>,
    pub trades: Vec<Trade>,
    pub balances: Vec<Balance>,
    #[serde(rename = "monthlyReturns")]
    pub monthly_returns: Vec<MonthlyReturnData>,
    #[serde(rename = "returnDistribution")]
    pub return_distribution: Vec<DistributionData>,
    pub metrics: Metrics,
    pub version: Option<i64>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacktestDataHistoryReq {
    pub run_id: i64,
    pub strategy_id: i64,
}

// pub async fn run_backtest(
//     State(state): State<AppState>,
//     Json(params): Json<BacktestParameters>,
// ) -> Result<Json<RunBacktestDataRes>, StatusCode> {
//     let strategy_params = params.params;
//     let run_params = RunBacktestReq {
//         timeframe: params.timeframe,
//         strategy_id: params.strategy_id,
//         params: strategy_params.clone().into(),
//     };
//     let backtest_result = state
//         .backtest_service
//         .run_backtest(run_params)
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

//     let response = RunBacktestDataRes {
//         metrics: backtest_result.metrics.into(),
//         trades: backtest_result.trades,
//         balances: backtest_result.balances,
//         params: Some(strategy_params),
//         monthly_returns: backtest_result.monthly_returns,
//         return_distribution: backtest_result.return_distribution,
//         version: backtest_result.version,
//         date: backtest_result.date,
//     };

//     Ok(Json(response))
// }

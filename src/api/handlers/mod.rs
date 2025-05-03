use axum::Json;
use backtest::{Metrics, StrategyParams};
use chrono::Utc;
use serde::{Deserialize, Serialize};

pub mod algorithm;
pub mod backtest;
pub mod market_price;
pub mod trade;
pub mod trade_strategy;
pub mod user_auth;

pub use algorithm::*;
pub use market_price::*;
pub use trade::*;
pub use trade_strategy::*;
pub use user_auth::*;

use crate::service::backtest_service::{BacktestMetrics, RunBacktestParameters};

#[derive(Debug, Deserialize, Serialize)]
pub struct PageQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub search: Option<String>,
    pub sort: Option<String>,
    pub status: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    10
}

#[derive(Debug, Serialize)]
pub struct PingResponse {
    pub message: String,
    pub timestamp: String,
}

pub async fn ping() -> Json<PingResponse> {
    Json(PingResponse {
        message: "pong".to_string(),
        timestamp: Utc::now().to_rfc3339(),
    })
}

impl From<StrategyParams> for RunBacktestParameters {
    fn from(value: StrategyParams) -> Self {
        RunBacktestParameters {
            sma_fast: value.sma_fast,
            sma_slow: value.sma_slow,
            stop_loss: value.stop_loss,
            risk_level: value.risk_level,
            take_profit: value.take_profit,
            use_trailing_stop: value.use_trailing_stop,
            trailing_stop_distance: value.trailing_stop_distance,
        }
    }
}

impl From<BacktestMetrics> for Metrics {
    fn from(value: BacktestMetrics) -> Self {
        Metrics {
            sharpe_ratio: value.sharpe_ratio,
            strategy_return: value.strategy_return,
            max_drawdown: value.max_drawdown,
            win_rate: value.win_rate,
            total_trades: value.total_trades,
            profit_factor: value.profit_factor,
        }
    }
}

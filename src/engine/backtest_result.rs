use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BacktestResult {
    pub total_return: f64,
    pub cagr: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub profit_factor: f64,
    pub win_rate: f64,
    pub final_capital: f64,
    pub trades: Vec<Trade>,
    pub balances: Vec<Balance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub date: String,
    #[serde(rename = "balance")]
    pub capital: f64,
    // #[serde(rename = "marketBalance")]
    // pub market: f64,
    pub trades: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub date: String,
    #[serde(rename = "type")]
    pub trade_type: TradeType,
    pub result: TradeResultType,
    pub profit: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone)]
pub enum TradeResultType {
    #[serde(rename = "win")]
    Win,
    #[serde(rename = "loss")]
    Loss,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TradeType {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyParameters {
    #[serde(rename = "smaFast")]
    sma_fast: u32,
    #[serde(rename = "smaSlow")]
    sma_slow: u32,
    #[serde(rename = "riskLevel")]
    risk_level: String,
    #[serde(rename = "stopLoss")]
    stop_loss: u32,
    #[serde(rename = "takeProfit")]
    take_profit: u32,
    #[serde(rename = "useTrailingStop")]
    use_trailing_stop: bool,
    #[serde(rename = "trailingStopDistance")]
    trailing_stop_distance: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RiskLevel {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
}

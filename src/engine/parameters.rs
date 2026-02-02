use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::strategy::{market_data::MarketData, strategy_type::StrategyType};

// use super::backtester::PortfolioAsset;

pub struct StrategyRunParams {
    pub name: String,
    pub strategy: StrategyType,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub use_trailing_stop: bool,
    pub trailing_stop_distance: f64,
    pub starting_capital: f64,
    pub market_data: Option<Vec<MarketData>>,
    pub risk_per_trade: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRunParameters {
    pub position_type: Option<String>,
    pub mean_type: Option<String>,
    pub ma_type: Option<String>,
    pub fast_period: Option<u32>,
    pub slow_period: Option<u32>,
    pub entry_threshold: Option<f64>,
    pub exit_threshold: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub risk_per_trade: Option<f64>,
    pub position_size: Option<f64>,
    pub max_concurrent_positions: Option<u32>,
    pub slippage: Option<f64>,
    pub commission: Option<f64>,
    pub entry_delay: Option<u32>,
    pub min_holding_period: Option<u32>,
    pub max_holding_period: Option<u32>,
    pub reversion_style: Option<String>,
    pub lookback_period: Option<u32>,
    pub exit_z_score: Option<f64>,
    pub entry_z_score: Option<f64>,
    pub band_multiplier: Option<f64>,
    pub cooldown_period: Option<u32>,
}

impl StrategyRunParameters {
    pub fn normalize_percentages(&mut self) {
        fn div100(x: &mut Option<f64>) {
            if let Some(v) = x {
                *x = Some(*v / 100.0);
            }
        }
        div100(&mut self.stop_loss);
        div100(&mut self.take_profit);
        div100(&mut self.risk_per_trade);
        div100(&mut self.position_size);
        div100(&mut self.slippage);
        div100(&mut self.commission);
    }

    pub fn normalized(&self) -> Self {
        fn norm(x: &Option<f64>) -> Option<f64> {
            x.map(|v| v / 100.0)
        }

        StrategyRunParameters {
            position_type: self.position_type.clone(),
            mean_type: self.mean_type.clone(),
            ma_type: self.ma_type.clone(),
            fast_period: self.fast_period,
            slow_period: self.slow_period,
            entry_threshold: self.entry_threshold,
            exit_threshold: self.exit_threshold,
            stop_loss: norm(&self.stop_loss),
            take_profit: norm(&self.take_profit),
            risk_per_trade: norm(&self.risk_per_trade),
            position_size: norm(&self.position_size),
            max_concurrent_positions: self.max_concurrent_positions,
            slippage: norm(&self.slippage),
            commission: norm(&self.commission),
            entry_delay: self.entry_delay,
            min_holding_period: self.min_holding_period,
            max_holding_period: self.max_holding_period,
            reversion_style: self.reversion_style.clone(),
            lookback_period: self.lookback_period,
            exit_z_score: self.exit_z_score,
            entry_z_score: self.entry_z_score,
            band_multiplier: self.band_multiplier,
            cooldown_period: self.cooldown_period,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunLabStrategy {
    pub r#type: String,
    pub sub_type: Option<String>,
    pub initial_capital: f64,
    pub position_type: String,
    pub strategy_run_params: StrategyRunParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacktestInput {
    pub r#type: String,
    pub initial_capital: f64,
    pub strategy_run_params: Value,
}

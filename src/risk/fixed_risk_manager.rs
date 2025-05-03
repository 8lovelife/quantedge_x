use serde_json::Value;

use crate::{domain::RiskManager, strategy::signal::Signal};

pub struct FixedRiskManager {
    pub stop_loss: f64,   // 如 0.05 = 5%
    pub take_profit: f64, // 如 0.10 = 10%
}

impl FixedRiskManager {
    pub fn from_params(params: &Value) -> Box<dyn RiskManager> {
        let stop_loss = params
            .get("stopLoss")
            .and_then(Value::as_f64)
            .unwrap_or(0.05);

        let take_profit = params
            .get("takeProfit")
            .and_then(Value::as_u64)
            .map(|v| v as f64)
            .unwrap_or(0.1);

        Box::new(FixedRiskManager {
            stop_loss,
            take_profit,
        })
    }
}

impl RiskManager for FixedRiskManager {
    fn check_risk(&self, price: f64, position: f64, entry_price: Option<f64>) -> Option<Signal> {
        if let Some(ep) = entry_price {
            if position > 0.0 {
                // 多头
                if price <= ep * (1.0 - self.stop_loss)  // 跌破止损
                   || price >= ep * (1.0 + self.take_profit)
                // 突破止盈
                {
                    return Some(Signal::Exit);
                }
            } else if position < 0.0 {
                // 空头
                if price >= ep * (1.0 + self.stop_loss)  // 涨破止损
                   || price <= ep * (1.0 - self.take_profit)
                // 跌破止盈
                {
                    return Some(Signal::Exit);
                }
            }
        }
        None
    }
}

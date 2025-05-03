use serde_json::Value;

use crate::domain::RiskManager;

use super::fixed_risk_manager::FixedRiskManager;

pub struct RiskManagerFactory;

impl RiskManagerFactory {
    pub fn build(params: &Value) -> Box<dyn RiskManager> {
        let stop_loss = params
            .get("stopLoss")
            .and_then(Value::as_f64)
            .unwrap_or(0.05);

        let take_profit = params
            .get("takeProfit")
            .and_then(Value::as_f64)
            .unwrap_or(0.1);

        Box::new(FixedRiskManager {
            stop_loss,
            take_profit,
        })
    }
}

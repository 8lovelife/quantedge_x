use serde_json::Value;

use crate::domain::PositionSizer;

use super::{fixed_fractional_sizer::FixedFractionalSizer, fixed_size_sizer::FixedSizeSizer};

pub struct SizerFactory;

impl SizerFactory {
    pub fn build(params: &Value) -> Box<dyn PositionSizer> {
        let risk_per_trade = params.get("riskPerTrade").and_then(Value::as_f64);
        let stop_loss = params
            .get("stopLoss")
            .and_then(Value::as_f64)
            .unwrap_or(0.02);
        let position_size = params
            .get("positionSize")
            .and_then(Value::as_f64)
            .unwrap_or(1.0);

        if let Some(risk) = risk_per_trade {
            Box::new(FixedFractionalSizer::new(risk, stop_loss, position_size))
        } else {
            Box::new(FixedSizeSizer::new(position_size))
        }
    }
}

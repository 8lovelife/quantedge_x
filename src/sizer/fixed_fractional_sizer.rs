use crate::domain::PositionSizer;
use crate::strategy::strategy_context::StrategyContext;

pub struct FixedFractionalSizer {
    risk_per_trade: f64,
    stop_loss: f64,
    max_position_size: f64,
}

impl FixedFractionalSizer {
    pub fn new(risk_per_trade: f64, stop_loss: f64, max_position_size: f64) -> Self {
        assert!(
            (0.0 < risk_per_trade && risk_per_trade <= 1.0)
                && (0.0 < stop_loss && stop_loss <= 1.0)
                && (0.0 < max_position_size && max_position_size <= 1.0),
            "risk_per_trade, stop_loss, max_position_size must all be in (0,1]"
        );
        Self {
            risk_per_trade,
            stop_loss,
            max_position_size,
        }
    }
}

impl PositionSizer for FixedFractionalSizer {
    fn calc(&self, price: f64, ctx: &StrategyContext) -> f64 {
        let equity = ctx.account_equity.max(0.0);

        let max_risk_amount = equity * self.risk_per_trade;
        let risk_qty = if price > 0.0 {
            max_risk_amount / (price * self.stop_loss)
        } else {
            0.0
        };

        let max_position_amount = equity * self.max_position_size;
        let position_qty = if price > 0.0 {
            max_position_amount / price
        } else {
            0.0
        };

        risk_qty.min(position_qty)
    }
}

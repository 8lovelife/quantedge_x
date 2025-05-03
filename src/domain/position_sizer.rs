use crate::strategy::strategy_context::StrategyContext;

pub trait PositionSizer {
    fn calc(&self, entry_price: f64, ctx: &StrategyContext) -> f64;
}

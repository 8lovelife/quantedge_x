use super::{
    market_data::MarketData,
    position::{PositionType, TradePosition},
    signal::Signal,
    strategy_context::StrategyContext,
};

pub trait Strategy {
    fn generate_signal(&mut self, price: f64, position: f64) -> Signal;
    fn update(&mut self, market_data: &MarketData, current_position: &Option<TradePosition>);
    fn name(&self) -> &str;
    fn apply_parameters(&mut self, entry_threshold: Option<f64>, exit_threshold: Option<f64>);
    fn on_tick(&mut self, ctx: &mut StrategyContext, data: &MarketData) -> Signal {
        let raw_sig = self.generate_signal(data.close_price, ctx.position);
        // ctx.apply_signal(sig, data.timestamp.clone(), data.close_price);
        match raw_sig {
            Signal::EnterLong(_) if !self.supports_long() => Signal::Hold,
            Signal::EnterShort(_) if !self.supports_short() => Signal::Hold,
            other => other,
        }
    }

    fn position_type(&self) -> &PositionType;

    fn supports_long(&self) -> bool {
        matches!(
            self.position_type(),
            PositionType::Long | PositionType::Both
        )
    }
    fn supports_short(&self) -> bool {
        matches!(
            self.position_type(),
            PositionType::Short | PositionType::Both
        )
    }
}

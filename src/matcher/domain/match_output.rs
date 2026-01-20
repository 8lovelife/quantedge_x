use crate::{
    matcher::domain::execution_result::TradeEventResult, models::level_update::LevelChange,
};

pub struct MatchOutput {
    pub deltas: LevelChange,
    pub trade_event: TradeEventResult,
}

impl MatchOutput {
    pub fn new(deltas: LevelChange, trade_event: TradeEventResult) -> Self {
        Self {
            deltas,
            trade_event,
        }
    }
}

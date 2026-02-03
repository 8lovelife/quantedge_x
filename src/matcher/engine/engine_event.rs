use crate::{
    matcher::{domain::execution_result::TradeEventResult, engine::event_kind::EventKind},
    models::level_update::LevelChange,
};

#[derive(Debug)]
pub enum EngineEvent {
    LevelChange(LevelChange),
    TradeEventResult(TradeEventResult),
}

impl EngineEvent {
    pub fn kind(&self) -> EventKind {
        match self {
            EngineEvent::LevelChange(_) => EventKind::LevelChange,
            EngineEvent::TradeEventResult(_) => EventKind::TradeEventResult,
        }
    }

    pub fn level_change(self) -> Option<LevelChange> {
        match self {
            EngineEvent::LevelChange(lc) => Some(lc),
            _ => None,
        }
    }

    pub fn trade_event_result(self) -> Option<TradeEventResult> {
        match self {
            EngineEvent::TradeEventResult(trade) => Some(trade),
            _ => None,
        }
    }
}

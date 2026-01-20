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
}

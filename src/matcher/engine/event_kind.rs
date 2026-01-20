#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    LevelChange,
    TradeEventResult,
}

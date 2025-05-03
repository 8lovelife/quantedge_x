use super::direction::Direction;

/// 上下文仅存储状态和配对信息，不直接持有 executor 或 sizer
pub struct StrategyContext {
    /// 当前净持仓，>0 多头，<0 空头，==0 空仓
    pub position: f64,
    /// 当前账户净值，可选
    pub account_equity: f64,
    /// 当前未平仓入场信息：(entry_time, entry_price, quantity, direction)
    pub current_entry: Option<(String, f64, f64, Direction)>,
}

impl StrategyContext {
    pub fn new(initial_capital: f64) -> Self {
        Self {
            position: 0.0,
            account_equity: initial_capital,
            current_entry: None,
        }
    }

    /// 清理与实际 position 不符的 entry
    pub fn reconcile_entry(&mut self) {
        if let Some((_, _, _, dir)) = &self.current_entry {
            match dir {
                Direction::Long if self.position <= 0.0 => self.current_entry = None,
                Direction::Short if self.position >= 0.0 => self.current_entry = None,
                _ => {}
            }
        }
    }
}

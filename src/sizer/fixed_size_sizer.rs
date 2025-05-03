use crate::{domain::PositionSizer, strategy::strategy_context::StrategyContext};

/// 固定头寸计算器：每次开仓都下固定数量
pub struct FixedSizeSizer {
    size: f64,
}

impl FixedSizeSizer {
    /// 创建新的 FixedSizeSizer
    /// # Arguments
    /// * `size` - 每单数量，例如1.0表示1个合约/单位
    pub fn new(size: f64) -> Self {
        assert!(size >= 0.0, "size must be non-negative");
        Self { size }
    }
}

impl PositionSizer for FixedSizeSizer {
    /// 计算下单数量：直接返回固定值
    fn calc(&self, _price: f64, _ctx: &StrategyContext) -> f64 {
        self.size
    }
}

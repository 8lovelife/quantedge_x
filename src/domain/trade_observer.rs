use crate::strategy::direction::Direction;

pub trait TradeObserver {
    fn on_trade(&mut self, record: &TradeRecord);
}

/// 一笔完整交易的记录
#[derive(Debug, Clone)]
pub struct TradeRecord {
    /// 开仓时间，ISO 8601 格式字符串或 DateTime<Utc>
    pub entry_time: String,
    /// 平仓时间
    pub exit_time: String,
    /// 开仓价格
    pub entry_price: f64,
    /// 平仓价格
    pub exit_price: f64,
    /// 交易手数或合约数量
    pub quantity: f64,
    /// 多头或空头方向
    pub direction: Direction,
    /// 单笔盈亏
    pub pnl: f64,
    /// 持仓时长（以分钟/条 bar 数等计）
    pub holding_time: String,
}

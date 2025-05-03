#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Direction {
    Long,  // 多头：开仓时买入，平仓时卖出
    Short, // 空头：开仓时卖出，平仓时买入
}

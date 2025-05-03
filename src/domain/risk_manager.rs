use crate::strategy::signal::Signal;

pub trait RiskManager {
    /// 每个 Tick 调用，传入当前价格与持仓信息
    /// 返回 Some(Signal::Exit) 表示要平仓，否则 None
    fn check_risk(&self, price: f64, position: f64, entry_price: Option<f64>) -> Option<Signal>;
}

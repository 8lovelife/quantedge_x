#[derive(Debug, Clone)]
pub struct TradeTick {
    pub symbol: String,
    pub price: f64,
    pub qty: f64,
    pub ts: i64, // exchange timestamp - ms
}

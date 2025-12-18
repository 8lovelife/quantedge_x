use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct MarketPriceData {
    pub symbol: String,
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

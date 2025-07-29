use chrono::{DateTime, Utc};

// O:   Open Price
// H:   Highest Price
// L:   Lowest Price
// C:   Close Price
// V:   VOlumn
#[derive(Debug, Clone)]
pub struct Candle {
    pub symbol: String,
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

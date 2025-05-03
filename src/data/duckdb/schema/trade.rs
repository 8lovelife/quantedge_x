use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Trade {
    pub id: String,
    pub strategy: String,
    pub trade_type: String,
    pub asset: String,
    pub amount: f64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    pub status: String,
    pub profit: Option<f64>,
}

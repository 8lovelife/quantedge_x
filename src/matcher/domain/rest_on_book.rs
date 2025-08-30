use chrono::{DateTime, Utc};

use crate::matcher::domain::{order::OrderSide, price_ticks::PriceTicks, qty_lots::QtyLots};

pub struct RestOnBook {
    pub side: OrderSide,
    pub limit: PriceTicks,
    pub qty: QtyLots,
    pub expires_at: Option<DateTime<Utc>>,
}

use crate::matcher::domain::{price_ticks::PriceTicks, qty_lots::QtyLots};

#[derive(Debug, Clone)]
pub struct TradeTick {
    pub symbol: String,
    pub price: f64,
    pub qty: f64,
    pub ts: i64, // exchange timestamp - ms
}

#[derive(Debug, Clone)]
pub struct TradeTickInternal {
    pub symbol: String,
    pub price: PriceTicks,
    pub qty: QtyLots,
    pub ts: i64, // exchange timestamp - ms
}

impl TradeTickInternal {
    pub fn to_f64(&self, tick_size: f64, lot_size: f64) -> TradeTick {
        TradeTick {
            symbol: self.symbol.clone(),
            price: self.price.to_f64(tick_size),
            qty: self.qty.to_f64(lot_size),
            ts: self.ts,
        }
    }
}

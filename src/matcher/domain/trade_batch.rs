use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

use crate::models::trade_tick::TradeTick;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeBatch {
    pub symbol: String,
    pub order_id: u64,
    pub trades: Vec<TradeTick>,
}

impl fmt::Display for TradeBatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_qty: f64 = self.trades.iter().map(|t| t.qty).sum();

        write!(
            f,
            "TradeBatch(Symbol: {}, Order: {}, Count: {}, TotalQty: {}): {:?}",
            self.symbol,
            self.order_id,
            self.trades.len(),
            total_qty,
            self.trades // This uses TradeTick's Debug/Display
        )
    }
}

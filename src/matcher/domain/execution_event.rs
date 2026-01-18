use anyhow::Ok;

use crate::{
    matcher::domain::{price_ticks::PriceTicks, qty_lots::QtyLots, reject_reason::RejectReason},
    models::trade_tick::{TradeTick, TradeTickInternal},
    utils::time::now_millis,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum ExecutionEvent {
    Placed {
        order_id: Option<u64>,
        qty: QtyLots,
        price: PriceTicks,
        expires_at: Option<u64>,
    },

    Traded {
        taker_order_id: u64,
        maker_order_id: u64,
        qty: QtyLots,
        price: PriceTicks,
        taker_completed: bool,
        maker_completed: bool,
    },

    Cancelled {
        order_id: u64,
        cancelled: QtyLots,
        fully_cancelled: bool,
    },

    Rejected {
        order_id: u64,
        reason: RejectReason,
    },
}

impl ExecutionEvent {
    pub fn build_trade_tick(&self, symbol: &str) -> Option<TradeTickInternal> {
        if let ExecutionEvent::Traded { qty, price, .. } = self {
            let tick = TradeTickInternal {
                symbol: symbol.to_string(),
                price: *price,
                qty: *qty,
                ts: now_millis(),
            };
            return Some(tick);
        }
        None
    }
}

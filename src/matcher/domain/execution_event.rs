use crate::matcher::domain::{
    price_ticks::PriceTicks, qty_lots::QtyLots, reject_reason::RejectReason,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
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

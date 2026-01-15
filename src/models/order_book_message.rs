use crate::matcher::domain::{price_ticks::PriceTicks, qty_lots::QtyLots};

pub enum OrderBookMessage {
    Snapshot {
        bids: Vec<(PriceTicks, QtyLots)>,
        asks: Vec<(PriceTicks, QtyLots)>,
        last_update_id: u64,
    },
    Delta {
        bids: Vec<(PriceTicks, QtyLots)>,
        asks: Vec<(PriceTicks, QtyLots)>,
        start_id: u64,
        end_id: u64,
    },
}

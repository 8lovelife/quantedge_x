use crate::matcher::domain::{price_ticks::PriceTicks, qty_lots::QtyLots};

#[derive(Debug)]
pub enum OrderBookMessage {
    Snapshot {
        bids: Vec<(PriceTicks, QtyLots)>,
        asks: Vec<(PriceTicks, QtyLots)>,
        last_update_id: u64,
    },
    Delta {
        bids: Vec<(PriceTicks, Option<QtyLots>)>,
        asks: Vec<(PriceTicks, Option<QtyLots>)>,
        start_id: u64,
        end_id: u64,
    },
}

impl OrderBookMessage {
    pub fn end_id(&self) -> u64 {
        match self {
            OrderBookMessage::Snapshot { last_update_id, .. } => *last_update_id,
            OrderBookMessage::Delta { end_id, .. } => *end_id,
        }
    }
}

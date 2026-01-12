use std::collections::HashMap;

use crate::{
    matcher::domain::{
        execution_result::ExecutionResult, price_ticks::PriceTicks, qty_lots::QtyLots,
    },
    models::order_book_message::OrderBookMessage,
};

pub struct DeltaBuilder {
    changes: HashMap<PriceTicks, QtyLots>,
    first_update_id: u64,
    last_update_id: u64,
    threshold: usize,
}

impl DeltaBuilder {
    pub fn on_event_result(&mut self, result: ExecutionResult, update_id: u64) {
        if self.changes.len() >= self.threshold {
            self.flush();
        }
    }

    pub fn flush(&mut self) -> OrderBookMessage {
        OrderBookMessage::Delta {
            bids: Vec::new(),
            asks: Vec::new(),
            start_id: 1,
            end_id: 1,
        }
    }
}

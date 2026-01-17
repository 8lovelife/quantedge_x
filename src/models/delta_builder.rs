use crate::{
    domain::order::Side,
    models::{
        level_update::{LevelChange, LevelUpdate},
        order_book_message::OrderBookMessage,
    },
};

pub struct DeltaBuilder {
    changes: Vec<LevelUpdate>,
    first_update_id: Option<u64>,
    last_update_id: Option<u64>,
    threshold: usize,
}

impl DeltaBuilder {
    pub fn new(threshold: usize) -> Self {
        Self {
            changes: Vec::new(),
            first_update_id: None,
            last_update_id: None,
            threshold,
        }
    }
    pub fn on_level_updates(&mut self, updates: LevelChange) -> Option<OrderBookMessage> {
        let update_id = updates.update_id;
        let level_updates = updates.level_updates;
        self.first_update_id.get_or_insert(update_id);
        self.last_update_id = Some(update_id);

        self.changes.extend(level_updates);

        if self.changes.len() >= self.threshold {
            Some(self.flush())
        } else {
            None
        }
    }

    pub fn flush(&mut self) -> OrderBookMessage {
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for update in self.changes.drain(..) {
            match update.side {
                Side::Bid => bids.push((update.price, update.new_qty)),
                Side::Ask => asks.push((update.price, update.new_qty)),
            }
        }

        let msg = OrderBookMessage::Delta {
            bids,
            asks,
            start_id: self.first_update_id.unwrap(),
            end_id: self.last_update_id.unwrap(),
        };

        self.first_update_id = None;
        self.last_update_id = None;

        msg
    }
}

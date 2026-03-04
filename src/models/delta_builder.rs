use std::collections::BTreeMap;

use crate::{
    domain::order::Side,
    matcher::domain::{price_ticks::PriceTicks, qty_lots::QtyLots},
    models::{level_update::LevelChange, order_book_message::OrderBookMessage},
};

pub struct DeltaBuilder {
    bids: BTreeMap<PriceTicks, Option<QtyLots>>,
    asks: BTreeMap<PriceTicks, Option<QtyLots>>,
    first_update_id: Option<u64>,
    last_update_id: Option<u64>,
}

impl DeltaBuilder {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            first_update_id: None,
            last_update_id: None,
        }
    }
    pub fn on_level_updates(&mut self, change: LevelChange) {
        let update_id = change.update_id;
        if self.first_update_id.is_none() {
            self.first_update_id = Some(update_id);
        }
        self.last_update_id = Some(update_id);

        for update in change.level_updates {
            match update.side {
                Side::Bid => self.bids.insert(update.price, update.new_qty),
                Side::Ask => self.asks.insert(update.price, update.new_qty),
            };
        }
    }

    pub fn flush(&mut self) -> Option<OrderBookMessage> {
        let last_id = self.last_update_id?;
        let first_id = self.first_update_id?;

        let bids_map = std::mem::take(&mut self.bids);
        let asks_map = std::mem::take(&mut self.asks);

        let bids = bids_map.into_iter().collect();
        let asks = asks_map.into_iter().collect();

        let msg = OrderBookMessage::Delta {
            bids,
            asks,
            start_id: first_id,
            end_id: last_id,
        };

        self.first_update_id = None;
        self.last_update_id = None;

        Some(msg)
    }

    pub fn reset(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.first_update_id = None;
        self.last_update_id = None;
    }
}

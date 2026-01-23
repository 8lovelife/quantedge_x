use std::collections::BTreeMap;

use crate::{
    domain::order::Side,
    matcher::domain::{price_ticks::PriceTicks, qty_lots::QtyLots},
    models::{level_update::LevelUpdate, order_book_message::OrderBookMessage},
};

pub struct DepthAggregator {
    top_n: usize,

    bids: BTreeMap<PriceTicks, QtyLots>,
    asks: BTreeMap<PriceTicks, QtyLots>,

    first_update_id: Option<u64>,
    last_update_id: Option<u64>,
}

impl DepthAggregator {
    pub fn new(top_n: usize) -> Self {
        Self {
            top_n,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            first_update_id: None,
            last_update_id: None,
        }
    }
    pub fn ingest(&mut self, updates: Vec<LevelUpdate>, first: u64, last: u64) {
        self.first_update_id.get_or_insert(first);
        self.last_update_id = Some(last);

        for u in updates {
            let book = match u.side {
                Side::Bid => &mut self.bids,
                Side::Ask => &mut self.asks,
            };

            match u.new_qty {
                Some(qty) => {
                    book.insert(u.price, qty);
                }
                None => {
                    book.remove(&u.price);
                }
            }
        }
    }

    pub fn snapshot(&mut self) -> Option<OrderBookMessage> {
        let last = self.last_update_id?;

        let bids: Vec<_> = self
            .bids
            .iter()
            .rev()
            .take(self.top_n)
            .map(|(p, q)| (*p, *q))
            .collect();

        let asks: Vec<_> = self
            .asks
            .iter()
            .take(self.top_n)
            .map(|(p, q)| (*p, *q))
            .collect();

        self.bids.clear();
        self.asks.clear();
        self.first_update_id = None;
        self.last_update_id = None;

        Some(OrderBookMessage::Snapshot {
            bids,
            asks,
            last_update_id: last,
        })
    }
}

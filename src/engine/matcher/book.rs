use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PriceTicks(pub i64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct QtyLots(pub i64);

#[derive(Debug, Clone, Copy)]
pub struct Scales {
    pub tick_size: i64,
    pub lot_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: u64,
    pub side: OrderSide,
    pub px: PriceTicks,
    pub qty: QtyLots,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderEvent {
    New(Order),
    Cancel(u64),
}

pub struct OrderBook {
    pub bids: BTreeMap<PriceTicks, Vec<Order>>,
    pub asks: BTreeMap<PriceTicks, Vec<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        let book = match order.side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        };
        book.entry(order.px).or_default().push(order);
    }
}

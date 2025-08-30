use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::matcher::{
    domain::{fill::Fill, order::Order, qty_lots::QtyLots},
    policy::price_level::{fifo::FifoPriceLevel, price_level::PriceLevelPolicy},
};

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct MakerPriceLevel {
    pub inner: FifoPriceLevel,
}

impl MakerPriceLevel {
    pub fn new() -> Self {
        Self {
            inner: FifoPriceLevel::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.total.0 == 0
    }
}

impl PriceLevelPolicy for MakerPriceLevel {
    fn add(&mut self, o: Order) -> anyhow::Result<()> {
        self.inner.add(o)
    }

    fn cancel(&mut self, id: u64) -> anyhow::Result<bool> {
        self.inner.cancel(id)
    }

    fn total(&self) -> anyhow::Result<QtyLots> {
        self.inner.total()
    }

    fn allocate(&mut self, want: QtyLots) -> (Vec<Fill>, QtyLots) {
        self.inner.allocate(want)
    }
}

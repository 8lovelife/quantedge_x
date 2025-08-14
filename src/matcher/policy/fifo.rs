use std::collections::VecDeque;

use anyhow::Ok;
use serde::{Deserialize, Serialize};

use crate::matcher::{
    domain::{fill::Fill, order::Order, qty_lots::QtyLots},
    policy::price_level::PriceLevelPolicy,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct FifoPriceLevel {
    pub total: QtyLots,
    pub orders: VecDeque<Order>,
}

impl PriceLevelPolicy for FifoPriceLevel {
    fn add(&mut self, o: Order) -> anyhow::Result<()> {
        self.total += o.qty;
        self.orders.push_back(o);
        Ok(())
    }

    fn cancel(&mut self, id: u64) -> anyhow::Result<bool> {
        if let Some(pos) = self.orders.iter().position(|x| x.id == id) {
            self.orders.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn total(&self) -> anyhow::Result<QtyLots> {
        todo!()
    }

    fn allocate(&mut self, want: QtyLots) -> Vec<Fill> {
        todo!()
    }
}

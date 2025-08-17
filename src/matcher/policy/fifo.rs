use std::collections::VecDeque;

use anyhow::Ok;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::matcher::{
    domain::{fill::Fill, order::Order, qty_lots::QtyLots},
    policy::price_level::PriceLevelPolicy,
};

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
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
            self.total -= self.orders[pos].qty;
            self.orders.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn total(&self) -> anyhow::Result<QtyLots> {
        Ok(self.total)
    }

    fn allocate(&mut self, mut want: QtyLots) -> (Vec<Fill>, QtyLots) {
        let mut out = Vec::new();
        let mut filled = QtyLots(0);
        while want.0 > 0 {
            if let Some(front) = self.orders.front_mut() {
                let take = QtyLots(front.qty.0.min(want.0));
                if take.0 == 0 {
                    break;
                }
                front.qty -= take;
                self.total -= take;
                want -= take;
                filled += take;
                out.push(Fill {
                    order_id: front.id,
                    qty: take,
                });
                if front.qty.0 == 0 {
                    self.orders.pop_front();
                }
            } else {
                break;
            }
        }
        (out, filled)
    }
}

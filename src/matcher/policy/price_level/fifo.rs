use std::collections::VecDeque;

use anyhow::Ok;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::matcher::{
    domain::{allocation_result::AllocationResult, fill::Fill, order::Order, qty_lots::QtyLots},
    policy::price_level::price_level::PriceLevelPolicy,
};

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct FifoPriceLevel {
    pub total: QtyLots,
    pub orders: VecDeque<Order>,
}

impl FifoPriceLevel {
    pub fn new() -> Self {
        Self {
            total: QtyLots(0),
            orders: VecDeque::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.total.0 == 0
    }
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

    fn allocate(&mut self, mut want: QtyLots) -> anyhow::Result<AllocationResult> {
        let mut out = Vec::new();
        let mut done_ids = Vec::new();
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
                    price: front.px,
                });
                if front.qty.0 == 0 {
                    done_ids.push(front.id);
                    self.orders.pop_front();
                }
            } else {
                break;
            }
        }
        Result::Ok(AllocationResult::new(out, filled, done_ids))
    }
}

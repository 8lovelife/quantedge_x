use crate::{
    domain::order::Side,
    matcher::domain::{price_ticks::PriceTicks, qty_lots::QtyLots},
};

#[derive(Debug)]
pub struct LevelChange {
    pub update_id: u64,
    pub level_updates: Vec<LevelUpdate>,
}

impl LevelChange {
    pub fn new(update_id: u64, level_updates: Vec<LevelUpdate>) -> LevelChange {
        LevelChange {
            update_id,
            level_updates,
        }
    }
}

#[derive(Debug)]
pub struct LevelUpdate {
    pub side: Side,
    pub price: PriceTicks,
    pub new_qty: Option<QtyLots>,
}

impl LevelUpdate {
    pub fn new(side: Side, price: PriceTicks, new_qty: Option<QtyLots>) -> LevelUpdate {
        LevelUpdate {
            side,
            price,
            new_qty,
        }
    }
}

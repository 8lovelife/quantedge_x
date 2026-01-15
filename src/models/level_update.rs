use crate::{
    domain::order::Side,
    matcher::domain::{price_ticks::PriceTicks, qty_lots::QtyLots},
};

#[derive(Debug)]
pub struct LevelUpdate {
    pub side: Side,
    pub price: PriceTicks,
    pub new_qty: QtyLots,
}

impl LevelUpdate {
    pub fn new(side: Side, price: PriceTicks, new_qty: QtyLots) -> LevelUpdate {
        LevelUpdate {
            side,
            price,
            new_qty,
        }
    }
}

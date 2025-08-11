use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::{price_ticks::PriceTicks, qty_lots::QtyLots, scales::Scales};

#[derive(Debug, Clone, Serialize, Deserialize, Decode, Encode)]
pub struct Order {
    pub id: u64,
    pub side: OrderSide,
    pub px: PriceTicks,
    pub qty: QtyLots,
}

impl Order {
    pub fn new(
        id: u64,
        side: OrderSide,
        px: f64,
        qty: f64,
        scales: &Scales,
    ) -> Result<Self, String> {
        Ok(Self {
            id,
            side,
            px: scales.to_ticks_strict(px)?,
            qty: scales.to_lots_strict(qty)?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderEvent {
    New(Order),
    Cancel(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, Decode, Encode)]
pub enum OrderSide {
    Buy,
    Sell,
}

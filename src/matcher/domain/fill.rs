use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::matcher::domain::{price_ticks::PriceTicks, qty_lots::QtyLots};

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Fill {
    pub order_id: u64,
    pub qty: QtyLots,
    pub price: PriceTicks,
}

use crate::matcher::domain::{fill::Fill, qty_lots::QtyLots};
pub enum SweepResult {
    None {
        want: QtyLots,
    },
    Partial {
        fills: Vec<Fill>,
        filled: QtyLots,
        leftover: QtyLots,
        completed_order_ids: Vec<u64>,
    },
    Full {
        fills: Vec<Fill>,
        filled: QtyLots,
        completed_order_ids: Vec<u64>,
    },
}

impl SweepResult {
    pub fn build(
        fills: Vec<Fill>,
        filled: QtyLots,
        want: QtyLots,
        completed_order_ids: Vec<u64>,
    ) -> Self {
        if filled.is_zero() {
            SweepResult::None { want }
        } else if filled == want {
            SweepResult::Full {
                fills,
                filled,
                completed_order_ids,
            }
        } else {
            SweepResult::Partial {
                fills,
                filled,
                leftover: want - filled,
                completed_order_ids,
            }
        }
    }
}

use crate::matcher::domain::{fill::Fill, qty_lots::QtyLots};

pub struct AllocationResult {
    pub fills: Vec<Fill>,
    pub filled: QtyLots,
    pub completed_ids: Vec<u64>,
}

impl AllocationResult {
    pub fn new(fills: Vec<Fill>, filled: QtyLots, completed_ids: Vec<u64>) -> Self {
        AllocationResult {
            fills,
            filled,
            completed_ids,
        }
    }
}

use crate::matcher::domain::{fill::Fill, qty_lots::QtyLots};

pub struct AllocationResult {
    pub fills: Vec<Fill>,
    pub filled: QtyLots,
    pub completed_ids: Vec<u64>,
}

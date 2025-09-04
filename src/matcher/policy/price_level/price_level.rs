use crate::matcher::domain::{
    allocation_result::AllocationResult, order::Order, qty_lots::QtyLots,
};

pub trait PriceLevelPolicy {
    fn add(&mut self, o: Order) -> anyhow::Result<()>;
    fn cancel(&mut self, id: u64) -> anyhow::Result<bool>;
    fn total(&self) -> anyhow::Result<QtyLots>;
    fn allocate(&mut self, want: QtyLots) -> anyhow::Result<AllocationResult>;
}

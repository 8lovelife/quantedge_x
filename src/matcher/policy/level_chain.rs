use crate::matcher::{
    domain::{fill::Fill, order::Order, qty_lots::QtyLots},
    policy::price_level::PriceLevelPolicy,
};

pub struct StaticLevelChain<A, B> {
    a: A,
    b: B,
}

impl<A, B> StaticLevelChain<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A, B> PriceLevelPolicy for StaticLevelChain<A, B>
where
    A: PriceLevelPolicy,
    B: PriceLevelPolicy,
{
    fn add(&mut self, o: Order) -> anyhow::Result<()> {
        self.a.add(o)
    }

    fn cancel(&mut self, id: u64) -> anyhow::Result<bool> {
        self.a.cancel(id)
    }

    fn total(&self) -> anyhow::Result<QtyLots> {
        self.a.total()
    }

    fn allocate(&mut self, want: QtyLots) -> (Vec<Fill>, QtyLots) {
        self.a.allocate(want)
    }
}

pub struct PriceLevelChain {
    pub stages: Vec<Box<dyn PriceLevelPolicy>>,
}

impl PriceLevelChain {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn push<L: PriceLevelPolicy + 'static>(mut self, level: L) -> Self {
        self.stages.push(Box::new(level));
        self
    }
}

impl PriceLevelPolicy for PriceLevelChain {
    fn add(&mut self, o: Order) -> anyhow::Result<()> {
        todo!()
    }

    fn cancel(&mut self, id: u64) -> anyhow::Result<bool> {
        todo!()
    }

    fn total(&self) -> anyhow::Result<QtyLots> {
        todo!()
    }

    fn allocate(&mut self, want: QtyLots) -> (Vec<Fill>, QtyLots) {
        todo!()
    }
}

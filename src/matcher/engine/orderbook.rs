use std::collections::{BTreeMap, HashMap};

use anyhow::Ok;

use crate::matcher::{
    domain::{
        order::{Order, OrderSide},
        price_ticks::PriceTicks,
    },
    policy::price_level::PriceLevelPolicy,
};

pub struct OrderBook<L, F>
where
    L: PriceLevelPolicy,
    F: Fn() -> L + Clone,
{
    bids: BTreeMap<PriceTicks, L>,
    asks: BTreeMap<PriceTicks, L>,
    new_level: F,
    id_index: HashMap<u64, (OrderSide, PriceTicks)>,
}

impl<L, F> OrderBook<L, F>
where
    L: PriceLevelPolicy,
    F: Fn() -> L + Clone,
{
    pub fn new(factory: F) -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            new_level: factory,
            id_index: HashMap::new(),
        }
    }

    pub fn add_order(&mut self, o: Order) -> anyhow::Result<()> {
        let id = o.id;
        let side = o.side;
        let px = o.px;
        let sid_map = match o.side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        };

        let factory = self.new_level.clone();
        let lvl = sid_map.entry(o.px).or_insert_with(|| factory());
        self.id_index.insert(id, (side, px));
        lvl.add(o)
    }

    fn cancel(&mut self, id: u64) -> anyhow::Result<bool> {
        let Some((side, px)) = self.id_index.remove(&id) else {
            return Ok(false);
        };
        let side_map = match side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        };
        if let Some(level) = side_map.get_mut(&px) {
            let removed = level.cancel(id)?;
            if removed && level.total()?.0 == 0 {
                side_map.remove(&px);
            }
            return Ok(removed);
        }
        Ok(false)
    }
}

use std::{
    collections::{BTreeMap, HashMap},
    i64,
};

use anyhow::Ok;

use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{
        fill::Fill,
        order::{Order, OrderSide},
        price_ticks::PriceTicks,
        qty_lots::QtyLots,
    },
    policy::price_level::price_level::PriceLevelPolicy,
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

impl<L, F> OrderBookOps for OrderBook<L, F>
where
    L: PriceLevelPolicy,
    F: Fn() -> L + Clone,
{
    fn liquidity_up_to_ask(&self, limit: PriceTicks, want: QtyLots) -> anyhow::Result<QtyLots> {
        let mut acc = QtyLots(0);
        for (_px, lvl) in self.asks.range(PriceTicks(i64::MIN)..=limit) {
            let qtys = lvl.total()?;
            acc += qtys;
            if acc >= want {
                return Result::Ok(acc);
            }
        }
        Result::Ok(acc)
    }

    fn sweep_asks_up_to(
        &mut self,
        limit: PriceTicks,
        mut want: QtyLots,
    ) -> anyhow::Result<(Vec<Fill>, QtyLots)> {
        let init_want = want;
        let mut clear_pxs = Vec::new();
        let mut fills = Vec::new();
        for (&px, lvl) in self.asks.range_mut(PriceTicks(i64::MIN)..limit) {
            let (mut part, got) = lvl.allocate(want);
            fills.append(&mut part);
            want -= got;
            if lvl.total()?.0 == 0 {
                clear_pxs.push(px);
            }
            if want.0 <= 0 {
                break;
            }
        }
        for px in clear_pxs {
            self.asks.remove(&px);
        }

        let filled = QtyLots(fills.iter().map(|f| f.qty.0).sum());
        debug_assert_eq!(filled, init_want - want);

        Result::Ok((fills, init_want - want))
    }

    fn liquidity_down_to_bid(&self, limit: PriceTicks, want: QtyLots) -> anyhow::Result<QtyLots> {
        let mut acc = QtyLots(0);
        for (_px, lvl) in self.bids.range(limit..).rev() {
            let qtys = lvl.total()?;
            acc += qtys;
            if acc >= want {
                return Result::Ok(acc);
            }
        }
        Result::Ok(acc)
    }

    fn sweep_bids_down_to(
        &mut self,
        limit: PriceTicks,
        mut want: QtyLots,
    ) -> anyhow::Result<(Vec<Fill>, QtyLots)> {
        let init_want = want;
        let mut clear_pxs = Vec::new();
        let mut fills = Vec::new();
        for (&px, lvl) in self.bids.range_mut(limit..).rev() {
            let (mut part, got) = lvl.allocate(want);
            fills.append(&mut part);
            want -= got;
            if lvl.total()?.0 == 0 {
                clear_pxs.push(px);
            }
            if want.0 <= 0 {
                break;
            }
        }
        for px in clear_pxs {
            self.bids.remove(&px);
        }
        let filled = QtyLots(fills.iter().map(|f| f.qty.0).sum());
        debug_assert_eq!(filled, init_want - want);
        Result::Ok((fills, init_want - want))
    }

    fn sweep_market_buy(&mut self, mut want: QtyLots) -> anyhow::Result<(Vec<Fill>, QtyLots)> {
        let init_want = want;
        let mut clear_pxs = Vec::new();
        let mut fills = Vec::new();
        for (&px, lvl) in self.asks.iter_mut() {
            let (mut part, got) = lvl.allocate(want);
            fills.append(&mut part);
            want -= got;
            if lvl.total()?.0 == 0 {
                clear_pxs.push(px);
            }
            if want.0 <= 0 {
                break;
            }
        }

        for px in clear_pxs {
            self.bids.remove(&px);
        }

        let filled = QtyLots(fills.iter().map(|f| f.qty.0).sum());
        debug_assert_eq!(filled, init_want - want);
        Result::Ok((fills, init_want - want))
    }
    // fn sweep_market_sell(&mut self, mut want: QtyLots) -> anyhow::Result<(Vec<Fill>, QtyLots)> {
    //     let init_want = want;
    //     let mut clear_pxs = Vec::new();
    //     let mut fills = Vec::new();
    //     for (&px, lvl) in self.bids.iter_mut() {
    //         let (mut part, got) = lvl.allocate(want);
    //         fills.append(&mut part);
    //         want -= got;
    //         if lvl.total()?.0 == 0 {
    //             clear_pxs.push(px);
    //         }
    //         if want.0 <= 0 {
    //             break;
    //         }
    //     }
    //     for px in clear_pxs {
    //         self.bids.remove(&px);
    //     }
    //     let filled = QtyLots(fills.iter().map(|f| f.qty.0).sum());
    //     debug_assert_eq!(filled, init_want - want);
    //     Result::Ok((fills, init_want - want))
    // }
}

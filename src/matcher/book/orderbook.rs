use std::{
    collections::{BTreeMap, HashMap, HashSet},
    i64,
};

use anyhow::Ok;
use bincode::{Decode, Encode};
use log::Level;

use crate::{
    domain::order::Side,
    matcher::{
        book::{book_manager::OrderBookData, book_ops::OrderBookOps},
        domain::{
            order::{Order, OrderSide},
            price_ticks::PriceTicks,
            qty_lots::QtyLots,
            sweep_result::SweepResult,
        },
        policy::price_level::price_level::PriceLevelPolicy,
    },
    models::level_update::{LevelChange, LevelUpdate},
};

pub struct OrderBook<L, F>
where
    L: PriceLevelPolicy + Encode + Decode<()>,
    F: Fn() -> L + Clone,
{
    bids: BTreeMap<PriceTicks, L>,
    asks: BTreeMap<PriceTicks, L>,
    new_level: F,
    id_index: HashMap<u64, (OrderSide, PriceTicks)>,
    last_update_id: u64,
}

impl<L, F> OrderBook<L, F>
where
    L: PriceLevelPolicy + Encode + Decode<()>,
    F: Fn() -> L + Clone,
{
    pub fn new(factory: F) -> Self {
        Self {
            bids: BTreeMap::<PriceTicks, L>::new(),
            asks: BTreeMap::<PriceTicks, L>::new(),
            new_level: factory,
            id_index: HashMap::new(),
            last_update_id: 0,
        }
    }

    pub fn build(
        bids: BTreeMap<PriceTicks, L>,
        asks: BTreeMap<PriceTicks, L>,
        id_index: HashMap<u64, (OrderSide, PriceTicks)>,
        factory: F,
        last_update_id: u64,
    ) -> Self {
        Self {
            bids,
            asks,
            new_level: factory,
            id_index,
            last_update_id,
        }
    }

    pub fn snapshot(&self) -> OrderBookData<L> {
        OrderBookData {
            bids: self.bids().clone(),
            asks: self.asks().clone(),
            id_index: self.id_index().clone(),
            last_update_id: self.last_update_id,
        }
    }

    pub fn size(&self) -> usize {
        self.id_index.len()
    }

    pub fn bids(&self) -> &BTreeMap<PriceTicks, L> {
        &self.bids
    }

    pub fn asks(&self) -> &BTreeMap<PriceTicks, L> {
        &self.asks
    }

    pub fn id_index(&self) -> &HashMap<u64, (OrderSide, PriceTicks)> {
        &self.id_index
    }

    pub fn last_update_id(&self) -> u64 {
        self.last_update_id
    }

    pub fn increase_update_id(&mut self) {
        self.last_update_id += 1
    }

    pub fn level_qty(&self, side: Side, price: PriceTicks) -> anyhow::Result<Option<QtyLots>> {
        if let Some(level) = match side {
            Side::Ask => self.asks.get(&price),
            Side::Bid => self.bids.get(&price),
        } {
            let qty = level.total()?;
            Ok(Some(qty))
        } else {
            Ok(None)
        }
    }
}

impl<L, F> OrderBookOps for OrderBook<L, F>
where
    L: PriceLevelPolicy + Encode + Decode<()>,
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
    ) -> anyhow::Result<SweepResult> {
        let init_want = want;
        let mut clear_pxs = Vec::new();
        let mut fills = Vec::new();
        let mut completed_order_ids = Vec::new();
        for (&px, lvl) in self.asks.range_mut(PriceTicks(i64::MIN)..limit) {
            let mut allocation_result = lvl.allocate(want)?;
            let part = allocation_result.fills.as_mut();
            let got = allocation_result.filled;
            let done_ids = allocation_result.completed_ids.as_mut();
            fills.append(part);
            completed_order_ids.append(done_ids);
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

        self.increase_update_id();
        Result::Ok(SweepResult::build(
            fills,
            filled,
            init_want,
            completed_order_ids,
        ))
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
    ) -> anyhow::Result<SweepResult> {
        let init_want = want;
        let mut clear_pxs = Vec::new();
        let mut fills = Vec::new();
        let mut completed_order_ids = Vec::new();
        for (&px, lvl) in self.bids.range_mut(limit..).rev() {
            let mut allocation_result = lvl.allocate(want)?;
            let part = allocation_result.fills.as_mut();
            let got = allocation_result.filled;
            let done_ids = allocation_result.completed_ids.as_mut();
            fills.append(part);
            completed_order_ids.append(done_ids);
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

        self.increase_update_id();
        Result::Ok(SweepResult::build(
            fills,
            filled,
            init_want,
            completed_order_ids,
        ))
    }

    fn sweep_market_buy(&mut self, mut want: QtyLots) -> anyhow::Result<SweepResult> {
        let init_want = want;
        let mut clear_pxs = Vec::new();
        let mut fills = Vec::new();
        let mut completed_order_ids = Vec::new();
        for (&px, lvl) in self.asks.iter_mut() {
            let mut allocation_result = lvl.allocate(want)?;
            let part = allocation_result.fills.as_mut();
            let got = allocation_result.filled;
            let done_ids = allocation_result.completed_ids.as_mut();
            fills.append(part);
            completed_order_ids.append(done_ids);
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
        Result::Ok(SweepResult::build(
            fills,
            filled,
            init_want,
            completed_order_ids,
        ))
    }

    fn add_order(&mut self, o: Order) -> anyhow::Result<()> {
        let id = o.id;
        let side = o.side;
        let px = o.px;
        let sid_map = match o.side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        };
        let lvl = sid_map.entry(o.px).or_insert_with(|| {
            let factory = self.new_level.clone();
            factory()
        });
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

    fn info(&self) -> anyhow::Result<String> {
        let mut out = String::new();

        out.push_str("=== OrderBook Snapshot ===\n");

        out.push_str("-- Bids --\n");
        for (price, level) in self.bids.iter().rev() {
            let count = level.total()?;
            out.push_str(&format!("{{ price: {}, count: {} }}\n", price.0, count));
        }

        out.push_str("-- Asks --\n");
        for (price, level) in self.asks.iter() {
            let count = level.total()?;
            out.push_str(&format!("{{ price: {}, count: {} }}\n", price.0, count));
        }

        out.push_str(&format!("last update id {}", self.last_update_id));

        Ok(out)
    }

    type Level = L;

    type Factory = F;

    fn get_orderbook(&self) -> anyhow::Result<&OrderBook<Self::Level, Self::Factory>> {
        Ok(self)
    }

    fn level_update(&self, prices: HashMap<Side, Vec<PriceTicks>>) -> anyhow::Result<LevelChange> {
        let mut seen = HashSet::new();
        let mut level_updates = Vec::new();
        for (side, price_list) in prices {
            for price in price_list {
                if !seen.insert((side, price)) {
                    continue;
                }
                let qty = self.level_qty(side, price)?;
                level_updates.push(LevelUpdate::new(side, price, qty));
            }
        }
        let update_id = self.last_update_id;
        let level_change = LevelChange::new(update_id, level_updates);
        Ok(level_change)
    }
}

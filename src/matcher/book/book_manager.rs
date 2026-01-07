use anyhow::Result;
use bincode::{Decode, Encode, config::standard, encode_into_std_write};
use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
};

use crate::matcher::{
    book::orderbook::OrderBook,
    domain::{order::OrderSide, price_ticks::PriceTicks},
    policy::price_level::price_level::PriceLevelPolicy,
    storage::Storage,
};

pub struct OrderBookManager<L, F, S>
where
    L: PriceLevelPolicy + Encode + Decode<()>,
    F: Fn() -> L + Clone,
{
    storage: S,
    new_level: F,
    _phantom: PhantomData<L>,
}

impl<L, F, S> OrderBookManager<L, F, S>
where
    L: PriceLevelPolicy + Encode + Decode<()>,
    F: Fn() -> L + Clone,
    S: Storage,
{
    pub fn new(storage: S, new_level: F) -> Self {
        Self {
            storage,
            new_level,
            _phantom: PhantomData,
        }
    }

    pub fn save<LL, FF>(&self, book: &OrderBook<LL, FF>) -> Result<()>
    where
        LL: PriceLevelPolicy + Encode + Decode<()>,
        FF: Fn() -> LL + Clone,
    {
        let data = book.snapshot();
        let mut buf = Vec::new();
        encode_into_std_write(&data, &mut buf, standard())?;
        self.storage.save_snapshot(&buf)
    }

    pub fn load(&self) -> Result<OrderBook<L, F>> {
        let bytes = self
            .storage
            .load_latest_snapshot()?
            .ok_or_else(|| anyhow::anyhow!("No snapshot found"))?;

        let (data, _): (OrderBookData<L>, _) =
            bincode::decode_from_slice(&bytes, bincode::config::standard())?;

        Ok(OrderBook::build(
            data.bids,
            data.asks,
            data.id_index,
            self.new_level.clone(),
            data.last_update_id,
        ))
    }
}

#[derive(Encode, Decode)]
pub struct OrderBookData<L>
where
    L: PriceLevelPolicy + Encode + Decode<()>,
{
    pub bids: BTreeMap<PriceTicks, L>,
    pub asks: BTreeMap<PriceTicks, L>,
    pub id_index: HashMap<u64, (OrderSide, PriceTicks)>,
    pub last_update_id: u64,
}

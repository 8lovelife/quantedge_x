use crate::matcher::domain::{
    order::Order, price_ticks::PriceTicks, qty_lots::QtyLots, sweep_result::SweepResult,
};

pub trait OrderBookOps {
    fn liquidity_up_to_ask(&self, limit: PriceTicks, want: QtyLots) -> anyhow::Result<QtyLots>;
    fn sweep_asks_up_to(&mut self, limit: PriceTicks, want: QtyLots)
    -> anyhow::Result<SweepResult>;
    fn liquidity_down_to_bid(&self, limit: PriceTicks, want: QtyLots) -> anyhow::Result<QtyLots>;
    fn sweep_bids_down_to(
        &mut self,
        limit: PriceTicks,
        want: QtyLots,
    ) -> anyhow::Result<SweepResult>;

    fn sweep_market_buy(&mut self, want: QtyLots) -> anyhow::Result<SweepResult>;
    // fn sweep_market_sell(&mut self, want: QtyLots) -> anyhow::Result<(Vec<Fill>, QtyLots)>;

    fn add_order(&mut self, o: Order) -> anyhow::Result<()>;

    fn cancel(&mut self, id: u64) -> anyhow::Result<bool>;

    fn info(&self) -> anyhow::Result<String>;
}

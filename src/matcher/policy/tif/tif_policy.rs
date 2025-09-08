use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{price_ticks::PriceTicks, qty_lots::QtyLots, tif_policy_result::TifPolicyResult},
};

pub trait TifPolicy {
    fn execute_buy<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifPolicyResult>;

    fn execute_sell<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifPolicyResult>;
}

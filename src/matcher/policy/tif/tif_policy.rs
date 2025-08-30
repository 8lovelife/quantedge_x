use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{price_ticks::PriceTicks, qty_lots::QtyLots, tif_result::TifResult},
};

pub trait TifPolicy {
    fn execute_buy<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifResult>;

    fn execute_sell<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifResult>;
}

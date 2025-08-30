use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{price_ticks::PriceTicks, qty_lots::QtyLots, tif_result::TifResult},
    policy::tif::tif_policy::TifPolicy,
};

pub struct IocPolicy;

impl TifPolicy for IocPolicy {
    fn execute_buy<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifResult> {
        let limit = limit.expect("IOC buy must have a limit price");
        let (fills, filled) = book.sweep_asks_up_to(limit, want)?;
        let result = TifResult::accepted_with_cancel(fills, filled, want - filled);
        Result::Ok(result)
    }

    fn execute_sell<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifResult> {
        let limit = limit.expect("IOC sell must have a limit price");
        let (fills, filled) = book.sweep_bids_down_to(limit, want)?;
        let result = TifResult::accepted_with_cancel(fills, filled, want - filled);
        Result::Ok(result)
    }
}

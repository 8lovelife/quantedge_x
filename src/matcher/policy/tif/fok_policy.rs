use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{price_ticks::PriceTicks, qty_lots::QtyLots, tif_result::TifResult},
    policy::tif::tif_policy::TifPolicy,
};

pub struct FokPolicy;

impl TifPolicy for FokPolicy {
    fn execute_buy<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifResult> {
        let limit = limit.expect("FOK buy must have a limit price");
        if book.liquidity_up_to_ask(limit, want)? < want {
            return Result::Ok(TifResult::rejected_with_cancel(want));
        }
        let sweep_result = book.sweep_asks_up_to(limit, want)?;
        Result::Ok(TifResult::accepted(sweep_result.fills, sweep_result.filled))
    }

    fn execute_sell<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifResult> {
        let limit = limit.expect("FOK sell must have a limit price");
        if book.liquidity_down_to_bid(limit, want)? < want {
            return Result::Ok(TifResult::rejected_with_cancel(want));
        }
        let sweep_result = book.sweep_bids_down_to(limit, want)?;
        Result::Ok(TifResult::accepted(sweep_result.fills, sweep_result.filled))
    }
}

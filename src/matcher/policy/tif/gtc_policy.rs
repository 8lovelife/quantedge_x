use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{order::OrderSide, price_ticks::PriceTicks, qty_lots::QtyLots, tif_result::TifResult},
    policy::tif::tif_policy::TifPolicy,
};

pub struct GtcPolicy;

impl TifPolicy for GtcPolicy {
    fn execute_buy<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifResult> {
        let limit = limit.expect("GTC buy must have a limit price");
        let sweep_result = book.sweep_asks_up_to(limit, want)?;
        let mut result = TifResult::accepted(sweep_result.fills, sweep_result.filled);
        let rest_qty = sweep_result.leftover;
        if rest_qty.0 > 0 {
            result.with_rest(OrderSide::Buy, limit, rest_qty, None);
        }
        Result::Ok(result)
    }

    fn execute_sell<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifResult> {
        let limit = limit.expect("GTC sell must have a limit price");
        let sweep_result = book.sweep_bids_down_to(limit, want)?;
        let mut result = TifResult::accepted(sweep_result.fills, sweep_result.filled);
        let rest_qty = sweep_result.leftover;
        if rest_qty.0 > 0 {
            result.with_rest(OrderSide::Sell, limit, rest_qty, None);
        }
        Result::Ok(result)
    }
}

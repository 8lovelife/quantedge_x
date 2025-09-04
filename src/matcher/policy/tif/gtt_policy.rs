use chrono::{DateTime, Utc};

use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{order::OrderSide, price_ticks::PriceTicks, qty_lots::QtyLots, tif_result::TifResult},
    policy::tif::tif_policy::TifPolicy,
};

pub struct GttPolicy {
    pub expires_at: DateTime<Utc>,
}

impl TifPolicy for GttPolicy {
    fn execute_buy<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifResult> {
        let limit = limit.expect("GTT buy must have a limit price");
        let sweep_result = book.sweep_asks_up_to(limit, want)?;
        let mut result = TifResult::accepted(sweep_result.fills, sweep_result.filled);
        let rest_qty = sweep_result.leftover;
        if rest_qty.0 > 0 {
            result.with_rest(
                OrderSide::Buy,
                limit,
                sweep_result.leftover,
                Some(self.expires_at),
            );
        }
        Result::Ok(result)
    }

    fn execute_sell<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifResult> {
        let limit = limit.expect("GTT sell must have a limit price");
        let sweep_result = book.sweep_bids_down_to(limit, want)?;
        let mut result = TifResult::accepted(sweep_result.fills, sweep_result.filled);
        let rest_qty = sweep_result.leftover;
        if rest_qty.0 > 0 {
            result.with_rest(OrderSide::Sell, limit, rest_qty, Some(self.expires_at));
        }
        Result::Ok(result)
    }
}

use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{
        price_ticks::PriceTicks, qty_lots::QtyLots, sweep_result::SweepResult,
        tif_policy_result::TifPolicyResult, tif_result::TifResult,
    },
    policy::tif::tif_policy::TifPolicy,
};

pub struct FokPolicy;

impl TifPolicy for FokPolicy {
    fn execute_buy<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifPolicyResult> {
        let limit = limit.expect("FOK buy must have a limit price");
        if book.liquidity_up_to_ask(limit, want)? < want {
            return Result::Ok(TifPolicyResult::rejected(want));
        }
        match book.sweep_asks_up_to(limit, want)? {
            SweepResult::Full {
                fills,
                filled,
                completed_order_ids,
            } => Ok(TifPolicyResult::accepted(
                fills,
                filled,
                Some(completed_order_ids),
            )),
            SweepResult::Partial { .. } | SweepResult::None { .. } => Err(anyhow::anyhow!(
                "FOK strategy: unexpected partial or none fill after liquidity check"
            )),
        }
    }

    fn execute_sell<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifPolicyResult> {
        let limit = limit.expect("FOK sell must have a limit price");
        if book.liquidity_down_to_bid(limit, want)? < want {
            return Result::Ok(TifPolicyResult::rejected(want));
        }
        match book.sweep_bids_down_to(limit, want)? {
            SweepResult::Full {
                fills,
                filled,
                completed_order_ids,
            } => Ok(TifPolicyResult::accepted(
                fills,
                filled,
                Some(completed_order_ids),
            )),
            SweepResult::Partial { .. } | SweepResult::None { .. } => Err(anyhow::anyhow!(
                "FOK strategy: unexpected partial or none fill after liquidity check"
            )),
        }
    }
}

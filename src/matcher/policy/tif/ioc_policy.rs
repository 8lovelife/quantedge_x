use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{
        price_ticks::PriceTicks, qty_lots::QtyLots, reject_reason::RejectReason,
        sweep_result::SweepResult, tif_policy_result::TifPolicyResult, tif_result::TifResult,
    },
    policy::tif::tif_policy::TifPolicy,
};

pub struct IocPolicy;

impl TifPolicy for IocPolicy {
    fn execute_buy<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifPolicyResult> {
        let limit = limit.expect("IOC buy must have a limit price");
        match book.sweep_asks_up_to(limit, want)? {
            SweepResult::None { want } => Ok(TifPolicyResult::rejected(
                want,
                RejectReason::NoMatchingOrder,
            )),

            SweepResult::Partial {
                fills,
                filled,
                leftover,
                completed_order_ids,
            } => Ok(TifPolicyResult::accepted_with_cancel(
                fills,
                filled,
                leftover,
                Some(completed_order_ids),
            )),

            SweepResult::Full {
                fills,
                filled,
                completed_order_ids,
            } => Ok(TifPolicyResult::accepted(
                fills,
                filled,
                Some(completed_order_ids),
            )),
        }
    }

    fn execute_sell<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifPolicyResult> {
        let limit = limit.expect("IOC sell must have a limit price");
        match book.sweep_bids_down_to(limit, want)? {
            SweepResult::None { want } => Ok(TifPolicyResult::rejected(
                want,
                RejectReason::NoMatchingOrder,
            )),

            SweepResult::Partial {
                fills,
                filled,
                leftover,
                completed_order_ids,
            } => Ok(TifPolicyResult::accepted_with_cancel(
                fills,
                filled,
                leftover,
                Some(completed_order_ids),
            )),

            SweepResult::Full {
                fills,
                filled,
                completed_order_ids,
            } => Ok(TifPolicyResult::accepted(
                fills,
                filled,
                Some(completed_order_ids),
            )),
        }
    }
}

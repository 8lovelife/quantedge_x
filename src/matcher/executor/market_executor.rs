use anyhow::Ok;

use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{
        execution_result::ExecutionResult, order::Order, reject_reason::RejectReason,
        sweep_result::SweepResult, tif_policy_result::TifPolicyResult,
    },
    executor::order_executor::OrderTypeExecutor,
};

pub struct MarketExecutor;

impl<T: OrderBookOps> OrderTypeExecutor<T> for MarketExecutor {
    fn execute(&self, order: Order, book: &mut T) -> anyhow::Result<ExecutionResult> {
        let resp = match book.sweep_market_buy(order.qty)? {
            SweepResult::None { want } => {
                TifPolicyResult::rejected(want, RejectReason::NoMatchingOrder)
            }

            SweepResult::Partial {
                fills,
                filled,
                leftover,
                completed_order_ids,
            } => TifPolicyResult::accepted_with_cancel(
                fills,
                filled,
                leftover,
                Some(completed_order_ids),
            ),

            SweepResult::Full {
                fills,
                filled,
                completed_order_ids,
            } => TifPolicyResult::accepted(fills, filled, Some(completed_order_ids)),
        };
        Ok(ExecutionResult::from_tif_result(order, resp))
    }
}

use anyhow::Ok;

use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{
        execution_result::ExecutionResult,
        order::{Order, OrderSide},
        reject_reason::RejectReason,
        sweep_result::SweepResult,
        tif_policy_result::TifPolicyResult,
        tif_result::TifResult,
    },
    executor::order_executor::OrderTypeExecutor,
};

pub struct MarketExecutor;

impl<T: OrderBookOps> OrderTypeExecutor<T> for MarketExecutor {
    fn execute(&self, order: Order, book: &mut T) -> anyhow::Result<ExecutionResult> {
        // match order.side {
        //     OrderSide::Buy => {
        //         let (fills, filled) = book.sweep_market_buy(order.qty)?;
        //         Result::Ok(TifResult::accepted_with_cancel(
        //             fills,
        //             filled,
        //             order.qty - filled,
        //         ))
        //     }
        //     OrderSide::Sell => {
        //         let (fills, filled) = book.sweep_market_sell(order.qty)?;
        //         Result::Ok(TifResult::accepted_with_cancel(
        //             fills,
        //             filled,
        //             order.qty - filled,
        //         ))
        //     }
        // }

        // let sweep_result = book.sweep_market_buy(order.qty)?;

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
        // Result::Ok(TifResult::accepted_with_cancel(
        //     sweep_result.fills,
        //     sweep_result.filled,
        //     sweep_result.leftover,
        // ))

        Ok(ExecutionResult::from_tif_result(order, resp))
    }
}

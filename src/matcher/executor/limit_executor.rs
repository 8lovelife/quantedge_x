use anyhow::Ok;

use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{
        execution_result::ExecutionResult,
        order::{Order, OrderSide},
        tif_policy_result::TifPolicyResult,
    },
    executor::order_executor::OrderTypeExecutor,
    policy::tif::tif_policy::TifPolicy,
};

pub struct LimitExecutor<P: TifPolicy> {
    policy: P,
}

impl<P: TifPolicy> LimitExecutor<P> {
    pub fn new(p: P) -> Self {
        Self { policy: p }
    }
}

impl<P: TifPolicy, T: OrderBookOps> OrderTypeExecutor<T> for LimitExecutor<P> {
    fn execute(&self, order: Order, book: &mut T) -> anyhow::Result<ExecutionResult> {
        let resp = match order.side {
            OrderSide::Buy => self.policy.execute_buy(book, Some(order.px), order.qty)?,
            OrderSide::Sell => self.policy.execute_sell(book, Some(order.px), order.qty)?,
        };

        if let TifPolicyResult::AcceptedAndPlaced { ref rest, .. } = resp {
            let mut rest_order = order.clone();
            rest_order.qty = rest.qty;
            book.add_order(rest_order)?;
        }

        Ok(ExecutionResult::from_tif_result(order, resp))
    }
}

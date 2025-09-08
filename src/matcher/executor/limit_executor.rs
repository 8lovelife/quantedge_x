use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{
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
    fn execute(&self, order: Order, book: &mut T) -> anyhow::Result<TifPolicyResult> {
        let resp = match order.side {
            OrderSide::Buy => self.policy.execute_buy(book, Some(order.px), order.qty)?,
            OrderSide::Sell => self.policy.execute_sell(book, Some(order.px), order.qty)?,
        };

        // if let Some(rest) = resp.rest.as_ref() {
        //     book.insert_resting(rest.clone())?;
        // }

        Result::Ok(resp)
    }
}

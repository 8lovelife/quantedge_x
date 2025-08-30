use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{
        order::{Order, OrderSide},
        tif_result::TifResult,
    },
    executor::order_executor::OrderTypeExecutor,
    policy::tif::tif_policy::TifPolicy,
};

pub struct LimitExecutor<P: TifPolicy> {
    policy: P,
}

impl<P: TifPolicy, T: OrderBookOps> OrderTypeExecutor<T> for LimitExecutor<P> {
    fn execute(&mut self, order: Order, book: &mut T) -> anyhow::Result<TifResult> {
        match order.side {
            OrderSide::Buy => self.policy.execute_buy(book, Some(order.px), order.qty),
            OrderSide::Sell => self.policy.execute_sell(book, Some(order.px), order.qty),
        }
    }
}

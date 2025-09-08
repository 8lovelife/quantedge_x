use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{order::Order, tif_policy_result::TifPolicyResult},
};

pub trait OrderTypeExecutor<T: OrderBookOps> {
    fn execute(&self, order: Order, book: &mut T) -> anyhow::Result<TifPolicyResult>;
}

use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{order::Order, tif_result::TifResult},
};

pub trait OrderTypeExecutor<T: OrderBookOps> {
    fn execute(&self, order: Order, book: &mut T) -> anyhow::Result<TifResult>;
}

use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{execution_result::ExecutionResult, order::Order},
};

pub trait OrderTypeExecutor<T: OrderBookOps> {
    fn execute(&self, order: Order, book: &mut T) -> anyhow::Result<ExecutionResult>;
}

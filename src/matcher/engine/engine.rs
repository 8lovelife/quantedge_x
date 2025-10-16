use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{
        execution_result::ExecutionResult,
        order::{Order, OrderType},
        tif_policy_result::TifPolicyResult,
        tif_result::TifResult,
    },
    executor::{
        limit_executor::LimitExecutor, market_executor::MarketExecutor,
        order_executor::OrderTypeExecutor,
    },
    policy::tif::tif_policy_factory::obtain_tif_policy,
};

pub struct Engine;

impl Engine {
    pub fn execute<T: OrderBookOps>(
        &mut self,
        order: Order,
        book: &mut T,
    ) -> anyhow::Result<ExecutionResult> {
        match order.order_type {
            OrderType::Market => {
                let market_executor = MarketExecutor;
                market_executor.execute(order, book)
            }
            OrderType::Limit => {
                let tif_policy = obtain_tif_policy(order.tif);
                let limit_executor = LimitExecutor::new(tif_policy);
                limit_executor.execute(order, book)
            }
        }
    }
}

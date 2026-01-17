use anyhow::Ok;
use tokio::sync::mpsc;

use crate::{
    matcher::{
        book::book_ops::OrderBookOps,
        domain::{
            execution_result::ExecutionResult,
            order::{Order, OrderType},
        },
        executor::{
            limit_executor::LimitExecutor, market_executor::MarketExecutor,
            order_executor::OrderTypeExecutor,
        },
        policy::tif::tif_policy_factory::obtain_tif_policy,
    },
    models::level_update::LevelChange,
};

#[derive(Clone)]
pub struct Engine {
    tx_delta: mpsc::Sender<LevelChange>,
}

impl Engine {
    pub fn new(tx_delta: mpsc::Sender<LevelChange>) -> Self {
        Self { tx_delta }
    }

    pub async fn publish(&self, level_change: LevelChange) {
        let _ = self.tx_delta.send(level_change).await;
    }

    pub async fn execute<T: OrderBookOps>(
        &mut self,
        order: Order,
        book: &mut T,
    ) -> anyhow::Result<ExecutionResult> {
        let result = match order.order_type {
            OrderType::Market => {
                let market_executor = MarketExecutor;
                market_executor.execute(order, book)
            }
            OrderType::Limit => {
                let tif_policy = obtain_tif_policy(order.tif);
                let limit_executor = LimitExecutor::new(tif_policy);
                limit_executor.execute(order, book)
            }
        }?;
        let prices = result.prices.clone();
        let level_updates = book.level_update(prices)?;
        self.publish(level_updates).await;
        Ok(result)
    }
}

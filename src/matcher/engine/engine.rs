use std::sync::Arc;

use anyhow::Ok;
use tokio::task::JoinHandle;

use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{
        execution_result::ExecutionResult,
        match_output::MatchOutput,
        order::{Order, OrderType},
    },
    engine::{
        engine_event::EngineEvent,
        event_router::{EventRouter, create_router_with_workers},
    },
    executor::{
        limit_executor::LimitExecutor, market_executor::MarketExecutor,
        order_executor::OrderTypeExecutor,
    },
    policy::tif::tif_policy_factory::obtain_tif_policy,
};

type RouteFn = Arc<dyn Fn(EngineEvent) + Send + Sync>;

pub struct Engine {
    router: EventRouter,
}

impl Engine {
    pub fn new(levelchange_handler: RouteFn, trade_tick_handler: RouteFn) -> Self {
        let router = create_router_with_workers(levelchange_handler, trade_tick_handler);
        Self { router }
    }

    pub fn start(&mut self) -> JoinHandle<()> {
        self.router.spawn()
    }

    pub async fn handle(&self, out: MatchOutput) {
        self.router.send_event(out).await;
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
        let events = result.build_trade_event();

        let match_out = MatchOutput::new(level_updates, events);
        self.handle(match_out).await;

        Ok(result)
    }
}

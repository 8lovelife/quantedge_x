use std::{sync::Arc, time::Duration};

use anyhow::Ok;
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
    data::market_data_bus::start_market_data_bus,
    matcher::{
        book::book_ops::OrderBookOps,
        domain::{
            execution_result::ExecutionResult,
            match_output::MatchOutput,
            order::{Order, OrderType},
            trade_batch::TradeBatch,
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
    },
    models::{
        level_update::LevelChange, order_book_message::OrderBookMessage,
        order_book_publisher::OrderBookPublisher,
    },
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

    pub fn start_with_publisher() -> Engine {
        let (change_tx, mut change_rx) = mpsc::channel::<LevelChange>(10000);
        let (ob_out_tx, ob_out_rx) = mpsc::channel::<OrderBookMessage>(100);

        // let (trade_tx, mut trade_rx) = mpsc::channel::<TradeBatch>(5000);

        let (trade_out_tx, trade_out_rx) = mpsc::channel::<TradeBatch>(1000);

        let level_change_handler: RouteFn = {
            let tx = change_tx.clone();
            Arc::new(move |e: EngineEvent| {
                if let EngineEvent::LevelChange(change) = e {
                    let _ = tx.try_send(change);
                }
            })
        };
        let trade_tick_handler = {
            let tx = trade_out_tx.clone();
            let symbol = "AAAA/USDT".to_string();
            let tick_size = 0.1;
            let lot_size = 0.01;

            Arc::new(move |e: EngineEvent| {
                if let EngineEvent::TradeEventResult(trade_result) = e {
                    if let Some(batch) = trade_result.to_trade_batch(&symbol, tick_size, lot_size) {
                        if let Err(err) = tx.try_send(batch) {
                            eprintln!("Trade channel full, dropping batch: {:?}", err);
                        }
                    }
                }
            })
        };

        tokio::spawn(async move {
            let mut publisher = OrderBookPublisher::new(100);
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    Some(change) = change_rx.recv() => {
                        publisher.on_level_change(change);
                    }
                    _ = interval.tick() => {
                        if let Some(msg) = publisher.publish_tick() {
                            if ob_out_tx.send(msg).await.is_err() { break; }
                        }
                    }
                }
            }
        });

        tokio::spawn(async move {
            let market_tx = start_market_data_bus("USDT".to_string(), 1000).await;
            let mut rx = trade_out_rx;
            while let Some(msg) = rx.recv().await {
                for tick in msg.trades {
                    let _ = market_tx.send(tick);
                }
            }
        });

        let engine = Engine::new(level_change_handler, trade_tick_handler);
        engine
    }

    pub fn build_with_publisher() -> (
        Engine,
        mpsc::Receiver<OrderBookMessage>,
        mpsc::Receiver<TradeBatch>,
    ) {
        let (change_tx, mut change_rx) = mpsc::channel::<LevelChange>(10000);
        let (ob_out_tx, ob_out_rx) = mpsc::channel::<OrderBookMessage>(100);

        // let (trade_tx, mut trade_rx) = mpsc::channel::<TradeBatch>(5000);

        let (trade_out_tx, trade_out_rx) = mpsc::channel::<TradeBatch>(1000);

        let level_change_handler: RouteFn = {
            let tx = change_tx.clone();
            Arc::new(move |e: EngineEvent| {
                if let EngineEvent::LevelChange(change) = e {
                    let _ = tx.try_send(change);
                }
            })
        };
        let trade_tick_handler = {
            let tx = trade_out_tx.clone();
            let symbol = "AAAA/USDT".to_string();
            let tick_size = 0.1;
            let lot_size = 0.01;

            Arc::new(move |e: EngineEvent| {
                if let EngineEvent::TradeEventResult(trade_result) = e {
                    if let Some(batch) = trade_result.to_trade_batch(&symbol, tick_size, lot_size) {
                        if let Err(err) = tx.try_send(batch) {
                            eprintln!("Trade channel full, dropping batch: {:?}", err);
                        }
                    }
                }
            })
        };

        tokio::spawn(async move {
            let mut publisher = OrderBookPublisher::new(100);
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    Some(change) = change_rx.recv() => {
                        publisher.on_level_change(change);
                    }
                    _ = interval.tick() => {
                        if let Some(msg) = publisher.publish_tick() {
                            if ob_out_tx.send(msg).await.is_err() { break; }
                        }
                    }
                }
            }
        });

        let engine = Engine::new(level_change_handler, trade_tick_handler);
        (engine, ob_out_rx, trade_out_rx)
    }

    pub fn build() -> Engine {
        let (change_tx, mut change_rx) = mpsc::channel::<LevelChange>(10000);
        let (ws_tx, mut ws_rx) = mpsc::channel::<OrderBookMessage>(1000);

        let handler: RouteFn = {
            let tx = change_tx.clone();
            Arc::new(move |e: EngineEvent| {
                if let EngineEvent::LevelChange(change) = e {
                    if let Err(e) = tx.try_send(change) {
                        eprintln!("Warning: Engine is faster than Publisher! {:?}", e);
                    }
                }
            })
        };

        tokio::spawn(async move {
            let mut publisher = OrderBookPublisher::new(100);
            let mut interval = tokio::time::interval(Duration::from_millis(100));

            loop {
                tokio::select! {
                    Some(change) = change_rx.recv() => {
                        publisher.on_level_change(change);
                    }
                    _ = interval.tick() => {
                        if let Some(msg) = publisher.publish_tick() {
                            let _ = ws_tx.send(msg).await;
                        }
                    }
                }
            }
        });

        tokio::spawn(async move {
            while let Some(msg) = ws_rx.recv().await {
                println!("Sending to Client: {:?}", msg);
            }
        });

        let trade_tick_handler = Arc::new(|e: EngineEvent| println!());
        Engine::new(handler, trade_tick_handler)
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

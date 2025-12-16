use std::time::Duration;

use anyhow::Ok;
use bincode::{Decode, Encode};
use chrono::Utc;
use log::info;
use tokio::sync::mpsc;

use crate::matcher::{
    book::{book_manager::OrderBookManager, book_ops::OrderBookOps},
    domain::book_info::BookInfo,
    engine::engine::Engine,
    policy::price_level::{fifo::FifoPriceLevel, price_level::PriceLevelPolicy},
    runtime::{book_client::BookClient, cmd::Cmd},
    storage::{Storage, localfile_storage::LocalFileStorage},
};

pub struct BookActor<T: OrderBookOps, L, F, S>
where
    T: OrderBookOps + Send + 'static,
    L: PriceLevelPolicy + Encode + Decode<()>,
    F: Fn() -> L + Clone,
    S: Storage,
{
    pub rx: mpsc::Receiver<Cmd>,
    pub book: T,
    pub engine: Engine,
    pub book_manager: OrderBookManager<L, F, S>,
}

impl<T: OrderBookOps, L, F, S> BookActor<T, L, F, S>
where
    T: OrderBookOps + Send + 'static,
    L: PriceLevelPolicy + Encode + Decode<()>,
    F: Fn() -> L + Clone,
    S: Storage,
{
    pub fn new(
        rx: mpsc::Receiver<Cmd>,
        book: T,
        engine: Engine,
        book_manager: OrderBookManager<L, F, S>,
    ) -> Self {
        BookActor {
            rx,
            book,
            engine,
            book_manager,
        }
    }

    pub fn build_actor(
        book: T,
        capacity: usize,
        secs: u64,
    ) -> (BookClient, tokio::task::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel::<Cmd>(capacity);
        let book_client = BookClient::new(tx.clone());

        let storage = LocalFileStorage::new(".orderbook_snapshot", 10, "btc-usdt");
        let factory = || FifoPriceLevel::new();
        let book_manager = OrderBookManager::new(storage, factory);

        let actor = BookActor::new(rx, book, Engine, book_manager);

        let handle = tokio::spawn(async move {
            actor.run_loop(secs).await;
        });

        (book_client, handle)
    }

    pub fn re_build_actor(capacity: usize, secs: u64) -> (BookClient, tokio::task::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel::<Cmd>(capacity);
        let book_client = BookClient::new(tx.clone());

        let storage = LocalFileStorage::new(".orderbook_snapshot", 10, "btc-usdt");
        let factory = || FifoPriceLevel::new();
        let book_manager = OrderBookManager::new(storage, factory);
        let book = book_manager.load().unwrap();
        let actor = BookActor::new(rx, book, Engine, book_manager);

        let handle = tokio::spawn(async move {
            actor.run_loop(secs).await;
        });

        (book_client, handle)
    }

    pub fn run(book: T, capacity: usize) -> (BookClient, tokio::task::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel::<Cmd>(capacity);
        let book_client = BookClient::new(tx);

        let handle = tokio::spawn(async move {
            let storage = LocalFileStorage::new(".orderbook_snapshot", 10, "btc-usdt");
            let factory = || FifoPriceLevel::new();
            let book_manager = OrderBookManager::new(storage, factory);

            let mut actor = BookActor {
                rx,
                book,
                engine: Engine,
                book_manager,
            };

            let mut hb = tokio::time::interval(Duration::from_secs(60));
            hb.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            hb.tick().await;

            loop {
                tokio::select! {

                    _ = hb.tick() => {
                        if let Err(e) = actor.handle_tick().await {
                            eprintln!("[actor] handle_tick error: {e:#}");
                        }
                    }

                    maybe = actor.rx.recv() => {
                        match maybe {
                            Some(cmd) => {
                                if let Err(e) = actor.handle_cmd(cmd).await {
                                    eprintln!("[actor] handle_cmd error: {e:#}");
                                }
                                if let Err(e) = actor.drain_batch(256).await {
                                    eprintln!("[actor] drain_batch error: {e:#}");
                                }
                            }
                            None => {
                                break;
                            }
                        }
                    }
                }
            }
        });

        (book_client, handle)
    }

    pub async fn run_loop(mut self, secs: u64) {
        let mut hb = tokio::time::interval(Duration::from_secs(secs));
        hb.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        hb.tick().await;

        loop {
            tokio::select! {

                _ = hb.tick() => {
                    if let Err(e) = self.handle_tick().await {
                        eprintln!("[actor] handle_tick error: {e:#}");
                    }
                }

                maybe = self.rx.recv() => {
                    match maybe {
                        Some(cmd) => {
                            if let Err(e) = self.handle_cmd(cmd).await {
                                eprintln!("[actor] handle_cmd error: {e:#}");
                            }
                            if let Err(e) = self.drain_batch(256).await {
                                eprintln!("[actor] drain_batch error: {e:#}");
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }
            }
        }
    }

    pub async fn handle_tick(&mut self) -> anyhow::Result<()> {
        let now = Utc::now();
        info!("[tick] {}", now.to_rfc3339());
        let book = self.book.get_orderbook()?;
        self.book_manager.save(book)?;
        Result::Ok(())
    }

    pub async fn handle_cmd(&mut self, cmd: Cmd) -> anyhow::Result<()> {
        match cmd {
            Cmd::Info { resp } => {
                let res = self.book.info()?;
                if let Some(tx) = resp {
                    let _ = tx.send(Ok(BookInfo::new(res)));
                }
            }
            Cmd::Place { order, resp } => {
                let res = self.engine.execute(order, &mut self.book);
                if let Some(tx) = resp {
                    let _ = tx.send(res);
                }
            }
            Cmd::Cancel { id, resp } => {
                let res = self.book.cancel(id);
                if let Some(tx) = resp {
                    let _ = tx.send(res);
                }
            }
        }
        Result::Ok(())
    }

    pub async fn drain_batch(&mut self, max: usize) -> anyhow::Result<()> {
        for _ in 0..max {
            match self.rx.try_recv() {
                Result::Ok(cmd) => self.handle_cmd(cmd).await?,
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
        Result::Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use rand::Rng;

    use crate::matcher::{
        book::{book_ops::OrderBookOps, orderbook::OrderBook},
        domain::{
            execution_event::ExecutionEvent,
            order::{Order, OrderSide, OrderType},
            price_ticks::PriceTicks,
            qty_lots::QtyLots,
            reject_reason::RejectReason,
            scales::Scales,
            time_in_force::TimeInForce,
        },
        policy::price_level::fifo::FifoPriceLevel,
        runtime::actor::BookActor,
        storage::localfile_storage::LocalFileStorage,
    };

    pub fn random_order(id: u64, scales: &Scales) -> Order {
        let mut rng = rand::thread_rng();

        // 价格范围：100.00 ~ 200.00
        let px_ticks_range =
            (100.00 * scales.tick_size as f64) as i64..=(200.00 * scales.tick_size as f64) as i64;
        let px_ticks = rng.gen_range(px_ticks_range.clone());
        let px = PriceTicks(px_ticks);

        // 数量范围：0.005 ~ 5.000
        let qty_lots_range =
            (0.005 * scales.lot_size as f64) as i64..=(5.000 * scales.lot_size as f64) as i64;
        let qty_lots = rng.gen_range(qty_lots_range.clone());
        let qty = QtyLots(qty_lots);

        // 随机方向
        let side = if rng.gen_bool(0.5) {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };
        Order {
            id,
            side,
            px,
            qty,
            order_type: OrderType::Limit,
            tif: TimeInForce::IOC,
        }
    }

    #[tokio::test]
    async fn test_book() {
        // let factory: fn() -> FifoPriceLevel = FifoPriceLevel::new;
        // let order_book: OrderBook<FifoPriceLevel, _> = OrderBook::new(factory);
        // let (client, _jh) =
        //     BookActor::<_, FifoPriceLevel, _, LocalFileStorage>::run(order_book, 1024);

        // 1. 定义工厂函数指针类型
        fn factory() -> FifoPriceLevel {
            FifoPriceLevel::new()
        }

        // 2. 显式 OrderBook 类型
        let order_book: OrderBook<FifoPriceLevel, fn() -> FifoPriceLevel> = OrderBook::new(factory);

        // 3. 显式 BookActor 类型
        let (client, _jh) = BookActor::<
            OrderBook<FifoPriceLevel, fn() -> FifoPriceLevel>, // T
            FifoPriceLevel,                                    // L
            fn() -> FifoPriceLevel,                            // F
            LocalFileStorage,                                  // S
        >::build_actor(order_book, 1024, 5);

        let client_clone = Arc::new(client);
        let mut handles = Vec::new();
        let scales = Scales::new(100, 1000);
        for id in 1..=1400000 {
            let mut order = random_order(id, &scales);
            order.tif = TimeInForce::GTC;
            let client = client_clone.clone();
            let handle = tokio::spawn(async move {
                let result = client.place_order(order).await;
                result
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await.unwrap();
            // println!("{:?}", result);
        }

        let client = client_clone.clone();
        let info = client.info_book().await.unwrap();
        println!("order book info {}", info.info);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn actor_smoke_exec() {
        // let factory = || FifoPriceLevel::new();
        // let (client, _jh) = BookActor::run(OrderBook::new(factory), 1024);

        // 1. 定义工厂函数指针类型
        fn factory() -> FifoPriceLevel {
            FifoPriceLevel::new()
        }

        // 2. 显式 OrderBook 类型
        let order_book: OrderBook<FifoPriceLevel, fn() -> FifoPriceLevel> = OrderBook::new(factory);

        // 3. 显式 BookActor 类型
        let (client, _jh) = BookActor::<
            OrderBook<FifoPriceLevel, fn() -> FifoPriceLevel>, // T
            FifoPriceLevel,                                    // L
            fn() -> FifoPriceLevel,                            // F
            LocalFileStorage,                                  // S
        >::run(order_book, 1024);
        let order = Order {
            id: 1,
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            tif: TimeInForce::IOC,
            px: PriceTicks(0),
            qty: QtyLots(100),
        };

        let mut result = client.place_order(order).await.unwrap();
        assert_eq!(1, result.events.len());
        let event = result.events.pop().unwrap();
        assert_eq!(
            ExecutionEvent::Rejected {
                order_id: 1,
                reason: RejectReason::NoMatchingOrder
            },
            event
        );
    }

    #[test]
    fn order_book_test() {
        let factory = || FifoPriceLevel::new();
        let mut order_book = OrderBook::new(factory);
        let scales = Scales::new(100, 1000);
        for id in 0..50000 {
            let order = random_order(id, &scales);
            order_book.add_order(order).unwrap();
        }
        assert_eq!(50000 as usize, order_book.size());

        for id in 0..50000 {
            order_book.cancel(id).unwrap();
        }
        assert_eq!(0, order_book.size());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn gtc_order_test() {
        let order = Order {
            id: 1,
            side: OrderSide::Buy,
            px: PriceTicks(2000),
            qty: QtyLots(20),
            order_type: OrderType::Limit,
            tif: TimeInForce::GTC,
        };

        // let factory = || FifoPriceLevel::new();
        // let book = OrderBook::new(factory);
        // let (client, _jh) = BookActor::run(book, 1024);

        // 1. 定义工厂函数指针类型
        fn factory() -> FifoPriceLevel {
            FifoPriceLevel::new()
        }

        // 2. 显式 OrderBook 类型
        let order_book: OrderBook<FifoPriceLevel, fn() -> FifoPriceLevel> = OrderBook::new(factory);

        // 3. 显式 BookActor 类型
        let (client, _jh) = BookActor::<
            OrderBook<FifoPriceLevel, fn() -> FifoPriceLevel>, // T
            FifoPriceLevel,                                    // L
            fn() -> FifoPriceLevel,                            // F
            LocalFileStorage,                                  // S
        >::run(order_book, 1024);

        let result = client.place_order(order).await.unwrap();
        println!("{:?}", result);
        let info = client.info_book().await.unwrap();
        println!("order book info {}", info.info);

        let order = Order {
            id: 2,
            side: OrderSide::Sell,
            px: PriceTicks(2000),
            qty: QtyLots(30),
            order_type: OrderType::Limit,
            tif: TimeInForce::GTC,
        };

        let result = client.place_order(order).await.unwrap();
        println!("{:?}", result);
        let info = client.info_book().await.unwrap();
        println!("order book info {}", info.info);
        let order = Order {
            id: 3,
            side: OrderSide::Sell,
            px: PriceTicks(1000),
            qty: QtyLots(10),
            order_type: OrderType::Limit,
            tif: TimeInForce::IOC,
        };

        let result = client.place_order(order).await.unwrap();
        println!("{:?}", result);
        let info = client.info_book().await.unwrap();
        println!("order book info {}", info.info);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn gtc_new_order_test() {
        let order = Order {
            id: 1,
            side: OrderSide::Buy,
            px: PriceTicks(2000),
            qty: QtyLots(20),
            order_type: OrderType::Limit,
            tif: TimeInForce::GTC,
        };

        fn factory() -> FifoPriceLevel {
            FifoPriceLevel::new()
        }

        let order_book: OrderBook<FifoPriceLevel, fn() -> FifoPriceLevel> = OrderBook::new(factory);

        let (client, _jh) = BookActor::<
            OrderBook<FifoPriceLevel, fn() -> FifoPriceLevel>, // T
            FifoPriceLevel,                                    // L
            fn() -> FifoPriceLevel,                            // F
            LocalFileStorage,                                  // S
        >::build_actor(order_book, 1024, 300);

        let result = client.place_order(order).await.unwrap();
        println!("{:?}", result);
        let info = client.info_book().await.unwrap();
        println!("order book info {}", info.info);

        let order = Order {
            id: 2,
            side: OrderSide::Sell,
            px: PriceTicks(2000),
            qty: QtyLots(30),
            order_type: OrderType::Limit,
            tif: TimeInForce::GTC,
        };

        let result = client.place_order(order).await.unwrap();
        println!("{:?}", result);
        let info = client.info_book().await.unwrap();
        println!("order book info {}", info.info);
        let order = Order {
            id: 3,
            side: OrderSide::Sell,
            px: PriceTicks(1000),
            qty: QtyLots(10),
            order_type: OrderType::Limit,
            tif: TimeInForce::IOC,
        };

        let result = client.place_order(order).await.unwrap();
        println!("{:?}", result);
        let info = client.info_book().await.unwrap();
        println!("order book info {}", info.info);
    }

    #[tokio::test]
    async fn re_build_actor() {
        let (client, _jh) = BookActor::<
            OrderBook<FifoPriceLevel, fn() -> FifoPriceLevel>, // T
            FifoPriceLevel,                                    // L
            fn() -> FifoPriceLevel,                            // F
            LocalFileStorage,                                  // S
        >::re_build_actor(1024, 300);
        let info = client.info_book().await.unwrap();
        println!("order book info {}", info.info);
    }
}

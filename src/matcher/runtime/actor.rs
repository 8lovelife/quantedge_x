use std::time::Duration;

use chrono::Utc;
use tokio::sync::mpsc;

use crate::matcher::{
    book::book_ops::OrderBookOps,
    engine::engine::Engine,
    runtime::{book_client::BookClient, cmd::Cmd},
};

pub struct BookActor<T: OrderBookOps> {
    pub rx: mpsc::Receiver<Cmd>,
    pub book: T,
    pub engine: Engine,
}

impl<T: OrderBookOps> BookActor<T>
where
    T: OrderBookOps + Send + 'static,
{
    pub fn run(book: T, capacity: usize) -> (BookClient, tokio::task::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel::<Cmd>(capacity);
        let book_client = BookClient::new(tx);

        let handle = tokio::spawn(async move {
            let mut actor = BookActor {
                rx,
                book,
                engine: Engine,
            };

            let mut hb = tokio::time::interval(Duration::from_millis(100));
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

    pub async fn handle_tick(&mut self) -> anyhow::Result<()> {
        let now = Utc::now();
        println!("[tick] {}", now.to_rfc3339());
        Result::Ok(())
    }

    pub async fn handle_cmd(&mut self, cmd: Cmd) -> anyhow::Result<()> {
        match cmd {
            Cmd::Place { order, resp } => {
                let res = self.engine.execute(order, &mut self.book);
                if let Some(tx) = resp {
                    let _ = tx.send(res);
                }
            }
            Cmd::Cancel { id, resp } => {}
        }
        Result::Ok(())
    }

    pub async fn drain_batch(&mut self, max: usize) -> anyhow::Result<()> {
        for _ in 0..max {
            match self.rx.try_recv() {
                Ok(cmd) => self.handle_cmd(cmd).await?,
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
        Result::Ok(())
    }
}

#[cfg(test)]
mod tests {

    use rand::Rng;
    use tokio::sync::mpsc;

    use crate::matcher::{
        book::orderbook::OrderBook,
        domain::{
            order::{Order, OrderEvent, OrderSide, OrderType},
            price_ticks::PriceTicks,
            qty_lots::QtyLots,
            scales::Scales,
            tif_result::TifStatus,
            time_in_force::TimeInForce,
        },
        policy::price_level::fifo::FifoPriceLevel,
        runtime::actor::BookActor,
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

    // #[tokio::test]
    // async fn test_book() {
    //     let mut order_book = OrderBook::new();
    //     let (tx, mut rx) = mpsc::channel::<OrderEvent>(1000);

    //     tokio::spawn(async move {
    //         while let Some(event) = rx.recv().await {
    //             match event {
    //                 OrderEvent::New(order) => {
    //                     order_book.add_order(order);
    //                     order_book.match_orders();
    //                 }
    //                 OrderEvent::Cancel(id) => {
    //                     order_book.cancel_order(id);
    //                 }
    //             }
    //         }
    //     });

    //     let scales = Scales::new(100, 1000);
    //     for id in 1..=5000 {
    //         let order = random_order(id, &scales);
    //         tx.send(OrderEvent::New(order)).await.unwrap();
    //     }

    //     tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    // }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn actor_smoke_exec() {
        let factory = || FifoPriceLevel::new();
        let (client, _jh) = BookActor::run(OrderBook::new(factory), 1024);
        let order = Order {
            id: 1,
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            tif: TimeInForce::GTC,
            px: PriceTicks(0),
            qty: QtyLots(100),
        };

        let result = client.place_order(order).await.unwrap();
        assert_eq!(QtyLots(0), result.filled)
    }
}

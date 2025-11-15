use std::{sync::Arc, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};

use futures_util::future::join_all;
use quantedge_x::matcher::{
    book::{book_ops::OrderBookOps, orderbook::OrderBook},
    domain::{
        order::{Order, OrderSide, OrderType},
        price_ticks::PriceTicks,
        qty_lots::QtyLots,
        scales::Scales,
        time_in_force::TimeInForce,
    },
    policy::price_level::fifo::FifoPriceLevel,
    runtime::actor::BookActor,
    storage::localfile_storage::LocalFileStorage,
};
use rand::Rng;
use tokio::runtime::Runtime;

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

fn bench_sequential_match_orders(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let scales = Scales::new(100, 1000);
    let orders: Vec<Order> = (1..=500_000).map(|id| random_order(id, &scales)).collect();
    c.bench_function("sequential_match_500000_orders", |b| {
        b.to_async(&rt).iter(|| async {
            // let factory = || FifoPriceLevel::new();
            // let (client, _jh) = BookActor::run(OrderBook::new(factory), 1024);

            // 1. 定义工厂函数指针类型
            fn factory() -> FifoPriceLevel {
                FifoPriceLevel::new()
            }

            // 2. 显式 OrderBook 类型
            let order_book: OrderBook<FifoPriceLevel, fn() -> FifoPriceLevel> =
                OrderBook::new(factory);

            // 3. 显式 BookActor 类型
            let (client, _jh) = BookActor::<
                OrderBook<FifoPriceLevel, fn() -> FifoPriceLevel>, // T
                FifoPriceLevel,                                    // L
                fn() -> FifoPriceLevel,                            // F
                LocalFileStorage,                                  // S
            >::run(order_book, 1024);

            for order in orders.iter() {
                let _ = client.place_order(order.clone()).await.unwrap();
            }
        });
    });
}

fn bench_concurrent_match_orders(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("concurrent_match_500000_orders", |b| {
        b.to_async(&rt).iter(|| async {
            // let factory = || FifoPriceLevel::new();
            // let (client, _jh) = BookActor::run(OrderBook::new(factory), 1024);

            // 1. 定义工厂函数指针类型
            fn factory() -> FifoPriceLevel {
                FifoPriceLevel::new()
            }

            // 2. 显式 OrderBook 类型
            let order_book: OrderBook<FifoPriceLevel, fn() -> FifoPriceLevel> =
                OrderBook::new(factory);

            // 3. 显式 BookActor 类型
            let (client, _jh) = BookActor::<
                OrderBook<FifoPriceLevel, fn() -> FifoPriceLevel>, // T
                FifoPriceLevel,                                    // L
                fn() -> FifoPriceLevel,                            // F
                LocalFileStorage,                                  // S
            >::run(order_book, 1024);
            let client = Arc::new(client);

            let scales = Scales::new(100, 1000);
            let orders: Vec<Order> = (1..=500_000).map(|id| random_order(id, &scales)).collect();

            let futures: Vec<_> = orders
                .into_iter()
                .map(|order| {
                    let client = client.clone();
                    tokio::spawn(async move {
                        let _ = client.place_order(order).await;
                    })
                })
                .collect();

            let _ = join_all(futures).await;
        });
    });
}

fn bench_sequential_add_orders(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let scales = Scales::new(100, 1000);
    let orders: Vec<Order> = (1..=5_000_000)
        .map(|id| random_order(id, &scales))
        .collect();
    c.bench_function("sequential_place_5000000_orders", |b| {
        b.to_async(&rt).iter(|| async {
            let factory = || FifoPriceLevel::new();
            let mut order_book = OrderBook::new(factory);
            for order in orders.iter() {
                order_book.add_order(order.clone()).unwrap();
            }
        });
    });
}

fn bench_sequential_cancel_orders(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("sequential_cancel_5000000_orders", |b| {
        b.to_async(&rt).iter(|| async {
            let factory = || FifoPriceLevel::new();
            let mut order_book = OrderBook::new(factory);
            for id in 0..5_000_000 {
                order_book.cancel(id).unwrap();
            }
        });
    });
}

fn custom_criterion() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(3))
}

criterion_group!(
    name = match_benches;
    config = custom_criterion();
    targets = bench_sequential_match_orders, bench_concurrent_match_orders
);

criterion_group!(
    name = order_book_benches;
    config = custom_criterion();
    targets = bench_sequential_add_orders,bench_sequential_cancel_orders
);

criterion_main!(match_benches, order_book_benches);

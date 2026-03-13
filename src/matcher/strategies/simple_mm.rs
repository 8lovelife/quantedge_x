use rand::Rng;
use tokio::{
    task::JoinHandle,
    time::{Duration, sleep},
};

use crate::matcher::{
    domain::{
        order::{Order, OrderSide, OrderType},
        price_ticks::PriceTicks,
        qty_lots::QtyLots,
        time_in_force::TimeInForce,
    },
    runtime::book_client::BookClient,
};

pub struct SimpleMarketMaker {
    client: BookClient,
    symbol: String,
    mid_price: i64,
    spread: i64,
    size: QtyLots,
    order_ids: Vec<u64>,
    order_seq: u64,
}

impl SimpleMarketMaker {
    pub fn new(
        client: BookClient,
        symbol: String,
        mid_price: i64,
        spread: i64,
        size: QtyLots,
    ) -> Self {
        Self {
            client,
            symbol,
            mid_price,
            spread,
            size,
            order_ids: Vec::new(),
            order_seq: 1,
        }
    }

    pub fn start(self) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async move {
            self.run_loop().await;
            Ok(())
        })
    }

    async fn run_loop(mut self) {
        loop {
            self.quote().await;
            sleep(Duration::from_millis(200)).await;
        }
    }

    async fn quote(&mut self) {
        let (buy_offset, sell_offset, move_mid) = {
            let mut rng = rand::thread_rng();
            (
                rng.gen_range(-1..=1),
                rng.gen_range(-1..=1),
                rng.gen_bool(0.3),
            )
        };

        for id in &self.order_ids {
            let _ = self.client.cancel_order(*id).await;
        }
        self.order_ids.clear();

        let buy_px = self.mid_price - self.spread / 2 + buy_offset;
        let sell_px = self.mid_price + self.spread / 2 + sell_offset;

        let buy_id = self.next_id();
        let buy_order = Order {
            id: buy_id,
            order_type: OrderType::Limit,
            tif: TimeInForce::GTC,
            side: OrderSide::Buy,
            px: PriceTicks(buy_px),
            qty: self.size,
        };

        let _ = self.client.place_order(buy_order).await;
        self.order_ids.push(buy_id);

        let sell_id = self.next_id();
        let sell_order = Order {
            id: sell_id,
            order_type: OrderType::Limit,
            tif: TimeInForce::GTC,
            side: OrderSide::Sell,
            px: PriceTicks(sell_px),
            qty: self.size,
        };

        let _ = self.client.place_order(sell_order).await;
        self.order_ids.push(sell_id);

        // println!(
        //     "MM quote | bid={} ask={} spread={}",
        //     buy_px,
        //     sell_px,
        //     sell_px - buy_px
        // );

        if move_mid {
            self.mid_price += buy_offset;
        }
    }

    fn next_id(&mut self) -> u64 {
        let id = self.order_seq;
        self.order_seq += 1;
        id
    }
}

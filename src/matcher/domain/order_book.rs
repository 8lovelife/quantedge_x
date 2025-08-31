use std::{
    collections::BTreeMap,
    fs::{self, File, rename},
    io::Write,
    path::{Path, PathBuf},
};

use bincode::{Decode, Encode, config::standard, decode_from_std_read, encode_into_std_write};
use chrono::Utc;

use super::{
    order::{Order, OrderSide},
    price_ticks::PriceTicks,
};

#[derive(Encode, Decode, Debug)]
pub struct OrderBook {
    pub bids: BTreeMap<PriceTicks, Vec<Order>>,
    pub asks: BTreeMap<PriceTicks, Vec<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        let book = match order.side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        };
        book.entry(order.px).or_default().push(order);
    }

    pub fn cancel_order(&mut self, id: u64) {
        for book in [&mut self.bids, &mut self.asks] {
            for orders in book.values_mut() {
                if let Some(pos) = orders.iter().position(|o| o.id == id) {
                    orders.remove(pos);
                    break;
                }
            }
        }
    }

    pub fn match_orders(&mut self) {
        while let (Some(mut best_bid_e), Some(mut best_ask_e)) =
            (self.bids.last_entry(), self.asks.first_entry())
        {
            let best_bid = *best_bid_e.key();
            let best_ask = *best_ask_e.key();
            if best_bid < best_ask {
                break;
            }

            let bid_orders = best_bid_e.get_mut();
            let ask_orders = best_ask_e.get_mut();

            if bid_orders.is_empty() {
                best_bid_e.remove();
                continue;
            }
            if ask_orders.is_empty() {
                best_ask_e.remove();
                continue;
            }

            let bid_order = bid_orders.first_mut().unwrap();
            let ask_order = ask_orders.first_mut().unwrap();
            let trade_qty = bid_order.qty.min(ask_order.qty);

            println!(
                "Matched: {} (ask {}@{}/{}) <-> (bid {}@{}/{})",
                trade_qty,
                ask_order.id,
                best_ask,
                ask_order.qty,
                bid_order.id,
                best_bid,
                bid_order.qty
            );
            bid_order.qty -= trade_qty;
            ask_order.qty -= trade_qty;

            if bid_order.qty.0 == 0 {
                bid_orders.remove(0);
                if bid_orders.is_empty() {
                    best_bid_e.remove();
                }
            }
            if ask_order.qty.0 == 0 {
                ask_orders.remove(0);
                if ask_orders.is_empty() {
                    best_ask_e.remove();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use rand::Rng;
    use tokio::sync::mpsc;

    use crate::matcher::domain::{
        order::{Order, OrderEvent, OrderSide},
        order_book::OrderBook,
        price_ticks::PriceTicks,
        qty_lots::QtyLots,
        scales::Scales,
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

        Order { id, side, px, qty }
    }

    #[tokio::test]
    async fn test_book() {
        let mut order_book = OrderBook::new();
        let (tx, mut rx) = mpsc::channel::<OrderEvent>(1000);

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    OrderEvent::New(order) => {
                        order_book.add_order(order);
                        order_book.match_orders();
                    }
                    OrderEvent::Cancel(id) => {
                        order_book.cancel_order(id);
                    }
                }
            }
        });

        let scales = Scales::new(100, 1000);
        for id in 1..=5000 {
            let order = random_order(id, &scales);
            tx.send(OrderEvent::New(order)).await.unwrap();
        }

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}

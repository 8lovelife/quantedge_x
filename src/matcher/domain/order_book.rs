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

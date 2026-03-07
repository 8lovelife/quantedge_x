use serde::{Deserialize, Serialize};

use crate::models::{market_price_data::MarketPriceData, order_book_message::OrderBookMessage};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum MarketMessage {
    #[serde(rename = "orderbook")]
    OrderBook {
        symbol: String,
        ts: u64,
        data: OrderBookMessage,
    },

    #[serde(rename = "kline")]
    Kline {
        symbol: String,
        ts: u64,
        data: MarketPriceData,
    },
}

use crate::matcher::domain::order_book::OrderBook;

use anyhow::Result;

pub trait Storage {
    fn save_snapshot(&self, book: &OrderBook) -> Result<()>;
    fn load_latest_snapshot(&self) -> Result<Option<OrderBook>>;
}

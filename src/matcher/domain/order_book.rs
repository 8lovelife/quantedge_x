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

#[derive(Encode, Decode)]
pub struct OrderBook {
    pub bids: BTreeMap<PriceTicks, Vec<Order>>,
    pub asks: BTreeMap<PriceTicks, Vec<Order>>,
    pub snapshot_dir: String,
}

impl OrderBook {
    pub fn new() -> Self {
        let dir = ".book_snapshot".to_string();
        let dir_path = Path::new(dir.as_str());
        if !dir_path.exists() {
            fs::create_dir_all(dir_path).expect("Failed to create .data directory");
        }
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            snapshot_dir: dir,
        }
    }

    pub fn save_snapshot(&self) -> std::io::Result<()> {
        let path = format!(
            "{}/coin_{}.bin",
            self.snapshot_dir.as_str(),
            Utc::now().timestamp()
        );
        let tmp_path = format!("{}.tmp", path);
        let f: File = std::fs::File::create(&tmp_path)?;
        let mut w = std::io::BufWriter::new(f);
        encode_into_std_write(self, &mut w, standard())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        w.flush()?;
        w.get_ref().sync_all()?;
        rename(tmp_path, path)?;
        Ok(())
    }

    fn latest_snapshot_path(&self) -> Option<String> {
        let mut entries: Vec<_> = fs::read_dir(self.snapshot_dir.as_str())
            .ok()?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "bin")
                    .unwrap_or(false)
            })
            .collect();

        entries.sort_by_key(|e| e.file_name());
        entries
            .pop()
            .map(|e| e.path().to_string_lossy().into_owned())
    }

    pub fn prune_keep_last_n(&self, keep: usize) -> std::io::Result<()> {
        let mut files: Vec<PathBuf> = fs::read_dir(self.snapshot_dir.as_str())?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("bin"))
            .collect();
        files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        let files_len = files.len();
        if files.len() > keep {
            for p in files.into_iter().take(files_len - keep) {
                let _ = fs::remove_file(p);
            }
        }
        Ok(())
    }

    pub fn load_snapshot(&self) -> std::io::Result<Self> {
        if let Some(path) = self.latest_snapshot_path() {
            let f = std::fs::File::open(path)?;
            let mut r = std::io::BufReader::new(f);
            let book: Self = decode_from_std_read(&mut r, standard())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            return Ok(book);
        } else {
            Ok(Self::new())
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
        });

        let scales = Scales::new(100, 1000);
        for id in 1..=5000 {
            let order = random_order(id, &scales);
            tx.send(OrderEvent::New(order)).await.unwrap();
        }

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}

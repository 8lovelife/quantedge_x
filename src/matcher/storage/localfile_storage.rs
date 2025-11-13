use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Ok;
use bincode::{config::standard, decode_from_std_read, encode_into_std_write};

use crate::matcher::storage::Storage;

pub struct LocalFileStorage {
    pub root: PathBuf,
    pub snap_prefix: String,
    pub keep: usize,
}

impl LocalFileStorage {
    pub fn new<P: AsRef<Path>>(root: P, keep: usize, snap_prefix: &str) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            snap_prefix: snap_prefix.to_string(),
            keep,
        }
    }

    fn snapshot_name_ts(&self) -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("{}_{}.bin", self.snap_prefix, ts)
    }

    fn is_snapshot_file(&self, path: &Path) -> bool {
        let filename = path.file_name().and_then(OsStr::to_str).unwrap_or("");
        filename.starts_with(&self.snap_prefix) && filename.ends_with(".bin")
    }

    fn list_snapshots_desc(&self) -> anyhow::Result<Vec<PathBuf>> {
        let mut entries =
            fs::read_dir(&self.root).unwrap_or_else(|_| panic!("dir: {:?}", self.root));

        let mut files = Vec::new();
        while let Some(ent) = entries.next() {
            let p = ent?.path();
            if self.is_snapshot_file(&p) {
                files.push(p);
            }
        }

        files.sort_by(|a, b| {
            let an = a.file_name().unwrap_or_default();
            let bn = b.file_name().unwrap_or_default();
            bn.cmp(&an)
        });

        Ok(files)
    }

    fn prune_keep_last_n(&self) -> anyhow::Result<()> {
        let files = self.list_snapshots_desc()?;
        for old in files.into_iter().skip(self.keep) {
            let _ = fs::remove_file(&old);
        }
        Ok(())
    }
}

impl Storage for LocalFileStorage {
    fn save_snapshot(&self, data: &[u8]) -> anyhow::Result<()> {
        let name = self.snapshot_name_ts();
        let dst: PathBuf = self.root.join(name);
        let tmp: PathBuf = dst.with_extension("tmp");
        fs::create_dir_all(&self.root)?;
        let f = File::create(&tmp)?;
        let mut w = BufWriter::new(f);
        encode_into_std_write(data, &mut w, standard())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        w.flush()?;
        w.get_ref().sync_all()?;
        fs::rename(tmp, dst)?;
        Ok(())
    }

    fn load_latest_snapshot(&self) -> anyhow::Result<Option<Vec<u8>>> {
        let files = self.list_snapshots_desc()?;
        if let Some(newest) = files.first() {
            let f = std::fs::File::open(newest)?;
            let mut r = std::io::BufReader::new(f);
            let book = decode_from_std_read(&mut r, standard())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            return Ok(Some(book));
        } else {
            return Ok(None);
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use std::time::Instant;

    use crate::matcher::{
        book::{book_manager::OrderBookManager, book_ops::OrderBookOps, orderbook::OrderBook},
        domain::{
            order::{Order, OrderSide, OrderType},
            price_ticks::PriceTicks,
            qty_lots::QtyLots,
            scales::Scales,
            time_in_force::TimeInForce,
        },
        policy::price_level::fifo::FifoPriceLevel,
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

    #[test]
    fn test_save_and_load_snapshot() {
        let storage = LocalFileStorage::new(".orderbook_snapshot", 10, "btc-usdt");
        let factory = || FifoPriceLevel::new();
        let mut order_book = OrderBook::new(factory);
        let scales = Scales::new(100, 1000);
        for id in 0..50000 {
            let order = random_order(id, &scales);
            order_book.add_order(order).unwrap();
        }

        let save_book_info = order_book.info().unwrap();
        let start = Instant::now();
        let book = order_book.get_orderbook().unwrap();
        let book_manager = OrderBookManager::new(storage, factory);
        book_manager.save(book).unwrap();
        println!(
            "save snapshot cost {}",
            Instant::now().duration_since(start).as_secs(),
        );

        let start = Instant::now();
        let order_book = book_manager.load().unwrap();
        println!(
            "load snapshot cost {}",
            Instant::now().duration_since(start).as_secs(),
        );
        let load_book_info = order_book.info().unwrap();

        assert_eq!(save_book_info, load_book_info)
    }
}

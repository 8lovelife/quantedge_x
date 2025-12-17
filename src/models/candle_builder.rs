#[derive(Debug)]
pub struct CandleBuilder {
    open_ts: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl CandleBuilder {
    fn new(ts: i64, price: f64, qty: f64) -> Self {
        Self {
            open_ts: ts,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: qty,
        }
    }

    fn update(&mut self, price: f64, qty: f64) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
        self.volume += qty;
    }
}

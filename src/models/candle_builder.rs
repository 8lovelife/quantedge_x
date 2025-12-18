#[derive(Debug)]
pub struct CandleBuilder {
    pub open_ts: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl CandleBuilder {
    pub fn new(ts: i64, price: f64, qty: f64) -> Self {
        Self {
            open_ts: ts,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: qty,
        }
    }

    pub fn update(&mut self, price: f64, qty: f64) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
        self.volume += qty;
    }
}

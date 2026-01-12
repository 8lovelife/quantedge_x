pub enum OrderBookMessage {
    Snapshot {
        bids: Vec<(i64, f64)>,
        asks: Vec<(i64, f64)>,
        last_update_id: u64,
    },
    Delta {
        bids: Vec<(i64, f64)>,
        asks: Vec<(i64, f64)>,
        start_id: u64,
        end_id: u64,
    },
}

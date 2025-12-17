use std::collections::HashMap;

use crate::models::candle_builder::CandleBuilder;

pub struct CandleAggregator {
    interval_ms: u64,
    active: HashMap<String, CandleBuilder>,
}

impl CandleAggregator {
    pub fn new(interval_ms: u64) -> Self {
        Self {
            interval_ms,
            active: HashMap::new(),
        }
    }
}

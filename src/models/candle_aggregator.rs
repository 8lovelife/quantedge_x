use std::collections::HashMap;

use crate::models::{
    candle_builder::CandleBuilder, market_price_data::MarketPriceData, trade_tick::TradeTick,
};

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

    pub fn on_tick(&mut self, tick: TradeTick) -> Option<MarketPriceData> {
        let window_start = tick.ts - (tick.ts % self.interval_ms as i64);

        match self.active.get_mut(&tick.symbol) {
            Some(candle) if candle.open_ts == window_start => {
                candle.update(tick.price, tick.qty);
                None
            }
            Some(prev) => {
                let finished = MarketPriceData {
                    symbol: tick.symbol.clone(),
                    timestamp: prev.open_ts,
                    open: prev.open,
                    high: prev.high,
                    low: prev.low,
                    close: prev.close,
                    volume: prev.volume,
                };

                *prev = CandleBuilder::new(window_start, tick.price, tick.qty);

                Some(finished)
            }
            None => {
                self.active.insert(
                    tick.symbol.clone(),
                    CandleBuilder::new(window_start, tick.price, tick.qty),
                );
                None
            }
        }
    }
}

use crate::{data::data_feed::DataFeed, strategy::market_data::MarketData};

use super::coin_market::CoinsMarket;

#[derive(Clone)]
pub struct MarketDataFeed {
    pub records: Vec<MarketData>,
    pub cursor: usize,
}

impl MarketDataFeed {
    pub fn from_coins_market(coin: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let ohlcv_datas = CoinsMarket::get_coin_ohlcv(coin)?;
        let records = ohlcv_datas
            .iter()
            .map(|ohlc| MarketData {
                timestamp: ohlc.timestamp.0.to_rfc3339(),
                close_price: ohlc.close,
            })
            .collect();
        Ok(Self { records, cursor: 0 })
    }
}

impl DataFeed for MarketDataFeed {
    fn next(&mut self) -> Option<MarketData> {
        if self.cursor < self.records.len() {
            let md = self.records[self.cursor].clone();
            self.cursor += 1;
            Some(md)
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.cursor = 0;
    }
}

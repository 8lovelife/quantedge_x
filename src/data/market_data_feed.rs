use std::sync::OnceLock;

use tokio::sync::{broadcast, mpsc};

use crate::{
    data::{
        data_feed::DataFeed,
        market_data_bus::{combine_ticks, start_market_data_bus},
    },
    models::{
        candle_aggregator::CandleAggregator, market_price_data::MarketPriceData,
        trade_tick::TradeTick,
    },
    strategy::market_data::MarketData,
};

use super::coin_market::CoinsMarket;

static MARKET_DATA_FEED: OnceLock<broadcast::Sender<TradeTick>> = OnceLock::new();

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

pub async fn symbol_kline_task(
    symbol: String,
    interval_ms: u64,
    broadcast_tx: broadcast::Sender<MarketPriceData>,
) {
    let tick_bus = start_market_data_bus(symbol.to_string(), interval_ms).await;
    let raw_rx = tick_bus.subscribe();

    let (combine_tx, _) = broadcast::channel::<TradeTick>(1000);
    let mut combined_rx = combine_tx.subscribe();

    tokio::spawn(async move {
        combine_ticks(raw_rx, combine_tx, interval_ms).await;
    });

    let mut builder = CandleAggregator::new(interval_ms);
    while let Ok(tick) = combined_rx.recv().await {
        if tick.symbol != symbol {
            continue;
        }
        let finished_candle = { builder.on_tick(tick).await };
        if let Some(candle) = finished_candle {
            let _ = broadcast_tx.send(candle);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::broadcast;

    use crate::{
        data::market_data_feed::symbol_kline_task, models::market_price_data::MarketPriceData,
    };

    #[tokio::test]
    async fn test_market_feed() {
        let (tick_broadcast_tx, _) = broadcast::channel::<MarketPriceData>(1000);
        let mut frontend_rx = tick_broadcast_tx.subscribe();
        let symbols = vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()];
        for symbol in symbols {
            tokio::spawn(symbol_kline_task(symbol, 2_000, tick_broadcast_tx.clone()));
        }
        tokio::spawn(async move {
            while let Ok(candle) = frontend_rx.recv().await {
                println!("Kline: {:?}", candle);
            }
        });
        loop {
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    }
}

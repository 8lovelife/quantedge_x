use std::{sync::Arc, time::Duration};

use tokio::sync::{Mutex, broadcast, mpsc};

use crate::{
    data::data_feed::DataFeed,
    models::{
        candle_aggregator::CandleAggregator, market_price_data::MarketPriceData,
        trade_tick::TradeTick,
    },
    strategy::market_data::MarketData,
};

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

async fn start_binance_ws(tx: mpsc::Sender<TradeTick>) {
    loop {
        let tick = TradeTick {
            symbol: "BTC/USDT".into(),
            ts: chrono::Utc::now().timestamp_millis(),
            price: rand::random::<f64>() * 30000.0,
            qty: rand::random::<f64>() * 5.0,
        };
        let _ = tx.send(tick).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn start_coinbase_ws(tx: mpsc::Sender<TradeTick>) {
    loop {
        let tick = TradeTick {
            symbol: "ETH/USDT".into(),
            ts: chrono::Utc::now().timestamp_millis(),
            price: rand::random::<f64>() * 2000.0,
            qty: rand::random::<f64>() * 10.0,
        };
        let _ = tx.send(tick).await;
        tokio::time::sleep(Duration::from_millis(700)).await;
    }
}

async fn tick_broadcast_task(
    mut tick_rx: mpsc::Receiver<TradeTick>,
    broadcast_tx: broadcast::Sender<TradeTick>,
) {
    while let Some(tick) = tick_rx.recv().await {
        let _ = broadcast_tx.send(tick);
    }
}

async fn symbol_kline_task(
    symbol: String,
    interval_ms: u64,
    mut rx: broadcast::Receiver<TradeTick>,
    broadcast_tx: broadcast::Sender<MarketPriceData>,
) {
    let builder = Arc::new(Mutex::new(CandleAggregator::new(interval_ms)));

    while let Ok(tick) = rx.recv().await {
        if tick.symbol != symbol {
            continue;
        }

        let finished_candle = {
            let mut b = builder.lock().await;
            b.on_tick(tick).await
        };

        if let Some(candle) = finished_candle {
            let _ = broadcast_tx.send(candle);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::{broadcast, mpsc};

    use crate::{
        data::market_data_feed::{
            start_binance_ws, start_coinbase_ws, symbol_kline_task, tick_broadcast_task,
        },
        models::{market_price_data::MarketPriceData, trade_tick::TradeTick},
    };

    #[tokio::test]
    async fn test_market_feed() {
        let (tick_tx, tick_rx) = mpsc::channel::<TradeTick>(1000);
        let (tick_broadcast_tx, _) = broadcast::channel::<TradeTick>(1000);
        let (candle_broadcast_tx, _) = broadcast::channel::<MarketPriceData>(1000);

        tokio::spawn(start_binance_ws(tick_tx.clone()));
        tokio::spawn(start_coinbase_ws(tick_tx.clone()));

        tokio::spawn(tick_broadcast_task(tick_rx, tick_broadcast_tx.clone()));

        let symbols = vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()];
        for symbol in symbols {
            let rx = tick_broadcast_tx.subscribe();
            let candle_tx = candle_broadcast_tx.clone();
            tokio::spawn(symbol_kline_task(symbol, 2_000, rx, candle_tx));
        }

        let mut frontend_rx = candle_broadcast_tx.subscribe();
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

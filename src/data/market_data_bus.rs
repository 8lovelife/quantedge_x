use std::{collections::HashMap, sync::OnceLock, time::Duration};

use chrono::Utc;
use tokio::sync::broadcast;

use crate::{
    data::market_data_connector::{start_binance_ws, start_coinbase_ws},
    models::trade_tick::TradeTick,
};

static MARKET_DATA_BUS: OnceLock<broadcast::Sender<TradeTick>> = OnceLock::new();

pub async fn start_market_data_bus(
    symbol: String,
    interval_ms: u64,
) -> broadcast::Sender<TradeTick> {
    MARKET_DATA_BUS
        .get_or_init(|| {
            let (tx, _) = broadcast::channel::<TradeTick>(1000);
            tokio::spawn(start_binance_ws(tx.clone(), symbol.clone()));
            tokio::spawn(start_coinbase_ws(tx.clone(), symbol.clone()));
            tx
        })
        .clone()
}

pub async fn combine_ticks(
    mut rx: broadcast::Receiver<TradeTick>,
    tx: broadcast::Sender<TradeTick>,
    interval_ms: u64,
) {
    let mut buffer: HashMap<String, Vec<TradeTick>> = HashMap::new();
    let mut ticker = tokio::time::interval(Duration::from_millis(interval_ms));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            maybe_tick = rx.recv() => {
                match maybe_tick {
                    Ok(tick) => buffer.entry(tick.symbol.clone()).or_default().push(tick),
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("Warning: lagged {} ticks", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            _ = ticker.tick() => {
                let now = Utc::now().timestamp_millis();
                for (symbol, ticks) in buffer.drain() {
                    if ticks.is_empty() { continue; }
                    let total_qty: f64 = ticks.iter().map(|t| t.qty).sum();
                    let weighted_price = ticks.iter().map(|t| t.qty * t.price).sum::<f64>() / total_qty;
                    let trade_tick = TradeTick {
                        symbol,
                        price: weighted_price,
                        qty: total_qty,
                        ts: now,
                    };
                    let _ = tx.send(trade_tick);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::Utc;
    use tokio::sync::broadcast;

    use crate::models::trade_tick::TradeTick;

    #[tokio::test]
    async fn test_market_data_bus() {
        let (tx, _) = broadcast::channel::<TradeTick>(1000);
        let mut rx = tx.subscribe();
        tokio::spawn(async move {
            loop {
                let tick = TradeTick {
                    symbol: "BTC/USDT".into(),
                    price: rand::random::<f64>() * 30000.0,
                    qty: rand::random::<f64>() * 5.0,
                    ts: Utc::now().timestamp_millis(),
                };
                let _ = tx.send(tick);
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
        while let Ok(tick) = rx.recv().await {
            println!("tick : {:?}", tick);
        }
    }
}

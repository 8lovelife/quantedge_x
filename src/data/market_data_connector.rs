use std::time::Duration;

use rand::{Rng, SeedableRng, rngs::StdRng};
use tokio::sync::{broadcast, mpsc};

use crate::models::trade_tick::TradeTick;

pub async fn start_binance_ws(tx: broadcast::Sender<TradeTick>, symbol: String) {
    let symbols = vec!["BTC/USDT", "ETH/USDT", "SOL/USDT"];
    let mut rng = StdRng::from_entropy();

    loop {
        let idx = rng.gen_range(0..symbols.len());
        let symbol = symbols[idx];
        let tick = TradeTick {
            symbol: symbol.to_string(),
            ts: chrono::Utc::now().timestamp_millis(),
            price: rand::random::<f64>() * 30000.0,
            qty: rand::random::<f64>() * 5.0,
        };
        let _ = tx.send(tick);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

pub async fn start_coinbase_ws(tx: broadcast::Sender<TradeTick>, symbol: String) {
    let symbols = vec!["BTC/USDT", "ETH/USDT", "SOL/USDT"];
    let mut rng = StdRng::from_entropy();
    loop {
        let idx = rng.gen_range(0..symbols.len());
        let symbol = symbols[idx];
        let tick = TradeTick {
            symbol: symbol.to_string(),
            ts: chrono::Utc::now().timestamp_millis(),
            price: rand::random::<f64>() * 2000.0,
            qty: rand::random::<f64>() * 10.0,
        };
        let _ = tx.send(tick);
        tokio::time::sleep(Duration::from_millis(70)).await;
    }
}

// async fn combine_ticks(
//     mut rx: mpsc::Receiver<TradeTick>,
//     tx: mpsc::Sender<TradeTick>,
//     interval_ms: u64,
// ) {
//     let mut buffer: HashMap<String, Vec<TradeTick>> = HashMap::new();
//     let mut last_emit = Utc::now().timestamp_millis();
//     while let Some(tick) = rx.recv().await {
//         buffer.entry(tick.symbol.clone()).or_default().push(tick);
//         let now = Utc::now().timestamp_millis();
//         if now - last_emit >= interval_ms as i64 {
//             for (symbol, ticks) in buffer.drain() {
//                 let total_qty: f64 = ticks.iter().map(|t| t.qty).sum();
//                 let weighted_price = ticks.iter().map(|t| t.qty * t.price).sum::<f64>() / total_qty;
//                 let trade_tick = TradeTick {
//                     symbol,
//                     price: weighted_price,
//                     qty: total_qty,
//                     ts: now,
//                 };
//                 let _ = tx.send(trade_tick).await;
//             }
//             last_emit = now;
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use tokio::sync::{broadcast, mpsc};

    use crate::{
        data::{
            market_data_bus::combine_ticks,
            market_data_connector::{start_binance_ws, start_coinbase_ws},
        },
        models::trade_tick::TradeTick,
    };

    #[tokio::test]
    async fn test_connector() {
        let (tx, _) = broadcast::channel::<TradeTick>(1000);
        let mut rx = tx.subscribe();
        tokio::spawn(start_binance_ws(tx.clone(), "BTC/USDT".to_string()));
        tokio::spawn(start_coinbase_ws(tx.clone(), "ETH/USDC".to_string()));
        while let Ok(tick) = rx.recv().await {
            println!("tick: {:?}", tick);
        }
    }

    #[tokio::test]
    async fn test_combine_ticket() {
        let (raw_x, _) = broadcast::channel::<TradeTick>(1000);

        tokio::spawn(start_binance_ws(raw_x.clone(), "BTC/USDT".to_string()));
        tokio::spawn(start_coinbase_ws(raw_x.clone(), "ETH/USDC".to_string()));

        let (tx, _) = broadcast::channel::<TradeTick>(1000);

        let raw_rx = raw_x.subscribe();
        tokio::spawn(combine_ticks(raw_rx, tx.clone(), 3000));

        let mut rx = tx.subscribe();
        while let Ok(tick) = rx.recv().await {
            println!("tick: {:?}", tick);
        }
    }
}

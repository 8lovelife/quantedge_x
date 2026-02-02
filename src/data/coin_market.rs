use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    thread,
};

use serde::{Deserialize, Serialize};

use super::{
    duckdb::{Ohlcv, OhlcvRepository},
    fetcher::common::{fetch_ohlcv_data_coinapi, get_asset_symbol_mapping},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOhlcv {
    pub timestamp: String,
    pub ohlcv: HashMap<String, Ohlcv>,
}

pub struct CoinsMarket;

impl CoinsMarket {
    pub fn get_coins_ohlcv(coins: Vec<String>) -> Result<Vec<MarketOhlcv>, Box<dyn Error>> {
        let mut handles = Vec::new();
        for coin in coins {
            let handle = thread::spawn(move || {
                let result = CoinsMarket::get_coin_ohlcv(&coin).unwrap();
                Ok::<(String, Vec<Ohlcv>), Box<dyn std::error::Error + Send + Sync + 'static>>((
                    coin, result,
                ))
            });
            handles.push(handle);
        }

        let mut coin_ohlcv_map = HashMap::new();

        for handle in handles {
            let (coin, data) = handle.join().unwrap().unwrap();
            coin_ohlcv_map.insert(coin, data);
        }

        // Build timestamp -> coin -> Ohlcv map
        let mut timeline: BTreeMap<String, HashMap<String, Ohlcv>> = BTreeMap::new();

        for (symbol, list) in coin_ohlcv_map {
            for ohlcv in list {
                timeline
                    .entry(ohlcv.timestamp.0.to_rfc3339())
                    .or_default()
                    .insert(symbol.clone(), ohlcv);
            }
        }

        let result = timeline
            .into_iter()
            .map(|(timestamp, ohlcv)| MarketOhlcv { timestamp, ohlcv })
            .collect();

        Ok(result)
    }

    pub fn get_coin_ohlcv(crypto_id: &str) -> Result<Vec<Ohlcv>, Box<dyn Error>> {
        let asset_symbol = get_asset_symbol_mapping();
        let crypto_id: &str = asset_symbol
            .get(crypto_id) // Option<&String>
            .map_or(
                // 如果 None，就退回原始 crypto_id
                crypto_id,
                |s| s, // 如果 Some(s)，就用 s.as_str()
            );
        // let ohlc_datas = fetch_ohlc_data(crypto_id)?;
        // let volumn_datas = fetch_volume_data(crypto_id)?;

        // let ohlcv = ohlc_datas
        //     .into_iter()
        //     .map(|entry| Ohlcv {
        //         id: Uuid::new_v4().to_string(),
        //         volume: *volumn_datas.get(&entry.timestamp).unwrap_or(&0.0),
        //         symbol: entry.symbol,
        //         timestamp: DateTime::<Utc>::from_timestamp(entry.timestamp as i64 / 1000, 0)
        //             .unwrap(),
        //         open: entry.open,
        //         high: entry.high,
        //         low: entry.low,
        //         close: entry.close,
        //     })
        //     .collect();
        // Ok(ohlcv)

        let ohlcv_dao = OhlcvRepository::new(None)?;
        let ohlcv_data = match ohlcv_dao.get_by_symbol(crypto_id) {
            Ok(db_data) if !db_data.is_empty() => db_data,
            _ => match fetch_ohlcv_data_coinapi(crypto_id) {
                Ok(api_data) => api_data,
                _ => vec![],
            },
        };

        Ok(ohlcv_data)
    }

    pub fn sync_coin_ohlcv(crypto_id: &str) -> Result<(), Box<dyn Error>> {
        let ohlcv_dao = OhlcvRepository::new(None)?;
        let mut ohlcv_datas = Self::get_coin_ohlcv(crypto_id)?;
        for ohlcv in ohlcv_datas.iter_mut() {
            ohlcv.symbol = crypto_id.to_string();
        }
        ohlcv_dao.batch_insert(&ohlcv_datas).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::CoinsMarket;

    #[test]
    fn test_get_coin_ohlcv() {
        let mut crypto_id = "bitcoin";
        let ohlcv_datas = CoinsMarket::get_coin_ohlcv(crypto_id).unwrap();
        println!(
            "Sample OHLCV Data: {}",
            serde_json::to_string_pretty(&ohlcv_datas).unwrap()
        );
    }

    #[test]
    fn sync_coin_ohlcv() {
        let crypto_id = "solana"; // solana , ethereum, bitcoin
        CoinsMarket::sync_coin_ohlcv(crypto_id);
    }

    #[test]
    fn test_get_coins_ohlcv() {
        let coins = vec![
            "solana".to_string(),
            "ethereum".to_string(),
            "bitcoin".to_string(),
        ];

        let result = CoinsMarket::get_coins_ohlcv(coins).unwrap();
        println!("{:?}", result)
    }
}

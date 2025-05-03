use std::{collections::HashMap, error::Error};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    data::duckdb::{Ohlcv, types::Timestamp},
    engine::backtester::AssetAllocation,
    utils::path::get_coin_api_key,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct OhlcData {
    pub symbol: String,
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn get_coin_symbol_mapping() -> HashMap<&'static str, &'static str> {
    let mut mapping = HashMap::new();
    mapping.insert("bitcoin", "BITSTAMP_SPOT_BTC_USD");
    mapping.insert("ethereum", "BITSTAMP_SPOT_ETH_USD");
    mapping.insert("solana", "BITSTAMP_SPOT_SOL_USD");
    mapping.insert("bnb", "BINANCE_SPOT_BNB_USDT");
    mapping
}

pub fn get_asset_symbol_mapping() -> HashMap<&'static str, &'static str> {
    let mut mapping = HashMap::new();
    mapping.insert("BTC", "bitcoin");
    mapping.insert("ETH", "ethereum");
    mapping.insert("SOL", "solana");
    mapping.insert("BNB", "BNB");
    mapping.insert("SOL/USDT", "solana");
    mapping.insert("BTC/USDT", "bitcoin");
    mapping.insert("ETH/USDT", "ethereum");
    mapping.insert("BNB/USDT", "bnb");

    mapping
}

pub fn map_asset_symbols(symbols: Vec<String>) -> Vec<String> {
    let mapping = get_asset_symbol_mapping();

    symbols
        .into_iter()
        .filter_map(|s| mapping.get(s.as_str()).map(|&mapped| mapped.to_string()))
        .collect()
}

pub fn map_asset_allocation_symbols(input: Vec<AssetAllocation>) -> Vec<AssetAllocation> {
    let mapping = get_asset_symbol_mapping();

    input
        .into_iter()
        .filter_map(|alloc| {
            mapping
                .get(alloc.symbol.as_str())
                .map(|&mapped_symbol| AssetAllocation {
                    symbol: mapped_symbol.to_string(),
                    allocation: alloc.allocation,
                })
        })
        .collect()
}

pub fn fetch_volume_data(id: &str) -> Result<HashMap<i64, f64>, Box<dyn Error>> {
    let url = format!(
        "https://api.coingecko.com/api/v3/coins/{}/market_chart?vs_currency=usd&days=90",
        id
    );
    let response: serde_json::Value = reqwest::blocking::get(&url)?.json()?;
    let volume_data = response["total_volumes"]
        .as_array()
        .ok_or("Failed to parse volume data")?
        .iter()
        .map(|entry| {
            let timestamp = entry[0].as_i64().unwrap() / 1000; // Convert ms to sec
            let volume = entry[1].as_f64().unwrap();
            (timestamp, volume)
        })
        .collect::<HashMap<i64, f64>>();

    Ok(volume_data)
}

pub fn fetch_ohlc_data(id: &str) -> Result<Vec<OhlcData>, Box<dyn Error>> {
    let url = format!(
        "https://api.coingecko.com/api/v3/coins/{}/ohlc?vs_currency=usd&days=90",
        id
    );

    let response: Vec<Vec<f64>> = reqwest::blocking::get(&url)?.json()?;

    let ohlc_data: Vec<OhlcData> = response
        .into_iter()
        .map(|entry| OhlcData {
            symbol: id.to_owned(),
            timestamp: (entry[0] as i64) / 1000,
            open: entry[1],
            high: entry[2],
            low: entry[3],
            close: entry[4],
        })
        .collect();

    Ok(ohlc_data)
}

pub fn fetch_ohlcv_data_coinapi(symbol: &str) -> Result<Vec<Ohlcv>, Box<dyn Error>> {
    let coin_mapping = get_coin_symbol_mapping();
    let url = format!(
        "https://rest.coinapi.io/v1/ohlcv/{}/history?period_id=1DAY&limit=90",
        coin_mapping.get(symbol).unwrap()
    );
    let client = reqwest::blocking::Client::new();
    let response: serde_json::Value = client
        .get(&url)
        .header("X-CoinAPI-Key", get_coin_api_key())
        .send()?
        .json()?; // Parse as generic JSON first

    // Ensure response is an array
    let entries = response
        .as_array()
        .ok_or("Expected JSON array from CoinAPI")?;

    let ohlc_data: Vec<Ohlcv> = entries
        .iter()
        .map(|entry| Ohlcv {
            symbol: symbol.to_uppercase(),
            timestamp: Timestamp(
                entry["time_period_start"]
                    .as_str()
                    .ok_or("Missing timestamp")
                    .unwrap()
                    .parse::<DateTime<Utc>>()
                    .unwrap(),
            ),
            open: entry["price_open"].as_f64().unwrap_or(0.0),
            high: entry["price_high"].as_f64().unwrap_or(0.0),
            low: entry["price_low"].as_f64().unwrap_or(0.0),
            close: entry["price_close"].as_f64().unwrap_or(0.0),
            volume: entry["volume_traded"].as_f64().unwrap_or(0.0),
        })
        .collect();

    Ok(ohlc_data)
}

#[cfg(test)]
mod tests {
    use crate::data::fetcher::{common::fetch_ohlcv_data_coinapi, fetch_volume_data};

    use super::fetch_ohlc_data;

    #[test]
    fn test_fetch_ohlc_data() {
        let ohlc_datas = fetch_ohlc_data("bitcoin").unwrap();
        println!(
            "Sample ohlc datas {}",
            serde_json::to_string_pretty(&ohlc_datas).unwrap()
        )
    }

    #[test]
    fn test_fetch_volume_data() {
        let volume_datas = fetch_volume_data("bitcoin").unwrap();
        println!(
            "Sample volume datas {}",
            serde_json::to_string_pretty(&volume_datas).unwrap()
        )
    }

    #[test]
    fn test_fetch_ohlcv_data_coinapi() {
        let ohlvc_datas = fetch_ohlcv_data_coinapi("bitcoin").unwrap();
        println!(
            "Sample volume datas {}",
            serde_json::to_string_pretty(&ohlvc_datas).unwrap()
        )
    }
}

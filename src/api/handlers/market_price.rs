use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;

#[derive(Debug, Deserialize)]
pub struct PriceQuery {
    pub symbol: String,
    pub timeframe: String,
}

#[derive(Debug, Serialize)]
pub struct PriceResponse {
    pub symbol: String,
    pub timeframe: String,
    pub data: Vec<PriceData>,
}

#[derive(Debug, Serialize)]
pub struct PriceData {
    pub timestamp: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub async fn get_price(
    Query(query): Query<PriceQuery>,
    State(state): State<AppState>,
) -> Result<Json<PriceResponse>, StatusCode> {
    let ohlcv_data = state
        .ohlcv_repo
        .get_by_symbol(&query.symbol)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data = ohlcv_data
        .into_iter()
        .map(|ohlcv| PriceData {
            timestamp: ohlcv.timestamp.0.to_rfc3339(),
            open: ohlcv.open,
            high: ohlcv.high,
            low: ohlcv.low,
            close: ohlcv.close,
            volume: ohlcv.volume,
        })
        .collect();

    Ok(Json(PriceResponse {
        symbol: query.symbol,
        timeframe: query.timeframe,
        data,
    }))
}

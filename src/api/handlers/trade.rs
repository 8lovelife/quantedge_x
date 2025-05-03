use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use serde::{Deserialize, Serialize};

use crate::api::AppState;

use super::PageQuery;

#[derive(Debug, Serialize)]
pub struct TradeResponse {
    pub page: u32,
    pub limit: u32,
    pub data: Vec<TradeData>,
}

#[derive(Debug, Serialize)]
pub struct TradeData {
    pub id: String,
    pub strategy: String,
    pub trade_type: String,
    pub asset: String,
    pub amount: f64,
    pub price: f64,
    pub timestamp: String,
    pub status: String,
    pub profit: Option<f64>,
}

pub async fn get_recent_trades(
    Query(query): Query<PageQuery>,
    State(state): State<AppState>,
) -> Result<Json<TradeResponse>, StatusCode> {
    let trades = state
        .trade_repo
        .get_recent_trades(query.page, query.limit)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data = trades
        .into_iter()
        .map(|trade| TradeData {
            id: trade.id,
            strategy: trade.strategy,
            trade_type: trade.trade_type,
            asset: trade.asset,
            amount: trade.amount,
            price: trade.price,
            timestamp: trade.timestamp.to_rfc3339(),
            status: trade.status,
            profit: trade.profit,
        })
        .collect();

    Ok(Json(TradeResponse {
        page: query.page,
        limit: query.limit,
        data,
    }))
}

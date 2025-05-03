use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    api::{AppState, handlers::get_current_user_from_cookie},
    data::{
        duckdb::{
            repository::{
                TradeStrategyRepository, backtest_run_repository::BacktestRunHistoryRepository,
            },
            schema::{StrategyTemplate, TradeStrategy, backtest_run_history::BacktestRunHistory},
        },
        sleddb::ChartDB,
    },
    engine::{
        backtest_result::{Balance, Trade},
        backtester::AssetAllocation,
        parameters::StrategyRunParameters,
    },
    indicators::{DistributionData, MonthlyReturnData},
    service::{
        backtest_service::RunBacktestData,
        strategy_service::{ProgressStage, Strategy, StrategyBuilder, StrategyService},
    },
};

use super::{
    PageQuery,
    backtest::{Metrics, RunBacktestHistoryReq},
};

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct BacktestParameters {
//     pub params: StrategyParams,
//     pub timeframe: String,
//     #[serde(rename = "strategyId")]
//     pub strategy_id: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct StrategyParams {
//     #[serde(rename = "smaFast")]
//     pub sma_fast: u32,
//     #[serde(rename = "smaSlow")]
//     pub sma_slow: u32,
//     #[serde(rename = "riskLevel")]
//     pub risk_level: String,
//     #[serde(rename = "stopLoss")]
//     pub stop_loss: u32,
//     #[serde(rename = "takeProfit")]
//     pub take_profit: u32,
//     #[serde(rename = "useTrailingStop")]
//     pub use_trailing_stop: bool,
//     #[serde(rename = "trailingStopDistance")]
//     pub trailing_stop_distance: u32,
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeStrategyRequest {
    pub name: String,
    pub description: String,
    pub algorithm: String,
    pub risk: String,
    pub allocation: u32,
    pub timeframe: String,
    pub assets: Vec<AssetAllocation>,
    pub status: String,
    pub parameters: serde_json::Value, // Storing JSON string for parameters
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyRunApplyRequest {
    pub version: i64,
}

pub struct StrategyTypeRequest {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub timeframe: String,
    pub algorithm_type: String,
}

// {"symbol":"BTC/USDT","weight":33,"direction":"both"}
pub struct StrategyParamsRequest {
    pub symbol: String,
    pub weight: u32,
    pub direction: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeStrategyData {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub algorithm: String,
    pub risk: String,
    pub allocation: u32,
    pub timeframe: String,
    pub assets: String,
    pub status: String,
    pub parameters: serde_json::Value, // Storing JSON string for parameters
}

#[derive(Debug, Serialize)]
pub struct TradeStrategyDataResponse {
    pub total: u32,
    pub data: Vec<TradeStrategy>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrategySummaryResponse {
    pub total: u32,
    pub data: Vec<StrategySummary>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StrategySummary {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub status: String,
    pub description: String,
    pub created: String,
    pub updated: String,
    #[serde(rename = "backtestPerformance")]
    pub backtest_performance: Option<Value>,
    #[serde(rename = "paperPerformance")]
    pub paper_performance: Option<Value>,
    #[serde(rename = "livePerformance")]
    pub live_performance: Option<Value>,
    #[serde(rename = "isIncomplete")]
    pub is_incomplete: bool,
    #[serde(rename = "latestBacktestVersion")]
    pub latest_backtest_version: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyTemplateResponse {
    pub total: u32,
    pub data: Vec<StrategyTemplate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub return_: f64,
    pub drawdown: f64,
    #[serde(rename = "winRate")]
    pub win_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdQuery {
    pub id: i64,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateStatusRequest {
    status: String,
}

#[derive(Deserialize, Serialize)]
pub struct StrategyBuilderReq {
    id: Option<i64>,
    data: Value,
}

pub async fn delete_draft_strategie_by_id(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<i64>,
) -> Result<Json<Response>, StatusCode> {
    let user_info = get_current_user_from_cookie(jar)?;
    let startegy_service = StrategyService::build(user_info.id);
    let effect_size = startegy_service
        .delete_draft_strategy(id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(Response {
        success: effect_size,
    }))
}

pub async fn build_strategy(
    State(state): State<AppState>,
    Path(stage): Path<String>,
    jar: CookieJar,
    Json(payload): Json<StrategyBuilderReq>,
) -> Result<Json<Strategy>, StatusCode> {
    let user_info = get_current_user_from_cookie(jar)?;
    println!("recevied {}", serde_json::to_string(&payload).unwrap());
    let mut builder = StrategyBuilder {
        progress_stage: ProgressStage::from_str(&stage).unwrap(),
        id: payload.id,
        progress_in_type: None,
        progress_in_assets: None,
        progress_in_parameters: None,
        progress_in_risk: None,
    };

    match stage.as_str() {
        "type" => builder.progress_in_type = serde_json::from_value(payload.data).ok(),
        "assets" => builder.progress_in_assets = Some(payload.data),
        "parameters" => builder.progress_in_parameters = Some(payload.data),
        "risk" => builder.progress_in_risk = Some(payload.data),
        _ => {}
    }

    let strategy_service = StrategyService::build(user_info.id);
    let new_trade_strategy = strategy_service
        .build_strategy(builder)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(new_trade_strategy))
}

pub async fn add_trade_strategy(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<TradeStrategyRequest>,
) -> Result<Json<TradeStrategy>, StatusCode> {
    let user_info = get_current_user_from_cookie(jar)?;

    println!("recevied {}", serde_json::to_string(&payload).unwrap());
    let trade_strategy = TradeStrategy {
        id: None,
        name: payload.name,
        description: payload.description,
        algorithm: payload.algorithm,
        risk: payload.risk,
        allocation: payload.allocation,
        timeframe: payload.timeframe,
        assets: serde_json::to_string(&payload.assets).unwrap(),
        status: payload.status,
        parameters: serde_json::to_string(&payload.parameters).unwrap(),
        latest_backtest_version: None,
    };

    let trade_strategy_dao =
        TradeStrategyRepository::build(state.user_connection_manager.get_connection(user_info.id));
    let new_trade_strategy = trade_strategy_dao
        .create(trade_strategy)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(new_trade_strategy))
}

pub async fn get_strategy_summarys(
    Query(query): Query<PageQuery>,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Json<StrategySummaryResponse>, StatusCode> {
    let user_info = get_current_user_from_cookie(jar)?;
    println!("recevied query {}", serde_json::to_string(&query).unwrap());
    let strategy_servivce = StrategyService::build(user_info.id);
    let status = query.status;

    let strategy_summarys = strategy_servivce
        .get_strategy_summarys_by_page(query.page, query.limit, &status)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = strategy_servivce
        .get_strategy_summarys_count(&status)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let strategys_summarys = strategy_summarys
        .into_iter()
        .map(|summary| StrategySummary {
            id: summary.id,
            name: summary.name,
            r#type: summary.algorithm_type,
            status: summary.lifecycle,
            description: summary.description,
            created: summary.created_at.0.to_rfc3339(),
            updated: summary.updated_at.0.to_rfc3339(),
            backtest_performance: summary.backtest_performance,
            paper_performance: summary.paper_performance,
            live_performance: summary.live_performance,
            is_incomplete: summary.progress != "completed",
            latest_backtest_version: summary.latest_backtest_version,
        })
        .collect();

    println!("response {:?}", serde_json::to_string(&strategys_summarys));

    Ok(Json(StrategySummaryResponse {
        total: total,
        data: strategys_summarys,
    }))
}

pub async fn get_trade_strategies(
    Query(query): Query<PageQuery>,
    State(state): State<AppState>,
) -> Result<Json<TradeStrategyDataResponse>, StatusCode> {
    let trade_strategys = state
        .trade_strategy_repo
        .get_by_page(query.page, query.limit)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = state
        .trade_strategy_repo
        .get_count()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TradeStrategyDataResponse {
        total: total,
        data: trade_strategys,
    }))
}

pub async fn delete_trade_strategie_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Response>, StatusCode> {
    let effect_size = state
        .trade_strategy_repo
        .delete_by_id(id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(Response {
        success: effect_size != 0,
    }))
}

pub async fn update_strategy_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Result<Json<Response>, StatusCode> {
    let effect_size = state
        .trade_strategy_repo
        .update_status_by_id(id, &payload.status)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(Response {
        success: effect_size != 0,
    }))
}

pub async fn update_strategy(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<TradeStrategyRequest>,
) -> Result<Json<TradeStrategyData>, StatusCode> {
    let trade_strategy = TradeStrategy {
        id: Some(id),
        name: payload.name,
        description: payload.description,
        algorithm: payload.algorithm,
        risk: payload.risk,
        allocation: payload.allocation,
        timeframe: payload.timeframe,
        assets: serde_json::to_string(&payload.assets).unwrap(),
        status: payload.status,
        parameters: serde_json::to_string(&payload.parameters).unwrap(),
        latest_backtest_version: None,
    };
    let updated_strategy = state
        .trade_strategy_repo
        .update_by_id(id, trade_strategy)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let trade_strategy_data = TradeStrategyData {
        id: id,
        name: updated_strategy.name,
        description: updated_strategy.description,
        algorithm: updated_strategy.algorithm,
        risk: updated_strategy.risk,
        allocation: updated_strategy.allocation,
        timeframe: updated_strategy.timeframe,
        assets: updated_strategy.assets,
        status: updated_strategy.status,
        parameters: payload.parameters,
    };

    Ok(Json(trade_strategy_data))
}

pub async fn get_strategy_details(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(id_query): Query<IdQuery>,
) -> Result<Json<Strategy>, StatusCode> {
    let user_info = get_current_user_from_cookie(jar)?;
    let strategy_service = StrategyService::build(user_info.id);
    println!("recevied strategy id {}", id_query.id);
    let trade_strategy = strategy_service
        .get_strategy_details(id_query.id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    println!("response {:?}", serde_json::to_string(&trade_strategy));
    Ok(Json(trade_strategy))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyBacktestRunRequest {
    #[serde(rename = "strategyId")]
    pub strategy_id: i64,
    #[serde(rename = "type")]
    pub r#type: String,
    pub timeframe: String,
    #[serde(rename = "initialCapital")]
    pub initial_capital: f64,
    pub params: StrategyRunParameters,
}

#[derive(Debug, Serialize)]
pub struct StrategyBacktestRunResponse {
    pub params: Option<StrategyRunParameters>,
    pub trades: Vec<Trade>,
    pub balances: Vec<Balance>,
    #[serde(rename = "monthlyReturns")]
    pub monthly_returns: Vec<MonthlyReturnData>,
    #[serde(rename = "returnDistribution")]
    pub return_distribution: Vec<DistributionData>,
    pub metrics: Metrics,
    pub version: Option<i64>,
    pub date: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRunComparison {
    pub run_id: i64,
    pub strategy_run_history: BacktestRunHistory,
    pub backtest_data: StrategyBacktestRunResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestHistoryDataReq {
    #[serde(rename = "strategyId")]
    pub strategy_id: i64,
    pub version: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct StrategyRunHistoryRes {
    pub historys: Vec<BacktestRunHistory>,
}

pub async fn run_strategy_backtest(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<StrategyBacktestRunRequest>,
) -> Result<Json<StrategyBacktestRunResponse>, StatusCode> {
    let user_info = get_current_user_from_cookie(jar)?;

    let params: StrategyRunParameters = req.params.clone();
    let backtest_result = state
        .backtest_service
        .run_strategy_backtest(user_info.id, req)
        .map_err(|e| {
            println!("run_strategy_backtest failed: {:#?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response = StrategyBacktestRunResponse {
        metrics: backtest_result.metrics.into(),
        trades: backtest_result.trades,
        balances: backtest_result.balances,
        params: Some(params),
        monthly_returns: backtest_result.monthly_returns,
        return_distribution: backtest_result.return_distribution,
        version: backtest_result.version,
        date: backtest_result.date,
    };

    Ok(Json(response))
}

pub async fn backtest_history_data(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(params): Query<BacktestHistoryDataReq>,
) -> Result<Json<Option<StrategyBacktestRunResponse>>, StatusCode> {
    let user_info = get_current_user_from_cookie(jar)?;
    println!("strategy backtest_data req {:?}", params);
    let strategy_id = params.strategy_id;
    let run_id = params.version.unwrap_or(0);
    let run_history_dao = BacktestRunHistoryRepository::build(user_info.id);
    let strategy_run_history = run_history_dao
        .get_by_id(run_id)
        .map_err(|error| {
            eprintln!("DB error while fetching run history backtest: {:?}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let strategy_params = strategy_run_history.parameters.to_string();

    let chart_db = ChartDB::build(user_info.id);
    let backtest_result: RunBacktestData = chart_db
        .retrieve_strategy_chart(strategy_id, run_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // let chart_key = format!("instance-{strategy_id}-{run_id}");
    // let chart_db = ChartDB::new().unwrap();
    // let backtest_result: RunBacktestData = chart_db
    //     .retrieve(&chart_key)
    //     .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    //     .ok_or(StatusCode::NOT_FOUND)?;

    let response = StrategyBacktestRunResponse {
        metrics: backtest_result.metrics.into(),
        trades: backtest_result.trades,
        balances: backtest_result.balances,
        params: Some(serde_json::from_str(&strategy_params).unwrap()),
        monthly_returns: backtest_result.monthly_returns,
        return_distribution: backtest_result.return_distribution,
        version: backtest_result.version,
        date: backtest_result.date,
    };

    println!("strategy backtest_data res {:?}", response);

    Ok(Json(Some(response)))
}

pub async fn backtest_run_history(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(params): Query<RunBacktestHistoryReq>,
) -> Result<Json<StrategyRunHistoryRes>, StatusCode> {
    let user_info = get_current_user_from_cookie(jar)?;
    let run_history_dao = BacktestRunHistoryRepository::build(user_info.id);
    let backtest_runs = run_history_dao
        .get_latest_top_run(params.strategy_id, params.top)
        .map_err(|error| {
            eprintln!("DB error while fetching run history: {:?}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response = StrategyRunHistoryRes {
        historys: backtest_runs,
    };
    println!("strategy_run_history_data res {:?}", &response);
    Ok(Json(response))
}

pub async fn strategy_run_comparison_data(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(params): Query<BacktestHistoryDataReq>,
) -> Result<Json<Vec<StrategyRunComparison>>, StatusCode> {
    let user_info = get_current_user_from_cookie(jar)?;
    println!("strategy_run_comparison_data req {:?}", params);
    let strategy_id = params.strategy_id;
    let run_history_dao = BacktestRunHistoryRepository::build(user_info.id);
    let strategy_run_history = run_history_dao
        .get_latest_top_run(strategy_id, 10)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let comparison_datas = strategy_run_history
        .iter()
        .map(|run_history| {
            let run_id = run_history.id.unwrap_or(0);
            let strategy_params = run_history.parameters.to_string();

            // let chart_key = format!("instance-{strategy_id}-{run_id}");
            // let chart_db = ChartDB::new().unwrap();
            // let backtest_result: RunBacktestData = chart_db.retrieve(&chart_key).unwrap().unwrap();

            let chart_db = ChartDB::build(user_info.id);
            let backtest_result: RunBacktestData = chart_db
                .retrieve_strategy_chart(strategy_id, run_id)
                .unwrap()
                .unwrap();

            StrategyRunComparison {
                run_id,
                strategy_run_history: run_history.clone(),
                backtest_data: StrategyBacktestRunResponse {
                    metrics: backtest_result.metrics.into(),
                    trades: backtest_result.trades,
                    balances: backtest_result.balances,
                    params: Some(serde_json::from_str(&strategy_params).unwrap()),
                    monthly_returns: backtest_result.monthly_returns,
                    return_distribution: backtest_result.return_distribution,
                    version: backtest_result.version,
                    date: backtest_result.date,
                },
            }
        })
        .collect::<Vec<_>>();

    println!("strategy_run_history_data res {:?}", &comparison_datas);
    Ok(Json(comparison_datas))
}

pub async fn appy_strategy_run(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    jar: CookieJar,
    Json(payload): Json<StrategyRunApplyRequest>,
) -> Result<(), StatusCode> {
    let user_info = get_current_user_from_cookie(jar)?;
    let strategy_id = id;
    let version = payload.version;

    let strategy_service = StrategyService::build(user_info.id);
    strategy_service
        .apply_strategy_run(strategy_id, version)
        .unwrap();
    Ok(())
}

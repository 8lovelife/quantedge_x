use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    api::AppState,
    data::{
        duckdb::schema::{LabRunHistory, StrategyTemplate},
        sleddb::ChartDB,
    },
    engine::{
        backtest_result::{Balance, Trade},
        parameters::StrategyRunParameters,
    },
    indicators::calculator::{DistributionData, MonthlyReturnData},
    service::backtest_service::{
        RunBacktestData, RunStrategyCoreParameters, RunStrategyExecutionParameters,
        RunStrategyParameters, RunStrategyRiskParameters,
    },
};

use super::{
    PageQuery, StrategyTemplateResponse,
    backtest::{LabBacktestRunRequest, Metrics},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Algorithm {
    pub id: Option<i64>,
    pub label: String,
    pub value: String,
    pub desc: String,
    pub info: String,
    #[serde(rename = "defaultParameters")]
    pub parameters: Option<HashMap<String, Value>>,
    #[serde(rename = "defaultRisk")]
    pub risk_parameters: Option<HashMap<String, Value>>,
    #[serde(rename = "defaultExecution")]
    pub execution_parameters: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabRunHistoryDataReq {
    #[serde(rename = "templateId")]
    pub template_id: i64,
    pub version: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct LabRunHistoryBacktestRes {
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
pub struct LabRunHistoryRes {
    pub historys: Vec<LabRunHistory>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabRunComparison {
    pub run_id: i64,
    pub lab_run_history: LabRunHistory,
    pub backtest_data: LabBacktestRunResponse,
}

#[derive(Debug, Serialize)]
pub struct LabBacktestRunResponse {
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

impl Algorithm {
    pub fn to_run_backtest_strategy_parameters(
        &self,
    ) -> Result<RunStrategyParameters, Box<dyn std::error::Error>> {
        // Extract and map `defaultParameters` to `RunBacktestCoreParameters`
        let core = match self.value.as_str() {
            "ma-crossover" => {
                let fast_period = self
                    .parameters
                    .as_ref()
                    .and_then(|params| params.get("fastPeriod"))
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing fastPeriod")?;
                let slow_period = self
                    .parameters
                    .as_ref()
                    .and_then(|params| params.get("slowPeriod"))
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing slowPeriod")?;
                let entry_threshold = self
                    .parameters
                    .as_ref()
                    .and_then(|params| params.get("entryThreshold"))
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing entryThreshold")?;
                let exit_threshold = self
                    .parameters
                    .as_ref()
                    .and_then(|params| params.get("exitThreshold"))
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing exitThreshold")?;

                RunStrategyCoreParameters::MovingAverage {
                    fast_period,
                    slow_period,
                    entry_threshold,
                    exit_threshold,
                }
            }
            "mean-reversion" => {
                let mean_type = self
                    .parameters
                    .as_ref()
                    .and_then(|params| params.get("meanType"))
                    .and_then(|v| v.as_str())
                    .ok_or("Missing meanType")?
                    .to_string();
                let lookback_period = self
                    .parameters
                    .as_ref()
                    .and_then(|params| params.get("lookbackPeriod"))
                    .and_then(|v| v.as_u64())
                    .ok_or("Missing lookbackPeriod")? as u32;
                let entry_threshold = self
                    .parameters
                    .as_ref()
                    .and_then(|params| params.get("entryThreshold"))
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing entryThreshold")?;
                let exit_threshold = self
                    .parameters
                    .as_ref()
                    .and_then(|params| params.get("exitThreshold"))
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing exitThreshold")?;

                RunStrategyCoreParameters::MeanReversion {
                    mean_type,
                    lookback_period,
                    entry_threshold,
                    exit_threshold,
                }
            }
            "bollinger-bands" => {
                let period = self
                    .parameters
                    .as_ref()
                    .and_then(|params| params.get("period"))
                    .and_then(|v| v.as_u64())
                    .ok_or("Missing period")? as u32;
                let std_dev = self
                    .parameters
                    .as_ref()
                    .and_then(|params| params.get("stdDev"))
                    .and_then(|v| v.as_f64())
                    .ok_or("Missing stdDev")?;

                RunStrategyCoreParameters::BollingerBands { period, std_dev }
            }
            _ => RunStrategyCoreParameters::Custom {
                parameters: serde_json::to_value(self.parameters.clone()).unwrap_or_default(),
            },
        };

        // Extract and map `defaultRisk` to `RunBacktestRiskParameters`
        let risk = RunStrategyRiskParameters {
            stop_loss: self
                .risk_parameters
                .as_ref()
                .and_then(|params| params.get("stopLoss"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            take_profit: self
                .risk_parameters
                .as_ref()
                .and_then(|params| params.get("takeProfit"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            risk_per_trade: self
                .risk_parameters
                .as_ref()
                .and_then(|params| params.get("riskPerTrade"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            position_size: self
                .risk_parameters
                .as_ref()
                .and_then(|params| params.get("positionSize"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            max_concurrent_positions: self
                .risk_parameters
                .as_ref()
                .and_then(|params| params.get("maxConcurrentPositions"))
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as u32,
        };

        // Extract and map `defaultExecution` to `RunBacktestExecutionParameters`
        let execution = RunStrategyExecutionParameters {
            slippage: self
                .execution_parameters
                .as_ref()
                .and_then(|params| params.get("slippage"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            commission: self
                .execution_parameters
                .as_ref()
                .and_then(|params| params.get("commission"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            entry_delay: self
                .execution_parameters
                .as_ref()
                .and_then(|params| params.get("entryDelay"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            min_holding_period: self
                .execution_parameters
                .as_ref()
                .and_then(|params| params.get("minHoldingPeriod"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            max_holding_period: self
                .execution_parameters
                .as_ref()
                .and_then(|params| params.get("maxHoldingPeriod"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
        };

        // Construct and return `RunBacktestStrategyParameters`
        Ok(RunStrategyParameters {
            core,
            risk,
            execution,
        })
    }
}

pub async fn get_strategy_template_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Option<StrategyTemplate>>, StatusCode> {
    let strategy_template = state
        .strategy_template_repo
        .get_template_by_id(id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(strategy_template))
}

pub async fn get_strategy_templates(
    Query(query): Query<PageQuery>,
    State(state): State<AppState>,
) -> Result<Json<StrategyTemplateResponse>, StatusCode> {
    println!("recevied query {}", serde_json::to_string(&query).unwrap());
    let response = match query.status.as_deref() {
        Some("lab") => {
            let strategy_summaries = state
                .strategy_template_repo
                .get_templates_by_page(query.page, query.limit)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let total = state
                .strategy_template_repo
                .get_count()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            StrategyTemplateResponse {
                total,
                data: strategy_summaries,
            }
        }
        Some("community") => StrategyTemplateResponse {
            total: 0,
            data: vec![],
        },
        _ => {
            let strategy_summaries = state
                .strategy_template_repo
                .get_templates_by_page(query.page, query.limit)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let total = state
                .strategy_template_repo
                .get_count()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            StrategyTemplateResponse {
                total,
                data: strategy_summaries,
            }
        }
    };

    println!("response {:?}", serde_json::to_string(&response));

    Ok(Json(response))
}

pub async fn run_lab_backtest(
    State(state): State<AppState>,
    Json(req): Json<LabBacktestRunRequest>,
) -> Result<Json<LabBacktestRunResponse>, StatusCode> {
    let params: StrategyRunParameters = req.params.clone();

    let backtest_result = state
        .backtest_service
        .run_lab_backtest(req)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = LabBacktestRunResponse {
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

pub async fn lab_run_history_backtest_data(
    State(state): State<AppState>,
    Query(params): Query<LabRunHistoryDataReq>,
) -> Result<Json<Option<LabBacktestRunResponse>>, StatusCode> {
    println!("lab backtest_data req {:?}", params);
    let template_id = params.template_id;
    let run_id = params.version.unwrap_or(0);

    let lab_run_history = state
        .lab_run_history_repo
        .get_by_id(run_id)
        .map_err(|error| {
            eprintln!("DB error while fetching run history backtest: {:?}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let strategy_params = lab_run_history.parameters.to_string();

    let chart_key = format!("template-{template_id}-{run_id}");
    let chart_db = ChartDB::new().unwrap();
    let backtest_result: RunBacktestData = chart_db
        .retrieve(&chart_key)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let response = LabBacktestRunResponse {
        metrics: backtest_result.metrics.into(),
        trades: backtest_result.trades,
        balances: backtest_result.balances,
        params: Some(serde_json::from_str(&strategy_params).unwrap()),
        monthly_returns: backtest_result.monthly_returns,
        return_distribution: backtest_result.return_distribution,
        version: backtest_result.version,
        date: backtest_result.date,
    };

    println!("lab backtest_data res {:?}", response);

    Ok(Json(Some(response)))
}

pub async fn lab_run_comparison_data(
    State(state): State<AppState>,
    Query(params): Query<LabRunHistoryDataReq>,
) -> Result<Json<Vec<LabRunComparison>>, StatusCode> {
    println!("lab_run_history_data req {:?}", params);
    let template_id = params.template_id;

    let lab_run_history = state
        .lab_run_history_repo
        .get_latest_top_run(template_id, 10)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let comparison_datas = lab_run_history
        .iter()
        .map(|lab_run_history| {
            let run_id = lab_run_history.id.unwrap_or(0);
            let strategy_params = lab_run_history.parameters.to_string();

            let chart_key = format!("template-{template_id}-{run_id}");
            let chart_db = ChartDB::new().unwrap();
            let backtest_result: RunBacktestData = chart_db.retrieve(&chart_key).unwrap().unwrap();

            LabRunComparison {
                run_id,
                lab_run_history: lab_run_history.clone(),
                backtest_data: LabBacktestRunResponse {
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

    println!("lab_run_history_data res {:?}", &comparison_datas);
    Ok(Json(comparison_datas))
}

pub async fn lab_run_history_data(
    State(state): State<AppState>,
    Query(params): Query<LabRunHistoryDataReq>,
) -> Result<Json<LabRunHistoryRes>, StatusCode> {
    println!("lab_run_history_data req {:?}", params);
    let template_id = params.template_id;

    let lab_run_history = state
        .lab_run_history_repo
        .get_latest_top_run(template_id, 10)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = LabRunHistoryRes {
        historys: lab_run_history,
    };

    println!("lab_run_history_data res {:?}", &response);

    Ok(Json(response))
}

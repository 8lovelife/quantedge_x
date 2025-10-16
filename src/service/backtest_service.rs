use std::{error::Error, sync::Arc};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    api::handlers::{StrategyBacktestRunRequest, backtest::LabBacktestRunRequest},
    data::{
        duckdb::{
            OhlcvRepository,
            repository::{
                LabRunHistoryRepository, StrategyBuildsRepository, StrategyTemplateRepository,
                TradeStrategyRepository,
                backtest_run_repository::BacktestRunHistoryRepository,
                connection::{UserConnectionManager, get_user_connection_manager},
            },
            schema::{LabRunHistory, backtest_run_history::BacktestRunHistory},
            types::Timestamp,
        },
        market_data_feed::MarketDataFeed,
        sleddb::ChartDB,
    },
    engine::{
        backtest_result::{Balance, Trade},
        backtester::BacktestDriver,
        parameters::BacktestInput,
    },
    indicators::{
        DistributionData, calculate_daily_return_distribution, calculate_monthly_returns,
        calculator::MonthlyReturnData,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunBacktestParameters {
    pub sma_fast: f64,
    pub sma_slow: f64,
    pub stop_loss: f64,
    pub risk_level: String,
    pub take_profit: f64,
    pub use_trailing_stop: bool,
    pub trailing_stop_distance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStrategyRiskParameters {
    pub stop_loss: f64,
    pub take_profit: f64,
    pub risk_per_trade: f64,
    pub position_size: f64,
    pub max_concurrent_positions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStrategyExecutionParameters {
    pub slippage: f64,
    pub commission: f64,
    pub entry_delay: u32,
    pub min_holding_period: u32,
    pub max_holding_period: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RunStrategyCoreParameters {
    MovingAverage {
        fast_period: f64,
        slow_period: f64,
        entry_threshold: f64,
        exit_threshold: f64,
    },
    MeanReversion {
        mean_type: String,
        lookback_period: u32,
        entry_threshold: f64,
        exit_threshold: f64,
    },
    BollingerBands {
        period: u32,
        std_dev: f64,
    },
    Custom {
        parameters: Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStrategyParameters {
    pub core: RunStrategyCoreParameters,
    pub risk: RunStrategyRiskParameters,
    pub execution: RunStrategyExecutionParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketDetails {
    pub pairs: String,
    pub timeframe: String,
    // #[serde(rename = "initialCapital")]
    pub initial_capital: f64,
    // #[serde(rename = "positionType")]
    pub position_type: String,
    // #[serde(rename = "subType")]
    pub sub_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRunMarketDetails {
    pub timeframe: String,
    pub initial_capital: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunBacktestReq {
    pub params: RunBacktestParameters,
    pub timeframe: String,
    pub strategy_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BacktestMetrics {
    #[serde(rename = "strategyReturn")]
    pub strategy_return: f64,
    #[serde(rename = "maxDrawdown")]
    pub max_drawdown: f64,
    #[serde(rename = "winRate")]
    pub win_rate: f64,
    #[serde(rename = "sharpeRatio")]
    pub sharpe_ratio: f64,
    #[serde(rename = "totalTrades")]
    pub total_trades: u64,
    #[serde(rename = "profitFactor")]
    pub profit_factor: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunBacktestData {
    pub trades: Vec<Trade>,
    pub balances: Vec<Balance>,
    pub metrics: BacktestMetrics,
    #[serde(rename = "monthlyReturns")]
    pub monthly_returns: Vec<MonthlyReturnData>,
    #[serde(rename = "returnDistribution")]
    pub return_distribution: Vec<DistributionData>,
    pub version: Option<i64>,
    pub date: Option<String>,
}

pub struct BacktestService {
    // backtest_run_history_repo: Arc<BacktestRunHistoryRepository>,
    trade_strategy_repo: Arc<TradeStrategyRepository>,
    ohlcv_repo: Arc<OhlcvRepository>,
    // strategy_builds_repo: Arc<StrategyBuildsRepository>,
    strategy_template_repo: Arc<StrategyTemplateRepository>,
    user_connection_manager: Arc<UserConnectionManager>,
}

impl BacktestService {
    pub fn new() -> Result<Self, duckdb::Error> {
        let ohlcv_repo = Arc::new(OhlcvRepository::new(None)?);
        let trade_strategy_repo = Arc::new(TradeStrategyRepository::new(None)?);
        // let backtest_run_history_repo = Arc::new(BacktestRunHistoryRepository::new()?);
        // let strategy_builds_repo = Arc::new(StrategyBuildsRepository::new(None)?);
        let strategy_template_repo = Arc::new(StrategyTemplateRepository::new()?);

        Ok(Self {
            // backtest_run_history_repo: backtest_run_history_repo,
            trade_strategy_repo: trade_strategy_repo,
            ohlcv_repo: ohlcv_repo,
            // strategy_builds_repo: strategy_builds_repo,
            strategy_template_repo,
            user_connection_manager: get_user_connection_manager(),
        })
    }

    // pub async fn run_grid(req: GridSearchRequest) -> Result<GridResult> {
    //     let combos = expand_grid(&req.grid_params);
    // }

    pub fn backtest_data_history(
        &self,
        run_id: i64,
        strategy_id: i64,
    ) -> Result<RunBacktestData, Box<dyn Error>> {
        let chart_db = ChartDB::new()?;
        let chart_key = format!("{strategy_id}-{run_id}");
        let chart_json: Option<RunBacktestData> = chart_db.retrieve(&chart_key)?;
        chart_json.ok_or_else(|| "Chart data not found".into())
    }

    // pub fn run_backtest(
    //     &self,
    //     run_backtest: RunBacktestReq,
    // ) -> Result<RunBacktestData, duckdb::Error> {
    //     let strategy_id = run_backtest.strategy_id;

    //     let strategy = self
    //         .strategy_builds_repo
    //         .get_by_id(strategy_id)
    //         .expect(format!("trade strategy with id {} db error", strategy_id).as_str())
    //         .expect(format!("trade strategy with id {} is not exist", strategy_id).as_str());

    //     let asset_allocation: Vec<AssetAllocation> = match strategy.assets {
    //         Some(value) => {
    //             let raw_assets: Vec<serde_json::Value> = serde_json::from_value(value).unwrap();
    //             raw_assets
    //                 .into_iter()
    //                 .map(|raw_asset| {
    //                     let symbol = raw_asset["symbol"].as_str().unwrap().to_string();
    //                     let allocation = raw_asset["weight"].as_u64().unwrap() as u32; // Map "weight" to "allocation"
    //                     AssetAllocation { symbol, allocation }
    //                 })
    //                 .collect()
    //         }
    //         None => Vec::new(),
    //     };

    //     let asset_symbols: Vec<String> = asset_allocation
    //         .iter()
    //         .map(|a| a.symbol.to_string())
    //         .collect();
    //     let asset_symbols = map_asset_symbols(asset_symbols);
    //     let asset_allocation = map_asset_allocation_symbols(asset_allocation);

    //     let portfolio_asset = PortfolioAsset::new(asset_allocation);

    //     // let algorithm: StrategyTypeName = strategy.algorithm.parse().unwrap();
    //     let run_params = StrategyRunParams {
    //         name: "EMA(5,10)".to_string(),
    //         strategy: StrategyType::MA {
    //             short_period: MovingAverageType::EMA(1),
    //             long_period: MovingAverageType::EMA(5),
    //         },
    //         stop_loss: 2.0,
    //         take_profit: 3.0,
    //         use_trailing_stop: false,
    //         trailing_stop_distance: 0.0,
    //         starting_capital: 10000.0,
    //         market_data: None,
    //         risk_per_trade: 2.0,
    //     };

    //     let run_history_repo = BacktestRunRepository::new()?;
    //     let run_history = BacktestRunHistory {
    //         id: None,
    //         strategy_id: strategy_id,
    //         parameters: serde_json::to_value(&run_params).unwrap(),
    //         status: "running".to_string(),
    //         start_time: Timestamp(Utc::now()),
    //         end_time: None,
    //         created_at: None,
    //         market_details: None,
    //         performance: None,
    //     };
    //     let mut running_backtest = run_history_repo.create(run_history)?;

    //     let mut backtester = Backtester::new(run_params);
    //     let ohlcv_datas = CoinsMarket::get_coins_ohlcv(asset_symbols);

    //     if let Ok(ohlcv) = ohlcv_datas {
    //         let market_data: Vec<MarketData> = portfolio_asset.to_portfolio_market_data(ohlcv);
    //         backtester.run(market_data);
    //         running_backtest.status = "success".to_string();
    //     } else {
    //         running_backtest.status = "failed".to_string();
    //     }

    //     running_backtest.end_time = Some(Timestamp(Utc::now()));
    //     run_history_repo.update(&running_backtest)?;

    //     let run_id = running_backtest.id.unwrap();

    //     let backtest_result = backtester.get_backtest_result().unwrap();
    //     let metrics = BacktestMetrics {
    //         win_rate: backtest_result.win_rate,
    //         max_drawdown: backtest_result.max_drawdown,
    //         sharpe_ratio: backtest_result.sharpe_ratio,
    //         strategy_return: backtest_result.total_return,
    //         total_trades: backtest_result.trades.len() as u64,
    //         profit_factor: backtest_result.profit_factor,
    //     };

    //     let metrics_value = serde_json::to_value(&metrics).unwrap();

    //     let _ = self
    //         .strategy_builds_repo
    //         .update_lasest_backtest_by_id(strategy_id, &metrics_value, run_id)
    //         .unwrap();

    //     let balances = backtest_result.balances;
    //     let daily_returns = calculate_daily_return_distribution(&balances, 1.0);
    //     let monthly_returns = calculate_monthly_returns(&balances);
    //     let result = RunBacktestData {
    //         trades: backtest_result.trades,
    //         balances: balances,
    //         metrics,
    //         monthly_returns: monthly_returns,
    //         return_distribution: daily_returns,
    //         version: running_backtest.id,
    //         date: Some(running_backtest.start_time.0.to_rfc3339()),
    //     };

    //     let run_id = running_backtest.id.unwrap();
    //     let chart_key = format!("instance-{strategy_id}-{run_id}");

    //     let chart_db = ChartDB::new().unwrap();
    //     chart_db.store_json(&chart_key, &result).unwrap();

    //     let result_string = serde_json::to_string(&result).unwrap();
    //     println!("run backtest result {}", &result_string);
    //     Ok(result)
    // }

    pub fn run_strategy_backtest(
        &self,
        user_id: i64,
        run_strategy_backtest: StrategyBacktestRunRequest,
    ) -> Result<RunBacktestData, duckdb::Error> {
        let strategy_id = run_strategy_backtest.strategy_id;
        let strategy_type = run_strategy_backtest.r#type;

        let strategy_builds_dao = StrategyBuildsRepository::build(user_id);
        let strategy_build = strategy_builds_dao.get_by_id(strategy_id)?;

        let strategy_build = strategy_build.unwrap();

        let mut params = run_strategy_backtest.params;

        let market_details = StrategyRunMarketDetails {
            timeframe: run_strategy_backtest.timeframe,
            initial_capital: run_strategy_backtest.initial_capital,
        };

        let run_history = BacktestRunHistory {
            id: None,
            strategy_id,
            parameters: serde_json::to_value(&params).unwrap(),
            performance: None,
            status: "running".to_string(),
            start_time: Timestamp(Utc::now()),
            end_time: None,
            created_at: None,
            market_details: serde_json::to_value(&market_details).unwrap(),
        };

        let backtest_run_history_dao = BacktestRunHistoryRepository::build(user_id);

        let mut lab_running_backtest = backtest_run_history_dao.create(run_history)?;

        params.normalize_percentages();

        let assets = strategy_build.assets;

        let (symbol_opt, direction_opt) = assets
            .as_ref()
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.get(0))
            .map(|first| {
                let symbol = first.get("symbol").and_then(|s| s.as_str());
                let direction = first.get("direction").and_then(|d| d.as_str());
                (symbol, direction)
            })
            .unwrap_or((None, None));

        let symbol = symbol_opt.unwrap();
        let direction = direction_opt.unwrap();
        params.position_type = Some(direction.to_string());

        let backtest_input = BacktestInput {
            r#type: strategy_type,
            initial_capital: market_details.initial_capital,
            strategy_run_params: serde_json::to_value(&params).unwrap(),
        };

        let datafeed = MarketDataFeed::from_coins_market(symbol);

        let backtest_result = match datafeed {
            Ok(ohlcv) => {
                let backtest_driver = BacktestDriver::new(backtest_input, ohlcv);
                let res = backtest_driver.build_and_run_backtest();
                lab_running_backtest.status = "success".to_string();
                Some(res)
            }
            Err(_) => {
                lab_running_backtest.status = "failed".to_string();
                backtest_run_history_dao.update(&lab_running_backtest)?;
                None
            }
        };

        let backtest_result = backtest_result.unwrap();
        let metrics = BacktestMetrics {
            win_rate: backtest_result.win_rate,
            max_drawdown: backtest_result.max_drawdown,
            sharpe_ratio: backtest_result.sharpe_ratio,
            strategy_return: backtest_result.total_return,
            total_trades: backtest_result.trades.len() as u64,
            profit_factor: backtest_result.profit_factor,
        };

        lab_running_backtest.end_time = Some(Timestamp(Utc::now()));
        lab_running_backtest.performance = Some(serde_json::to_value(&metrics).unwrap());
        backtest_run_history_dao.update(&lab_running_backtest)?;

        let run_id: i64 = lab_running_backtest.id.unwrap();
        let _ = strategy_builds_dao
            .update_lasest_backtest_by_id(strategy_id, run_id)
            .unwrap();

        let balances = backtest_result.balances;
        let daily_returns = calculate_daily_return_distribution(&balances, 1.0);
        let monthly_returns = calculate_monthly_returns(&balances);
        let result = RunBacktestData {
            trades: backtest_result.trades,
            balances,
            metrics,
            monthly_returns,
            return_distribution: daily_returns,
            version: lab_running_backtest.id,
            date: Some(lab_running_backtest.start_time.0.to_rfc3339()),
        };

        let run_id = lab_running_backtest.id.unwrap();
        let chart_db = ChartDB::build(user_id);

        chart_db
            .store_strategy_chart(strategy_id, run_id, &result)
            .unwrap();

        let result_string = serde_json::to_string(&result).unwrap();
        println!("run strategy backtest result {}", &result_string);
        Ok(result)
    }

    pub fn run_lab_backtest(
        &self,
        run_lab_backtest: LabBacktestRunRequest,
    ) -> Result<RunBacktestData, duckdb::Error> {
        let template_id = run_lab_backtest.template_id;
        let strategy_type = run_lab_backtest.r#type;

        let pairs = run_lab_backtest.pairs;
        let mut params = run_lab_backtest.params;
        params.normalize_percentages();
        let position_type = run_lab_backtest.position_type;
        params.position_type = Some(position_type.clone());

        let mut sub_type = run_lab_backtest.sub_type;
        if sub_type.is_none() {
            sub_type = params.ma_type.clone();
            if sub_type.is_none() {
                sub_type = params.mean_type.clone();
            }
        }

        let market_details = MarketDetails {
            pairs: pairs.clone(),
            timeframe: run_lab_backtest.timeframe,
            initial_capital: run_lab_backtest.initial_capital,
            position_type: position_type.clone(),
            sub_type: sub_type.clone(),
        };

        let run_history_repo = LabRunHistoryRepository::new()?;
        let run_history = LabRunHistory {
            id: None,
            template_id: template_id,
            parameters: serde_json::to_value(&params).unwrap(),
            performance: None,
            status: "running".to_string(),
            start_time: Timestamp(Utc::now()),
            end_time: None,
            created_at: None,
            market_details: serde_json::to_value(&market_details).unwrap(),
        };

        let mut lab_running_backtest = run_history_repo.create(run_history)?;

        let backtest_input = BacktestInput {
            r#type: strategy_type,
            initial_capital: run_lab_backtest.initial_capital,
            strategy_run_params: serde_json::to_value(&params).unwrap(),
        };

        let datafeed = MarketDataFeed::from_coins_market(&pairs);

        let backtest_result = match datafeed {
            Ok(ohlcv) => {
                let backtest_driver = BacktestDriver::new(backtest_input, ohlcv);
                let res = backtest_driver.build_and_run_backtest();
                lab_running_backtest.status = "success".to_string();
                Some(res)
            }
            Err(_) => {
                lab_running_backtest.status = "failed".to_string();
                run_history_repo.update(&lab_running_backtest)?;
                None
            }
        };

        let backtest_result = backtest_result.unwrap();
        let metrics = BacktestMetrics {
            win_rate: backtest_result.win_rate,
            max_drawdown: backtest_result.max_drawdown,
            sharpe_ratio: backtest_result.sharpe_ratio,
            strategy_return: backtest_result.total_return,
            total_trades: backtest_result.trades.len() as u64,
            profit_factor: backtest_result.profit_factor,
        };

        lab_running_backtest.end_time = Some(Timestamp(Utc::now()));
        lab_running_backtest.performance = Some(serde_json::to_value(&metrics).unwrap());
        run_history_repo.update(&lab_running_backtest)?;

        // summary metrics
        let run_id: i64 = lab_running_backtest.id.unwrap();
        let metrics_value = serde_json::to_value(&metrics).unwrap();
        let _ = self
            .strategy_template_repo
            .update_backtest_version_by_id(template_id, &metrics_value, run_id)
            .unwrap();

        let balances = backtest_result.balances;
        let daily_returns = calculate_daily_return_distribution(&balances, 1.0);
        let monthly_returns = calculate_monthly_returns(&balances);
        let result = RunBacktestData {
            trades: backtest_result.trades,
            balances,
            metrics,
            monthly_returns,
            return_distribution: daily_returns,
            version: lab_running_backtest.id,
            date: Some(lab_running_backtest.start_time.0.to_rfc3339()),
        };

        let run_id = lab_running_backtest.id.unwrap();
        let chart_key = format!("template-{template_id}-{run_id}");

        let chart_db = ChartDB::new().unwrap();
        chart_db.store_json(&chart_key, &result).unwrap();

        let result_string = serde_json::to_string(&result).unwrap();
        println!("run lab backtest result {}", &result_string);
        Ok(result)
    }
}

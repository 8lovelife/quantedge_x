use std::{error::Error, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::task::Id;

use crate::{
    data::{
        duckdb::{
            repository::{
                StrategyBuildsRepository, backtest_run_repository::BacktestRunHistoryRepository,
            },
            schema::{StrategyBuilds, strategy_builds::ProgressInType},
        },
        sleddb::ChartDB,
    },
    utils::params::split_params,
};

use super::backtest_service::RunBacktestData;

#[derive(Debug, Serialize, Deserialize)]
pub struct Strategy {
    pub id: i64,
    pub name: String,
    pub r#type: String, // `type` is a reserved keyword in Rust
    pub status: String,
    pub performance: Option<Value>, // Placeholder for a future performance struct
    pub description: String,
    pub configuration: StrategyConfiguration,
    pub trades: Option<Vec<Trade>>,
    pub logs: Option<Vec<LogEntry>>,
    #[serde(rename = "marketDetails")]
    pub market_details: Option<Value>,
    #[serde(rename = "latestBacktestVersion")]
    pub latest_backtest_version: Option<i64>,
    #[serde(rename = "applyBacktestVersion")]
    pub apply_backtest_version: Option<i64>,
    #[serde(rename = "backtestPerformance")]
    pub backtest_performance: Option<Value>,
    #[serde(rename = "paperPerformance")]
    pub paper_performance: Option<Value>,
    #[serde(rename = "livePerformance")]
    pub live_performance: Option<Value>,
    #[serde(rename = "created")]
    pub created_timestamp: String,
    #[serde(rename = "updated")]
    pub updated_timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyConfiguration {
    pub parameters: Option<Value>,
    pub assets: Option<Value>,
    #[serde(rename = "riskManagement")]
    pub risk_management: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
    pub id: i64,
    pub date: String,
    pub r#type: String,
    pub asset: String,
    pub price: f64,
    pub size: f64,
    pub pnl: Option<f64>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProgressStage {
    Type,
    Parameters,
    Assets,
    Risk,
}

impl ProgressStage {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "type" => Some(Self::Type),
            "parameters" => Some(Self::Parameters),
            "assets" => Some(Self::Assets),
            "risk" => Some(Self::Risk),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Parameters => "parameters",
            Self::Assets => "assets",
            Self::Risk => "risk",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Lifecycle {
    Draft,
    Backtested,
    PaperTrading,
    Live,
    Archived,
}

impl Lifecycle {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(Self::Draft),
            "backtested" => Some(Self::Backtested),
            "paper_trading" => Some(Self::PaperTrading),
            "live" => Some(Self::Live),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Backtested => "backtested",
            Self::PaperTrading => "paper_trading",
            Self::Live => "live",
            Self::Archived => "archived",
        }
    }
}

pub fn can_transition(from: Lifecycle, to: Lifecycle) -> bool {
    match from {
        Lifecycle::Draft => matches!(to, Lifecycle::Backtested | Lifecycle::Archived),
        Lifecycle::Backtested => matches!(to, Lifecycle::PaperTrading | Lifecycle::Archived),
        Lifecycle::PaperTrading => matches!(to, Lifecycle::Live | Lifecycle::Archived),
        Lifecycle::Live => matches!(to, Lifecycle::Archived),
        Lifecycle::Archived => matches!(to, Lifecycle::Draft), // optional
    }
}

pub struct StrategyBuilder {
    pub progress_stage: ProgressStage,
    pub id: Option<i64>,
    pub progress_in_type: Option<ProgressInType>,
    pub progress_in_parameters: Option<Value>,
    pub progress_in_assets: Option<Value>,
    pub progress_in_risk: Option<Value>,
}

pub struct StrategyService {
    strategy_builds: Arc<StrategyBuildsRepository>,
    backtest_run_history: Arc<BacktestRunHistoryRepository>,
    chart_db: Arc<ChartDB>,
}

impl StrategyService {
    pub fn build(user_id: i64) -> Self {
        let chart_db = ChartDB::build(user_id);
        let strategy_builds = Arc::new(StrategyBuildsRepository::build(user_id));
        let backtest_run_history = Arc::new(BacktestRunHistoryRepository::build(user_id));
        Self {
            strategy_builds,
            backtest_run_history,
            chart_db: Arc::new(chart_db),
        }
    }

    // pub fn new() -> Result<Self, Box<dyn Error>> {
    //     let chart_db = ChartDB::new()?;
    //     let strategy_builds = Arc::new(StrategyBuildsRepository::new(None)?);
    //     Ok(Self {
    //         strategy_builds: strategy_builds,
    //         chart_db: Arc::new(chart_db),
    //     })
    // }

    pub fn delete_draft_strategy(&self, id: i64) -> Result<bool, Box<dyn Error>> {
        let result = self.strategy_builds.delete_by_id(id)?;
        if result == 0 { Ok(false) } else { Ok(true) }
    }

    pub fn get_strategy_summarys_by_page(
        &self,
        page: u32,
        page_size: u32,
        status: &Option<String>,
    ) -> Result<Vec<StrategyBuilds>, Box<dyn Error>> {
        let strategy_summarys = match status.as_deref() {
            Some(status) => self
                .strategy_builds
                .get_by_status_page(page, page_size, status)?,
            None => self.strategy_builds.get_by_page(page, page_size)?,
        };
        Ok(strategy_summarys)
    }

    pub fn get_strategy_summarys_count(
        &self,
        status: &Option<String>,
    ) -> Result<u32, Box<dyn Error>> {
        let count = match status.as_deref() {
            Some(status) => self.strategy_builds.get_count_by_status(status)?,
            None => self.strategy_builds.get_count()?,
        };
        Ok(count)
    }

    pub fn build_strategy(&self, builder: StrategyBuilder) -> Result<Strategy, Box<dyn Error>> {
        let progress_stage = builder.progress_stage;
        let strategy_id = match progress_stage {
            ProgressStage::Type => {
                if let Some(id) = builder.id {
                    let next_state = self.build_next_stage(id, &progress_stage);
                    self.strategy_builds.update_type_stage(
                        id,
                        &builder.progress_in_type.unwrap(),
                        next_state,
                    )?;
                    id
                } else if let Some(r#type) = builder.progress_in_type {
                    let result = self.strategy_builds.create(r#type)?;
                    result.id.unwrap()
                } else {
                    return Err("Missing strategy ID for type stage".into());
                }
            }
            ProgressStage::Parameters => match (builder.id, builder.progress_in_parameters) {
                (Some(id), Some(params)) => {
                    let next_state = self.build_next_stage(id, &progress_stage);
                    self.strategy_builds
                        .update_parameters(id, &params, next_state)?;
                    id
                }
                _ => return Err("Missing strategy ID or Parameters for Parameters stage".into()),
            },
            ProgressStage::Assets => match (builder.id, builder.progress_in_assets) {
                (Some(id), Some(assets)) => {
                    let next_state: &str = self.build_next_stage(id, &progress_stage);
                    self.strategy_builds
                        .update_assets(id, &assets, next_state)?;
                    id
                }
                _ => return Err("Missing strategy ID or Assets for Assets stage".into()),
            },
            ProgressStage::Risk => match (builder.id, builder.progress_in_risk) {
                (Some(id), Some(risk)) => {
                    let next_state: &str = self.build_next_stage(id, &progress_stage);
                    self.strategy_builds.update_risk(id, &risk, next_state)?;
                    id
                }
                _ => return Err("Missing strategy ID or Risk for Risk stage".into()),
            },
        };

        let details = self.get_strategy_details(strategy_id)?;

        Ok(details)
    }

    pub fn build_next_stage(&self, id: i64, next_stage: &ProgressStage) -> &'static str {
        let (progress, _lifecycle) = self.strategy_builds.get_status(id).unwrap();
        let progress = ProgressStage::from_str(progress.as_str()).unwrap();

        if *next_stage <= progress {
            return progress.as_str();
        } else if next_stage == &ProgressStage::Risk {
            return "completed";
        } else {
            return next_stage.as_str();
        }
    }

    pub fn get_strategy_details(&self, strategy_id: i64) -> Result<Strategy, Box<dyn Error>> {
        let strategy_details = self
            .strategy_builds
            .get_by_id(strategy_id)
            .expect(format!("trade strategy with id {} db error", strategy_id).as_str())
            .expect(format!("trade strategy with id {} is not exist", strategy_id).as_str());

        let mut strategy = Strategy {
            id: strategy_id,
            name: strategy_details.name,
            description: strategy_details.description,
            r#type: strategy_details.algorithm_type,
            status: strategy_details.lifecycle,
            performance: None,
            configuration: StrategyConfiguration {
                parameters: strategy_details.parameters,
                assets: strategy_details.assets,
                risk_management: strategy_details.risk,
            },
            latest_backtest_version: strategy_details.latest_backtest_version,
            backtest_performance: strategy_details.backtest_performance,
            paper_performance: strategy_details.paper_performance,
            live_performance: strategy_details.live_performance,
            trades: None,
            logs: None,
            created_timestamp: strategy_details.created_at.0.to_rfc3339(),
            updated_timestamp: strategy_details.updated_at.0.to_rfc3339(),
            market_details: strategy_details.market_details,
            apply_backtest_version: strategy_details.apply_backtest_version,
        };

        // if matches!(
        //     strategy.status.as_str(),
        //     "backtested" | "paper_trading" | "live"
        // ) {
        //     // backtest result
        //     let latest_run_id = strategy_details.latest_backtest_version.unwrap();
        //     let chart_key = format!("{strategy_id}-{latest_run_id}");
        //     let chart_json: Option<RunBacktestData> = self.chart_db.retrieve(&chart_key)?;
        //     let backtest_data = chart_json.unwrap();
        //     let backtest_result = backtest_data.metrics;
        //     let value: Value = serde_json::to_value(&backtest_result).unwrap();
        //     strategy.performance = Some(value);
        // }
        Ok((strategy))
    }

    pub fn apply_strategy_run(&self, id: i64, version: i64) -> Result<bool, Box<dyn Error>> {
        let strategy_build = self.strategy_builds.get_by_id(id).unwrap().unwrap();
        let algorithm = strategy_build.algorithm_type;
        let version_run_details = self
            .backtest_run_history
            .get_by_id(version)
            .unwrap()
            .unwrap();
        let market_details = version_run_details.market_details;
        let all_parameters = version_run_details.parameters;

        let split_parameters = split_params(&all_parameters, algorithm.as_str());

        let result = self.strategy_builds.update_apply_backtest_by_id(
            id,
            version,
            &split_parameters.strategy,
            &split_parameters.risk,
            &version_run_details.performance.unwrap(),
            &market_details,
        )?;
        if result == 0 { Ok(false) } else { Ok(true) }
    }
}

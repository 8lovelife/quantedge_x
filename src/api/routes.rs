use crate::{
    data::duckdb::repository::{
        LabRunHistoryRepository, OhlcvRepository, StrategyTemplateRepository, TradeRepository,
        TradeStrategyRepository,
        connection::{UserConnectionManager, get_user_connection_manager},
    },
    service::{backtest_service::BacktestService, user_auth_service::UserAuthService},
    ws::push_stream::{BroadcastMap, handle_web_socket},
};
use axum::{
    Extension, Router,
    routing::{delete, get, patch, post, put},
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use super::handlers::{
    add_trade_strategy, appy_strategy_run, backtest_history_data, backtest_run_history,
    build_strategy, delete_draft_strategie_by_id, get_current_user, get_price, get_recent_trades,
    get_strategy_details, get_strategy_summarys, get_strategy_template_by_id,
    get_strategy_templates, lab_run_comparison_data, lab_run_history_backtest_data,
    lab_run_history_data, ping, revoke_current_user, run_lab_backtest, run_strategy_backtest,
    strategy_run_comparison_data, update_strategy, update_strategy_status, user_auth, user_login,
    user_register,
};

#[derive(Clone)]
pub struct AppState {
    pub user_connection_manager: Arc<UserConnectionManager>,
    pub ohlcv_repo: Arc<OhlcvRepository>,
    pub trade_repo: Arc<TradeRepository>,
    pub trade_strategy_repo: Arc<TradeStrategyRepository>,
    // pub backtest_run_history_repo: Arc<BacktestRunHistoryRepository>,
    pub lab_run_history_repo: Arc<LabRunHistoryRepository>,
    pub strategy_template_repo: Arc<StrategyTemplateRepository>,
    pub backtest_service: Arc<BacktestService>,
    // pub build_strategy_service: Arc<StrategyService>,
    pub user_auth_service: Arc<UserAuthService>,
}

pub fn create_router() -> Router {
    let user_connection_manager = get_user_connection_manager();
    let ohlcv_repo = Arc::new(OhlcvRepository::new(None).unwrap());
    let trade_repo = Arc::new(TradeRepository::new(None).unwrap());
    let trade_strategy_repo = Arc::new(TradeStrategyRepository::new(None).unwrap());
    // let backtest_run_history_repo = Arc::new(BacktestRunHistoryRepository::new().unwrap());
    let backtest_service = Arc::new(BacktestService::new().unwrap());
    // let build_strategy_service = Arc::new(StrategyService::new().unwrap());
    let lab_run_history_repo = Arc::new(LabRunHistoryRepository::new().unwrap());
    let strategy_template_repo = Arc::new(StrategyTemplateRepository::new().unwrap());
    let user_auth_service = Arc::new(UserAuthService::new());
    let state = AppState {
        user_connection_manager,
        ohlcv_repo,
        trade_repo,
        trade_strategy_repo,
        // backtest_run_history_repo,
        strategy_template_repo,
        lab_run_history_repo,
        backtest_service,
        // build_strategy_service,
        user_auth_service,
    };

    let broadcast_map: BroadcastMap = Arc::new(RwLock::new(HashMap::new()));

    Router::new()
        .route("/ws", get(handle_web_socket))
        .layer(Extension(broadcast_map.clone()))
        .route("/api/user/auth", post(user_auth))
        .route("/api/ping", get(ping))
        .route("/api/login", get(get_current_user))
        .route("/api/login", post(user_login))
        .route("/api/register", post(user_register))
        .route("/api/logout", delete(revoke_current_user))
        .route("/api/price", get(get_price))
        .route("/api/trades/recent", get(get_recent_trades))
        .route("/api/strategies", post(add_trade_strategy))
        .route("/api/strategies/run", post(run_strategy_backtest))
        .route("/api/strategies/run/:id", put(appy_strategy_run))
        .route("/api/strategies/run/history", get(backtest_run_history))
        .route("/api/strategies/run/backtest", get(backtest_history_data))
        .route(
            "/api/strategies/run/comparison",
            get(strategy_run_comparison_data),
        )
        .route("/api/strategies", get(get_strategy_summarys))
        .route("/api/strategies/:id", delete(delete_draft_strategie_by_id))
        .route("/api/strategies/:id", put(update_strategy))
        .route("/api/strategies/details", get(get_strategy_details))
        .route("/api/algorithms", get(get_strategy_templates))
        .route("/api/algorithms/:id", get(get_strategy_template_by_id))
        .route("/api/lab", get(get_strategy_templates))
        .route("/api/lab/:id", get(get_strategy_template_by_id))
        .route("/api/lab/run", post(run_lab_backtest))
        .route("/api/lab/run/history", get(lab_run_history_data))
        .route("/api/lab/run/backtest", get(lab_run_history_backtest_data))
        .route("/api/lab/run/comparison", get(lab_run_comparison_data))
        // .route("/api/lab/run/optimize", method_router)
        .route("/api/strategies/:id/status", patch(update_strategy_status))
        // .route("/api/backtest/history", get(backtest_run_history))
        .route("/api/strategies/draft/:stage", post(build_strategy))
        .with_state(state)
}

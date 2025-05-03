pub mod initialize_db;
pub mod repository;
pub mod schema;
pub mod types;

use rand::rngs::mock;
pub use repository::{OhlcvRepository, TradeRepository};
pub use schema::{Ohlcv, Trade};

#[cfg(test)]
mod tests {
    use crate::data::duckdb::{
        repository::{BacktestRunRepository, StrategyTemplateRepository},
        schema::{
            StrategyTemplate, TradeStrategy, backtest_run_history::BacktestRun,
            strategy_template::StrategyParameterConfig,
        },
        types::Timestamp,
    };

    use super::{
        repository::{StrategyBuildsRepository, TradeStrategyRepository, trade_strategy},
        schema::{StrategyBuilds, strategy_builds::ProgressInType},
        *,
    };
    use chrono::{DateTime, Utc};
    use serde_json::json;
    use uuid::Uuid;

    fn setup_test_db() -> (OhlcvRepository, TradeRepository, TradeStrategyRepository) {
        let ohlcv_repo = OhlcvRepository::new(None).unwrap();
        let trade_repo = TradeRepository::new(None).unwrap();
        let trade_strategy_repo = TradeStrategyRepository::new(None).unwrap();
        (ohlcv_repo, trade_repo, trade_strategy_repo)
    }

    #[test]
    fn test_ohlcv_repository() {
        let (ohlcv_repo, _, _) = setup_test_db();

        // Test getting data for different symbols
        let btc_data = ohlcv_repo.get_by_symbol("BTC").unwrap();
        let eth_data = ohlcv_repo.get_by_symbol("ETH").unwrap();
        let sol_data = ohlcv_repo.get_by_symbol("SOL").unwrap();

        // Verify we have data for each symbol
        assert!(!btc_data.is_empty(), "BTC data should not be empty");
        assert!(!eth_data.is_empty(), "ETH data should not be empty");
        assert!(!sol_data.is_empty(), "SOL data should not be empty");

        // Test getting data by symbol and timestamp
        if let Some(btc_record) = btc_data.first() {
            let retrieved = ohlcv_repo
                .get_by_symbol_and_timestamp(&btc_record.symbol, btc_record.timestamp)
                .unwrap()
                .unwrap();
            assert_eq!(retrieved.id, btc_record.id);
            assert_eq!(retrieved.symbol, btc_record.symbol);
            assert_eq!(retrieved.timestamp, btc_record.timestamp);
        }
    }

    #[test]
    fn test_trade_repository() {
        let (_, trade_repo, _) = setup_test_db();

        // Create a test trade
        let trade = Trade {
            id: "test-trade-1".to_string(),
            strategy: "test-strategy".to_string(),
            trade_type: "BUY".to_string(),
            asset: "BTC".to_string(),
            amount: 1.5,
            price: 42000.0,
            timestamp: DateTime::parse_from_rfc3339("2024-03-13T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            status: "OPEN".to_string(),
            profit: None,
        };

        // Test create
        trade_repo.create(&trade).unwrap();

        // Test get_by_id
        let retrieved = trade_repo.get_by_id(&trade.id).unwrap().unwrap();
        assert_eq!(retrieved.id, trade.id);
        assert_eq!(retrieved.strategy, trade.strategy);
        assert_eq!(retrieved.trade_type, trade.trade_type);
        assert_eq!(retrieved.asset, trade.asset);
        assert_eq!(retrieved.amount, trade.amount);
        assert_eq!(retrieved.price, trade.price);
        assert_eq!(retrieved.timestamp, trade.timestamp);
        assert_eq!(retrieved.status, trade.status);
        assert_eq!(retrieved.profit, trade.profit);

        // Test update
        let mut updated_trade = trade.clone();
        updated_trade.status = "CLOSED".to_string();
        updated_trade.profit = Some(150.0);
        trade_repo.update(&updated_trade).unwrap();

        let retrieved_updated = trade_repo.get_by_id(&trade.id).unwrap().unwrap();
        assert_eq!(retrieved_updated.status, "CLOSED");
        assert_eq!(retrieved_updated.profit, Some(150.0));

        // Test delete
        trade_repo.delete(&trade.id).unwrap();
        let deleted = trade_repo.get_by_id(&trade.id).unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_update_strategy_latest_version() {
        let (_, _, trade_strategy_repo) = setup_test_db();
        let _ = trade_strategy_repo.update_backtest_version_by_id(1, 33);
    }

    #[test]
    fn test_trade_strategy_repository() {
        let (_, _, trade_strategy_repo) = setup_test_db();

        // Create a test trade strategy
        let trade_strategy = TradeStrategy {
            id: None,
            name: "MATEST".to_string(),
            description: format!("Description for {}", "MATEST"),
            algorithm: "moving_average".to_string(),
            risk: "medium".to_string(),
            allocation: 10,
            timeframe: "1d".to_string(),
            assets: "BTC".to_string(),
            status: "active".to_string(),
            parameters: r#"{"smaFast": "10", "smaSlow": "50"}"#.to_string(),
            latest_backtest_version: None,
        };

        // Test create
        let new_trade_strategy = trade_strategy_repo.create(trade_strategy).unwrap();
        println!("{:?}", &new_trade_strategy);

        // Test get_by_id
        let retrieved = trade_strategy_repo
            .get_by_id(new_trade_strategy.id.unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.id, new_trade_strategy.id);
        assert_eq!(retrieved.name, new_trade_strategy.name);
        assert_eq!(retrieved.description, new_trade_strategy.description);
        assert_eq!(retrieved.algorithm, new_trade_strategy.algorithm);
        assert_eq!(retrieved.risk, new_trade_strategy.risk);
        assert_eq!(retrieved.allocation, new_trade_strategy.allocation);
        assert_eq!(retrieved.timeframe, new_trade_strategy.timeframe);
        assert_eq!(retrieved.assets, new_trade_strategy.assets);
        assert_eq!(retrieved.status, new_trade_strategy.status);
        assert_eq!(retrieved.parameters, new_trade_strategy.parameters);
    }

    #[test]
    fn test_backtest_repository() {
        // Create a new BacktestRunRepository instance
        let backtest_repo = BacktestRunRepository::new().unwrap();

        // Create a new BacktestRun instance with sample data
        let new_run = BacktestRun {
            id: None,
            strategy_id: 123,
            parameters: r#"{"param":"value"}"#.to_string(),
            status: "running".to_string(),
            start_time: Timestamp(Utc::now()),
            end_time: None,
            created_at: None,
        };

        println!("show UTC {:?}", serde_json::to_string(&new_run));

        // Test creation: insert the run and verify it gets an ID
        let created_run = backtest_repo.create(new_run).unwrap();
        assert!(created_run.id.is_some(), "Created run should have an ID");

        // Test pagination: list paginated results and ensure the created run is present
        let paginated_runs = backtest_repo.get_by_page(10, 0).unwrap();

        println!("BackTestRun {:?}", &paginated_runs);
        assert!(
            !paginated_runs.is_empty(),
            "Paginated list should not be empty"
        );
        assert!(
            paginated_runs.iter().any(|run| run.id == created_run.id),
            "Created run should be in paginated results"
        );

        // Test get_by_id: retrieve the run and verify its strategy_id
        let retrieved = backtest_repo
            .get_by_id(created_run.id.unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.strategy_id, 123);

        // Test deletion: delete the run and verify it's removed
        backtest_repo.delete(created_run.id.unwrap()).unwrap();
        let deleted = backtest_repo.get_by_id(created_run.id.unwrap()).unwrap();
        assert!(deleted.is_none(), "Run should be deleted");
    }

    #[test]
    fn test_top_backtest_repository() {
        let backtest_repo = BacktestRunRepository::new().unwrap();
        let top_5 = backtest_repo.get_latest_top_run(123, 5).unwrap();
        println!("{:?}", &top_5);
    }

    #[test]
    fn test_insert_and_read_strategy() {
        let strategy_repo = StrategyBuildsRepository::new(None).unwrap();
        let name = "My Strategy";
        let description = "Test description";
        let algo_type = "mean-reversion";
        let parameters = json!({ "fastPeriod": 5, "slowPeriod": 20 }).to_string();
        let risk = json!({ "maxDrawdown": 10 }).to_string();

        let algorithm_type = ProgressInType {
            id: None,
            name: "My Strategy".to_string(),
            description: "Test description".to_string(),
            algorithm_type: "mean-reversion".to_string(),
        };
        let result = strategy_repo.create(algorithm_type);
        println!("{:?}", result);
    }

    #[test]
    fn test_update_parameter() {
        let strategy_repo = StrategyBuildsRepository::new(None).unwrap();
        let name = "My Strategy";
        let description = "Test description";
        let algo_type = "mean-reversion";
        let parameters = json!({ "fastPeriod": 15, "slowPeriod": 20 });
        let risk = json!({ "maxDrawdown": 10 }).to_string();

        let algorithm_type = ProgressInType {
            id: None,
            name: "My Strategy".to_string(),
            description: "Test description".to_string(),
            algorithm_type: "mean-reversion".to_string(),
        };
        let result = strategy_repo.update_parameters(2, &parameters, "test");
        let result = strategy_repo.get_by_id(2);
        println!("{:?}", result);
    }

    #[test]
    fn test_update_assets() {
        let strategy_repo = StrategyBuildsRepository::new(None).unwrap();
        let name = "My Strategy";
        let description = "Test description";
        let algo_type = "mean-reversion";
        let parameters = json!({ "fastPeriod": 5, "slowPeriod": 20 });
        let assets = json!( [{"symbol":"BTC","weight":60},{"symbol":"ETH","weight":40}] );
        let risk = json!({ "maxDrawdown": 10 });

        let algorithm_type = ProgressInType {
            id: None,
            name: "My Strategy".to_string(),
            description: "Test description".to_string(),
            algorithm_type: "mean-reversion".to_string(),
        };
        let result = strategy_repo.update_assets(2, &assets, "test");
        let result = strategy_repo.get_by_id(2);
        println!("{:?}", result);
    }

    #[test]
    fn test_update_risk() {
        let strategy_repo = StrategyBuildsRepository::new(None).unwrap();
        let name = "My Strategy";
        let description = "Test description";
        let algo_type = "mean-reversion";
        let parameters = json!({ "fastPeriod": 5, "slowPeriod": 20 });
        let assets = json!( [{"symbol":"BTC","weight":60},{"symbol":"ETH","weight":40}] );
        let risk = json!({ "maxDrawdown": 10 });

        let algorithm_type = ProgressInType {
            id: None,
            name: "My Strategy".to_string(),
            description: "Test description".to_string(),
            algorithm_type: "mean-reversion".to_string(),
        };
        // let result = strategy_repo.update_risk(2, &risk);
        let result = strategy_repo.get_by_id(2).unwrap().unwrap();
        println!("{:?}", serde_json::to_string(&result));
    }

    #[test]
    fn test_get_template_by_id() {
        use crate::data::duckdb::repository::StrategyTemplateRepository;

        let repo = StrategyTemplateRepository::new().expect("failed to init repo");

        let fetched = repo
            .get_template_by_id(1)
            .expect("failed to fetch")
            .expect("not found");

        println!("{:?}", fetched);
    }

    #[test]
    fn test_create_strategy_templates() {
        use chrono::Utc;
        use serde_json::json;

        let repository = StrategyTemplateRepository::new().expect("failed to create repo");

        let now = Some(Utc::now().naive_utc());

        let ma_crossover_template = StrategyTemplate {
            id: None,
            name: "MA Crossover".to_string(),
            r#type: "ma-crossover".to_string(),
            description: "A trend-following strategy that generates buy/sell signals based on the crossover of fast and slow moving averages.".to_string(),
            info: "The Moving Average (MA) Crossover strategy enters a long position when the fast MA (e.g., 10-day) crosses above the slow MA (e.g., 30-day), and exits or goes short when the fast MA crosses below the slow MA. This approach helps capture trend reversals and is often used in both equity and crypto markets. Variants such as SMA, EMA, or WMA are supported to accommodate different smoothing preferences.".to_string(),
            parameters: Some(StrategyParameterConfig {
                options: Some(json!({
                    "meanType": ["sma", "ema", "wma"],
                })),
                default: Some(json!({
                    "meanType": "sma",
                    "fastPeriod": 10,
                    "slowPeriod": 30,
                    "rebalanceInterval": "daily",
                    "entryThreshold": 1.0,
                    "exitThreshold": 0.5
                }))
            }),
            risk: Some(json!({
                "stopLoss": 5.0,
                "takeProfit": 10.0,
                "riskPerTrade": 2.0,
                "positionSize": 30.0,
                "maxConcurrentPositions": 1
            })),
            execution: Some(json!({
                "slippage": 0.1,
                "commission": 0.05,
                "entryDelay": 1,
                "minHoldingPeriod": 3,
                "maxHoldingPeriod": 10
            })),
            performance: None,
            likes: Some(0),
            usage: Some(0),
            author: Some("test".to_string()),
            latest_lab_backtest_version: Some(0),
            created_at: None,
            updated_at: None,
        };

        let mean_reversion_template = StrategyTemplate {
            id: None,
            name: "Mean Reversion".to_string(),
            r#type: "mean-reversion".to_string(),
            description: "A reversion-based strategy that assumes prices tend to revert to their historical mean over time.".to_string(),
            info: "The Mean Reversion strategy identifies trading opportunities when asset prices deviate significantly from their historical average. It enters a long position when the price drops far below the mean, and a short position when it rises well above it...".to_string(),
            parameters: Some(StrategyParameterConfig {
                options: Some(json!({
                    "meanType": ["sma", "ema"],
                    "reversionStyle": ["z-score", "bollinger"],
                })),
                default: Some(json!({
                    "lookbackPeriod": 20,
                    "entryZScore": 2.0,
                    "exitZScore": 0.5,
                    "meanType": "ema",
                    "reversionStyle": "z-score",
                    "positionType": "both",
                    "rebalanceInterval": "daily",
                    "cooldownPeriod": 5,
                    "bandMultiplier":2
                }))
            }),
            risk: Some(json!({
                "stopLoss": 5.0,
                "takeProfit": 10.0,
                "riskPerTrade": 2.0,
                "positionSize": 30.0,
                "maxConcurrentPositions": 1
            })),
            execution: Some(json!({
                "slippage": 0.1,
                "commission": 0.05,
                "entryDelay": 1,
                "minHoldingPeriod": 3,
                "maxHoldingPeriod": 10
            })),
            performance: None,
            likes: Some(0),
            usage: Some(0),
            author: Some("system".to_string()),
            latest_lab_backtest_version: Some(0),
            created_at: None,
            updated_at: None,
        };

        // Insert MA Crossover
        let created_ma = repository
            .create_template(ma_crossover_template)
            .expect("failed to insert MA Crossover");

        assert!(created_ma.id.is_some());
        assert_eq!(created_ma.r#type, "ma-crossover");

        // Insert Mean Reversion
        let created_mr = repository
            .create_template(mean_reversion_template)
            .expect("failed to insert Mean Reversion");

        assert!(created_mr.id.is_some());
        assert_eq!(created_mr.r#type, "mean-reversion");
    }

    #[test]
    fn test_get_templates_by_page() {
        let repo = StrategyTemplateRepository::new().expect("failed to create repository");

        let page = 1;
        let page_size = 2;
        let results = repo
            .get_templates_by_page(page, page_size)
            .expect("pagination failed");

        let page2 = repo
            .get_templates_by_page(2, 2)
            .expect("second page failed");
        assert!(page2.len() <= 2);

        if results.len() == 2 && page2.len() == 2 {
            assert_ne!(results[0].id, page2[0].id);
        }
    }
}

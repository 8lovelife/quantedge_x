use std::sync::{Arc, Mutex, RwLock};

use duckdb::Connection;

pub fn initialize_schema(conn: Arc<Mutex<Connection>>) {
    let sql = r#"
    CREATE SEQUENCE IF NOT EXISTS strategy_builds_seq START 1;

    CREATE TABLE IF NOT EXISTS strategy_builds (
        id BIGINT PRIMARY KEY DEFAULT nextval('strategy_builds_seq'),
        name VARCHAR NOT NULL,
        description TEXT NOT NULL,
        algorithm_type VARCHAR,
        algorithm_sub_type VARCHAR,
        assets JSON,
        parameters JSON,
        risk JSON,
        progress VARCHAR CHECK(progress IN ('type', 'parameters', 'assets', 'risk', 'completed')) DEFAULT 'type',
        lifecycle VARCHAR CHECK(lifecycle IN ('draft', 'backtest', 'paper', 'live','archived')) DEFAULT 'draft',
        backtest_performance JSON,
        paper_performance JSON,
        live_performance JSON,
        market_details JSON,
        apply_backtest_version BIGINT DEFAULT 0,
        latest_backtest_version BIGINT DEFAULT 0,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        UNIQUE(name)
    );

    CREATE SEQUENCE IF NOT EXISTS strategy_run_backtest_seq START 1;
    CREATE TABLE IF NOT EXISTS strategy_backtest_run_history (
        id BIGINT PRIMARY KEY DEFAULT nextval('strategy_run_backtest_seq'),
        strategy_id BIGINT NOT NULL,
        parameters JSON NOT NULL,
        performance JSON,
        market_details JSON,
        status     VARCHAR CHECK (status IN ('pending', 'running', 'success', 'failed')) NOT NULL,
        start_time TIMESTAMP NOT NULL,               
        end_time   TIMESTAMP, 
        created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    );    
    "#;

    conn.lock()
        .unwrap()
        .execute_batch(sql)
        .expect("Failed to initialize schema");
}
